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

//! Request types for users, roles, privileges, and privilege groups.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::required;
use base64::Engine;
use std::collections::HashSet;

///////////////////////////////////////////////////////////////////////////////
// CreateUserRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_user operation.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateUserRequest {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) description: Option<String>,
}

impl std::fmt::Debug for CreateUserRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CreateUserRequest")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("description", &self.description)
            .finish()
    }
}

impl CreateUserRequest {
    fn empty() -> Self {
        Self {
            username: Default::default(),
            password: Default::default(),
            description: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CreateUserRequestBuilder {
        CreateUserRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateUserRequestBuilder {
        CreateUserRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the password.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Returns the description.
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    pub(crate) fn into_proto(self) -> milvus::CreateCredentialRequest {
        milvus::CreateCredentialRequest {
            base: None,
            username: self.username,
            password: base64::engine::general_purpose::STANDARD.encode(self.password),
            created_utc_timestamps: 0,
            modified_utc_timestamps: 0,
            description: self.description,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateUserRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateUserRequest.
#[derive(Debug, Clone)]
pub struct CreateUserRequestBuilder {
    value: CreateUserRequest,
}

impl CreateUserRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Sets the password and returns the updated value.
    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.value.password = value.into();
        self
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = Some(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateUserRequest> {
        required("username", &self.value.username)?;
        required("password", &self.value.password)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdatePasswordRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 update_password operation.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpdatePasswordRequest {
    pub(crate) username: String,
    pub(crate) old_password: String,
    pub(crate) new_password: String,
}

impl std::fmt::Debug for UpdatePasswordRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UpdatePasswordRequest")
            .field("username", &self.username)
            .field("old_password", &"[REDACTED]")
            .field("new_password", &"[REDACTED]")
            .finish()
    }
}

impl UpdatePasswordRequest {
    fn empty() -> Self {
        Self {
            username: Default::default(),
            old_password: Default::default(),
            new_password: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> UpdatePasswordRequestBuilder {
        UpdatePasswordRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UpdatePasswordRequestBuilder {
        UpdatePasswordRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the old password.
    pub fn old_password(&self) -> &str {
        &self.old_password
    }

    /// Returns the new password.
    pub fn new_password(&self) -> &str {
        &self.new_password
    }

    pub(crate) fn into_proto(self) -> milvus::UpdateCredentialRequest {
        let mut v = milvus::UpdateCredentialRequest::default();
        v.username = self.username;
        v.old_password = base64::engine::general_purpose::STANDARD.encode(self.old_password);
        v.new_password = base64::engine::general_purpose::STANDARD.encode(self.new_password);
        v
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdatePasswordRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UpdatePasswordRequest.
#[derive(Debug, Clone)]
pub struct UpdatePasswordRequestBuilder {
    value: UpdatePasswordRequest,
}

impl UpdatePasswordRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Sets the old password and returns the updated value.
    pub fn old_password(mut self, value: impl Into<String>) -> Self {
        self.value.old_password = value.into();
        self
    }

    /// Sets the new password and returns the updated value.
    pub fn new_password(mut self, value: impl Into<String>) -> Self {
        self.value.new_password = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<UpdatePasswordRequest> {
        required("username", &self.value.username)?;
        required("old_password", &self.value.old_password)?;
        required("new_password", &self.value.new_password)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateUserRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 update_user operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpdateUserRequest {
    pub(crate) username: String,
    pub(crate) description: String,
}

impl UpdateUserRequest {
    fn empty() -> Self {
        Self {
            username: Default::default(),
            description: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> UpdateUserRequestBuilder {
        UpdateUserRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UpdateUserRequestBuilder {
        UpdateUserRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    pub(crate) fn into_proto(self) -> milvus::UpdateCredentialRequest {
        milvus::UpdateCredentialRequest {
            base: None,
            username: self.username,
            description: Some(self.description),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateUserRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UpdateUserRequest.
#[derive(Debug, Clone)]
pub struct UpdateUserRequestBuilder {
    value: UpdateUserRequest,
}

impl UpdateUserRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<UpdateUserRequest> {
        required("username", &self.value.username)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropUserRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_user operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropUserRequest {
    pub(crate) username: String,
}

impl DropUserRequest {
    fn empty() -> Self {
        Self {
            username: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropUserRequestBuilder {
        DropUserRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropUserRequestBuilder {
        DropUserRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn into_proto(self) -> milvus::DeleteCredentialRequest {
        milvus::DeleteCredentialRequest {
            base: None,
            username: self.username,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropUserRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropUserRequest.
#[derive(Debug, Clone)]
pub struct DropUserRequestBuilder {
    value: DropUserRequest,
}

impl DropUserRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropUserRequest> {
        required("username", &self.value.username)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListUsersRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_users operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListUsersRequest;

impl ListUsersRequest {
    /// Creates a builder for this request.
    pub fn builder() -> ListUsersRequestBuilder {
        ListUsersRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListUsersRequestBuilder {
        ListUsersRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::ListCredUsersRequest {
        milvus::ListCredUsersRequest::default()
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListUsersRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListUsersRequest.
#[derive(Debug, Clone, Copy)]
pub struct ListUsersRequestBuilder;

impl ListUsersRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListUsersRequest> {
        Ok(ListUsersRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRolesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_roles operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListRolesRequest;

impl ListRolesRequest {
    /// Creates a builder for this request.
    pub fn builder() -> ListRolesRequestBuilder {
        ListRolesRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListRolesRequestBuilder {
        ListRolesRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::SelectRoleRequest {
        milvus::SelectRoleRequest {
            base: None,
            role: None,
            include_user_info: false,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRolesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRolesRequest.
#[derive(Debug, Clone, Copy)]
pub struct ListRolesRequestBuilder;

impl ListRolesRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListRolesRequest> {
        Ok(ListRolesRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateRoleRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateRoleRequest {
    pub(crate) role_name: String,
    pub(crate) description: String,
}

impl CreateRoleRequest {
    fn empty() -> Self {
        Self {
            role_name: Default::default(),
            description: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CreateRoleRequestBuilder {
        CreateRoleRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateRoleRequestBuilder {
        CreateRoleRequestBuilder { value: self }
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    pub(crate) fn into_proto(self) -> milvus::CreateRoleRequest {
        milvus::CreateRoleRequest {
            base: None,
            entity: Some(milvus::RoleEntity {
                name: self.role_name,
                description: self.description,
            }),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateRoleRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateRoleRequest.
#[derive(Debug, Clone)]
pub struct CreateRoleRequestBuilder {
    value: CreateRoleRequest,
}

impl CreateRoleRequestBuilder {
    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateRoleRequest> {
        required("role_name", &self.value.role_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterRoleRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AlterRoleRequest {
    pub(crate) role_name: String,
    pub(crate) description: String,
}

impl AlterRoleRequest {
    fn empty() -> Self {
        Self {
            role_name: Default::default(),
            description: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AlterRoleRequestBuilder {
        AlterRoleRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterRoleRequestBuilder {
        AlterRoleRequestBuilder { value: self }
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    pub(crate) fn into_proto(self) -> milvus::AlterRoleRequest {
        milvus::AlterRoleRequest {
            base: None,
            role_name: self.role_name,
            description: self.description,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterRoleRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterRoleRequest.
#[derive(Debug, Clone)]
pub struct AlterRoleRequestBuilder {
    value: AlterRoleRequest,
}

impl AlterRoleRequestBuilder {
    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AlterRoleRequest> {
        required("role_name", &self.value.role_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropRoleRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropRoleRequest {
    pub(crate) role_name: String,
    pub(crate) force: bool,
}

impl DropRoleRequest {
    fn empty() -> Self {
        Self {
            role_name: Default::default(),
            force: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropRoleRequestBuilder {
        DropRoleRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropRoleRequestBuilder {
        DropRoleRequestBuilder { value: self }
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns whether the request should force.
    pub fn should_force(&self) -> bool {
        self.force
    }

    pub(crate) fn into_proto(self) -> milvus::DropRoleRequest {
        milvus::DropRoleRequest {
            base: None,
            role_name: self.role_name,
            force_drop: self.force,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropRoleRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropRoleRequest.
#[derive(Debug, Clone)]
pub struct DropRoleRequestBuilder {
    value: DropRoleRequest,
}

impl DropRoleRequestBuilder {
    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Sets the force and returns the updated value.
    pub fn force(mut self, value: bool) -> Self {
        self.value.force = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropRoleRequest> {
        required("role_name", &self.value.role_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GrantRoleRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 grant_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GrantRoleRequest {
    pub(crate) username: String,
    pub(crate) role_name: String,
}

impl GrantRoleRequest {
    fn empty() -> Self {
        Self {
            username: Default::default(),
            role_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GrantRoleRequestBuilder {
        GrantRoleRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GrantRoleRequestBuilder {
        GrantRoleRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    pub(crate) fn into_proto(self) -> milvus::OperateUserRoleRequest {
        milvus::OperateUserRoleRequest {
            base: None,
            username: self.username,
            role_name: self.role_name,
            r#type: milvus::OperateUserRoleType::AddUserToRole as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GrantRoleRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GrantRoleRequest.
#[derive(Debug, Clone)]
pub struct GrantRoleRequestBuilder {
    value: GrantRoleRequest,
}

impl GrantRoleRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GrantRoleRequest> {
        validate_user_role(&self.value.username, &self.value.role_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RevokeRoleRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 revoke_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RevokeRoleRequest {
    pub(crate) username: String,
    pub(crate) role_name: String,
}

impl RevokeRoleRequest {
    fn empty() -> Self {
        Self {
            username: Default::default(),
            role_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RevokeRoleRequestBuilder {
        RevokeRoleRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RevokeRoleRequestBuilder {
        RevokeRoleRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    pub(crate) fn into_proto(self) -> milvus::OperateUserRoleRequest {
        milvus::OperateUserRoleRequest {
            base: None,
            username: self.username,
            role_name: self.role_name,
            r#type: milvus::OperateUserRoleType::RemoveUserFromRole as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RevokeRoleRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RevokeRoleRequest.
#[derive(Debug, Clone)]
pub struct RevokeRoleRequestBuilder {
    value: RevokeRoleRequest,
}

impl RevokeRoleRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RevokeRoleRequest> {
        validate_user_role(&self.value.username, &self.value.role_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeRoleRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeRoleRequest {
    pub(crate) role_name: String,
    pub(crate) database_name: String,
}

impl DescribeRoleRequest {
    fn empty() -> Self {
        Self {
            role_name: Default::default(),
            database_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DescribeRoleRequestBuilder {
        DescribeRoleRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeRoleRequestBuilder {
        DescribeRoleRequestBuilder { value: self }
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub(crate) fn into_proto(
        self,
    ) -> (
        String,
        milvus::SelectGrantRequest,
        milvus::SelectRoleRequest,
    ) {
        let role_name = self.role_name;
        let role = milvus::RoleEntity {
            name: role_name.clone(),
            description: String::new(),
        };
        (
            role_name,
            milvus::SelectGrantRequest {
                base: None,
                entity: Some(milvus::GrantEntity {
                    role: Some(role.clone()),
                    object: None,
                    object_name: String::new(),
                    grantor: None,
                    db_name: self.database_name,
                }),
            },
            milvus::SelectRoleRequest {
                base: None,
                role: Some(role),
                include_user_info: false,
            },
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeRoleRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeRoleRequest.
#[derive(Debug, Clone)]
pub struct DescribeRoleRequestBuilder {
    value: DescribeRoleRequest,
}

impl DescribeRoleRequestBuilder {
    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeRoleRequest> {
        required("role_name", &self.value.role_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeUserRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_user operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeUserRequest {
    pub(crate) username: String,
    pub(crate) include_roles: bool,
}

impl DescribeUserRequest {
    /// Creates a builder for this request.
    pub fn builder() -> DescribeUserRequestBuilder {
        DescribeUserRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeUserRequestBuilder {
        DescribeUserRequestBuilder { value: self }
    }

    /// Returns the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns whether the request should include roles.
    pub fn should_include_roles(&self) -> bool {
        self.include_roles
    }

    pub(crate) fn into_proto(self) -> milvus::SelectUserRequest {
        milvus::SelectUserRequest {
            base: None,
            user: Some(milvus::UserEntity {
                name: self.username,
            }),
            include_role_info: self.include_roles,
        }
    }
}

impl DescribeUserRequest {
    fn empty() -> Self {
        Self {
            username: String::new(),
            include_roles: true,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeUserRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeUserRequest.
#[derive(Debug, Clone)]
pub struct DescribeUserRequestBuilder {
    value: DescribeUserRequest,
}

impl DescribeUserRequestBuilder {
    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.value.username = value.into();
        self
    }

    /// Sets the include roles and returns the updated value.
    pub fn include_roles(mut self, value: bool) -> Self {
        self.value.include_roles = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeUserRequest> {
        required("username", &self.value.username)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GrantPrivilegeRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 grant_privilege operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GrantPrivilegeRequest {
    pub(crate) role_name: String,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) privilege: String,
}

impl GrantPrivilegeRequest {
    fn empty() -> Self {
        Self {
            role_name: Default::default(),
            database_name: Default::default(),
            collection_name: Default::default(),
            privilege: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GrantPrivilegeRequestBuilder {
        GrantPrivilegeRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GrantPrivilegeRequestBuilder {
        GrantPrivilegeRequestBuilder { value: self }
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the privilege.
    pub fn privilege(&self) -> &str {
        &self.privilege
    }

    pub(crate) fn into_proto(self) -> milvus::OperatePrivilegeV2Request {
        milvus::OperatePrivilegeV2Request {
            base: None,
            role: Some(milvus::RoleEntity {
                name: self.role_name,
                description: String::new(),
            }),
            grantor: Some(milvus::GrantorEntity {
                user: None,
                privilege: Some(milvus::PrivilegeEntity {
                    name: self.privilege,
                }),
            }),
            r#type: milvus::OperatePrivilegeType::Grant as i32,
            db_name: self.database_name,
            collection_name: self.collection_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GrantPrivilegeRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GrantPrivilegeRequest.
#[derive(Debug, Clone)]
pub struct GrantPrivilegeRequestBuilder {
    value: GrantPrivilegeRequest,
}

impl GrantPrivilegeRequestBuilder {
    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the privilege and returns the updated value.
    pub fn privilege(mut self, value: impl Into<String>) -> Self {
        self.value.privilege = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GrantPrivilegeRequest> {
        validate_privilege(
            &self.value.role_name,
            &self.value.database_name,
            &self.value.collection_name,
            &self.value.privilege,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RevokePrivilegeRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 revoke_privilege operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RevokePrivilegeRequest {
    pub(crate) role_name: String,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) privilege: String,
}

impl RevokePrivilegeRequest {
    fn empty() -> Self {
        Self {
            role_name: Default::default(),
            database_name: Default::default(),
            collection_name: Default::default(),
            privilege: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RevokePrivilegeRequestBuilder {
        RevokePrivilegeRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RevokePrivilegeRequestBuilder {
        RevokePrivilegeRequestBuilder { value: self }
    }

    /// Returns the role name.
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the privilege.
    pub fn privilege(&self) -> &str {
        &self.privilege
    }

    pub(crate) fn into_proto(self) -> milvus::OperatePrivilegeV2Request {
        milvus::OperatePrivilegeV2Request {
            base: None,
            role: Some(milvus::RoleEntity {
                name: self.role_name,
                description: String::new(),
            }),
            grantor: Some(milvus::GrantorEntity {
                user: None,
                privilege: Some(milvus::PrivilegeEntity {
                    name: self.privilege,
                }),
            }),
            r#type: milvus::OperatePrivilegeType::Revoke as i32,
            db_name: self.database_name,
            collection_name: self.collection_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RevokePrivilegeRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RevokePrivilegeRequest.
#[derive(Debug, Clone)]
pub struct RevokePrivilegeRequestBuilder {
    value: RevokePrivilegeRequest,
}

impl RevokePrivilegeRequestBuilder {
    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.value.role_name = value.into();
        self
    }

    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the privilege and returns the updated value.
    pub fn privilege(mut self, value: impl Into<String>) -> Self {
        self.value.privilege = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RevokePrivilegeRequest> {
        validate_privilege(
            &self.value.role_name,
            &self.value.database_name,
            &self.value.collection_name,
            &self.value.privilege,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreatePrivilegeGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_privilege_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreatePrivilegeGroupRequest {
    pub(crate) group_name: String,
}

impl CreatePrivilegeGroupRequest {
    fn empty() -> Self {
        Self {
            group_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CreatePrivilegeGroupRequestBuilder {
        CreatePrivilegeGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreatePrivilegeGroupRequestBuilder {
        CreatePrivilegeGroupRequestBuilder { value: self }
    }

    /// Returns the group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    pub(crate) fn into_proto(self) -> milvus::CreatePrivilegeGroupRequest {
        milvus::CreatePrivilegeGroupRequest {
            base: None,
            group_name: self.group_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreatePrivilegeGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreatePrivilegeGroupRequest.
#[derive(Debug, Clone)]
pub struct CreatePrivilegeGroupRequestBuilder {
    value: CreatePrivilegeGroupRequest,
}

impl CreatePrivilegeGroupRequestBuilder {
    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.value.group_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreatePrivilegeGroupRequest> {
        required("group_name", &self.value.group_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropPrivilegeGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_privilege_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropPrivilegeGroupRequest {
    pub(crate) group_name: String,
}

impl DropPrivilegeGroupRequest {
    fn empty() -> Self {
        Self {
            group_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropPrivilegeGroupRequestBuilder {
        DropPrivilegeGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropPrivilegeGroupRequestBuilder {
        DropPrivilegeGroupRequestBuilder { value: self }
    }

    /// Returns the group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    pub(crate) fn into_proto(self) -> milvus::DropPrivilegeGroupRequest {
        milvus::DropPrivilegeGroupRequest {
            base: None,
            group_name: self.group_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropPrivilegeGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropPrivilegeGroupRequest.
#[derive(Debug, Clone)]
pub struct DropPrivilegeGroupRequestBuilder {
    value: DropPrivilegeGroupRequest,
}

impl DropPrivilegeGroupRequestBuilder {
    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.value.group_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropPrivilegeGroupRequest> {
        required("group_name", &self.value.group_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPrivilegeGroupsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_privilege_groups operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPrivilegeGroupsRequest;

impl ListPrivilegeGroupsRequest {
    /// Creates a builder for this request.
    pub fn builder() -> ListPrivilegeGroupsRequestBuilder {
        ListPrivilegeGroupsRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListPrivilegeGroupsRequestBuilder {
        ListPrivilegeGroupsRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::ListPrivilegeGroupsRequest {
        milvus::ListPrivilegeGroupsRequest::default()
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPrivilegeGroupsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListPrivilegeGroupsRequest.
#[derive(Debug, Clone, Copy)]
pub struct ListPrivilegeGroupsRequestBuilder;

impl ListPrivilegeGroupsRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListPrivilegeGroupsRequest> {
        Ok(ListPrivilegeGroupsRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddPrivilegesToGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 add_privileges_to_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AddPrivilegesToGroupRequest {
    pub(crate) group_name: String,
    pub(crate) privileges: HashSet<String>,
}

impl AddPrivilegesToGroupRequest {
    fn empty() -> Self {
        Self {
            group_name: Default::default(),
            privileges: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AddPrivilegesToGroupRequestBuilder {
        AddPrivilegesToGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AddPrivilegesToGroupRequestBuilder {
        AddPrivilegesToGroupRequestBuilder { value: self }
    }

    /// Returns the group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    /// Returns the privileges.
    pub fn privileges(&self) -> &HashSet<String> {
        &self.privileges
    }

    pub(crate) fn into_proto(self) -> milvus::OperatePrivilegeGroupRequest {
        milvus::OperatePrivilegeGroupRequest {
            base: None,
            group_name: self.group_name,
            privileges: self
                .privileges
                .into_iter()
                .map(|name| milvus::PrivilegeEntity { name })
                .collect(),
            r#type: milvus::OperatePrivilegeGroupType::AddPrivilegesToGroup as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddPrivilegesToGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AddPrivilegesToGroupRequest.
#[derive(Debug, Clone)]
pub struct AddPrivilegesToGroupRequestBuilder {
    value: AddPrivilegesToGroupRequest,
}

impl AddPrivilegesToGroupRequestBuilder {
    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.value.group_name = value.into();
        self
    }

    /// Sets the privileges and returns the updated value.
    pub fn privileges(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.privileges = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the privilege and returns the updated value.
    pub fn privilege(mut self, value: impl Into<String>) -> Self {
        self.value.privileges.insert(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AddPrivilegesToGroupRequest> {
        validate_privilege_group_members(&self.value.group_name, &self.value.privileges)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RemovePrivilegesFromGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 remove_privileges_from_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RemovePrivilegesFromGroupRequest {
    pub(crate) group_name: String,
    pub(crate) privileges: HashSet<String>,
}

impl RemovePrivilegesFromGroupRequest {
    fn empty() -> Self {
        Self {
            group_name: Default::default(),
            privileges: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RemovePrivilegesFromGroupRequestBuilder {
        RemovePrivilegesFromGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RemovePrivilegesFromGroupRequestBuilder {
        RemovePrivilegesFromGroupRequestBuilder { value: self }
    }

    /// Returns the group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    /// Returns the privileges.
    pub fn privileges(&self) -> &HashSet<String> {
        &self.privileges
    }

    pub(crate) fn into_proto(self) -> milvus::OperatePrivilegeGroupRequest {
        milvus::OperatePrivilegeGroupRequest {
            base: None,
            group_name: self.group_name,
            privileges: self
                .privileges
                .into_iter()
                .map(|name| milvus::PrivilegeEntity { name })
                .collect(),
            r#type: milvus::OperatePrivilegeGroupType::RemovePrivilegesFromGroup as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RemovePrivilegesFromGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RemovePrivilegesFromGroupRequest.
#[derive(Debug, Clone)]
pub struct RemovePrivilegesFromGroupRequestBuilder {
    value: RemovePrivilegesFromGroupRequest,
}

impl RemovePrivilegesFromGroupRequestBuilder {
    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.value.group_name = value.into();
        self
    }

    /// Sets the privileges and returns the updated value.
    pub fn privileges(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.privileges = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the privilege and returns the updated value.
    pub fn privilege(mut self, value: impl Into<String>) -> Self {
        self.value.privileges.insert(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RemovePrivilegesFromGroupRequest> {
        validate_privilege_group_members(&self.value.group_name, &self.value.privileges)?;
        Ok(self.value)
    }
}

fn validate_user_role(username: &str, role_name: &str) -> Result<()> {
    required("username", username)?;
    required("role_name", role_name)
}

fn validate_privilege(
    role_name: &str,
    database_name: &str,
    collection_name: &str,
    privilege: &str,
) -> Result<()> {
    required("role_name", role_name)?;
    required("database_name", database_name)?;
    required("collection_name", collection_name)?;
    required("privilege", privilege)
}

fn validate_privilege_group_members(group_name: &str, privileges: &HashSet<String>) -> Result<()> {
    required("group_name", group_name)?;
    if privileges.is_empty() {
        return Err(Error::validation(
            "privileges".into(),
            "must contain at least one privilege".into(),
        ));
    }
    if privileges.iter().any(String::is_empty) {
        return Err(Error::validation(
            "privileges".into(),
            "must not contain empty values".into(),
        ));
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod update_user_tests {
    use super::{CreateUserRequest, UpdatePasswordRequest, UpdateUserRequest};

    #[test]
    fn update_user_only_sets_sdk_description_fields() {
        let proto = UpdateUserRequest::builder()
            .username("alice")
            .description("data engineer")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(proto.username, "alice");
        assert_eq!(proto.description.as_deref(), Some("data engineer"));
        assert!(proto.old_password.is_empty());
        assert!(proto.new_password.is_empty());
    }

    #[test]
    fn credential_passwords_are_base64_encoded_for_rpc() {
        let create = CreateUserRequest::builder()
            .username("alice")
            .password("Test1234!")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(create.password, "VGVzdDEyMzQh");

        let update = UpdatePasswordRequest::builder()
            .username("alice")
            .old_password("old")
            .new_password("new")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(update.old_password, "b2xk");
        assert_eq!(update.new_password, "bmV3");
    }

    #[test]
    fn credential_request_debug_output_redacts_passwords() {
        let create_builder = CreateUserRequest::builder()
            .username("alice")
            .password("create-password-secret")
            .description("data engineer");
        let create_builder_debug = format!("{create_builder:?}");
        assert!(create_builder_debug.contains("alice"));
        assert!(create_builder_debug.contains("[REDACTED]"));
        assert!(!create_builder_debug.contains("create-password-secret"));
        let create_debug = format!(
            "{:?}",
            create_builder.build().expect("valid create user request")
        );
        assert!(!create_debug.contains("create-password-secret"));

        let update_builder = UpdatePasswordRequest::builder()
            .username("alice")
            .old_password("old-password-secret")
            .new_password("new-password-secret");
        let update_builder_debug = format!("{update_builder:?}");
        assert!(update_builder_debug.contains("[REDACTED]"));
        assert!(!update_builder_debug.contains("old-password-secret"));
        assert!(!update_builder_debug.contains("new-password-secret"));
        let update_debug = format!(
            "{:?}",
            update_builder
                .build()
                .expect("valid update password request")
        );
        assert!(!update_debug.contains("old-password-secret"));
        assert!(!update_debug.contains("new-password-secret"));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod describe_role_tests {
    use super::{DescribeRoleRequest, ListRolesRequest};

    #[test]
    fn describe_role_uses_database_scope_and_excludes_user_info() {
        let (role_name, grant, role) = DescribeRoleRequest::builder()
            .role_name("analyst")
            .database_name("catalog")
            .build()
            .expect("valid request")
            .into_proto();

        assert_eq!(role_name, "analyst");
        let entity = grant.entity.expect("grant selector");
        assert_eq!(entity.role.expect("selected role").name, "analyst");
        assert_eq!(entity.db_name, "catalog");
        assert!(!role.include_user_info);
        assert_eq!(role.role.expect("role selector").name, "analyst");
    }

    #[test]
    fn list_roles_has_no_selector_and_excludes_user_info() {
        let proto = ListRolesRequest::builder()
            .build()
            .expect("valid request")
            .into_proto();
        assert!(proto.role.is_none());
        assert!(!proto.include_user_info);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod dedicated_operation_tests {
    use super::{
        AddPrivilegesToGroupRequest, GrantPrivilegeRequest, GrantRoleRequest,
        RemovePrivilegesFromGroupRequest, RevokePrivilegeRequest, RevokeRoleRequest,
    };
    use crate::proto::milvus;

    #[test]
    fn role_requests_encode_fixed_operations() {
        let grant = GrantRoleRequest::builder()
            .username("alice")
            .role_name("analyst")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(grant.username, "alice");
        assert_eq!(grant.role_name, "analyst");
        assert_eq!(
            grant.r#type,
            milvus::OperateUserRoleType::AddUserToRole as i32
        );

        let revoke = RevokeRoleRequest::builder()
            .username("alice")
            .role_name("analyst")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(
            revoke.r#type,
            milvus::OperateUserRoleType::RemoveUserFromRole as i32
        );
    }

    #[test]
    fn privilege_requests_encode_fixed_operations() {
        let grant = GrantPrivilegeRequest::builder()
            .role_name("analyst")
            .database_name("catalog")
            .collection_name("books")
            .privilege("Search")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(grant.r#type, milvus::OperatePrivilegeType::Grant as i32);
        assert_eq!(grant.db_name, "catalog");
        assert_eq!(grant.collection_name, "books");

        let revoke = RevokePrivilegeRequest::builder()
            .role_name("analyst")
            .database_name("catalog")
            .collection_name("books")
            .privilege("Search")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(revoke.r#type, milvus::OperatePrivilegeType::Revoke as i32);
    }

    #[test]
    fn privilege_group_requests_encode_fixed_operations_and_deduplicate() {
        let add = AddPrivilegesToGroupRequest::builder()
            .group_name("readers")
            .privileges(["Search", "Query"])
            .privilege("Search")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(add.privileges.len(), 2);
        assert_eq!(
            add.r#type,
            milvus::OperatePrivilegeGroupType::AddPrivilegesToGroup as i32
        );

        let remove = RemovePrivilegesFromGroupRequest::builder()
            .group_name("readers")
            .privilege("Query")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(remove.privileges.len(), 1);
        assert_eq!(
            remove.r#type,
            milvus::OperatePrivilegeGroupType::RemovePrivilegesFromGroup as i32
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
    fn list_users_request_default_values() {
        assert_eq!(
            ListUsersRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::ListCredUsersRequest::default()
        );
    }

    #[test]
    fn list_users_request_populated_values() {
        let value = ListUsersRequest::builder().build().expect("valid request");
        assert_eq!(value.into_proto(), milvus::ListCredUsersRequest::default());
    }

    #[test]
    fn list_roles_request_default_values() {
        let proto = ListRolesRequest::builder()
            .build()
            .expect("valid request")
            .into_proto();
        assert!(proto.role.is_none());
        assert!(!proto.include_user_info);
    }

    #[test]
    fn list_roles_request_populated_values() {
        let proto = ListRolesRequest::builder()
            .build()
            .expect("valid request")
            .into_proto();
        assert!(proto.role.is_none());
        assert!(!proto.include_user_info);
    }

    #[test]
    fn list_privilege_groups_request_default_values() {
        assert_eq!(
            ListPrivilegeGroupsRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::ListPrivilegeGroupsRequest::default()
        );
    }

    #[test]
    fn list_privilege_groups_request_populated_values() {
        let value = ListPrivilegeGroupsRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(
            value.into_proto(),
            milvus::ListPrivilegeGroupsRequest::default()
        );
    }

    #[test]
    fn create_user_request_default_values() {
        let value = CreateUserRequest::empty();
        let expected_username: String = String::new();
        let expected_password: String = String::new();
        let expected_description: Option<String> = None;

        assert_eq!(value.username().to_owned(), expected_username);
        assert_eq!(value.password().to_owned(), expected_password);
        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn create_user_request_populated_values() {
        let username = "username-value".to_owned();
        let password = "password-value".to_owned();
        let description = "description-value".to_owned();
        let value = CreateUserRequest::builder()
            .username(username.clone())
            .password(password.clone())
            .description(description.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
        assert_eq!(value.password().to_owned(), password);
        assert_eq!(value.description().to_owned(), Some(description));
    }

    #[test]
    fn update_password_request_default_values() {
        let value = UpdatePasswordRequest::empty();
        let expected_username: String = String::new();
        let expected_old_password: String = String::new();
        let expected_new_password: String = String::new();

        assert_eq!(value.username().to_owned(), expected_username);
        assert_eq!(value.old_password().to_owned(), expected_old_password);
        assert_eq!(value.new_password().to_owned(), expected_new_password);
    }

    #[test]
    fn update_password_request_populated_values() {
        let username = "username-value".to_owned();
        let old_password = "old_password-value".to_owned();
        let new_password = "new_password-value".to_owned();
        let value = UpdatePasswordRequest::builder()
            .username(username.clone())
            .old_password(old_password.clone())
            .new_password(new_password.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
        assert_eq!(value.old_password().to_owned(), old_password);
        assert_eq!(value.new_password().to_owned(), new_password);
    }

    #[test]
    fn update_user_request_default_values() {
        let value = UpdateUserRequest::empty();
        let expected_username: String = String::new();
        let expected_description: String = String::new();

        assert_eq!(value.username().to_owned(), expected_username);
        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn update_user_request_populated_values() {
        let username = "username-value".to_owned();
        let description = "description-value".to_owned();
        let value = UpdateUserRequest::builder()
            .username(username.clone())
            .description(description.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
        assert_eq!(value.description().to_owned(), description);
    }

    #[test]
    fn drop_user_request_default_values() {
        let value = DropUserRequest::empty();
        let expected_username: String = String::new();

        assert_eq!(value.username().to_owned(), expected_username);
    }

    #[test]
    fn drop_user_request_populated_values() {
        let username = "username-value".to_owned();
        let value = DropUserRequest::builder()
            .username(username.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
    }

    #[test]
    fn create_role_request_default_values() {
        let value = CreateRoleRequest::empty();
        let expected_role_name: String = String::new();
        let expected_description: String = String::new();

        assert_eq!(value.role_name().to_owned(), expected_role_name);
        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn create_role_request_populated_values() {
        let role_name = "role_name-value".to_owned();
        let description = "description-value".to_owned();
        let value = CreateRoleRequest::builder()
            .role_name(role_name.clone())
            .description(description.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.role_name().to_owned(), role_name);
        assert_eq!(value.description().to_owned(), description);
    }

    #[test]
    fn alter_role_request_default_values() {
        let value = AlterRoleRequest::empty();
        let expected_role_name: String = String::new();
        let expected_description: String = String::new();

        assert_eq!(value.role_name().to_owned(), expected_role_name);
        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn alter_role_request_populated_values() {
        let role_name = "role_name-value".to_owned();
        let description = "description-value".to_owned();
        let value = AlterRoleRequest::builder()
            .role_name(role_name.clone())
            .description(description.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.role_name().to_owned(), role_name);
        assert_eq!(value.description().to_owned(), description);
    }

    #[test]
    fn drop_role_request_default_values() {
        let value = DropRoleRequest::empty();
        let expected_role_name: String = String::new();
        let expected_force: bool = false;

        assert_eq!(value.role_name().to_owned(), expected_role_name);
        assert_eq!(value.should_force().to_owned(), expected_force);
    }

    #[test]
    fn drop_role_request_populated_values() {
        let role_name = "role_name-value".to_owned();
        let force = true;
        let value = DropRoleRequest::builder()
            .role_name(role_name.clone())
            .force(force.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.role_name().to_owned(), role_name);
        assert_eq!(value.should_force().to_owned(), force);
    }

    #[test]
    fn grant_role_request_default_values() {
        let value = GrantRoleRequest::empty();
        let expected_username: String = String::new();
        let expected_role_name: String = String::new();

        assert_eq!(value.username().to_owned(), expected_username);
        assert_eq!(value.role_name().to_owned(), expected_role_name);
    }

    #[test]
    fn grant_role_request_populated_values() {
        let username = "username-value".to_owned();
        let role_name = "role_name-value".to_owned();
        let value = GrantRoleRequest::builder()
            .username(username.clone())
            .role_name(role_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
        assert_eq!(value.role_name().to_owned(), role_name);
    }

    #[test]
    fn revoke_role_request_default_values() {
        let value = RevokeRoleRequest::empty();
        let expected_username: String = String::new();
        let expected_role_name: String = String::new();

        assert_eq!(value.username().to_owned(), expected_username);
        assert_eq!(value.role_name().to_owned(), expected_role_name);
    }

    #[test]
    fn revoke_role_request_populated_values() {
        let username = "username-value".to_owned();
        let role_name = "role_name-value".to_owned();
        let value = RevokeRoleRequest::builder()
            .username(username.clone())
            .role_name(role_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
        assert_eq!(value.role_name().to_owned(), role_name);
    }

    #[test]
    fn describe_role_request_default_values() {
        let value = DescribeRoleRequest::empty();
        let expected_role_name: String = String::new();
        let expected_database_name: String = String::new();

        assert_eq!(value.role_name().to_owned(), expected_role_name);
        assert_eq!(value.database_name().to_owned(), expected_database_name);
    }

    #[test]
    fn describe_role_request_populated_values() {
        let role_name = "role_name-value".to_owned();
        let database_name = "database_name-value".to_owned();
        let value = DescribeRoleRequest::builder()
            .role_name(role_name.clone())
            .database_name(database_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.role_name().to_owned(), role_name);
        assert_eq!(value.database_name().to_owned(), database_name);
    }

    #[test]
    fn describe_user_request_default_values() {
        let value = DescribeUserRequest::empty();
        let expected_username: String = String::new();
        let expected_include_roles: bool = true;

        assert_eq!(value.username().to_owned(), expected_username);
        assert_eq!(
            value.should_include_roles().to_owned(),
            expected_include_roles
        );
    }

    #[test]
    fn describe_user_request_populated_values() {
        let username = "username-value".to_owned();
        let include_roles = true;
        let value = DescribeUserRequest::builder()
            .username(username.clone())
            .include_roles(include_roles.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.username().to_owned(), username);
        assert_eq!(value.should_include_roles().to_owned(), include_roles);
    }

    #[test]
    fn grant_privilege_request_default_values() {
        let value = GrantPrivilegeRequest::empty();
        let expected_role_name: String = String::new();
        let expected_database_name: String = String::new();
        let expected_collection_name: String = String::new();
        let expected_privilege: String = String::new();

        assert_eq!(value.role_name().to_owned(), expected_role_name);
        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.privilege().to_owned(), expected_privilege);
    }

    #[test]
    fn grant_privilege_request_populated_values() {
        let role_name = "role_name-value".to_owned();
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let privilege = "privilege-value".to_owned();
        let value = GrantPrivilegeRequest::builder()
            .role_name(role_name.clone())
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .privilege(privilege.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.role_name().to_owned(), role_name);
        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.privilege().to_owned(), privilege);
    }

    #[test]
    fn revoke_privilege_request_default_values() {
        let value = RevokePrivilegeRequest::empty();
        let expected_role_name: String = String::new();
        let expected_database_name: String = String::new();
        let expected_collection_name: String = String::new();
        let expected_privilege: String = String::new();

        assert_eq!(value.role_name().to_owned(), expected_role_name);
        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.privilege().to_owned(), expected_privilege);
    }

    #[test]
    fn revoke_privilege_request_populated_values() {
        let role_name = "role_name-value".to_owned();
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let privilege = "privilege-value".to_owned();
        let value = RevokePrivilegeRequest::builder()
            .role_name(role_name.clone())
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .privilege(privilege.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.role_name().to_owned(), role_name);
        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.privilege().to_owned(), privilege);
    }

    #[test]
    fn create_privilege_group_request_default_values() {
        let value = CreatePrivilegeGroupRequest::empty();
        let expected_group_name: String = String::new();

        assert_eq!(value.group_name().to_owned(), expected_group_name);
    }

    #[test]
    fn create_privilege_group_request_populated_values() {
        let group_name = "group_name-value".to_owned();
        let value = CreatePrivilegeGroupRequest::builder()
            .group_name(group_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.group_name().to_owned(), group_name);
    }

    #[test]
    fn drop_privilege_group_request_default_values() {
        let value = DropPrivilegeGroupRequest::empty();
        let expected_group_name: String = String::new();

        assert_eq!(value.group_name().to_owned(), expected_group_name);
    }

    #[test]
    fn drop_privilege_group_request_populated_values() {
        let group_name = "group_name-value".to_owned();
        let value = DropPrivilegeGroupRequest::builder()
            .group_name(group_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.group_name().to_owned(), group_name);
    }

    #[test]
    fn add_privileges_to_group_request_default_values() {
        let value = AddPrivilegesToGroupRequest::empty();
        let expected_group_name: String = String::new();
        let expected_privileges: HashSet<String> = Default::default();

        assert_eq!(value.group_name().to_owned(), expected_group_name);
        assert_eq!(value.privileges().to_owned(), expected_privileges);
    }

    #[test]
    fn add_privileges_to_group_request_populated_values() {
        let group_name = "group_name-value".to_owned();
        let privileges = HashSet::from(["privileges-value".to_owned()]);
        let value = AddPrivilegesToGroupRequest::builder()
            .group_name(group_name.clone())
            .privileges(privileges.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.group_name().to_owned(), group_name);
        assert_eq!(value.privileges().to_owned(), privileges);
    }

    #[test]
    fn remove_privileges_from_group_request_default_values() {
        let value = RemovePrivilegesFromGroupRequest::empty();
        let expected_group_name: String = String::new();
        let expected_privileges: HashSet<String> = Default::default();

        assert_eq!(value.group_name().to_owned(), expected_group_name);
        assert_eq!(value.privileges().to_owned(), expected_privileges);
    }

    #[test]
    fn remove_privileges_from_group_request_populated_values() {
        let group_name = "group_name-value".to_owned();
        let privileges = HashSet::from(["privileges-value".to_owned()]);
        let value = RemovePrivilegesFromGroupRequest::builder()
            .group_name(group_name.clone())
            .privileges(privileges.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.group_name().to_owned(), group_name);
        assert_eq!(value.privileges().to_owned(), privileges);
    }
}
