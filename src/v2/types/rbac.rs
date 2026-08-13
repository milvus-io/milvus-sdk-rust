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

//! User, role, grant, and privilege-group domain types.

///////////////////////////////////////////////////////////////////////////////
// GrantItem
///////////////////////////////////////////////////////////////////////////////
/// A privilege granted to a role.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GrantItem {
    pub(crate) object_type: String,
    pub(crate) object_name: String,
    pub(crate) database_name: String,
    pub(crate) role_name: String,
    pub(crate) privilege: String,
    pub(crate) grantor_name: String,
}

impl GrantItem {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            object_type: String::new(),
            object_name: String::new(),
            database_name: String::new(),
            role_name: String::new(),
            privilege: String::new(),
            grantor_name: String::new(),
        }
    }

    /// Sets the object type and returns the updated value.
    pub fn object_type(mut self, value: impl Into<String>) -> Self {
        self.object_type = value.into();
        self
    }

    /// Sets the object type and returns this value for further mutation.
    pub fn set_object_type(&mut self, value: impl Into<String>) -> &mut Self {
        self.object_type = value.into();
        self
    }

    /// Returns the configured object type.
    pub fn get_object_type(&self) -> &str {
        &self.object_type
    }

    /// Sets the object name and returns the updated value.
    pub fn object_name(mut self, value: impl Into<String>) -> Self {
        self.object_name = value.into();
        self
    }

    /// Sets the object name and returns this value for further mutation.
    pub fn set_object_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.object_name = value.into();
        self
    }

    /// Returns the configured object name.
    pub fn get_object_name(&self) -> &str {
        &self.object_name
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

    /// Returns the configured database name.
    pub fn get_database_name(&self) -> &str {
        &self.database_name
    }

    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.role_name = value.into();
        self
    }

    /// Sets the role name and returns this value for further mutation.
    pub fn set_role_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.role_name = value.into();
        self
    }

    /// Returns the configured role name.
    pub fn get_role_name(&self) -> &str {
        &self.role_name
    }

    /// Sets the privilege and returns the updated value.
    pub fn privilege(mut self, value: impl Into<String>) -> Self {
        self.privilege = value.into();
        self
    }

    /// Sets the privilege and returns this value for further mutation.
    pub fn set_privilege(&mut self, value: impl Into<String>) -> &mut Self {
        self.privilege = value.into();
        self
    }

    /// Returns the configured privilege.
    pub fn get_privilege(&self) -> &str {
        &self.privilege
    }

    /// Sets the grantor name and returns the updated value.
    pub fn grantor_name(mut self, value: impl Into<String>) -> Self {
        self.grantor_name = value.into();
        self
    }

    /// Sets the grantor name and returns this value for further mutation.
    pub fn set_grantor_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.grantor_name = value.into();
        self
    }

    /// Returns the configured grantor name.
    pub fn get_grantor_name(&self) -> &str {
        &self.grantor_name
    }
}

///////////////////////////////////////////////////////////////////////////////
// RoleDescription
///////////////////////////////////////////////////////////////////////////////
/// Role metadata and its granted privileges.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RoleDescription {
    pub(crate) role_name: String,
    pub(crate) description: String,
    pub(crate) grant_items: Vec<GrantItem>,
}

