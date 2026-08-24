// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! ClientV2 transport, connection settings, and shared RPC behavior.
//!
//! Most applications start with [`ClientV2::new`] and [`ConnectConfig`], then call a feature
//! method with a validated request from [`crate::v2::request`]. Each feature area is implemented
//! in a private submodule of this client module and re-exported through [`crate::v2`].
//!
//! A typical operation looks like this:
//!
//! ```no_run
//! # use milvus::v2::prelude::*;
//! # async fn example() -> Result<()> {
//! let client = ClientV2::new(
//!     &ConnectConfig::new()
//!         .uri("http://localhost:19530")
//!         .token("root:Milvus"),
//! )
//! .await?;
//! let request = CheckHealthRequest::builder().build()?;
//! let response = client.check_health(request).await?;
//! println!("healthy: {}", response.is_healthy());
//! # Ok(())
//! # }
//! ```
//!
//! RPC deadlines apply to individual attempts. Retry behavior is centralized, and mutation
//! requests use non-idempotent semantics when replaying an ambiguous transport failure could
//! duplicate a server-side operation.

use crate::proto::milvus::client_telemetry_service_client::ClientTelemetryServiceClient;
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::{common, milvus};
use crate::v2::error::{Error, Result};
use crate::v2::types::{is_global_endpoint, ConnectConfig, RetryConfig};
use parking_lot::RwLock;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tonic::codegen::InterceptedService;
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};
use tonic::{Code, Request, Response, Status};

use cache::SchemaLoadScope;

