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

//! ClientV2 user, role, privilege, and privilege-group operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Create an user with username and password to login milvus.
    pub async fn create_user(&self, request: request::rbac::CreateUserRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, create_credential, request.into_proto())?;
        self.status(status)
    }

    /// Update password of an user.
    pub async fn update_password(
        &self,
        request: request::rbac::UpdatePasswordRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, update_credential, request.into_proto())?;
        self.status(status)
    }

    /// Updates a user account.
    pub async fn update_user(&self, request: request::rbac::UpdateUserRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, update_credential, request.into_proto())?;
        self.status(status)
    }

    /// Drop an user.
    pub async fn drop_user(&self, request: request::rbac::DropUserRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, delete_credential, request.into_proto())?;
        self.status(status)
    }

    /// List users.
    pub async fn list_users(
        &self,
        request: request::rbac::ListUsersRequest,
    ) -> Result<response::rbac::ListUsersResponse> {
        let response = rpc_with_retry!(self, list_cred_users, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::rbac::ListUsersResponse::from_proto(response))
    }

    /// List roles.
    pub async fn list_roles(
        &self,
        request: request::rbac::ListRolesRequest,
    ) -> Result<response::rbac::ListRolesResponse> {
        let response = rpc_with_retry!(self, select_role, request.into_proto())?;
        status_to_result(&response.status)?;
        response::rbac::ListRolesResponse::from_proto(response)
    }

    /// Create a role with specific privileges.
    pub async fn create_role(&self, request: request::rbac::CreateRoleRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, create_role, request.into_proto())?;
        self.status(status)
    }

    /// Updates a role.
    pub async fn alter_role(&self, request: request::rbac::AlterRoleRequest) -> Result<()> {
        let status = status_rpc_with_retry!(NonIdempotent, self, alter_role, request.into_proto())?;
        self.status(status)
    }

    /// Drop a role.
    pub async fn drop_role(&self, request: request::rbac::DropRoleRequest) -> Result<()> {
        let status = status_rpc_with_retry!(NonIdempotent, self, drop_role, request.into_proto())?;
        self.status(status)
    }

    /// Grant a role to an user.
    pub async fn grant_role(&self, request: request::rbac::GrantRoleRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, operate_user_role, request.into_proto())?;
        self.status(status)
    }

    /// Revoke a role from an user.
    pub async fn revoke_role(&self, request: request::rbac::RevokeRoleRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, operate_user_role, request.into_proto())?;
        self.status(status)
    }

    /// Describe an role.
    pub async fn describe_role(
        &self,
        request: request::rbac::DescribeRoleRequest,
    ) -> Result<response::rbac::DescribeRoleResponse> {
        let (role_name, grant_request, role_request) = request.into_proto();
        let grant_response = rpc_with_retry!(self, select_grant, grant_request)?;
        status_to_result(&grant_response.status)?;
        let role_response = rpc_with_retry!(self, select_role, role_request)?;
        status_to_result(&role_response.status)?;
        response::rbac::DescribeRoleResponse::from_proto(role_name, grant_response, role_response)
    }

    /// Describe an user.
    pub async fn describe_user(
        &self,
        request: request::rbac::DescribeUserRequest,
    ) -> Result<response::rbac::DescribeUserResponse> {
        let response = rpc_with_retry!(self, select_user, request.into_proto())?;
        status_to_result(&response.status)?;
        response::rbac::DescribeUserResponse::from_proto(response)
    }

    /// Grants a privilege to a role.
    pub async fn grant_privilege(
        &self,
        request: request::rbac::GrantPrivilegeRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            operate_privilege_v2,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Revokes a privilege from a role.
    pub async fn revoke_privilege(
        &self,
        request: request::rbac::RevokePrivilegeRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            operate_privilege_v2,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Create a privilege group.
    pub async fn create_privilege_group(
        &self,
        request: request::rbac::CreatePrivilegeGroupRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            create_privilege_group,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Drop a privilege group.
    pub async fn drop_privilege_group(
        &self,
        request: request::rbac::DropPrivilegeGroupRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            drop_privilege_group,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// List all the privilege groups.
    pub async fn list_privilege_groups(
        &self,
        request: request::rbac::ListPrivilegeGroupsRequest,
    ) -> Result<response::rbac::ListPrivilegeGroupsResponse> {
        let response = rpc_with_retry!(self, list_privilege_groups, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::rbac::ListPrivilegeGroupsResponse::from_proto(
            response,
        ))
    }

    /// Add privileges to a privilege group.
    pub async fn add_privileges_to_group(
        &self,
        request: request::rbac::AddPrivilegesToGroupRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            operate_privilege_group,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Remove privileges from a privilege group.
    pub async fn remove_privileges_from_group(
        &self,
        request: request::rbac::RemovePrivilegesFromGroupRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            operate_privilege_group,
            request.into_proto()
        )?;
        self.status(status)
    }
}
