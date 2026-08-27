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
use crate::v2::error::{Error, Result};
use crate::v2::{request, response};

impl ClientV2 {
    /// Creates a user account with a username and password.
    ///
    /// The caller must have the corresponding administrative privilege. Milvus can accept RBAC
    /// management calls with authorization disabled; enabling authorization is required when the
    /// deployment must enforce the resulting permissions for subsequent operations.
    pub async fn create_user(&self, request: request::rbac::CreateUserRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, create_credential, request.into_proto())?;
        self.status(status)
    }

    /// Changes an existing user's password.
    ///
    /// When the request sets `reset_connection`, the client re-establishes its connection using
    /// the updated username/password credentials, mirroring the pymilvus `reset_connection`
    /// convenience, so subsequent requests authenticate with the new password.
    ///
    /// The password change is applied server-side before the connection is reset. If the reset
    /// fails, the returned error states that the password was changed but the connection could not
    /// be re-established; callers should re-authenticate with the new password rather than retry
    /// the update with the old one.
    pub async fn update_password(
        &self,
        request: request::rbac::UpdatePasswordRequest,
    ) -> Result<()> {
        let reset_connection = request.should_reset_connection();
        let username = request.username().to_owned();
        let new_password = request.new_password().to_owned();
        let status =
            status_rpc_with_retry!(NonIdempotent, self, update_credential, request.into_proto())?;
        self.status(status)?;
        if reset_connection {
            let mut config = self.connect_config.read().clone();
            config.set_token(format!("{username}:{new_password}"));
            self.reset_connection(config).await.map_err(|error| {
                Error::Unexpected(format!(
                    "password changed but connection reset failed: {error}"
                ))
            })?;
        }
        Ok(())
    }

    /// Updates mutable properties of a user account.
    pub async fn update_user(&self, request: request::rbac::UpdateUserRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, update_credential, request.into_proto())?;
        self.status(status)
    }

    /// Drops a user account.
    pub async fn drop_user(&self, request: request::rbac::DropUserRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, delete_credential, request.into_proto())?;
        self.status(status)
    }

    /// Lists user accounts visible to the caller.
    pub async fn list_users(
        &self,
        request: request::rbac::ListUsersRequest,
    ) -> Result<response::rbac::ListUsersResponse> {
        let response = rpc_with_retry!(self, list_cred_users, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::rbac::ListUsersResponse::from_proto(response))
    }

    /// Lists roles visible to the caller.
    pub async fn list_roles(
        &self,
        request: request::rbac::ListRolesRequest,
    ) -> Result<response::rbac::ListRolesResponse> {
        let response = rpc_with_retry!(self, select_role, request.into_proto())?;
        status_to_result(&response.status)?;
        response::rbac::ListRolesResponse::from_proto(response)
    }

    /// Creates a role that can receive privileges.
    pub async fn create_role(&self, request: request::rbac::CreateRoleRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, create_role, request.into_proto())?;
        self.status(status)
    }

    /// Updates a role's mutable properties.
    pub async fn alter_role(&self, request: request::rbac::AlterRoleRequest) -> Result<()> {
        let status = status_rpc_with_retry!(NonIdempotent, self, alter_role, request.into_proto())?;
        self.status(status)
    }

    /// Drops a role.
    pub async fn drop_role(&self, request: request::rbac::DropRoleRequest) -> Result<()> {
        let status = status_rpc_with_retry!(NonIdempotent, self, drop_role, request.into_proto())?;
        self.status(status)
    }

    /// Assigns a role to a user.
    pub async fn grant_role(&self, request: request::rbac::GrantRoleRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, operate_user_role, request.into_proto())?;
        self.status(status)
    }

    /// Removes a role assignment from a user.
    pub async fn revoke_role(&self, request: request::rbac::RevokeRoleRequest) -> Result<()> {
        let status =
            status_rpc_with_retry!(NonIdempotent, self, operate_user_role, request.into_proto())?;
        self.status(status)
    }

    /// Retrieves a role and its assignments.
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

    /// Retrieves a user and its role assignments.
    pub async fn describe_user(
        &self,
        request: request::rbac::DescribeUserRequest,
    ) -> Result<response::rbac::DescribeUserResponse> {
        let response = rpc_with_retry!(self, select_user, request.into_proto())?;
        status_to_result(&response.status)?;
        response::rbac::DescribeUserResponse::from_proto(response)
    }

    /// Grants a privilege on a resource to a role.
    ///
    /// Collection-scoped grants use the v2 privilege RPC; grants configured with the legacy
    /// `object_type`/`object_name` surface (for example USER objects or explicit GLOBAL scope)
    /// use the v1 `OperatePrivilege` RPC.
    pub async fn grant_privilege(
        &self,
        request: request::rbac::GrantPrivilegeRequest,
    ) -> Result<()> {
        if request.uses_legacy_object() {
            let status = status_rpc_with_retry!(
                NonIdempotent,
                self,
                operate_privilege,
                request.into_legacy_proto()
            )?;
            self.status(status)
        } else {
            let status = status_rpc_with_retry!(
                NonIdempotent,
                self,
                operate_privilege_v2,
                request.into_proto()
            )?;
            self.status(status)
        }
    }

    /// Revokes a privilege from a role.
    ///
    /// Collection-scoped revokes use the v2 privilege RPC; revokes configured with the legacy
    /// `object_type`/`object_name` surface use the v1 `OperatePrivilege` RPC.
    pub async fn revoke_privilege(
        &self,
        request: request::rbac::RevokePrivilegeRequest,
    ) -> Result<()> {
        if request.uses_legacy_object() {
            let status = status_rpc_with_retry!(
                NonIdempotent,
                self,
                operate_privilege,
                request.into_legacy_proto()
            )?;
            self.status(status)
        } else {
            let status = status_rpc_with_retry!(
                NonIdempotent,
                self,
                operate_privilege_v2,
                request.into_proto()
            )?;
            self.status(status)
        }
    }

    /// Creates a named group of privileges.
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

    /// Drops a privilege group.
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

    /// Lists privilege groups visible to the caller.
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

    /// Adds privileges to a privilege group.
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

    /// Removes privileges from a privilege group.
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
