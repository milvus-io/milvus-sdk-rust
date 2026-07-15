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

use super::common::MockServer;
use milvus::v2::error::Error;
use milvus::v2::request::rbac::*;
use milvus::v2::RetryConfig;
use std::time::Duration;
use tonic::Code;

const AMBIGUOUS_TRANSPORT_CODES: [Code; 5] = [
    Code::Unavailable,
    Code::Unknown,
    Code::Internal,
    Code::Aborted,
    Code::Cancelled,
];

#[tokio::test]
async fn update_password_does_not_retry_ambiguous_transport_errors() {
    let server = MockServer::start().await;
    server.client.set_retry_param(
        RetryConfig::new()
            .max_attempts(3)
            .initial_backoff(Duration::ZERO)
            .max_backoff(Duration::ZERO),
    );

    for (attempt, code) in AMBIGUOUS_TRANSPORT_CODES.into_iter().enumerate() {
        server
            .service
            .fail_next_transport("update_credential", code);
        let error = server
            .client
            .update_password(
                UpdatePasswordRequest::builder()
                    .username("alice")
                    .old_password("old-password")
                    .new_password("new-password")
                    .build()
                    .expect("valid update-password request"),
            )
            .await
            .expect_err("ambiguous password-update failure must be returned");
        assert!(matches!(error, Error::Grpc(status) if status.code() == code));
        assert_eq!(server.service.call_count("update_credential"), attempt + 1);
    }

    server.shutdown().await;
}

#[tokio::test]
async fn rbac_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_user(
            CreateUserRequest::builder()
                .username("alice")
                .password("password")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .update_password(
            UpdatePasswordRequest::builder()
                .username("alice")
                .old_password("password")
                .new_password("new-password")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .update_user(
            UpdateUserRequest::builder()
                .username("alice")
                .description("engineer")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let users = client
        .list_users(ListUsersRequest::builder().build().expect("valid request"))
        .await
        .unwrap();
    assert_eq!(users.usernames().to_owned(), ["alice"]);

    client
        .create_role(
            CreateRoleRequest::builder()
                .role_name("reader")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .alter_role(
            AlterRoleRequest::builder()
                .role_name("reader")
                .description("read access")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let roles = client
        .list_roles(ListRolesRequest::builder().build().expect("valid request"))
        .await
        .unwrap();
    assert_eq!(roles.role_names().to_owned(), ["reader"]);

    client
        .grant_role(
            GrantRoleRequest::builder()
                .username("alice")
                .role_name("reader")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let user = client
        .describe_user(
            DescribeUserRequest::builder()
                .username("alice")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(user.users().len().to_owned(), 1);
    let user = &user.users()[0];
    assert_eq!(user.get_username().to_owned(), "alice");
    assert_eq!(user.get_description().to_owned(), "engineer");
    assert_eq!(user.get_roles().to_owned(), ["reader"]);

    client
        .grant_privilege(
            GrantPrivilegeRequest::builder()
                .role_name("reader")
                .database_name("default")
                .collection_name("books")
                .privilege("Query")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let role = client
        .describe_role(
            DescribeRoleRequest::builder()
                .role_name("reader")
                .database_name("default")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = role.description();
    assert_eq!(description.get_role_name().to_owned(), "reader");
    assert_eq!(description.get_description().to_owned(), "read access");
    assert_eq!(description.get_grant_items().len().to_owned(), 1);
    let grant = &description.get_grant_items()[0];
    assert_eq!(grant.get_object_type().to_owned(), "Collection");
    assert_eq!(grant.get_object_name().to_owned(), "books");
    assert_eq!(grant.get_database_name().to_owned(), "default");
    assert_eq!(grant.get_role_name().to_owned(), "reader");
    assert_eq!(grant.get_privilege().to_owned(), "Query");
    assert!(grant.get_grantor_name().is_empty());

    client
        .create_privilege_group(
            CreatePrivilegeGroupRequest::builder()
                .group_name("readers")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .add_privileges_to_group(
            AddPrivilegesToGroupRequest::builder()
                .group_name("readers")
                .privilege("Query")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let groups = client
        .list_privilege_groups(
            ListPrivilegeGroupsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(groups.groups().len().to_owned(), 1);
    assert_eq!(groups.groups()[0].get_group_name().to_owned(), "readers");
    assert_eq!(groups.groups()[0].get_privileges().to_owned(), ["Query"]);

    client
        .revoke_privilege(
            RevokePrivilegeRequest::builder()
                .role_name("reader")
                .database_name("default")
                .collection_name("books")
                .privilege("Query")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let role = client
        .describe_role(
            DescribeRoleRequest::builder()
                .role_name("reader")
                .database_name("default")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(role.description().get_grant_items().is_empty());

    client
        .revoke_role(
            RevokeRoleRequest::builder()
                .username("alice")
                .role_name("reader")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let user = client
        .describe_user(
            DescribeUserRequest::builder()
                .username("alice")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(user.users()[0].get_roles().is_empty());

    client
        .remove_privileges_from_group(
            RemovePrivilegesFromGroupRequest::builder()
                .group_name("readers")
                .privilege("Query")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let groups = client
        .list_privilege_groups(
            ListPrivilegeGroupsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(groups.groups()[0].get_privileges().is_empty());

    client
        .drop_privilege_group(
            DropPrivilegeGroupRequest::builder()
                .group_name("readers")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(client
        .list_privilege_groups(
            ListPrivilegeGroupsRequest::builder()
                .build()
                .expect("valid request")
        )
        .await
        .unwrap()
        .groups()
        .is_empty());

    client
        .drop_role(
            DropRoleRequest::builder()
                .role_name("reader")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(client
        .list_roles(ListRolesRequest::builder().build().expect("valid request"))
        .await
        .unwrap()
        .role_names()
        .is_empty());

    client
        .drop_user(
            DropUserRequest::builder()
                .username("alice")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(client
        .list_users(ListUsersRequest::builder().build().expect("valid request"))
        .await
        .unwrap()
        .usernames()
        .is_empty());

    server.assert_request_contains("create_credential", &["username: \"alice\""]);
    server.assert_any_request_contains("update_credential", &["username: \"alice\""]);
    server.assert_any_request_contains(
        "operate_user_role",
        &["username: \"alice\"", "role_name: \"reader\""],
    );
    server.assert_any_request_contains("operate_privilege_v2", &["name: \"Query\""]);
    server.assert_any_request_contains("operate_privilege_group", &["group_name: \"readers\""]);
    server.assert_request_contains("delete_credential", &["username: \"alice\""]);

    for rpc in [
        "create_credential",
        "update_credential",
        "list_cred_users",
        "select_role",
        "create_role",
        "alter_role",
        "operate_user_role",
        "select_grant",
        "select_user",
        "operate_privilege_v2",
        "create_privilege_group",
        "list_privilege_groups",
        "operate_privilege_group",
        "drop_privilege_group",
        "drop_role",
        "delete_credential",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}
