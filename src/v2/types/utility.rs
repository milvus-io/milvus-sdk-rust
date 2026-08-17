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

//! Utility, maintenance, analyzer, compaction, and segment types.

use crate::proto::{common, milvus};

///////////////////////////////////////////////////////////////////////////////
// SegmentState
///////////////////////////////////////////////////////////////////////////////
/// Lifecycle state of a Milvus segment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SegmentState {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the NotExist case.
    NotExist,
    /// Represents the Growing case.
    Growing,
    /// Represents the Sealed case.
    Sealed,
    /// Represents the Flushed case.
    Flushed,
    /// Represents the Flushing case.
    Flushing,
    /// Represents the Dropped case.
    Dropped,
}

impl SegmentState {
    pub(crate) fn from_proto(value: i32) -> Self {
        match common::SegmentState::try_from(value).ok() {
            Some(common::SegmentState::NotExist) => Self::NotExist,
            Some(common::SegmentState::Growing) => Self::Growing,
            Some(common::SegmentState::Sealed) => Self::Sealed,
            Some(common::SegmentState::Flushed) => Self::Flushed,
            Some(common::SegmentState::Flushing) => Self::Flushing,
            Some(common::SegmentState::Dropped) => Self::Dropped,
            _ => Self::Unknown,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SegmentLevel
///////////////////////////////////////////////////////////////////////////////
/// Compaction level assigned to a segment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SegmentLevel {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the Legacy case.
    Legacy,
    /// Represents the L0 case.
    L0,
    /// Represents the L1 case.
    L1,
    /// Represents the L2 case.
    L2,
}

impl SegmentLevel {
    pub(crate) fn from_proto(value: i32) -> Self {
        match common::SegmentLevel::try_from(value).ok() {
            Some(common::SegmentLevel::Legacy) => Self::Legacy,
            Some(common::SegmentLevel::L0) => Self::L0,
            Some(common::SegmentLevel::L1) => Self::L1,
            Some(common::SegmentLevel::L2) => Self::L2,
            _ => Self::Unknown,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CompactionStateCode
///////////////////////////////////////////////////////////////////////////////
/// Execution state of a compaction task.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompactionStateCode {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the Executing case.
    Executing,
    /// Represents the Completed case.
    Completed,
}

impl CompactionStateCode {
    pub(crate) fn from_proto(value: i32) -> Self {
        match common::CompactionState::try_from(value).ok() {
            Some(common::CompactionState::Executing) => Self::Executing,
            Some(common::CompactionState::Completed) => Self::Completed,
            _ => Self::Unknown,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// PersistentSegmentInfo
///////////////////////////////////////////////////////////////////////////////
/// Metadata for a persisted data segment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PersistentSegmentInfo {
    pub(crate) segment_id: i64,
    pub(crate) collection_id: i64,
    pub(crate) partition_id: i64,
    pub(crate) row_count: i64,
    pub(crate) state: SegmentState,
    pub(crate) collection_name: String,
    pub(crate) level: SegmentLevel,
    pub(crate) sorted: bool,
    pub(crate) storage_version: i64,
}

impl PersistentSegmentInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            segment_id: 0,
            collection_id: 0,
            partition_id: 0,
            row_count: 0,
            state: SegmentState::Unknown,
            collection_name: String::new(),
            level: SegmentLevel::Unknown,
            sorted: false,
            storage_version: 0,
        }
    }

    /// Sets the segment id and returns the updated value.
    pub fn segment_id(mut self, value: i64) -> Self {
        self.segment_id = value;
        self
    }

    /// Sets the segment id and returns this value for further mutation.
    pub fn set_segment_id(&mut self, value: i64) -> &mut Self {
        self.segment_id = value;
        self
    }

    /// Returns the configured segment id.
    pub fn get_segment_id(&self) -> i64 {
        self.segment_id
    }

    /// Sets the collection id and returns the updated value.
    pub fn collection_id(mut self, value: i64) -> Self {
        self.collection_id = value;
        self
    }

    /// Sets the collection id and returns this value for further mutation.
    pub fn set_collection_id(&mut self, value: i64) -> &mut Self {
        self.collection_id = value;
        self
    }

    /// Returns the configured collection id.
    pub fn get_collection_id(&self) -> i64 {
        self.collection_id
    }

    /// Sets the partition id and returns the updated value.
    pub fn partition_id(mut self, value: i64) -> Self {
        self.partition_id = value;
        self
    }

    /// Sets the partition id and returns this value for further mutation.
    pub fn set_partition_id(&mut self, value: i64) -> &mut Self {
        self.partition_id = value;
        self
    }

    /// Returns the configured partition id.
    pub fn get_partition_id(&self) -> i64 {
        self.partition_id
    }

    /// Sets the row count and returns the updated value.
    pub fn row_count(mut self, value: i64) -> Self {
        self.row_count = value;
        self
    }

    /// Sets the row count and returns this value for further mutation.
    pub fn set_row_count(&mut self, value: i64) -> &mut Self {
        self.row_count = value;
        self
    }

    /// Returns the configured row count.
    pub fn get_row_count(&self) -> i64 {
        self.row_count
    }

    /// Sets the state and returns the updated value.
    pub fn state(mut self, value: SegmentState) -> Self {
        self.state = value;
        self
    }

    /// Sets the state and returns this value for further mutation.
    pub fn set_state(&mut self, value: SegmentState) -> &mut Self {
        self.state = value;
        self
    }

    /// Returns the configured state.
    pub fn get_state(&self) -> SegmentState {
        self.state
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.collection_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn set_collection_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.collection_name = value.into();
        self
    }

    /// Returns the configured collection name.
    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Sets the level and returns the updated value.
    pub fn level(mut self, value: SegmentLevel) -> Self {
        self.level = value;
        self
    }

    /// Sets the level and returns this value for further mutation.
    pub fn set_level(&mut self, value: SegmentLevel) -> &mut Self {
        self.level = value;
        self
    }

    /// Returns the configured level.
    pub fn get_level(&self) -> SegmentLevel {
        self.level
    }

    /// Sets the sorted and returns the updated value.
    pub fn sorted(mut self, value: bool) -> Self {
        self.sorted = value;
        self
    }

    /// Sets the sorted and returns this value for further mutation.
    pub fn set_sorted(&mut self, value: bool) -> &mut Self {
        self.sorted = value;
        self
    }

    /// Returns the configured sorted.
    pub fn get_sorted(&self) -> bool {
        self.sorted
    }

    /// Sets the storage version and returns the updated value.
    pub fn storage_version(mut self, value: i64) -> Self {
        self.storage_version = value;
        self
    }

    /// Sets the storage version and returns this value for further mutation.
    pub fn set_storage_version(&mut self, value: i64) -> &mut Self {
        self.storage_version = value;
        self
    }

    /// Returns the configured storage version.
    pub fn get_storage_version(&self) -> i64 {
        self.storage_version
    }
}

///////////////////////////////////////////////////////////////////////////////
// QuerySegmentInfo
///////////////////////////////////////////////////////////////////////////////
/// Metadata for a segment loaded by query nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct QuerySegmentInfo {
    pub(crate) segment_id: i64,
    pub(crate) collection_id: i64,
    pub(crate) partition_id: i64,
    pub(crate) memory_size: i64,
    pub(crate) row_count: i64,
    pub(crate) index_name: String,
    pub(crate) index_id: i64,
    pub(crate) node_ids: Vec<i64>,
    pub(crate) state: SegmentState,
    pub(crate) collection_name: String,
    pub(crate) level: SegmentLevel,
    pub(crate) sorted: bool,
    pub(crate) storage_version: i64,
}

impl QuerySegmentInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            segment_id: 0,
            collection_id: 0,
            partition_id: 0,
            memory_size: 0,
            row_count: 0,
            index_name: String::new(),
            index_id: 0,
            node_ids: Vec::new(),
            state: SegmentState::Unknown,
            collection_name: String::new(),
            level: SegmentLevel::Unknown,
            sorted: false,
            storage_version: 0,
        }
    }

    /// Sets the segment id and returns the updated value.
    pub fn segment_id(mut self, value: i64) -> Self {
        self.segment_id = value;
        self
    }

    /// Sets the segment id and returns this value for further mutation.
    pub fn set_segment_id(&mut self, value: i64) -> &mut Self {
        self.segment_id = value;
        self
    }

    /// Returns the configured segment id.
    pub fn get_segment_id(&self) -> i64 {
        self.segment_id
    }

    /// Sets the collection id and returns the updated value.
    pub fn collection_id(mut self, value: i64) -> Self {
        self.collection_id = value;
        self
    }

    /// Sets the collection id and returns this value for further mutation.
    pub fn set_collection_id(&mut self, value: i64) -> &mut Self {
        self.collection_id = value;
        self
    }

    /// Returns the configured collection id.
    pub fn get_collection_id(&self) -> i64 {
        self.collection_id
    }

    /// Sets the partition id and returns the updated value.
    pub fn partition_id(mut self, value: i64) -> Self {
        self.partition_id = value;
        self
    }

    /// Sets the partition id and returns this value for further mutation.
    pub fn set_partition_id(&mut self, value: i64) -> &mut Self {
        self.partition_id = value;
        self
    }

    /// Returns the configured partition id.
    pub fn get_partition_id(&self) -> i64 {
        self.partition_id
    }

    /// Sets the memory size and returns the updated value.
    pub fn memory_size(mut self, value: i64) -> Self {
        self.memory_size = value;
        self
    }

    /// Sets the memory size and returns this value for further mutation.
    pub fn set_memory_size(&mut self, value: i64) -> &mut Self {
        self.memory_size = value;
        self
    }

    /// Returns the configured memory size.
    pub fn get_memory_size(&self) -> i64 {
        self.memory_size
    }

    /// Sets the row count and returns the updated value.
    pub fn row_count(mut self, value: i64) -> Self {
        self.row_count = value;
        self
    }

    /// Sets the row count and returns this value for further mutation.
    pub fn set_row_count(&mut self, value: i64) -> &mut Self {
        self.row_count = value;
        self
    }

    /// Returns the configured row count.
    pub fn get_row_count(&self) -> i64 {
        self.row_count
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.index_name = value.into();
        self
    }

    /// Sets the index name and returns this value for further mutation.
    pub fn set_index_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.index_name = value.into();
        self
    }

    /// Returns the configured index name.
    pub fn get_index_name(&self) -> &str {
        &self.index_name
    }

    /// Sets the index id and returns the updated value.
    pub fn index_id(mut self, value: i64) -> Self {
        self.index_id = value;
        self
    }

    /// Sets the index id and returns this value for further mutation.
    pub fn set_index_id(&mut self, value: i64) -> &mut Self {
        self.index_id = value;
        self
    }

    /// Returns the configured index id.
    pub fn get_index_id(&self) -> i64 {
        self.index_id
    }

    /// Sets the node ids and returns the updated value.
    pub fn node_ids(mut self, value: Vec<i64>) -> Self {
        self.node_ids = value;
        self
    }

    /// Sets the node ids and returns this value for further mutation.
    pub fn set_node_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.node_ids = value;
        self
    }

    /// Returns the configured node ids.
    pub fn get_node_ids(&self) -> &[i64] {
        &self.node_ids
    }

    /// Sets the state and returns the updated value.
    pub fn state(mut self, value: SegmentState) -> Self {
        self.state = value;
        self
    }

    /// Sets the state and returns this value for further mutation.
    pub fn set_state(&mut self, value: SegmentState) -> &mut Self {
        self.state = value;
        self
    }

    /// Returns the configured state.
    pub fn get_state(&self) -> SegmentState {
        self.state
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.collection_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn set_collection_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.collection_name = value.into();
        self
    }

    /// Returns the configured collection name.
    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Sets the level and returns the updated value.
    pub fn level(mut self, value: SegmentLevel) -> Self {
        self.level = value;
        self
    }

    /// Sets the level and returns this value for further mutation.
    pub fn set_level(&mut self, value: SegmentLevel) -> &mut Self {
        self.level = value;
        self
    }

    /// Returns the configured level.
    pub fn get_level(&self) -> SegmentLevel {
        self.level
    }

    /// Sets the sorted and returns the updated value.
    pub fn sorted(mut self, value: bool) -> Self {
        self.sorted = value;
        self
    }

    /// Sets the sorted and returns this value for further mutation.
    pub fn set_sorted(&mut self, value: bool) -> &mut Self {
        self.sorted = value;
        self
    }

    /// Returns the configured sorted.
    pub fn get_sorted(&self) -> bool {
        self.sorted
    }

    /// Sets the storage version and returns the updated value.
    pub fn storage_version(mut self, value: i64) -> Self {
        self.storage_version = value;
        self
    }

    /// Sets the storage version and returns this value for further mutation.
    pub fn set_storage_version(&mut self, value: i64) -> &mut Self {
        self.storage_version = value;
        self
    }

    /// Returns the configured storage version.
    pub fn get_storage_version(&self) -> i64 {
        self.storage_version
    }

    /// Adds one add node id to the existing values.
    pub fn add_node_id(mut self, value: i64) -> Self {
        self.node_ids.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// CompactionMerge
///////////////////////////////////////////////////////////////////////////////
/// Input and output segments participating in a compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompactionMerge {
    pub(crate) source_segment_ids: Vec<i64>,
    pub(crate) target_segment_id: i64,
}

