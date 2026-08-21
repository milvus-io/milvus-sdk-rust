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

//! Response types returned by snapshot backup and restore operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
pub use crate::v2::types::{RestoreSnapshotJobInfo, RestoreSnapshotStateCode};

///////////////////////////////////////////////////////////////////////////////
// ListSnapshotsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_snapshots operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSnapshotsResponse {
    pub(crate) snapshots: Vec<String>,
}

impl ListSnapshotsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> ListSnapshotsResponseBuilder {
        ListSnapshotsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the snapshot names.
    pub fn snapshots(&self) -> &[String] {
        &self.snapshots
    }

    pub(crate) fn from_proto(v: milvus::ListSnapshotsResponse) -> Self {
        Self {
            snapshots: v.snapshots,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListSnapshotsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListSnapshotsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListSnapshotsResponseBuilder {
    value: ListSnapshotsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListSnapshotsResponseBuilder {
    /// Sets the snapshot names and returns the updated value.
    pub fn snapshots(mut self, value: Vec<String>) -> Self {
        self.value.snapshots = value;
        self
    }

    /// Validates the configured values and builds the response.
    pub fn build(self) -> ListSnapshotsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeSnapshotResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_snapshot operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeSnapshotResponse {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
    pub(crate) create_ts: i64,
    pub(crate) s3_location: String,
}

impl DescribeSnapshotResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            collection_name: String::new(),
            partition_names: Vec::new(),
            create_ts: 0,
            s3_location: String::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeSnapshotResponseBuilder {
        DescribeSnapshotResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the snapshot name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the snapshot description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition names in the snapshot.
    pub fn partition_names(&self) -> &[String] {
        &self.partition_names
    }

    /// Returns the create timestamp.
    pub fn create_ts(&self) -> i64 {
        self.create_ts
    }

    /// Returns the S3 location of the snapshot meta file, if exported.
    pub fn s3_location(&self) -> &str {
        &self.s3_location
    }

    pub(crate) fn from_proto(v: milvus::DescribeSnapshotResponse) -> Self {
        Self {
            name: v.name,
            description: v.description,
            collection_name: v.collection_name,
            partition_names: v.partition_names,
            create_ts: v.create_ts,
            s3_location: v.s3_location,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeSnapshotResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeSnapshotResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeSnapshotResponseBuilder {
    value: DescribeSnapshotResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeSnapshotResponseBuilder {
    /// Sets the snapshot name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.value.name = value.into();
        self
    }

    /// Sets the snapshot description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the partition names and returns the updated value.
    pub fn partition_names(mut self, value: Vec<String>) -> Self {
        self.value.partition_names = value;
        self
    }

    /// Sets the create timestamp and returns the updated value.
    pub fn create_ts(mut self, value: i64) -> Self {
        self.value.create_ts = value;
        self
    }

    /// Sets the S3 location and returns the updated value.
    pub fn s3_location(mut self, value: impl Into<String>) -> Self {
        self.value.s3_location = value.into();
        self
    }

    /// Validates the configured values and builds the response.
    pub fn build(self) -> DescribeSnapshotResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// RestoreSnapshotResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 restore_snapshot operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct RestoreSnapshotResponse {
    pub(crate) job_id: i64,
}

impl RestoreSnapshotResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self { job_id: 0 }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> RestoreSnapshotResponseBuilder {
        RestoreSnapshotResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the restore job id.
    pub fn job_id(&self) -> i64 {
        self.job_id
    }

    pub(crate) fn from_proto(v: milvus::RestoreSnapshotResponse) -> Self {
        Self { job_id: v.job_id }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RestoreSnapshotResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RestoreSnapshotResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct RestoreSnapshotResponseBuilder {
    value: RestoreSnapshotResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl RestoreSnapshotResponseBuilder {
    /// Sets the restore job id and returns the updated value.
    pub fn job_id(mut self, value: i64) -> Self {
        self.value.job_id = value;
        self
    }

    /// Validates the configured values and builds the response.
    pub fn build(self) -> RestoreSnapshotResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRestoreSnapshotStateResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_restore_snapshot_state operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetRestoreSnapshotStateResponse {
    pub(crate) job_info: RestoreSnapshotJobInfo,
}

impl GetRestoreSnapshotStateResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            job_info: RestoreSnapshotJobInfo::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> GetRestoreSnapshotStateResponseBuilder {
        GetRestoreSnapshotStateResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the restore job info.
    pub fn job_info(&self) -> &RestoreSnapshotJobInfo {
        &self.job_info
    }

    pub(crate) fn from_proto(v: milvus::GetRestoreSnapshotStateResponse) -> Result<Self> {
        let job_info = v.info.ok_or_else(|| {
            Error::MalformedResponse("get restore snapshot state response has no job info".into())
        })?;
        Ok(Self {
            job_info: RestoreSnapshotJobInfo::from_proto(job_info),
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRestoreSnapshotStateResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetRestoreSnapshotStateResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetRestoreSnapshotStateResponseBuilder {
    value: GetRestoreSnapshotStateResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetRestoreSnapshotStateResponseBuilder {
    /// Sets the restore job info and returns the updated value.
    pub fn job_info(mut self, value: RestoreSnapshotJobInfo) -> Self {
        self.value.job_info = value;
        self
    }

    /// Validates the configured values and builds the response.
    pub fn build(self) -> GetRestoreSnapshotStateResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRestoreSnapshotJobsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_restore_snapshot_jobs operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListRestoreSnapshotJobsResponse {
    pub(crate) jobs: Vec<RestoreSnapshotJobInfo>,
}

impl ListRestoreSnapshotJobsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self { jobs: Vec::new() }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> ListRestoreSnapshotJobsResponseBuilder {
        ListRestoreSnapshotJobsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the restore jobs.
    pub fn jobs(&self) -> &[RestoreSnapshotJobInfo] {
        &self.jobs
    }

    pub(crate) fn from_proto(v: milvus::ListRestoreSnapshotJobsResponse) -> Self {
        Self {
            jobs: v
                .jobs
                .into_iter()
                .map(RestoreSnapshotJobInfo::from_proto)
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRestoreSnapshotJobsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRestoreSnapshotJobsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListRestoreSnapshotJobsResponseBuilder {
    value: ListRestoreSnapshotJobsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListRestoreSnapshotJobsResponseBuilder {
    /// Sets the restore jobs and returns the updated value.
    pub fn jobs(mut self, value: Vec<RestoreSnapshotJobInfo>) -> Self {
        self.value.jobs = value;
        self
    }

    /// Validates the configured values and builds the response.
    pub fn build(self) -> ListRestoreSnapshotJobsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// PinSnapshotDataResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 pin_snapshot_data operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct PinSnapshotDataResponse {
    pub(crate) pin_id: i64,
}

impl PinSnapshotDataResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self { pin_id: 0 }
    }

    #[cfg(test)]
    pub(crate) fn builder() -> PinSnapshotDataResponseBuilder {
        PinSnapshotDataResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the pin id used to unpin the snapshot data.
    pub fn pin_id(&self) -> i64 {
        self.pin_id
    }

    pub(crate) fn from_proto(v: milvus::PinSnapshotDataResponse) -> Self {
        Self { pin_id: v.pin_id }
    }
}

///////////////////////////////////////////////////////////////////////////////
// PinSnapshotDataResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for PinSnapshotDataResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct PinSnapshotDataResponseBuilder {
    value: PinSnapshotDataResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl PinSnapshotDataResponseBuilder {
    /// Sets the pin id and returns the updated value.
    pub fn pin_id(mut self, value: i64) -> Self {
        self.value.pin_id = value;
        self
    }

    /// Validates the configured values and builds the response.
    pub fn build(self) -> PinSnapshotDataResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod decoding_tests {
    use super::{DescribeSnapshotResponse, GetRestoreSnapshotStateResponse};
    use crate::proto::milvus;
    use crate::v2::error::Error;

    #[test]
    fn describe_snapshot_response_decodes_proto_fields() {
        let response = DescribeSnapshotResponse::from_proto(milvus::DescribeSnapshotResponse {
            status: None,
            name: "snap-1".into(),
            description: "backup".into(),
            collection_name: "books".into(),
            partition_names: vec!["p1".into(), "p2".into()],
            create_ts: 123,
            s3_location: "s3://bucket/export".into(),
            ..Default::default()
        });
        assert_eq!(response.name(), "snap-1");
        assert_eq!(response.description(), "backup");
        assert_eq!(response.collection_name(), "books");
        assert_eq!(
            response.partition_names(),
            &["p1".to_owned(), "p2".to_owned()]
        );
        assert_eq!(response.create_ts(), 123);
        assert_eq!(response.s3_location(), "s3://bucket/export");
    }

    #[test]
    fn get_restore_snapshot_state_rejects_a_missing_job_info() {
        let error =
            GetRestoreSnapshotStateResponse::from_proto(milvus::GetRestoreSnapshotStateResponse {
                status: None,
                info: None,
                ..Default::default()
            })
            .unwrap_err();
        assert!(matches!(error, Error::MalformedResponse(_)));
    }

    #[test]
    fn get_restore_snapshot_state_decodes_job_info() {
        let response =
            GetRestoreSnapshotStateResponse::from_proto(milvus::GetRestoreSnapshotStateResponse {
                status: None,
                info: Some(milvus::RestoreSnapshotInfo {
                    job_id: 7,
                    snapshot_name: "snap-1".into(),
                    db_name: "default".into(),
                    collection_name: "books".into(),
                    state: milvus::RestoreSnapshotState::RestoreSnapshotCompleted as i32,
                    progress: 100,
                    reason: String::new(),
                    start_time: 10,
                    time_cost: 20,
                }),
                ..Default::default()
            })
            .expect("valid job info");
        assert_eq!(response.job_info().get_job_id(), 7);
        assert_eq!(
            response.job_info().get_state(),
            super::RestoreSnapshotStateCode::Completed
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn list_snapshots_response_default_values() {
        let value = ListSnapshotsResponse::builder().build();
        assert!(value.snapshots().is_empty());
    }

    #[test]
    fn list_snapshots_response_populated_values() {
        let value = ListSnapshotsResponse::builder()
            .snapshots(vec!["snap-1".into(), "snap-2".into()])
            .build();
        assert_eq!(
            value.snapshots(),
            &["snap-1".to_owned(), "snap-2".to_owned()]
        );
    }

    #[test]
    fn describe_snapshot_response_default_values() {
        let value = DescribeSnapshotResponse::builder().build();
        assert_eq!(value.name(), "");
        assert_eq!(value.description(), "");
        assert_eq!(value.collection_name(), "");
        assert!(value.partition_names().is_empty());
        assert_eq!(value.create_ts(), 0);
        assert_eq!(value.s3_location(), "");
    }

    #[test]
    fn describe_snapshot_response_populated_values() {
        let value = DescribeSnapshotResponse::builder()
            .name("snap-1")
            .description("backup")
            .collection_name("books")
            .partition_names(vec!["p1".into(), "p2".into()])
            .create_ts(123)
            .s3_location("s3://bucket/export")
            .build();
        assert_eq!(value.name(), "snap-1");
        assert_eq!(value.description(), "backup");
        assert_eq!(value.collection_name(), "books");
        assert_eq!(value.partition_names(), &["p1".to_owned(), "p2".to_owned()]);
        assert_eq!(value.create_ts(), 123);
        assert_eq!(value.s3_location(), "s3://bucket/export");
    }

    #[test]
    fn restore_snapshot_response_populated_values() {
        let value = RestoreSnapshotResponse::builder().job_id(7).build();
        assert_eq!(value.job_id(), 7);
    }

    #[test]
    fn get_restore_snapshot_state_response_populated_values() {
        let job_info = RestoreSnapshotJobInfo::new().job_id(7);
        let value = GetRestoreSnapshotStateResponse::builder()
            .job_info(job_info.clone())
            .build();
        assert_eq!(value.job_info(), &job_info);
    }

    #[test]
    fn list_restore_snapshot_jobs_response_populated_values() {
        let job_info = RestoreSnapshotJobInfo::new().job_id(7);
        let value = ListRestoreSnapshotJobsResponse::builder()
            .jobs(vec![job_info.clone()])
            .build();
        assert_eq!(value.jobs(), &[job_info]);
    }

    #[test]
    fn pin_snapshot_data_response_populated_values() {
        let value = PinSnapshotDataResponse::builder().pin_id(42).build();
        assert_eq!(value.pin_id(), 42);
    }
}