macro_rules! trace_debug {
    ($($field:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::debug!($($field)*);
    };
}

macro_rules! trace_info {
    ($($field:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::info!($($field)*);
    };
}

macro_rules! trace_warn {
    ($($field:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::warn!($($field)*);
    };
}

macro_rules! rpc_with_retry {
    ($semantics:ident, $client:expr, $method:ident, $request:expr) => {{
        let request = $request;
        $client
            .retry_rpc(
                || Ok(request.clone()),
                $crate::v2::client::RetrySemantics::$semantics,
                |mut service, request| async move { service.$method(request).await },
                |response| response.status.clone(),
            )
            .await
    }};
    ($client:expr, $method:ident, $request:expr) => {{
        rpc_with_retry!(Idempotent, $client, $method, $request)
    }};
}

macro_rules! status_rpc_with_retry {
    (Idempotent, $client:expr, $method:ident, $request:expr) => {{
        let request = $request;
        $client
            .retry_rpc(
                || Ok(request.clone()),
                $crate::v2::client::RetrySemantics::Idempotent,
                |mut service, request| async move { service.$method(request).await },
                |status| Some(status.clone()),
            )
            .await
    }};
    (NonIdempotent, $client:expr, $method:ident, $request:expr) => {{
        let request = $request;
        $client
            .retry_rpc(
                || Ok(request.clone()),
                $crate::v2::client::RetrySemantics::NonIdempotent,
                |mut service, request| async move { service.$method(request).await },
                |status| Some(status.clone()),
            )
            .await
    }};
}

mod alias;
mod cache;
mod cdc;
mod collection;
mod database;
mod dml;
mod dql;
mod global_cluster;
mod index;
mod internal;
mod iterator;
mod partition;
mod rbac;
mod resource_group;
mod session;
mod snapshot;
mod telemetry;
mod utility;

pub use iterator::{QueryIterator, SearchIterator, SearchIteratorV1, SearchIteratorV2};
pub use session::MilvusClientV2Session;
pub use telemetry::{
    new_client_request_id, with_client_request_id, ClientTelemetry, ClientTelemetryCommand,
    ClientTelemetryCommandReply, TelemetryErrorInfo, TelemetryMetrics, TelemetryOperationMetrics,
    TelemetrySnapshot,
};
pub use utility::OptimizeTask;

type Service = MilvusServiceClient<InterceptedService<Channel, V2Interceptor>>;
type TelemetryService = ClientTelemetryServiceClient<InterceptedService<Channel, V2Interceptor>>;
pub(super) type TransportGeneration = u64;

#[derive(Clone)]
pub(super) struct ServiceBundle {
    milvus: Service,
    telemetry: TelemetryService,
    generation: TransportGeneration,
}

pub(super) type SharedServices = Arc<RwLock<ServiceBundle>>;

fn service_bundle(
    channel: Channel,
    interceptor: V2Interceptor,
    generation: TransportGeneration,
) -> ServiceBundle {
    ServiceBundle {
        milvus: MilvusServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
        telemetry: ClientTelemetryServiceClient::with_interceptor(channel, interceptor),
        generation,
    }
}

fn normalize_database(database: String) -> Result<String> {
    let database = if database.is_empty() {
        "default".to_owned()
    } else {
        database
    };
    database
        .parse::<MetadataValue<tonic::metadata::Ascii>>()
        .map_err(|_| {
            Error::validation(
                "database".into(),
                "database name is not valid gRPC metadata".into(),
            )
        })?;
    Ok(database)
}

///////////////////////////////////////////////////////////////////////////////
// RetrySemantics
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetrySemantics {
    Idempotent,
    NonIdempotent,
}

///////////////////////////////////////////////////////////////////////////////
// V2Interceptor
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
struct V2Interceptor {
    token: Option<MetadataValue<tonic::metadata::Ascii>>,
    database: Arc<RwLock<String>>,
    database_explicit: Arc<AtomicBool>,
}

impl Interceptor for V2Interceptor {
    fn call(
        &mut self,
        mut request: Request<()>,
    ) -> std::result::Result<Request<()>, tonic::Status> {
        if let Some(token) = &self.token {
            request
                .metadata_mut()
                .insert("authorization", token.clone());
        }
        let database = self.database.read();
        if !database.is_empty()
            && (database.as_str() != "default"
                || self
                    .database_explicit
                    .load(std::sync::atomic::Ordering::Acquire))
        {
            let value = database.parse().map_err(|_| {
                tonic::Status::invalid_argument("database name is not valid metadata")
            })?;
            request.metadata_mut().insert("dbname", value);
        }
        let request_millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_millis().to_string())
            .unwrap_or_else(|_| "0".to_owned());
        if let Ok(value) = request_millis.parse() {
            request
                .metadata_mut()
                .insert("client-request-unixmsec", value);
        }
        // Only validated lowercase non-zero OTel TraceIDs reach this point. Malformed
        // caller values are omitted because the server would ignore them as trace IDs.
        if let Some(request_id) = telemetry::current_client_request_id() {
            if !request_id.is_empty() {
                if let Ok(value) = request_id.parse() {
                    request.metadata_mut().insert("client_request_id", value);
                }
            }
        }
        Ok(request)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ClientV2
///////////////////////////////////////////////////////////////////////////////
/// Asynchronous request/response-based client for Milvus.
///
/// Clones share the connection, selected database, and RPC settings. Collection
/// schemas and last-DML timestamps are cached process-wide by endpoint,
/// database, and collection, so independently connected clients for the same
/// endpoint share completed cache entries without colliding with other clusters.
/// In-flight schema loads are isolated between independently connected clients.
///
/// # Thread safety
///
/// `ClientV2` is safe to share across threads and asynchronous tasks. Cloning a client is cheap;
/// clones use the same connection, selected database, RPC settings, and per-client schema-load
/// scope. After [`ClientV2::new`] succeeds, RPC methods may be called concurrently, subject to
/// these restrictions:
///
/// - Concurrent DML and DQL calls are supported. Request values are owned by each call, and Rust's
///   normal synchronization rules apply to iterator objects or other mutable user-owned state.
///   Visibility between concurrent operations follows the requested consistency level; await a
///   DML call before requiring a later DQL call to observe it.
/// - DDL operations that change a database, collection identity, schema, or alias should be
///   serialized with DML and DQL calls targeting the affected objects. Results are not guaranteed
///   when those operations overlap.
/// - [`ClientV2::use_database`], [`ClientV2::set_rpc_deadline`], and
///   [`ClientV2::set_retry_param`] are internally synchronized but update state shared by every
///   clone. Serialize configuration changes with RPC creation when deterministic request settings
///   are required.
#[derive(Clone)]
pub struct ClientV2 {
    service: SharedServices,
    database: Arc<RwLock<String>>,
    database_explicit: Arc<AtomicBool>,
    rpc_timeout: Arc<RwLock<Duration>>,
    retry: Arc<RwLock<RetryConfig>>,
    cache_endpoint: Arc<String>,
    schema_load_scope: Arc<SchemaLoadScope>,
    global_cluster: Option<Arc<global_cluster::GlobalCluster>>,
    telemetry: ClientTelemetry,
}

impl ClientV2 {
    /// Creates a ClientV2 and connects it using the supplied configuration.
    pub async fn new(config: &ConnectConfig) -> Result<Self> {
        Self::connect(config.clone()).await
    }

    async fn connect(param: ConnectConfig) -> Result<Self> {
        let database = Arc::new(RwLock::new(normalize_database(param.database.clone())?));
        let database_explicit = Arc::new(AtomicBool::new(!param.database.is_empty()));
        let token = param
            .token
            .as_deref()
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| {
                Error::validation("token".into(), "token is not valid HTTP metadata".into())
            })?;

        validate_client_identity(&param)?;

        let (service, database, cache_endpoint, global_cluster) = if is_global_endpoint(&param.uri)
        {
            let topology = global_cluster::fetch_topology(&param.uri, &param).await?;
            let primary_endpoint = topology.primary()?.endpoint().to_owned();
            let primary_uri = global_cluster::cluster_endpoint_uri(&primary_endpoint, &param)?;
            let mut services = global_cluster::build_services(
                &primary_uri,
                &param,
                &database,
                &database_explicit,
                0,
            )
            .await?;
            wait_for_server(&mut services.milvus, param.connect_timeout).await?;
            let service = Arc::new(RwLock::new(services));
            // The cache endpoint is the global URI itself, resolved with the same
            // scheme-upgrade logic as the plain connection path (an explicit http:// global
            // URI is upgraded to https:// when TLS is enabled) rather than the member-endpoint
            // validator used for the primary endpoint above.
            let cache_endpoint = Arc::new(tls_endpoint_uri(&param.uri, tls_enabled(&param))?);
            let global = Arc::new(global_cluster::GlobalCluster::new(
                param.uri.clone(),
                param.clone(),
                Arc::clone(&database),
                Arc::clone(&database_explicit),
                Arc::clone(&service),
                topology,
            ));
            global.start_refresh();
            (service, database, cache_endpoint, Some(global))
        } else {
            let (endpoint, effective_endpoint_uri) = configured_endpoint(&param).await?;
            let channel = endpoint.connect_lazy();
            let interceptor = V2Interceptor {
                token,
                database: Arc::clone(&database),
                database_explicit: Arc::clone(&database_explicit),
            };
            let mut services = service_bundle(channel, interceptor, 0);
            wait_for_server(&mut services.milvus, param.connect_timeout).await?;
            (
                Arc::new(RwLock::new(services)),
                database,
                Arc::new(effective_endpoint_uri),
                None,
            )
        };

        let telemetry = ClientTelemetry::new(
            param.telemetry.clone(),
            Arc::clone(&service),
            Arc::clone(&database),
            Arc::clone(&database_explicit),
            &param,
        );
        telemetry.start();

        Ok(Self {
            service,
            database,
            database_explicit,
            rpc_timeout: Arc::new(RwLock::new(param.rpc_timeout)),
            retry: Arc::new(RwLock::new(param.retry)),
            cache_endpoint,
            schema_load_scope: Arc::new(SchemaLoadScope::new()),
            global_cluster,
            telemetry,
        })
    }

    /// Sets the timeout applied independently to each RPC attempt.
    pub fn set_rpc_deadline(&self, timeout: Duration) {
        *self.rpc_timeout.write() = timeout;
    }

    /// Replaces the retry policy used by subsequent RPC calls.
    pub fn set_retry_param(&self, retry: RetryConfig) {
        *self.retry.write() = retry;
    }

    /// Returns the shared client-side telemetry manager.
    pub fn telemetry(&self) -> ClientTelemetry {
        self.telemetry.clone()
    }

    /// Creates a cluster-scoped session view bound to the given cluster identifier.
    ///
    /// The returned [`MilvusClientV2Session`] exposes the DQL surface only and routes every
    /// request to the target global-cluster identifier. It shares this client's channel,
    /// selected database, RPC settings, and caches.
    ///
    /// A global-cluster connection is not required: the session is a routing directive that
    /// attaches `cluster_id` to each request's extra params. On a global cluster the identifier
    /// selects the member cluster to serve the request; on a regular server it is forwarded as an
    /// ordinary param, which the server ignores if it does not support cluster routing.
    pub fn session(&self, cluster_id: impl Into<String>) -> Result<MilvusClientV2Session> {
        let cluster_id = cluster_id.into();
        if cluster_id.is_empty() {
            return Err(Error::validation(
                "cluster_id".into(),
                "must not be empty".into(),
            ));
        }
        Ok(MilvusClientV2Session::new(self.clone(), cluster_id))
    }

    /// Triggers a reactive global-cluster failover probe on `UNAVAILABLE`.
    ///
    /// The probe runs in a detached task so it does not block the retry loop on a full topology
    /// fetch (which has its own retry backoff). A debounce window inside [`GlobalCluster::on_unavailable`]
    /// coalesces repeated probes so a burst of `UNAVAILABLE` attempts does not hammer the global
    /// REST endpoint; the rebuilt channel is picked up by later retry attempts.
    fn refresh_global_on_unavailable(&self) {
        if let Some(global) = &self.global_cluster {
            let global = Arc::clone(global);
            tokio::spawn(async move {
                global.on_unavailable().await;
            });
        }
    }

    async fn retry_rpc<Req, Resp, MakeRequest, Call, CallFuture, GetStatus>(
        &self,
        mut make_request: MakeRequest,
        semantics: RetrySemantics,
        mut call: Call,
        get_status: GetStatus,
    ) -> Result<Resp>
    where
        MakeRequest: FnMut() -> Result<Req>,
        Call: FnMut(Service, Request<Req>) -> CallFuture,
        CallFuture: Future<Output = std::result::Result<Response<Resp>, Status>>,
        GetStatus: Fn(&Resp) -> Option<crate::proto::common::Status>,
    {
        self.retry_call(
            || {
                let service = self.service.read().milvus.clone();
                let request = self.rpc_request(make_request()?);
                Ok(call(service, request))
            },
            Some(get_status),
            semantics,
        )
        .await
    }

    async fn retry_transport<Req, Resp, Call, CallFuture>(
        &self,
        request: Req,
        apply_rpc_timeout: bool,
        mut call: Call,
    ) -> Result<Resp>
    where
        Req: Clone,
        Call: FnMut(Service, Request<Req>) -> CallFuture,
        CallFuture: Future<Output = std::result::Result<Response<Resp>, Status>>,
    {
        self.retry_call(
            || {
                let service = self.service.read().milvus.clone();
                let request = if apply_rpc_timeout {
                    self.rpc_request(request.clone())
                } else {
                    Request::new(request.clone())
                };
                Ok(call(service, request))
            },
            None::<fn(&Resp) -> Option<crate::proto::common::Status>>,
            RetrySemantics::Idempotent,
        )
        .await
    }

    async fn retry_call<Resp, Call, CallFuture, GetStatus>(
        &self,
        mut call: Call,
        get_status: Option<GetStatus>,
        semantics: RetrySemantics,
    ) -> Result<Resp>
    where
        Call: FnMut() -> Result<CallFuture>,
        CallFuture: Future<Output = std::result::Result<Response<Resp>, Status>>,
        GetStatus: Fn(&Resp) -> Option<crate::proto::common::Status>,
    {
        let retry = self.retry.read().clone();
        let max_attempts = retry.max_attempts.max(1);
        let started = Instant::now();
        let mut backoff = retry.initial_backoff.min(retry.max_backoff);
        let multiplier = if retry.backoff_multiplier.is_finite() && retry.backoff_multiplier > 0.0 {
            retry.backoff_multiplier
        } else {
            1.0
        };

        for attempt in 1..=max_attempts {
            trace_debug!(
                target: "milvus_sdk::retry",
                attempt,
                max_attempts,
                semantics = ?semantics,
                "starting Milvus RPC attempt"
            );
            let call = call()?;
            let outcome = if retry.max_retry_timeout.is_zero() {
                call.await
            } else {
                let remaining = retry.max_retry_timeout.saturating_sub(started.elapsed());
                if remaining.is_zero() {
                    trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, timeout_ms = retry.max_retry_timeout.as_millis(), "Milvus RPC retry deadline reached before attempt completed");
                    return Err(retry_attempt_timed_out(retry.max_retry_timeout, attempt));
                }
                match tokio::time::timeout(remaining, call).await {
                    Ok(outcome) => outcome,
                    Err(_) => {
                        trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, timeout_ms = retry.max_retry_timeout.as_millis(), "Milvus RPC attempt exceeded total retry deadline");
                        return Err(retry_attempt_timed_out(retry.max_retry_timeout, attempt));
                    }
                }
            };
            let failure = match outcome {
                Ok(response) => {
                    let response = response.into_inner();
                    if let Some(get_status) = &get_status {
                        let status = get_status(&response).ok_or_else(|| {
                            Error::MalformedResponse(
                                "RPC response does not contain a status".into(),
                            )
                        })?;
                        if let Err(error) =
                            crate::v2::error::status_to_result(&Some(status.clone()))
                        {
                            let rate_limited = is_rate_limit(&status);
                            let region_switch = is_replicate_violation(&status);
                            if region_switch {
                                trace_info!(target: "milvus_sdk::retry", attempt, max_attempts, "detected global cluster region switch (REPLICATE_VIOLATION), triggering topology refresh");
                                self.refresh_global_on_unavailable();
                            }
                            if region_switch
                                || (rate_limited && retry.retry_on_rate_limit)
                                || (!rate_limited
                                    && status.retriable
                                    && semantics == RetrySemantics::Idempotent)
                            {
                                trace_debug!(
                                    target: "milvus_sdk::retry",
                                    attempt,
                                    max_attempts,
                                    rate_limited,
                                    retriable = status.retriable,
                                    semantics = ?semantics,
                                    "Milvus server status is eligible for retry"
                                );
                                error
                            } else {
                                trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, rate_limited, retriable = status.retriable, semantics = ?semantics, error = %error, "Milvus server status is not eligible for retry");
                                return Err(error);
                            }
                        } else {
                            trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, elapsed_ms = started.elapsed().as_millis(), "Milvus RPC completed successfully");
                            return Ok(response);
                        }
                    } else {
                        trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, elapsed_ms = started.elapsed().as_millis(), "Milvus RPC completed successfully");
                        return Ok(response);
                    }
                }
                Err(status)
                    if semantics == RetrySemantics::Idempotent
                        && is_retryable_grpc(status.code()) =>
                {
                    trace_debug!(
                        target: "milvus_sdk::retry",
                        attempt,
                        max_attempts,
                        grpc_code = ?status.code(),
                        "gRPC transport status is eligible for retry"
                    );
                    if status.code() == tonic::Code::Unavailable {
                        self.refresh_global_on_unavailable();
                    }
                    Error::Grpc(status)
                }
                Err(status) => {
                    trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, grpc_code = ?status.code(), semantics = ?semantics, "gRPC transport status is not eligible for retry");
                    if status.code() == tonic::Code::Unavailable {
                        self.refresh_global_on_unavailable();
                    }
                    return Err(status.into());
                }
            };

            if attempt >= max_attempts {
                trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, error = %failure, elapsed_ms = started.elapsed().as_millis(), "Milvus RPC retry attempts exhausted");
                return Err(retry_exhausted(max_attempts, failure));
            }
            if retry_timeout_reached(started, backoff, retry.max_retry_timeout) {
                trace_debug!(target: "milvus_sdk::retry", attempt, max_attempts, error = %failure, timeout_ms = retry.max_retry_timeout.as_millis(), elapsed_ms = started.elapsed().as_millis(), "Milvus RPC retry timeout reached");
                return Err(retry_timed_out(retry.max_retry_timeout, attempt, &failure));
            }

            trace_debug!(
                target: "milvus_sdk::retry",
                attempt,
                max_attempts,
                backoff_ms = backoff.as_millis(),
                "waiting before Milvus RPC retry"
            );
            tokio::time::sleep(backoff).await;
            backoff = next_backoff(backoff, multiplier, retry.max_backoff);
        }

        unreachable!("retry loop always returns")
    }
}