impl CompactionMerge {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            source_segment_ids: Vec::new(),
            target_segment_id: 0,
        }
    }

    /// Sets the source segment ids and returns the updated value.
    pub fn source_segment_ids(mut self, value: Vec<i64>) -> Self {
        self.source_segment_ids = value;
        self
    }

    /// Sets the source segment ids and returns this value for further mutation.
    pub fn set_source_segment_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.source_segment_ids = value;
        self
    }

    /// Returns the configured source segment ids.
    pub fn get_source_segment_ids(&self) -> &[i64] {
        &self.source_segment_ids
    }

    /// Sets the target segment id and returns the updated value.
    pub fn target_segment_id(mut self, value: i64) -> Self {
        self.target_segment_id = value;
        self
    }

    /// Sets the target segment id and returns this value for further mutation.
    pub fn set_target_segment_id(&mut self, value: i64) -> &mut Self {
        self.target_segment_id = value;
        self
    }

    /// Returns the configured target segment id.
    pub fn get_target_segment_id(&self) -> i64 {
        self.target_segment_id
    }

    /// Adds one add source segment id to the existing values.
    pub fn add_source_segment_id(mut self, value: i64) -> Self {
        self.source_segment_ids.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// AnalyzerToken
///////////////////////////////////////////////////////////////////////////////
/// One token produced by a text analyzer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AnalyzerToken {
    pub(crate) text: String,
    pub(crate) start_offset: i64,
    pub(crate) end_offset: i64,
    pub(crate) position: i64,
    pub(crate) position_length: i64,
    pub(crate) hash: u32,
}

