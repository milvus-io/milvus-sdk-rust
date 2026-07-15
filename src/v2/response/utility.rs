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

//! Response types returned by utility and maintenance operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
pub use crate::v2::types::{
    AnalyzerResult, AnalyzerToken, CompactionMerge, PersistentSegmentInfo, QuerySegmentInfo,
};
use crate::v2::types::{CompactionStateCode, SegmentLevel, SegmentState};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// GetServerVersionResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 server_version operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetServerVersionResponse {
    pub(crate) version: String,
    pub(crate) build_time: Option<String>,
    pub(crate) git_commit: Option<String>,
    pub(crate) go_version: Option<String>,
    pub(crate) deploy_mode: Option<String>,
}

impl GetServerVersionResponse {
    #[cfg(test)]
    fn empty() -> Self {
        Self {
            version: String::new(),
            build_time: None,
            git_commit: None,
            go_version: None,
            deploy_mode: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> GetServerVersionResponseBuilder {
        GetServerVersionResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn build_time(&self) -> Option<&str> {
        self.build_time.as_deref()
    }

    pub fn git_commit(&self) -> Option<&str> {
        self.git_commit.as_deref()
    }

    pub fn go_version(&self) -> Option<&str> {
        self.go_version.as_deref()
    }

    pub fn deploy_mode(&self) -> Option<&str> {
        self.deploy_mode.as_deref()
    }

    pub(crate) fn from_version_proto(value: milvus::GetVersionResponse) -> Self {
        Self {
            version: value.version,
            build_time: None,
            git_commit: None,
            go_version: None,
            deploy_mode: None,
        }
    }

    pub(crate) fn from_connect_proto(value: milvus::ConnectResponse) -> Result<Self> {
        let info = value.server_info.ok_or_else(|| {
            Error::MalformedResponse(
                "detailed server-version response does not contain server_info".into(),
            )
        })?;
        Ok(Self {
            version: info.build_tags,
            build_time: Some(info.build_time),
            git_commit: Some(info.git_commit),
            go_version: Some(info.go_version),
            deploy_mode: Some(info.deploy_mode),
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetServerVersionResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetServerVersionResponse in tests.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct GetServerVersionResponseBuilder {
    value: GetServerVersionResponse,
}

#[cfg(test)]
impl GetServerVersionResponseBuilder {
    pub fn version(mut self, value: impl Into<String>) -> Self {
        self.value.version = value.into();
        self
    }

    pub fn build_time(mut self, value: impl Into<String>) -> Self {
        self.value.build_time = Some(value.into());
        self
    }

    pub fn git_commit(mut self, value: impl Into<String>) -> Self {
        self.value.git_commit = Some(value.into());
        self
    }

    pub fn go_version(mut self, value: impl Into<String>) -> Self {
        self.value.go_version = Some(value.into());
        self
    }

    pub fn deploy_mode(mut self, value: impl Into<String>) -> Self {
        self.value.deploy_mode = Some(value.into());
        self
    }

    pub fn build(self) -> GetServerVersionResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod get_server_version_response_tests {
    use super::GetServerVersionResponse;
    use crate::proto::{common, milvus};
    use crate::v2::error::Error;

    #[test]
    fn detailed_server_version_rejects_missing_server_info() {
        let error = GetServerVersionResponse::from_connect_proto(milvus::ConnectResponse {
            status: Some(common::Status::default()),
            server_info: None,
            ..Default::default()
        })
        .expect_err("a detailed response must contain server_info");

        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("server_info"));
    }
}

///////////////////////////////////////////////////////////////////////////////
// CheckHealthResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 check_health operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckHealthResponse {
    pub(crate) is_healthy: bool,
    pub(crate) reasons: Vec<String>,
    pub(crate) quota_states: Vec<String>,
}

impl CheckHealthResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            is_healthy: false,
            reasons: Vec::new(),
            quota_states: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> CheckHealthResponseBuilder {
        CheckHealthResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }

    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }

    pub fn quota_states(&self) -> &[String] {
        &self.quota_states
    }

    pub(crate) fn from_proto(value: milvus::CheckHealthResponse) -> Self {
        Self {
            is_healthy: value.is_healthy,
            reasons: value.reasons,
            quota_states: value
                .quota_states
                .into_iter()
                .map(|state| {
                    milvus::QuotaState::try_from(state)
                        .map(|value| value.as_str_name().to_owned())
                        .unwrap_or_else(|_| state.to_string())
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CheckHealthResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CheckHealthResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct CheckHealthResponseBuilder {
    value: CheckHealthResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl CheckHealthResponseBuilder {
    pub fn is_healthy(mut self, value: bool) -> Self {
        self.value.is_healthy = value;
        self
    }

    pub fn reasons(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.reasons = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn quota_states(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.quota_states = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> CheckHealthResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 flush operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlushResponse {
    pub(crate) database_name: String,
    pub(crate) segment_ids: HashMap<String, Vec<i64>>,
    pub(crate) flush_timestamps: HashMap<String, u64>,
}

impl FlushResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            database_name: String::new(),
            segment_ids: HashMap::new(),
            flush_timestamps: HashMap::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> FlushResponseBuilder {
        FlushResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn segment_ids(&self) -> &HashMap<String, Vec<i64>> {
        &self.segment_ids
    }

    pub fn flush_timestamps(&self) -> &HashMap<String, u64> {
        &self.flush_timestamps
    }

    pub(crate) fn from_proto(value: milvus::FlushResponse) -> Self {
        Self {
            database_name: value.db_name,
            segment_ids: value
                .coll_seg_i_ds
                .into_iter()
                .map(|(name, ids)| (name, ids.data))
                .collect(),
            flush_timestamps: value.coll_flush_ts,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for FlushResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct FlushResponseBuilder {
    value: FlushResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl FlushResponseBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn segment_ids(mut self, value: HashMap<String, Vec<i64>>) -> Self {
        self.value.segment_ids = value;
        self
    }

    pub fn flush_timestamps(mut self, value: HashMap<String, u64>) -> Self {
        self.value.flush_timestamps = value;
        self
    }

    pub fn build(self) -> FlushResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushAllResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 flush_all operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlushAllResponse {
    pub(crate) flush_all_timestamp: u64,
}

impl FlushAllResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            flush_all_timestamp: 0,
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> FlushAllResponseBuilder {
        FlushAllResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn flush_all_timestamp(&self) -> u64 {
        self.flush_all_timestamp
    }

    // Milvus 2.6 still returns the deprecated aggregate flush timestamp.
    #[allow(deprecated)]
    pub(crate) fn from_proto(value: milvus::FlushAllResponse) -> Self {
        Self {
            flush_all_timestamp: value.flush_all_ts,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushAllResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for FlushAllResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct FlushAllResponseBuilder {
    value: FlushAllResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl FlushAllResponseBuilder {
    pub fn flush_all_timestamp(mut self, value: u64) -> Self {
        self.value.flush_all_timestamp = value;
        self
    }

    pub fn build(self) -> FlushAllResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetFlushAllStateResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_flush_all_state operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetFlushAllStateResponse {
    pub(crate) flushed: bool,
}

impl GetFlushAllStateResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self { flushed: false }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetFlushAllStateResponseBuilder {
        GetFlushAllStateResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn is_flushed(&self) -> bool {
        self.flushed
    }

    pub(crate) fn from_proto(value: milvus::GetFlushAllStateResponse) -> Self {
        Self {
            flushed: value.flushed,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetFlushAllStateResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetFlushAllStateResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetFlushAllStateResponseBuilder {
    value: GetFlushAllStateResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetFlushAllStateResponseBuilder {
    pub fn flushed(mut self, value: bool) -> Self {
        self.value.flushed = value;
        self
    }

    pub fn build(self) -> GetFlushAllStateResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPersistentSegmentsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_persistent_segments operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPersistentSegmentsResponse {
    pub(crate) segments: Vec<PersistentSegmentInfo>,
}

impl ListPersistentSegmentsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListPersistentSegmentsResponseBuilder {
        ListPersistentSegmentsResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn segments(&self) -> &[PersistentSegmentInfo] {
        &self.segments
    }

    pub(crate) fn from_proto(
        value: milvus::GetPersistentSegmentInfoResponse,
        collection_name: String,
    ) -> Self {
        Self {
            segments: value
                .infos
                .into_iter()
                .map(|v| PersistentSegmentInfo {
                    segment_id: v.segment_id,
                    collection_id: v.collection_id,
                    partition_id: v.partition_id,
                    row_count: v.num_rows,
                    state: SegmentState::from_proto(v.state),
                    collection_name: collection_name.clone(),
                    level: SegmentLevel::from_proto(v.level),
                    sorted: v.is_sorted,
                    storage_version: v.storage_version,
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPersistentSegmentsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListPersistentSegmentsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListPersistentSegmentsResponseBuilder {
    value: ListPersistentSegmentsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListPersistentSegmentsResponseBuilder {
    pub fn segments(mut self, value: Vec<PersistentSegmentInfo>) -> Self {
        self.value.segments = value;
        self
    }

    pub fn build(self) -> ListPersistentSegmentsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListQuerySegmentsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_query_segments operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListQuerySegmentsResponse {
    pub(crate) segments: Vec<QuerySegmentInfo>,
}

impl ListQuerySegmentsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListQuerySegmentsResponseBuilder {
        ListQuerySegmentsResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn segments(&self) -> &[QuerySegmentInfo] {
        &self.segments
    }

    pub(crate) fn from_proto(
        value: milvus::GetQuerySegmentInfoResponse,
        collection_name: String,
    ) -> Self {
        Self {
            segments: value
                .infos
                .into_iter()
                .map(|v| QuerySegmentInfo {
                    segment_id: v.segment_id,
                    collection_id: v.collection_id,
                    partition_id: v.partition_id,
                    memory_size: v.mem_size,
                    row_count: v.num_rows,
                    index_name: v.index_name,
                    index_id: v.index_id,
                    node_ids: v.node_ids,
                    state: SegmentState::from_proto(v.state),
                    collection_name: collection_name.clone(),
                    level: SegmentLevel::from_proto(v.level),
                    sorted: v.is_sorted,
                    storage_version: v.storage_version,
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListQuerySegmentsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListQuerySegmentsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListQuerySegmentsResponseBuilder {
    value: ListQuerySegmentsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListQuerySegmentsResponseBuilder {
    pub fn segments(mut self, value: Vec<QuerySegmentInfo>) -> Self {
        self.value.segments = value;
        self
    }

    pub fn build(self) -> ListQuerySegmentsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// CompactResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 compact operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompactResponse {
    pub(crate) compaction_id: i64,
    pub(crate) plan_count: i64,
}

impl CompactResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            compaction_id: 0,
            plan_count: 0,
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> CompactResponseBuilder {
        CompactResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn compaction_id(&self) -> i64 {
        self.compaction_id
    }

    pub fn plan_count(&self) -> i64 {
        self.plan_count
    }

    pub(crate) fn from_proto(value: milvus::ManualCompactionResponse) -> Self {
        Self {
            compaction_id: value.compaction_id,
            plan_count: i64::from(value.compaction_plan_count),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CompactResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CompactResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct CompactResponseBuilder {
    value: CompactResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl CompactResponseBuilder {
    pub fn compaction_id(mut self, value: i64) -> Self {
        self.value.compaction_id = value;
        self
    }

    pub fn plan_count(mut self, value: i64) -> Self {
        self.value.plan_count = value;
        self
    }

    pub fn build(self) -> CompactResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// OptimizeResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 optimize operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OptimizeResponse {
    pub(crate) status_text: String,
    pub(crate) collection_name: String,
    pub(crate) compaction_id: i64,
    pub(crate) target_size: String,
    pub(crate) progress_history: Vec<String>,
}

impl OptimizeResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            status_text: String::new(),
            collection_name: String::new(),
            compaction_id: 0,
            target_size: String::new(),
            progress_history: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> OptimizeResponseBuilder {
        OptimizeResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn status_text(&self) -> &str {
        &self.status_text
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn compaction_id(&self) -> i64 {
        self.compaction_id
    }

    pub fn target_size(&self) -> &str {
        &self.target_size
    }

    pub fn progress_history(&self) -> &[String] {
        &self.progress_history
    }
}

///////////////////////////////////////////////////////////////////////////////
// OptimizeResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for OptimizeResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct OptimizeResponseBuilder {
    value: OptimizeResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl OptimizeResponseBuilder {
    pub fn status_text(mut self, value: impl Into<String>) -> Self {
        self.value.status_text = value.into();
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn compaction_id(mut self, value: i64) -> Self {
        self.value.compaction_id = value;
        self
    }

    pub fn target_size(mut self, value: impl Into<String>) -> Self {
        self.value.target_size = value.into();
        self
    }

    pub fn progress_history(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.progress_history = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> OptimizeResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionStateResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_compaction_state operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetCompactionStateResponse {
    pub(crate) state: CompactionStateCode,
    pub(crate) executing_plans: i64,
    pub(crate) timed_out_plans: i64,
    pub(crate) completed_plans: i64,
    pub(crate) failed_plans: i64,
}

impl GetCompactionStateResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            state: CompactionStateCode::default(),
            executing_plans: 0,
            timed_out_plans: 0,
            completed_plans: 0,
            failed_plans: 0,
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetCompactionStateResponseBuilder {
        GetCompactionStateResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn state(&self) -> CompactionStateCode {
        self.state
    }

    pub fn executing_plans(&self) -> i64 {
        self.executing_plans
    }

    pub fn timed_out_plans(&self) -> i64 {
        self.timed_out_plans
    }

    pub fn completed_plans(&self) -> i64 {
        self.completed_plans
    }

    pub fn failed_plans(&self) -> i64 {
        self.failed_plans
    }

    pub(crate) fn from_proto(value: milvus::GetCompactionStateResponse) -> Self {
        Self {
            state: CompactionStateCode::from_proto(value.state),
            executing_plans: value.executing_plan_no,
            timed_out_plans: value.timeout_plan_no,
            completed_plans: value.completed_plan_no,
            failed_plans: value.failed_plan_no,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionStateResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetCompactionStateResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetCompactionStateResponseBuilder {
    value: GetCompactionStateResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetCompactionStateResponseBuilder {
    pub fn state(mut self, value: CompactionStateCode) -> Self {
        self.value.state = value;
        self
    }

    pub fn executing_plans(mut self, value: i64) -> Self {
        self.value.executing_plans = value;
        self
    }

    pub fn timed_out_plans(mut self, value: i64) -> Self {
        self.value.timed_out_plans = value;
        self
    }

    pub fn completed_plans(mut self, value: i64) -> Self {
        self.value.completed_plans = value;
        self
    }

    pub fn failed_plans(mut self, value: i64) -> Self {
        self.value.failed_plans = value;
        self
    }

    pub fn build(self) -> GetCompactionStateResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionPlansResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_compaction_plans operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetCompactionPlansResponse {
    pub(crate) state: CompactionStateCode,
    pub(crate) merges: Vec<CompactionMerge>,
}

impl GetCompactionPlansResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            state: CompactionStateCode::default(),
            merges: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetCompactionPlansResponseBuilder {
        GetCompactionPlansResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn state(&self) -> CompactionStateCode {
        self.state
    }

    pub fn merges(&self) -> &[CompactionMerge] {
        &self.merges
    }

    pub(crate) fn from_proto(value: milvus::GetCompactionPlansResponse) -> Self {
        Self {
            state: CompactionStateCode::from_proto(value.state),
            merges: value
                .merge_infos
                .into_iter()
                .map(|v| CompactionMerge {
                    source_segment_ids: v.sources,
                    target_segment_id: v.target,
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionPlansResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetCompactionPlansResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetCompactionPlansResponseBuilder {
    value: GetCompactionPlansResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetCompactionPlansResponseBuilder {
    pub fn state(mut self, value: CompactionStateCode) -> Self {
        self.value.state = value;
        self
    }

    pub fn merges(mut self, value: Vec<CompactionMerge>) -> Self {
        self.value.merges = value;
        self
    }

    pub fn build(self) -> GetCompactionPlansResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// RunAnalyzerResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 run_analyzer operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RunAnalyzerResponse {
    pub(crate) results: Vec<AnalyzerResult>,
}

impl RunAnalyzerResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> RunAnalyzerResponseBuilder {
        RunAnalyzerResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn results(&self) -> &[AnalyzerResult] {
        &self.results
    }

    pub(crate) fn from_proto(value: milvus::RunAnalyzerResponse) -> Self {
        Self {
            results: value
                .results
                .into_iter()
                .map(|result| AnalyzerResult {
                    tokens: result
                        .tokens
                        .into_iter()
                        .map(|v| AnalyzerToken {
                            text: v.token,
                            start_offset: v.start_offset,
                            end_offset: v.end_offset,
                            position: v.position,
                            position_length: v.position_length,
                            hash: v.hash,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RunAnalyzerResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RunAnalyzerResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct RunAnalyzerResponseBuilder {
    value: RunAnalyzerResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl RunAnalyzerResponseBuilder {
    pub fn results(mut self, value: Vec<AnalyzerResult>) -> Self {
        self.value.results = value;
        self
    }

    pub fn build(self) -> RunAnalyzerResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod segment_response_tests {
    use super::{ListPersistentSegmentsResponse, ListQuerySegmentsResponse};
    use crate::proto::{common, milvus};
    use crate::v2::types::{SegmentLevel, SegmentState};

    #[test]
    fn segment_responses_preserve_request_collection_name() {
        let persistent = ListPersistentSegmentsResponse::from_proto(
            milvus::GetPersistentSegmentInfoResponse {
                infos: vec![milvus::PersistentSegmentInfo {
                    segment_id: 10,
                    collection_id: 20,
                    partition_id: 30,
                    num_rows: 40,
                    state: common::SegmentState::Flushed as i32,
                    level: common::SegmentLevel::L1 as i32,
                    is_sorted: true,
                    storage_version: 2,
                }],
                ..Default::default()
            },
            "books".to_owned(),
        );
        let segment = &persistent.segments()[0];
        assert_eq!(segment.get_collection_name().to_owned(), "books");
        assert_eq!(segment.get_state().to_owned(), SegmentState::Flushed);
        assert_eq!(segment.get_level().to_owned(), SegmentLevel::L1);

        let query = ListQuerySegmentsResponse::from_proto(
            milvus::GetQuerySegmentInfoResponse {
                infos: vec![milvus::QuerySegmentInfo {
                    segment_id: 11,
                    collection_id: 21,
                    partition_id: 31,
                    mem_size: 1_024,
                    num_rows: 41,
                    index_name: "vector_idx".to_owned(),
                    index_id: 51,
                    state: common::SegmentState::Sealed as i32,
                    node_ids: vec![61, 62],
                    level: common::SegmentLevel::L2 as i32,
                    is_sorted: false,
                    storage_version: 3,
                    ..Default::default()
                }],
                ..Default::default()
            },
            "books".to_owned(),
        );
        let segment = &query.segments()[0];
        assert_eq!(segment.get_collection_name().to_owned(), "books");
        assert_eq!(segment.get_state().to_owned(), SegmentState::Sealed);
        assert_eq!(segment.get_level().to_owned(), SegmentLevel::L2);
        assert_eq!(segment.get_node_ids().to_owned(), [61, 62]);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn get_server_version_response_default_values() {
        let value = GetServerVersionResponse::builder().build();
        assert!(value.version().is_empty());
        assert_eq!(value.build_time(), None);
        assert_eq!(value.git_commit(), None);
        assert_eq!(value.go_version(), None);
        assert_eq!(value.deploy_mode(), None);
    }

    #[test]
    fn get_server_version_response_populated_values() {
        let value = GetServerVersionResponse::builder()
            .version("2.6.20")
            .build_time("2026-07-29")
            .git_commit("abcdef")
            .go_version("go1.24")
            .deploy_mode("STANDALONE")
            .build();
        assert_eq!(value.version(), "2.6.20");
        assert_eq!(value.build_time(), Some("2026-07-29"));
        assert_eq!(value.git_commit(), Some("abcdef"));
        assert_eq!(value.go_version(), Some("go1.24"));
        assert_eq!(value.deploy_mode(), Some("STANDALONE"));
    }

    #[test]
    fn check_health_response_default_values() {
        let value = CheckHealthResponse::builder().build();
        let expected_is_healthy: bool = false;
        let expected_reasons: Vec<String> = Default::default();
        let expected_quota_states: Vec<String> = Default::default();

        assert_eq!(value.is_healthy().to_owned(), expected_is_healthy);
        assert_eq!(value.reasons().to_owned(), expected_reasons);
        assert_eq!(value.quota_states().to_owned(), expected_quota_states);
    }

    #[test]
    fn check_health_response_populated_values() {
        let is_healthy = true;
        let reasons = vec!["reasons-value".to_owned()];
        let quota_states = vec!["quota_states-value".to_owned()];
        let value = CheckHealthResponse::builder()
            .is_healthy(is_healthy.clone())
            .reasons(reasons.clone())
            .quota_states(quota_states.clone())
            .build();

        assert_eq!(value.is_healthy().to_owned(), is_healthy);
        assert_eq!(value.reasons().to_owned(), reasons);
        assert_eq!(value.quota_states().to_owned(), quota_states);
    }

    #[test]
    fn flush_response_default_values() {
        let value = FlushResponse::builder().build();
        let expected_database_name: String = String::new();
        let expected_segment_ids: HashMap<String, Vec<i64>> = Default::default();
        let expected_flush_timestamps: HashMap<String, u64> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.segment_ids().to_owned(), expected_segment_ids);
        assert_eq!(
            value.flush_timestamps().to_owned(),
            expected_flush_timestamps
        );
    }

    #[test]
    fn flush_response_populated_values() {
        let database_name = "database_name-value".to_owned();
        let segment_ids = HashMap::from([("key-value".to_owned(), vec![7])]);
        let flush_timestamps = HashMap::from([("key-value".to_owned(), 7)]);
        let value = FlushResponse::builder()
            .database_name(database_name.clone())
            .segment_ids(segment_ids.clone())
            .flush_timestamps(flush_timestamps.clone())
            .build();

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.segment_ids().to_owned(), segment_ids);
        assert_eq!(value.flush_timestamps().to_owned(), flush_timestamps);
    }

    #[test]
    fn flush_all_response_default_values() {
        let value = FlushAllResponse::builder().build();
        let expected_flush_all_timestamp: u64 = 0;

        assert_eq!(value.flush_all_timestamp(), expected_flush_all_timestamp);
    }

    #[test]
    fn flush_all_response_populated_values() {
        let flush_all_timestamp = 7;
        let value = FlushAllResponse::builder()
            .flush_all_timestamp(flush_all_timestamp.clone())
            .build();

        assert_eq!(value.flush_all_timestamp().to_owned(), flush_all_timestamp);
    }

    #[test]
    fn get_flush_all_state_response_default_values() {
        let value = GetFlushAllStateResponse::builder().build();
        let expected_flushed: bool = false;

        assert_eq!(value.is_flushed().to_owned(), expected_flushed);
    }

    #[test]
    fn get_flush_all_state_response_populated_values() {
        let flushed = true;
        let value = GetFlushAllStateResponse::builder()
            .flushed(flushed.clone())
            .build();

        assert_eq!(value.is_flushed().to_owned(), flushed);
    }

    #[test]
    fn list_persistent_segments_response_default_values() {
        let value = ListPersistentSegmentsResponse::builder().build();
        let expected_segments: Vec<PersistentSegmentInfo> = Default::default();

        assert_eq!(value.segments().to_owned(), expected_segments);
    }

    #[test]
    fn list_persistent_segments_response_populated_values() {
        let segments = vec![PersistentSegmentInfo::new()];
        let value = ListPersistentSegmentsResponse::builder()
            .segments(segments.clone())
            .build();

        assert_eq!(value.segments().to_owned(), segments);
    }

    #[test]
    fn list_query_segments_response_default_values() {
        let value = ListQuerySegmentsResponse::builder().build();
        let expected_segments: Vec<QuerySegmentInfo> = Default::default();

        assert_eq!(value.segments().to_owned(), expected_segments);
    }

    #[test]
    fn list_query_segments_response_populated_values() {
        let segments = vec![QuerySegmentInfo::new()];
        let value = ListQuerySegmentsResponse::builder()
            .segments(segments.clone())
            .build();

        assert_eq!(value.segments().to_owned(), segments);
    }

    #[test]
    fn compact_response_default_values() {
        let value = CompactResponse::builder().build();
        let expected_compaction_id: i64 = 0;
        let expected_plan_count: i64 = 0;

        assert_eq!(value.compaction_id().to_owned(), expected_compaction_id);
        assert_eq!(value.plan_count().to_owned(), expected_plan_count);
    }

    #[test]
    fn compact_response_populated_values() {
        let compaction_id = 7;
        let plan_count = 7;
        let value = CompactResponse::builder()
            .compaction_id(compaction_id.clone())
            .plan_count(plan_count.clone())
            .build();

        assert_eq!(value.compaction_id().to_owned(), compaction_id);
        assert_eq!(value.plan_count().to_owned(), plan_count);
    }

    #[test]
    fn optimize_response_default_values() {
        let value = OptimizeResponse::builder().build();
        let expected_status_text: String = String::new();
        let expected_collection_name: String = String::new();
        let expected_compaction_id: i64 = 0;
        let expected_target_size: String = String::new();
        let expected_progress_history: Vec<String> = Default::default();

        assert_eq!(value.status_text().to_owned(), expected_status_text);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.compaction_id().to_owned(), expected_compaction_id);
        assert_eq!(value.target_size().to_owned(), expected_target_size);
        assert_eq!(
            value.progress_history().to_owned(),
            expected_progress_history
        );
    }

    #[test]
    fn optimize_response_populated_values() {
        let status_text = "status_text-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let compaction_id = 7;
        let target_size = "target_size-value".to_owned();
        let progress_history = vec!["progress_history-value".to_owned()];
        let value = OptimizeResponse::builder()
            .status_text(status_text.clone())
            .collection_name(collection_name.clone())
            .compaction_id(compaction_id.clone())
            .target_size(target_size.clone())
            .progress_history(progress_history.clone())
            .build();

        assert_eq!(value.status_text().to_owned(), status_text);
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.compaction_id().to_owned(), compaction_id);
        assert_eq!(value.target_size().to_owned(), target_size);
        assert_eq!(value.progress_history().to_owned(), progress_history);
    }

    #[test]
    fn get_compaction_state_response_default_values() {
        let value = GetCompactionStateResponse::builder().build();
        let expected_state: CompactionStateCode = Default::default();
        let expected_executing_plans: i64 = 0;
        let expected_timed_out_plans: i64 = 0;
        let expected_completed_plans: i64 = 0;
        let expected_failed_plans: i64 = 0;

        assert_eq!(value.state().to_owned(), expected_state);
        assert_eq!(value.executing_plans().to_owned(), expected_executing_plans);
        assert_eq!(value.timed_out_plans().to_owned(), expected_timed_out_plans);
        assert_eq!(value.completed_plans().to_owned(), expected_completed_plans);
        assert_eq!(value.failed_plans().to_owned(), expected_failed_plans);
    }

    #[test]
    fn get_compaction_state_response_populated_values() {
        let state = CompactionStateCode::Completed;
        let executing_plans = 7;
        let timed_out_plans = 7;
        let completed_plans = 7;
        let failed_plans = 7;
        let value = GetCompactionStateResponse::builder()
            .state(state.clone())
            .executing_plans(executing_plans.clone())
            .timed_out_plans(timed_out_plans.clone())
            .completed_plans(completed_plans.clone())
            .failed_plans(failed_plans.clone())
            .build();

        assert_eq!(value.state().to_owned(), state);
        assert_eq!(value.executing_plans().to_owned(), executing_plans);
        assert_eq!(value.timed_out_plans().to_owned(), timed_out_plans);
        assert_eq!(value.completed_plans().to_owned(), completed_plans);
        assert_eq!(value.failed_plans().to_owned(), failed_plans);
    }

    #[test]
    fn get_compaction_plans_response_default_values() {
        let value = GetCompactionPlansResponse::builder().build();
        let expected_state: CompactionStateCode = Default::default();
        let expected_merges: Vec<CompactionMerge> = Default::default();

        assert_eq!(value.state().to_owned(), expected_state);
        assert_eq!(value.merges().to_owned(), expected_merges);
    }

    #[test]
    fn get_compaction_plans_response_populated_values() {
        let state = CompactionStateCode::Completed;
        let merges = vec![CompactionMerge::new()];
        let value = GetCompactionPlansResponse::builder()
            .state(state.clone())
            .merges(merges.clone())
            .build();

        assert_eq!(value.state().to_owned(), state);
        assert_eq!(value.merges().to_owned(), merges);
    }

    #[test]
    fn run_analyzer_response_default_values() {
        let value = RunAnalyzerResponse::builder().build();
        let expected_results: Vec<AnalyzerResult> = Default::default();

        assert_eq!(value.results().to_owned(), expected_results);
    }

    #[test]
    fn run_analyzer_response_populated_values() {
        let results = vec![AnalyzerResult::new()];
        let value = RunAnalyzerResponse::builder()
            .results(results.clone())
            .build();

        assert_eq!(value.results().to_owned(), results);
    }
}
