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

//! Global-cluster topology discovery and primary failover.
//!
//! A Milvus deployment may be fronted by a logical global-cluster endpoint that serves a REST
//! topology of member clusters. This module fetches that topology, resolves the writable primary,
//! and rebuilds the gRPC channel when the primary endpoint changes.

use super::{ServiceBundle, SharedServices, TransportGeneration, V2Interceptor};
use crate::v2::error::{Error, Result};
use crate::v2::types::{topology_url, GlobalTopology};
use parking_lot::RwLock;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_FETCH_ATTEMPTS: u32 = 3;
const BASE_DELAY: Duration = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(10);
const REFRESH_INTERVAL: Duration = Duration::from_secs(300);
/// Minimum spacing between reactive topology probes triggered by `UNAVAILABLE` errors.
///
/// Keeps a burst of retry attempts from each firing a full topology fetch (with its own backoff)
/// against the global REST endpoint.
const ON_UNAVAILABLE_DEBOUNCE: Duration = Duration::from_secs(5);

/// Fetches the global-cluster topology from a global endpoint's REST API.
///
/// The HTTP client is built from the supplied [`ConnectConfig`] so TLS discovery honors the same
/// connection timeout, custom CA, client identity, and server-name override as the gRPC channel.
/// Retries transient HTTP failures (transport errors, 5xx, 429) with bounded exponential backoff
/// and returns immediately on client errors (4xx).
///
/// The entire discovery is capped by `connect_config.connect_timeout`, matching the plain
/// (non-global) path so a short user timeout is not silently exceeded while a global endpoint's
/// REST service is unreachable.
pub(crate) async fn fetch_topology(
    global_endpoint: &str,
    connect_config: &super::ConnectConfig,
) -> Result<GlobalTopology> {
    let url = topology_url(global_endpoint, super::tls_enabled(connect_config));
    let client = build_topology_http_client(connect_config).await?;

    let discover = fetch_topology_with_retries(&url, connect_config, &client);
    if connect_config.connect_timeout.is_zero() {
        return discover.await;
    }
    match tokio::time::timeout(connect_config.connect_timeout, discover).await {
        Ok(result) => result,
        Err(_) => Err(Error::Timeout(format!(
            "global topology discovery exceeded the {}ms connect timeout",
            connect_config.connect_timeout.as_millis()
        ))),
    }
}

/// Retries the topology fetch with bounded exponential backoff.
async fn fetch_topology_with_retries(
    url: &str,
    connect_config: &super::ConnectConfig,
    client: &Client,
) -> Result<GlobalTopology> {
    let mut delay = BASE_DELAY;
    let mut last_error = None;
    for attempt in 0..MAX_FETCH_ATTEMPTS {
        let mut request = client.get(url);
        if let Some(token) = connect_config.raw_token() {
            request = request.bearer_auth(token);
        }
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                last_error = Some(Error::Unexpected(format!(
                    "failed to fetch global topology: {error}"
                )));
                if attempt + 1 < MAX_FETCH_ATTEMPTS {
                    sleep_with_jitter(delay, attempt).await;
                    delay = (delay * 2).min(MAX_DELAY);
                }
                continue;
            }
        };
        if response.status() != StatusCode::OK {
            // Retry transient failures (transport-level 5xx and rate-limit 429) with backoff, but
            // return immediately on client errors (4xx): a wrong token or mistyped URL can never
            // succeed on retry, so waiting only adds avoidable latency.
            let transient = response.status().is_server_error()
                || response.status() == StatusCode::TOO_MANY_REQUESTS;
            last_error = Some(Error::Unexpected(format!(
                "global topology request failed with status {}",
                response.status()
            )));
            if !transient {
                break;
            }
            if attempt + 1 < MAX_FETCH_ATTEMPTS {
                sleep_with_jitter(delay, attempt).await;
                delay = (delay * 2).min(MAX_DELAY);
            }
            continue;
        }
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => {
                last_error = Some(Error::Unexpected(format!(
                    "failed to read global topology body: {error}"
                )));
                if attempt + 1 < MAX_FETCH_ATTEMPTS {
                    sleep_with_jitter(delay, attempt).await;
                    delay = (delay * 2).min(MAX_DELAY);
                }
                continue;
            }
        };
        match crate::v2::types::parse_topology_response(&body) {
            Ok(topology) => return Ok(topology),
            Err(error) => {
                // A truncated or partial 200 body (e.g. a proxy cutting the response during a
                // backend switch) is transient, so retry it with the same backoff as transport
                // errors and 5xx/429. A well-formed body with a non-zero `code` is a definitive
                // server rejection (e.g. an auth failure), so return it immediately.
                let retryable = matches!(error, Error::MalformedResponse(_));
                last_error = Some(Error::Unexpected(format!(
                    "failed to parse global topology response: {error}"
                )));
                if !retryable {
                    return Err(error);
                }
                if attempt + 1 < MAX_FETCH_ATTEMPTS {
                    sleep_with_jitter(delay, attempt).await;
                    delay = (delay * 2).min(MAX_DELAY);
                }
                continue;
            }
        }
    }
    Err(last_error
        .unwrap_or_else(|| Error::Unexpected("failed to fetch global topology".to_owned())))
}

/// Sleeps for `base` plus up to 10% jitter, mirroring pymilvus's retry backoff.
async fn sleep_with_jitter(base: Duration, attempt: u32) {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(0)
        .wrapping_add(u64::from(attempt).wrapping_mul(0x9E3779B97F4A7C15));
    let jitter_fraction = seed % 10;
    let jitter = Duration::from_nanos(base.as_nanos() as u64 / 100 * jitter_fraction);
    tokio::time::sleep(base + jitter).await;
}

