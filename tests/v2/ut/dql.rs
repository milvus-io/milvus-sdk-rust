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

use super::common::{wait_for_operation_totals, MockServer};
use milvus::v2::prelude::*;
use milvus::v2::response::dql::{QueryResponse, SearchResponse};
use std::time::Duration;
use tonic::Code;

fn search_request() -> SearchRequest {
    SearchRequest::builder()
        .collection_name("books")
        .vector_field("vector")
        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
        .metric_type(MetricType::Cosine)
        .limit(5)
        .build()
        .expect("valid request")
}

fn telemetry_query_request() -> QueryRequest {
    QueryRequest::builder()
        .collection_name("books")
        .filter("id > 0")
        .build()
        .expect("valid query request")
}

fn telemetry_get_request() -> GetRequest {
    GetRequest::builder()
        .collection_name("books")
        .ids(Ids::Int64(vec![1]))
        .build()
        .expect("valid get request")
}

fn telemetry_hybrid_search_request() -> HybridSearchRequest {
    HybridSearchRequest::builder()
        .collection_name("books")
        .sub_requests(vec![SubSearchRequest::builder()
            .vector_field("vector")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .metric_type(MetricType::Cosine)
            .limit(5)
            .build()
            .expect("valid sub-search request")])
        .rerank(
            Function::new()
                .name("rrf")
                .function_type(FunctionType::Rerank),
        )
        .limit(5)
        .build()
        .expect("valid hybrid-search request")
}

#[tokio::test]
async fn dql_telemetry_records_each_logical_result_once() {
    let server = MockServer::start_with_telemetry(
        TelemetryConfig::new().heartbeat_interval(Duration::from_millis(5)),
    )
    .await;

    server
        .client
        .query(telemetry_query_request())
        .await
        .expect("mock query succeeds");
    server
        .service
        .fail_next_transport("query", Code::InvalidArgument);
    server
        .client
        .query(telemetry_query_request())
        .await
        .expect_err("non-retriable mock query failure reaches the caller");

    server
        .client
        .get(telemetry_get_request())
        .await
        .expect("mock get succeeds");
    server
        .service
        .fail_next_transport("query", Code::InvalidArgument);
    server
        .client
        .get(telemetry_get_request())
        .await
        .expect_err("non-retriable mock get failure reaches the caller");

    server
        .client
        .search(search_request())
        .await
        .expect("mock search succeeds");
    server
        .service
        .fail_next_transport("search", Code::InvalidArgument);
    server
        .client
        .search(search_request())
        .await
        .expect_err("non-retriable mock search failure reaches the caller");

    server
        .client
        .hybrid_search(telemetry_hybrid_search_request())
        .await
        .expect("mock hybrid search succeeds");
    server
        .service
        .fail_next_transport("hybrid_search", Code::InvalidArgument);
    server
        .client
        .hybrid_search(telemetry_hybrid_search_request())
        .await
        .expect_err("non-retriable mock hybrid-search failure reaches the caller");

    for (operation, expected_requests, expected_successes, expected_errors) in [
        ("Query", 4, 2, 2),
        ("Search", 2, 1, 1),
        ("HybridSearch", 2, 1, 1),
    ] {
        let totals = wait_for_operation_totals(&server.client, operation, expected_requests).await;
        assert_eq!(totals.request_count, expected_requests, "{operation}");
        assert_eq!(totals.success_count, expected_successes, "{operation}");
        assert_eq!(totals.error_count, expected_errors, "{operation}");
        assert!(totals.max_latency_ms > 0.0, "{operation}");
    }
    assert_eq!(server.service.call_count("query"), 4);
    assert_eq!(server.service.call_count("search"), 2);
    assert_eq!(server.service.call_count("hybrid_search"), 2);
    server.shutdown().await;
}

