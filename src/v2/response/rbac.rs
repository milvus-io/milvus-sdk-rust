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

//! Response types returned by user, role, and privilege operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
pub use crate::v2::types::{GrantItem, PrivilegeGroupInfo, RoleDescription, UserDescription};

///////////////////////////////////////////////////////////////////////////////
// ListUsersResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_users operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListUsersResponse {
    pub(crate) usernames: Vec<String>,
}

impl ListUsersResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            usernames: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListUsersResponseBuilder {
        ListUsersResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the usernames.
    pub fn usernames(&self) -> &[String] {
        &self.usernames
    }

    pub(crate) fn from_proto(value: milvus::ListCredUsersResponse) -> Self {
        Self {
            usernames: value.usernames,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListUsersResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListUsersResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListUsersResponseBuilder {
    value: ListUsersResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListUsersResponseBuilder {
    /// Sets the usernames and returns the updated value.
    pub fn usernames(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.usernames = values.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> ListUsersResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRolesResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_roles operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListRolesResponse {
    pub(crate) role_names: Vec<String>,
}

impl ListRolesResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            role_names: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListRolesResponseBuilder {
        ListRolesResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the role names.
    pub fn role_names(&self) -> &[String] {
        &self.role_names
    }

    pub(crate) fn from_proto(value: milvus::SelectRoleResponse) -> Result<Self> {
        Ok(Self {
            role_names: value
                .results
                .into_iter()
                .map(|result| {
                    result.role.map(|role| role.name).ok_or_else(|| {
                        Error::MalformedResponse("role result contains no role".into())
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRolesResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRolesResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListRolesResponseBuilder {
    value: ListRolesResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListRolesResponseBuilder {
    /// Sets the role names and returns the updated value.
    pub fn role_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.role_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> ListRolesResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeRoleResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_role operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeRoleResponse {
    pub(crate) description: RoleDescription,
}

impl DescribeRoleResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            description: RoleDescription::new(),
        }
    }
}

impl DescribeRoleResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeRoleResponseBuilder {
        DescribeRoleResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the description.
    pub fn description(&self) -> &RoleDescription {
        &self.description
    }

    pub(crate) fn from_proto(
        requested_role_name: String,
        grants: milvus::SelectGrantResponse,
        roles: milvus::SelectRoleResponse,
    ) -> Result<Self> {
        let role = roles
            .results
            .into_iter()
            .next()
            .and_then(|result| result.role)
            .ok_or_else(|| {
                Error::MalformedResponse(format!(
                    "describe role response contains no role for {requested_role_name:?}"
                ))
            })?;
        let description = RoleDescription {
            role_name: role.name,
            description: role.description,
            grant_items: grants
                .entities
                .into_iter()
                .map(|entity| {
                    let role = entity.role.ok_or_else(|| {
                        Error::MalformedResponse("grant entity contains no role".into())
                    })?;
                    let object = entity.object.ok_or_else(|| {
                        Error::MalformedResponse("grant entity contains no object".into())
                    })?;
                    let grantor = entity.grantor.ok_or_else(|| {
                        Error::MalformedResponse("grant entity contains no grantor".into())
                    })?;
                    let grantor_name = grantor.user.map(|user| user.name).unwrap_or_default();
                    let privilege = grantor.privilege.ok_or_else(|| {
                        Error::MalformedResponse("grantor contains no privilege".into())
                    })?;
                    Ok(GrantItem {
                        object_type: object.name,
                        object_name: entity.object_name,
                        database_name: entity.db_name,
                        role_name: role.name,
                        privilege: privilege.name,
                        grantor_name,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        };
        Ok(Self { description })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeRoleResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeRoleResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeRoleResponseBuilder {
    value: DescribeRoleResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeRoleResponseBuilder {
    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: RoleDescription) -> Self {
        self.value.description = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> DescribeRoleResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeUserResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_user operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeUserResponse {
    pub(crate) users: Vec<UserDescription>,
}

impl DescribeUserResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self { users: Vec::new() }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeUserResponseBuilder {
        DescribeUserResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the users.
    pub fn users(&self) -> &[UserDescription] {
        &self.users
    }

    pub(crate) fn from_proto(value: milvus::SelectUserResponse) -> Result<Self> {
        Ok(Self {
            users: value
                .results
                .into_iter()
                .map(|v| {
                    let user = v.user.ok_or_else(|| {
                        Error::MalformedResponse("user result contains no user".into())
                    })?;
                    Ok(UserDescription {
                        username: user.name,
                        description: v.description,
                        roles: v.roles.into_iter().map(|r| r.name).collect(),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeUserResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeUserResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeUserResponseBuilder {
    value: DescribeUserResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeUserResponseBuilder {
    /// Sets the users and returns the updated value.
    pub fn users(mut self, value: Vec<UserDescription>) -> Self {
        self.value.users = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> DescribeUserResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPrivilegeGroupsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_privilege_groups operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPrivilegeGroupsResponse {
    pub(crate) groups: Vec<PrivilegeGroupInfo>,
}

impl ListPrivilegeGroupsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self { groups: Vec::new() }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListPrivilegeGroupsResponseBuilder {
        ListPrivilegeGroupsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the groups.
    pub fn groups(&self) -> &[PrivilegeGroupInfo] {
        &self.groups
    }

    pub(crate) fn from_proto(value: milvus::ListPrivilegeGroupsResponse) -> Self {
        Self {
            groups: value
                .privilege_groups
                .into_iter()
                .map(|v| PrivilegeGroupInfo {
                    group_name: v.group_name,
                    privileges: v.privileges.into_iter().map(|p| p.name).collect(),
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPrivilegeGroupsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListPrivilegeGroupsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListPrivilegeGroupsResponseBuilder {
    value: ListPrivilegeGroupsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListPrivilegeGroupsResponseBuilder {
    /// Sets the groups and returns the updated value.
    pub fn groups(mut self, value: Vec<PrivilegeGroupInfo>) -> Self {
        self.value.groups = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> ListPrivilegeGroupsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod describe_role_tests {
    use super::{DescribeRoleResponse, DescribeUserResponse, ListRolesResponse};
    use crate::proto::milvus;
    use crate::v2::error::Error;

    #[test]
    fn describe_role_combines_grants_with_role_metadata() {
        let response = DescribeRoleResponse::from_proto(
            "analyst".to_owned(),
            milvus::SelectGrantResponse {
                entities: vec![milvus::GrantEntity {
                    role: Some(milvus::RoleEntity {
                        name: "analyst".to_owned(),
                        description: String::new(),
                    }),
                    object: Some(milvus::ObjectEntity {
                        name: "Collection".to_owned(),
                    }),
                    object_name: "books".to_owned(),
                    grantor: Some(milvus::GrantorEntity {
                        user: Some(milvus::UserEntity {
                            name: "root".to_owned(),
                        }),
                        privilege: Some(milvus::PrivilegeEntity {
                            name: "Search".to_owned(),
                        }),
                    }),
                    db_name: "catalog".to_owned(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            milvus::SelectRoleResponse {
                results: vec![milvus::RoleResult {
                    role: Some(milvus::RoleEntity {
                        name: "analyst".to_owned(),
                        description: "read-only role".to_owned(),
                    }),
                    users: Vec::new(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("valid describe role response");

        let description = response.description();
        assert_eq!(description.get_role_name().to_owned(), "analyst");
        assert_eq!(description.get_description().to_owned(), "read-only role");
        let grant = &description.get_grant_items()[0];
        assert_eq!(grant.get_object_type().to_owned(), "Collection");
        assert_eq!(grant.get_object_name().to_owned(), "books");
        assert_eq!(grant.get_database_name().to_owned(), "catalog");
        assert_eq!(grant.get_role_name().to_owned(), "analyst");
        assert_eq!(grant.get_privilege().to_owned(), "Search");
        assert_eq!(grant.get_grantor_name().to_owned(), "root");
    }

    #[test]
    fn list_roles_extracts_role_names_without_users() {
        let response = ListRolesResponse::from_proto(milvus::SelectRoleResponse {
            results: vec![
                milvus::RoleResult {
                    role: Some(milvus::RoleEntity {
                        name: "analyst".to_owned(),
                        description: "read-only".to_owned(),
                    }),
                    users: vec![milvus::UserEntity {
                        name: "alice".to_owned(),
                    }],
                    ..Default::default()
                },
                milvus::RoleResult {
                    role: Some(milvus::RoleEntity {
                        name: "writer".to_owned(),
                        description: String::new(),
                    }),
                    users: Vec::new(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .expect("valid list roles response");

        assert_eq!(response.role_names().to_owned(), ["analyst", "writer"]);
    }

    #[test]
    fn rbac_responses_reject_missing_nested_entities() {
        let list_error = ListRolesResponse::from_proto(milvus::SelectRoleResponse {
            results: vec![milvus::RoleResult::default()],
            ..Default::default()
        })
        .unwrap_err();
        assert!(matches!(list_error, Error::MalformedResponse(_)));

        let user_error = DescribeUserResponse::from_proto(milvus::SelectUserResponse {
            results: vec![milvus::UserResult::default()],
            ..Default::default()
        })
        .unwrap_err();
        assert!(matches!(user_error, Error::MalformedResponse(_)));

        let role_error = DescribeRoleResponse::from_proto(
            "analyst".into(),
            milvus::SelectGrantResponse::default(),
            milvus::SelectRoleResponse::default(),
        )
        .unwrap_err();
        assert!(matches!(role_error, Error::MalformedResponse(_)));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn list_users_response_default_values() {
        let value = ListUsersResponse::builder().build();
        let expected_usernames: Vec<String> = Default::default();

        assert_eq!(value.usernames().to_owned(), expected_usernames);
    }

    #[test]
    fn list_users_response_populated_values() {
        let usernames = vec!["usernames-value".to_owned()];
        let value = ListUsersResponse::builder()
            .usernames(usernames.clone())
            .build();

        assert_eq!(value.usernames().to_owned(), usernames);
    }

    #[test]
    fn list_roles_response_default_values() {
        let value = ListRolesResponse::builder().build();
        let expected_role_names: Vec<String> = Default::default();

        assert_eq!(value.role_names().to_owned(), expected_role_names);
    }

    #[test]
    fn list_roles_response_populated_values() {
        let role_names = vec!["role_names-value".to_owned()];
        let value = ListRolesResponse::builder()
            .role_names(role_names.clone())
            .build();

        assert_eq!(value.role_names().to_owned(), role_names);
    }

    #[test]
    fn describe_role_response_default_values() {
        let value = DescribeRoleResponse::builder().build();
        let expected_description = RoleDescription::new();

        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn describe_role_response_populated_values() {
        let description = RoleDescription::new();
        let value = DescribeRoleResponse::builder()
            .description(description.clone())
            .build();

        assert_eq!(value.description().to_owned(), description);
    }

    #[test]
    fn describe_user_response_default_values() {
        let value = DescribeUserResponse::builder().build();
        let expected_users: Vec<UserDescription> = Default::default();

        assert_eq!(value.users().to_owned(), expected_users);
    }

    #[test]
    fn describe_user_response_populated_values() {
        let users = vec![UserDescription::new()];
        let value = DescribeUserResponse::builder().users(users.clone()).build();

        assert_eq!(value.users().to_owned(), users);
    }

    #[test]
    fn list_privilege_groups_response_default_values() {
        let value = ListPrivilegeGroupsResponse::builder().build();
        let expected_groups: Vec<PrivilegeGroupInfo> = Default::default();

        assert_eq!(value.groups().to_owned(), expected_groups);
    }

    #[test]
    fn list_privilege_groups_response_populated_values() {
        let groups = vec![PrivilegeGroupInfo::new()];
        let value = ListPrivilegeGroupsResponse::builder()
            .groups(groups.clone())
            .build();

        assert_eq!(value.groups().to_owned(), groups);
    }
}
