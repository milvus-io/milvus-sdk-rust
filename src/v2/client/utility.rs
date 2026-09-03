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

//! ClientV2 utility, maintenance, health, and segment operations.

use super::ClientV2;
use crate::proto::{milvus, schema};
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::types::{CompactionStateCode, IndexStateCode, LoadState, TargetSizeUnit};
use crate::v2::{request, response};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::{sleep, timeout, Instant};

const FLUSH_POLL_INTERVAL: Duration = Duration::from_secs(1);
const FLUSH_ALL_POLL_INTERVAL: Duration = Duration::from_millis(500);
const FLUSH_VISIBILITY_DELAY: Duration = Duration::from_secs(1);
const OPTIMIZE_POLL_INTERVAL: Duration = Duration::from_millis(500);

///////////////////////////////////////////////////////////////////////////////
// OptimizeTask
///////////////////////////////////////////////////////////////////////////////
/// Handle for monitoring or cancelling an asynchronous optimization task.
#[derive(Clone)]
pub struct OptimizeTask {
    state: Arc<OptimizeTaskState>,
}

impl OptimizeTask {
    fn new() -> Self {
        Self {
            state: Arc::new(OptimizeTaskState {
                done: AtomicBool::new(false),
                cancelled: AtomicBool::new(false),
                progress: RwLock::new(Vec::new()),
                result: RwLock::new(None),
                notify: Notify::new(),
            }),
        }
    }

    /// Wait for the optimization result. A timeout less than or equal to zero
    /// waits indefinitely.
    pub async fn get_result(&self, timeout_ms: i64) -> Result<response::utility::OptimizeResponse> {
        let wait = async {
            loop {
                let notified = self.state.notify.notified();
                if let Some(result) = self.state.result.read().clone() {
                    return result;
                }
                notified.await;
            }
        };

        if timeout_ms > 0 {
            timeout(Duration::from_millis(timeout_ms as u64), wait)
                .await
                .map_err(|_| Error::Timeout("waiting for optimization result".into()))?
        } else {
            wait.await
        }
    }

    /// Cooperatively cancel the task. In-flight RPCs are allowed to finish,
    /// and cancellation is observed before the next optimization stage.
    pub fn cancel(&self) -> bool {
        if self.is_done() {
            return false;
        }
        if !self.state.cancelled.swap(true, Ordering::SeqCst) {
            self.state.progress.write().push("cancelling".into());
        }
        true
    }

    /// Returns whether the optimization task has finished.
    pub fn is_done(&self) -> bool {
        self.state.done.load(Ordering::SeqCst)
    }

    /// Returns whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::SeqCst)
    }

    /// Returns the latest optimization progress message.
    pub fn current_progress(&self) -> Option<String> {
        self.state.progress.read().last().cloned()
    }

    /// Returns all optimization progress messages recorded so far.
    pub fn progress_history(&self) -> Vec<String> {
        self.state.progress.read().clone()
    }
}

///////////////////////////////////////////////////////////////////////////////
// OptimizeTaskState
///////////////////////////////////////////////////////////////////////////////
struct OptimizeTaskState {
    done: AtomicBool,
    cancelled: AtomicBool,
    progress: RwLock<Vec<String>>,
    result: RwLock<Option<Result<response::utility::OptimizeResponse>>>,
    notify: Notify,
}

impl OptimizeTaskState {
    fn add_progress(&self, value: impl Into<String>) {
        if !self.done.load(Ordering::SeqCst) && !self.cancelled.load(Ordering::SeqCst) {
            self.progress.write().push(value.into());
        }
    }

    fn check_active(&self) -> Result<()> {
        if self.cancelled.load(Ordering::SeqCst) {
            Err(Error::Cancelled("optimization task".into()))
        } else {
            Ok(())
        }
    }

    fn complete(&self, result: &Result<response::utility::OptimizeResponse>) {
        let stored = result.clone().map(|mut response| {
            response.progress_history = self.progress.read().clone();
            response
        });
        *self.result.write() = Some(stored);
        self.done.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }
}

