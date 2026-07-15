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

use milvus::v2::request::rbac::{
    AddPrivilegesToGroupRequest, CreatePrivilegeGroupRequest, CreateRoleRequest, CreateUserRequest,
    DescribeRoleRequest, DescribeUserRequest, DropPrivilegeGroupRequest, DropRoleRequest,
    DropUserRequest, GrantRoleRequest, ListPrivilegeGroupsRequest, ListRolesRequest,
    RemovePrivilegesFromGroupRequest, RevokeRoleRequest,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::common;

#[tokio::test]
async fn role_lifecycle() {
    let client = common::client().await;
    let role_name = common::unique_name("role");
    let username = format!(
        "u{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after Unix epoch")
            .as_millis()
    );

    client
        .create_user(
            CreateUserRequest::builder()
                .username(&username)
                .password("Test1234!")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create user");

    client
        .create_role(
            CreateRoleRequest::builder()
                .role_name(&role_name)
                .description("read-only role")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create role");

    let roles = client
        .list_roles(ListRolesRequest::builder().build().expect("valid request"))
        .await
        .expect("list roles after create");
    assert!(roles.role_names().contains(&role_name));

    let response = client
        .describe_role(
            DescribeRoleRequest::builder()
                .role_name(&role_name)
                .database_name("default")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe role");
    let description = response.description();
    assert_eq!(description.get_role_name().to_owned(), role_name);
    assert_eq!(description.get_description().to_owned(), "read-only role");
    assert!(description.get_grant_items().is_empty());

    client
        .grant_role(
            GrantRoleRequest::builder()
                .username(&username)
                .role_name(&role_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("grant role");
    let user = client
        .describe_user(
            DescribeUserRequest::builder()
                .username(&username)
                .include_roles(true)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe user after granting role");
    assert!(user.users()[0].get_roles().contains(&role_name));

    client
        .revoke_role(
            RevokeRoleRequest::builder()
                .username(&username)
                .role_name(&role_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("revoke role");
    let user = client
        .describe_user(
            DescribeUserRequest::builder()
                .username(&username)
                .include_roles(true)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe user after revoking role");
    assert!(!user.users()[0].get_roles().contains(&role_name));

    client
        .drop_user(
            DropUserRequest::builder()
                .username(&username)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop user");

    client
        .drop_role(
            DropRoleRequest::builder()
                .role_name(&role_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop role");

    let roles = client
        .list_roles(ListRolesRequest::builder().build().expect("valid request"))
        .await
        .expect("list roles after drop");
    assert!(!roles.role_names().contains(&role_name));
}

#[tokio::test]
async fn privilege_group_lifecycle() {
    let client = common::client().await;
    let group_name = common::unique_name("privilege_group");

    client
        .create_privilege_group(
            CreatePrivilegeGroupRequest::builder()
                .group_name(&group_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create privilege group");
    client
        .add_privileges_to_group(
            AddPrivilegesToGroupRequest::builder()
                .group_name(&group_name)
                .privileges(["Search", "Query"])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("add privileges to group");

    let groups = client
        .list_privilege_groups(
            ListPrivilegeGroupsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list privilege groups after add");
    let group = groups
        .groups()
        .iter()
        .find(|group| group.get_group_name() == &group_name)
        .expect("created privilege group");
    assert!(group.get_privileges().contains(&"Search".to_owned()));
    assert!(group.get_privileges().contains(&"Query".to_owned()));

    client
        .remove_privileges_from_group(
            RemovePrivilegesFromGroupRequest::builder()
                .group_name(&group_name)
                .privilege("Query")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("remove privilege from group");
    let groups = client
        .list_privilege_groups(
            ListPrivilegeGroupsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list privilege groups after remove");
    let group = groups
        .groups()
        .iter()
        .find(|group| group.get_group_name() == &group_name)
        .expect("privilege group after remove");
    assert!(group.get_privileges().contains(&"Search".to_owned()));
    assert!(!group.get_privileges().contains(&"Query".to_owned()));

    client
        .drop_privilege_group(
            DropPrivilegeGroupRequest::builder()
                .group_name(&group_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop privilege group");
}
