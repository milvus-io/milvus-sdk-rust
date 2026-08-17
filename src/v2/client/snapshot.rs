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

//! ClientV2 snapshot backup and restore operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Creates a snapshot of a collection.
    ///
    /// `compaction_protection_seconds` protects the referenced segments from
    /// compaction for the given duration; `0` disables the protection.
    pub async fn create_snapshot(
        &self,
        request: request::snapshot::CreateSnapshotRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            create_snapshot,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Drops a snapshot of a collection.
    pub async fn drop_snapshot(
        &self,
        request: request::snapshot::DropSnapshotRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            drop_snapshot,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Lists snapshot names for a collection, or for the whole database when the
    /// collection name is omitted.
    pub async fn list_snapshots(
        &self,
        request: request::snapshot::ListSnapshotsRequest,
    ) -> Result<response::snapshot::ListSnapshotsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, list_snapshots, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::snapshot::ListSnapshotsResponse::from_proto(
            response,
        ))
    }

    /// Describes a snapshot of a collection.
    pub async fn describe_snapshot(
        &self,
        request: request::snapshot::DescribeSnapshotRequest,
    ) -> Result<response::snapshot::DescribeSnapshotResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, describe_snapshot, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::snapshot::DescribeSnapshotResponse::from_proto(
            response,
        ))
    }

    /// Restores a snapshot to a new collection.
    ///
    /// The target collection must not exist. The returned job id tracks the
    /// asynchronous restore through [`ClientV2::get_restore_snapshot_state`].
    pub async fn restore_snapshot(
        &self,
        request: request::snapshot::RestoreSnapshotRequest,
    ) -> Result<response::snapshot::RestoreSnapshotResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            NonIdempotent,
            self,
            restore_snapshot,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::snapshot::RestoreSnapshotResponse::from_proto(
            response,
        ))
    }

    /// Retrieves the state and progress of a restore-snapshot job.
    pub async fn get_restore_snapshot_state(
        &self,
        request: request::snapshot::GetRestoreSnapshotStateRequest,
    ) -> Result<response::snapshot::GetRestoreSnapshotStateResponse> {
        let response = rpc_with_retry!(self, get_restore_snapshot_state, request.into_proto())?;
        status_to_result(&response.status)?;
        response::snapshot::GetRestoreSnapshotStateResponse::from_proto(response)
    }

    /// Lists restore-snapshot jobs for a collection, or for the whole database
    /// when the collection name is omitted.
    pub async fn list_restore_snapshot_jobs(
        &self,
        request: request::snapshot::ListRestoreSnapshotJobsRequest,
    ) -> Result<response::snapshot::ListRestoreSnapshotJobsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            self,
            list_restore_snapshot_jobs,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::snapshot::ListRestoreSnapshotJobsResponse::from_proto(response))
    }

    /// Pins the data referenced by a snapshot so it is not reclaimed.
    ///
    /// `ttl_seconds` of `0` means the pin never expires; otherwise it auto-expires
    /// after the given number of seconds. The returned pin id is passed to
    /// [`ClientV2::unpin_snapshot_data`].
    pub async fn pin_snapshot_data(
        &self,
        request: request::snapshot::PinSnapshotDataRequest,
    ) -> Result<response::snapshot::PinSnapshotDataResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            NonIdempotent,
            self,
            pin_snapshot_data,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::snapshot::PinSnapshotDataResponse::from_proto(
            response,
        ))
    }

    /// Releases a snapshot data pin created by [`ClientV2::pin_snapshot_data`].
    pub async fn unpin_snapshot_data(
        &self,
        request: request::snapshot::UnpinSnapshotDataRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            unpin_snapshot_data,
            request.into_proto()
        )?;
        self.status(status)
    }
}
