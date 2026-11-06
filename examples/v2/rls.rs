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

// Row-level security (RLS) lets you control which rows a principal can read or write.
// This example manages policies and principal tags, then shows how a request that
// carries an `rls_principal` is enforced by Milvus (row filtering and write checks).
// Enable RBAC in Milvus and have the root user credentials ready.
// Read the doc to know more about RLS in Milvus: https://milvus.io/docs/row-level-security.md

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use std::collections::HashMap;
use utils::*;

const COLLECTION: &str = "RUST_V2_RLS_COLLECTION";
const POLICY: &str = "rust_v2_rls_policy";
const ALICE: &str = "alice";
const BOB: &str = "bob";

fn row(id: i64, team: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "vector": [1.0f32, 2.0f32],
        "team": team,
    })
}

async fn cleanup(client: &ClientV2) {
    let _ = client
        .drop_row_policy(
            DropRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .build()
                .expect("valid drop-row-policy request"),
        )
        .await;
    drop_collection(client, COLLECTION).await;
}

async fn create_collection(client: &ClientV2) -> Result<()> {
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("vector")
                .data_type(sdk::DataType::FloatVector)
                .dimension(2),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("team")
                .data_type(sdk::DataType::VarChar)
                .max_length(64),
        );
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                // RLS is enforced only when the collection opts in via this property.
                .properties(HashMap::from([(
                    "rls.enabled".to_owned(),
                    "true".to_owned(),
                )]))
                .build()?,
        )
        .await?;
    println!("Collection created: {COLLECTION}");
    Ok(())
}

async fn run_rls_example(client: &ClientV2) -> Result<()> {
    // A policy expression may reference the current principal and its tags:
    //   $current_principal_tags['team']  ->  the 'team' tag of the request principal.
    client
        .create_row_policy(
            CreateRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .policy_type(RowPolicyType::Permissive)
                .actions([
                    RowPolicyAction::Search,
                    RowPolicyAction::Query,
                    RowPolicyAction::Insert,
                ])
                .using_expr("team == $current_principal_tags['team']")
                .check_expr("team == $current_principal_tags['team']")
                .description("A principal only sees or writes rows of its own team")
                .build()?,
        )
        .await?;
    println!("Row policy created: {POLICY}");

    let policies = client
        .list_row_policies(
            ListRowPoliciesRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("After create, row policies:");
    for policy in policies.policies() {
        println!(
            "\t{} type={:?} actions={:?} using={} check={}",
            policy.get_policy_name(),
            policy.get_policy_type(),
            policy.get_actions(),
            policy.get_using_expr(),
            policy.get_check_expr()
        );
    }

    client
        .update_row_policy(
            UpdateRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .policy_type(RowPolicyType::Permissive)
                .actions([
                    RowPolicyAction::Search,
                    RowPolicyAction::Query,
                    RowPolicyAction::Insert,
                    RowPolicyAction::Delete,
                ])
                .using_expr("team == $current_principal_tags['team']")
                .check_expr("team == $current_principal_tags['team']")
                .description("Updated: a principal sees or writes only rows of its own team")
                .build()?,
        )
        .await?;
    println!("Row policy updated: {POLICY}");

    client
        .set_rls_principal_tags(
            SetRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(ALICE)
                .tags(serde_json::json!({"team": "engineering", "level": 3}))
                .build()?,
        )
        .await?;
    client
        .set_rls_principal_tags(
            SetRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(BOB)
                .tags(serde_json::json!({"team": "marketing"}))
                .build()?,
        )
        .await?;
    println!("Principal tags set: {ALICE}=engineering, {BOB}=marketing");

    let tags = client
        .get_rls_principal_tags(
            GetRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(ALICE)
                .build()?,
        )
        .await?;
    println!("{ALICE} tags: {}", tags.tags());

    let principals = client
        .list_rls_principals(
            ListRLSPrincipalsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Principals with tags: {:?}", principals.principal_names());

    client
        .delete_rls_principal_tags(
            DeleteRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(ALICE)
                .tag_keys(["level"])
                .build()?,
        )
        .await?;
    println!("Alice tag deleted: level");

    // ---- enforcement: writes are checked against the policy ----
    client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows([row(1, "engineering")])
                .rls_principal(ALICE)
                .build()?,
        )
        .await?;
    println!("Insert engineering row as {ALICE}: allowed");

    match client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows([row(2, "marketing")])
                .rls_principal(ALICE)
                .build()?,
        )
        .await
    {
        Ok(_) => println!("Insert marketing row as {ALICE}: unexpected success"),
        Err(_) => println!("Insert marketing row as {ALICE}: rejected by RLS check expression"),
    }

    client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows([row(3, "marketing")])
                .rls_principal(BOB)
                .build()?,
        )
        .await?;
    println!("Insert marketing row as {BOB}: allowed");
    flush(client, COLLECTION).await?;

    // Load the collection so query can run.
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

    // ---- enforcement: reads are filtered by the principal's tags ----
    let alice_rows = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id > 0")
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .rls_principal(ALICE)
                .build()?,
        )
        .await?;
    println!(
        "Query rows as {ALICE}: {}",
        alice_rows.results().get_row_count()
    );

    let bob_rows = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id > 0")
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .rls_principal(BOB)
                .build()?,
        )
        .await?;
    println!(
        "Query rows as {BOB}: {}",
        bob_rows.results().get_row_count()
    );

    let all_rows = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id > 0")
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .skip_rls(true)
                .build()?,
        )
        .await?;
    println!(
        "Query rows with skip_rls: {}",
        all_rows.results().get_row_count()
    );

    match client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id > 0")
                .build()?,
        )
        .await
    {
        Ok(_) => println!("Query without rls_principal: unexpected success"),
        Err(_) => println!("Query without rls_principal: denied, rls_principal is required"),
    }

    client
        .drop_row_policy(
            DropRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .build()?,
        )
        .await?;
    println!("Row policy dropped: {POLICY}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    cleanup(&client).await;
    create_collection(&client).await?;
    run_rls_example(&client).await?;
    cleanup(&client).await;
    Ok(())
}