impl AnalyzerToken {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            start_offset: 0,
            end_offset: 0,
            position: 0,
            position_length: 0,
            hash: 0,
        }
    }

    /// Sets the text and returns the updated value.
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    /// Sets the text and returns this value for further mutation.
    pub fn set_text(&mut self, value: impl Into<String>) -> &mut Self {
        self.text = value.into();
        self
    }

    /// Returns the configured text.
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Sets the start offset and returns the updated value.
    pub fn start_offset(mut self, value: i64) -> Self {
        self.start_offset = value;
        self
    }

    /// Sets the start offset and returns this value for further mutation.
    pub fn set_start_offset(&mut self, value: i64) -> &mut Self {
        self.start_offset = value;
        self
    }

    /// Returns the configured start offset.
    pub fn get_start_offset(&self) -> i64 {
        self.start_offset
    }

    /// Sets the end offset and returns the updated value.
    pub fn end_offset(mut self, value: i64) -> Self {
        self.end_offset = value;
        self
    }

    /// Sets the end offset and returns this value for further mutation.
    pub fn set_end_offset(&mut self, value: i64) -> &mut Self {
        self.end_offset = value;
        self
    }

    /// Returns the configured end offset.
    pub fn get_end_offset(&self) -> i64 {
        self.end_offset
    }

    /// Sets the position and returns the updated value.
    pub fn position(mut self, value: i64) -> Self {
        self.position = value;
        self
    }

    /// Sets the position and returns this value for further mutation.
    pub fn set_position(&mut self, value: i64) -> &mut Self {
        self.position = value;
        self
    }

    /// Returns the configured position.
    pub fn get_position(&self) -> i64 {
        self.position
    }

    /// Sets the position length and returns the updated value.
    pub fn position_length(mut self, value: i64) -> Self {
        self.position_length = value;
        self
    }

    /// Sets the position length and returns this value for further mutation.
    pub fn set_position_length(&mut self, value: i64) -> &mut Self {
        self.position_length = value;
        self
    }

    /// Returns the configured position length.
    pub fn get_position_length(&self) -> i64 {
        self.position_length
    }

    /// Sets the hash and returns the updated value.
    pub fn hash(mut self, value: u32) -> Self {
        self.hash = value;
        self
    }

    /// Sets the hash and returns this value for further mutation.
    pub fn set_hash(&mut self, value: u32) -> &mut Self {
        self.hash = value;
        self
    }

    /// Returns the configured hash.
    pub fn get_hash(&self) -> u32 {
        self.hash
    }
}