/// Builds the topology REST HTTP client from [`ConnectConfig`].
///
/// The connection timeout, custom CA certificate, client identity, and TLS server-name override
/// are honored so a private-CA or mTLS global endpoint can be discovered before the correctly
/// configured gRPC channel is built.
async fn build_topology_http_client(config: &super::ConnectConfig) -> Result<Client> {
    let mut builder = Client::builder()
        .connect_timeout(config.connect_timeout)
        .timeout(REQUEST_TIMEOUT);

    if let Some(server_name) = &config.tls_server_name {
        // reqwest derives the verified hostname from the URL, so an explicit server-name override
        // requires a preconfigured rustls config with a fixed-name verifier.
        let tls = build_topology_tls_config(config, server_name).await?;
        builder = builder.use_preconfigured_tls(Some(tls));
    } else {
        if let Some(path) = &config.ca_certificate {
            let pem = super::read_tls_file("ca_certificate", path).await?;
            let cert = reqwest::Certificate::from_pem(&pem)
                .map_err(|error| Error::validation("ca_certificate".into(), error.to_string()))?;
            builder = builder.add_root_certificate(cert);
        }
        if let (Some(cert_path), Some(key_path)) = (&config.client_certificate, &config.client_key)
        {
            let (cert, key) = tokio::try_join!(
                super::read_tls_file("client_certificate", cert_path),
                super::read_tls_file("client_key", key_path)
            )?;
            // reqwest's Identity::from_pem reads both the private key and certificate from one
            // PEM buffer; concatenate the two files.
            let mut pem = key;
            pem.extend_from_slice(&cert);
            let identity = reqwest::Identity::from_pem(&pem).map_err(|error| {
                Error::validation("client_certificate".into(), error.to_string())
            })?;
            builder = builder.identity(identity);
        }
    }

    builder.build().map_err(|error| {
        Error::Unexpected(format!("failed to build topology HTTP client: {error}"))
    })
}

/// Verifies the peer certificate against a fixed server name, ignoring the URL host.
#[derive(Debug)]
struct FixedServerNameVerifier {
    name: rustls::pki_types::ServerName<'static>,
    inner: Arc<rustls::client::WebPkiServerVerifier>,
}

impl rustls::client::danger::ServerCertVerifier for FixedServerNameVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        self.inner
            .verify_server_cert(end_entity, intermediates, &self.name, ocsp_response, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// Builds a preconfigured rustls client config that validates against the configured
/// server name while honoring the custom CA and client identity from [`ConnectConfig`].
async fn build_topology_tls_config(
    config: &super::ConnectConfig,
    server_name: &str,
) -> Result<rustls::ClientConfig> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};

    let name = ServerName::try_from(server_name.to_owned())
        .map_err(|error| Error::validation("tls_server_name".into(), error.to_string()))?;

    let mut root_store = rustls::RootCertStore::empty();
    if let Some(path) = &config.ca_certificate {
        let pem = super::read_tls_file("ca_certificate", path).await?;
        for cert in CertificateDer::pem_slice_iter(&pem) {
            let cert = cert
                .map_err(|error| Error::validation("ca_certificate".into(), error.to_string()))?;
            root_store
                .add(cert)
                .map_err(|error| Error::validation("ca_certificate".into(), error.to_string()))?;
        }
    } else {
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = rustls::client::WebPkiServerVerifier::builder_with_provider(
        Arc::new(root_store),
        Arc::clone(&provider),
    )
    .build()
    .map_err(|error| {
        Error::Unexpected(format!("failed to build topology TLS verifier: {error}"))
    })?;
    let verifier = Arc::new(FixedServerNameVerifier {
        name,
        inner: verifier,
    });

    let builder = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .map_err(|error| {
            Error::Unexpected(format!(
                "failed to configure topology TLS versions: {error}"
            ))
        })?;
    let builder = builder
        .dangerous()
        .with_custom_certificate_verifier(verifier);

    if let (Some(cert_path), Some(key_path)) = (&config.client_certificate, &config.client_key) {
        let (cert, key) = tokio::try_join!(
            super::read_tls_file("client_certificate", cert_path),
            super::read_tls_file("client_key", key_path)
        )?;
        let certs: std::result::Result<Vec<CertificateDer<'static>>, _> =
            CertificateDer::pem_slice_iter(&cert).collect();
        let certs = certs
            .map_err(|error| Error::validation("client_certificate".into(), error.to_string()))?;
        let key = PrivateKeyDer::from_pem_slice(&key)
            .map_err(|error| Error::validation("client_key".into(), error.to_string()))?;
        builder
            .with_client_auth_cert(certs, key)
            .map_err(|error| Error::validation("client_certificate".into(), error.to_string()))
    } else {
        Ok(builder.with_no_client_auth())
    }
}

/// Builds a gRPC channel and Milvus service for a member-cluster endpoint.
///
/// The channel is created lazily; `wait_for_server` on the caller performs the
/// initial connectivity check.
pub(crate) async fn build_services(
    endpoint_uri: &str,
    connect_config: &super::ConnectConfig,
    database: &Arc<RwLock<String>>,
    generation: TransportGeneration,
) -> Result<ServiceBundle> {
    let endpoint = super::build_endpoint(endpoint_uri, connect_config).await?;
    let channel = endpoint.connect_lazy();
    let token = connect_config
        .token
        .as_ref()
        .map(|value| value.parse())
        .transpose()
        .map_err(|_| {
            Error::validation("token".into(), "token is not valid HTTP metadata".into())
        })?;
    let interceptor = V2Interceptor {
        token,
        database: Arc::clone(database),
    };
    Ok(super::service_bundle(channel, interceptor, generation))
}

/// Shared global-cluster state that manages topology discovery and primary failover.
///
/// Dropping the last strong reference to a `GlobalCluster` aborts its background refresh task
/// (via the cancellation token) so the task and its captured resources do not outlive the client.
///////////////////////////////////////////////////////////////////////////////
// GlobalCluster
///////////////////////////////////////////////////////////////////////////////
pub(crate) struct GlobalCluster {
    global_endpoint: String,
    connect_config: super::ConnectConfig,
    database: Arc<RwLock<String>>,
    service: SharedServices,
    topology: RwLock<GlobalTopology>,
    last_unavailable_probe: std::sync::Mutex<Option<std::time::Instant>>,
    probe_in_progress: std::sync::atomic::AtomicBool,
    apply_lock: tokio::sync::Mutex<()>,
    shutdown: tokio_util::sync::CancellationToken,
}

