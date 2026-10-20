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
use milvus::v2::request::dml::{DeleteRequest, InsertRequest, UpsertRequest};
use milvus::v2::request::dql::{
    GetRequest, HybridSearchRequest, QueryIteratorRequest, QueryRequest, SearchIteratorRequest,
    SearchRequest, SubSearchRequest,
};
use milvus::v2::request::rls::*;
use milvus::v2::types::{FieldData, Ids, RowPolicyAction, RowPolicyType, SearchVectors};

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
                .actions([
                    RowPolicyAction::Query,
                    RowPolicyAction::Search,
                    RowPolicyAction::Insert,
                ])
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
        [
            RowPolicyAction::Query,
            RowPolicyAction::Search,
            RowPolicyAction::Insert,
        ]
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
                .using_expr("region == 'east'")
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
                .using_expr("region == 'east'")
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
                .tags(serde_json::json!({"region": "west"}))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("set principal tags");
    assert_eq!(server.service.call_count("set_rls_principal_tags"), 1);

    // A subsequent call upserts only the provided keys onto the existing tags.
    server
        .client
        .set_rls_principal_tags(
            SetRLSPrincipalTagsRequest::builder()
                .collection_name("books")
                .principal_name("alice")
                .tags(serde_json::json!({"level": [1, 2]}))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("merge principal tags");

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

#[tokio::test]
async fn dml_and_dql_requests_carry_rls_fields_on_the_wire() {
    let server = MockServer::start().await;

    let insert = InsertRequest::builder()
        .collection_name("books")
        .rls_principal("alice")
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::VarChar {
                name: "text".into(),
                values: vec!["book".into()],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2]],
            },
        ])
        .build()
        .expect("valid insert request");
    server.client.insert(insert).await.expect("insert succeeds");
    let text = server.service.request_text("insert");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");
    assert!(text.contains("skip_rls: false"), "{text}");

    let upsert = UpsertRequest::builder()
        .insert(
            InsertRequest::builder()
                .collection_name("books")
                .rls_principal("bob")
                .skip_rls(true)
                .columns(vec![
                    FieldData::Int64 {
                        name: "id".into(),
                        values: vec![1],
                    },
                    FieldData::VarChar {
                        name: "text".into(),
                        values: vec!["book".into()],
                    },
                    FieldData::FloatVector {
                        name: "vector".into(),
                        values: vec![vec![0.1, 0.2]],
                    },
                ])
                .build()
                .expect("valid insert request"),
        )
        .build()
        .expect("valid upsert request");
    server.client.upsert(upsert).await.expect("upsert succeeds");
    let text = server.service.request_text("upsert");
    assert!(text.contains("rls_principal: \"bob\""), "{text}");
    assert!(text.contains("skip_rls: true"), "{text}");

    let delete = DeleteRequest::builder()
        .collection_name("books")
        .filter("id > 0")
        .rls_principal("alice")
        .build()
        .expect("valid delete request");
    server.client.delete(delete).await.expect("delete succeeds");
    let text = server.service.request_text("delete");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");

    let query = QueryRequest::builder()
        .collection_name("books")
        .filter("id > 0")
        .rls_principal("alice")
        .build()
        .expect("valid query request");
    server.client.query(query).await.expect("query succeeds");
    let text = server.service.request_text("query");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");

    let get = GetRequest::builder()
        .collection_name("books")
        .ids(Ids::Int64(vec![1]))
        .rls_principal("alice")
        .build()
        .expect("valid get request");
    server.client.get(get).await.expect("get succeeds");
    let text = server.service.request_text("query");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");

    let search = SearchRequest::builder()
        .collection_name("books")
        .vector_field("vector")
        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
        .limit(5)
        .rls_principal("alice")
        .build()
        .expect("valid search request");
    server.client.search(search).await.expect("search succeeds");
    let text = server.service.request_text("search");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");

    let hybrid = HybridSearchRequest::builder()
        .collection_name("books")
        .sub_requests(vec![SubSearchRequest::builder()
            .vector_field("vector")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .limit(5)
            .build()
            .expect("valid sub-search request")])
        .rls_principal("alice")
        .build()
        .expect("valid hybrid-search request");
    server
        .client
        .hybrid_search(hybrid)
        .await
        .expect("hybrid search succeeds");
    let text = server.service.request_text("hybrid_search");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");
}

#[tokio::test]
async fn iterators_carry_rls_fields_on_the_wire() {
    let server = MockServer::start().await;

    let query_iterator = QueryIteratorRequest::builder()
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .rls_principal("alice")
                .build()
                .expect("valid query request"),
        )
        .batch_size(10)
        .build()
        .expect("valid iterator request");
    let iterator = server
        .client
        .query_iterator(query_iterator)
        .await
        .expect("create query iterator");
    let text = server.service.request_text("query");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");
    drop(iterator);

    let search_iterator = SearchIteratorRequest::builder()
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .rls_principal("alice")
                .build()
                .expect("valid search request"),
        )
        .batch_size(10)
        .limit(1)
        .build()
        .expect("valid iterator request");
    let iterator = server
        .client
        .search_iterator(search_iterator)
        .await
        .expect("create search iterator");
    let text = server.service.request_text("search");
    assert!(text.contains("rls_principal: \"alice\""), "{text}");
    drop(iterator);
}

#[tokio::test]
async fn create_row_policy_rejects_duplicates_and_update_rejects_missing() {
    let server = MockServer::start().await;
    let request = || {
        CreateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("region_filter")
            .policy_type(RowPolicyType::Restrictive)
            .actions([RowPolicyAction::Query])
            .using_expr("region == 'west'")
            .build()
            .expect("valid request")
    };

    server
        .client
        .create_row_policy(request())
        .await
        .expect("first create succeeds");
    server
        .client
        .create_row_policy(request())
        .await
        .expect_err("duplicate create must fail");

    server
        .client
        .update_row_policy(
            UpdateRowPolicyRequest::builder()
                .collection_name("books")
                .policy_name("missing_policy")
                .policy_type(RowPolicyType::Permissive)
                .actions([RowPolicyAction::Query])
                .using_expr("region == 'west'")
                .build()
                .expect("valid request"),
        )
        .await
        .expect_err("update of a missing policy must fail");
}