async fn configured_endpoint(param: &ConnectConfig) -> Result<(Endpoint, String)> {
    validate_client_identity(param)?;

    let tls_enabled = tls_enabled(param);
    let endpoint_uri = tls_endpoint_uri(&param.uri, tls_enabled)?;
    let endpoint = build_endpoint(&endpoint_uri, param).await?;
    Ok((endpoint, endpoint_uri))
}

fn tls_enabled(param: &ConnectConfig) -> bool {
    param.uri.starts_with("https://")
        || param.tls_server_name.is_some()
        || param.ca_certificate.is_some()
        || param.client_certificate.is_some()
        || param.client_key.is_some()
}

async fn build_endpoint(endpoint_uri: &str, param: &ConnectConfig) -> Result<Endpoint> {
    let mut endpoint = Endpoint::from_shared(endpoint_uri.to_owned())
        .map_err(|error| Error::validation("uri".into(), error.to_string()))?
        .connect_timeout(param.connect_timeout)
        .http2_keep_alive_interval(param.keepalive_time)
        .keep_alive_timeout(param.keepalive_timeout)
        .keep_alive_while_idle(param.keepalive_while_idle);

    if tls_enabled(param) {
        let mut tls = ClientTlsConfig::new();
        if let Some(server_name) = &param.tls_server_name {
            tls = tls.domain_name(server_name);
        }
        if let Some(path) = &param.ca_certificate {
            let certificate = read_tls_file("ca_certificate", path).await?;
            tls = tls.ca_certificate(Certificate::from_pem(certificate));
        } else {
            tls = tls.with_enabled_roots();
        }
        if let (Some(certificate_path), Some(key_path)) =
            (&param.client_certificate, &param.client_key)
        {
            let (certificate, key) = tokio::try_join!(
                read_tls_file("client_certificate", certificate_path),
                read_tls_file("client_key", key_path)
            )?;
            tls = tls.identity(Identity::from_pem(certificate, key));
        }
        endpoint = endpoint
            .tls_config(tls)
            .map_err(|error| Error::validation("tls".into(), error.to_string()))?;
    }

    Ok(endpoint)
}

