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

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use serde_json::json;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_RBAC";
    const ROLE: &str = "rust_v2_role";
    const USER: &str = "rust_v2_user";
    const GROUP: &str = "rust_v2_privilege_group";
    let client = client().await?;
    drop_collection(&client, COLLECTION).await;
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name("pk")
                .data_type(sdk::DataType::Int64)
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("vector")
                .data_type(sdk::DataType::FloatVector)
                .dimension(8),
        );
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .build()?,
        )
        .await?;
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    sdk::IndexParam::new()
                        .field_name("vector")
                        .index_type(sdk::IndexType::AutoIndex)
                        .metric_type(sdk::MetricType::L2),
                )
                .build()?,
        )
        .await?;
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    println!(
        "roles: {:?}",
        client
            .list_roles(ListRolesRequest::builder().build()?)
            .await?
            .role_names()
    );
    println!(
        "users: {:?}",
        client
            .list_users(ListUsersRequest::builder().build()?)
            .await?
            .usernames()
    );
    let _ = client
        .drop_privilege_group(
            DropPrivilegeGroupRequest::builder()
                .group_name(GROUP)
                .build()?,
        )
        .await;
    client
        .create_privilege_group(
            CreatePrivilegeGroupRequest::builder()
                .group_name(GROUP)
                .build()?,
        )
        .await?;
    client
        .add_privileges_to_group(
            AddPrivilegesToGroupRequest::builder()
                .group_name(GROUP)
                .privileges(["Search", "Query"])
                .build()?,
        )
        .await?;
    let _ = client
        .drop_role(
            DropRoleRequest::builder()
                .role_name(ROLE)
                .force(true)
                .build()?,
        )
        .await;
    client
        .create_role(CreateRoleRequest::builder().role_name(ROLE).build()?)
        .await?;
    client
        .grant_privilege(
            GrantPrivilegeRequest::builder()
                .role_name(ROLE)
                .database_name("default")
                .collection_name(COLLECTION)
                .privilege(GROUP)
                .build()?,
        )
        .await?;
    let role = client
        .describe_role(
            DescribeRoleRequest::builder()
                .role_name(ROLE)
                .database_name("default")
                .build()?,
        )
        .await?;
    println!(
        "Role '{ROLE}' privileges: {:?}",
        role.description().get_grant_items()
    );

    let _ = client
        .drop_user(DropUserRequest::builder().username(USER).build()?)
        .await;
    client
        .create_user(
            CreateUserRequest::builder()
                .username(USER)
                .password("aaaaaa")
                .build()?,
        )
        .await?;
    client
        .update_password(
            UpdatePasswordRequest::builder()
                .username(USER)
                .old_password("aaaaaa")
                .new_password("123456")
                .build()?,
        )
        .await?;
    client
        .grant_role(
            GrantRoleRequest::builder()
                .username(USER)
                .role_name(ROLE)
                .build()?,
        )
        .await?;
    let users = client
        .describe_user(DescribeUserRequest::builder().username(USER).build()?)
        .await?;
    for user in users.users() {
        println!(
            "User '{}' roles: {:?}",
            user.get_username(),
            user.get_roles()
        );
    }
    for group in client
        .list_privilege_groups(ListPrivilegeGroupsRequest::builder().build()?)
        .await?
        .groups()
    {
        println!(
            "Privilege group '{}': {:?}",
            group.get_group_name(),
            group.get_privileges()
        );
    }
    println!(
        "roles: {:?}",
        client
            .list_roles(ListRolesRequest::builder().build()?)
            .await?
            .role_names()
    );
    println!(
        "users: {:?}",
        client
            .list_users(ListUsersRequest::builder().build()?)
            .await?
            .usernames()
    );

    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".into());
    let user_client = ClientV2::new(
        &ConnectConfig::new()
            .uri(uri)
            .username_password(USER, "123456"),
    )
    .await?;
    let row = json!({"vector":float_vector(8)});
    match user_client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .row(row)
                .build()?,
        )
        .await
    {
        Ok(_) => println!("UNEXPECTED! Insert is expected to fail but it succeeded"),
        Err(error) => println!("Insert failed with error: {error}"),
    }
    let count = user_client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .build()?,
        )
        .await?;
    println!("count(*) = {}", query_count(count.results())?);

    client
        .remove_privileges_from_group(
            RemovePrivilegesFromGroupRequest::builder()
                .group_name(GROUP)
                .privileges(["Search", "Query"])
                .build()?,
        )
        .await?;
    client
        .revoke_privilege(
            RevokePrivilegeRequest::builder()
                .role_name(ROLE)
                .database_name("default")
                .collection_name(COLLECTION)
                .privilege(GROUP)
                .build()?,
        )
        .await?;
    client
        .revoke_role(
            RevokeRoleRequest::builder()
                .username(USER)
                .role_name(ROLE)
                .build()?,
        )
        .await?;
    client
        .drop_user(DropUserRequest::builder().username(USER).build()?)
        .await?;
    client
        .drop_role(DropRoleRequest::builder().role_name(ROLE).build()?)
        .await?;
    client
        .drop_privilege_group(
            DropPrivilegeGroupRequest::builder()
                .group_name(GROUP)
                .build()?,
        )
        .await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