#[tokio::test]
async fn client_request_id_reaches_query_wire_only_when_valid() {
    const VALID: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    let server = MockServer::start_with_telemetry(TelemetryConfig::new().enabled(false)).await;

    for request_id in [
        VALID,
        "legacy-request-id",
        "00000000000000000000000000000000",
        "",
    ] {
        with_client_request_id(request_id, server.client.query(telemetry_query_request()))
            .await
            .expect("mock query succeeds");
    }

    assert_eq!(
        server.service.client_request_ids("query"),
        vec![Some(VALID.to_owned()), None, None, None]
    );
    server.shutdown().await;
}

#[test]
fn result_types_support_borrowed_row_iteration() {
    let query_results = QueryResults::new()
        .output_fields(vec![
            FieldData::int64("id", vec![1, 2]),
            FieldData::varchar("text", vec!["first".into(), "second".into()]),
        ])
        .output_field_names(["id", "text"]);
    let mut query_rows: ResultRowIter<'_> = query_results.rows().expect("valid query rows");
    let query_row: ResultRow<'_> = query_rows.next().expect("first query row");
    assert_eq!(query_row.element_offset(), None);
    assert_eq!(query_row.get_i64("id").unwrap(), 1);
    assert_eq!(query_row.get_str("text").unwrap(), "first");
    assert!(matches!(
        query_row.get("id").unwrap(),
        ResultValue::Int64(1)
    ));
    assert!(matches!(
        query_row.get("text").unwrap(),
        ResultValue::String("first")
    ));
    assert_eq!(query_row.to_entity_row().unwrap()["text"], "first");
    assert_eq!(query_rows.len(), 1);

    let single_result = SingleResult::new()
        .ids(Ids::Int64(vec![10]))
        .scores(vec![0.75])
        .element_indices(Some(vec![3]))
        .output_fields(vec![FieldData::varchar("text", vec!["match".into()])])
        .output_field_names(["text"])
        .primary_field_name("id")
        .score_field_name("score");
    let search_results = SearchResults::new().results(vec![single_result]);
    assert_eq!(search_results.len(), 1);
    let result: &SingleResult = search_results.iter().next().expect("one target result");
    let mut search_rows: ResultRowIter<'_> = result.rows().expect("valid search rows");
    let search_row: ResultRow<'_> = search_rows.next().expect("first search row");
    assert_eq!(search_row.element_offset(), Some(3));
    assert_eq!(search_row.get_i64("id").unwrap(), 10);
    assert_eq!(search_row.get_f32("score").unwrap(), 0.75);
    assert_eq!(search_row.get_str("text").unwrap(), "match");
    assert!(matches!(
        search_row.get("id").unwrap(),
        ResultValue::Int64(10)
    ));
    assert!(matches!(
        search_row.get("score").unwrap(),
        ResultValue::Float(0.75)
    ));
    assert_eq!(search_row.to_entity_row().unwrap()["id"], 10);
    assert!(search_rows.next().is_none());
}

pub(super) fn assert_query_response(response: &QueryResponse) {
    assert_eq!(response.session_timestamp().to_owned(), 300);
    let results = response.results();
    assert_eq!(results.get_output_field_names().to_owned(), ["id", "text"]);
    assert_eq!(results.get_row_count().to_owned(), 1);
    assert!(!results.is_empty());
    assert!(results.get_output_field("id").is_some());
    assert!(results.get_output_field("text").is_some());
    let mut borrowed = results.rows().unwrap();
    let row = borrowed.next().expect("one query result row");
    assert_eq!(row.get_i64("id").unwrap(), 1);
    assert_eq!(row.get_str("text").unwrap(), "book");
    assert!(borrowed.next().is_none());
    let rows = results.get_output_rows().unwrap();
    assert_eq!(rows[0]["id"], 1);
    assert_eq!(rows[0]["text"], "book");
    assert_eq!(results.get_output_row(0).unwrap().to_owned(), rows[0]);
}

