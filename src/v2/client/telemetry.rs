// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. See the License for the
// specific language governing permissions and limitations
// under the License.

//! Client-side operation telemetry, heartbeat delivery, and server commands.

use super::SharedServices;
use crate::proto::{common, milvus};
use crate::v2::error::status_to_result;
use crate::v2::types::{ConnectConfig, TelemetryConfig};
use chrono::{DateTime, Utc};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;
use tonic::{Code, Request};
use uuid::Uuid;

const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_UNSUPPORTED_BACKOFF: Duration = Duration::from_secs(30 * 60);
const LATENCY_SAMPLE_CAPACITY: usize = 1000;
const MAX_REPLY_PAYLOAD_SIZE: usize = 1024 * 1024;
const HISTORY_RETENTION: Duration = Duration::from_secs(60 * 60);
const MAX_HISTORY_SNAPSHOTS: usize = 3600;
const SAMPLING_SCALE: u64 = 1_000_000_000;

tokio::task_local! {
    static CLIENT_REQUEST_ID: String;
}

/// Runs a future with a caller-provided `client_request_id`.
///
/// The ID must be a non-zero, 32-character lowercase OpenTelemetry TraceID.
/// Malformed values are omitted from both gRPC metadata and telemetry error
/// correlation because the server would reject them as trace identifiers.
pub async fn with_client_request_id<F>(request_id: impl Into<String>, future: F) -> F::Output
where
    F: Future,
{
    CLIENT_REQUEST_ID.scope(request_id.into(), future).await
}

/// Generates a non-zero lowercase 32-character OpenTelemetry TraceID.
pub fn new_client_request_id() -> String {
    Uuid::new_v4().simple().to_string()
}

pub(super) fn current_client_request_id() -> Option<String> {
    CLIENT_REQUEST_ID
        .try_with(Clone::clone)
        .ok()
        .filter(|value| valid_trace_id(value))
}

fn valid_trace_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && value.bytes().any(|byte| byte != b'0')
}

fn current_telemetry_trace_id() -> String {
    current_client_request_id().unwrap_or_default()
}

fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Aggregated telemetry for one operation or collection.
pub struct TelemetryMetrics {
    /// Number of completed logical calls.
    pub request_count: i64,
    /// Number of successful logical calls.
    pub success_count: i64,
    /// Number of failed logical calls.
    pub error_count: i64,
    /// Mean end-to-end latency in milliseconds.
    pub avg_latency_ms: f64,
    /// P99 end-to-end latency in milliseconds.
    pub p99_latency_ms: f64,
    /// Maximum end-to-end latency in milliseconds.
    pub max_latency_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
/// Metrics for one logical SDK operation.
pub struct TelemetryOperationMetrics {
    /// Canonical operation name.
    pub operation: String,
    /// Global metrics for the operation.
    pub global: TelemetryMetrics,
    /// Optional per-collection metrics.
    pub collections: BTreeMap<String, TelemetryMetrics>,
}

#[derive(Debug, Clone, PartialEq)]
/// One telemetry metrics window.
pub struct TelemetrySnapshot {
    /// Inclusive window start in Unix milliseconds.
    pub timestamp: i64,
    /// Inclusive window end in Unix milliseconds.
    pub end_time: i64,
    /// Operation metrics captured in the window.
    pub metrics: Vec<TelemetryOperationMetrics>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// A recent client-side operation error.
pub struct TelemetryErrorInfo {
    /// Unix timestamp in milliseconds.
    pub timestamp: i64,
    /// Canonical operation name.
    pub operation: String,
    /// Error text.
    #[serde(rename = "error_msg")]
    pub error_message: String,
    /// Collection associated with the call.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub collection: String,
    /// Valid OpenTelemetry TraceID associated with the call.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq)]
/// A server command in the client-side command registry.
pub struct ClientTelemetryCommand {
    /// Server-assigned command identifier.
    pub command_id: String,
    /// Free-form command type.
    pub command_type: String,
    /// Raw JSON payload.
    pub payload: Vec<u8>,
    /// Server creation timestamp in Unix milliseconds.
    pub create_time: i64,
    /// Whether the command is a persistent configuration.
    pub persistent: bool,
    /// Server-computed targeting scope.
    pub target_scope: String,
}

#[derive(Debug, Clone, PartialEq)]
/// Reply produced by a client telemetry command handler.
pub struct ClientTelemetryCommandReply {
    /// Command identifier being answered.
    pub command_id: String,
    /// Whether handling succeeded.
    pub success: bool,
    /// Error text when handling failed.
    pub error_message: String,
    /// Raw JSON reply payload.
    pub payload: Vec<u8>,
}

impl ClientTelemetryCommandReply {
    fn success(command_id: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            command_id: command_id.into(),
            success: true,
            error_message: String::new(),
            payload,
        }
    }

    fn failure(command_id: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            command_id: command_id.into(),
            success: false,
            error_message: error_message.into(),
            payload: Vec::new(),
        }
    }
}

#[derive(Default)]
struct MetricBucket {
    request_count: i64,
    success_count: i64,
    error_count: i64,
    total_latency_us: i128,
    max_latency_us: i64,
    samples: VecDeque<i64>,
}

impl MetricBucket {
    fn record(&mut self, latency_us: i64, success: bool) {
        self.request_count += 1;
        self.success_count += i64::from(success);
        self.error_count += i64::from(!success);
        self.total_latency_us += i128::from(latency_us);
        self.max_latency_us = self.max_latency_us.max(latency_us);
        if self.samples.len() == LATENCY_SAMPLE_CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back(latency_us);
    }

    fn take(&mut self) -> Option<TelemetryMetrics> {
        if self.request_count == 0 {
            return None;
        }
        let mut samples: Vec<_> = self.samples.iter().copied().collect();
        samples.sort_unstable();
        let p99_index = ((samples.len() as f64) * 0.99) as usize;
        let p99_us = samples[p99_index.min(samples.len() - 1)];
        let metrics = TelemetryMetrics {
            request_count: self.request_count,
            success_count: self.success_count,
            error_count: self.error_count,
            avg_latency_ms: self.total_latency_us as f64 / self.request_count as f64 / 1000.0,
            p99_latency_ms: p99_us as f64 / 1000.0,
            max_latency_ms: self.max_latency_us as f64 / 1000.0,
        };
        *self = Self::default();
        Some(metrics)
    }
}

#[derive(Default)]
struct OperationCollector {
    global: MetricBucket,
    collections: BTreeMap<String, MetricBucket>,
}

impl OperationCollector {
    fn record(&mut self, collection: Option<&str>, latency_us: i64, success: bool) {
        self.global.record(latency_us, success);
        if let Some(collection) = collection {
            self.collections
                .entry(collection.to_owned())
                .or_default()
                .record(latency_us, success);
        }
    }

    fn take(&mut self, operation: String) -> Option<TelemetryOperationMetrics> {
        let global = self.global.take()?;
        let mut collections = BTreeMap::new();
        for (name, bucket) in &mut self.collections {
            if let Some(metrics) = bucket.take() {
                collections.insert(name.clone(), metrics);
            }
        }
        self.collections
            .retain(|_, bucket| bucket.request_count != 0);
        Some(TelemetryOperationMetrics {
            operation,
            global,
            collections,
        })
    }
}

type CommandHandler = dyn Fn(&ClientTelemetryCommand) -> ClientTelemetryCommandReply + Send + Sync;

struct RuntimeState {
    config: TelemetryConfig,
    collectors: BTreeMap<String, OperationCollector>,
    snapshots: VecDeque<TelemetrySnapshot>,
    last_snapshot_end: i64,
    errors: VecDeque<TelemetryErrorInfo>,
    enabled_collections: BTreeSet<String>,
    all_collections_enabled: bool,
    pending_replies: VecDeque<common::CommandReply>,
    config_hash: String,
    last_command_timestamp: i64,
    executed_commands: HashMap<String, i64>,
    sampling_accumulator: u64,
}

impl RuntimeState {
    fn new(mut config: TelemetryConfig) -> Self {
        if config.heartbeat_interval.is_zero() {
            config.heartbeat_interval = DEFAULT_HEARTBEAT_INTERVAL;
        }
        if !config.sampling_rate.is_finite() {
            config.sampling_rate = 0.0;
        }
        config.sampling_rate = config.sampling_rate.clamp(0.0, 1.0);
        if config.error_max_count == 0 {
            config.error_max_count = 100;
        }
        Self {
            config,
            collectors: BTreeMap::new(),
            snapshots: VecDeque::new(),
            last_snapshot_end: 0,
            errors: VecDeque::new(),
            enabled_collections: BTreeSet::new(),
            all_collections_enabled: false,
            pending_replies: VecDeque::new(),
            config_hash: String::new(),
            last_command_timestamp: 0,
            executed_commands: HashMap::new(),
            sampling_accumulator: 0,
        }
    }