impl GlobalCluster {
    pub(crate) fn new(
        global_endpoint: String,
        connect_config: super::ConnectConfig,
        database: Arc<RwLock<String>>,
        service: SharedServices,
        topology: GlobalTopology,
    ) -> Self {
        Self {
            global_endpoint,
            connect_config,
            database,
            service,
            topology: RwLock::new(topology),
            last_unavailable_probe: std::sync::Mutex::new(None),
            probe_in_progress: std::sync::atomic::AtomicBool::new(false),
            apply_lock: tokio::sync::Mutex::new(()),
            shutdown: tokio_util::sync::CancellationToken::new(),
        }
    }

    /// Returns a snapshot of the current topology.
    #[cfg(test)]
    pub(crate) fn topology(&self) -> GlobalTopology {
        self.topology.read().clone()
    }

    /// Spawns the background topology refresh loop.
    ///
    /// The loop re-fetches the topology every `REFRESH_INTERVAL` and only applies a topology whose
    /// version is strictly newer than the cached one, mirroring pymilvus's `TopologyRefresher`.
    /// When a newer topology advertises a changed primary, the loop fails over proactively by
    /// rebuilding the gRPC channel (see [`Self::refresh`]); `on_unavailable` additionally reacts to
    /// a primary change discovered after an `UNAVAILABLE` error.
    ///
    /// The task holds only a `Weak<Self>` reference so it does not keep the `GlobalCluster` (and
    /// its `ConnectConfig`/service/topology) alive. When the parent client drops its strong
    /// reference, `Drop` cancels the token and the loop exits.
    pub(crate) fn start_refresh(self: &Arc<Self>) {
        let this = Arc::downgrade(self);
        // `tokio::time::interval`'s first tick fires immediately; the connect path already fetched
        // the topology and verified the primary, so consume that first tick without refreshing.
        let first_tick = tokio::time::Instant::now() + REFRESH_INTERVAL;
        let mut interval = tokio::time::interval_at(first_tick, REFRESH_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let shutdown_rx = self.shutdown.clone().child_token();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        if let Some(this) = this.upgrade() {
                            this.refresh().await;
                        } else {
                            break;
                        }
                    }
                }
            }
        });
    }

    async fn refresh(&self) {
        let topology = match fetch_topology(&self.global_endpoint, &self.connect_config).await {
            Ok(topology) => topology,
            Err(error) => {
                trace_warn!(target: "milvus_sdk::global_cluster", error = %error, "global topology refresh failed; keeping cached topology");
                let _ = error;
                return;
            }
        };
        self.apply_topology(topology).await;
    }

    /// Reacts to a gRPC `UNAVAILABLE` error by re-fetching the topology and rebuilding the
    /// channel to the current primary, mirroring pymilvus's `on_unavailable`.
    ///
    /// The channel is only swapped when the primary actually changed; a failed topology fetch
    /// still recovers by reconnecting to the current primary.
    ///
    /// The topology fetch is debounced: if `on_unavailable` already ran within [`ON_UNAVAILABLE_DEBOUNCE`],
    /// the probe is skipped so a burst of `UNAVAILABLE` retries does not hammer the global REST
    /// endpoint with one full fetch (and its own retry backoff) per attempt.
    pub(crate) async fn on_unavailable(&self) {
        if !self.begin_probe() {
            trace_debug!(target: "milvus_sdk::global_cluster", "on_unavailable probe debounced or already in flight; keeping cached topology");
            return;
        }
        let _probe = ProbeGuard::new(self);
        let old_primary = self
            .topology
            .read()
            .primary()
            .ok()
            .map(|primary| primary.endpoint().to_owned());
        let topology = match fetch_topology(&self.global_endpoint, &self.connect_config).await {
            Ok(topology) => topology,
            Err(error) => {
                trace_warn!(target: "milvus_sdk::global_cluster", error = %error, "topology refresh on UNAVAILABLE failed; recovering to current primary");
                let _ = error;
                if let Some(endpoint) = &old_primary {
                    // Serialize with `apply_topology` (the other writer of `self.service`) so a
                    // concurrent background refresh cannot commit a new primary while this
                    // recovery rebuilds to the old one.
                    let _apply_guard = self.apply_lock.lock().await;
                    // Re-validate the captured `old_primary` after acquiring the lock: a
                    // concurrent background refresh may have committed a newer topology (and
                    // swapped the channel) while this fetch was in flight, in which case
                    // rebuilding to the stale snapshot would fail the client back to a retired
                    // primary. Only rebuild when the cached primary is still the captured one.
                    let current_primary = self
                        .topology
                        .read()
                        .primary()
                        .ok()
                        .map(|primary| primary.endpoint().to_owned());
                    if current_primary.as_deref() == Some(endpoint.as_str()) {
                        self.rebuild_to(endpoint).await;
                    }
                }
                return;
            }
        };
        self.apply_topology(topology).await;
    }

    /// Returns `true` if a probe may start: no probe is currently in flight and the last probe
    /// completed outside the debounce window.
    fn begin_probe(&self) -> bool {
        let mut last_probe = self.last_unavailable_probe.lock().expect("probe lock");
        if self
            .probe_in_progress
            .load(std::sync::atomic::Ordering::SeqCst)
            || last_probe
                .map(|probed_at| probed_at.elapsed() < ON_UNAVAILABLE_DEBOUNCE)
                .unwrap_or(false)
        {
            return false;
        }
        self.probe_in_progress
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = &mut last_probe;
        true
    }

    /// Marks a probe complete: clears the in-flight flag and records the completion time.
    ///
    /// Recording the completion (rather than the start) time keeps probes serialized: an
    /// overlapping `UNAVAILABLE` burst cannot start a second fetch while the first is still
    /// running, which is the precondition for the stale-service-swap race in `apply_topology`.
    fn finish_probe(&self) {
        *self.last_unavailable_probe.lock().expect("probe lock") = Some(std::time::Instant::now());
        self.probe_in_progress
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Applies a freshly fetched topology under the shared version guard used by both failover
    /// paths.
    ///
    /// A topology is only committed when its version is strictly newer than the cached one, so a
    /// stale response (e.g. from a lagging REST replica) never overwrites a fresher topology or
    /// fails the client back to a primary that a newer refresh had already replaced. When the
    /// writable primary changed, the gRPC channel is rebuilt first and the topology committed only
    /// on success so the stored primary always matches the endpoint the client routes to.
    async fn apply_topology(&self, topology: GlobalTopology) {
        // Serialize topology applications: both the background refresh loop and `on_unavailable`
        // call this, and a concurrent pair is the precondition for installing a stale primary's
        // channel. While this task awaits `rebuild_to` (gRPC handshake + TLS reads), a second
        // `apply_topology` would otherwise commit a newer primary and then have its swap
        // overwritten when the slower task finally finishes.
        let _apply_guard = self.apply_lock.lock().await;
        let new_primary = match topology.primary() {
            Ok(primary) => primary.clone(),
            Err(error) => {
                trace_warn!(target: "milvus_sdk::global_cluster", error = %error, "refreshed topology has no primary; keeping cached topology");
                let _ = error;
                return;
            }
        };
        let (primary_changed, version_newer) = {
            let current = self.topology.read();
            let primary_changed = current
                .primary()
                .ok()
                .map(|current_primary| current_primary.endpoint() != new_primary.endpoint())
                .unwrap_or(true);
            (primary_changed, topology.version() > current.version())
        };
        if primary_changed {
            // A changed primary triggers failover even when the version is not strictly newer:
            // the global REST service or its backing store may have restarted and reset the
            // version counter, in which case the strict `>` guard would otherwise discard every
            // future topology and pin the client to a dead primary forever. An unchanged primary
            // still requires a newer version so a lagging replica cannot overwrite a fresher
            // topology.
            trace_info!(
                target: "milvus_sdk::global_cluster",
                endpoint = %new_primary.endpoint(),
                version = topology.version(),
                "global cluster primary changed; rebuilding channel"
            );
            if self.rebuild_to(new_primary.endpoint()).await {
                self.commit_applied(topology, true);
            }
        } else if version_newer {
            // Same primary: commit the refreshed topology so replica/version changes stay
            // observable, and keep the existing channel.
            self.commit_applied(topology, false);
        } else {
            trace_debug!(
                target: "milvus_sdk::global_cluster",
                version = topology.version(),
                "ignoring stale topology with an unchanged primary; keeping cached topology"
            );
        }
    }

    /// Commits `topology` under the version guard.
    ///
    /// `force` permits committing a changed-primary topology even when its version is not strictly
    /// newer than the cached one, recovering from a version-counter reset. The version guard is
    /// re-applied here (not only in [`Self::apply_topology`]) so a concurrent refresh that commits
    /// a newer topology in between is never overwritten.
    fn commit_applied(&self, topology: GlobalTopology, force: bool) {
        let mut cached = self.topology.write();
        if !force && topology.version() <= cached.version() {
            trace_debug!(
                target: "milvus_sdk::global_cluster",
                version = topology.version(),
                "ignoring stale topology; keeping cached topology"
            );
            return;
        }
        *cached = topology;
    }

    /// Rebuilds the gRPC channel to the given member endpoint, honoring TLS, and swaps it in only
    /// after the endpoint completes a gRPC Connect handshake (the same verification the initial
    /// connection performs). Returns whether the channel was rebuilt; on failure the previous
    /// channel is kept.
    async fn rebuild_to(&self, endpoint: &str) -> bool {
        let endpoint_uri = match cluster_endpoint_uri(endpoint, &self.connect_config) {
            Ok(uri) => uri,
            Err(error) => {
                trace_warn!(target: "milvus_sdk::global_cluster", error = %error, "refused insecure member endpoint; keeping previous primary");
                let _ = error;
                return false;
            }
        };
        let generation = self.service.read().generation.saturating_add(1);
        match build_services(
            &endpoint_uri,
            &self.connect_config,
            &self.database,
            generation,
        )
        .await
        {
            Ok(mut services) => {
                // A TCP connect alone does not prove the endpoint is ready to serve gRPC, so
                // perform the same bounded Connect handshake the initial connection uses before
                // committing the new primary; a plain-HTTP load balancer or still-starting service
                // is rejected instead of being committed and cycling UNAVAILABLE failures.
                if super::wait_for_server(&mut services.milvus, self.connect_config.connect_timeout)
                    .await
                    .is_err()
                {
                    trace_warn!(target: "milvus_sdk::global_cluster",
                        endpoint,
                        "member endpoint did not complete the gRPC Connect handshake; keeping previous primary");
                    return false;
                }
                *self.service.write() = services;
                true
            }
            Err(error) => {
                trace_warn!(target: "milvus_sdk::global_cluster", error = %error, "failed to rebuild channel for new primary; keeping previous primary");
                let _ = error;
                false
            }
        }
    }
}