fn validate_client_identity(param: &ConnectConfig) -> Result<()> {
    match (&param.client_certificate, &param.client_key) {
        (Some(_), None) => Err(Error::validation(
            "client_key".into(),
            "must be specified together with client_certificate".into(),
        )),
        (None, Some(_)) => Err(Error::validation(
            "client_certificate".into(),
            "must be specified together with client_key".into(),
        )),
        _ => Ok(()),
    }
}

fn tls_endpoint_uri(uri: &str, tls_enabled: bool) -> Result<String> {
    if !tls_enabled || uri.starts_with("https://") {
        return Ok(uri.to_owned());
    }
    if let Some(address) = uri.strip_prefix("http://") {
        return Ok(format!("https://{address}"));
    }
    Err(Error::validation(
        "uri".into(),
        "TLS configuration requires an http:// or https:// URI".into(),
    ))
}

async fn read_tls_file(parameter: &str, path: &str) -> Result<Vec<u8>> {
    tokio::fs::read(path).await.map_err(|error| {
        Error::validation(
            parameter.to_owned(),
            format!("failed to read PEM file {path:?}: {error}"),
        )
    })
}

async fn wait_for_server(service: &mut Service, connect_timeout: Duration) -> Result<()> {
    const RETRY_INTERVAL: Duration = Duration::from_millis(50);

    let started = Instant::now();
    loop {
        let remaining = connect_timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(connection_timeout_error(connect_timeout));
        }

        let mut request = Request::new(milvus::ConnectRequest {
            base: None,
            client_info: Some(common::ClientInfo {
                sdk_type: "Rust".into(),
                sdk_version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            }),
        });
        request.set_timeout(remaining);

        match tokio::time::timeout(remaining, service.connect(request)).await {
            Ok(Ok(response)) => {
                crate::v2::error::status_to_result(&response.into_inner().status)?;
                return Ok(());
            }
            Ok(Err(status)) if status.code() == Code::Unavailable => {}
            Ok(Err(status)) if status.code() == Code::DeadlineExceeded => {
                return Err(connection_timeout_error(connect_timeout));
            }
            Ok(Err(status)) => return Err(Error::Grpc(status)),
            Err(_) => return Err(connection_timeout_error(connect_timeout)),
        }

        let remaining = connect_timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(connection_timeout_error(connect_timeout));
        }
        tokio::time::sleep(RETRY_INTERVAL.min(remaining)).await;
    }
}