impl RoleDescription {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            role_name: String::new(),
            description: String::new(),
            grant_items: Vec::new(),
        }
    }

    /// Sets the role name and returns the updated value.
    pub fn role_name(mut self, value: impl Into<String>) -> Self {
        self.role_name = value.into();
        self
    }

    /// Sets the role name and returns this value for further mutation.
    pub fn set_role_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.role_name = value.into();
        self
    }

    /// Returns the configured role name.
    pub fn get_role_name(&self) -> &str {
        &self.role_name
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the grant items and returns the updated value.
    pub fn grant_items(mut self, value: Vec<GrantItem>) -> Self {
        self.grant_items = value;
        self
    }

    /// Sets the grant items and returns this value for further mutation.
    pub fn set_grant_items(&mut self, value: Vec<GrantItem>) -> &mut Self {
        self.grant_items = value;
        self
    }

    /// Returns the configured grant items.
    pub fn get_grant_items(&self) -> &[GrantItem] {
        &self.grant_items
    }

    /// Adds one add grant item to the existing values.
    pub fn add_grant_item(mut self, value: GrantItem) -> Self {
        self.grant_items.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// UserDescription
///////////////////////////////////////////////////////////////////////////////
/// User metadata and assigned roles.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UserDescription {
    pub(crate) username: String,
    pub(crate) description: String,
    pub(crate) roles: Vec<String>,
}

impl UserDescription {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            username: String::new(),
            description: String::new(),
            roles: Vec::new(),
        }
    }

    /// Sets the username and returns the updated value.
    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.username = value.into();
        self
    }

    /// Sets the username and returns this value for further mutation.
    pub fn set_username(&mut self, value: impl Into<String>) -> &mut Self {
        self.username = value.into();
        self
    }

    /// Returns the configured username.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the roles and returns the updated value.
    pub fn roles(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.roles = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the roles and returns this value for further mutation.
    pub fn set_roles(&mut self, values: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.roles = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured roles.
    pub fn get_roles(&self) -> &[String] {
        &self.roles
    }

    /// Adds one add role to the existing values.
    pub fn add_role(mut self, value: impl Into<String>) -> Self {
        self.roles.push(value.into());
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// PrivilegeGroupInfo
///////////////////////////////////////////////////////////////////////////////
/// A named group of Milvus privileges.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrivilegeGroupInfo {
    pub(crate) group_name: String,
    pub(crate) privileges: Vec<String>,
}

impl PrivilegeGroupInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            group_name: String::new(),
            privileges: Vec::new(),
        }
    }

    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.group_name = value.into();
        self
    }

    /// Sets the group name and returns this value for further mutation.
    pub fn set_group_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.group_name = value.into();
        self
    }

    /// Returns the configured group name.
    pub fn get_group_name(&self) -> &str {
        &self.group_name
    }

    /// Sets the privileges and returns the updated value.
    pub fn privileges(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.privileges = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the privileges and returns this value for further mutation.
    pub fn set_privileges(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.privileges = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured privileges.
    pub fn get_privileges(&self) -> &[String] {
        &self.privileges
    }

    /// Adds one add privilege to the existing values.
    pub fn add_privilege(mut self, value: impl Into<String>) -> Self {
        self.privileges.push(value.into());
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn grant_item_default_values() {
        let value = GrantItem::new();
        let expected_object_type: String = String::new();
        let expected_object_name: String = String::new();
        let expected_database_name: String = String::new();
        let expected_role_name: String = String::new();
        let expected_privilege: String = String::new();
        let expected_grantor_name: String = String::new();

        assert_eq!(value.get_object_type().to_owned(), expected_object_type);
        assert_eq!(value.get_object_name().to_owned(), expected_object_name);
        assert_eq!(value.get_database_name().to_owned(), expected_database_name);
        assert_eq!(value.get_role_name().to_owned(), expected_role_name);
        assert_eq!(value.get_privilege().to_owned(), expected_privilege);
        assert_eq!(value.get_grantor_name().to_owned(), expected_grantor_name);
    }

    #[test]
    fn grant_item_populated_values() {
        let object_type = "object_type-value".to_owned();
        let object_name = "object_name-value".to_owned();
        let database_name = "database_name-value".to_owned();
        let role_name = "role_name-value".to_owned();
        let privilege = "privilege-value".to_owned();
        let grantor_name = "grantor_name-value".to_owned();
        let value = GrantItem::new()
            .object_type(object_type.clone())
            .object_name(object_name.clone())
            .database_name(database_name.clone())
            .role_name(role_name.clone())
            .privilege(privilege.clone())
            .grantor_name(grantor_name.clone());

        assert_eq!(value.get_object_type().to_owned(), object_type);
        assert_eq!(value.get_object_name().to_owned(), object_name);
        assert_eq!(value.get_database_name().to_owned(), database_name);
        assert_eq!(value.get_role_name().to_owned(), role_name);
        assert_eq!(value.get_privilege().to_owned(), privilege);
        assert_eq!(value.get_grantor_name().to_owned(), grantor_name);
    }

    #[test]
    fn role_description_default_values() {
        let value = RoleDescription::new();
        let expected_role_name: String = String::new();
        let expected_description: String = String::new();
        let expected_grant_items: Vec<GrantItem> = Default::default();

        assert_eq!(value.get_role_name().to_owned(), expected_role_name);
        assert_eq!(value.get_description().to_owned(), expected_description);
        assert_eq!(value.get_grant_items().to_owned(), expected_grant_items);
    }

    #[test]
    fn role_description_populated_values() {
        let role_name = "role_name-value".to_owned();
        let description = "description-value".to_owned();
        let grant_items = vec![GrantItem::new()];
        let value = RoleDescription::new()
            .role_name(role_name.clone())
            .description(description.clone())
            .grant_items(grant_items.clone());

        assert_eq!(value.get_role_name().to_owned(), role_name);
        assert_eq!(value.get_description().to_owned(), description);
        assert_eq!(value.get_grant_items().to_owned(), grant_items);
    }

    #[test]
    fn user_description_default_values() {
        let value = UserDescription::new();
        let expected_username: String = String::new();
        let expected_description: String = String::new();
        let expected_roles: Vec<String> = Default::default();

        assert_eq!(value.get_username().to_owned(), expected_username);
        assert_eq!(value.get_description().to_owned(), expected_description);
        assert_eq!(value.get_roles().to_owned(), expected_roles);
    }

    #[test]
    fn user_description_populated_values() {
        let username = "username-value".to_owned();
        let description = "description-value".to_owned();
        let roles = vec!["roles-value".to_owned()];
        let value = UserDescription::new()
            .username(username.clone())
            .description(description.clone())
            .roles(roles.clone());

        assert_eq!(value.get_username().to_owned(), username);
        assert_eq!(value.get_description().to_owned(), description);
        assert_eq!(value.get_roles().to_owned(), roles);
    }

    #[test]
    fn privilege_group_info_default_values() {
        let value = PrivilegeGroupInfo::new();
        let expected_group_name: String = String::new();
        let expected_privileges: Vec<String> = Default::default();

        assert_eq!(value.get_group_name().to_owned(), expected_group_name);
        assert_eq!(value.get_privileges().to_owned(), expected_privileges);
    }

    #[test]
    fn privilege_group_info_populated_values() {
        let group_name = "group_name-value".to_owned();
        let privileges = vec!["privileges-value".to_owned()];
        let value = PrivilegeGroupInfo::new()
            .group_name(group_name.clone())
            .privileges(privileges.clone());

        assert_eq!(value.get_group_name().to_owned(), group_name);
        assert_eq!(value.get_privileges().to_owned(), privileges);
    }
}