/// RAII guard that marks an `on_unavailable` probe complete when dropped.
///
/// Dropping the guard (on every exit path, including the fetch-failure early return) clears the
/// in-flight flag and records the probe completion time so the debounce window starts from when the
/// probe actually finished.
struct ProbeGuard<'a> {
    cluster: &'a GlobalCluster,
}

impl<'a> ProbeGuard<'a> {
    fn new(cluster: &'a GlobalCluster) -> Self {
        Self { cluster }
    }
}

impl Drop for ProbeGuard<'_> {
    fn drop(&mut self) {
        self.cluster.finish_probe();
    }
}

impl Drop for GlobalCluster {
    fn drop(&mut self) {
        // Cancel the background refresh task so it does not leak for the process lifetime.
        self.shutdown.cancel();
    }
}

/// Returns the primary endpoint URI for a global endpoint, resolving scheme and TLS.
///
/// When the global connection is TLS-enabled, an explicit `http://` member endpoint from the
/// topology is rejected rather than preserved, so the auth token is never sent over plaintext.
pub(crate) fn cluster_endpoint_uri(
    cluster_endpoint: &str,
    connect_config: &super::ConnectConfig,
) -> Result<String> {
    let tls = super::tls_enabled(connect_config);
    let scheme = if tls { "https" } else { "http" };
    if cluster_endpoint.starts_with("https://") {
        Ok(cluster_endpoint.to_owned())
    } else if cluster_endpoint.starts_with("http://") {
        if tls {
            Err(Error::Unexpected(format!(
                "topology returned an insecure member endpoint {cluster_endpoint:?} for a TLS-enabled global connection"
            )))
        } else {
            Ok(cluster_endpoint.to_owned())
        }
    } else {
        Ok(format!("{scheme}://{cluster_endpoint}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::types::parse_topology_response;
    use std::sync::{Arc as StdArc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A test TCP server serving a mutable topology JSON body.
    struct TopologyServer {
        body: StdArc<Mutex<String>>,
        address: std::net::SocketAddr,
    }

    impl TopologyServer {
        async fn start(initial_body: &str) -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind test listener");
            let address = listener.local_addr().expect("listener address");
            let body = StdArc::new(Mutex::new(initial_body.to_owned()));
            let server_body = StdArc::clone(&body);
            tokio::spawn(async move {
                loop {
                    let (mut socket, _) = match listener.accept().await {
                        Ok(accepted) => accepted,
                        Err(_) => return,
                    };
                    let mut buffer = Vec::new();
                    while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                        let mut chunk = [0u8; 512];
                        let read = socket.read(&mut chunk).await.expect("read request");
                        if read == 0 {
                            break;
                        }
                        buffer.extend_from_slice(&chunk[..read]);
                    }
                    let body = server_body.lock().expect("body lock").clone();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            });
            Self { body, address }
        }

        fn endpoint(&self) -> String {
            format!("http://{}", self.address)
        }

        fn set_body(&self, body: &str) {
            *self.body.lock().expect("body lock") = body.to_owned();
        }
    }

    const PRIMARY_A_BODY: &str = r#"{"code":0,"data":{"version":1,"clusters":[
        {"clusterId":"primary","endpoint":"host-a:19530","capability":3},
        {"clusterId":"replica","endpoint":"host-b:19530","capability":1}
    ]}}"#;

    /// Builds a topology body whose primary points at `primary_endpoint`.
    fn topology_body_with_primary(primary_endpoint: &str, version: i64) -> String {
        format!(
            r#"{{"code":0,"data":{{"version":{version},"clusters":[
                {{"clusterId":"primary","endpoint":"{primary_endpoint}","capability":3}},
                {{"clusterId":"replica","endpoint":"host-a:19530","capability":1}}
            ]}}}}"#
        )
    }

    /// A minimal gRPC server that answers the `Connect` RPC, so the refresh path's
    /// `wait_for_server` handshake check succeeds for the new primary.
    #[derive(Clone, Default)]
    struct ConnectOnlyMilvus;

    #[tonic::async_trait]
    impl crate::proto::milvus::milvus_service_server::MilvusService for ConnectOnlyMilvus {
        async fn connect(
            &self,
            _request: tonic::Request<crate::proto::milvus::ConnectRequest>,
        ) -> std::result::Result<
            tonic::Response<crate::proto::milvus::ConnectResponse>,
            tonic::Status,
        > {
            Ok(tonic::Response::new(
                crate::proto::milvus::ConnectResponse {
                    status: Some(crate::proto::common::Status {
                        code: 0,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ))
        }
    }

    /// Binds a gRPC server that answers the `Connect` RPC, so the refresh path's
    /// `wait_for_server` handshake succeeds for the new primary.
    struct ReachableEndpoint {
        endpoint: String,
        _accept_task: tokio::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>,
    }

    impl ReachableEndpoint {
        async fn start() -> Self {
            use crate::proto::milvus::milvus_service_server::MilvusServiceServer;
            use tokio_stream::wrappers::TcpListenerStream;
            use tonic::transport::Server;
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind reachable endpoint");
            let endpoint = listener.local_addr().expect("listener address").to_string();
            let server = Server::builder()
                .add_service(MilvusServiceServer::new(ConnectOnlyMilvus))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    std::future::pending::<()>().await
                });
            let accept_task = tokio::spawn(server);
            Self {
                endpoint,
                _accept_task: accept_task,
            }
        }
    }

    async fn test_global_cluster(
        server: &TopologyServer,
        initial_body: &str,
    ) -> StdArc<GlobalCluster> {
        let config = super::super::ConnectConfig::new().uri("http://global:19530");
        let database = Arc::new(RwLock::new("default".to_owned()));
        let topology = parse_topology_response(initial_body).expect("initial topology");
        let primary = topology
            .primary()
            .expect("initial primary")
            .endpoint()
            .to_owned();
        let service = Arc::new(RwLock::new(
            build_services(
                &cluster_endpoint_uri(&primary, &config).expect("initial primary uri"),
                &config,
                &database,
                0,
            )
            .await
            .expect("build initial service"),
        ));
        StdArc::new(GlobalCluster::new(
            server.endpoint(),
            config,
            database,
            service,
            topology,
        ))
    }

    #[test]
    fn cluster_endpoint_uri_resolves_scheme() {
        let plain = super::super::ConnectConfig::new().uri("http://global:19530");
        assert_eq!(
            cluster_endpoint_uri("host-a:19530", &plain).expect("plain uri"),
            "http://host-a:19530"
        );
        assert_eq!(
            cluster_endpoint_uri("http://host-a:19530", &plain).expect("explicit plain uri"),
            "http://host-a:19530"
        );
        let tls = super::super::ConnectConfig::new().uri("https://global:19530");
        assert_eq!(
            cluster_endpoint_uri("host-a:19530", &tls).expect("tls uri"),
            "https://host-a:19530"
        );
        assert_eq!(
            cluster_endpoint_uri("https://host-a:19530", &tls).expect("explicit tls uri"),
            "https://host-a:19530"
        );
    }

    #[test]
    fn cluster_endpoint_uri_rejects_insecure_member_endpoints_when_tls_is_enabled() {
        let config = super::super::ConnectConfig::new().uri("https://global:19530");
        let error = cluster_endpoint_uri("http://host-a:19530", &config)
            .expect_err("an explicit http member endpoint must be rejected on TLS");
        assert!(
            error.to_string().contains("insecure member endpoint"),
            "unexpected error: {error}"
        );

        let tls_server_name = super::super::ConnectConfig::new()
            .uri("http://global:19530")
            .tls_server_name("global.example.com");
        assert!(cluster_endpoint_uri("http://host-a:19530", &tls_server_name).is_err());
    }

    #[tokio::test]
    async fn fetch_topology_reads_the_rest_endpoint() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let expected_auth = "Bearer test-token";
        let expected_auth = std::sync::Arc::new(expected_auth.to_owned());
        let server_auth = std::sync::Arc::clone(&expected_auth);
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept connection");
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buffer = Vec::new();
            // Read until the HTTP header block is fully received.
            while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                let mut chunk = [0u8; 512];
                let read = socket.read(&mut chunk).await.expect("read request");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            let request = String::from_utf8_lossy(&buffer).into_owned();
            let has_auth = request
                .to_lowercase()
                .contains(&format!("authorization: {}", server_auth.to_lowercase()));
            let body = r#"{"code":0,"data":{"version":3,"clusters":[
                {"clusterId":"primary","endpoint":"host-a:19530","capability":3},
                {"clusterId":"replica","endpoint":"host-b:19530","capability":1}
            ]}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
            has_auth
        });

        let endpoint = format!("http://{address}");
        let config = super::super::ConnectConfig::new().token("test-token");
        let topology = fetch_topology(&endpoint, &config)
            .await
            .expect("fetch topology");
        assert_eq!(topology.version(), 3);
        assert_eq!(topology.clusters().len(), 2);
        let primary = topology.primary().expect("primary cluster");
        assert_eq!(primary.endpoint(), "host-a:19530");
        assert!(handle.await.expect("server task"), "auth header was sent");
    }

    #[tokio::test]
    async fn fetch_topology_surfaces_a_non_zero_response_code() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let handle = tokio::spawn(async move {
            // Serve a fixed error response for every retry attempt. The request must be fully
            // consumed before responding so the connection closes cleanly on every platform;
            // responding without reading it can reset the connection on Windows.
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(accepted) => accepted,
                    Err(_) => return,
                };
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buffer = Vec::new();
                while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    let mut chunk = [0u8; 512];
                    let read = socket.read(&mut chunk).await.expect("read request");
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                }
                let body = r#"{"code":7,"message":"forbidden","data":null}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        let endpoint = format!("http://{address}");
        let config = super::super::ConnectConfig::new();
        let error = fetch_topology(&endpoint, &config)
            .await
            .expect_err("non-zero topology code must fail");
        assert!(error.to_string().contains("forbidden"));
        // The server task loops to serve every retry attempt; detach it.
        drop(handle);
    }

    #[tokio::test]
    async fn fetch_topology_returns_immediately_on_client_errors() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let server_requests = std::sync::Arc::clone(&requests);
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept connection");
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buffer = Vec::new();
            while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                let mut chunk = [0u8; 512];
                let read = socket.read(&mut chunk).await.expect("read request");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            server_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = socket.write_all(response.as_bytes()).await;
        });

        let endpoint = format!("http://{address}");
        let config = super::super::ConnectConfig::new();
        let error = fetch_topology(&endpoint, &config)
            .await
            .expect_err("a 404 must fail");
        assert!(
            error.to_string().contains("404"),
            "unexpected error: {error}"
        );
        assert_eq!(
            requests.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "a client error must not be retried"
        );
        handle.await.expect("server task");
    }

    #[tokio::test]
    async fn fetch_topology_retries_server_errors() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let server_requests = std::sync::Arc::clone(&requests);
        let handle = tokio::spawn(async move {
            // Serve a 503 for every retry attempt.
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(accepted) => accepted,
                    Err(_) => return,
                };
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buffer = Vec::new();
                while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    let mut chunk = [0u8; 512];
                    let read = socket.read(&mut chunk).await.expect("read request");
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                }
                server_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let response = "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n";
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        let endpoint = format!("http://{address}");
        let config = super::super::ConnectConfig::new();
        let error = fetch_topology(&endpoint, &config)
            .await
            .expect_err("a 503 must fail");
        assert!(
            error.to_string().contains("503"),
            "unexpected error: {error}"
        );
        assert_eq!(
            requests.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "a server error must be retried up to MAX_FETCH_ATTEMPTS"
        );
        drop(handle);
    }

    #[tokio::test]
    async fn fetch_topology_returns_immediately_on_a_non_zero_code_body() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let server_requests = std::sync::Arc::clone(&requests);
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept connection");
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buffer = Vec::new();
            while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                let mut chunk = [0u8; 512];
                let read = socket.read(&mut chunk).await.expect("read request");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            server_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let body = r#"{"code":7,"message":"forbidden","data":null}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        });

        let endpoint = format!("http://{address}");
        let config = super::super::ConnectConfig::new();
        let error = fetch_topology(&endpoint, &config)
            .await
            .expect_err("a non-zero code must fail");
        assert!(
            error.to_string().contains("forbidden"),
            "unexpected error: {error}"
        );
        assert_eq!(
            requests.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "a permanent business error must not be retried"
        );
        handle.await.expect("server task");
    }

    #[tokio::test]
    async fn fetch_topology_is_bounded_by_connect_timeout() {
        use std::time::Duration;

        // A server that never responds: the fetch must abort once connect_timeout elapses instead
        // of running the full 3-attempt + backoff sequence.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept connection");
            // Accept and then stall without ever completing the HTTP response.
            let _ = &mut socket;
            tokio::time::sleep(Duration::from_secs(60)).await;
        });

        let endpoint = format!("http://{address}");
        let mut config = super::super::ConnectConfig::new();
        config.set_connect_timeout(Duration::from_millis(200));

        let started = std::time::Instant::now();
        let error = fetch_topology(&endpoint, &config)
            .await
            .expect_err("an unresponsive endpoint must time out");
        let elapsed = started.elapsed();

        assert!(
            matches!(error, Error::Timeout(_)),
            "unexpected error: {error}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "discovery must be capped by connect_timeout, took {elapsed:?}"
        );
        drop(handle);
    }

    #[tokio::test]
    async fn background_refresh_commits_only_newer_topology_versions() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // A newer version is committed even when the primary endpoint is unchanged.
        let replica_swap = r#"{"code":0,"data":{"version":3,"clusters":[
            {"clusterId":"primary","endpoint":"host-a:19530","capability":3},
            {"clusterId":"replica","endpoint":"host-c:19530","capability":1}
        ]}}"#;
        server.set_body(replica_swap);
        global.refresh().await;
        assert_eq!(
            global.topology().version(),
            3,
            "version bump must be committed"
        );
        assert_eq!(
            global.topology().clusters()[1].endpoint(),
            "host-c:19530",
            "replica change must be committed"
        );

        // An older version must not overwrite the cached topology.
        server.set_body(PRIMARY_A_BODY);
        global.refresh().await;
        assert_eq!(
            global.topology().version(),
            3,
            "a stale topology must not overwrite a newer one"
        );
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530"
        );
    }

    #[tokio::test]
    async fn background_refresh_proactively_fails_over_to_a_reachable_new_primary() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530"
        );

        let reachable = ReachableEndpoint::start().await;
        server.set_body(&topology_body_with_primary(&reachable.endpoint, 2));
        global.refresh().await;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            reachable.endpoint,
            "background refresh must proactively fail over to a reachable new primary"
        );
        assert_eq!(global.topology().version(), 2);
    }

    #[tokio::test]
    async fn background_refresh_keeps_the_previous_primary_when_the_new_primary_is_unreachable() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // Reserve a port and release it so nothing listens there: the reachability probe fails and
        // the previous primary must be kept instead of committing a dead endpoint.
        let dead_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind dead endpoint");
        let dead_address = dead_listener.local_addr().expect("dead address");
        drop(dead_listener);

        server.set_body(&topology_body_with_primary(&dead_address.to_string(), 2));
        global.refresh().await;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530",
            "an unreachable new primary must not be committed"
        );
    }

    #[tokio::test]
    async fn start_refresh_task_exits_when_the_cluster_is_dropped() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;
        global.start_refresh();

        // Dropping the last strong reference cancels the background task via the Drop impl; the
        // spawned task must exit rather than keep the cluster (and its resources) alive.
        drop(global);
        tokio::task::yield_now().await;
    }

    #[tokio::test]
    async fn on_unavailable_rebuilds_the_channel_and_commits_a_changed_primary() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530"
        );

        let reachable = ReachableEndpoint::start().await;
        server.set_body(&topology_body_with_primary(&reachable.endpoint, 2));
        global.on_unavailable().await;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            reachable.endpoint,
            "a reachable new primary discovered on UNAVAILABLE must be committed"
        );
        assert_eq!(global.topology().version(), 2);
    }

    #[tokio::test]
    async fn on_unavailable_keeps_the_previous_primary_when_the_new_primary_is_unreachable() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // Reserve a port and release it so nothing listens there: the reachability probe fails and
        // the previous primary must be kept instead of committing a dead endpoint.
        let dead_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind dead endpoint");
        let dead_address = dead_listener.local_addr().expect("dead address");
        drop(dead_listener);

        server.set_body(&topology_body_with_primary(&dead_address.to_string(), 5));
        global.on_unavailable().await;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530",
            "an unreachable new primary must not be committed"
        );
    }

    #[tokio::test]
    async fn on_unavailable_keeps_the_previous_primary_when_the_rebuild_fails() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // A primary endpoint with a space is not a valid URI, so build_service/Endpoint::from_shared
        // fails deterministically without needing a real reachable gRPC server.
        let bad_primary = r#"{"code":0,"data":{"version":4,"clusters":[
            {"clusterId":"primary","endpoint":"bad host:19530","capability":3}
        ]}}"#;
        server.set_body(bad_primary);
        global.on_unavailable().await;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530",
            "a failed rebuild must keep the previous primary"
        );
    }

    #[tokio::test]
    async fn on_unavailable_keeps_the_cached_topology_when_the_fetch_fails() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // A non-zero code makes fetch_topology fail without retrying, so on_unavailable must keep
        // the cached topology and current channel.
        server.set_body(r#"{"code":7,"message":"boom","data":null}"#);
        global.on_unavailable().await;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530",
            "a failed topology fetch must keep the cached topology"
        );
        assert_eq!(global.topology().version(), 1);
    }

    #[tokio::test]
    async fn on_unavailable_probes_are_debounced_within_the_window() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // The first on_unavailable triggers a real fetch; a second call within the debounce window
        // must skip the fetch entirely and keep the cached topology.
        let reachable = ReachableEndpoint::start().await;
        server.set_body(&topology_body_with_primary(&reachable.endpoint, 2));
        global.on_unavailable().await;
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            reachable.endpoint,
            "the first probe must apply the new topology"
        );

        server.set_body(&topology_body_with_primary(&reachable.endpoint, 3));
        global.on_unavailable().await;
        assert_eq!(
            global.topology().version(),
            2,
            "a probe inside the debounce window must not re-fetch"
        );
    }

    #[tokio::test]
    async fn on_unavailable_does_not_downgrade_a_newer_cached_topology() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // A background refresh commits a newer topology pointing at a reachable new primary.
        let newer = ReachableEndpoint::start().await;
        server.set_body(&topology_body_with_primary(&newer.endpoint, 3));
        global.refresh().await;
        assert_eq!(global.topology().version(), 3);

        // A lagging REST replica then serves an older topology on UNAVAILABLE: it must not revert
        // the cached topology or fail the client back to the previous primary.
        server.set_body(&topology_body_with_primary("host-a:19530", 2));
        global.on_unavailable().await;

        assert_eq!(
            global.topology().version(),
            3,
            "an older on_unavailable fetch must not downgrade the cached topology"
        );
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            newer.endpoint,
            "an older on_unavailable fetch must not fail back to a previous primary"
        );
    }

    #[tokio::test]
    async fn overlapping_topology_applications_do_not_install_a_stale_primary_channel() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // Two concurrent apply_topology calls: one for v2 primary Y, one for v3 primary Z. The
        // apply_lock serializes them, so whichever runs second wins and the slower rebuild cannot
        // install the older primary's channel after the newer topology is committed.
        let primary_y = ReachableEndpoint::start().await;
        let primary_z = ReachableEndpoint::start().await;
        let topology_y =
            parse_topology_response(&topology_body_with_primary(&primary_y.endpoint, 2))
                .expect("topology v2");
        let topology_z =
            parse_topology_response(&topology_body_with_primary(&primary_z.endpoint, 3))
                .expect("topology v3");

        let (first, second) = tokio::join!(
            global.apply_topology(topology_y),
            global.apply_topology(topology_z)
        );
        let _ = (first, second);

        assert_eq!(
            global.topology().version(),
            3,
            "the newer topology must win"
        );
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            primary_z.endpoint,
            "the stored primary must match the newest applied topology"
        );
    }

    #[tokio::test]
    async fn a_changed_primary_fails_over_even_after_a_version_counter_reset() {
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;

        // A background refresh first commits a newer topology pointing at a reachable primary.
        let primary_v3 = ReachableEndpoint::start().await;
        server.set_body(&topology_body_with_primary(&primary_v3.endpoint, 3));
        global.refresh().await;
        assert_eq!(global.topology().version(), 3);

        // The global REST service restarts and resets its version counter to 1 while advertising a
        // different reachable primary. A strict `>` guard would discard this forever and pin the
        // client to the old primary; the changed primary must still trigger failover.
        let primary_reset = ReachableEndpoint::start().await;
        server.set_body(&topology_body_with_primary(&primary_reset.endpoint, 1));
        global.refresh().await;

        assert_eq!(
            global.topology().version(),
            1,
            "a version reset with a changed primary must still be applied"
        );
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            primary_reset.endpoint,
            "the client must fail over to the new primary after a version reset"
        );
    }

    #[tokio::test]
    async fn fetch_topology_retries_a_200_response_with_an_unparseable_body() {
        // A 200 response whose body cannot be parsed is transient (a proxy may cut the response
        // during a backend switch), so it must be retried with backoff rather than failing the
        // caller immediately: serve an unparseable body first, then a valid one.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("listener address");
        let requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let server_requests = std::sync::Arc::clone(&requests);
        let handle = tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            for _ in 0..2 {
                let (mut socket, _) = listener.accept().await.expect("accept connection");
                let mut buffer = Vec::new();
                while !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    let mut chunk = [0u8; 512];
                    let read = socket.read(&mut chunk).await.expect("read request");
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                }
                let count = server_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let body = if count == 0 {
                    // Truncated JSON: a valid prefix with the body cut mid-token.
                    r#"{"code":0,"data":{"version":1,"clusters":["#
                } else {
                    r#"{"code":0,"data":{"version":2,"clusters":[
                        {"clusterId":"primary","endpoint":"host-a:19530","capability":3}
                    ]}}"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        let endpoint = format!("http://{address}");
        let config = super::super::ConnectConfig::new();
        let topology = fetch_topology(&endpoint, &config)
            .await
            .expect("a transient unparseable 200 body must be retried");
        assert_eq!(topology.version(), 2);
        assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 2);
        handle.await.expect("server task");
    }

    #[tokio::test]
    async fn on_unavailable_does_not_rebuild_to_a_stale_primary_committed_during_the_fetch() {
        // The recovery path re-validates the captured `old_primary` after acquiring `apply_lock`:
        // if a concurrent background refresh committed a newer primary while the failed fetch was
        // in flight, the recovery must not rebuild the channel back to the stale snapshot.
        let server = TopologyServer::start(PRIMARY_A_BODY).await;
        let global = test_global_cluster(&server, PRIMARY_A_BODY).await;
        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            "host-a:19530"
        );

        // A concurrent refresh commits a newer primary pointing at a reachable endpoint.
        let newer = ReachableEndpoint::start().await;
        let newer_topology =
            parse_topology_response(&topology_body_with_primary(&newer.endpoint, 2))
                .expect("newer topology");

        // The topology fetch fails, but only after the concurrent refresh has already swapped the
        // channel to the newer primary. Capture the stale primary first so the recovery path sees
        // a snapshot that no longer matches the cached primary by the time it rebuilds.
        let stale_primary = global
            .topology()
            .primary()
            .expect("stale primary")
            .endpoint()
            .to_owned();

        // Simulate a background refresh committing a newer topology while `on_unavailable`'s fetch
        // is in flight by applying it directly, then make the REST endpoint serve a failing body so
        // the fetch fails and the recovery path runs.
        server.set_body(r#"{"code":7,"message":"boom","data":null}"#);
        let (_, on_unavailable) = tokio::join!(
            global.apply_topology(newer_topology),
            global.on_unavailable()
        );
        let _ = on_unavailable;

        assert_eq!(
            global.topology().primary().expect("primary").endpoint(),
            newer.endpoint,
            "a concurrent newer primary must not be reverted to the stale snapshot"
        );
        assert!(
            stale_primary != newer.endpoint,
            "test setup: the newer primary must differ from the stale snapshot"
        );
        assert_eq!(global.topology().version(), 2);
    }
}
