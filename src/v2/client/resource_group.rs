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

//! ClientV2 resource-group operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Creates a resource group used to isolate query-node resources.
    pub async fn create_resource_group(
        &self,
        request: request::resource_group::CreateResourceGroupRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            create_resource_group,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Drops a resource group after its resources are no longer assigned to it.
    pub async fn drop_resource_group(
        &self,
        request: request::resource_group::DropResourceGroupRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(Idempotent, self, drop_resource_group, request.into_proto())?;
        self.status(status)
    }

    /// Updates the properties or node allocation of resource groups.
    pub async fn update_resource_groups(
        &self,
        request: request::resource_group::UpdateResourceGroupsRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            update_resource_groups,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Transfers query nodes between resource groups.
    pub async fn transfer_node(
        &self,
        request: request::resource_group::TransferNodeRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, transfer_node, request.into_proto())?;
        self.status(status)
    }

    /// Transfers collection replicas from one resource group to another.
    pub async fn transfer_replica(
        &self,
        request: request::resource_group::TransferReplicaRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, transfer_replica, request.into_proto())?;
        self.status(status)
    }

    /// Lists resource groups visible in the current database.
    pub async fn list_resource_groups(
        &self,
        request: request::resource_group::ListResourceGroupsRequest,
    ) -> Result<response::resource_group::ListResourceGroupsResponse> {
        let response = rpc_with_retry!(self, list_resource_groups, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::resource_group::ListResourceGroupsResponse::from_proto(response))
    }

    /// Retrieves a resource group's node and replica assignments.
    pub async fn describe_resource_group(
        &self,
        request: request::resource_group::DescribeResourceGroupRequest,
    ) -> Result<response::resource_group::DescribeResourceGroupResponse> {
        let response = rpc_with_retry!(self, describe_resource_group, request.into_proto())?;
        status_to_result(&response.status)?;
        response::resource_group::DescribeResourceGroupResponse::from_proto(response)
    }
}