pub(super) fn assert_search_response(response: &SearchResponse) {
    assert_eq!(response.session_timestamp().to_owned(), 301);
    assert_eq!(response.cost().to_owned(), 7);
    assert_eq!(response.scanned_remote_bytes().to_owned(), 11);
    assert_eq!(response.scanned_total_bytes().to_owned(), 13);
    assert_eq!(response.cache_hit_ratio().to_owned(), 0.5);
    let results = response.results();
    assert!(results.get_recalls().is_empty());
    assert_eq!(results.len(), 1);
    assert!(!results.is_empty());
    assert_eq!(results.get_results().len().to_owned(), 1);
    let result = &results.get_results()[0];
    assert_eq!(result.get_ids().to_owned(), Ids::Int64(vec![1]));
    assert_eq!(result.get_scores().to_owned(), [0.9]);
    assert_eq!(result.get_output_field_names().to_owned(), ["text"]);
    assert!(result.get_output_field("text").is_some());
    assert_eq!(result.get_primary_field_name().to_owned(), "id");
    assert_eq!(result.get_score_field_name().to_owned(), "score");
    assert_eq!(result.get_highlight_results().len().to_owned(), 1);
    assert!(result.get_highlight_results()[0].is_empty());
    assert_eq!(result.len(), 1);
    assert!(!result.is_empty());
    let mut borrowed = result.rows().unwrap();
    let row = borrowed.next().expect("one search result row");
    assert_eq!(row.get_i64("id").unwrap(), 1);
    assert_eq!(row.get_str("text").unwrap(), "book");
    assert!((row.get_f32("score").unwrap() - 0.9).abs() < 1e-6);
    assert!(borrowed.next().is_none());
    let rows = result.get_output_rows().unwrap();
    assert_eq!(rows[0]["id"], 1);
    assert_eq!(rows[0]["text"], "book");
    assert!((rows[0]["score"].as_f64().unwrap() - 0.9).abs() < 1e-6);
    assert_eq!(result.get_output_row(0).unwrap().to_owned(), rows[0]);
}

#[tokio::test]
async fn dql_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    let query = client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_query_response(&query);

    let query_by_ids = client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .ids(Ids::Int64(vec![1, 2]))
                .output_fields(["text"])
                .build()
                .expect("valid ID query request"),
        )
        .await
        .unwrap();
    assert_query_response(&query_by_ids);

    let get = client
        .get(
            GetRequest::builder()
                .collection_name("books")
                .ids(Ids::Int64(vec![1]))
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_query_response(&get);

    let search = client.search(search_request()).await.unwrap();
    assert_search_response(&search);

    let search_by_ids = client
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .ids(Ids::Int64(vec![1, 2]))
                .limit(5)
                .build()
                .expect("valid ID search request"),
        )
        .await
        .unwrap();
    assert_search_response(&search_by_ids);

    let hybrid = client
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name("books")
                .sub_requests(vec![SubSearchRequest::builder()
                    .vector_field("vector")
                    .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                    .metric_type(MetricType::Cosine)
                    .limit(5)
                    .build()
                    .expect("valid request")])
                .rerank(
                    Function::new()
                        .name("rrf")
                        .function_type(FunctionType::Rerank),
                )
                .limit(5)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_search_response(&hybrid);

    server.assert_any_request_contains(
        "query",
        &[
            "collection_name: \"books\"",
            "expr: \"id in {__milvus_v2_query_ids}\"",
            "__milvus_v2_query_ids",
            "LongData",
            "data: [",
            "1",
            "2",
        ],
    );
    server.assert_any_request_contains("search", &["collection_name: \"books\"", "nq: 1"]);
    server.assert_request_contains("search", &["Some(Ids", "IntId", "data: [1, 2]", "nq: 2"]);
    server.assert_request_contains(
        "hybrid_search",
        &["collection_name: \"books\"", "requests:"],
    );

    for rpc in ["query", "describe_collection", "search", "hybrid_search"] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn session_timestamps_are_isolated_by_connection_endpoint() {
    let first = MockServer::start().await;
    let second = MockServer::start().await;

    first
        .client
        .insert(
            InsertRequest::builder()
                .collection_name("books")
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
        .await
        .expect("insert on first cluster");

    first
        .client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid first query"),
        )
        .await
        .expect("query first cluster");
    second
        .client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid second query"),
        )
        .await
        .expect("query second cluster");

    first.assert_request_contains("query", &["guarantee_timestamp: 10"]);
    second.assert_request_contains("query", &["guarantee_timestamp: 1"]);

    first.shutdown().await;
    second.shutdown().await;
}