fn connection_timeout_error(connect_timeout: Duration) -> Error {
    Error::Timeout(format!(
        "connecting to Milvus after {}ms",
        connect_timeout.as_millis()
    ))
}

fn is_retryable_grpc(code: Code) -> bool {
    !matches!(
        code,
        Code::DeadlineExceeded
            | Code::PermissionDenied
            | Code::Unauthenticated
            | Code::InvalidArgument
            | Code::AlreadyExists
            | Code::ResourceExhausted
            | Code::Unimplemented
    )
}

/// Server error marker for a global-cluster region switch, mirroring pymilvus and the Java SDK.
///
/// A streaming replicate-violation rejection means the request reached a replica that is no longer
/// the primary for its key range after a region switch, so the client must refresh the global
/// topology (and rebuild to the current primary) before retrying.
const STREAMING_CODE_REPLICATE_VIOLATION: &str = "STREAMING_CODE_REPLICATE_VIOLATION";

/// Returns whether the server status signals a global-cluster region switch.
fn is_replicate_violation(status: &crate::proto::common::Status) -> bool {
    status.reason.contains(STREAMING_CODE_REPLICATE_VIOLATION)
}

#[allow(deprecated)]
fn is_rate_limit(status: &crate::proto::common::Status) -> bool {
    status.error_code == crate::proto::common::ErrorCode::RateLimit as i32 || status.code == 8
}

fn next_backoff(current: Duration, multiplier: f64, maximum: Duration) -> Duration {
    Duration::try_from_secs_f64(current.as_secs_f64() * multiplier)
        .unwrap_or(maximum)
        .min(maximum)
}

fn retry_timeout_reached(started: Instant, backoff: Duration, limit: Duration) -> bool {
    if limit.is_zero() {
        return false;
    }
    let elapsed = started.elapsed();
    elapsed >= limit
        || elapsed
            .checked_add(backoff)
            .map_or(true, |future| future >= limit)
}

fn retry_exhausted(max_attempts: u32, failure: Error) -> Error {
    Error::RetryExhausted {
        attempts: max_attempts,
        source: Box::new(failure),
    }
}

fn retry_timed_out(limit: Duration, attempts: u32, failure: &Error) -> Error {
    Error::Timeout(format!(
        "RPC retry timeout after {}ms and {attempts} attempts: {failure}",
        limit.as_millis()
    ))
}

