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

use milvus::v2::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".into());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".into());
    let suffix = unique_suffix();
    // Milvus 2.6 defaults maxUsernameLength to 32 bytes. Keep the username compact while retaining
    // the process ID and millisecond timestamp for uniqueness across tutorial runs.
    let username = format!("u{suffix}");
    let role = format!("rust_tutorial_role_{suffix}");
    let group = format!("rust_tutorial_group_{suffix}");
    // ClientV2::new connects with an administrator credential. The token must be allowed to create
    // users, roles, privilege groups, and grants.
    println!("Calling ClientV2::new: connect with an administrator credential");
    let client = ClientV2::new(&ConnectConfig::new().uri(uri).token(token)).await?;
    println!("ClientV2::new completed");

    let result = demonstrate_rbac(&client, &username, &role, &group).await;
    let cleanup = cleanup_rbac(&client, &username, &role, &group).await;
    if let Err(error) = &cleanup {
        eprintln!("RBAC cleanup failed: {error}");
    }
    result?;
    cleanup
}

async fn demonstrate_rbac(
    client: &ClientV2,
    username: &str,
    role: &str,
    group: &str,
) -> Result<()> {
    // create_user creates a login identity. `username` is the login name, `password` is its secret,
    // and `description` is optional administrator-facing metadata.
    println!("Calling create_user: create {username:?}");
    client
        .create_user(
            CreateUserRequest::builder()
                .username(username)
                .password("RustTutorial!123")
                .description("temporary Rust SDK tutorial user")
                .build()?,
        )
        .await?;
    println!("create_user completed");
    // create_role creates a named permission container; privileges are attached in later calls.
    println!("Calling create_role: create {role:?}");
    client
        .create_role(
            CreateRoleRequest::builder()
                .role_name(role)
                .description("temporary Rust SDK tutorial role")
                .build()?,
        )
        .await?;
    println!("create_role completed");
    // create_privilege_group creates a reusable custom group identified by `group_name`.
    println!("Calling create_privilege_group: create {group:?}");
    client
        .create_privilege_group(
            CreatePrivilegeGroupRequest::builder()
                .group_name(group)
                .build()?,
        )
        .await?;
    println!("create_privilege_group completed");
    // add_privileges_to_group adds built-in privileges to the group. `Query` permits scalar reads
    // and `Search` permits vector searches.
    println!("Calling add_privileges_to_group: add Query and Search");
    client
        .add_privileges_to_group(
            AddPrivilegesToGroupRequest::builder()
                .group_name(group)
                .privileges(["Query", "Search"])
                .build()?,
        )
        .await?;
    println!("add_privileges_to_group completed");
    // grant_privilege attaches the custom group to the role. `database_name` selects the database,
    // `collection_name("*")` covers all collections, and `privilege` names the group.
    println!("Calling grant_privilege: attach {group:?} to {role:?}");
    client
        .grant_privilege(
            GrantPrivilegeRequest::builder()
                .role_name(role)
                .database_name("default")
                .collection_name("*")
                .privilege(group)
                .build()?,
        )
        .await?;
    println!("grant_privilege completed");
    // grant_role assigns the named role to the named user.
    println!("Calling grant_role: assign {role:?} to {username:?}");
    client
        .grant_role(
            GrantRoleRequest::builder()
                .username(username)
                .role_name(role)
                .build()?,
        )
        .await?;
    println!("grant_role completed");

    // describe_user returns user metadata and, with `include_roles(true)`, assigned role names.
    println!("Calling describe_user: inspect {username:?} and its roles");
    let user = client
        .describe_user(
            DescribeUserRequest::builder()
                .username(username)
                .include_roles(true)
                .build()?,
        )
        .await?;
    println!("describe_user completed");
    for user in user.users() {
        println!(
            "user {:?}, roles={:?}",
            user.get_username(),
            user.get_roles()
        );
    }
    // describe_role returns role metadata and grants scoped to `database_name`.
    println!("Calling describe_role: inspect grants for {role:?}");
    let role_description = client
        .describe_role(
            DescribeRoleRequest::builder()
                .role_name(role)
                .database_name("default")
                .build()?,
        )
        .await?;
    println!("describe_role completed");
    println!(
        "role {:?}, grants={:?}",
        role_description.description().get_role_name(),
        role_description.description().get_grant_items()
    );
    Ok(())
}

async fn cleanup_rbac(client: &ClientV2, username: &str, role: &str, group: &str) -> Result<()> {
    let mut failures = Vec::new();

    // revoke_role removes the user's role assignment. Cleanup records a failure but continues so
    // the remaining server-level resources still have a chance to be removed.
    println!("Calling revoke_role: remove {role:?} from {username:?}");
    let revoke_role_result = client
        .revoke_role(
            RevokeRoleRequest::builder()
                .username(username)
                .role_name(role)
                .build()?,
        )
        .await;
    record_cleanup_result(&mut failures, "revoke_role", revoke_role_result);

    // revoke_privilege detaches the group from this role/database/collection scope.
    println!("Calling revoke_privilege: detach {group:?} from {role:?}");
    let revoke_privilege_result = client
        .revoke_privilege(
            RevokePrivilegeRequest::builder()
                .role_name(role)
                .database_name("default")
                .collection_name("*")
                .privilege(group)
                .build()?,
        )
        .await;
    record_cleanup_result(&mut failures, "revoke_privilege", revoke_privilege_result);

    // remove_privileges_from_group removes the built-in privileges before deleting the group.
    println!("Calling remove_privileges_from_group: remove Query and Search");
    let remove_privileges_result = client
        .remove_privileges_from_group(
            RemovePrivilegesFromGroupRequest::builder()
                .group_name(group)
                .privileges(["Query", "Search"])
                .build()?,
        )
        .await;
    record_cleanup_result(
        &mut failures,
        "remove_privileges_from_group",
        remove_privileges_result,
    );

    // drop_user permanently removes the temporary login identity.
    println!("Calling drop_user: remove {username:?}");
    let drop_user_result = client
        .drop_user(DropUserRequest::builder().username(username).build()?)
        .await;
    record_cleanup_result(&mut failures, "drop_user", drop_user_result);

    // drop_role removes the temporary role; `force(true)` also clears remaining associations.
    println!("Calling drop_role: remove {role:?}");
    let drop_role_result = client
        .drop_role(
            DropRoleRequest::builder()
                .role_name(role)
                .force(true)
                .build()?,
        )
        .await;
    record_cleanup_result(&mut failures, "drop_role", drop_role_result);

    // drop_privilege_group removes the custom privilege group. This final attempt runs even when
    // earlier cleanup operations failed.
    println!("Calling drop_privilege_group: remove {group:?}");
    let drop_privilege_group_result = client
        .drop_privilege_group(
            DropPrivilegeGroupRequest::builder()
                .group_name(group)
                .build()?,
        )
        .await;
    record_cleanup_result(
        &mut failures,
        "drop_privilege_group",
        drop_privilege_group_result,
    );

    if failures.is_empty() {
        Ok(())
    } else {
        Err(Error::Unexpected(format!(
            "RBAC cleanup failed after all teardown steps were attempted: {}",
            failures.join("; ")
        )))
    }
}

fn record_cleanup_result(failures: &mut Vec<String>, operation: &str, result: Result<()>) {
    match result {
        Ok(()) => println!("{operation} completed: ok"),
        Err(error) => {
            println!("{operation} completed: failed (cleanup will continue)");
            failures.push(format!("{operation}: {error}"));
        }
    }
}

fn unique_suffix() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}_{millis}", std::process::id())
}
