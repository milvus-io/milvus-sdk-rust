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

//! Request types for utility, maintenance, health, and segment operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::{non_empty_strings, positive_i64, required, required_slice};
use crate::v2::types::TargetSizeUnit;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// GetServerVersionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 server_version operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetServerVersionRequest {
    pub(crate) detail: bool,
}

impl GetServerVersionRequest {
    fn empty() -> Self {
        Self { detail: false }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetServerVersionRequestBuilder {
        GetServerVersionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetServerVersionRequestBuilder {
        GetServerVersionRequestBuilder { value: self }
    }

    /// Returns whether detail enabled.
    pub fn is_detail_enabled(&self) -> bool {
        self.detail
    }

    pub(crate) fn into_get_version_proto(self) -> milvus::GetVersionRequest {
        milvus::GetVersionRequest::default()
    }

    pub(crate) fn into_connect_proto(self) -> milvus::ConnectRequest {
        milvus::ConnectRequest::default()
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetServerVersionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetServerVersionRequest.
#[derive(Debug, Clone)]
pub struct GetServerVersionRequestBuilder {
    value: GetServerVersionRequest,
}

impl GetServerVersionRequestBuilder {
    /// Sets the detail and returns the updated value.
    pub fn detail(mut self, value: bool) -> Self {
        self.value.detail = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetServerVersionRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CheckHealthRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 check_health operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckHealthRequest;

impl CheckHealthRequest {
    /// Creates a builder for this request.
    pub fn builder() -> CheckHealthRequestBuilder {
        CheckHealthRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CheckHealthRequestBuilder {
        CheckHealthRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::CheckHealthRequest {
        milvus::CheckHealthRequest::default()
    }
}

///////////////////////////////////////////////////////////////////////////////
// CheckHealthRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CheckHealthRequest.
#[derive(Debug, Clone, Copy)]
pub struct CheckHealthRequestBuilder;

impl CheckHealthRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CheckHealthRequest> {
        Ok(CheckHealthRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 flush operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlushRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_names: Vec<String>,
    /// Maximum time to wait for all returned segments to be flushed.
    /// Zero waits indefinitely; negative values are invalid.
    pub(crate) wait_flushed_ms: i64,
}

impl FlushRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_names: Default::default(),
            wait_flushed_ms: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> FlushRequestBuilder {
        FlushRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> FlushRequestBuilder {
        FlushRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection names.
    pub fn collection_names(&self) -> &[String] {
        &self.collection_names
    }

    /// Returns the wait flushed ms.
    pub fn wait_flushed_ms(&self) -> i64 {
        self.wait_flushed_ms
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::FlushRequest {
        milvus::FlushRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_names: self.collection_names,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for FlushRequest.
#[derive(Debug, Clone)]
pub struct FlushRequestBuilder {
    value: FlushRequest,
}

impl FlushRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection names and returns the updated value.
    pub fn collection_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.collection_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the maximum flush wait in milliseconds.
    ///
    /// Zero waits indefinitely. Negative values are rejected by [`Self::build`].
    pub fn wait_flushed_ms(mut self, value: i64) -> Self {
        self.value.wait_flushed_ms = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<FlushRequest> {
        required_slice("collection_names", &self.value.collection_names)?;
        non_empty_strings("collection_names", &self.value.collection_names)?;
        if self.value.wait_flushed_ms < 0 {
            return Err(Error::validation(
                "wait_flushed_ms".into(),
                "must not be negative".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushAllRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 flush_all operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlushAllRequest {
    pub(crate) database_name: Option<String>,
    /// Maximum time to wait for the cluster-wide flush to complete.
    /// Zero waits indefinitely; negative values are invalid.
    pub(crate) wait_flushed_ms: i64,
}

impl FlushAllRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            wait_flushed_ms: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> FlushAllRequestBuilder {
        FlushAllRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> FlushAllRequestBuilder {
        FlushAllRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the wait flushed ms.
    pub fn wait_flushed_ms(&self) -> i64 {
        self.wait_flushed_ms
    }

    // Milvus 2.6 still accepts the deprecated database selector.
    #[allow(deprecated)]
    pub(crate) fn into_proto(self, default_db: &str) -> milvus::FlushAllRequest {
        let mut value = milvus::FlushAllRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// FlushAllRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for FlushAllRequest.
#[derive(Debug, Clone)]
pub struct FlushAllRequestBuilder {
    value: FlushAllRequest,
}

impl FlushAllRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the maximum cluster-wide flush wait in milliseconds.
    ///
    /// Zero waits indefinitely. Negative values are rejected by [`Self::build`].
    pub fn wait_flushed_ms(mut self, value: i64) -> Self {
        self.value.wait_flushed_ms = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<FlushAllRequest> {
        if self.value.wait_flushed_ms < 0 {
            return Err(Error::validation(
                "wait_flushed_ms".into(),
                "must not be negative".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetFlushAllStateRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_flush_all_state operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetFlushAllStateRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) flush_all_timestamp: u64,
    pub(crate) channel_timestamps: HashMap<String, u64>,
}

impl GetFlushAllStateRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            flush_all_timestamp: Default::default(),
            channel_timestamps: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetFlushAllStateRequestBuilder {
        GetFlushAllStateRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetFlushAllStateRequestBuilder {
        GetFlushAllStateRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the flush all timestamp.
    pub fn flush_all_timestamp(&self) -> u64 {
        self.flush_all_timestamp
    }

    /// Returns the channel timestamps.
    pub fn channel_timestamps(&self) -> &HashMap<String, u64> {
        &self.channel_timestamps
    }

    // Milvus 2.6 accepts the legacy aggregate timestamp/database fields alongside
    // the per-channel timestamps used by newer flush-all responses.
    #[allow(deprecated)]
    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetFlushAllStateRequest {
        let mut value = milvus::GetFlushAllStateRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.flush_all_ts = self.flush_all_timestamp;
        value.flush_all_tss = self.channel_timestamps;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetFlushAllStateRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetFlushAllStateRequest.
#[derive(Debug, Clone)]
pub struct GetFlushAllStateRequestBuilder {
    value: GetFlushAllStateRequest,
}

impl GetFlushAllStateRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the flush all timestamp and returns the updated value.
    pub fn flush_all_timestamp(mut self, value: u64) -> Self {
        self.value.flush_all_timestamp = value;
        self
    }

    /// Sets the channel timestamps and returns the updated value.
    pub fn channel_timestamps(mut self, value: HashMap<String, u64>) -> Self {
        self.value.channel_timestamps = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetFlushAllStateRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPersistentSegmentsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_persistent_segments operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPersistentSegmentsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListPersistentSegmentsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListPersistentSegmentsRequestBuilder {
        ListPersistentSegmentsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListPersistentSegmentsRequestBuilder {
        ListPersistentSegmentsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetPersistentSegmentInfoRequest {
        let mut value = milvus::GetPersistentSegmentInfoRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPersistentSegmentsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListPersistentSegmentsRequest.
#[derive(Debug, Clone)]
pub struct ListPersistentSegmentsRequestBuilder {
    value: ListPersistentSegmentsRequest,
}

impl ListPersistentSegmentsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListPersistentSegmentsRequest> {
        validate_collection_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListQuerySegmentsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_query_segments operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListQuerySegmentsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListQuerySegmentsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListQuerySegmentsRequestBuilder {
        ListQuerySegmentsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListQuerySegmentsRequestBuilder {
        ListQuerySegmentsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetQuerySegmentInfoRequest {
        let mut value = milvus::GetQuerySegmentInfoRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListQuerySegmentsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListQuerySegmentsRequest.
#[derive(Debug, Clone)]
pub struct ListQuerySegmentsRequestBuilder {
    value: ListQuerySegmentsRequest,
}

impl ListQuerySegmentsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListQuerySegmentsRequest> {
        validate_collection_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CompactRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 compact operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompactRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    /// Target segment size after compaction, expressed in `target_size_unit`.
    /// Zero means the server chooses its default target size.
    pub(crate) target_size: i64,
    /// Unit of [`Self::target_size`]; the value is converted to MB before being sent to Milvus.
    pub(crate) target_size_unit: TargetSizeUnit,
    pub(crate) clustering_compaction: bool,
    pub(crate) is_l0: bool,
}

impl CompactRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            target_size: Default::default(),
            target_size_unit: Default::default(),
            clustering_compaction: Default::default(),
            is_l0: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CompactRequestBuilder {
        CompactRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CompactRequestBuilder {
        CompactRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the target size in the configured unit.
    pub fn target_size(&self) -> i64 {
        self.target_size
    }

    /// Returns the target size unit.
    pub fn target_size_unit(&self) -> TargetSizeUnit {
        self.target_size_unit
    }

    /// Returns whether clustering compaction.
    pub fn is_clustering_compaction(&self) -> bool {
        self.clustering_compaction
    }

    /// Returns whether L0 compaction.
    pub fn is_l0(&self) -> bool {
        self.is_l0
    }

    /// Returns the effective target size in megabytes, `None` when no target size was set.
    pub(crate) fn target_size_mb(&self) -> Result<Option<i64>> {
        if self.target_size == 0 {
            return Ok(None);
        }
        let mb = target_size_to_mb(self.target_size, self.target_size_unit)?;
        if mb < 1 {
            return Err(Error::validation(
                "target_size".into(),
                "target size is too small and rounds to zero MB".into(),
            ));
        }
        Ok(Some(mb))
    }

    pub(crate) fn into_proto(self, default_db: &str) -> Result<milvus::ManualCompactionRequest> {
        let target_size = self.target_size_mb()?.unwrap_or_default();
        Ok(milvus::ManualCompactionRequest {
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            target_size,
            major_compaction: self.clustering_compaction,
            l0_compaction: self.is_l0,
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// CompactRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CompactRequest.
#[derive(Debug, Clone)]
pub struct CompactRequestBuilder {
    value: CompactRequest,
}

impl CompactRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the target size and returns the updated value.
    ///
    /// The value is interpreted in the configured [`Self::target_size_unit`] (MB by default) and
    /// converted to MB before being sent to Milvus. Zero lets the server choose its default.
    pub fn target_size(mut self, value: i64) -> Self {
        self.value.target_size = value;
        self
    }

    /// Sets the target size unit and returns the updated value.
    pub fn target_size_unit(mut self, value: TargetSizeUnit) -> Self {
        self.value.target_size_unit = value;
        self
    }

    /// Sets the clustering compaction and returns the updated value.
    pub fn clustering_compaction(mut self, value: bool) -> Self {
        self.value.clustering_compaction = value;
        self
    }

    /// Sets the L0 compaction and returns the updated value.
    pub fn is_l0(mut self, value: bool) -> Self {
        self.value.is_l0 = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CompactRequest> {
        validate_collection_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        if self.value.target_size < 0 {
            return Err(Error::validation(
                "target_size".into(),
                "must not be negative".into(),
            ));
        }
        self.value.target_size_mb()?;
        Ok(self.value)
    }
}

/// Converts a target size in the given unit to megabytes, matching pymilvus's
/// `b/kb/mb/gb/tb/pb` to MB conversion. Returns an error when the value is out of range.
fn target_size_to_mb(target_size: i64, unit: TargetSizeUnit) -> Result<i64> {
    const MB_BYTES: i64 = 1024 * 1024;
    let bytes = i128::from(target_size) * i128::from(unit.bytes_per_unit());
    let megabytes = bytes / i128::from(MB_BYTES);
    if megabytes > i128::from(i64::MAX) {
        return Err(Error::validation(
            "target_size".into(),
            "target size is too large".into(),
        ));
    }
    Ok(megabytes as i64)
}

///////////////////////////////////////////////////////////////////////////////
// OptimizeRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 optimize operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OptimizeRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    /// Target segment size such as `512MB` or `1GB`. Empty means that no
    /// explicit target size is sent to Milvus.
    pub(crate) target_size: String,
    /// Start the optimization in a Tokio task and return immediately.
    pub(crate) async_mode: bool,
    /// Overall task timeout in milliseconds. A value less than or equal to
    /// zero means no overall timeout.
    pub(crate) timeout_ms: i64,
}

impl OptimizeRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            target_size: Default::default(),
            async_mode: Default::default(),
            timeout_ms: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> OptimizeRequestBuilder {
        OptimizeRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> OptimizeRequestBuilder {
        OptimizeRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the target size.
    pub fn target_size(&self) -> &str {
        &self.target_size
    }

    /// Returns whether async.
    pub fn is_async(&self) -> bool {
        self.async_mode
    }

    /// Returns the timeout ms.
    pub fn timeout_ms(&self) -> i64 {
        self.timeout_ms
    }
}

///////////////////////////////////////////////////////////////////////////////
// OptimizeRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for OptimizeRequest.
#[derive(Debug, Clone)]
pub struct OptimizeRequestBuilder {
    value: OptimizeRequest,
}

impl OptimizeRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the target size and returns the updated value.
    pub fn target_size(mut self, value: impl Into<String>) -> Self {
        self.value.target_size = value.into();
        self
    }

    /// Sets the async mode and returns the updated value.
    pub fn async_mode(mut self, value: bool) -> Self {
        self.value.async_mode = value;
        self
    }

    /// Sets the timeout ms and returns the updated value.
    pub fn timeout_ms(mut self, value: i64) -> Self {
        self.value.timeout_ms = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<OptimizeRequest> {
        validate_collection_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionStateRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_compaction_state operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetCompactionStateRequest {
    pub(crate) compaction_id: i64,
}

impl GetCompactionStateRequest {
    fn empty() -> Self {
        Self {
            compaction_id: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetCompactionStateRequestBuilder {
        GetCompactionStateRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetCompactionStateRequestBuilder {
        GetCompactionStateRequestBuilder { value: self }
    }

    /// Returns the compaction id.
    pub fn compaction_id(&self) -> i64 {
        self.compaction_id
    }

    pub(crate) fn into_proto(self) -> milvus::GetCompactionStateRequest {
        milvus::GetCompactionStateRequest {
            compaction_id: self.compaction_id,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionStateRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetCompactionStateRequest.
#[derive(Debug, Clone)]
pub struct GetCompactionStateRequestBuilder {
    value: GetCompactionStateRequest,
}

impl GetCompactionStateRequestBuilder {
    /// Sets the compaction id and returns the updated value.
    pub fn compaction_id(mut self, value: i64) -> Self {
        self.value.compaction_id = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetCompactionStateRequest> {
        positive_i64("compaction_id", self.value.compaction_id)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionPlansRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_compaction_plans operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetCompactionPlansRequest {
    pub(crate) compaction_id: i64,
}

impl GetCompactionPlansRequest {
    fn empty() -> Self {
        Self {
            compaction_id: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetCompactionPlansRequestBuilder {
        GetCompactionPlansRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetCompactionPlansRequestBuilder {
        GetCompactionPlansRequestBuilder { value: self }
    }

    /// Returns the compaction id.
    pub fn compaction_id(&self) -> i64 {
        self.compaction_id
    }

    pub(crate) fn into_proto(self) -> milvus::GetCompactionPlansRequest {
        milvus::GetCompactionPlansRequest {
            compaction_id: self.compaction_id,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCompactionPlansRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetCompactionPlansRequest.
#[derive(Debug, Clone)]
pub struct GetCompactionPlansRequestBuilder {
    value: GetCompactionPlansRequest,
}

impl GetCompactionPlansRequestBuilder {
    /// Sets the compaction id and returns the updated value.
    pub fn compaction_id(mut self, value: i64) -> Self {
        self.value.compaction_id = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetCompactionPlansRequest> {
        positive_i64("compaction_id", self.value.compaction_id)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RunAnalyzerRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 run_analyzer operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RunAnalyzerRequest {
    pub(crate) analyzer_params: String,
    pub(crate) texts: Vec<String>,
    pub(crate) with_detail: bool,
    pub(crate) with_hash: bool,
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: String,
    pub(crate) analyzer_names: Vec<String>,
}

impl RunAnalyzerRequest {
    fn empty() -> Self {
        Self {
            analyzer_params: Default::default(),
            texts: Default::default(),
            with_detail: Default::default(),
            with_hash: Default::default(),
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            analyzer_names: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RunAnalyzerRequestBuilder {
        RunAnalyzerRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RunAnalyzerRequestBuilder {
        RunAnalyzerRequestBuilder { value: self }
    }

    /// Returns the analyzer params.
    pub fn analyzer_params(&self) -> &str {
        &self.analyzer_params
    }

    /// Returns the texts.
    pub fn texts(&self) -> &[String] {
        &self.texts
    }

    /// Returns whether the request should include detail.
    pub fn should_include_detail(&self) -> bool {
        self.with_detail
    }

    /// Returns whether the request should include hash.
    pub fn should_include_hash(&self) -> bool {
        self.with_hash
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the analyzer names.
    pub fn analyzer_names(&self) -> &[String] {
        &self.analyzer_names
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::RunAnalyzerRequest {
        milvus::RunAnalyzerRequest {
            base: None,
            analyzer_params: self.analyzer_params,
            placeholder: self.texts.into_iter().map(String::into_bytes).collect(),
            with_detail: self.with_detail,
            with_hash: self.with_hash,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            field_name: self.field_name,
            analyzer_names: self.analyzer_names,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RunAnalyzerRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RunAnalyzerRequest.
#[derive(Debug, Clone)]
pub struct RunAnalyzerRequestBuilder {
    value: RunAnalyzerRequest,
}

impl RunAnalyzerRequestBuilder {
    /// Sets the analyzer params and returns the updated value.
    pub fn analyzer_params(mut self, value: impl Into<String>) -> Self {
        self.value.analyzer_params = value.into();
        self
    }

    /// Sets the texts and returns the updated value.
    pub fn texts(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.texts = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns this value configured with with detail.
    pub fn with_detail(mut self, value: bool) -> Self {
        self.value.with_detail = value;
        self
    }

    /// Returns this value configured with with hash.
    pub fn with_hash(mut self, value: bool) -> Self {
        self.value.with_hash = value;
        self
    }

    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = value.into();
        self
    }

    /// Sets the analyzer names and returns the updated value.
    pub fn analyzer_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.analyzer_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RunAnalyzerRequest> {
        required_slice("texts", &self.value.texts)?;
        if self.value.analyzer_params.trim().is_empty() {
            required("collection_name", &self.value.collection_name)?;
            required("field_name", &self.value.field_name)?;
        }
        non_empty_strings("analyzer_names", &self.value.analyzer_names)?;
        Ok(self.value)
    }
}

fn validate_collection_target(_database_name: Option<&str>, collection_name: &str) -> Result<()> {
    required("collection_name", collection_name)
}

///////////////////////////////////////////////////////////////////////////////
// RefreshExternalCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 refresh_external_collection operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RefreshExternalCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) external_source: String,
    pub(crate) external_spec: Option<serde_json::Value>,
}

impl RefreshExternalCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            external_source: Default::default(),
            external_spec: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RefreshExternalCollectionRequestBuilder {
        RefreshExternalCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RefreshExternalCollectionRequestBuilder {
        RefreshExternalCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the external source.
    pub fn external_source(&self) -> &str {
        &self.external_source
    }

    /// Returns the external spec.
    pub fn external_spec(&self) -> Option<&serde_json::Value> {
        self.external_spec.as_ref()
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::RefreshExternalCollectionRequest {
        milvus::RefreshExternalCollectionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            external_source: self.external_source,
            external_spec: self
                .external_spec
                .as_ref()
                .map(serde_json::Value::to_string)
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RefreshExternalCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RefreshExternalCollectionRequest.
#[derive(Debug, Clone)]
pub struct RefreshExternalCollectionRequestBuilder {
    value: RefreshExternalCollectionRequest,
}

impl RefreshExternalCollectionRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the external source and returns the updated value.
    pub fn external_source(mut self, value: impl Into<String>) -> Self {
        self.value.external_source = value.into();
        self
    }

    /// Sets the external spec and returns the updated value.
    pub fn external_spec(mut self, value: serde_json::Value) -> Self {
        self.value.external_spec = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RefreshExternalCollectionRequest> {
        required("collection_name", &self.value.collection_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRefreshExternalCollectionProgressRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_refresh_external_collection_progress operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetRefreshExternalCollectionProgressRequest {
    pub(crate) job_id: i64,
}

impl GetRefreshExternalCollectionProgressRequest {
    fn empty() -> Self {
        Self { job_id: 0 }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetRefreshExternalCollectionProgressRequestBuilder {
        GetRefreshExternalCollectionProgressRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetRefreshExternalCollectionProgressRequestBuilder {
        GetRefreshExternalCollectionProgressRequestBuilder { value: self }
    }

    /// Returns the job id.
    pub fn job_id(&self) -> i64 {
        self.job_id
    }

    pub(crate) fn into_proto(self) -> milvus::GetRefreshExternalCollectionProgressRequest {
        milvus::GetRefreshExternalCollectionProgressRequest {
            base: None,
            job_id: self.job_id,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRefreshExternalCollectionProgressRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetRefreshExternalCollectionProgressRequest.
#[derive(Debug, Clone)]
pub struct GetRefreshExternalCollectionProgressRequestBuilder {
    value: GetRefreshExternalCollectionProgressRequest,
}

impl GetRefreshExternalCollectionProgressRequestBuilder {
    /// Sets the job id and returns the updated value.
    pub fn job_id(mut self, value: i64) -> Self {
        self.value.job_id = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetRefreshExternalCollectionProgressRequest> {
        positive_i64("job_id", self.value.job_id)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRefreshExternalCollectionJobsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_refresh_external_collection_jobs operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListRefreshExternalCollectionJobsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListRefreshExternalCollectionJobsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListRefreshExternalCollectionJobsRequestBuilder {
        ListRefreshExternalCollectionJobsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListRefreshExternalCollectionJobsRequestBuilder {
        ListRefreshExternalCollectionJobsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(
        self,
        default_db: &str,
    ) -> milvus::ListRefreshExternalCollectionJobsRequest {
        milvus::ListRefreshExternalCollectionJobsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRefreshExternalCollectionJobsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRefreshExternalCollectionJobsRequest.
#[derive(Debug, Clone)]
pub struct ListRefreshExternalCollectionJobsRequestBuilder {
    value: ListRefreshExternalCollectionJobsRequest,
}

impl ListRefreshExternalCollectionJobsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListRefreshExternalCollectionJobsRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddFileResourceRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 add_file_resource operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AddFileResourceRequest {
    pub(crate) name: String,
    pub(crate) path: String,
}

impl AddFileResourceRequest {
    fn empty() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AddFileResourceRequestBuilder {
        AddFileResourceRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AddFileResourceRequestBuilder {
        AddFileResourceRequestBuilder { value: self }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the path.
    pub fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn into_proto(self) -> milvus::AddFileResourceRequest {
        milvus::AddFileResourceRequest {
            base: None,
            name: self.name,
            path: self.path,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddFileResourceRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AddFileResourceRequest.
#[derive(Debug, Clone)]
pub struct AddFileResourceRequestBuilder {
    value: AddFileResourceRequest,
}

impl AddFileResourceRequestBuilder {
    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.value.name = value.into();
        self
    }

    /// Sets the path and returns the updated value.
    pub fn path(mut self, value: impl Into<String>) -> Self {
        self.value.path = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AddFileResourceRequest> {
        required("name", &self.value.name)?;
        required("path", &self.value.path)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RemoveFileResourceRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 remove_file_resource operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RemoveFileResourceRequest {
    pub(crate) name: String,
}

impl RemoveFileResourceRequest {
    fn empty() -> Self {
        Self {
            name: String::new(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RemoveFileResourceRequestBuilder {
        RemoveFileResourceRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RemoveFileResourceRequestBuilder {
        RemoveFileResourceRequestBuilder { value: self }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn into_proto(self) -> milvus::RemoveFileResourceRequest {
        milvus::RemoveFileResourceRequest {
            base: None,
            name: self.name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RemoveFileResourceRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RemoveFileResourceRequest.
#[derive(Debug, Clone)]
pub struct RemoveFileResourceRequestBuilder {
    value: RemoveFileResourceRequest,
}

impl RemoveFileResourceRequestBuilder {
    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.value.name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RemoveFileResourceRequest> {
        required("name", &self.value.name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListFileResourcesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_file_resources operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListFileResourcesRequest;

impl ListFileResourcesRequest {
    /// Creates a builder for this request.
    pub fn builder() -> ListFileResourcesRequestBuilder {
        ListFileResourcesRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListFileResourcesRequestBuilder {
        ListFileResourcesRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::ListFileResourcesRequest {
        milvus::ListFileResourcesRequest {
            base: None,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListFileResourcesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListFileResourcesRequest.
#[derive(Debug, Clone, Copy)]
pub struct ListFileResourcesRequestBuilder;

impl ListFileResourcesRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListFileResourcesRequest> {
        Ok(ListFileResourcesRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod flush_request_tests {
    use super::{FlushAllRequest, FlushRequest};

    #[test]
    fn wait_flushed_ms_validation_and_sdk_only_configuration() {
        assert!(FlushRequest::builder()
            .collection_names(["books"])
            .wait_flushed_ms(0)
            .build()
            .is_ok());
        assert!(FlushRequest::builder()
            .collection_names(["books"])
            .wait_flushed_ms(-1)
            .build()
            .is_err());
        assert!(FlushAllRequest::builder()
            .wait_flushed_ms(0)
            .build()
            .is_ok());
        assert!(FlushAllRequest::builder()
            .wait_flushed_ms(-1)
            .build()
            .is_err());

        let request = FlushRequest::builder()
            .collection_names(["books"])
            .wait_flushed_ms(1_500)
            .build()
            .expect("valid request");
        assert_eq!(request.wait_flushed_ms, 1_500);
        let proto = request.into_proto("default");
        assert_eq!(proto.collection_names, vec!["books"]);

        let request = FlushAllRequest::builder()
            .wait_flushed_ms(2_000)
            .build()
            .expect("valid request");
        assert_eq!(request.wait_flushed_ms, 2_000);
        let _proto = request.into_proto("default");
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn get_server_version_request_default_values() {
        let value = GetServerVersionRequest::empty();
        assert!(!value.is_detail_enabled());
        assert_eq!(
            value.into_get_version_proto(),
            milvus::GetVersionRequest::default()
        );
    }

    #[test]
    fn get_server_version_request_populated_values() {
        let value = GetServerVersionRequest::builder()
            .detail(true)
            .build()
            .expect("valid request");
        assert!(value.is_detail_enabled());
        assert_eq!(
            value.into_connect_proto(),
            milvus::ConnectRequest::default()
        );
    }

    #[test]
    fn check_health_request_default_values() {
        assert_eq!(
            CheckHealthRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::CheckHealthRequest::default()
        );
    }

    #[test]
    fn check_health_request_populated_values() {
        let value = CheckHealthRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto(), milvus::CheckHealthRequest::default());
    }

    #[test]
    fn flush_request_default_values() {
        let value = FlushRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_names: Vec<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(
            value.collection_names().to_owned(),
            expected_collection_names
        );
    }

    #[test]
    fn flush_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_names = vec!["collection_names-value".to_owned()];
        let value = FlushRequest::builder()
            .database_name(database_name.clone())
            .collection_names(collection_names.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_names().to_owned(), collection_names);
    }

    #[test]
    fn flush_all_request_default_values() {
        let value = FlushAllRequest::empty();
        let expected_database_name: Option<String> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
    }

    #[test]
    fn flush_all_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let value = FlushAllRequest::builder()
            .database_name(database_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
    }

    #[test]
    fn get_flush_all_state_request_default_values() {
        let value = GetFlushAllStateRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_flush_all_timestamp: u64 = 0;
        let expected_channel_timestamps: HashMap<String, u64> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.flush_all_timestamp(), expected_flush_all_timestamp);
        assert_eq!(
            value.channel_timestamps().to_owned(),
            expected_channel_timestamps
        );
    }

    #[test]
    fn get_flush_all_state_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let flush_all_timestamp = 7;
        let channel_timestamps = HashMap::from([("key-value".to_owned(), 7)]);
        let value = GetFlushAllStateRequest::builder()
            .database_name(database_name.clone())
            .flush_all_timestamp(flush_all_timestamp.clone())
            .channel_timestamps(channel_timestamps.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.flush_all_timestamp().to_owned(), flush_all_timestamp);
        assert_eq!(value.channel_timestamps().to_owned(), channel_timestamps);
    }

    #[test]
    fn list_persistent_segments_request_default_values() {
        let value = ListPersistentSegmentsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn list_persistent_segments_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = ListPersistentSegmentsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn list_query_segments_request_default_values() {
        let value = ListQuerySegmentsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn list_query_segments_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = ListQuerySegmentsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn compact_request_default_values() {
        let value = CompactRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_target_size: i64 = 0;
        let expected_clustering_compaction: bool = false;
        let expected_is_l0: bool = false;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.target_size().to_owned(), expected_target_size);
        assert_eq!(value.target_size_unit(), TargetSizeUnit::MB);
        assert_eq!(
            value.is_clustering_compaction(),
            expected_clustering_compaction
        );
        assert_eq!(value.is_l0(), expected_is_l0);
    }

    #[test]
    fn compact_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let target_size = 7;
        let clustering_compaction = true;
        let is_l0 = true;
        let value = CompactRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .target_size(target_size.clone())
            .target_size_unit(TargetSizeUnit::GB)
            .clustering_compaction(clustering_compaction.clone())
            .is_l0(is_l0.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.target_size().to_owned(), target_size);
        assert_eq!(value.target_size_unit(), TargetSizeUnit::GB);
        assert_eq!(
            value.is_clustering_compaction().to_owned(),
            clustering_compaction
        );
        assert_eq!(value.is_l0().to_owned(), is_l0);
    }

    #[test]
    fn compact_request_target_size_converts_units_to_megabytes() {
        let in_mb = CompactRequest::builder()
            .collection_name("books")
            .target_size(512)
            .target_size_unit(TargetSizeUnit::MB)
            .build()
            .expect("valid request");
        assert_eq!(in_mb.target_size_mb().expect("convert"), Some(512));

        let in_gb = CompactRequest::builder()
            .collection_name("books")
            .target_size(2)
            .target_size_unit(TargetSizeUnit::GB)
            .build()
            .expect("valid request");
        assert_eq!(in_gb.target_size_mb().expect("convert"), Some(2048));

        let in_kb = CompactRequest::builder()
            .collection_name("books")
            .target_size(1024)
            .target_size_unit(TargetSizeUnit::KB)
            .build()
            .expect("valid request");
        assert_eq!(in_kb.target_size_mb().expect("convert"), Some(1));

        let in_bytes = CompactRequest::builder()
            .collection_name("books")
            .target_size(1024 * 1024)
            .target_size_unit(TargetSizeUnit::B)
            .build()
            .expect("valid request");
        assert_eq!(in_bytes.target_size_mb().expect("convert"), Some(1));

        let unset = CompactRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        assert_eq!(unset.target_size_mb().expect("convert"), None);
    }

    #[test]
    fn compact_request_target_size_rejects_sub_megabyte_values() {
        let too_small = CompactRequest::builder()
            .collection_name("books")
            .target_size(1)
            .target_size_unit(TargetSizeUnit::B)
            .build()
            .expect_err("a sub-megabyte target size must be rejected");
        assert!(too_small.to_string().contains("zero MB"));
    }

    #[test]
    fn compact_request_encodes_l0_and_target_size_flags() {
        let proto = CompactRequest::builder()
            .collection_name("books")
            .target_size(1)
            .target_size_unit(TargetSizeUnit::GB)
            .is_l0(true)
            .build()
            .expect("valid request")
            .into_proto("default")
            .expect("valid conversion");
        assert_eq!(proto.l0_compaction, true);
        assert_eq!(proto.target_size, 1024);
        assert_eq!(proto.major_compaction, false);
    }

    #[test]
    fn optimize_request_default_values() {
        let value = OptimizeRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn optimize_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = OptimizeRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn get_compaction_state_request_default_values() {
        let value = GetCompactionStateRequest::empty();
        let expected_compaction_id: i64 = 0;

        assert_eq!(value.compaction_id().to_owned(), expected_compaction_id);
    }

    #[test]
    fn get_compaction_state_request_populated_values() {
        let compaction_id = 7;
        let value = GetCompactionStateRequest::builder()
            .compaction_id(compaction_id.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.compaction_id().to_owned(), compaction_id);
    }

    #[test]
    fn get_compaction_plans_request_default_values() {
        let value = GetCompactionPlansRequest::empty();
        let expected_compaction_id: i64 = 0;

        assert_eq!(value.compaction_id().to_owned(), expected_compaction_id);
    }

    #[test]
    fn get_compaction_plans_request_populated_values() {
        let compaction_id = 7;
        let value = GetCompactionPlansRequest::builder()
            .compaction_id(compaction_id.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.compaction_id().to_owned(), compaction_id);
    }

    #[test]
    fn run_analyzer_request_default_values() {
        let value = RunAnalyzerRequest::empty();
        let expected_analyzer_params: String = String::new();
        let expected_texts: Vec<String> = Default::default();
        let expected_with_detail: bool = false;
        let expected_with_hash: bool = false;
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field_name: String = String::new();
        let expected_analyzer_names: Vec<String> = Default::default();

        assert_eq!(value.analyzer_params().to_owned(), expected_analyzer_params);
        assert_eq!(value.texts().to_owned(), expected_texts);
        assert_eq!(
            value.should_include_detail().to_owned(),
            expected_with_detail
        );
        assert_eq!(value.should_include_hash().to_owned(), expected_with_hash);
        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field_name().to_owned(), expected_field_name);
        assert_eq!(value.analyzer_names().to_owned(), expected_analyzer_names);
    }

    #[test]
    fn run_analyzer_request_populated_values() {
        let analyzer_params = "analyzer_params-value".to_owned();
        let texts = vec!["texts-value".to_owned()];
        let with_detail = true;
        let with_hash = true;
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field_name = "field_name-value".to_owned();
        let analyzer_names = vec!["analyzer_names-value".to_owned()];
        let value = RunAnalyzerRequest::builder()
            .analyzer_params(analyzer_params.clone())
            .texts(texts.clone())
            .with_detail(with_detail.clone())
            .with_hash(with_hash.clone())
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field_name(field_name.clone())
            .analyzer_names(analyzer_names.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.analyzer_params().to_owned(), analyzer_params);
        assert_eq!(value.texts().to_owned(), texts);
        assert_eq!(value.should_include_detail().to_owned(), with_detail);
        assert_eq!(value.should_include_hash().to_owned(), with_hash);
        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field_name().to_owned(), field_name);
        assert_eq!(value.analyzer_names().to_owned(), analyzer_names);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod external_collection_request_tests {
    use super::*;
    use crate::v2::error::Error;

    #[test]
    fn refresh_external_collection_request_default_values() {
        let value = RefreshExternalCollectionRequest::empty();
        assert_eq!(value.database_name(), &None);
        assert_eq!(value.collection_name(), "");
        assert_eq!(value.external_source(), "");
        assert!(value.external_spec().is_none());
    }

    #[test]
    fn refresh_external_collection_request_populated_values() {
        let value = RefreshExternalCollectionRequest::builder()
            .database_name("default")
            .collection_name("books")
            .external_source("s3://bucket/path")
            .external_spec(serde_json::json!({"format": "parquet"}))
            .build()
            .expect("valid request");
        assert_eq!(value.database_name().as_deref(), Some("default"));
        assert_eq!(value.collection_name(), "books");
        assert_eq!(value.external_source(), "s3://bucket/path");
        assert_eq!(
            value.external_spec(),
            Some(&serde_json::json!({"format": "parquet"}))
        );

        let proto = value.into_proto("default");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.external_source, "s3://bucket/path");
        assert_eq!(proto.external_spec, r#"{"format":"parquet"}"#);
    }

    #[test]
    fn refresh_external_collection_request_uses_selected_database_and_rejects_empty_collection() {
        let value = RefreshExternalCollectionRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("analytics").db_name, "analytics");
        assert!(matches!(
            RefreshExternalCollectionRequest::builder()
                .build()
                .unwrap_err(),
            Error::Validation(_)
        ));
    }

    #[test]
    fn get_refresh_external_collection_progress_request_populated_values() {
        let value = GetRefreshExternalCollectionProgressRequest::builder()
            .job_id(7)
            .build()
            .expect("valid request");
        assert_eq!(value.job_id(), 7);
        assert_eq!(value.into_proto().job_id, 7);
        assert!(GetRefreshExternalCollectionProgressRequest::builder()
            .job_id(0)
            .build()
            .is_err());
    }

    #[test]
    fn list_refresh_external_collection_jobs_request_is_valid_without_a_collection() {
        let value = ListRefreshExternalCollectionJobsRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("default").collection_name, "");
        let value = ListRefreshExternalCollectionJobsRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("default").collection_name, "books");
    }

    #[test]
    fn add_file_resource_request_populated_values() {
        let value = AddFileResourceRequest::builder()
            .name("data-files")
            .path("s3://bucket/path")
            .build()
            .expect("valid request");
        let proto = value.into_proto();
        assert_eq!(proto.name, "data-files");
        assert_eq!(proto.path, "s3://bucket/path");
        assert!(AddFileResourceRequest::builder().build().is_err());
        assert!(AddFileResourceRequest::builder()
            .name("data-files")
            .build()
            .is_err());
    }

    #[test]
    fn remove_file_resource_request_populated_values() {
        let value = RemoveFileResourceRequest::builder()
            .name("data-files")
            .build()
            .expect("valid request");
        assert_eq!(value.name(), "data-files");
        assert_eq!(value.into_proto().name, "data-files");
        assert!(RemoveFileResourceRequest::builder().build().is_err());
    }

    #[test]
    fn list_file_resources_request_is_valid() {
        assert_eq!(
            ListFileResourcesRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::ListFileResourcesRequest {
                base: None,
                ..Default::default()
            }
        );
    }
}