impl ClientV2 {
    /// Returns the compile-time Rust SDK package version.
    pub fn sdk_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Returns the connected Milvus server version and optional build details.
    pub async fn server_version(
        &self,
        request: request::utility::GetServerVersionRequest,
    ) -> Result<response::utility::GetServerVersionResponse> {
        if request.is_detail_enabled() {
            let response = rpc_with_retry!(self, connect, request.into_connect_proto())?;
            status_to_result(&response.status)?;
            return response::utility::GetServerVersionResponse::from_connect_proto(response);
        }

        let response = rpc_with_retry!(self, get_version, request.into_get_version_proto())?;
        status_to_result(&response.status)?;
        Ok(response::utility::GetServerVersionResponse::from_version_proto(response))
    }

    /// Checks whether the connected Milvus server is healthy.
    pub async fn check_health(
        &self,
        request: request::utility::CheckHealthRequest,
    ) -> Result<response::utility::CheckHealthResponse> {
        let response = rpc_with_retry!(self, check_health, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::utility::CheckHealthResponse::from_proto(response))
    }

    /// Flushes pending insert data for a collection into durable storage.
    pub async fn flush(
        &self,
        mut request: request::utility::FlushRequest,
    ) -> Result<response::utility::FlushResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        request.database_name = Some(database.clone());
        let wait_flushed_ms = request.wait_flushed_ms;
        let response = rpc_with_retry!(self, flush, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        self.wait_for_flush(&database, &response, wait_flushed_ms)
            .await?;
        Ok(response::utility::FlushResponse::from_proto(response))
    }

    /// Flushes pending insert data for all collections into durable storage.
    // Milvus 2.6 still returns the deprecated aggregate flush timestamp.
    #[allow(deprecated)]
    /// Performs the flush all operation.
    pub async fn flush_all(
        &self,
        mut request: request::utility::FlushAllRequest,
    ) -> Result<response::utility::FlushAllResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        request.database_name = Some(database.clone());
        let wait_flushed_ms = request.wait_flushed_ms;
        let response = rpc_with_retry!(self, flush_all, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        self.wait_for_flush_all(&database, response.flush_all_ts, wait_flushed_ms)
            .await?;
        Ok(response::utility::FlushAllResponse::from_proto(response))
    }

