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

use crate::proto::common;

///////////////////////////////////////////////////////////////////////////////
// SegmentState
///////////////////////////////////////////////////////////////////////////////
/// Lifecycle state of a Milvus segment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SegmentState {
    #[default]
    Unknown,
    NotExist,
    Growing,
    Sealed,
    Flushed,
    Flushing,
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
    Unknown,
    Legacy,
    L0,
    L1,
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
    Unknown,
    Executing,
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

    pub fn segment_id(mut self, value: i64) -> Self {
        self.segment_id = value;
        self
    }

    pub fn set_segment_id(&mut self, value: i64) -> &mut Self {
        self.segment_id = value;
        self
    }

    pub fn get_segment_id(&self) -> i64 {
        self.segment_id
    }

    pub fn collection_id(mut self, value: i64) -> Self {
        self.collection_id = value;
        self
    }

    pub fn set_collection_id(&mut self, value: i64) -> &mut Self {
        self.collection_id = value;
        self
    }

    pub fn get_collection_id(&self) -> i64 {
        self.collection_id
    }

    pub fn partition_id(mut self, value: i64) -> Self {
        self.partition_id = value;
        self
    }

    pub fn set_partition_id(&mut self, value: i64) -> &mut Self {
        self.partition_id = value;
        self
    }

    pub fn get_partition_id(&self) -> i64 {
        self.partition_id
    }

    pub fn row_count(mut self, value: i64) -> Self {
        self.row_count = value;
        self
    }

    pub fn set_row_count(&mut self, value: i64) -> &mut Self {
        self.row_count = value;
        self
    }

    pub fn get_row_count(&self) -> i64 {
        self.row_count
    }

    pub fn state(mut self, value: SegmentState) -> Self {
        self.state = value;
        self
    }

    pub fn set_state(&mut self, value: SegmentState) -> &mut Self {
        self.state = value;
        self
    }

    pub fn get_state(&self) -> SegmentState {
        self.state
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.collection_name = value.into();
        self
    }

    pub fn set_collection_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.collection_name = value.into();
        self
    }

    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn level(mut self, value: SegmentLevel) -> Self {
        self.level = value;
        self
    }

    pub fn set_level(&mut self, value: SegmentLevel) -> &mut Self {
        self.level = value;
        self
    }

    pub fn get_level(&self) -> SegmentLevel {
        self.level
    }

    pub fn sorted(mut self, value: bool) -> Self {
        self.sorted = value;
        self
    }

    pub fn set_sorted(&mut self, value: bool) -> &mut Self {
        self.sorted = value;
        self
    }

    pub fn get_sorted(&self) -> bool {
        self.sorted
    }

    pub fn storage_version(mut self, value: i64) -> Self {
        self.storage_version = value;
        self
    }

    pub fn set_storage_version(&mut self, value: i64) -> &mut Self {
        self.storage_version = value;
        self
    }

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

    pub fn segment_id(mut self, value: i64) -> Self {
        self.segment_id = value;
        self
    }

    pub fn set_segment_id(&mut self, value: i64) -> &mut Self {
        self.segment_id = value;
        self
    }

    pub fn get_segment_id(&self) -> i64 {
        self.segment_id
    }

    pub fn collection_id(mut self, value: i64) -> Self {
        self.collection_id = value;
        self
    }

    pub fn set_collection_id(&mut self, value: i64) -> &mut Self {
        self.collection_id = value;
        self
    }

    pub fn get_collection_id(&self) -> i64 {
        self.collection_id
    }

    pub fn partition_id(mut self, value: i64) -> Self {
        self.partition_id = value;
        self
    }

    pub fn set_partition_id(&mut self, value: i64) -> &mut Self {
        self.partition_id = value;
        self
    }

    pub fn get_partition_id(&self) -> i64 {
        self.partition_id
    }

    pub fn memory_size(mut self, value: i64) -> Self {
        self.memory_size = value;
        self
    }

    pub fn set_memory_size(&mut self, value: i64) -> &mut Self {
        self.memory_size = value;
        self
    }

    pub fn get_memory_size(&self) -> i64 {
        self.memory_size
    }

    pub fn row_count(mut self, value: i64) -> Self {
        self.row_count = value;
        self
    }

    pub fn set_row_count(&mut self, value: i64) -> &mut Self {
        self.row_count = value;
        self
    }

    pub fn get_row_count(&self) -> i64 {
        self.row_count
    }

    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.index_name = value.into();
        self
    }

    pub fn set_index_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.index_name = value.into();
        self
    }

    pub fn get_index_name(&self) -> &str {
        &self.index_name
    }

    pub fn index_id(mut self, value: i64) -> Self {
        self.index_id = value;
        self
    }

    pub fn set_index_id(&mut self, value: i64) -> &mut Self {
        self.index_id = value;
        self
    }

    pub fn get_index_id(&self) -> i64 {
        self.index_id
    }

    pub fn node_ids(mut self, value: Vec<i64>) -> Self {
        self.node_ids = value;
        self
    }

    pub fn set_node_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.node_ids = value;
        self
    }

    pub fn get_node_ids(&self) -> &[i64] {
        &self.node_ids
    }

    pub fn state(mut self, value: SegmentState) -> Self {
        self.state = value;
        self
    }

    pub fn set_state(&mut self, value: SegmentState) -> &mut Self {
        self.state = value;
        self
    }

    pub fn get_state(&self) -> SegmentState {
        self.state
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.collection_name = value.into();
        self
    }

    pub fn set_collection_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.collection_name = value.into();
        self
    }

    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn level(mut self, value: SegmentLevel) -> Self {
        self.level = value;
        self
    }

    pub fn set_level(&mut self, value: SegmentLevel) -> &mut Self {
        self.level = value;
        self
    }

    pub fn get_level(&self) -> SegmentLevel {
        self.level
    }

    pub fn sorted(mut self, value: bool) -> Self {
        self.sorted = value;
        self
    }

    pub fn set_sorted(&mut self, value: bool) -> &mut Self {
        self.sorted = value;
        self
    }

    pub fn get_sorted(&self) -> bool {
        self.sorted
    }

    pub fn storage_version(mut self, value: i64) -> Self {
        self.storage_version = value;
        self
    }

    pub fn set_storage_version(&mut self, value: i64) -> &mut Self {
        self.storage_version = value;
        self
    }

    pub fn get_storage_version(&self) -> i64 {
        self.storage_version
    }

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
    pub fn new() -> Self {
        Self {
            source_segment_ids: Vec::new(),
            target_segment_id: 0,
        }
    }

    pub fn source_segment_ids(mut self, value: Vec<i64>) -> Self {
        self.source_segment_ids = value;
        self
    }

    pub fn set_source_segment_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.source_segment_ids = value;
        self
    }

    pub fn get_source_segment_ids(&self) -> &[i64] {
        &self.source_segment_ids
    }

    pub fn target_segment_id(mut self, value: i64) -> Self {
        self.target_segment_id = value;
        self
    }

    pub fn set_target_segment_id(&mut self, value: i64) -> &mut Self {
        self.target_segment_id = value;
        self
    }

    pub fn get_target_segment_id(&self) -> i64 {
        self.target_segment_id
    }

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

    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn set_text(&mut self, value: impl Into<String>) -> &mut Self {
        self.text = value.into();
        self
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn start_offset(mut self, value: i64) -> Self {
        self.start_offset = value;
        self
    }

    pub fn set_start_offset(&mut self, value: i64) -> &mut Self {
        self.start_offset = value;
        self
    }

    pub fn get_start_offset(&self) -> i64 {
        self.start_offset
    }

    pub fn end_offset(mut self, value: i64) -> Self {
        self.end_offset = value;
        self
    }

    pub fn set_end_offset(&mut self, value: i64) -> &mut Self {
        self.end_offset = value;
        self
    }

    pub fn get_end_offset(&self) -> i64 {
        self.end_offset
    }

    pub fn position(mut self, value: i64) -> Self {
        self.position = value;
        self
    }

    pub fn set_position(&mut self, value: i64) -> &mut Self {
        self.position = value;
        self
    }

    pub fn get_position(&self) -> i64 {
        self.position
    }

    pub fn position_length(mut self, value: i64) -> Self {
        self.position_length = value;
        self
    }

    pub fn set_position_length(&mut self, value: i64) -> &mut Self {
        self.position_length = value;
        self
    }

    pub fn get_position_length(&self) -> i64 {
        self.position_length
    }

    pub fn hash(mut self, value: u32) -> Self {
        self.hash = value;
        self
    }

    pub fn set_hash(&mut self, value: u32) -> &mut Self {
        self.hash = value;
        self
    }

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
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn tokens(mut self, value: Vec<AnalyzerToken>) -> Self {
        self.tokens = value;
        self
    }

    pub fn set_tokens(&mut self, value: Vec<AnalyzerToken>) -> &mut Self {
        self.tokens = value;
        self
    }

    pub fn get_tokens(&self) -> &[AnalyzerToken] {
        &self.tokens
    }

    pub fn add_token(mut self, value: AnalyzerToken) -> Self {
        self.tokens.push(value);
        self
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
