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

//! ClientV2 partition operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Creates a named partition in an existing collection.
    pub async fn create_partition(
        &self,
        request: request::partition::CreatePartitionRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            create_partition,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Drops a partition and its associated index and segment data.
    pub async fn drop_partition(
        &self,
        request: request::partition::DropPartitionRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            drop_partition,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Checks whether a named partition exists.
    pub async fn has_partition(
        &self,
        request: request::partition::HasPartitionRequest,
    ) -> Result<response::partition::HasPartitionResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, has_partition, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::partition::HasPartitionResponse(response.value))
    }

    /// Lists partitions defined for a collection.
    pub async fn list_partitions(
        &self,
        request: request::partition::ListPartitionsRequest,
    ) -> Result<response::partition::ListPartitionsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, show_partitions, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        response::partition::ListPartitionsResponse::from_proto(response)
    }

    /// Loads selected partitions into query-node memory.
    ///
    /// A synchronous request polls load state until completion; an asynchronous request returns
    /// once the server accepts the load operation.
    pub async fn load_partitions(
        &self,
        mut request: request::partition::LoadPartitionsRequest,
    ) -> Result<()> {
        let sync = request.sync;
        let timeout_ms = request.timeout_ms;
        let refresh = request.refresh;
        let database = self.effective_database(request.database_name.as_deref());
        request.database_name = Some(database.clone());
        let collection = request.collection_name.clone();
        let partition_names = request.partition_names.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            load_partitions,
            request.into_proto(&database)
        )?;
        self.status(status)?;

        if !sync {
            return Ok(());
        }

        self.wait_for_collection_loading(
            &database,
            &collection,
            &partition_names,
            refresh,
            timeout_ms,
            "load partitions timed out",
        )
        .await
    }

    /// Releases selected partitions from query-node memory without dropping their data.
    pub async fn release_partitions(
        &self,
        request: request::partition::ReleasePartitionsRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            release_partitions,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Returns server-reported statistics for a partition, including its row count.
    pub async fn get_partition_stats(
        &self,
        request: request::partition::GetPartitionStatsRequest,
    ) -> Result<response::partition::GetPartitionStatsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            self,
            get_partition_statistics,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::partition::GetPartitionStatsResponse::from_proto(
            response,
        ))
    }
}