///////////////////////////////////////////////////////////////////////////////
// AnalyzerResult
///////////////////////////////////////////////////////////////////////////////
/// Tokens produced for one analyzed input string.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AnalyzerResult {
    pub(crate) tokens: Vec<AnalyzerToken>,
}

impl AnalyzerResult {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Sets the tokens and returns the updated value.
    pub fn tokens(mut self, value: Vec<AnalyzerToken>) -> Self {
        self.tokens = value;
        self
    }

    /// Sets the tokens and returns this value for further mutation.
    pub fn set_tokens(&mut self, value: Vec<AnalyzerToken>) -> &mut Self {
        self.tokens = value;
        self
    }

    /// Returns the configured tokens.
    pub fn get_tokens(&self) -> &[AnalyzerToken] {
        &self.tokens
    }

    /// Adds one add token to the existing values.
    pub fn add_token(mut self, value: AnalyzerToken) -> Self {
        self.tokens.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// RefreshExternalCollectionStateCode
///////////////////////////////////////////////////////////////////////////////
/// Execution state of a refresh-external-collection job.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RefreshExternalCollectionStateCode {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the Pending case.
    Pending,
    /// Represents the InProgress case.
    InProgress,
    /// Represents the Completed case.
    Completed,
    /// Represents the Failed case.
    Failed,
}

impl RefreshExternalCollectionStateCode {
    /// Returns the wire/display name of this state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "RefreshUnknown",
            Self::Pending => "RefreshPending",
            Self::InProgress => "RefreshInProgress",
            Self::Completed => "RefreshCompleted",
            Self::Failed => "RefreshFailed",
        }
    }

    pub(crate) fn from_proto(value: i32) -> Self {
        match milvus::RefreshExternalCollectionState::try_from(value).ok() {
            Some(milvus::RefreshExternalCollectionState::RefreshPending) => Self::Pending,
            Some(milvus::RefreshExternalCollectionState::RefreshInProgress) => Self::InProgress,
            Some(milvus::RefreshExternalCollectionState::RefreshCompleted) => Self::Completed,
            Some(milvus::RefreshExternalCollectionState::RefreshFailed) => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for RefreshExternalCollectionStateCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///////////////////////////////////////////////////////////////////////////////
// RefreshExternalCollectionJobInfo
///////////////////////////////////////////////////////////////////////////////
/// Metadata and progress of a refresh-external-collection job.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RefreshExternalCollectionJobInfo {
    pub(crate) job_id: i64,
    pub(crate) collection_name: String,
    pub(crate) state: RefreshExternalCollectionStateCode,
    pub(crate) progress: i32,
    pub(crate) reason: String,
    pub(crate) external_source: String,
    pub(crate) start_time: u64,
    pub(crate) end_time: u64,
}

impl RefreshExternalCollectionJobInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            job_id: 0,
            collection_name: String::new(),
            state: RefreshExternalCollectionStateCode::Unknown,
            progress: 0,
            reason: String::new(),
            external_source: String::new(),
            start_time: 0,
            end_time: 0,
        }
    }

    /// Sets the job id and returns the updated value.
    pub fn job_id(mut self, value: i64) -> Self {
        self.job_id = value;
        self
    }

    /// Sets the job id and returns this value for further mutation.
    pub fn set_job_id(&mut self, value: i64) -> &mut Self {
        self.job_id = value;
        self
    }

    /// Returns the job id.
    pub fn get_job_id(&self) -> i64 {
        self.job_id
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.collection_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn set_collection_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.collection_name = value.into();
        self
    }

    /// Returns the collection name.
    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Sets the state and returns the updated value.
    pub fn state(mut self, value: RefreshExternalCollectionStateCode) -> Self {
        self.state = value;
        self
    }

    /// Sets the state and returns this value for further mutation.
    pub fn set_state(&mut self, value: RefreshExternalCollectionStateCode) -> &mut Self {
        self.state = value;
        self
    }

    /// Returns the state.
    pub fn get_state(&self) -> RefreshExternalCollectionStateCode {
        self.state
    }

    /// Sets the progress percentage and returns the updated value.
    pub fn progress(mut self, value: i32) -> Self {
        self.progress = value;
        self
    }

    /// Sets the progress percentage and returns this value for further mutation.
    pub fn set_progress(&mut self, value: i32) -> &mut Self {
        self.progress = value;
        self
    }

    /// Returns the progress percentage in the range `0..=100`.
    pub fn get_progress(&self) -> i32 {
        self.progress
    }

    /// Sets the failure reason and returns the updated value.
    pub fn reason(mut self, value: impl Into<String>) -> Self {
        self.reason = value.into();
        self
    }

    /// Sets the failure reason and returns this value for further mutation.
    pub fn set_reason(&mut self, value: impl Into<String>) -> &mut Self {
        self.reason = value.into();
        self
    }

    /// Returns the failure reason.
    pub fn get_reason(&self) -> &str {
        &self.reason
    }

    /// Sets the external source and returns the updated value.
    pub fn external_source(mut self, value: impl Into<String>) -> Self {
        self.external_source = value.into();
        self
    }

    /// Sets the external source and returns this value for further mutation.
    pub fn set_external_source(&mut self, value: impl Into<String>) -> &mut Self {
        self.external_source = value.into();
        self
    }

    /// Returns the external source.
    pub fn get_external_source(&self) -> &str {
        &self.external_source
    }

    /// Sets the start time and returns the updated value.
    pub fn start_time(mut self, value: u64) -> Self {
        self.start_time = value;
        self
    }

    /// Sets the start time and returns this value for further mutation.
    pub fn set_start_time(&mut self, value: u64) -> &mut Self {
        self.start_time = value;
        self
    }

    /// Returns the start time.
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }

    /// Sets the end time and returns the updated value.
    pub fn end_time(mut self, value: u64) -> Self {
        self.end_time = value;
        self
    }

    /// Sets the end time and returns this value for further mutation.
    pub fn set_end_time(&mut self, value: u64) -> &mut Self {
        self.end_time = value;
        self
    }

    /// Returns the end time.
    pub fn get_end_time(&self) -> u64 {
        self.end_time
    }

    pub(crate) fn from_proto(value: milvus::RefreshExternalCollectionJobInfo) -> Self {
        Self {
            job_id: value.job_id,
            collection_name: value.collection_name,
            state: RefreshExternalCollectionStateCode::from_proto(value.state),
            progress: value.progress as i32,
            reason: value.reason,
            external_source: value.external_source,
            start_time: value.start_time as u64,
            end_time: value.end_time as u64,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FileResourceInfo
///////////////////////////////////////////////////////////////////////////////
/// Metadata of a named file resource registered for external-table workflows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileResourceInfo {
    pub(crate) name: String,
    pub(crate) path: String,
}

impl FileResourceInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Returns the name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the path and returns the updated value.
    pub fn path(mut self, value: impl Into<String>) -> Self {
        self.path = value.into();
        self
    }

    /// Sets the path and returns this value for further mutation.
    pub fn set_path(&mut self, value: impl Into<String>) -> &mut Self {
        self.path = value.into();
        self
    }

    /// Returns the path.
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub(crate) fn from_proto(value: milvus::FileResourceInfo) -> Self {
        Self {
            name: value.name,
            path: value.path,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod state_enum_tests {
    use super::{CompactionStateCode, SegmentLevel, SegmentState};
    use crate::proto::common;

    #[test]
    fn segment_state_converts_from_proto() {
        assert_eq!(
            SegmentState::from_proto(common::SegmentState::Flushed as i32),
            SegmentState::Flushed
        );
        assert_eq!(
            SegmentState::from_proto(common::SegmentState::Importing as i32),
            SegmentState::Unknown
        );
    }

    #[test]
    fn segment_level_converts_from_proto() {
        assert_eq!(
            SegmentLevel::from_proto(common::SegmentLevel::L2 as i32),
            SegmentLevel::L2
        );
        assert_eq!(SegmentLevel::from_proto(i32::MAX), SegmentLevel::Unknown);
    }

    #[test]
    fn compaction_state_code_converts_from_proto() {
        assert_eq!(
            CompactionStateCode::from_proto(common::CompactionState::Completed as i32),
            CompactionStateCode::Completed
        );
        assert_eq!(
            CompactionStateCode::from_proto(i32::MAX),
            CompactionStateCode::Unknown
        );
    }
}

#[cfg(test)]
mod refresh_external_collection_state_tests {
    use super::RefreshExternalCollectionStateCode;
    use crate::proto::milvus;

    #[test]
    fn refresh_external_collection_state_as_str_and_display() {
        let cases = [
            (
                RefreshExternalCollectionStateCode::Unknown,
                "RefreshUnknown",
            ),
            (
                RefreshExternalCollectionStateCode::Pending,
                "RefreshPending",
            ),
            (
                RefreshExternalCollectionStateCode::InProgress,
                "RefreshInProgress",
            ),
            (
                RefreshExternalCollectionStateCode::Completed,
                "RefreshCompleted",
            ),
            (RefreshExternalCollectionStateCode::Failed, "RefreshFailed"),
        ];
        for (state, expected) in cases {
            assert_eq!(state.as_str(), expected);
            assert_eq!(state.to_string(), expected);
        }
    }

    #[test]
    fn refresh_external_collection_state_maps_unknown_proto_values() {
        assert_eq!(
            RefreshExternalCollectionStateCode::from_proto(
                milvus::RefreshExternalCollectionState::RefreshPending as i32
            ),
            RefreshExternalCollectionStateCode::Pending
        );
        assert_eq!(
            RefreshExternalCollectionStateCode::from_proto(i32::MAX),
            RefreshExternalCollectionStateCode::Unknown
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn persistent_segment_info_default_values() {
        let value = PersistentSegmentInfo::new();
        let expected_segment_id: i64 = 0;
        let expected_collection_id: i64 = 0;
        let expected_partition_id: i64 = 0;
        let expected_row_count: i64 = 0;
        let expected_state: SegmentState = Default::default();
        let expected_collection_name: String = String::new();
        let expected_level: SegmentLevel = Default::default();
        let expected_sorted: bool = false;
        let expected_storage_version: i64 = 0;

        assert_eq!(value.get_segment_id().to_owned(), expected_segment_id);
        assert_eq!(value.get_collection_id().to_owned(), expected_collection_id);
        assert_eq!(value.get_partition_id().to_owned(), expected_partition_id);
        assert_eq!(value.get_row_count().to_owned(), expected_row_count);
        assert_eq!(value.get_state().to_owned(), expected_state);
        assert_eq!(
            value.get_collection_name().to_owned(),
            expected_collection_name
        );
        assert_eq!(value.get_level().to_owned(), expected_level);
        assert_eq!(value.get_sorted().to_owned(), expected_sorted);
        assert_eq!(
            value.get_storage_version().to_owned(),
            expected_storage_version
        );
    }

    #[test]
    fn persistent_segment_info_populated_values() {
        let segment_id = 7;
        let collection_id = 7;
        let partition_id = 7;
        let row_count = 7;
        let state = SegmentState::Flushed;
        let collection_name = "collection_name-value".to_owned();
        let level = SegmentLevel::L1;
        let sorted = true;
        let storage_version = 7;
        let value = PersistentSegmentInfo::new()
            .segment_id(segment_id.clone())
            .collection_id(collection_id.clone())
            .partition_id(partition_id.clone())
            .row_count(row_count.clone())
            .state(state.clone())
            .collection_name(collection_name.clone())
            .level(level.clone())
            .sorted(sorted.clone())
            .storage_version(storage_version.clone());

        assert_eq!(value.get_segment_id().to_owned(), segment_id);
        assert_eq!(value.get_collection_id().to_owned(), collection_id);
        assert_eq!(value.get_partition_id().to_owned(), partition_id);
        assert_eq!(value.get_row_count().to_owned(), row_count);
        assert_eq!(value.get_state().to_owned(), state);
        assert_eq!(value.get_collection_name().to_owned(), collection_name);
        assert_eq!(value.get_level().to_owned(), level);
        assert_eq!(value.get_sorted().to_owned(), sorted);
        assert_eq!(value.get_storage_version().to_owned(), storage_version);
    }

    #[test]
    fn query_segment_info_default_values() {
        let value = QuerySegmentInfo::new();
        let expected_segment_id: i64 = 0;
        let expected_collection_id: i64 = 0;
        let expected_partition_id: i64 = 0;
        let expected_memory_size: i64 = 0;
        let expected_row_count: i64 = 0;
        let expected_index_name: String = String::new();
        let expected_index_id: i64 = 0;
        let expected_node_ids: Vec<i64> = Default::default();
        let expected_state: SegmentState = Default::default();
        let expected_collection_name: String = String::new();
        let expected_level: SegmentLevel = Default::default();
        let expected_sorted: bool = false;
        let expected_storage_version: i64 = 0;

        assert_eq!(value.get_segment_id().to_owned(), expected_segment_id);
        assert_eq!(value.get_collection_id().to_owned(), expected_collection_id);
        assert_eq!(value.get_partition_id().to_owned(), expected_partition_id);
        assert_eq!(value.get_memory_size().to_owned(), expected_memory_size);
        assert_eq!(value.get_row_count().to_owned(), expected_row_count);
        assert_eq!(value.get_index_name().to_owned(), expected_index_name);
        assert_eq!(value.get_index_id().to_owned(), expected_index_id);
        assert_eq!(value.get_node_ids().to_owned(), expected_node_ids);
        assert_eq!(value.get_state().to_owned(), expected_state);
        assert_eq!(
            value.get_collection_name().to_owned(),
            expected_collection_name
        );
        assert_eq!(value.get_level().to_owned(), expected_level);
        assert_eq!(value.get_sorted().to_owned(), expected_sorted);
        assert_eq!(
            value.get_storage_version().to_owned(),
            expected_storage_version
        );
    }

    #[test]
    fn query_segment_info_populated_values() {
        let segment_id = 7;
        let collection_id = 7;
        let partition_id = 7;
        let memory_size = 7;
        let row_count = 7;
        let index_name = "index_name-value".to_owned();
        let index_id = 7;
        let node_ids = vec![7];
        let state = SegmentState::Flushed;
        let collection_name = "collection_name-value".to_owned();
        let level = SegmentLevel::L1;
        let sorted = true;
        let storage_version = 7;
        let value = QuerySegmentInfo::new()
            .segment_id(segment_id.clone())
            .collection_id(collection_id.clone())
            .partition_id(partition_id.clone())
            .memory_size(memory_size.clone())
            .row_count(row_count.clone())
            .index_name(index_name.clone())
            .index_id(index_id.clone())
            .node_ids(node_ids.clone())
            .state(state.clone())
            .collection_name(collection_name.clone())
            .level(level.clone())
            .sorted(sorted.clone())
            .storage_version(storage_version.clone());

        assert_eq!(value.get_segment_id().to_owned(), segment_id);
        assert_eq!(value.get_collection_id().to_owned(), collection_id);
        assert_eq!(value.get_partition_id().to_owned(), partition_id);
        assert_eq!(value.get_memory_size().to_owned(), memory_size);
        assert_eq!(value.get_row_count().to_owned(), row_count);
        assert_eq!(value.get_index_name().to_owned(), index_name);
        assert_eq!(value.get_index_id().to_owned(), index_id);
        assert_eq!(value.get_node_ids().to_owned(), node_ids);
        assert_eq!(value.get_state().to_owned(), state);
        assert_eq!(value.get_collection_name().to_owned(), collection_name);
        assert_eq!(value.get_level().to_owned(), level);
        assert_eq!(value.get_sorted().to_owned(), sorted);
        assert_eq!(value.get_storage_version().to_owned(), storage_version);
    }

    #[test]
    fn compaction_merge_default_values() {
        let value = CompactionMerge::new();
        let expected_source_segment_ids: Vec<i64> = Default::default();
        let expected_target_segment_id: i64 = 0;

        assert_eq!(
            value.get_source_segment_ids().to_owned(),
            expected_source_segment_ids
        );
        assert_eq!(
            value.get_target_segment_id().to_owned(),
            expected_target_segment_id
        );
    }

    #[test]
    fn compaction_merge_populated_values() {
        let source_segment_ids = vec![7];
        let target_segment_id = 7;
        let value = CompactionMerge::new()
            .source_segment_ids(source_segment_ids.clone())
            .target_segment_id(target_segment_id.clone());

        assert_eq!(
            value.get_source_segment_ids().to_owned(),
            source_segment_ids
        );
        assert_eq!(value.get_target_segment_id().to_owned(), target_segment_id);
    }

    #[test]
    fn analyzer_token_default_values() {
        let value = AnalyzerToken::new();
        let expected_text: String = String::new();
        let expected_start_offset: i64 = 0;
        let expected_end_offset: i64 = 0;
        let expected_position: i64 = 0;
        let expected_position_length: i64 = 0;
        let expected_hash: u32 = 0;

        assert_eq!(value.get_text().to_owned(), expected_text);
        assert_eq!(value.get_start_offset().to_owned(), expected_start_offset);
        assert_eq!(value.get_end_offset().to_owned(), expected_end_offset);
        assert_eq!(value.get_position().to_owned(), expected_position);
        assert_eq!(
            value.get_position_length().to_owned(),
            expected_position_length
        );
        assert_eq!(value.get_hash().to_owned(), expected_hash);
    }

    #[test]
    fn analyzer_token_populated_values() {
        let text = "text-value".to_owned();
        let start_offset = 7;
        let end_offset = 7;
        let position = 7;
        let position_length = 7;
        let hash = 7;
        let value = AnalyzerToken::new()
            .text(text.clone())
            .start_offset(start_offset.clone())
            .end_offset(end_offset.clone())
            .position(position.clone())
            .position_length(position_length.clone())
            .hash(hash.clone());

        assert_eq!(value.get_text().to_owned(), text);
        assert_eq!(value.get_start_offset().to_owned(), start_offset);
        assert_eq!(value.get_end_offset().to_owned(), end_offset);
        assert_eq!(value.get_position().to_owned(), position);
        assert_eq!(value.get_position_length().to_owned(), position_length);
        assert_eq!(value.get_hash().to_owned(), hash);
    }

    #[test]
    fn analyzer_result_default_values() {
        let value = AnalyzerResult::new();
        let expected_tokens: Vec<AnalyzerToken> = Default::default();

        assert_eq!(value.get_tokens().to_owned(), expected_tokens);
    }

    #[test]
    fn analyzer_result_populated_values() {
        let tokens = vec![AnalyzerToken::new()];
        let value = AnalyzerResult::new().tokens(tokens.clone());

        assert_eq!(value.get_tokens().to_owned(), tokens);
    }
}