#[tokio::test]
async fn collection_schemas_are_shared_by_endpoint_and_isolated_across_endpoints() {
    const COLLECTION: &str = "dql_schema_cache_sharing_books";

    let first = MockServer::start().await;
    first
        .client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name(COLLECTION)
                .dimension(2)
                .build()
                .expect("valid collection request"),
        )
        .await
        .expect("create collection on first endpoint");
    let same_endpoint = ClientV2::new(&ConnectConfig::new().uri(&first.uri))
        .await
        .expect("connect second client to first endpoint");

    for client in [&first.client, &same_endpoint] {
        client
            .get(
                GetRequest::builder()
                    .collection_name(COLLECTION)
                    .ids(Ids::Int64(vec![1]))
                    .build()
                    .expect("valid get request"),
            )
            .await
            .expect("get through shared schema cache");
    }
    assert_eq!(
        first.service.call_count("describe_collection"),
        1,
        "clients using the same endpoint share one schema cache entry"
    );

    let second = MockServer::start().await;
    second
        .client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name(COLLECTION)
                .dimension(2)
                .build()
                .expect("valid collection request"),
        )
        .await
        .expect("create collection on second endpoint");
    second
        .client
        .get(
            GetRequest::builder()
                .collection_name(COLLECTION)
                .ids(Ids::Int64(vec![1]))
                .build()
                .expect("valid get request"),
        )
        .await
        .expect("get through isolated endpoint cache");
    assert_eq!(second.service.call_count("describe_collection"), 1);

    first.shutdown().await;
    second.shutdown().await;
}

#[tokio::test]
async fn empty_database_name_uses_selected_database_for_dml_and_session_reads() {
    let server = MockServer::start().await;
    let client = &server.client;
    let collection = "selected_database_session_books";
    client
        .use_database("tenant")
        .expect("select tenant database");
    client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name(collection)
                .dimension(2)
                .build()
                .expect("valid collection request"),
        )
        .await
        .expect("create collection in selected database");

    client
        .insert(
            InsertRequest::builder()
                .database_name("")
                .collection_name(collection)
                .columns(vec![
                    FieldData::Int64 {
                        name: "id".into(),
                        values: vec![1],
                    },
                    FieldData::FloatVector {
                        name: "vector".into(),
                        values: vec![vec![0.1, 0.2]],
                    },
                ])
                .build()
                .expect("valid insert request"),
        )
        .await
        .expect("insert through selected database");
    client
        .query(
            QueryRequest::builder()
                .database_name("")
                .collection_name(collection)
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid query request"),
        )
        .await
        .expect("query through selected database");

    server.assert_any_request_contains(
        "describe_collection",
        &[
            "db_name: \"tenant\"",
            &format!("collection_name: \"{collection}\""),
        ],
    );
    server.assert_request_contains("query", &["guarantee_timestamp: 10"]);
    server.shutdown().await;
}

#[tokio::test]
async fn missing_primary_field_name_does_not_load_the_schema_cache() {
    let server = MockServer::start().await;
    let client = &server.client;
    let collection = "missing_primary_field";
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(collection)
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .metric_type(MetricType::Cosine)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search decoding preserves the missing primary-field name");
    let search_result = &search.results().get_results()[0];
    assert!(search_result.get_primary_field_name().is_empty());
    assert!(search_result.get_output_rows().is_err());

    let hybrid = client
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name(collection)
                .sub_requests(vec![SubSearchRequest::builder()
                    .vector_field("vector")
                    .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                    .metric_type(MetricType::Cosine)
                    .build()
                    .expect("valid request")])
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("hybrid-search decoding preserves the missing primary-field name");
    let hybrid_result = &hybrid.results().get_results()[0];
    assert!(hybrid_result.get_primary_field_name().is_empty());
    assert!(hybrid_result.get_output_rows().is_err());
    assert_eq!(server.service.call_count("describe_collection"), 0);
    server.shutdown().await;
}
