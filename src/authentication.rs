//! # Milvus Authentication Module
//!
//! This module provides comprehensive authentication and authorization functionality for the Milvus client.
//! It includes user management, role management, privilege management, and privilege group operations.
//!
//! ## Features
//!
//! - **User Management**: Create, delete, list, and describe users
//! - **Role Management**: Create, drop, list, and describe roles
//! - **Privilege Management**: Grant and revoke privileges to/from roles
//! - **Privilege Groups**: Create, manage, and operate on privilege groups
//! - **Password Management**: Update user passwords securely
//!
//! ## Security Features
//!
//! - All passwords are Base64 encoded before transmission
//! - Supports both v1 and v2 privilege management APIs
//! - Comprehensive error handling with proper status checking
//!
//! ## Usage Example
//!
//! ```rust
//! use milvus_sdk::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client
//!     let client = Client::new("http://localhost:19530").await?;
//!     
//!     // Create a user
//!     client.create_user("testuser", "password123").await?;
//!     
//!     // Create a role
//!     client.create_role("testrole").await?;
//!     
//!     // Grant role to user
//!     client.grant_role("testuser", "testrole").await?;
//!     
//!     // Grant privilege to role
//!     client.grant_privilege_v2("testrole", "Load", "my_collection", None).await?;
//!     
//!     Ok(())
//! }
//! ```

use crate::client::Client;
use crate::error::Result;
use crate::proto;
use crate::proto::common::{MsgBase, MsgType};
use crate::proto::milvus::DeleteCredentialRequest;
use crate::utils::status_to_result;
use base64::engine::general_purpose;
use base64::Engine;
use serde_json;
use std::collections::HashMap;