    fn should_sample(&mut self) -> bool {
        let rate = self.config.sampling_rate;
        if rate >= 1.0 {
            return true;
        }
        if rate <= 0.0 {
            return false;
        }
        let step = ((rate * SAMPLING_SCALE as f64) as u64).max(1);
        self.sampling_accumulator += step;
        if self.sampling_accumulator >= SAMPLING_SCALE {
            self.sampling_accumulator -= SAMPLING_SCALE;
            true
        } else {
            false
        }
    }
}

struct ClientConfigSnapshot {
    uri: String,
    username: String,
    initial_database: String,
    tls_enabled: bool,
    retry_max_attempts: u32,
    retry_max_backoff_ms: u64,
}

struct TelemetryInner {
    state: Mutex<RuntimeState>,
    command_handlers: RwLock<HashMap<String, Arc<CommandHandler>>>,
    command_lock: Mutex<()>,
    services: SharedServices,
    database: Arc<RwLock<String>>,
    database_explicit: Arc<AtomicBool>,
    client_config: ClientConfigSnapshot,
    client_id: String,
    client_id_stable: bool,
    unsupported_streak: AtomicU64,
    last_heartbeat_error: RwLock<Option<String>>,
    shutdown: CancellationToken,
}

impl Drop for TelemetryInner {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

/// Shared client-side telemetry manager.
#[derive(Clone)]
pub struct ClientTelemetry {
    inner: Arc<TelemetryInner>,
}

impl ClientTelemetry {
    pub(super) fn new(
        config: TelemetryConfig,
        services: SharedServices,
        database: Arc<RwLock<String>>,
        database_explicit: Arc<AtomicBool>,
        connect: &ConnectConfig,
    ) -> Self {
        let client_id_stable = !config.client_id.is_empty();
        let client_id = if client_id_stable {
            config.client_id.clone()
        } else {
            Uuid::new_v4().to_string()
        };
        let username = connect
            .raw_token()
            .and_then(|token| token.split_once(':').map(|(user, _)| user.to_owned()))
            .unwrap_or_default();
        let inner = TelemetryInner {
            state: Mutex::new(RuntimeState::new(config)),
            command_handlers: RwLock::new(HashMap::new()),
            command_lock: Mutex::new(()),
            services,
            database,
            database_explicit,
            client_config: ClientConfigSnapshot {
                uri: connect.uri.clone(),
                username,
                initial_database: connect.database.clone(),
                tls_enabled: super::tls_enabled(connect),
                retry_max_attempts: connect.retry.max_attempts,
                retry_max_backoff_ms: connect.retry.max_backoff.as_millis().min(u64::MAX as u128)
                    as u64,
            },
            client_id,
            client_id_stable,
            unsupported_streak: AtomicU64::new(0),
            last_heartbeat_error: RwLock::new(None),
            shutdown: CancellationToken::new(),
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Returns the stable runtime client identifier sent in heartbeats.
    pub fn client_id(&self) -> &str {
        &self.inner.client_id
    }

    /// Returns the current effective telemetry configuration.
    pub fn config(&self) -> TelemetryConfig {
        self.inner.state.lock().config.clone()
    }

    /// Reports whether the server is not currently known to reject telemetry as unimplemented.
    pub fn is_supported(&self) -> bool {
        self.inner.unsupported_streak.load(Ordering::Relaxed) == 0
    }

    /// Returns the most recent best-effort heartbeat failure.
    pub fn last_heartbeat_error(&self) -> Option<String> {
        self.inner.last_heartbeat_error.read().clone()
    }

    /// Returns recent operation errors, newest first.
    pub fn recent_errors(&self, max_count: usize) -> Vec<TelemetryErrorInfo> {
        self.inner
            .state
            .lock()
            .errors
            .iter()
            .rev()
            .take(max_count)
            .cloned()
            .collect()
    }

    /// Returns retained metric snapshots in chronological order.
    pub fn snapshots(&self) -> Vec<TelemetrySnapshot> {
        self.inner.state.lock().snapshots.iter().cloned().collect()
    }

    /// Registers or replaces a custom command handler.
    pub fn register_command_handler<F>(&self, command_type: impl Into<String>, handler: F)
    where
        F: Fn(&ClientTelemetryCommand) -> ClientTelemetryCommandReply + Send + Sync + 'static,
    {
        self.inner
            .command_handlers
            .write()
            .insert(command_type.into(), Arc::new(handler));
    }

    pub(super) fn start(&self) {
        if !self.inner.state.lock().config.enabled {
            return;
        }
        let weak = Arc::downgrade(&self.inner);
        tokio::spawn(async move { heartbeat_loop(weak).await });
    }

    pub(super) fn begin_operation(
        &self,
        operation: &'static str,
        collection: impl Into<String>,
    ) -> TelemetryOperationGuard {
        TelemetryOperationGuard {
            telemetry: self.clone(),
            operation,
            collection: collection.into(),
            request_id: current_telemetry_trace_id(),
            started: Instant::now(),
        }
    }
}

pub(super) struct TelemetryOperationGuard {
    telemetry: ClientTelemetry,
    operation: &'static str,
    collection: String,
    request_id: String,
    started: Instant,
}

impl TelemetryOperationGuard {
    pub(super) fn finish<T>(&self, result: &crate::v2::error::Result<T>) {
        self.telemetry.inner.record_operation(
            self.operation,
            &self.collection,
            &self.request_id,
            self.started.elapsed(),
            result.as_ref().err().map(ToString::to_string),
        );
    }
}

impl TelemetryInner {
    fn record_operation(
        &self,
        operation: &str,
        collection: &str,
        request_id: &str,
        latency: Duration,
        error: Option<String>,
    ) {
        let mut state = self.state.lock();
        if !state.config.enabled || !state.should_sample() {
            return;
        }
        let collection_enabled = !collection.is_empty()
            && (state.all_collections_enabled || state.enabled_collections.contains(collection));
        let latency_us = latency.as_micros().min(i64::MAX as u128) as i64;
        state
            .collectors
            .entry(operation.to_owned())
            .or_default()
            .record(
                collection_enabled.then_some(collection),
                latency_us,
                error.is_none(),
            );
        if let Some(error_message) = error {
            let capacity = state.config.error_max_count.max(1);
            if state.errors.len() == capacity {
                state.errors.pop_front();
            }
            state.errors.push_back(TelemetryErrorInfo {
                timestamp: now_millis(),
                operation: operation.to_owned(),
                error_message,
                collection: collection.to_owned(),
                request_id: request_id.to_owned(),
            });
        }
    }

    fn create_snapshot(&self) {
        let mut state = self.state.lock();
        if !state.config.enabled {
            return;
        }
        let now = now_millis();
        let interval_ms = state
            .config
            .heartbeat_interval
            .as_millis()
            .min(i64::MAX as u128) as i64;
        let start = if state.last_snapshot_end == 0 || state.last_snapshot_end > now {
            now.saturating_sub(interval_ms)
        } else {
            state.last_snapshot_end
        };
        state.last_snapshot_end = now;

        let mut metrics = Vec::new();
        for (operation, collector) in &mut state.collectors {
            if let Some(metric) = collector.take(operation.clone()) {
                metrics.push(metric);
            }
        }
        state.snapshots.push_back(TelemetrySnapshot {
            timestamp: start,
            end_time: now,
            metrics,
        });

        let oldest = now.saturating_sub(HISTORY_RETENTION.as_millis() as i64);
        while state
            .snapshots
            .front()
            .is_some_and(|snapshot| snapshot.end_time < oldest)
        {
            state.snapshots.pop_front();
        }
        while state.snapshots.len() > MAX_HISTORY_SNAPSHOTS {
            state.snapshots.pop_front();
        }
    }

    fn next_heartbeat_delay(&self) -> Duration {
        let interval = self.state.lock().config.heartbeat_interval;
        let streak = self.unsupported_streak.load(Ordering::Relaxed);
        if streak == 0 {
            return interval;
        }
        let mut backoff = interval;
        for _ in 0..streak {
            backoff = backoff.saturating_mul(2);
            if backoff >= MAX_UNSUPPORTED_BACKOFF {
                backoff = MAX_UNSUPPORTED_BACKOFF;
                break;
            }
        }
        backoff.max(interval)
    }

    fn build_client_info(&self) -> common::ClientInfo {
        let host = hostname::get()
            .ok()
            .and_then(|host| host.into_string().ok())
            .unwrap_or_default();
        let mut reserved = HashMap::from([
            ("client_id".to_owned(), self.client_id.clone()),
            (
                "client_id_stable".to_owned(),
                self.client_id_stable.to_string(),
            ),
        ]);
        let database = self.database.read().clone();
        // ClientV2 normalizes an omitted database to `default` internally. Track whether
        // the caller actually selected it so omitted and explicit `default` remain distinct.
        if !database.is_empty()
            && (database != "default" || self.database_explicit.load(Ordering::Acquire))
        {
            reserved.insert("db_name".to_owned(), database);
        }
        common::ClientInfo {
            sdk_type: "RustMilvusClient".to_owned(),
            sdk_version: env!("CARGO_PKG_VERSION").to_owned(),
            local_time: DateTime::<Utc>::from(SystemTime::now()).to_rfc3339(),
            user: self.client_config.username.clone(),
            host,
            reserved,
        }
    }

    fn heartbeat_request(&self) -> milvus::ClientHeartbeatRequest {
        let state = self.state.lock();
        let enabled_collections = &state.enabled_collections;
        let all_collections_enabled = state.all_collections_enabled;
        let metrics = if state.config.enabled {
            state
                .snapshots
                .back()
                .map(|snapshot| {
                    snapshot
                        .metrics
                        .iter()
                        .map(|operation| common::OperationMetrics {
                            operation: operation.operation.clone(),
                            global: Some(metrics_to_proto(&operation.global)),
                            collection_metrics: operation
                                .collections
                                .iter()
                                .filter(|(name, _)| {
                                    all_collections_enabled || enabled_collections.contains(*name)
                                })
                                .map(|(name, metrics)| (name.clone(), metrics_to_proto(metrics)))
                                .collect(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        milvus::ClientHeartbeatRequest {
            client_info: Some(self.build_client_info()),
            report_timestamp: now_millis(),
            metrics,
            command_replies: state.pending_replies.iter().cloned().collect(),
            config_hash: state.config_hash.clone(),
            last_command_timestamp: state.last_command_timestamp,
        }
    }

    async fn send_heartbeat(&self) {
        let request = self.heartbeat_request();
        let reply_count = request.command_replies.len();
        let (mut service, generation) = {
            let bundle = self.services.read();
            (bundle.telemetry.clone(), bundle.generation)
        };
        let mut request = Request::new(request);
        request.set_timeout(HEARTBEAT_TIMEOUT);
        let outcome =
            tokio::time::timeout(HEARTBEAT_TIMEOUT, service.client_heartbeat(request)).await;

        // A global-cluster failover may replace both stubs while this RPC is in flight.
        // Every result from the retired generation is discarded, including errors, so it
        // cannot clear replies, alter backoff, or execute stale commands.
        // Hold the generation read lock through the internal acceptance commit. A failover
        // that was already published makes the response stale; one that publishes after this
        // point is ordered after the accepted response and must not retroactively invalidate it.
        let generation_guard = self.services.read();
        if generation_guard.generation != generation {
            return;
        }

        let response = match outcome {
            Err(_) => {
                *self.last_heartbeat_error.write() =
                    Some("client telemetry heartbeat timed out".to_owned());
                return;
            }
            Ok(Err(status)) => {
                if status.code() == Code::Unimplemented {
                    self.unsupported_streak.fetch_add(1, Ordering::Relaxed);
                }
                *self.last_heartbeat_error.write() = Some(status.to_string());
                return;
            }
            Ok(Ok(response)) => response.into_inner(),
        };

        // Any actual RPC response proves that this transport implements the service,
        // even when the response carries a business failure.
        self.unsupported_streak.store(0, Ordering::Relaxed);
        if let Err(error) = status_to_result(&response.status) {
            *self.last_heartbeat_error.write() = Some(error.to_string());
            return;
        }
        *self.last_heartbeat_error.write() = None;

        {
            let mut state = self.state.lock();
            for _ in 0..reply_count.min(state.pending_replies.len()) {
                state.pending_replies.pop_front();
            }
        }
        let commands = response.commands;
        // Command handlers are user-extensible and may call normal client APIs. Do not hold
        // the service-generation lock while invoking them: an API retry may need the write
        // side of this same lock to publish a global-cluster failover.
        drop(generation_guard);
        self.process_commands(commands);
    }

    fn process_commands(&self, commands: Vec<common::ClientCommand>) {
        let _command_guard = self.command_lock.lock();
        self.process_commands_locked(commands);
    }

    fn process_commands_locked(&self, commands: Vec<common::ClientCommand>) {
        let (last_timestamp, mut max_timestamp) = {
            let state = self.state.lock();
            (state.last_command_timestamp, state.last_command_timestamp)
        };
        let has_persistent = commands.iter().any(|command| command.persistent);

        for command in &commands {
            max_timestamp = max_timestamp.max(command.create_time);
            let local = ClientTelemetryCommand {
                command_id: command.command_id.clone(),
                command_type: command.command_type.clone(),
                payload: command.payload.clone(),
                create_time: command.create_time,
                persistent: command.persistent,
                target_scope: command.target_scope.clone(),
            };

            let already_executed = {
                let state = self.state.lock();
                command.create_time < last_timestamp
                    || state.executed_commands.contains_key(&command.command_id)
            };
            if already_executed {
                self.state
                    .lock()
                    .pending_replies
                    .push_back(common::CommandReply {
                        command_id: command.command_id.clone(),
                        success: true,
                        error_message: String::new(),
                        payload: Vec::new(),
                    });
                continue;
            }

            let reply = self.handle_command(&local);
            let mut state = self.state.lock();
            state
                .executed_commands
                .insert(command.command_id.clone(), command.create_time);
            state.pending_replies.push_back(common::CommandReply {
                command_id: reply.command_id,
                success: reply.success,
                error_message: reply.error_message,
                payload: reply.payload,
            });
        }

        let mut state = self.state.lock();
        // Equal-timestamp IDs must remain: the server's timestamp comparison is strict,
        // so the same command can be returned repeatedly at the cursor.
        state
            .executed_commands
            .retain(|_, timestamp| *timestamp >= max_timestamp);
        if has_persistent {
            state.config_hash = calculate_config_hash(&commands);
        }
        if max_timestamp > state.last_command_timestamp {
            state.last_command_timestamp = max_timestamp;
        }
    }

    fn handle_command(&self, command: &ClientTelemetryCommand) -> ClientTelemetryCommandReply {
        match command.command_type.as_str() {
            "push_config" => self.handle_push_config(command),
            "collection_metrics" => self.handle_collection_metrics(command),
            "show_errors" => self.handle_show_errors(command),
            "show_latency_history" => self.handle_show_latency_history(command),
            "get_config" => self.handle_get_config(command),
            command_type => {
                // Clone under the read lock, then explicitly release it before calling
                // user code. A custom handler may register another handler itself.
                let handler = self.command_handlers.read().get(command_type).cloned();
                match handler {
                    Some(handler) => match catch_unwind(AssertUnwindSafe(|| handler(command))) {
                        Ok(reply) => reply,
                        Err(payload) => ClientTelemetryCommandReply::failure(
                            &command.command_id,
                            format!(
                                "custom command handler panicked: {}",
                                panic_payload_message(payload.as_ref())
                            ),
                        ),
                    },
                    None => ClientTelemetryCommandReply::failure(
                        &command.command_id,
                        format!("unknown command type: {command_type}"),
                    ),
                }
            }
        }
    }

    fn handle_push_config(&self, command: &ClientTelemetryCommand) -> ClientTelemetryCommandReply {
        let object = match json_object(&command.payload) {
            Ok(object) => object,
            Err(error) => {
                return ClientTelemetryCommandReply::failure(
                    &command.command_id,
                    format!("failed to parse config payload: {error}"),
                )
            }
        };

        let enabled = match optional_bool(&object, "enabled") {
            Ok(value) => value,
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        let heartbeat_ms = match optional_i64(&object, "heartbeat_interval_ms") {
            Ok(value) => value,
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        let sampling_rate = match optional_f64(&object, "sampling_rate") {
            Ok(value) => value,
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        // ttl_seconds is ignored by clients, but a known field with a malformed type
        // still rejects the entire typed payload just as Go's ConfigPayload decoder does.
        if let Err(error) = optional_i64(&object, "ttl_seconds") {
            return ClientTelemetryCommandReply::failure(&command.command_id, error);
        }
        if heartbeat_ms.is_some_and(|value| value <= 0) {
            return ClientTelemetryCommandReply::failure(
                &command.command_id,
                "heartbeat_interval_ms must be positive",
            );
        }

        let acted_on = ["enabled", "heartbeat_interval_ms", "sampling_rate"];
        let mut ignored: Vec<_> = object
            .keys()
            .filter(|key| !acted_on.contains(&key.as_str()))
            .cloned()
            .collect();
        ignored.sort();
        let mut applied = Vec::new();

        // All validation above precedes this lock: updates are all-or-nothing.
        let mut state = self.state.lock();
        if let Some(enabled) = enabled {
            state.config.enabled = enabled;
            applied.push("enabled");
        }
        if let Some(heartbeat_ms) = heartbeat_ms {
            state.config.heartbeat_interval = Duration::from_millis(heartbeat_ms as u64);
            applied.push("heartbeat_interval_ms");
        }
        if let Some(sampling_rate) = sampling_rate {
            state.config.sampling_rate = sampling_rate.clamp(0.0, 1.0);
            applied.push("sampling_rate");
        }
        drop(state);

        let payload = serde_json::to_vec(&json!({
            "applied": applied,
            "ignored": ignored,
        }))
        .unwrap_or_default();
        ClientTelemetryCommandReply::success(&command.command_id, payload)
    }

    fn handle_collection_metrics(
        &self,
        command: &ClientTelemetryCommand,
    ) -> ClientTelemetryCommandReply {
        if command.payload.is_empty() {
            let state = self.state.lock();
            let payload = serde_json::to_vec(&json!({
                "enabled_collections": state.enabled_collections.iter().collect::<Vec<_>>(),
                "all_collections_enabled": state.all_collections_enabled,
            }))
            .unwrap_or_default();
            return ClientTelemetryCommandReply::success(&command.command_id, payload);
        }
        let object = match json_object(&command.payload) {
            Ok(object) => object,
            Err(error) => {
                return ClientTelemetryCommandReply::failure(
                    &command.command_id,
                    format!("failed to parse collection_metrics payload: {error}"),
                )
            }
        };
        let enabled = match optional_bool(&object, "enabled") {
            Ok(value) => value.unwrap_or(false),
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        let collections = match optional_string_array(&object, "collections") {
            Ok(value) => value.unwrap_or_default(),
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        if let Err(error) = optional_string_array(&object, "metrics_types") {
            return ClientTelemetryCommandReply::failure(&command.command_id, error);
        }
        if enabled && collections.is_empty() {
            return ClientTelemetryCommandReply::failure(
                &command.command_id,
                "collections list cannot be empty when enabled=true",
            );
        }

        let wildcard = collections.iter().any(|collection| collection == "*");
        let mut state = self.state.lock();
        if enabled {
            if wildcard {
                state.all_collections_enabled = true;
            } else {
                state.enabled_collections.extend(collections);
            }
        } else if wildcard || collections.is_empty() {
            state.all_collections_enabled = false;
            state.enabled_collections.clear();
        } else {
            for collection in collections {
                state.enabled_collections.remove(&collection);
            }
        }
        ClientTelemetryCommandReply::success(&command.command_id, Vec::new())
    }

    fn handle_show_errors(&self, command: &ClientTelemetryCommand) -> ClientTelemetryCommandReply {
        let max_count = if command.payload.is_empty() {
            100
        } else {
            let object = match json_object(&command.payload) {
                Ok(object) => object,
                Err(error) => {
                    return ClientTelemetryCommandReply::failure(
                        &command.command_id,
                        format!("failed to parse show_errors payload: {error}"),
                    )
                }
            };
            match optional_i64(&object, "max_count") {
                Ok(Some(value)) if value > 0 => value as usize,
                Ok(_) => 100,
                Err(error) => {
                    return ClientTelemetryCommandReply::failure(&command.command_id, error)
                }
            }
        };
        let mut errors: Vec<_> = self
            .state
            .lock()
            .errors
            .iter()
            .rev()
            .take(max_count)
            .cloned()
            .collect();
        if errors.is_empty() {
            return ClientTelemetryCommandReply::success(&command.command_id, Vec::new());
        }
        let mut payload = serde_json::to_vec(&errors).unwrap_or_default();
        while payload.len() > MAX_REPLY_PAYLOAD_SIZE && errors.len() > 1 {
            errors.truncate(errors.len() / 2);
            payload = serde_json::to_vec(&errors).unwrap_or_default();
        }
        while payload.len() > MAX_REPLY_PAYLOAD_SIZE && !errors[0].error_message.is_empty() {
            // Halve at a UTF-8 boundary and re-encode. Basing the decision on the encoded
            // JSON size also handles quotes/backslashes, which may expand on serialization.
            let message = &mut errors[0].error_message;
            let mut new_len = message.len() / 2;
            while new_len > 0 && !message.is_char_boundary(new_len) {
                new_len -= 1;
            }
            message.truncate(new_len);
            if new_len > 0 {
                message.push_str("...(truncated)");
            }
            payload = serde_json::to_vec(&errors).unwrap_or_default();
        }
        if payload.len() > MAX_REPLY_PAYLOAD_SIZE {
            return ClientTelemetryCommandReply::failure(&command.command_id, "response too large");
        }
        ClientTelemetryCommandReply::success(&command.command_id, payload)
    }

    fn handle_show_latency_history(
        &self,
        command: &ClientTelemetryCommand,
    ) -> ClientTelemetryCommandReply {
        if command.payload.is_empty() {
            return ClientTelemetryCommandReply::failure(
                &command.command_id,
                "payload is required with start_time and end_time",
            );
        }
        let object = match json_object(&command.payload) {
            Ok(object) => object,
            Err(error) => {
                return ClientTelemetryCommandReply::failure(
                    &command.command_id,
                    format!("failed to parse show_latency_history payload: {error}"),
                )
            }
        };
        let start = match required_string(&object, "start_time")
            .and_then(|value| parse_rfc3339_millis(&value, "start_time"))
        {
            Ok(value) => value,
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        let end = match required_string(&object, "end_time")
            .and_then(|value| parse_rfc3339_millis(&value, "end_time"))
        {
            Ok(value) => value,
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        let detail = match optional_bool(&object, "detail") {
            Ok(value) => value.unwrap_or(false),
            Err(error) => return ClientTelemetryCommandReply::failure(&command.command_id, error),
        };
        if end < start {
            return ClientTelemetryCommandReply::failure(
                &command.command_id,
                "end_time must be after start_time",
            );
        }
        if end.saturating_sub(start) > HISTORY_RETENTION.as_millis() as i64 {
            return ClientTelemetryCommandReply::failure(
                &command.command_id,
                "time range cannot exceed 1 hour",
            );
        }

        let snapshots: Vec<_> = self
            .state
            .lock()
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.end_time >= start && snapshot.timestamp <= end)
            .cloned()
            .collect();
        let payload_value = if detail {
            json!({
                "snapshots": snapshots.iter().map(snapshot_json).collect::<Vec<_>>(),
                "total_snapshots": snapshots.len(),
            })
        } else {
            aggregate_snapshot_json(&snapshots, start, end)
        };
        let payload = serde_json::to_vec(&payload_value).unwrap_or_default();
        if payload.len() > MAX_REPLY_PAYLOAD_SIZE {
            return ClientTelemetryCommandReply::failure(
                &command.command_id,
                "response too large, try a smaller time range",
            );
        }
        ClientTelemetryCommandReply::success(&command.command_id, payload)
    }

    fn handle_get_config(&self, command: &ClientTelemetryCommand) -> ClientTelemetryCommandReply {
        let state = self.state.lock();
        let enabled_collections: Vec<_> = if state.all_collections_enabled {
            vec!["*".to_owned()]
        } else {
            state.enabled_collections.iter().cloned().collect()
        };
        let payload = serde_json::to_vec(&json!({
            "user_config": {
                "address": self.client_config.uri,
                "username": self.client_config.username,
                "db_name": self.client_config.initial_database,
                "enable_tls_auth": self.client_config.tls_enabled,
                "retry_max_retry": self.client_config.retry_max_attempts,
                "retry_max_backoff_ms": self.client_config.retry_max_backoff_ms,
                "current_db": self.database.read().clone(),
                "telemetry_enabled": state.config.enabled,
                "telemetry_heartbeat_interval_ms": state.config.heartbeat_interval.as_millis(),
                "telemetry_sampling_rate": state.config.sampling_rate,
                "enabled_collections": enabled_collections,
                "all_collections_enabled": state.all_collections_enabled,
            }
        }))
        .unwrap_or_default();
        ClientTelemetryCommandReply::success(&command.command_id, payload)
    }
}

fn json_object(payload: &[u8]) -> Result<Map<String, Value>, String> {
    if payload.is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_slice(payload).map_err(|error| error.to_string())? {
        Value::Object(object) => Ok(object),
        _ => Err("command payload must be a JSON object".to_owned()),
    }
}

fn optional_bool(object: &Map<String, Value>, key: &str) -> Result<Option<bool>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("{key} must be a boolean")),
    }
}

fn optional_i64(object: &Map<String, Value>, key: &str) -> Result<Option<i64>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| format!("{key} must be a signed 64-bit integer")),
        Some(_) => Err(format!("{key} must be an integer")),
    }
}

fn optional_f64(object: &Map<String, Value>, key: &str) -> Result<Option<f64>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value
            .as_f64()
            .filter(|value| value.is_finite())
            .map(Some)
            .ok_or_else(|| format!("{key} must be finite")),
        Some(_) => Err(format!("{key} must be a number")),
    }
}

fn optional_string_array(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<Vec<String>>, String> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("{key} must be an array of strings"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(format!("{key} must be an array of strings")),
    }
}

fn required_string(object: &Map<String, Value>, key: &str) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("payload is required with {key}"))
}

fn parse_rfc3339_millis(value: &str, name: &str) -> Result<i64, String> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.timestamp_millis())
        .map_err(|error| format!("invalid {name} format, expected RFC3339: {error}"))
}

fn calculate_config_hash(commands: &[common::ClientCommand]) -> String {
    let mut persistent: Vec<_> = commands
        .iter()
        .filter(|command| command.persistent)
        .collect();
    if persistent.is_empty() {
        return String::new();
    }
    persistent.sort_by(|left, right| left.command_id.cmp(&right.command_id));
    let mut digest = Sha256::new();
    for command in persistent {
        digest.update(command.command_id.as_bytes());
        digest.update(command.command_type.as_bytes());
        digest.update(&command.payload);
    }
    format!("{:x}", digest.finalize())[..16].to_owned()
}

fn snapshot_json(snapshot: &TelemetrySnapshot) -> Value {
    let metrics: Map<String, Value> = snapshot
        .metrics
        .iter()
        .map(|operation| {
            (
                operation.operation.clone(),
                serde_json::to_value(&operation.global).unwrap_or(Value::Null),
            )
        })
        .collect();
    json!({
        "timestamp": snapshot.timestamp,
        "end_time": snapshot.end_time,
        "metrics": metrics,
    })
}

fn aggregate_snapshot_json(snapshots: &[TelemetrySnapshot], start: i64, end: i64) -> Value {
    #[derive(Default)]
    struct Aggregate {
        request_count: i64,
        success_count: i64,
        error_count: i64,
        weighted_avg: f64,
        weighted_p99: f64,
        max_latency: f64,
    }
    let mut totals: BTreeMap<String, Aggregate> = BTreeMap::new();
    for snapshot in snapshots {
        for operation in &snapshot.metrics {
            let total = totals.entry(operation.operation.clone()).or_default();
            let metrics = &operation.global;
            total.request_count += metrics.request_count;
            total.success_count += metrics.success_count;
            total.error_count += metrics.error_count;
            total.weighted_avg += metrics.avg_latency_ms * metrics.request_count as f64;
            total.weighted_p99 += metrics.p99_latency_ms * metrics.request_count as f64;
            total.max_latency = total.max_latency.max(metrics.max_latency_ms);
        }
    }
    let metrics: Map<String, Value> = totals
        .into_iter()
        .map(|(operation, total)| {
            let denominator = total.request_count as f64;
            (
                operation,
                json!({
                    "request_count": total.request_count,
                    "success_count": total.success_count,
                    "error_count": total.error_count,
                    "avg_latency_ms": if total.request_count == 0 { 0.0 } else { total.weighted_avg / denominator },
                    "p99_latency_ms": if total.request_count == 0 { 0.0 } else { total.weighted_p99 / denominator },
                    "max_latency_ms": total.max_latency,
                }),
            )
        })
        .collect();
    json!({
        "aggregated": {"start_time": start, "end_time": end, "metrics": metrics},
        "snapshot_count": snapshots.len(),
    })
}

fn metrics_to_proto(metrics: &TelemetryMetrics) -> common::Metrics {
    common::Metrics {
        request_count: metrics.request_count,
        success_count: metrics.success_count,
        error_count: metrics.error_count,
        avg_latency_ms: metrics.avg_latency_ms,
        p99_latency_ms: metrics.p99_latency_ms,
        max_latency_ms: metrics.max_latency_ms,
    }
}

async fn heartbeat_loop(weak: Weak<TelemetryInner>) {
    loop {
        let Some(inner) = weak.upgrade() else {
            return;
        };
        inner.create_snapshot();
        inner.send_heartbeat().await;
        let delay = inner.next_heartbeat_delay();
        let shutdown = inner.shutdown.clone();
        drop(inner);
        tokio::select! {
            _ = shutdown.cancelled() => return,
            _ = tokio::time::sleep(delay) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::milvus::client_telemetry_service_server::{
        ClientTelemetryService, ClientTelemetryServiceServer,
    };
    use crate::v2::client::{service_bundle, V2Interceptor};
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;
    use tokio::sync::{mpsc, oneshot};
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::{Endpoint, Server};

    struct CapturedHeartbeat {
        request: milvus::ClientHeartbeatRequest,
        authorization: Option<String>,
        database: Option<String>,
        request_millis: Option<String>,
    }

    enum HeartbeatAction {
        Respond(milvus::ClientHeartbeatResponse),
        Fail(Code, &'static str),
        BlockedResponse {
            entered: oneshot::Sender<()>,
            release: oneshot::Receiver<()>,
            response: milvus::ClientHeartbeatResponse,
        },
    }

    #[derive(Clone)]
    struct MockTelemetryService {
        actions: Arc<tokio::sync::Mutex<VecDeque<HeartbeatAction>>>,
        captures: mpsc::UnboundedSender<CapturedHeartbeat>,
    }

    #[tonic::async_trait]
    impl ClientTelemetryService for MockTelemetryService {
        async fn client_heartbeat(
            &self,
            request: Request<milvus::ClientHeartbeatRequest>,
        ) -> Result<tonic::Response<milvus::ClientHeartbeatResponse>, tonic::Status> {
            let capture = CapturedHeartbeat {
                authorization: metadata_value(&request, "authorization"),
                database: metadata_value(&request, "dbname"),
                request_millis: metadata_value(&request, "client-request-unixmsec"),
                request: request.into_inner(),
            };
            self.captures
                .send(capture)
                .expect("heartbeat capture receiver");

            let action = self
                .actions
                .lock()
                .await
                .pop_front()
                .expect("unexpected heartbeat");
            match action {
                HeartbeatAction::Respond(response) => Ok(tonic::Response::new(response)),
                HeartbeatAction::Fail(code, message) => Err(tonic::Status::new(code, message)),
                HeartbeatAction::BlockedResponse {
                    entered,
                    release,
                    response,
                } => {
                    entered.send(()).expect("heartbeat entered receiver");
                    release.await.expect("heartbeat release sender");
                    Ok(tonic::Response::new(response))
                }
            }
        }
    }

    struct MockTelemetryServer {
        uri: String,
        captures: mpsc::UnboundedReceiver<CapturedHeartbeat>,
        shutdown: Option<oneshot::Sender<()>>,
        task: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    }

    impl MockTelemetryServer {
        async fn start(actions: Vec<HeartbeatAction>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind telemetry mock");
            let address = listener.local_addr().expect("telemetry mock address");
            let (capture_tx, capture_rx) = mpsc::unbounded_channel();
            let service = MockTelemetryService {
                actions: Arc::new(tokio::sync::Mutex::new(actions.into())),
                captures: capture_tx,
            };
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let task = tokio::spawn(
                Server::builder()
                    .add_service(ClientTelemetryServiceServer::new(service))
                    .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                        let _ = shutdown_rx.await;
                    }),
            );
            Self {
                uri: format!("http://{address}"),
                captures: capture_rx,
                shutdown: Some(shutdown_tx),
                task,
            }
        }

        async fn next_capture(&mut self) -> CapturedHeartbeat {
            tokio::time::timeout(Duration::from_secs(1), self.captures.recv())
                .await
                .expect("timed out waiting for heartbeat")
                .expect("heartbeat server stopped")
        }
    }

    impl Drop for MockTelemetryServer {
        fn drop(&mut self) {
            if let Some(shutdown) = self.shutdown.take() {
                let _ = shutdown.send(());
            }
            self.task.abort();
        }
    }

    fn metadata_value<T>(request: &Request<T>, key: &'static str) -> Option<String> {
        request
            .metadata()
            .get(key)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    }

    fn success_response(commands: Vec<common::ClientCommand>) -> milvus::ClientHeartbeatResponse {
        milvus::ClientHeartbeatResponse {
            status: Some(common::Status::default()),
            commands,
            ..Default::default()
        }
    }

    fn command_reply(command_id: &str) -> common::CommandReply {
        common::CommandReply {
            command_id: command_id.to_owned(),
            success: true,
            ..Default::default()
        }
    }

    fn transport_manager(
        config: TelemetryConfig,
        uri: &str,
        database_name: &str,
        token: &str,
        generation: u64,
    ) -> ClientTelemetry {
        let database = Arc::new(RwLock::new(database_name.to_owned()));
        let database_explicit = Arc::new(AtomicBool::new(!database_name.is_empty()));
        let connect = ConnectConfig::new()
            .uri(uri)
            .database(database_name)
            .token(token)
            .telemetry(config.clone());
        let interceptor = V2Interceptor {
            token: connect
                .get_token()
                .clone()
                .map(|value| value.parse().expect("authorization metadata")),
            database: Arc::clone(&database),
            database_explicit: Arc::clone(&database_explicit),
        };
        let channel = Endpoint::from_shared(uri.to_owned())
            .expect("telemetry endpoint")
            .connect_lazy();
        let services = Arc::new(RwLock::new(service_bundle(
            channel,
            interceptor,
            generation,
        )));
        ClientTelemetry::new(config, services, database, database_explicit, &connect)
    }

    fn manager(config: TelemetryConfig) -> ClientTelemetry {
        let database = Arc::new(RwLock::new("default".to_owned()));
        let database_explicit = Arc::new(AtomicBool::new(false));
        let channel = Endpoint::from_static("http://127.0.0.1:19530").connect_lazy();
        let interceptor = V2Interceptor {
            token: None,
            database: Arc::clone(&database),
            database_explicit: Arc::clone(&database_explicit),
        };
        let services = Arc::new(RwLock::new(service_bundle(channel, interceptor, 7)));
        let connect = ConnectConfig::new().telemetry(config.clone());
        ClientTelemetry::new(config, services, database, database_explicit, &connect)
    }

    fn command(command_id: &str, command_type: &str, payload: &[u8]) -> ClientTelemetryCommand {
        ClientTelemetryCommand {
            command_id: command_id.to_owned(),
            command_type: command_type.to_owned(),
            payload: payload.to_vec(),
            create_time: 10,
            persistent: false,
            target_scope: "global".to_owned(),
        }
    }

    #[test]
    fn trace_id_validation_is_strict_but_generation_is_valid() {
        assert!(valid_trace_id("4bf92f3577b34da6a3ce929d0e0e4736"));
        assert!(!valid_trace_id("00000000000000000000000000000000"));
        assert!(!valid_trace_id("4BF92F3577B34DA6A3CE929D0E0E4736"));
        assert!(!valid_trace_id("legacy-request-id"));
        assert!(valid_trace_id(&new_client_request_id()));
    }

    #[tokio::test]
    async fn malformed_client_request_id_is_not_exposed_to_the_interceptor() {
        with_client_request_id("legacy-request-id", async {
            assert_eq!(current_client_request_id(), None);
        })
        .await;
        with_client_request_id("4bf92f3577b34da6a3ce929d0e0e4736", async {
            assert_eq!(
                current_client_request_id().as_deref(),
                Some("4bf92f3577b34da6a3ce929d0e0e4736")
            );
        })
        .await;
    }

    #[tokio::test]
    async fn client_info_omits_synthetic_default_database_and_formats_local_time() {
        let telemetry = manager(TelemetryConfig::new());
        let info = telemetry.inner.build_client_info();

        assert!(!info.reserved.contains_key("db_name"));
        DateTime::parse_from_rfc3339(&info.local_time).expect("readable RFC3339 local_time");
    }

    #[tokio::test]
    async fn client_info_reports_an_explicit_default_database() {
        let telemetry = transport_manager(
            TelemetryConfig::new(),
            "http://127.0.0.1:19530",
            "default",
            "",
            0,
        );

        assert_eq!(
            telemetry
                .inner
                .build_client_info()
                .reserved
                .get("db_name")
                .map(String::as_str),
            Some("default")
        );
    }

    #[test]
    fn metrics_use_recent_ring_for_p99_and_reset_atomically() {
        let mut bucket = MetricBucket::default();
        for latency in 0..1100 {
            bucket.record(latency, latency % 2 == 0);
        }
        let metrics = bucket.take().expect("metrics");
        assert_eq!(metrics.request_count, 1100);
        assert_eq!(metrics.success_count, 550);
        assert_eq!(metrics.error_count, 550);
        assert_eq!(metrics.p99_latency_ms, 1.09);
        assert!(bucket.take().is_none());
    }

    #[tokio::test]
    async fn push_config_validates_atomically_and_reports_ignored_keys() {
        let telemetry = manager(TelemetryConfig::new().enabled(true));
        let reply = telemetry.inner.handle_command(&command(
            "bad",
            "push_config",
            br#"{"enabled":false,"heartbeat_interval_ms":0}"#,
        ));
        assert!(!reply.success);
        assert!(telemetry.config().enabled);

        let reply = telemetry.inner.handle_command(&command(
            "ttl",
            "push_config",
            br#"{"ttl_seconds":60,"future_option":true}"#,
        ));
        assert!(reply.success);
        let payload: Value = serde_json::from_slice(&reply.payload).expect("reply json");
        assert_eq!(payload["applied"], json!([]));
        assert_eq!(payload["ignored"], json!(["future_option", "ttl_seconds"]));

        let reply = telemetry.inner.handle_command(&command(
            "bad-ttl",
            "push_config",
            br#"{"ttl_seconds":"60"}"#,
        ));
        assert!(!reply.success);
    }

    #[tokio::test]
    async fn custom_handler_can_register_another_handler_without_deadlock() {
        let telemetry = manager(TelemetryConfig::new());
        let reentrant = telemetry.clone();
        telemetry.register_command_handler("custom", move |command| {
            reentrant.register_command_handler("registered-inside", |nested| {
                ClientTelemetryCommandReply::success(&nested.command_id, Vec::new())
            });
            ClientTelemetryCommandReply::success(&command.command_id, Vec::new())
        });

        let worker_telemetry = telemetry.clone();
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let reply = worker_telemetry
                .inner
                .handle_command(&command("outer", "custom", &[]));
            reply_tx.send(reply).expect("reply receiver");
        });
        let reply = reply_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("custom handler deadlocked while registering a handler");
        worker.join().expect("custom handler thread");

        assert!(reply.success);
        assert!(
            telemetry
                .inner
                .handle_command(&command("inner", "registered-inside", &[]))
                .success
        );
    }

    #[tokio::test]
    async fn panicking_custom_handler_returns_failure_and_heartbeat_continues() {
        let panic_command = common::ClientCommand {
            command_id: "panic-command".to_owned(),
            command_type: "panicking".to_owned(),
            create_time: 1,
            ..Default::default()
        };
        let mut server = MockTelemetryServer::start(vec![
            HeartbeatAction::Respond(success_response(vec![panic_command])),
            HeartbeatAction::Respond(success_response(Vec::new())),
        ])
        .await;
        let telemetry = transport_manager(
            TelemetryConfig::new().heartbeat_interval(Duration::from_millis(5)),
            &server.uri,
            "analytics",
            "root:Milvus",
            7,
        );
        telemetry.register_command_handler("panicking", |_| {
            std::panic::panic_any("diagnostic panic payload".to_owned())
        });

        telemetry.start();
        let first = server.next_capture().await;
        assert!(first.request.command_replies.is_empty());
        let second = server.next_capture().await;
        assert_eq!(second.request.command_replies.len(), 1);
        let reply = &second.request.command_replies[0];
        assert_eq!(reply.command_id, "panic-command");
        assert!(!reply.success);
        assert!(reply
            .error_message
            .contains("custom command handler panicked"));
        assert!(reply.error_message.contains("diagnostic panic payload"));
        telemetry.inner.shutdown.cancel();
    }

    #[tokio::test]
    async fn show_errors_truncates_utf8_by_encoded_size_below_one_mib() {
        let telemetry = manager(TelemetryConfig::new().error_max_count(1));
        telemetry
            .inner
            .state
            .lock()
            .errors
            .push_back(TelemetryErrorInfo {
                timestamp: 1,
                operation: "Search".to_owned(),
                error_message: "€".repeat(400_000),
                collection: "books".to_owned(),
                request_id: String::new(),
            });

        let reply = telemetry
            .inner
            .handle_command(&command("errors", "show_errors", &[]));
        assert!(reply.success, "{}", reply.error_message);
        assert!(reply.payload.len() <= MAX_REPLY_PAYLOAD_SIZE);
        let decoded: Value = serde_json::from_slice(&reply.payload).expect("valid error JSON");
        let errors = decoded.as_array().expect("error array");
        assert_eq!(errors.len(), 1);
        assert!(errors[0]["error_msg"]
            .as_str()
            .expect("error message")
            .ends_with("...(truncated)"));
    }

    #[test]
    fn config_hash_is_order_independent_and_payload_sensitive() {
        let first = common::ClientCommand {
            command_id: "b".to_owned(),
            command_type: "push_config".to_owned(),
            payload: br#"{"enabled":true}"#.to_vec(),
            create_time: 2,
            persistent: true,
            target_scope: "global".to_owned(),
        };
        let second = common::ClientCommand {
            command_id: "a".to_owned(),
            command_type: "push_config".to_owned(),
            payload: br#"{"sampling_rate":0.5}"#.to_vec(),
            create_time: 1,
            persistent: true,
            target_scope: "global".to_owned(),
        };
        let hash = calculate_config_hash(&[first.clone(), second.clone()]);
        assert_eq!(
            hash,
            calculate_config_hash(&[second.clone(), first.clone()])
        );
        assert_eq!(hash.len(), 16);
        let mut changed = first;
        changed.payload = br#"{"enabled":false}"#.to_vec();
        assert_ne!(hash, calculate_config_hash(&[second, changed]));
    }

    #[tokio::test]
    async fn equal_timestamp_command_remains_idempotent_across_repeated_batches() {
        let telemetry = manager(TelemetryConfig::new());
        let executions = Arc::new(AtomicUsize::new(0));
        let captured = Arc::clone(&executions);
        telemetry.register_command_handler("custom", move |command| {
            captured.fetch_add(1, Ordering::SeqCst);
            ClientTelemetryCommandReply::success(&command.command_id, Vec::new())
        });
        let command = common::ClientCommand {
            command_id: "same-ms".to_owned(),
            command_type: "custom".to_owned(),
            payload: Vec::new(),
            create_time: 42,
            persistent: false,
            target_scope: "global".to_owned(),
        };
        telemetry.inner.process_commands(vec![command.clone()]);
        telemetry.inner.process_commands(vec![command.clone()]);
        telemetry.inner.process_commands(vec![command]);
        let state = telemetry.inner.state.lock();
        assert_eq!(executions.load(Ordering::SeqCst), 1);
        assert_eq!(state.pending_replies.len(), 3);
        assert_eq!(state.last_command_timestamp, 42);
        assert_eq!(state.executed_commands.get("same-ms"), Some(&42));
    }

    #[tokio::test]
    async fn collection_metrics_are_filtered_when_disabled_before_wire_snapshot() {
        let telemetry = manager(TelemetryConfig::new());
        telemetry.inner.handle_command(&command(
            "enable",
            "collection_metrics",
            br#"{"enabled":true,"collections":["books"]}"#,
        ));
        telemetry
            .inner
            .record_operation("Search", "books", "", Duration::from_millis(2), None);
        telemetry.inner.create_snapshot();
        telemetry.inner.handle_command(&command(
            "disable",
            "collection_metrics",
            br#"{"enabled":false,"collections":["books"]}"#,
        ));
        let request = telemetry.inner.heartbeat_request();
        assert_eq!(request.metrics.len(), 1);
        assert!(request.metrics[0].collection_metrics.is_empty());
    }

    #[tokio::test]
    async fn disabled_telemetry_keeps_control_heartbeat_for_ack_and_reenable() {
        let disable = common::ClientCommand {
            command_id: "disable".to_owned(),
            command_type: "push_config".to_owned(),
            payload: br#"{"enabled":false,"heartbeat_interval_ms":50}"#.to_vec(),
            create_time: 1,
            persistent: true,
            target_scope: "global".to_owned(),
        };
        let reenable = common::ClientCommand {
            command_id: "reenable".to_owned(),
            command_type: "push_config".to_owned(),
            payload: br#"{"enabled":true}"#.to_vec(),
            create_time: 2,
            persistent: true,
            target_scope: "global".to_owned(),
        };
        let (second_entered_tx, second_entered_rx) = oneshot::channel();
        let (release_second_tx, release_second_rx) = oneshot::channel();
        let mut server = MockTelemetryServer::start(vec![
            HeartbeatAction::Respond(success_response(vec![disable])),
            HeartbeatAction::BlockedResponse {
                entered: second_entered_tx,
                release: release_second_rx,
                response: success_response(vec![reenable]),
            },
            HeartbeatAction::Respond(success_response(Vec::new())),
        ])
        .await;
        let opted_out = transport_manager(
            TelemetryConfig::new()
                .enabled(false)
                .heartbeat_interval(Duration::from_millis(5)),
            &server.uri,
            "analytics",
            "root:Milvus",
            6,
        );
        opted_out.start();
        assert!(
            tokio::time::timeout(Duration::from_millis(25), server.captures.recv())
                .await
                .is_err(),
            "an initial enabled=false must not start the heartbeat control plane"
        );
        drop(opted_out);

        let telemetry = transport_manager(
            TelemetryConfig::new(),
            &server.uri,
            "analytics",
            "root:Milvus",
            7,
        );

        telemetry
            .inner
            .record_operation("Search", "books", "", Duration::from_millis(2), None);
        telemetry.start();
        let initial = server.next_capture().await;
        assert_eq!(initial.request.metrics.len(), 1);
        for _ in 0..100 {
            if !telemetry.config().enabled
                && telemetry.inner.state.lock().pending_replies.len() == 1
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        assert!(!telemetry.config().enabled);
        let (snapshot_count, disable_hash) = {
            let state = telemetry.inner.state.lock();
            assert_eq!(state.pending_replies.len(), 1);
            assert_eq!(state.pending_replies[0].command_id, "disable");
            (state.snapshots.len(), state.config_hash.clone())
        };
        assert!(!disable_hash.is_empty());

        telemetry
            .inner
            .record_operation("Search", "books", "", Duration::from_millis(3), None);
        telemetry.inner.create_snapshot();
        assert_eq!(telemetry.inner.state.lock().snapshots.len(), snapshot_count);

        let disabled = server.next_capture().await;
        assert!(disabled.request.metrics.is_empty());
        assert_eq!(disabled.request.config_hash, disable_hash);
        assert_eq!(disabled.request.command_replies.len(), 1);
        assert_eq!(disabled.request.command_replies[0].command_id, "disable");
        tokio::time::timeout(Duration::from_secs(1), second_entered_rx)
            .await
            .expect("disabled control heartbeat did not reach the server")
            .expect("disabled control heartbeat sender stopped");
        release_second_tx
            .send(())
            .expect("disabled control heartbeat stopped before re-enable");
        for _ in 0..100 {
            if telemetry.config().enabled && telemetry.inner.state.lock().pending_replies.len() == 1
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        assert!(telemetry.config().enabled);
        let reenable_hash = {
            let state = telemetry.inner.state.lock();
            assert_eq!(state.pending_replies[0].command_id, "reenable");
            state.config_hash.clone()
        };
        assert!(!reenable_hash.is_empty());

        let reenabled = server.next_capture().await;
        assert_eq!(reenabled.request.config_hash, reenable_hash);
        assert_eq!(reenabled.request.command_replies.len(), 1);
        assert_eq!(reenabled.request.command_replies[0].command_id, "reenable");
        for _ in 0..100 {
            if telemetry.inner.state.lock().pending_replies.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        assert!(telemetry.inner.state.lock().pending_replies.is_empty());
        telemetry.inner.shutdown.cancel();
    }

    #[tokio::test]
    async fn heartbeat_uses_shared_interceptor_and_clears_only_sent_replies() {
        let (entered_tx, entered_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let mut server = MockTelemetryServer::start(vec![HeartbeatAction::BlockedResponse {
            entered: entered_tx,
            release: release_rx,
            response: success_response(Vec::new()),
        }])
        .await;
        let telemetry = transport_manager(
            TelemetryConfig::new().enabled(true),
            &server.uri,
            "analytics",
            "root:Milvus",
            7,
        );
        {
            let mut state = telemetry.inner.state.lock();
            state.pending_replies.push_back(command_reply("sent-a"));
            state.pending_replies.push_back(command_reply("sent-b"));
        }

        let inner = Arc::clone(&telemetry.inner);
        let heartbeat = tokio::spawn(async move { inner.send_heartbeat().await });
        tokio::time::timeout(Duration::from_secs(1), entered_rx)
            .await
            .expect("heartbeat did not reach server")
            .expect("heartbeat sender stopped");
        let captured = server.next_capture().await;
        assert_eq!(captured.authorization.as_deref(), Some("cm9vdDpNaWx2dXM="));
        assert_eq!(captured.database.as_deref(), Some("analytics"));
        assert!(
            captured
                .request_millis
                .as_deref()
                .expect("client-request-unixmsec")
                .parse::<u128>()
                .expect("millisecond metadata")
                > 0
        );
        assert_eq!(
            captured
                .request
                .client_info
                .as_ref()
                .and_then(|info| info.reserved.get("db_name"))
                .map(String::as_str),
            Some("analytics")
        );
        assert_eq!(
            captured
                .request
                .command_replies
                .iter()
                .map(|reply| reply.command_id.as_str())
                .collect::<Vec<_>>(),
            vec!["sent-a", "sent-b"]
        );

        telemetry
            .inner
            .state
            .lock()
            .pending_replies
            .push_back(command_reply("queued-later"));
        release_tx.send(()).expect("release heartbeat");
        tokio::time::timeout(Duration::from_secs(1), heartbeat)
            .await
            .expect("heartbeat did not finish")
            .expect("heartbeat task panicked");

        let state = telemetry.inner.state.lock();
        assert_eq!(state.pending_replies.len(), 1);
        assert_eq!(state.pending_replies[0].command_id, "queued-later");
    }

    #[tokio::test]
    async fn stale_generation_success_is_discarded_without_state_changes() {
        let (entered_tx, entered_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let stale_command = common::ClientCommand {
            command_id: "stale-command".to_owned(),
            command_type: "custom".to_owned(),
            create_time: 99,
            ..Default::default()
        };
        let server = MockTelemetryServer::start(vec![HeartbeatAction::BlockedResponse {
            entered: entered_tx,
            release: release_rx,
            response: success_response(vec![stale_command]),
        }])
        .await;
        let telemetry = transport_manager(
            TelemetryConfig::new().heartbeat_interval(Duration::from_millis(5)),
            &server.uri,
            "analytics",
            "root:Milvus",
            7,
        );
        let executions = Arc::new(AtomicUsize::new(0));
        let captured = Arc::clone(&executions);
        telemetry.register_command_handler("custom", move |command| {
            captured.fetch_add(1, Ordering::SeqCst);
            ClientTelemetryCommandReply::success(&command.command_id, Vec::new())
        });
        telemetry
            .inner
            .state
            .lock()
            .pending_replies
            .push_back(command_reply("keep-me"));
        telemetry
            .inner
            .unsupported_streak
            .store(1, Ordering::Relaxed);
        *telemetry.inner.last_heartbeat_error.write() = Some("existing failure".to_owned());

        let inner = Arc::clone(&telemetry.inner);
        let heartbeat = tokio::spawn(async move { inner.send_heartbeat().await });
        tokio::time::timeout(Duration::from_secs(1), entered_rx)
            .await
            .expect("heartbeat did not reach server")
            .expect("heartbeat sender stopped");
        telemetry.inner.services.write().generation = 8;
        release_tx.send(()).expect("release stale heartbeat");
        tokio::time::timeout(Duration::from_secs(1), heartbeat)
            .await
            .expect("stale heartbeat did not finish")
            .expect("heartbeat task panicked");

        assert_eq!(executions.load(Ordering::SeqCst), 0);
        assert!(!telemetry.is_supported());
        assert_eq!(
            telemetry.last_heartbeat_error().as_deref(),
            Some("existing failure")
        );
        assert_eq!(
            telemetry.inner.next_heartbeat_delay(),
            Duration::from_millis(10)
        );
        let state = telemetry.inner.state.lock();
        assert_eq!(state.pending_replies.len(), 1);
        assert_eq!(state.pending_replies[0].command_id, "keep-me");
        assert_eq!(state.last_command_timestamp, 0);
        assert!(state.executed_commands.is_empty());
    }

    #[tokio::test]
    async fn accepted_response_releases_generation_lock_before_custom_handler() {
        let server =
            MockTelemetryServer::start(vec![HeartbeatAction::Respond(success_response(vec![
                common::ClientCommand {
                    command_id: "custom-command".to_owned(),
                    command_type: "custom".to_owned(),
                    create_time: 1,
                    ..Default::default()
                },
            ]))])
            .await;
        let telemetry = transport_manager(
            TelemetryConfig::new(),
            &server.uri,
            "analytics",
            "root:Milvus",
            7,
        );
        let services = Arc::clone(&telemetry.inner.services);
        telemetry.register_command_handler("custom", move |command| {
            let mut bundle = services
                .try_write()
                .expect("generation lock leaked into custom command handler");
            bundle.generation = 8;
            ClientTelemetryCommandReply::success(&command.command_id, Vec::new())
        });

        telemetry.inner.send_heartbeat().await;

        assert_eq!(telemetry.inner.services.read().generation, 8);
        assert_eq!(telemetry.inner.state.lock().last_command_timestamp, 1);
    }

    #[tokio::test]
    async fn unimplemented_backoff_recovers_after_next_success() {
        let server = MockTelemetryServer::start(vec![
            HeartbeatAction::Fail(Code::Unimplemented, "telemetry disabled"),
            HeartbeatAction::Respond(success_response(Vec::new())),
        ])
        .await;
        let telemetry = transport_manager(
            TelemetryConfig::new().heartbeat_interval(Duration::from_millis(5)),
            &server.uri,
            "analytics",
            "root:Milvus",
            7,
        );

        telemetry.inner.send_heartbeat().await;
        assert!(!telemetry.is_supported());
        assert!(telemetry
            .last_heartbeat_error()
            .expect("unimplemented heartbeat error")
            .to_lowercase()
            .contains("unimplemented"));
        assert_eq!(
            telemetry.inner.next_heartbeat_delay(),
            Duration::from_millis(10)
        );

        telemetry.inner.send_heartbeat().await;
        assert!(telemetry.is_supported());
        assert_eq!(telemetry.last_heartbeat_error(), None);
        assert_eq!(
            telemetry.inner.next_heartbeat_delay(),
            Duration::from_millis(5)
        );
    }
}