    /// Retrieves the state of an earlier flush-all operation.
    pub async fn get_flush_all_state(
        &self,
        request: request::utility::GetFlushAllStateRequest,
    ) -> Result<response::utility::GetFlushAllStateResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, get_flush_all_state, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::utility::GetFlushAllStateResponse::from_proto(
            response,
        ))
    }

    /// Lists persisted segments reported by data nodes.
    pub async fn list_persistent_segments(
        &self,
        request: request::utility::ListPersistentSegmentsRequest,
    ) -> Result<response::utility::ListPersistentSegmentsResponse> {
        let database = self.current_database();
        let collection_name = request.collection_name().to_owned();
        let response = rpc_with_retry!(
            self,
            get_persistent_segment_info,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(
            response::utility::ListPersistentSegmentsResponse::from_proto(
                response,
                collection_name,
            ),
        )
    }

    /// Lists segments currently loaded on query nodes.
    pub async fn list_query_segments(
        &self,
        request: request::utility::ListQuerySegmentsRequest,
    ) -> Result<response::utility::ListQuerySegmentsResponse> {
        let database = self.current_database();
        let collection_name = request.collection_name().to_owned();
        let response =
            rpc_with_retry!(self, get_query_segment_info, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::utility::ListQuerySegmentsResponse::from_proto(
            response,
            collection_name,
        ))
    }

    /// Starts a compaction action for a collection.
    pub async fn compact(
        &self,
        request: request::utility::CompactRequest,
    ) -> Result<response::utility::CompactResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let description = self
            .describe_collection_uncached(&database, &collection)
            .await?;
        let mut raw = request.into_proto(&database)?;
        raw.db_name = database;
        raw.collection_id = description.collection_id;
        let response = rpc_with_retry!(self, manual_compaction, raw)?;
        status_to_result(&response.status)?;
        Ok(response::utility::CompactResponse::from_proto(response))
    }

    /// Starts an asynchronous collection-optimization task.
    ///
    /// Use the returned [`OptimizeTask`] to observe progress, wait for completion, or request
    /// cooperative cancellation. Optimization may involve multiple server-side stages.
    pub async fn optimize(
        &self,
        request: request::utility::OptimizeRequest,
    ) -> Result<OptimizeTask> {
        let task = OptimizeTask::new();
        if request.async_mode {
            let client = self.clone();
            let state = Arc::clone(&task.state);
            tokio::spawn(async move {
                let result = client.run_optimize(request, Arc::clone(&state)).await;
                state.complete(&result);
            });
            return Ok(task);
        }

        let result = self.run_optimize(request, Arc::clone(&task.state)).await;
        task.state.complete(&result);
        result?;
        Ok(task)
    }

    async fn run_optimize(
        &self,
        request: request::utility::OptimizeRequest,
        state: Arc<OptimizeTaskState>,
    ) -> Result<response::utility::OptimizeResponse> {
        let timeout_ms = request.timeout_ms;
        let run = self.run_optimize_inner(request, state);
        if timeout_ms > 0 {
            timeout(Duration::from_millis(timeout_ms as u64), run)
                .await
                .map_err(|_| Error::Timeout("optimizing collection".into()))?
        } else {
            run.await
        }
    }

    async fn run_optimize_inner(
        &self,
        request: request::utility::OptimizeRequest,
        state: Arc<OptimizeTaskState>,
    ) -> Result<response::utility::OptimizeResponse> {
        if request.collection_name.is_empty() {
            return Err(Error::validation(
                "collection_name".into(),
                "cannot be empty".into(),
            ));
        }
        let (target_size_mb, normalized_target_size) = parse_target_size_mb(&request.target_size)?;
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();

        state.add_progress("initializing");
        state.check_active()?;
        let description = self
            .describe_collection_uncached(&database, &collection)
            .await?;
        let schema = description.schema.as_ref().ok_or_else(|| {
            Error::MalformedResponse("collection description has no schema".into())
        })?;
        let vector_fields: std::collections::HashSet<&str> = schema
            .fields
            .iter()
            .filter_map(|field| {
                let data_type = schema::DataType::try_from(field.data_type).ok()?;
                matches!(
                    data_type,
                    schema::DataType::BinaryVector
                        | schema::DataType::FloatVector
                        | schema::DataType::Float16Vector
                        | schema::DataType::BFloat16Vector
                        | schema::DataType::SparseFloatVector
                        | schema::DataType::Int8Vector
                )
                .then_some(field.name.as_str())
            })
            .collect();

        let indexes = if vector_fields.is_empty() {
            Vec::new()
        } else {
            let response = self
                .list_indexes(request::index::ListIndexesRequest {
                    database_name: Some(database.clone()),
                    collection_name: collection.clone(),
                    field_name: String::new(),
                    index_name: String::new(),
                    timestamp: 0,
                })
                .await?;
            response
                .indexes
                .into_iter()
                .filter(|index| vector_fields.contains(index.field_name.as_str()))
                .collect()
        };

        self.wait_for_optimize_indexes(
            &database,
            &collection,
            &indexes,
            "waiting for indexes before compaction",
            &state,
        )
        .await?;

        state.check_active()?;
        state.add_progress("compacting");
        let compact_response = rpc_with_retry!(
            self,
            manual_compaction,
            milvus::ManualCompactionRequest {
                collection_id: description.collection_id,
                timetravel: 0,
                major_compaction: false,
                db_name: database.clone(),
                collection_name: collection.clone(),
                target_size: target_size_mb,
                ..Default::default()
            }
        )?;
        status_to_result(&compact_response.status)?;
        let compaction_id = compact_response.compaction_id;
        let compaction_plan_count = compact_response.compaction_plan_count;

        if compaction_plan_count < 0 {
            return Err(Error::MalformedResponse(format!(
                "manual compaction returned invalid plan count {compaction_plan_count}"
            )));
        }
        if compaction_plan_count == 0 {
            state.add_progress("no compaction required");
            return Ok(response::utility::OptimizeResponse {
                status_text: "success".into(),
                collection_name: collection,
                compaction_id,
                target_size: normalized_target_size,
                progress_history: Vec::new(),
            });
        }
        if compaction_id <= 0 {
            return Err(Error::MalformedResponse(format!(
                "manual compaction returned {compaction_plan_count} plans with invalid compaction ID {compaction_id}"
            )));
        }

        state.add_progress("waiting for compaction");
        loop {
            state.check_active()?;
            let compaction = self
                .get_compaction_state(
                    request::utility::GetCompactionStateRequest::builder()
                        .compaction_id(compaction_id)
                        .build()?,
                )
                .await?;
            trace_debug!(
                target: "milvus_sdk::polling",
                operation = "optimize_compaction",
                database = %database,
                collection = %collection,
                compaction_id,
                state = ?compaction.state,
                failed_plans = compaction.failed_plans,
                "optimize compaction polling state"
            );
            if compaction.failed_plans > 0 {
                return Err(Error::Unexpected("compaction failed".into()));
            }
            if compaction.state == CompactionStateCode::Completed {
                break;
            }
            sleep(OPTIMIZE_POLL_INTERVAL).await;
        }

        self.wait_for_optimize_indexes(
            &database,
            &collection,
            &indexes,
            "waiting for indexes after compaction",
            &state,
        )
        .await?;

        state.add_progress("checking load state");
        state.check_active()?;
        let load_state = self
            .get_load_state(request::collection::GetLoadStateRequest {
                database_name: Some(database.clone()),
                collection_name: collection.clone(),
                partition_names: Vec::new(),
            })
            .await?;
        if load_state.state == LoadState::Loaded {
            state.add_progress("refreshing load");
            self.refresh_load(
                request::collection::RefreshLoadRequest::builder()
                    .collection_name(&collection)
                    .database_name(&database)
                    .sync(true)
                    .timeout_ms(request.timeout_ms)
                    .build()?,
            )
            .await?;
        } else {
            state.add_progress("collection not loaded; skip refreshLoad");
        }

        Ok(response::utility::OptimizeResponse {
            status_text: "success".into(),
            collection_name: collection,
            compaction_id,
            target_size: normalized_target_size,
            progress_history: Vec::new(),
        })
    }

    async fn wait_for_optimize_indexes(
        &self,
        database: &str,
        collection: &str,
        indexes: &[response::index::IndexDesc],
        progress: &str,
        state: &OptimizeTaskState,
    ) -> Result<()> {
        if indexes.is_empty() {
            return Ok(());
        }
        state.add_progress(progress);
        loop {
            state.check_active()?;
            let mut finished = true;
            for index in indexes {
                let response = self
                    .describe_index(request::index::DescribeIndexRequest {
                        database_name: Some(database.to_owned()),
                        collection_name: collection.to_owned(),
                        field_name: index.field_name.clone(),
                        index_name: index.index_name.clone(),
                        timestamp: 0,
                    })
                    .await?;
                let description = response.indexes.first().ok_or_else(|| {
                    Error::MalformedResponse(format!("index not found: {}", index.index_name))
                })?;
                match description.state {
                    IndexStateCode::Failed => {
                        return Err(Error::Unexpected(
                            if description.failure_reason.is_empty() {
                                format!("index failed: {}", index.index_name)
                            } else {
                                description.failure_reason.clone()
                            },
                        ));
                    }
                    IndexStateCode::Finished | IndexStateCode::None => {}
                    _ => finished = false,
                }
            }
            if finished {
                return Ok(());
            }
            sleep(OPTIMIZE_POLL_INTERVAL).await;
        }
    }

    /// Retrieves the state of a compaction action.
    pub async fn get_compaction_state(
        &self,
        request: request::utility::GetCompactionStateRequest,
    ) -> Result<response::utility::GetCompactionStateResponse> {
        let response = rpc_with_retry!(self, get_compaction_state, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::utility::GetCompactionStateResponse::from_proto(
            response,
        ))
    }

    /// Retrieves the execution plans produced for a compaction action.
    pub async fn get_compaction_plans(
        &self,
        request: request::utility::GetCompactionPlansRequest,
    ) -> Result<response::utility::GetCompactionPlansResponse> {
        let response =
            rpc_with_retry!(self, get_compaction_state_with_plans, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::utility::GetCompactionPlansResponse::from_proto(
            response,
        ))
    }

    /// Runs a server-side analyzer and returns its result tokens.
    pub async fn run_analyzer(
        &self,
        request: request::utility::RunAnalyzerRequest,
    ) -> Result<response::utility::RunAnalyzerResponse> {
        let telemetry = self.telemetry.begin_operation("RunAnalyzer", "");
        let result = async {
            let database = self.current_database();
            let response = rpc_with_retry!(self, run_analyzer, request.into_proto(&database))?;
            status_to_result(&response.status)?;
            Ok(response::utility::RunAnalyzerResponse::from_proto(response))
        }
        .await;
        telemetry.finish(&result);
        result
    }

    /// Refreshes an external collection from its external data source.
    ///
    /// The operation is asynchronous and returns a job id that can be polled
    /// through [`ClientV2::get_refresh_external_collection_progress`].
    pub async fn refresh_external_collection(
        &self,
        request: request::utility::RefreshExternalCollectionRequest,
    ) -> Result<response::utility::RefreshExternalCollectionResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            NonIdempotent,
            self,
            refresh_external_collection,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::utility::RefreshExternalCollectionResponse::from_proto(response))
    }

    /// Retrieves the state and progress of a refresh-external-collection job.
    pub async fn get_refresh_external_collection_progress(
        &self,
        request: request::utility::GetRefreshExternalCollectionProgressRequest,
    ) -> Result<response::utility::GetRefreshExternalCollectionProgressResponse> {
        let response = rpc_with_retry!(
            self,
            get_refresh_external_collection_progress,
            request.into_proto()
        )?;
        status_to_result(&response.status)?;
        response::utility::GetRefreshExternalCollectionProgressResponse::from_proto(response)
    }

    /// Lists refresh-external-collection jobs for a collection, or for the whole
    /// database when the collection name is omitted.
    pub async fn list_refresh_external_collection_jobs(
        &self,
        request: request::utility::ListRefreshExternalCollectionJobsRequest,
    ) -> Result<response::utility::ListRefreshExternalCollectionJobsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            self,
            list_refresh_external_collection_jobs,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::utility::ListRefreshExternalCollectionJobsResponse::from_proto(response))
    }

    /// Registers a named file resource for external-table workflows.
    pub async fn add_file_resource(
        &self,
        request: request::utility::AddFileResourceRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, add_file_resource, request.into_proto())?;
        self.status(status)
    }

    /// Removes a registered file resource.
    pub async fn remove_file_resource(
        &self,
        request: request::utility::RemoveFileResourceRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            remove_file_resource,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Lists all registered file resources.
    pub async fn list_file_resources(
        &self,
        request: request::utility::ListFileResourcesRequest,
    ) -> Result<response::utility::ListFileResourcesResponse> {
        let response = rpc_with_retry!(self, list_file_resources, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::utility::ListFileResourcesResponse::from_proto(
            response,
        ))
    }

    async fn get_flush_state(
        &self,
        database: &str,
        segment_ids: Vec<i64>,
        flush_timestamp: u64,
    ) -> Result<bool> {
        let response = rpc_with_retry!(
            self,
            get_flush_state,
            milvus::GetFlushStateRequest {
                segment_i_ds: segment_ids,
                flush_ts: flush_timestamp,
                db_name: database.to_owned(),
                collection_name: String::new(),
                ..Default::default()
            }
        )?;
        status_to_result(&response.status)?;
        Ok(response.flushed)
    }

    async fn wait_for_flush(
        &self,
        database: &str,
        response: &milvus::FlushResponse,
        wait_flushed_ms: i64,
    ) -> Result<()> {
        let mut pending: HashMap<String, Vec<i64>> = response
            .coll_seg_i_ds
            .iter()
            .map(|(collection, ids)| (collection.clone(), ids.data.clone()))
            .filter(|(_, ids)| !ids.is_empty())
            .collect();
        if pending.is_empty() {
            return Ok(());
        }
        for collection in pending.keys() {
            if !response.coll_flush_ts.contains_key(collection) {
                return Err(Error::MalformedResponse(format!(
                    "flush response contains segment IDs for collection {collection:?} but no flush timestamp"
                )));
            }
        }

        let deadline = wait_timeout(wait_flushed_ms).map(|timeout| Instant::now() + timeout);
        loop {
            sleep(next_poll_delay(deadline, FLUSH_POLL_INTERVAL)?).await;
            let collections: Vec<String> = pending.keys().cloned().collect();
            for collection in collections {
                let segment_ids = pending.get(&collection).cloned().unwrap_or_default();
                let flush_timestamp = response
                    .coll_flush_ts
                    .get(&collection)
                    .copied()
                    .ok_or_else(|| {
                        Error::MalformedResponse(format!(
                            "flush response contains no timestamp for collection {collection:?}"
                        ))
                    })?;
                let poll = self.get_flush_state(database, segment_ids, flush_timestamp);
                if wait_for_flush_poll(deadline, poll).await? {
                    trace_debug!(
                        target: "milvus_sdk::polling",
                        operation = "flush",
                        database,
                        collection = %collection,
                        "collection flush completed"
                    );
                    pending.remove(&collection);
                }
            }
            if pending.is_empty() {
                break;
            }
        }

        // Allow server-side visibility to catch up after GetFlushState first
        // reports completion.
        let visibility_delay = deadline
            .map(|deadline| {
                FLUSH_VISIBILITY_DELAY.min(deadline.saturating_duration_since(Instant::now()))
            })
            .unwrap_or(FLUSH_VISIBILITY_DELAY);
        sleep(visibility_delay).await;
        Ok(())
    }

    #[allow(deprecated)]
    async fn wait_for_flush_all(
        &self,
        database: &str,
        flush_all_timestamp: u64,
        wait_flushed_ms: i64,
    ) -> Result<()> {
        let deadline = wait_timeout(wait_flushed_ms).map(|timeout| Instant::now() + timeout);
        loop {
            sleep(next_poll_delay(deadline, FLUSH_ALL_POLL_INTERVAL)?).await;
            let poll = async {
                rpc_with_retry!(
                    self,
                    get_flush_all_state,
                    milvus::GetFlushAllStateRequest {
                        base: None,
                        flush_all_ts: flush_all_timestamp,
                        db_name: database.to_owned(),
                        flush_targets: Vec::new(),
                        flush_all_tss: HashMap::new(),
                        ..Default::default()
                    }
                )
            };
            let response = wait_for_flush_poll(deadline, poll).await?;
            status_to_result(&response.status)?;
            trace_debug!(
                target: "milvus_sdk::polling",
                operation = "flush_all",
                database,
                flushed = response.flushed,
                "flush-all polling state"
            );
            if response.flushed {
                return Ok(());
            }
        }
    }
}