impl Client {
    /// Creates a new user with the specified username and password.
    ///
    /// This method creates a user account in the Milvus system. The password is Base64 encoded
    /// before being sent to the server for security.
    ///
    /// # Arguments
    ///
    /// * `user_name` - The username for the new user account
    /// * `password` - The password for the new user account (will be Base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the user already exists
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.create_user("john_doe", "secure_password123").await?;
    /// ```
    pub async fn create_user<S: Into<String>>(&self, user_name: S, password: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .create_credential(proto::milvus::CreateCredentialRequest {
                base: Some(MsgBase::new(MsgType::CreateCredential)),
                username: user_name.into(),
                password: general_purpose::STANDARD.encode(password.into().as_bytes()),
                created_utc_timestamps: 0, // Server will set the actual timestamp
                modified_utc_timestamps: 0, // Server will set the actual timestamp
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Deletes a user from the system.
    ///
    /// This method permanently removes a user account from the Milvus system.
    /// All roles and privileges associated with the user will also be removed.
    ///
    /// # Arguments
    ///
    /// * `user_name` - The username of the user to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the user doesn't exist
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.drop_user("john_doe").await?;
    /// ```
    pub async fn drop_user<S: Into<String>>(&self, user_name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .delete_credential(DeleteCredentialRequest {
                base: Some(MsgBase::new(MsgType::DeleteCredential)),
                username: user_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Lists all users in the system.
    ///
    /// This method retrieves a list of all user accounts in the Milvus system.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` containing all usernames, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// let users = client.list_users().await?;
    /// println!("Users: {:?}", users);
    /// ```
    pub async fn list_users(&self) -> Result<Vec<String>> {
        let res = self
            .client
            .clone()
            .list_cred_users(crate::proto::milvus::ListCredUsersRequest {
                base: Some(MsgBase::new(MsgType::ListCredUsernames)),
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res.usernames)
    }

    /// Describes a specific user and returns their role information.
    ///
    /// This method retrieves detailed information about a user, including all roles
    /// assigned to that user.
    ///
    /// # Arguments
    ///
    /// * `user_name` - The username of the user to describe
    ///
    /// # Returns
    ///
    /// Returns a `HashMap<String, Vec<String>>` where:
    /// - Keys are usernames
    /// - Values are lists of role names assigned to each user
    ///
    /// # Errors
    ///
    /// - Returns an error if the user doesn't exist
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// let user_info = client.describe_user("john_doe").await?;
    /// println!("User roles: {:?}", user_info);
    /// ```
    pub async fn describe_user<S: Into<String>>(
        &self,
        user_name: S,
    ) -> Result<HashMap<String, Vec<String>>> {
        let res = self
            .client
            .clone()
            .select_user(proto::milvus::SelectUserRequest {
                base: Some(MsgBase::new(MsgType::SelectUser)),
                user: Some(proto::milvus::UserEntity {
                    name: user_name.into(),
                }),
                include_role_info: true, // Include role information in the response
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        let user_info = res.results;
        let mut result = HashMap::new();
        for user in user_info {
            result.insert(
                user.user.unwrap().name,
                user.roles.iter().map(|r| r.name.clone()).collect(),
            );
        }
        Ok(result)
    }

    /// Creates a new role in the system.
    ///
    /// This method creates a role that can be assigned to users and granted privileges.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to create
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the role already exists
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.create_role("admin").await?;
    /// ```
    pub async fn create_role<S: Into<String>>(&self, role_name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .create_role(proto::milvus::CreateRoleRequest {
                base: Some(MsgBase::new(MsgType::CreateRole)),
                entity: Some(proto::milvus::RoleEntity {
                    name: role_name.into(),
                }),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Deletes a role from the system.
    ///
    /// This method permanently removes a role from the Milvus system.
    /// If `force_drop` is true, the role will be dropped even if it has users assigned to it.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to delete
    /// * `force_drop` - If true, force drop the role even if users are assigned to it
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the role has users assigned and `force_drop` is false
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.drop_role("old_role", false).await?;
    /// ```
    pub async fn drop_role<S: Into<String>>(&self, role_name: S, force_drop: bool) -> Result<()> {
        let res = self
            .client
            .clone()
            .drop_role(proto::milvus::DropRoleRequest {
                base: Some(MsgBase::new(MsgType::DropRole)),
                role_name: role_name.into(),
                force_drop,
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Grants a role to a user.
    ///
    /// This method assigns a role to a user, giving them all privileges associated with that role.
    ///
    /// # Arguments
    ///
    /// * `user_name` - The username of the user to grant the role to
    /// * `role_name` - The name of the role to grant
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the user doesn't exist
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the user already has the role
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.grant_role("john_doe", "admin").await?;
    /// ```
    pub async fn grant_role<S: Into<String>>(&self, user_name: S, role_name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_user_role(proto::milvus::OperateUserRoleRequest {
                base: Some(MsgBase::new(MsgType::OperateUserRole)),
                r#type: proto::milvus::OperateUserRoleType::AddUserToRole as i32,
                username: user_name.into(),
                role_name: role_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Lists all roles in the system.
    ///
    /// This method retrieves a list of all roles in the Milvus system.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` containing all role names, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// let roles = client.list_roles().await?;
    /// println!("Roles: {:?}", roles);
    /// ```
    pub async fn list_roles(&self) -> Result<Vec<String>> {
        let res = self
            .client
            .clone()
            .select_role(proto::milvus::SelectRoleRequest {
                base: Some(MsgBase::new(MsgType::SelectRole)),
                role: None,               // None means select all roles
                include_user_info: false, // Don't include user information
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res
            .results
            .iter()
            .map(|r| r.role.as_ref().unwrap().name.clone())
            .collect())
    }

    /// Describe a role and return its privileges
    ///
    /// This method returns information about a specific role including all its privileges.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to describe
    /// * `timeout` - Optional timeout in seconds
    ///
    /// # Returns
    ///
    /// Returns a `HashMap<String, serde_json::Value>` containing:
    /// - `"role"`: The role name
    /// - `"privileges"`: A list of privilege objects, each containing:
    ///   - `"object_type"`: The type of object (e.g., "Collection", "Global")
    ///   - `"object_name"`: The name of the object (e.g., "collection_name", "*")
    ///   - `"db_name"`: The database name (if applicable)
    ///   - `"role_name"`: The role name
    ///   - `"privilege"`: The privilege name (e.g., "Load", "CreateCollection")
    ///   - `"grantor_name"`: The name of the user who granted the privilege
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// let client = Client::new("http://localhost:19530").await?;
    /// let role_info = client.describe_role("admin", None).await?;
    ///
    /// println!("Role: {}", role_info["role"]);
    /// if let Some(privileges) = role_info.get("privileges") {
    ///     if let Some(privileges_array) = privileges.as_array() {
    ///         for privilege in privileges_array {
    ///             println!("Privilege: {}", privilege["privilege"]);
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn describe_role(
        &self,
        role_name: impl Into<String>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let role_name = role_name.into();

        // Create the SelectGrantRequest to query all privileges for this role
        let request = crate::proto::milvus::SelectGrantRequest {
            base: Some(crate::proto::common::MsgBase::new(
                crate::proto::common::MsgType::SelectGrant,
            )),
            entity: Some(crate::proto::milvus::GrantEntity {
                role: Some(crate::proto::milvus::RoleEntity {
                    name: role_name.clone(),
                }),
                object: None,                // None means get all objects for this role
                object_name: "".to_string(), // Empty string means get all object names
                db_name: "".to_string(),     // Empty string for default database
                grantor: None,
            }),
        };

        // Call the SelectGrant RPC to get privilege information
        let response = self
            .client
            .clone()
            .select_grant(request)
            .await?
            .into_inner();

        // Check if the operation was successful
        status_to_result(&response.status)?;

        // Parse the response and build the result
        let mut result = HashMap::new();
        result.insert("role".to_string(), serde_json::Value::String(role_name));

        let mut privileges = Vec::new();
        for entity in response.entities {
            // Create a JSON object for each privilege
            let privilege_obj = serde_json::json!({
                "object_type": entity.object.as_ref().map(|o| o.name.clone()).unwrap_or_default(),
                "object_name": entity.object_name,
                "db_name": entity.db_name,
                "role_name": entity.role.as_ref().map(|r| r.name.clone()).unwrap_or_default(),
                "privilege": entity.grantor.as_ref()
                    .and_then(|g| g.privilege.as_ref())
                    .map(|p| p.name.clone())
                    .unwrap_or_default(),
                "grantor_name": entity.grantor.as_ref()
                    .and_then(|g| g.user.as_ref())
                    .map(|u| u.name.clone())
                    .unwrap_or_default(),
            });
            privileges.push(privilege_obj);
        }

        result.insert(
            "privileges".to_string(),
            serde_json::Value::Array(privileges),
        );

        Ok(result)
    }

    /// Revokes a role from a user.
    ///
    /// This method removes a role from a user, revoking all privileges associated with that role.
    ///
    /// # Arguments
    ///
    /// * `user_name` - The username of the user to revoke the role from
    /// * `role_name` - The name of the role to revoke
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the user doesn't exist
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the user doesn't have the role
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.revoke_role("john_doe", "admin").await?;
    /// ```
    pub async fn revoke_role<S: Into<String>>(&self, user_name: S, role_name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_user_role(proto::milvus::OperateUserRoleRequest {
                base: Some(MsgBase::new(MsgType::OperateUserRole)),
                r#type: proto::milvus::OperateUserRoleType::RemoveUserFromRole as i32,
                username: user_name.into(),
                role_name: role_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Creates a new privilege group.
    ///
    /// Privilege groups allow you to group related privileges together for easier management.
    ///
    /// # Arguments
    ///
    /// * `group_name` - The name of the privilege group to create
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the privilege group already exists
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.create_privilege_group("read_only").await?;
    /// ```
    pub async fn create_privilege_group<S: Into<String>>(&self, group_name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .create_privilege_group(proto::milvus::CreatePrivilegeGroupRequest {
                base: Some(MsgBase::new(MsgType::CreatePrivilegeGroup)),
                group_name: group_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Adds privileges to a privilege group.
    ///
    /// This method adds one or more privileges to an existing privilege group.
    ///
    /// # Arguments
    ///
    /// * `group_name` - The name of the privilege group
    /// * `privileges` - A vector of privilege names to add to the group
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the privilege group doesn't exist
    /// - Returns an error if any of the privileges don't exist
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// let privileges = vec!["Load".to_string(), "Get".to_string()];
    /// client.add_privilege_to_group("read_only", privileges).await?;
    /// ```
    pub async fn add_privilege_to_group<S: Into<String>>(
        &self,
        group_name: S,
        privileges: Vec<String>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_privilege_group(proto::milvus::OperatePrivilegeGroupRequest {
                base: Some(MsgBase::new(MsgType::OperatePrivilegeGroup)),
                r#type: proto::milvus::OperatePrivilegeGroupType::AddPrivilegesToGroup as i32,
                group_name: group_name.into(),
                privileges: privileges
                    .iter()
                    .map(|p| proto::milvus::PrivilegeEntity { name: p.clone() })
                    .collect(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Lists all privilege groups and their associated privileges.
    ///
    /// This method retrieves information about all privilege groups in the system.
    ///
    /// # Returns
    ///
    /// Returns a `HashMap<String, Vec<String>>` where:
    /// - Keys are privilege group names
    /// - Values are lists of privilege names in each group
    ///
    /// # Errors
    ///
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// let groups = client.list_privilege_groups().await?;
    /// for (group_name, privileges) in groups {
    ///     println!("Group: {}, Privileges: {:?}", group_name, privileges);
    /// }
    /// ```
    pub async fn list_privilege_groups(&self) -> Result<HashMap<String, Vec<String>>> {
        let res = self
            .client
            .clone()
            .list_privilege_groups(proto::milvus::ListPrivilegeGroupsRequest {
                base: Some(MsgBase::new(MsgType::ListPrivilegeGroups)),
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        let mut result = HashMap::new();
        for group in res.privilege_groups {
            result.insert(
                group.group_name,
                group.privileges.iter().map(|p| p.name.clone()).collect(),
            );
        }
        Ok(result)
    }

    /// Deletes a privilege group.
    ///
    /// This method permanently removes a privilege group from the system.
    ///
    /// # Arguments
    ///
    /// * `group_name` - The name of the privilege group to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the privilege group doesn't exist
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.drop_privilege_group("old_group").await?;
    /// ```
    pub async fn drop_privilege_group<S: Into<String>>(&self, group_name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .drop_privilege_group(proto::milvus::DropPrivilegeGroupRequest {
                base: Some(MsgBase::new(MsgType::DropPrivilegeGroup)),
                group_name: group_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Revokes privileges from a role using the v1 API.
    ///
    /// This method removes specific privileges from a role. This is the legacy v1 API
    /// that requires specifying object type and object name.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to revoke privileges from
    /// * `object_type` - The type of object (e.g., "Collection", "Global")
    /// * `privilege` - The name of the privilege to revoke
    /// * `object_name` - The name of the object
    /// * `db_name` - Optional database name (use None for default database)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the privilege doesn't exist
    /// - Returns an error if the role doesn't have the privilege
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.revoke_privilege("admin", "Collection", "Load", "my_collection", None).await?;
    /// ```
    pub async fn revoke_privilege<S: Into<String>>(
        &self,
        role_name: S,
        object_type: S,
        privilege: S,
        object_name: S,
        db_name: Option<S>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_privilege(proto::milvus::OperatePrivilegeRequest {
                base: Some(MsgBase::new(MsgType::OperatePrivilege)),
                entity: Some(proto::milvus::GrantEntity {
                    role: Some(proto::milvus::RoleEntity {
                        name: role_name.into(),
                    }),
                    object: Some(proto::milvus::ObjectEntity {
                        name: object_type.into(),
                    }),
                    object_name: object_name.into(),
                    db_name: db_name.map(|d| d.into()).unwrap_or_default(),
                    grantor: Some(proto::milvus::GrantorEntity {
                        user: None,
                        privilege: Some(proto::milvus::PrivilegeEntity {
                            name: privilege.into(),
                        }),
                    }),
                }),
                r#type: proto::milvus::OperatePrivilegeType::Revoke as i32,
                version: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Revokes privileges from a role using the v2 API.
    ///
    /// This method removes specific privileges from a role. This is the newer v2 API
    /// that is collection-focused and simpler to use.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to revoke privileges from
    /// * `privilege` - The name of the privilege to revoke
    /// * `collection_name` - The name of the collection
    /// * `db_name` - Optional database name (use None for default database)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the privilege doesn't exist
    /// - Returns an error if the role doesn't have the privilege
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.revoke_privilege_v2("admin", "Load", "my_collection", None).await?;
    /// ```
    pub async fn revoke_privilege_v2<S: Into<String>>(
        &self,
        role_name: S,
        privilege: S,
        collection_name: S,
        db_name: Option<S>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_privilege_v2(proto::milvus::OperatePrivilegeV2Request {
                base: Some(MsgBase::new(MsgType::OperatePrivilegeV2)),
                role: Some(proto::milvus::RoleEntity {
                    name: role_name.into(),
                }),
                collection_name: collection_name.into(),
                db_name: db_name.map(|d| d.into()).unwrap_or_default(),
                grantor: Some(proto::milvus::GrantorEntity {
                    user: None,
                    privilege: Some(proto::milvus::PrivilegeEntity {
                        name: privilege.into(),
                    }),
                }),
                r#type: proto::milvus::OperatePrivilegeType::Revoke as i32,
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Removes privileges from a privilege group.
    ///
    /// This method removes one or more privileges from an existing privilege group.
    ///
    /// # Arguments
    ///
    /// * `group_name` - The name of the privilege group
    /// * `privileges` - A vector of privilege names to remove from the group
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the privilege group doesn't exist
    /// - Returns an error if any of the privileges don't exist in the group
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// let privileges = vec!["Load".to_string()];
    /// client.revoke_privilege_from_group("read_only", privileges).await?;
    /// ```
    pub async fn revoke_privilege_from_group<S: Into<String>>(
        &self,
        group_name: S,
        privileges: Vec<String>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_privilege_group(proto::milvus::OperatePrivilegeGroupRequest {
                base: Some(MsgBase::new(MsgType::OperatePrivilegeGroup)),
                r#type: proto::milvus::OperatePrivilegeGroupType::RemovePrivilegesFromGroup as i32,
                group_name: group_name.into(),
                privileges: privileges
                    .iter()
                    .map(|p| proto::milvus::PrivilegeEntity { name: p.clone() })
                    .collect(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Grants privileges to a role using the v1 API.
    ///
    /// This method grants specific privileges to a role. This is the legacy v1 API
    /// that requires specifying object type and object name.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to grant privileges to
    /// * `privilege` - The name of the privilege to grant
    /// * `object_type` - The type of object (e.g., "Collection", "Global")
    /// * `object_name` - The name of the object
    /// * `db_name` - Optional database name (use None for default database)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the privilege doesn't exist
    /// - Returns an error if the role already has the privilege
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.grant_privilege("admin", "Load", "Collection", "my_collection", None).await?;
    /// ```
    pub async fn grant_privilege<S: Into<String>>(
        &self,
        role_name: S,
        privilege: S,
        object_type: S,
        object_name: S,
        db_name: Option<S>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_privilege(proto::milvus::OperatePrivilegeRequest {
                base: Some(MsgBase::new(MsgType::OperatePrivilege)),
                entity: Some(proto::milvus::GrantEntity {
                    role: Some(proto::milvus::RoleEntity {
                        name: role_name.into(),
                    }),
                    object: Some(proto::milvus::ObjectEntity {
                        name: object_type.into(),
                    }),
                    object_name: object_name.into(),
                    db_name: db_name.map(|d| d.into()).unwrap_or_default(),
                    grantor: Some(proto::milvus::GrantorEntity {
                        user: None,
                        privilege: Some(proto::milvus::PrivilegeEntity {
                            name: privilege.into(),
                        }),
                    }),
                }),
                r#type: proto::milvus::OperatePrivilegeType::Grant as i32,
                version: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Grants privileges to a role using the v2 API.
    ///
    /// This method grants specific privileges to a role. This is the newer v2 API
    /// that is collection-focused and simpler to use.
    ///
    /// # Arguments
    ///
    /// * `role_name` - The name of the role to grant privileges to
    /// * `privilege` - The name of the privilege to grant
    /// * `collection_name` - The name of the collection
    /// * `db_name` - Optional database name (use None for default database)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the role doesn't exist
    /// - Returns an error if the privilege doesn't exist
    /// - Returns an error if the role already has the privilege
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.grant_privilege_v2("admin", "Load", "my_collection", None).await?;
    /// ```
    pub async fn grant_privilege_v2<S: Into<String>>(
        &self,
        role_name: S,
        privilege: S,
        collection_name: S,
        db_name: Option<S>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .operate_privilege_v2(proto::milvus::OperatePrivilegeV2Request {
                base: Some(MsgBase::new(MsgType::OperatePrivilegeV2)),
                role: Some(proto::milvus::RoleEntity {
                    name: role_name.into(),
                }),
                grantor: Some(proto::milvus::GrantorEntity {
                    user: None,
                    privilege: Some(proto::milvus::PrivilegeEntity {
                        name: privilege.into(),
                    }),
                }),
                r#type: proto::milvus::OperatePrivilegeType::Grant as i32,
                db_name: db_name.map(|d| d.into()).unwrap_or_default(),
                collection_name: collection_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Updates a user's password.
    ///
    /// This method allows a user to change their password. Both the old and new passwords
    /// are Base64 encoded before being sent to the server for security.
    ///
    /// # Arguments
    ///
    /// * `user_name` - The username of the user whose password to update
    /// * `old_password` - The current password (will be Base64 encoded)
    /// * `new_password` - The new password (will be Base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if the user doesn't exist
    /// - Returns an error if the old password is incorrect
    /// - Returns an error if the server is unavailable
    /// - Returns an error if the user lacks sufficient privileges
    ///
    /// # Security Notes
    ///
    /// - Both old and new passwords are Base64 encoded before transmission
    /// - The actual password hashing and verification is handled server-side
    /// - Time stamps are set to 0 and will be handled by the server
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Client::new("http://localhost:19530").await?;
    /// client.update_password("john_doe", "old_password", "new_secure_password").await?;
    /// ```
    pub async fn update_password<S: Into<String>>(
        &self,
        user_name: S,
        old_password: S,
        new_password: S,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .update_credential(proto::milvus::UpdateCredentialRequest {
                base: Some(MsgBase::new(MsgType::UpdateCredential)),
                username: user_name.into(),
                old_password: general_purpose::STANDARD.encode(old_password.into().as_bytes()),
                new_password: general_purpose::STANDARD.encode(new_password.into().as_bytes()),
                created_utc_timestamps: 0, // Server will set the actual timestamp
                modified_utc_timestamps: 0, // Server will set the actual timestamp
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }
}
