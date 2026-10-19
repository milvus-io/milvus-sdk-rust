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
use milvus::v2::request::rls::*;
use milvus::v2::types::{RowPolicyAction, RowPolicyType};

#[tokio::test]
async fn create_and_list_row_policies_round_trip() {
    let server = MockServer::start().await;

    server
        .client
        .create_row_policy(
            CreateRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("region_filter")
                .policy_type(RowPolicyType::Restrictive)
                .actions([RowPolicyAction::Query, RowPolicyAction::Search])
                .using_expr("region == 'west'")
                .check_expr("region == 'west'")
                .description("west only")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create row policy");
    assert_eq!(server.service.call_count("create_row_policy"), 1);

    let response = server
        .client
        .list_row_policies(
            ListRowPoliciesRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list row policies");
    assert_eq!(server.service.call_count("list_row_policies"), 1);
    assert_eq!(response.collection_name(), "books");
    assert_eq!(response.policies().len(), 1);
    let policy = &response.policies()[0];
    assert_eq!(policy.get_policy_name(), "region_filter");
    assert_eq!(policy.get_policy_type(), RowPolicyType::Restrictive);
    assert_eq!(
        policy.get_actions(),
        [RowPolicyAction::Query, RowPolicyAction::Search]
    );
    assert_eq!(policy.get_using_expr(), "region == 'west'");
    assert_eq!(policy.get_check_expr(), "region == 'west'");
    assert_eq!(policy.get_description(), "west only");
}

#[tokio::test]
async fn update_row_policy_replaces_definition() {
    let server = MockServer::start().await;

    server
        .client
        .create_row_policy(
            CreateRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("region_filter")
                .policy_type(RowPolicyType::Restrictive)
                .actions([RowPolicyAction::Query])
                .using_expr("region == 'east'")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create row policy");

    server
        .client
        .update_row_policy(
            UpdateRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("region_filter")
                .policy_type(RowPolicyType::Permissive)
                .actions([RowPolicyAction::Query, RowPolicyAction::HybridSearch])
                .using_expr("region == 'west'")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("update row policy");
    assert_eq!(server.service.call_count("update_row_policy"), 1);

    let response = server
        .client
        .list_row_policies(
            ListRowPoliciesRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list row policies");
    assert_eq!(response.policies().len(), 1);
    let policy = &response.policies()[0];
    assert_eq!(policy.get_policy_type(), RowPolicyType::Permissive);
    assert_eq!(
        policy.get_actions(),
        [RowPolicyAction::Query, RowPolicyAction::HybridSearch]
    );
    assert_eq!(policy.get_using_expr(), "region == 'west'");
}

#[tokio::test]
async fn drop_row_policy_removes_policy() {
    let server = MockServer::start().await;

    server
        .client
        .create_row_policy(
            CreateRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("region_filter")
                .policy_type(RowPolicyType::Permissive)
                .actions([RowPolicyAction::Query])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create row policy");

    server
        .client
        .drop_row_policy(
            DropRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("region_filter")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop row policy");
    assert_eq!(server.service.call_count("drop_row_policy"), 1);

    let response = server
        .client
        .list_row_policies(
            ListRowPoliciesRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list row policies");
    assert!(response.policies().is_empty());
}

#[tokio::test]
async fn row_policy_requests_default_to_client_database() {
    let server = MockServer::start().await;

    server
        .client
        .create_row_policy(
            CreateRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("p")
                .policy_type(RowPolicyType::Permissive)
                .actions([RowPolicyAction::Query])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create row policy");
    let request = server.service.request_text("create_row_policy");
    assert!(request.contains("db_name: \"default\""), "{request}");
}

#[tokio::test]
async fn set_get_and_list_rls_principal_tags_round_trip() {
    let server = MockServer::start().await;

    server
        .client
        .set_rls_principal_tags(
            SetRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .tags(serde_json::json!({"region": "west", "level": [1, 2]}))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("set principal tags");
    assert_eq!(server.service.call_count("set_rls_principal_tags"), 1);

    let response = server
        .client
        .get_rls_principal_tags(
            GetRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get principal tags");
    assert_eq!(server.service.call_count("get_rls_principal_tags"), 1);
    assert_eq!(response.principal_name(), "alice");
    assert_eq!(
        response.tags(),
        &serde_json::json!({"region": "west", "level": [1, 2]})
    );

    let response = server
        .client
        .list_rls_principals(
            ListRLSPrincipalsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list principals");
    assert_eq!(server.service.call_count("list_rls_principals"), 1);
    assert_eq!(response.principal_names(), ["alice"]);
}

#[tokio::test]
async fn delete_rls_principal_tags_removes_selected_or_all_tags() {
    let server = MockServer::start().await;

    server
        .client
        .set_rls_principal_tags(
            SetRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .tags(serde_json::json!({"region": "west", "level": [1, 2], "team": "search"}))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("set principal tags");

    server
        .client
        .delete_rls_principal_tags(
            DeleteRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .tag_keys(["region"])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("delete selected tags");
    let response = server
        .client
        .get_rls_principal_tags(
            GetRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get principal tags");
    assert_eq!(
        response.tags(),
        &serde_json::json!({"level": [1, 2], "team": "search"})
    );

    server
        .client
        .delete_rls_principal_tags(
            DeleteRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("delete all tags");
    let response = server
        .client
        .list_rls_principals(
            ListRLSPrincipalsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list principals");
    assert!(response.principal_names().is_empty());
    assert_eq!(server.service.call_count("delete_rls_principal_tags"), 2);
}