fn wait_timeout(wait_flushed_ms: i64) -> Option<Duration> {
    (wait_flushed_ms > 0).then(|| Duration::from_millis(wait_flushed_ms as u64))
}

fn next_poll_delay(deadline: Option<Instant>, poll_interval: Duration) -> Result<Duration> {
    match deadline {
        Some(deadline) => {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                Err(flush_wait_timeout())
            } else {
                Ok(poll_interval.min(remaining))
            }
        }
        None => Ok(poll_interval),
    }
}

async fn wait_for_flush_poll<T>(
    deadline: Option<Instant>,
    poll: impl std::future::Future<Output = Result<T>>,
) -> Result<T> {
    if let Some(deadline) = deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(flush_wait_timeout());
        }
        match timeout(remaining, poll).await {
            Ok(result) => result,
            Err(_) => Err(flush_wait_timeout()),
        }
    } else {
        poll.await
    }
}

fn flush_wait_timeout() -> Error {
    Error::Timeout("waiting for flush completion".into())
}

fn parse_target_size_mb(target_size: &str) -> Result<(i64, String)> {
    let text = target_size.trim();
    if text.is_empty() {
        return Ok((0, String::new()));
    }

    let unit_start = text
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_ascii_digit() && ch != '.').then_some(index))
        .unwrap_or(text.len());
    let number_text = text[..unit_start].trim();
    let number: f64 = number_text.parse().map_err(|_| {
        Error::validation(
            "target_size".into(),
            format!("invalid optimize target size: {target_size}"),
        )
    })?;
    if !number.is_finite() || number <= 0.0 {
        return Err(Error::validation(
            "target_size".into(),
            format!("invalid optimize target size: {target_size}"),
        ));
    }

    let unit: String = text[unit_start..]
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect();
    // Reuse `TargetSizeUnit::bytes_per_unit` so the unit-to-bytes mapping stays consistent with
    // `CompactRequest::target_size_unit` as unit semantics evolve.
    let multiplier = match unit.as_str() {
        "" | "B" => TargetSizeUnit::B.bytes_per_unit() as f64,
        "KB" => TargetSizeUnit::KB.bytes_per_unit() as f64,
        "MB" => TargetSizeUnit::MB.bytes_per_unit() as f64,
        "GB" => TargetSizeUnit::GB.bytes_per_unit() as f64,
        "TB" => TargetSizeUnit::TB.bytes_per_unit() as f64,
        "PB" => TargetSizeUnit::PB.bytes_per_unit() as f64,
        _ => {
            return Err(Error::validation(
                "target_size".into(),
                format!("invalid optimize target size unit: {target_size}"),
            ));
        }
    };
    let megabytes = number * multiplier / 1024.0_f64.powi(2);
    if megabytes < 1.0 || megabytes > i64::MAX as f64 {
        return Err(Error::validation(
            "target_size".into(),
            "optimize target size must be at least 1MB".into(),
        ));
    }
    let megabytes = megabytes as i64;
    Ok((megabytes, format!("{megabytes}MB")))
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{parse_target_size_mb, wait_timeout, OptimizeTask};
    use crate::v2::error::Error;
    use crate::v2::response::utility::OptimizeResponse;
    use std::time::Duration;

    #[test]
    fn wait_timeout_preserves_millisecond_precision() {
        assert_eq!(wait_timeout(0), None);
        assert_eq!(wait_timeout(-1), None);
        assert_eq!(wait_timeout(1), Some(Duration::from_millis(1)));
        assert_eq!(wait_timeout(1_000), Some(Duration::from_millis(1_000)));
        assert_eq!(wait_timeout(1_001), Some(Duration::from_millis(1_001)));
    }

    #[test]
    fn optimize_target_size_uses_megabytes() {
        assert_eq!(parse_target_size_mb("").unwrap(), (0, String::new()));
        assert_eq!(parse_target_size_mb("1MB").unwrap(), (1, "1MB".into()));
        assert_eq!(
            parse_target_size_mb("1.5 GB").unwrap(),
            (1536, "1536MB".into())
        );
        assert!(parse_target_size_mb("512KB").is_err());
        assert!(parse_target_size_mb("1XB").is_err());
    }

    #[tokio::test]
    async fn optimize_task_reports_progress_and_result() {
        let task = OptimizeTask::new();
        task.state.add_progress("initializing");
        let result = Ok(OptimizeResponse {
            status_text: "success".into(),
            collection_name: "books".into(),
            compaction_id: 7,
            target_size: "512MB".into(),
            progress_history: Vec::new(),
        });
        task.state.complete(&result);

        let response = task.get_result(100).await.unwrap();
        assert!(task.is_done());
        assert_eq!(response.compaction_id, 7);
        assert_eq!(response.progress_history, vec!["initializing"]);
    }

    #[tokio::test]
    async fn optimize_task_preserves_error_category() {
        let task = OptimizeTask::new();
        let result = Err(Error::Timeout("optimizing collection".into()));
        task.state.complete(&result);

        assert!(matches!(
            task.get_result(100).await,
            Err(Error::Timeout(message)) if message == "optimizing collection"
        ));
    }

    #[test]
    fn optimize_task_can_be_cancelled_cooperatively() {
        let task = OptimizeTask::new();
        assert!(task.cancel());
        assert!(task.is_cancelled());
        assert_eq!(task.current_progress().as_deref(), Some("cancelling"));
    }
}
