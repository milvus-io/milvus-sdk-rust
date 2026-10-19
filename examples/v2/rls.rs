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
    const COLLECTION: &str = "RUST_V2_RLS";
    const POLICY: &str = "rust_v2_region_policy";
    const PRINCIPAL: &str = "rust_v2_alice";
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
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("region")
                .data_type(sdk::DataType::VarChar)
                .max_length(16),
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

    // Create a row policy that restricts queries and searches to rows whose region matches the
    // caller's `region` principal tag.
    client
        .create_row_policy(
            CreateRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .policy_type(RowPolicyType::Restrictive)
                .actions([RowPolicyAction::Query, RowPolicyAction::Search])
                .using_expr("region == $region")
                .description("restrict reads by region tag")
                .build()?,
        )
        .await?;

    // Replace the policy definition, preserving its server-side identity.
    client
        .update_row_policy(
            UpdateRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .policy_type(RowPolicyType::Restrictive)
                .actions([
                    RowPolicyAction::Query,
                    RowPolicyAction::Search,
                    RowPolicyAction::QueryIterator,
                ])
                .using_expr("region == $region")
                .build()?,
        )
        .await?;

    // List the collection's row policies.
    let policies = client
        .list_row_policies(
            ListRowPoliciesRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    for policy in policies.policies() {
        println!(
            "policy {} type={:?} actions={:?} using={}",
            policy.get_policy_name(),
            policy.get_policy_type(),
            policy.get_actions(),
            policy.get_using_expr()
        );
    }

    // Set the principal's tags; `$region` in the policy resolves to this value.
    client
        .set_rls_principal_tags(
            SetRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(PRINCIPAL)
                .tags(json!({"region": "west"}))
                .build()?,
        )
        .await?;
    let tags = client
        .get_rls_principal_tags(
            GetRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(PRINCIPAL)
                .build()?,
        )
        .await?;
    println!("{PRINCIPAL} tags: {}", tags.tags());

    let principals = client
        .list_rls_principals(
            ListRLSPrincipalsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("principals with tags: {:?}", principals.principal_names());

    // Remove a single tag, then remove all remaining tags.
    client
        .delete_rls_principal_tags(
            DeleteRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(PRINCIPAL)
                .tag_keys(["region"])
                .build()?,
        )
        .await?;
    client
        .delete_rls_principal_tags(
            DeleteRLSPrincipalTagsRequest::builder()
                .collection_name(COLLECTION)
                .principal_name(PRINCIPAL)
                .build()?,
        )
        .await?;

    // Drop the policy and clean up the collection.
    client
        .drop_row_policy(
            DropRowPolicyRequest::builder()
                .collection_name(COLLECTION)
                .policy_name(POLICY)
                .build()?,
        )
        .await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