fn retry_attempt_timed_out(limit: Duration, attempt: u32) -> Error {
    Error::Timeout(format!(
        "RPC retry timeout after {}ms while attempt {attempt} was in progress",
        limit.as_millis()
    ))
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod retry_tests {
    use super::*;
    use crate::proto::common;
    use crate::v2::types::TelemetryConfig;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn client(retry: RetryConfig) -> ClientV2 {
        let database = Arc::new(RwLock::new("default".to_owned()));
        let database_explicit = Arc::new(AtomicBool::new(false));
        let channel = Endpoint::from_static("http://127.0.0.1:19530").connect_lazy();
        let interceptor = V2Interceptor {
            token: None,
            database: Arc::clone(&database),
            database_explicit: Arc::clone(&database_explicit),
        };
        let service = Arc::new(RwLock::new(service_bundle(channel, interceptor, 0)));
        let config = ConnectConfig::new().telemetry(TelemetryConfig::new().enabled(false));
        let telemetry = ClientTelemetry::new(
            config.telemetry.clone(),
            Arc::clone(&service),
            Arc::clone(&database),
            Arc::clone(&database_explicit),
            &config,
        );
        ClientV2 {
            service,
            database,
            database_explicit,
            rpc_timeout: Arc::new(RwLock::new(Duration::from_secs(1))),
            retry: Arc::new(RwLock::new(retry)),
            cache_endpoint: Arc::new("http://127.0.0.1:19530".to_owned()),
            schema_load_scope: Arc::new(SchemaLoadScope::new()),
            global_cluster: None,
            telemetry,
        }
    }

    fn fast_retry(max_attempts: u32) -> RetryConfig {
        RetryConfig {
            max_attempts,
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
            ..RetryConfig::new()
        }
    }

    fn retriable_server_status() -> common::Status {
        common::Status {
            code: 1000,
            reason: "temporarily unavailable".into(),
            retriable: true,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn client_clones_share_schema_load_scope_but_independent_clients_do_not() {
        let first = client(RetryConfig::new());
        let clone = first.clone();
        let second = client(RetryConfig::new());

        assert!(Arc::ptr_eq(
            &first.schema_load_scope,
            &clone.schema_load_scope
        ));
        assert!(!Arc::ptr_eq(
            &first.schema_load_scope,
            &second.schema_load_scope
        ));
    }

    #[test]
    fn interceptor_sends_base64_encoded_authorization_metadata() {
        let config = ConnectConfig::new()
            .uri("http://127.0.0.1:19530")
            .token("root:Milvus");
        let token = config.token.map(|value| value.parse()).transpose().unwrap();
        let mut interceptor = V2Interceptor {
            token,
            database: Arc::new(RwLock::new(String::new())),
            database_explicit: Arc::new(AtomicBool::new(false)),
        };

        let request = interceptor.call(Request::new(())).unwrap();
        assert_eq!(
            request
                .metadata()
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap(),
            "cm9vdDpNaWx2dXM="
        );
    }

    #[test]
    fn interceptor_distinguishes_unset_and_explicit_default_database() {
        let database = Arc::new(RwLock::new("default".to_owned()));
        let database_explicit = Arc::new(AtomicBool::new(false));
        let mut interceptor = V2Interceptor {
            token: None,
            database,
            database_explicit: Arc::clone(&database_explicit),
        };

        let unset = interceptor.call(Request::new(())).unwrap();
        assert!(unset.metadata().get("dbname").is_none());

        database_explicit.store(true, std::sync::atomic::Ordering::Release);
        let explicit = interceptor.call(Request::new(())).unwrap();
        assert_eq!(
            explicit
                .metadata()
                .get("dbname")
                .expect("explicit default database")
                .to_str()
                .unwrap(),
            "default"
        );
    }

    #[test]
    fn initial_database_normalizes_empty_to_default() {
        assert_eq!(normalize_database(String::new()).unwrap(), "default");
    }

    #[test]
    fn tls_options_upgrade_http_endpoint_to_https() {
        assert_eq!(
            tls_endpoint_uri("http://milvus.example.com:19530", true).unwrap(),
            "https://milvus.example.com:19530"
        );
        assert_eq!(
            tls_endpoint_uri("https://milvus.example.com:19530", true).unwrap(),
            "https://milvus.example.com:19530"
        );
        assert_eq!(
            tls_endpoint_uri("http://milvus.example.com:19530", false).unwrap(),
            "http://milvus.example.com:19530"
        );
    }

    #[tokio::test]
    async fn portless_tls_upgrade_uses_the_effective_uri_for_cache_identity() {
        let upgraded = ConnectConfig::new()
            .uri("http://milvus.example.com")
            .tls_server_name("milvus.example.com");
        let (_, upgraded_cache_endpoint) = configured_endpoint(&upgraded)
            .await
            .expect("valid upgraded TLS endpoint");

        let explicit = ConnectConfig::new().uri("https://milvus.example.com");
        let (_, explicit_cache_endpoint) = configured_endpoint(&explicit)
            .await
            .expect("valid explicit TLS endpoint");

        assert_eq!(upgraded_cache_endpoint, "https://milvus.example.com");
        assert_eq!(upgraded_cache_endpoint, explicit_cache_endpoint);
        assert_ne!(upgraded_cache_endpoint, upgraded.get_uri());
    }

    #[tokio::test]
    async fn tls_configuration_requires_a_complete_client_identity() {
        let missing_key = ConnectConfig::new().client_certificate("client.pem");
        let Err(Error::Validation(error)) = configured_endpoint(&missing_key).await else {
            panic!("client certificate without a key must be rejected");
        };
        assert_eq!(error.parameter(), "client_key");

        let missing_certificate = ConnectConfig::new().client_key("client-key.pem");
        let Err(Error::Validation(error)) = configured_endpoint(&missing_certificate).await else {
            panic!("client key without a certificate must be rejected");
        };
        assert_eq!(error.parameter(), "client_certificate");
    }

    #[tokio::test]
    async fn global_cluster_connections_also_require_a_complete_client_identity() {
        // The global branch bypasses configured_endpoint, so connect must validate the identity
        // before selecting either connection path rather than silently ignoring an incomplete TLS
        // identity.
        let missing_key = ConnectConfig::new()
            .uri("https://my.global-cluster.example.com:443")
            .client_certificate("client.pem");
        let Err(Error::Validation(error)) = ClientV2::new(&missing_key).await else {
            panic!("client certificate without a key must be rejected on a global endpoint");
        };
        assert_eq!(error.parameter(), "client_key");

        let missing_certificate = ConnectConfig::new()
            .uri("https://my.global-cluster.example.com:443")
            .client_key("client-key.pem");
        let Err(Error::Validation(error)) = ClientV2::new(&missing_certificate).await else {
            panic!("client key without a certificate must be rejected on a global endpoint");
        };
        assert_eq!(error.parameter(), "client_certificate");
    }

    #[tokio::test]
    async fn tls_configuration_reports_unreadable_certificate_files() {
        let config = ConnectConfig::new()
            .ca_certificate("/path/that/does/not/exist/milvus-sdk-rust-test-ca-certificate.pem");
        let Err(Error::Validation(error)) = configured_endpoint(&config).await else {
            panic!("an unreadable CA certificate must be rejected");
        };
        assert_eq!(error.parameter(), "ca_certificate");
    }

    #[tokio::test]
    async fn client_construction_rejects_invalid_database_metadata() {
        let config = ConnectConfig::new()
            .uri("http://127.0.0.1:1")
            .database("bad\nname");

        let result = ClientV2::new(&config).await;
        let Err(Error::Validation(error)) = result else {
            panic!("invalid initial database must fail during client construction");
        };
        assert_eq!(error.parameter(), "database");
    }

    #[test]
    fn retry_defaults_match_sdk_policy() {
        let retry = RetryConfig::new();
        assert_eq!(retry.max_attempts, 75);
        assert_eq!(retry.max_retry_timeout, Duration::ZERO);
        assert_eq!(retry.initial_backoff, Duration::from_millis(10));
        assert_eq!(retry.max_backoff, Duration::from_secs(3));
        assert_eq!(retry.backoff_multiplier, 3.0);
        assert!(retry.retry_on_rate_limit);
    }

    #[tokio::test]
    async fn zero_rpc_deadline_does_not_emit_grpc_timeout() {
        let client = client(RetryConfig::new());
        client.set_rpc_deadline(Duration::ZERO);
        let request = client.rpc_request(());
        assert!(request.metadata().get("grpc-timeout").is_none());

        client.set_rpc_deadline(Duration::from_secs(2));
        let request = client.rpc_request(());
        assert!(request.metadata().get("grpc-timeout").is_some());
    }

    #[tokio::test]
    async fn transient_grpc_errors_are_retried() {
        let client = client(fast_retry(3));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    let attempt = observed.fetch_add(1, Ordering::SeqCst) + 1;
                    Ok(async move {
                        if attempt < 3 {
                            Err(Status::unavailable("temporarily unavailable"))
                        } else {
                            Ok(Response::new(()))
                        }
                    })
                },
                None::<fn(&()) -> Option<common::Status>>,
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn permanent_grpc_errors_are_not_retried() {
        let client = client(fast_retry(5));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async { Err::<Response<()>, _>(Status::invalid_argument("bad request")) })
                },
                None::<fn(&()) -> Option<common::Status>>,
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(
            matches!(result, Err(Error::Grpc(status)) if status.code() == Code::InvalidArgument)
        );
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn rate_limit_server_errors_are_retried() {
        let client = client(fast_retry(3));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    let attempt = observed.fetch_add(1, Ordering::SeqCst) + 1;
                    Ok(async move {
                        let status = if attempt < 3 {
                            common::Status {
                                error_code: common::ErrorCode::RateLimit as i32,
                                reason: "rate limited".into(),
                                ..Default::default()
                            }
                        } else {
                            common::Status::default()
                        };
                        Ok(Response::new(status))
                    })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn explicitly_retriable_server_errors_are_not_retried_for_non_idempotent_calls() {
        let client = client(fast_retry(3));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async { Ok(Response::new(retriable_server_status())) })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::NonIdempotent,
            )
            .await;

        assert!(matches!(result, Err(Error::Server(_))));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn rate_limit_server_errors_are_retried_for_non_idempotent_calls() {
        let client = client(fast_retry(3));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    let attempt = observed.fetch_add(1, Ordering::SeqCst) + 1;
                    Ok(async move {
                        let status = if attempt < 3 {
                            common::Status {
                                error_code: common::ErrorCode::RateLimit as i32,
                                reason: "rate limited".into(),
                                ..Default::default()
                            }
                        } else {
                            common::Status::default()
                        };
                        Ok(Response::new(status))
                    })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::NonIdempotent,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn non_retriable_server_errors_are_returned_immediately() {
        let client = client(fast_retry(5));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async {
                        Ok(Response::new(common::Status {
                            code: 1000,
                            reason: "permanent failure".into(),
                            retriable: false,
                            ..Default::default()
                        }))
                    })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(matches!(
            result,
            Err(Error::Server(error))
                if error.code() == 1000 && error.reason() == "permanent failure"
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn explicitly_retriable_server_errors_report_exhaustion() {
        let client = client(fast_retry(3));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async { Ok(Response::new(retriable_server_status())) })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(matches!(
            result,
            Err(Error::RetryExhausted { attempts: 3, source })
                if matches!(*source, Error::Server(_))
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn explicitly_retriable_server_errors_respect_retry_timeout() {
        let client = client(RetryConfig {
            max_attempts: 5,
            max_retry_timeout: Duration::from_millis(5),
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(10),
            ..RetryConfig::new()
        });
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async { Ok(Response::new(retriable_server_status())) })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(matches!(result, Err(Error::Timeout(message)) if message.contains("timeout")));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_timeout_bounds_a_stalled_attempt() {
        let client = client(RetryConfig {
            max_attempts: 5,
            max_retry_timeout: Duration::from_millis(5),
            ..RetryConfig::new()
        });
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(std::future::pending::<
                        std::result::Result<Response<()>, Status>,
                    >())
                },
                None::<fn(&()) -> Option<common::Status>>,
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(matches!(
            result,
            Err(Error::Timeout(message)) if message.contains("attempt 1 was in progress")
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn retry_rpc_builds_one_request_per_non_idempotent_attempt() {
        let client = client(fast_retry(3));
        let builds = Arc::new(AtomicUsize::new(0));
        let observed_builds = Arc::clone(&builds);
        let calls = Arc::new(AtomicUsize::new(0));
        let observed_calls = Arc::clone(&calls);

        let result = client
            .retry_rpc(
                move || Ok(observed_builds.fetch_add(1, Ordering::SeqCst) + 1),
                RetrySemantics::NonIdempotent,
                move |_service, request| {
                    let attempt = observed_calls.fetch_add(1, Ordering::SeqCst) + 1;
                    assert_eq!(request.into_inner(), attempt);
                    async move {
                        let status = if attempt < 3 {
                            common::Status {
                                error_code: common::ErrorCode::RateLimit as i32,
                                reason: "rate limited".into(),
                                ..Default::default()
                            }
                        } else {
                            common::Status::default()
                        };
                        Ok(Response::new(status))
                    }
                },
                |status: &common::Status| Some(status.clone()),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(builds.load(Ordering::SeqCst), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_timeout_is_checked_before_sleeping() {
        let client = client(RetryConfig {
            max_attempts: 5,
            max_retry_timeout: Duration::from_millis(5),
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(10),
            ..RetryConfig::new()
        });
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async { Err::<Response<()>, _>(Status::unavailable("offline")) })
                },
                None::<fn(&()) -> Option<common::Status>>,
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(matches!(result, Err(Error::Timeout(message)) if message.contains("timeout")));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn rate_limit_retry_can_be_disabled() {
        let client = client(RetryConfig {
            retry_on_rate_limit: false,
            ..fast_retry(5)
        });
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async {
                        Ok(Response::new(common::Status {
                            error_code: common::ErrorCode::RateLimit as i32,
                            reason: "rate limited".into(),
                            retriable: true,
                            ..Default::default()
                        }))
                    })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::Idempotent,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn non_idempotent_calls_do_not_retry_ambiguous_transport_errors() {
        for code in [
            Code::Unavailable,
            Code::Unknown,
            Code::Internal,
            Code::Aborted,
            Code::Cancelled,
        ] {
            let client = client(fast_retry(5));
            let attempts = Arc::new(AtomicUsize::new(0));
            let observed = Arc::clone(&attempts);
            let result = client
                .retry_call(
                    move || {
                        observed.fetch_add(1, Ordering::SeqCst);
                        Ok(async move { Err::<Response<()>, _>(Status::new(code, "ack lost")) })
                    },
                    None::<fn(&()) -> Option<common::Status>>,
                    RetrySemantics::NonIdempotent,
                )
                .await;

            assert!(matches!(result, Err(Error::Grpc(status)) if status.code() == code));
            assert_eq!(attempts.load(Ordering::SeqCst), 1);
        }
    }

    #[tokio::test]
    async fn replicate_violation_server_status_is_retried_even_for_non_idempotent_calls() {
        // A streaming replicate-violation signals a global-cluster region switch. Mirroring the
        // Java SDK and pymilvus, it must be retried (after triggering a topology refresh) even for
        // a non-idempotent call that would otherwise never replay a server status error.
        let client = client(fast_retry(3));
        let attempts = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&attempts);
        let result = client
            .retry_call(
                move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(async {
                        Ok(Response::new(common::Status {
                            code: 1001,
                            reason: "STREAMING_CODE_REPLICATE_VIOLATION: region switched".into(),
                            retriable: false,
                            ..Default::default()
                        }))
                    })
                },
                Some(|status: &common::Status| Some(status.clone())),
                RetrySemantics::NonIdempotent,
            )
            .await;

        assert!(matches!(
            result,
            Err(Error::RetryExhausted { attempts: 3, source })
                if matches!(*source, Error::Server(_))
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn replicate_violation_marker_is_detected_in_server_reason() {
        let region_switch = common::Status {
            reason: "STREAMING_CODE_REPLICATE_VIOLATION: replica not primary".into(),
            ..Default::default()
        };
        assert!(is_replicate_violation(&region_switch));

        let ordinary = common::Status {
            reason: "temporarily unavailable".into(),
            ..Default::default()
        };
        assert!(!is_replicate_violation(&ordinary));
    }
}
