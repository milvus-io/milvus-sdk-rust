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

//! Request types for snapshot backup and restore operations.

use crate::proto::milvus;
use crate::v2::error::Result;
use crate::v2::request::validation::{non_negative_i64, positive_i64, required};

///////////////////////////////////////////////////////////////////////////////
// CreateSnapshotRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_snapshot operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateSnapshotRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) snapshot_name: String,
    pub(crate) description: String,
    pub(crate) compaction_protection_seconds: i64,
}

impl CreateSnapshotRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            snapshot_name: Default::default(),
            description: Default::default(),
            compaction_protection_seconds: 0,
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CreateSnapshotRequestBuilder {
        CreateSnapshotRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateSnapshotRequestBuilder {
        CreateSnapshotRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the snapshot name.
    pub fn snapshot_name(&self) -> &str {
        &self.snapshot_name
    }

    /// Returns the snapshot description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the compaction protection seconds.
    pub fn compaction_protection_seconds(&self) -> i64 {
        self.compaction_protection_seconds
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::CreateSnapshotRequest {
        milvus::CreateSnapshotRequest {
            base: None,
            name: self.snapshot_name,
            description: self.description,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            compaction_protection_seconds: self.compaction_protection_seconds,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateSnapshotRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateSnapshotRequest.
#[derive(Debug, Clone)]
pub struct CreateSnapshotRequestBuilder {
    value: CreateSnapshotRequest,
}

impl CreateSnapshotRequestBuilder {
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

    /// Sets the snapshot name and returns the updated value.
    pub fn snapshot_name(mut self, value: impl Into<String>) -> Self {
        self.value.snapshot_name = value.into();
        self
    }

    /// Sets the snapshot description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Sets the compaction protection seconds and returns the updated value.
    pub fn compaction_protection_seconds(mut self, value: i64) -> Self {
        self.value.compaction_protection_seconds = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateSnapshotRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("snapshot_name", &self.value.snapshot_name)?;
        non_negative_i64(
            "compaction_protection_seconds",
            self.value.compaction_protection_seconds,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropSnapshotRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_snapshot operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropSnapshotRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) snapshot_name: String,
}

impl DropSnapshotRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            snapshot_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropSnapshotRequestBuilder {
        DropSnapshotRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropSnapshotRequestBuilder {
        DropSnapshotRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the snapshot name.
    pub fn snapshot_name(&self) -> &str {
        &self.snapshot_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DropSnapshotRequest {
        milvus::DropSnapshotRequest {
            base: None,
            name: self.snapshot_name,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropSnapshotRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropSnapshotRequest.
#[derive(Debug, Clone)]
pub struct DropSnapshotRequestBuilder {
    value: DropSnapshotRequest,
}

impl DropSnapshotRequestBuilder {
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

    /// Sets the snapshot name and returns the updated value.
    pub fn snapshot_name(mut self, value: impl Into<String>) -> Self {
        self.value.snapshot_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropSnapshotRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("snapshot_name", &self.value.snapshot_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListSnapshotsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_snapshots operation.
///
/// When `collection_name` is empty, all snapshots in the database are returned.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSnapshotsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListSnapshotsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListSnapshotsRequestBuilder {
        ListSnapshotsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListSnapshotsRequestBuilder {
        ListSnapshotsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ListSnapshotsRequest {
        milvus::ListSnapshotsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListSnapshotsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListSnapshotsRequest.
#[derive(Debug, Clone)]
pub struct ListSnapshotsRequestBuilder {
    value: ListSnapshotsRequest,
}

impl ListSnapshotsRequestBuilder {
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
    pub fn build(self) -> Result<ListSnapshotsRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeSnapshotRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_snapshot operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeSnapshotRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) snapshot_name: String,
}

impl DescribeSnapshotRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            snapshot_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DescribeSnapshotRequestBuilder {
        DescribeSnapshotRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeSnapshotRequestBuilder {
        DescribeSnapshotRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the snapshot name.
    pub fn snapshot_name(&self) -> &str {
        &self.snapshot_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DescribeSnapshotRequest {
        milvus::DescribeSnapshotRequest {
            base: None,
            name: self.snapshot_name,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeSnapshotRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeSnapshotRequest.
#[derive(Debug, Clone)]
pub struct DescribeSnapshotRequestBuilder {
    value: DescribeSnapshotRequest,
}

impl DescribeSnapshotRequestBuilder {
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

    /// Sets the snapshot name and returns the updated value.
    pub fn snapshot_name(mut self, value: impl Into<String>) -> Self {
        self.value.snapshot_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeSnapshotRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("snapshot_name", &self.value.snapshot_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RestoreSnapshotRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 restore_snapshot operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RestoreSnapshotRequest {
    pub(crate) snapshot_name: String,
    pub(crate) source_database_name: Option<String>,
    pub(crate) source_collection_name: String,
    pub(crate) target_database_name: Option<String>,
    pub(crate) target_collection_name: String,
}

impl RestoreSnapshotRequest {
    fn empty() -> Self {
        Self {
            snapshot_name: Default::default(),
            source_database_name: Default::default(),
            source_collection_name: Default::default(),
            target_database_name: Default::default(),
            target_collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RestoreSnapshotRequestBuilder {
        RestoreSnapshotRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RestoreSnapshotRequestBuilder {
        RestoreSnapshotRequestBuilder { value: self }
    }

    /// Returns the snapshot name.
    pub fn snapshot_name(&self) -> &str {
        &self.snapshot_name
    }

    /// Returns the source database name.
    pub fn source_database_name(&self) -> &Option<String> {
        &self.source_database_name
    }

    /// Returns the source collection name.
    pub fn source_collection_name(&self) -> &str {
        &self.source_collection_name
    }

    /// Returns the target database name.
    pub fn target_database_name(&self) -> &Option<String> {
        &self.target_database_name
    }

    /// Returns the target collection name.
    pub fn target_collection_name(&self) -> &str {
        &self.target_collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::RestoreSnapshotRequest {
        milvus::RestoreSnapshotRequest {
            base: None,
            name: self.snapshot_name,
            db_name: self
                .source_database_name
                .unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.source_collection_name,
            rewrite_data: false,
            target_db_name: self
                .target_database_name
                .unwrap_or_else(|| default_db.to_owned()),
            target_collection_name: self.target_collection_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RestoreSnapshotRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RestoreSnapshotRequest.
#[derive(Debug, Clone)]
pub struct RestoreSnapshotRequestBuilder {
    value: RestoreSnapshotRequest,
}

impl RestoreSnapshotRequestBuilder {
    /// Sets the snapshot name and returns the updated value.
    pub fn snapshot_name(mut self, value: impl Into<String>) -> Self {
        self.value.snapshot_name = value.into();
        self
    }

    /// Sets the source database name and returns the updated value.
    pub fn source_database_name(mut self, value: impl Into<String>) -> Self {
        self.value.source_database_name = Some(value.into());
        self
    }

    /// Sets the source collection name and returns the updated value.
    pub fn source_collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.source_collection_name = value.into();
        self
    }

    /// Sets the target database name and returns the updated value.
    pub fn target_database_name(mut self, value: impl Into<String>) -> Self {
        self.value.target_database_name = Some(value.into());
        self
    }

    /// Sets the target collection name and returns the updated value.
    pub fn target_collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.target_collection_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RestoreSnapshotRequest> {
        required("snapshot_name", &self.value.snapshot_name)?;
        required("source_collection_name", &self.value.source_collection_name)?;
        required("target_collection_name", &self.value.target_collection_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRestoreSnapshotStateRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_restore_snapshot_state operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetRestoreSnapshotStateRequest {
    pub(crate) job_id: i64,
}

impl GetRestoreSnapshotStateRequest {
    fn empty() -> Self {
        Self { job_id: 0 }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetRestoreSnapshotStateRequestBuilder {
        GetRestoreSnapshotStateRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetRestoreSnapshotStateRequestBuilder {
        GetRestoreSnapshotStateRequestBuilder { value: self }
    }

    /// Returns the restore job id.
    pub fn job_id(&self) -> i64 {
        self.job_id
    }

    pub(crate) fn into_proto(self) -> milvus::GetRestoreSnapshotStateRequest {
        milvus::GetRestoreSnapshotStateRequest {
            base: None,
            job_id: self.job_id,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRestoreSnapshotStateRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetRestoreSnapshotStateRequest.
#[derive(Debug, Clone)]
pub struct GetRestoreSnapshotStateRequestBuilder {
    value: GetRestoreSnapshotStateRequest,
}

impl GetRestoreSnapshotStateRequestBuilder {
    /// Sets the restore job id and returns the updated value.
    pub fn job_id(mut self, value: i64) -> Self {
        self.value.job_id = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetRestoreSnapshotStateRequest> {
        positive_i64("job_id", self.value.job_id)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRestoreSnapshotJobsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_restore_snapshot_jobs operation.
///
/// When `collection_name` is empty, restore jobs of all collections in the
/// database are returned.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListRestoreSnapshotJobsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListRestoreSnapshotJobsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListRestoreSnapshotJobsRequestBuilder {
        ListRestoreSnapshotJobsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListRestoreSnapshotJobsRequestBuilder {
        ListRestoreSnapshotJobsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ListRestoreSnapshotJobsRequest {
        milvus::ListRestoreSnapshotJobsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRestoreSnapshotJobsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRestoreSnapshotJobsRequest.
#[derive(Debug, Clone)]
pub struct ListRestoreSnapshotJobsRequestBuilder {
    value: ListRestoreSnapshotJobsRequest,
}

impl ListRestoreSnapshotJobsRequestBuilder {
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
    pub fn build(self) -> Result<ListRestoreSnapshotJobsRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// PinSnapshotDataRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 pin_snapshot_data operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PinSnapshotDataRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) snapshot_name: String,
    pub(crate) ttl_seconds: i64,
}

impl PinSnapshotDataRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            snapshot_name: Default::default(),
            ttl_seconds: 0,
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> PinSnapshotDataRequestBuilder {
        PinSnapshotDataRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> PinSnapshotDataRequestBuilder {
        PinSnapshotDataRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the snapshot name.
    pub fn snapshot_name(&self) -> &str {
        &self.snapshot_name
    }

    /// Returns the TTL seconds.
    pub fn ttl_seconds(&self) -> i64 {
        self.ttl_seconds
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::PinSnapshotDataRequest {
        milvus::PinSnapshotDataRequest {
            base: None,
            name: self.snapshot_name,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            ttl_seconds: self.ttl_seconds,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// PinSnapshotDataRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for PinSnapshotDataRequest.
#[derive(Debug, Clone)]
pub struct PinSnapshotDataRequestBuilder {
    value: PinSnapshotDataRequest,
}

impl PinSnapshotDataRequestBuilder {
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

    /// Sets the snapshot name and returns the updated value.
    pub fn snapshot_name(mut self, value: impl Into<String>) -> Self {
        self.value.snapshot_name = value.into();
        self
    }

    /// Sets the TTL seconds and returns the updated value.
    pub fn ttl_seconds(mut self, value: i64) -> Self {
        self.value.ttl_seconds = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<PinSnapshotDataRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("snapshot_name", &self.value.snapshot_name)?;
        non_negative_i64("ttl_seconds", self.value.ttl_seconds)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// UnpinSnapshotDataRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 unpin_snapshot_data operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnpinSnapshotDataRequest {
    pub(crate) pin_id: i64,
}

impl UnpinSnapshotDataRequest {
    fn empty() -> Self {
        Self { pin_id: 0 }
    }

    /// Creates a builder for this request.
    pub fn builder() -> UnpinSnapshotDataRequestBuilder {
        UnpinSnapshotDataRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UnpinSnapshotDataRequestBuilder {
        UnpinSnapshotDataRequestBuilder { value: self }
    }

    /// Returns the pin id.
    pub fn pin_id(&self) -> i64 {
        self.pin_id
    }

    pub(crate) fn into_proto(self) -> milvus::UnpinSnapshotDataRequest {
        milvus::UnpinSnapshotDataRequest {
            base: None,
            pin_id: self.pin_id,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UnpinSnapshotDataRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UnpinSnapshotDataRequest.
#[derive(Debug, Clone)]
pub struct UnpinSnapshotDataRequestBuilder {
    value: UnpinSnapshotDataRequest,
}

impl UnpinSnapshotDataRequestBuilder {
    /// Sets the pin id and returns the updated value.
    pub fn pin_id(mut self, value: i64) -> Self {
        self.value.pin_id = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<UnpinSnapshotDataRequest> {
        positive_i64("pin_id", self.value.pin_id)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::error::Error;

    #[test]
    fn create_snapshot_request_default_values() {
        let value = CreateSnapshotRequest::empty();
        assert_eq!(value.database_name(), &None);
        assert_eq!(value.collection_name(), "");
        assert_eq!(value.snapshot_name(), "");
        assert_eq!(value.description(), "");
        assert_eq!(value.compaction_protection_seconds(), 0);
    }

    #[test]
    fn create_snapshot_request_populated_values() {
        let value = CreateSnapshotRequest::builder()
            .database_name("default")
            .collection_name("books")
            .snapshot_name("snap-1")
            .description("backup")
            .compaction_protection_seconds(300)
            .build()
            .expect("valid request");
        assert_eq!(value.database_name().as_deref(), Some("default"));
        assert_eq!(value.collection_name(), "books");
        assert_eq!(value.snapshot_name(), "snap-1");
        assert_eq!(value.description(), "backup");
        assert_eq!(value.compaction_protection_seconds(), 300);

        let proto = value.into_proto("default");
        assert_eq!(proto.name, "snap-1");
        assert_eq!(proto.description, "backup");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.compaction_protection_seconds, 300);
    }

    #[test]
    fn create_snapshot_request_uses_selected_database_when_omitted() {
        let value = CreateSnapshotRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("selected").db_name, "selected");
    }

    #[test]
    fn create_snapshot_request_rejects_invalid_input() {
        assert!(matches!(
            CreateSnapshotRequest::builder()
                .snapshot_name("snap-1")
                .build()
                .unwrap_err(),
            Error::Validation(_)
        ));
        assert!(matches!(
            CreateSnapshotRequest::builder()
                .collection_name("books")
                .build()
                .unwrap_err(),
            Error::Validation(_)
        ));
        assert!(CreateSnapshotRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .compaction_protection_seconds(-1)
            .build()
            .is_err());
    }

    #[test]
    fn drop_snapshot_request_populated_values() {
        let value = DropSnapshotRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.name, "snap-1");
        assert_eq!(proto.collection_name, "books");
        assert!(DropSnapshotRequest::builder().build().is_err());
    }

    #[test]
    fn list_snapshots_request_is_valid_without_a_collection() {
        let value = ListSnapshotsRequest::builder()
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.collection_name, "");

        let value = ListSnapshotsRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("default").collection_name, "books");
    }

    #[test]
    fn describe_snapshot_request_populated_values() {
        let value = DescribeSnapshotRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("default").name, "snap-1");
        assert!(DescribeSnapshotRequest::builder()
            .collection_name("books")
            .build()
            .is_err());
    }

    #[test]
    fn restore_snapshot_request_populated_values() {
        let value = RestoreSnapshotRequest::builder()
            .snapshot_name("snap-1")
            .source_collection_name("books")
            .target_collection_name("books_restored")
            .target_database_name("backup")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.name, "snap-1");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.target_collection_name, "books_restored");
        assert_eq!(proto.target_db_name, "backup");
        assert_eq!(proto.db_name, "default");
        assert!(!proto.rewrite_data);
        assert!(RestoreSnapshotRequest::builder()
            .source_collection_name("books")
            .target_collection_name("books_restored")
            .build()
            .is_err());
    }

    #[test]
    fn restore_snapshot_request_uses_selected_database_for_source_and_target() {
        let value = RestoreSnapshotRequest::builder()
            .snapshot_name("snap-1")
            .source_collection_name("books")
            .target_collection_name("books_restored")
            .build()
            .expect("valid request");
        let proto = value.into_proto("analytics");
        assert_eq!(proto.db_name, "analytics");
        assert_eq!(proto.target_db_name, "analytics");

        let value = RestoreSnapshotRequest::builder()
            .snapshot_name("snap-1")
            .source_database_name("source")
            .source_collection_name("books")
            .target_collection_name("books_restored")
            .build()
            .expect("valid request");
        let proto = value.into_proto("analytics");
        assert_eq!(proto.db_name, "source");
        assert_eq!(proto.target_db_name, "analytics");
    }

    #[test]
    fn get_restore_snapshot_state_request_populated_values() {
        let value = GetRestoreSnapshotStateRequest::builder()
            .job_id(7)
            .build()
            .expect("valid request");
        assert_eq!(value.job_id(), 7);
        assert_eq!(value.into_proto().job_id, 7);
        assert!(GetRestoreSnapshotStateRequest::builder()
            .job_id(0)
            .build()
            .is_err());
    }

    #[test]
    fn list_restore_snapshot_jobs_request_is_valid_without_a_collection() {
        let value = ListRestoreSnapshotJobsRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto("default").collection_name, "");
    }

    #[test]
    fn pin_snapshot_data_request_populated_values() {
        let value = PinSnapshotDataRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .ttl_seconds(3600)
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.name, "snap-1");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.ttl_seconds, 3600);
        assert!(PinSnapshotDataRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .ttl_seconds(-1)
            .build()
            .is_err());
    }

    #[test]
    fn unpin_snapshot_data_request_populated_values() {
        let value = UnpinSnapshotDataRequest::builder()
            .pin_id(7)
            .build()
            .expect("valid request");
        assert_eq!(value.pin_id(), 7);
        assert_eq!(value.into_proto().pin_id, 7);
        assert!(UnpinSnapshotDataRequest::builder()
            .pin_id(0)
            .build()
            .is_err());
    }

    #[test]
    fn into_builder_preserves_request_values() {
        let request = CreateSnapshotRequest::builder()
            .collection_name("books")
            .snapshot_name("snap-1")
            .build()
            .expect("valid request");
        let rebuilt = request
            .into_builder()
            .description("updated")
            .build()
            .expect("valid request");
        assert_eq!(rebuilt.collection_name(), "books");
        assert_eq!(rebuilt.snapshot_name(), "snap-1");
        assert_eq!(rebuilt.description(), "updated");
    }
}
