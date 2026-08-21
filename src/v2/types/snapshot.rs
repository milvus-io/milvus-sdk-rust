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

//! Shared domain types for snapshot backup and restore operations.

use crate::proto::milvus;

///////////////////////////////////////////////////////////////////////////////
// RestoreSnapshotStateCode
///////////////////////////////////////////////////////////////////////////////
/// Execution state of a restore-snapshot job.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RestoreSnapshotStateCode {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the Pending case.
    Pending,
    /// Represents the Executing case.
    Executing,
    /// Represents the Completed case.
    Completed,
    /// Represents the Failed case.
    Failed,
}

impl RestoreSnapshotStateCode {
    pub(crate) fn from_proto(value: i32) -> Self {
        match milvus::RestoreSnapshotState::try_from(value).ok() {
            Some(milvus::RestoreSnapshotState::RestoreSnapshotPending) => Self::Pending,
            Some(milvus::RestoreSnapshotState::RestoreSnapshotExecuting) => Self::Executing,
            Some(milvus::RestoreSnapshotState::RestoreSnapshotCompleted) => Self::Completed,
            Some(milvus::RestoreSnapshotState::RestoreSnapshotFailed) => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RestoreSnapshotJobInfo
///////////////////////////////////////////////////////////////////////////////
/// Metadata and progress of a restore-snapshot job.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RestoreSnapshotJobInfo {
    pub(crate) job_id: i64,
    pub(crate) snapshot_name: String,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) state: RestoreSnapshotStateCode,
    pub(crate) progress: i32,
    pub(crate) reason: String,
    pub(crate) start_time: u64,
    pub(crate) time_cost: u64,
}

impl RestoreSnapshotJobInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            job_id: 0,
            snapshot_name: String::new(),
            database_name: String::new(),
            collection_name: String::new(),
            state: RestoreSnapshotStateCode::Unknown,
            progress: 0,
            reason: String::new(),
            start_time: 0,
            time_cost: 0,
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

    /// Sets the snapshot name and returns the updated value.
    pub fn snapshot_name(mut self, value: impl Into<String>) -> Self {
        self.snapshot_name = value.into();
        self
    }

    /// Sets the snapshot name and returns this value for further mutation.
    pub fn set_snapshot_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.snapshot_name = value.into();
        self
    }

    /// Returns the snapshot name.
    pub fn get_snapshot_name(&self) -> &str {
        &self.snapshot_name
    }

    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.database_name = value.into();
        self
    }

    /// Sets the database name and returns this value for further mutation.
    pub fn set_database_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.database_name = value.into();
        self
    }

    /// Returns the database name.
    pub fn get_database_name(&self) -> &str {
        &self.database_name
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
    pub fn state(mut self, value: RestoreSnapshotStateCode) -> Self {
        self.state = value;
        self
    }

    /// Sets the state and returns this value for further mutation.
    pub fn set_state(&mut self, value: RestoreSnapshotStateCode) -> &mut Self {
        self.state = value;
        self
    }

    /// Returns the state.
    pub fn get_state(&self) -> RestoreSnapshotStateCode {
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

    /// Sets the start timestamp in milliseconds and returns the updated value.
    pub fn start_time(mut self, value: u64) -> Self {
        self.start_time = value;
        self
    }

    /// Sets the start timestamp in milliseconds and returns this value for further mutation.
    pub fn set_start_time(&mut self, value: u64) -> &mut Self {
        self.start_time = value;
        self
    }

    /// Returns the start timestamp in milliseconds.
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }

    /// Sets the time cost in milliseconds and returns the updated value.
    pub fn time_cost(mut self, value: u64) -> Self {
        self.time_cost = value;
        self
    }

    /// Sets the time cost in milliseconds and returns this value for further mutation.
    pub fn set_time_cost(&mut self, value: u64) -> &mut Self {
        self.time_cost = value;
        self
    }

    /// Returns the time cost in milliseconds.
    pub fn get_time_cost(&self) -> u64 {
        self.time_cost
    }

    pub(crate) fn from_proto(value: milvus::RestoreSnapshotInfo) -> Self {
        Self {
            job_id: value.job_id,
            snapshot_name: value.snapshot_name,
            database_name: value.db_name,
            collection_name: value.collection_name,
            state: RestoreSnapshotStateCode::from_proto(value.state),
            progress: value.progress,
            reason: value.reason,
            start_time: value.start_time,
            time_cost: value.time_cost,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{RestoreSnapshotJobInfo, RestoreSnapshotStateCode};
    use crate::proto::milvus;

    #[test]
    fn restore_snapshot_state_code_accepts_unknown_proto_values() {
        assert_eq!(
            RestoreSnapshotStateCode::from_proto(-1),
            RestoreSnapshotStateCode::Unknown
        );
        assert_eq!(
            RestoreSnapshotStateCode::from_proto(
                milvus::RestoreSnapshotState::RestoreSnapshotCompleted as i32
            ),
            RestoreSnapshotStateCode::Completed
        );
    }

    #[test]
    fn restore_snapshot_job_info_default_values() {
        let value = RestoreSnapshotJobInfo::new();
        assert_eq!(value.get_job_id(), 0);
        assert_eq!(value.get_snapshot_name(), "");
        assert_eq!(value.get_database_name(), "");
        assert_eq!(value.get_collection_name(), "");
        assert_eq!(value.get_state(), RestoreSnapshotStateCode::Unknown);
        assert_eq!(value.get_progress(), 0);
        assert_eq!(value.get_reason(), "");
        assert_eq!(value.get_start_time(), 0);
        assert_eq!(value.get_time_cost(), 0);
    }

    #[test]
    fn restore_snapshot_job_info_populated_values() {
        let mut value = RestoreSnapshotJobInfo::new()
            .job_id(7)
            .snapshot_name("snap-1")
            .database_name("default")
            .collection_name("books")
            .state(RestoreSnapshotStateCode::Executing)
            .progress(50)
            .reason("working")
            .start_time(1000)
            .time_cost(5);
        value.set_state(RestoreSnapshotStateCode::Completed);
        assert_eq!(value.get_job_id(), 7);
        assert_eq!(value.get_snapshot_name(), "snap-1");
        assert_eq!(value.get_database_name(), "default");
        assert_eq!(value.get_collection_name(), "books");
        assert_eq!(value.get_state(), RestoreSnapshotStateCode::Completed);
        assert_eq!(value.get_progress(), 50);
        assert_eq!(value.get_reason(), "working");
        assert_eq!(value.get_start_time(), 1000);
        assert_eq!(value.get_time_cost(), 5);
    }

    #[test]
    fn restore_snapshot_job_info_converts_from_proto() {
        let value = RestoreSnapshotJobInfo::from_proto(milvus::RestoreSnapshotInfo {
            job_id: 3,
            snapshot_name: "snap-a".into(),
            db_name: "default".into(),
            collection_name: "books".into(),
            state: milvus::RestoreSnapshotState::RestoreSnapshotFailed as i32,
            progress: 100,
            reason: "boom".into(),
            start_time: 11,
            time_cost: 22,
            ..Default::default()
        });
        assert_eq!(value.get_job_id(), 3);
        assert_eq!(value.get_state(), RestoreSnapshotStateCode::Failed);
        assert_eq!(value.get_reason(), "boom");
        assert_eq!(value.get_time_cost(), 22);
    }
}
