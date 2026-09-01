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
use super::dql::{assert_query_response, assert_search_response};
use milvus::v2::prelude::*;

#[tokio::test]
async fn search_iterator_v2_direct_describe_bypasses_the_schema_cache() {
    let server = MockServer::start().await;

    server
        .client
        .get(
            GetRequest::builder()
                .collection_name("books")
                .ids(Ids::Int64(vec![1]))
                .build()
                .expect("valid get request"),
        )
        .await
        .expect("prime schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    let iterator = server
        .client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .metric_type(MetricType::Cosine)
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(10)
                .limit(1)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator");
    assert!(matches!(iterator, SearchIterator::V2(_)));
    assert_eq!(server.service.call_count("describe_collection"), 2);

    server.shutdown().await;
}

#[tokio::test]
async fn zero_limit_iterators_finish_without_rpc_work() {
    let server = MockServer::start().await;

    let mut query = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("missing_collection")
                        .build()
                        .expect("valid query"),
                )
                .limit(0)
                .build()
                .expect("valid zero-limit query iterator"),
        )
        .await
        .expect("create zero-limit query iterator");
    assert!(query.next().await.expect("finish query iterator").is_none());

    let mut search = server
        .client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("missing_collection")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .build()
                        .expect("valid search request"),
                )
                .limit(0)
                .build()
                .expect("valid zero-limit search iterator"),
        )
        .await
        .expect("create zero-limit search iterator");
    assert!(search
        .next()
        .await
        .expect("finish search iterator")
        .is_none());

    for rpc in ["describe_collection", "describe_index", "query", "search"] {
        assert_eq!(
            server.service.call_count(rpc),
            0,
            "zero-limit iterators must not call {rpc}"
        );
    }

    server.shutdown().await;
}

#[tokio::test]
async fn iterator_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    let mut query = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("id > 0")
                        .limit(1)
                        .build()
                        .expect("valid request"),
                )
                .batch_size(10)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let query_page = query.next().await.unwrap().unwrap();
    assert_query_response(&query_page);
    assert!(query.next().await.unwrap().is_none());

    let mut search = client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .metric_type(MetricType::Cosine)
                        .build()
                        .expect("valid request"),
                )
                .batch_size(10)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(matches!(&search, SearchIterator::V2(_)));
    let search_page = search.next().await.unwrap().unwrap();
    assert_search_response(&search_page);
    assert!(search.next().await.unwrap().is_none());

    server.assert_called("query");
    server.assert_called("describe_collection");
    server.assert_called("search");
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_v2_lets_the_server_deduce_the_default_metric() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .metric_type(MetricType::Default)
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(10)
                .limit(1)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator");

    assert!(matches!(&iterator, SearchIterator::V2(_)));
    assert!(iterator.next().await.expect("fetch search page").is_some());
    assert_eq!(server.service.call_count("describe_index"), 0);
    assert!(server
        .service
        .request_texts("search")
        .iter()
        .all(|request| !request.contains("metric_type")));

    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_falls_back_to_legacy_range_search() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .filter("legacy_search_iterator")
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(2)
                .limit(3)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create legacy search iterator");
    assert!(matches!(&iterator, SearchIterator::V1(_)));

    let first = iterator
        .next()
        .await
        .expect("fetch first legacy page")
        .expect("first legacy page");
    let second = iterator
        .next()
        .await
        .expect("fetch second legacy page")
        .expect("second legacy page");
    assert_eq!(search_ids(&first), [1, 2]);
    assert_eq!(search_ids(&second), [3]);
    assert!(iterator
        .next()
        .await
        .expect("finish legacy iterator")
        .is_none());

    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].contains("search_iter_v2"));
    assert!(!requests[1].contains("search_iter_v2"));
    assert!(!requests[2].contains("search_iter_v2"));
    assert!(!requests[0].contains("metric_type"));
    assert!(requests[1].contains("key: \"metric_type\", value: \"COSINE\""));
    assert!(requests[2].contains("key: \"metric_type\", value: \"COSINE\""));
    assert!(requests[2].contains("range_filter"));
    assert!(requests[2].contains("radius"));
    assert!(requests[2].contains("id not in [2]"));
    assert_eq!(guarantee_timestamp(&requests[2]), 301);
    assert_eq!(server.service.call_count("describe_index"), 1);
    assert_eq!(server.service.call_count("describe_collection"), 2);

    server.shutdown().await;
}

#[tokio::test]
async fn legacy_hamming_iterator_crosses_empty_integer_distance_bands() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Binary(vec![vec![0b1010_1010]]))
                        .metric_type(MetricType::Hamming)
                        .filter("legacy_hamming_gap_iterator")
                        .build()
                        .expect("valid Hamming search request"),
                )
                .batch_size(1)
                .limit(2)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create legacy Hamming iterator");
    assert!(matches!(&iterator, SearchIterator::V1(_)));

    let first = iterator
        .next()
        .await
        .expect("fetch first Hamming page")
        .expect("first Hamming page");
    let second = iterator
        .next()
        .await
        .expect("cross empty distance band")
        .expect("second Hamming page");

    assert_eq!(search_ids(&first), [1]);
    assert_eq!(search_ids(&second), [2]);
    assert!(iterator
        .next()
        .await
        .expect("finish Hamming iterator")
        .is_none());

    let requests = server.service.request_texts("search");
    assert!(requests
        .iter()
        .any(|request| request.contains("key: \"radius\", value: \"2\"")));
    server.shutdown().await;
}

#[tokio::test]
async fn iterators_treat_empty_database_name_as_the_selected_database() {
    let server = MockServer::start().await;
    let client = &server.client;
    let collection = "selected_database_iterator_books";
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

    let mut query = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .database_name("")
                        .collection_name(collection)
                        .output_fields(["id"])
                        .limit(1)
                        .build()
                        .expect("valid query request"),
                )
                .batch_size(1)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create query iterator through selected database");
    assert!(query.next().await.expect("fetch query page").is_some());

    let mut search = client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .database_name("")
                        .collection_name(collection)
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .metric_type(MetricType::Cosine)
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(1)
                .limit(1)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator through selected database");
    assert!(search.next().await.expect("fetch search page").is_some());

    server.assert_any_request_contains(
        "describe_collection",
        &[
            "db_name: \"tenant\"",
            &format!("collection_name: \"{collection}\""),
        ],
    );
    server.shutdown().await;
}

#[tokio::test]
async fn iterators_pin_a_client_timestamp_when_probe_session_ts_is_zero() {
    let server = MockServer::start().await;
    let client = &server.client;

    let mut query = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("zero_session_ts")
                        .limit(3)
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");
    for _ in 0..3 {
        assert!(query.next().await.expect("fetch query page").is_some());
    }
    assert!(query.next().await.expect("finish query iterator").is_none());

    let query_requests = server.service.request_texts("query");
    assert_eq!(query_requests.len(), 4);
    let query_fallback = guarantee_timestamp(&query_requests[1]);
    assert_hybrid_timestamp(query_fallback);
    assert_eq!(guarantee_timestamp(&query_requests[2]), query_fallback);
    assert_eq!(guarantee_timestamp(&query_requests[3]), query_fallback);

    let mut search = client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .metric_type(MetricType::Cosine)
                        .filter("zero_session_ts")
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .limit(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create search iterator");
    assert!(search.next().await.expect("fetch search page").is_some());
    assert!(search
        .next()
        .await
        .expect("finish search iterator")
        .is_none());

    let search_requests = server.service.request_texts("search");
    assert_eq!(search_requests.len(), 3);
    assert_eq!(guarantee_timestamp(&search_requests[0]), 0);
    let search_fallback = guarantee_timestamp(&search_requests[1]);
    assert_hybrid_timestamp(search_fallback);
    assert_eq!(guarantee_timestamp(&search_requests[2]), search_fallback);

    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_does_not_advance_state_when_decoding_fails() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .metric_type(MetricType::Cosine)
                        .filter("decode_failure_search_iterator")
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(10)
                .limit(2)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator");

    assert!(iterator.next().await.is_err());
    let page = iterator
        .next()
        .await
        .expect("retry failed search page")
        .expect("search page");
    assert_search_response(&page);
    assert!(iterator
        .next()
        .await
        .expect("finish search iterator")
        .is_none());

    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 4);
    assert_eq!(guarantee_timestamp(&requests[0]), 0);
    for request in &requests[1..=2] {
        assert!(!request.contains("search_iter_id"));
        assert!(!request.contains("search_iter_last_bound"));
        assert_eq!(guarantee_timestamp(request), 301);
        assert!(request.contains("KeyValuePair { key: \"topk\", value: \"10\" }"));
    }
    assert!(requests[3].contains("search_iter_id"));
    assert!(requests[3].contains("search_iter_last_bound"));
    assert_eq!(guarantee_timestamp(&requests[3]), 301);
    assert!(requests[3].contains("KeyValuePair { key: \"topk\", value: \"10\" }"));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_advances_by_primary_key_beyond_query_window() {
    let server = MockServer::start().await;
    let client = &server.client;

    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("large_query_iterator")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(8_192)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let mut row_count = 0;
    while let Some(page) = iterator.next().await.expect("fetch query iterator page") {
        row_count += page.results().get_output_fields()[0].len();
    }
    assert_eq!(row_count, 17_000);

    let requests = server.service.request_texts("query");
    assert_eq!(
        requests.len(),
        4,
        "one timestamp probe and three data pages"
    );
    assert!(requests
        .iter()
        .all(|request| request.contains("KeyValuePair { key: \"offset\", value: \"0\" }")));
    assert!(requests[2].contains("expr: \"id > 8191 and (large_query_iterator)\""));
    assert!(requests[3].contains("expr: \"id > 16383 and (large_query_iterator)\""));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_resumes_from_an_initial_cursor() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("large_query_iterator")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(8_192)
                .cursor(QueryCursor::int64(0, 8_191))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let mut row_count = 0;
    while let Some(page) = iterator.next().await.expect("fetch query iterator page") {
        row_count += page.results().get_output_fields()[0].len();
    }
    // The initial cursor makes the first data page start after id 8191, so the returned rows are
    // the full 17_000 minus the 8_192 already consumed before resume.
    assert_eq!(row_count, 17_000 - 8_192);
    // The iterator exposes its cursor position after reading.
    assert!(iterator.cursor().is_some());

    let requests = server.service.request_texts("query");
    // The timestamp probe carries no cursor predicate; the first data page resumes after it.
    assert!(requests[1].contains("expr: \"id > 8191 and (large_query_iterator)\""));
    assert!(requests[2].contains("expr: \"id > 16383 and (large_query_iterator)\""));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_resume_ignores_request_offset() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("large_query_iterator")
                        .offset(1000)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(8_192)
                .cursor(QueryCursor::int64(0, 8_191))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    while let Some(page) = iterator.next().await.expect("fetch query iterator page") {
        let _ = page.results().get_output_fields()[0].len();
    }

    let requests = server.service.request_texts("query");
    // A captured cursor already positions past the previously consumed rows, so the offset must
    // not be re-applied on resume (no seek, first data page still starts after id 8191).
    assert!(
        !requests
            .iter()
            .any(|request| request.contains("key: \"iterator\", value: \"false\"")),
        "resuming from a cursor must not run an offset seek: {requests:?}"
    );
    assert!(requests[1].contains("expr: \"id > 8191 and (large_query_iterator)\""));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_element_filter_captures_and_resumes_from_element_cursor() {
    let server = MockServer::start().await;
    let filter = "element_filter(tags, tag == \"x\") and element_filter_query_iterator";

    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let first = iterator
        .next()
        .await
        .expect("fetch first page")
        .expect("non-empty first page");
    assert!(!query_ids(&first).is_empty());

    // The element-filter cursor must carry the element position within the last primary key.
    let cursor = iterator.cursor().expect("captured element cursor");
    assert!(
        cursor.get_last_element_offset().is_some(),
        "element-filter cursor must carry an element offset: {cursor:?}"
    );

    let mut resumed = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .cursor(cursor)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create resumed query iterator");
    let _ = resumed
        .next()
        .await
        .expect("fetch resumed page")
        .expect("non-empty resumed page");

    let requests = server.service.request_texts("query");
    let resumed_request = requests.last().expect("last query request");
    assert!(
        resumed_request.contains("id >= "),
        "resumed request must use an inclusive primary-key cursor: {resumed_request}"
    );
    assert!(
        resumed_request.contains("query_iter_last_pk"),
        "missing query_iter_last_pk: {resumed_request}"
    );
    assert!(
        resumed_request.contains("query_iter_last_element_offset"),
        "missing query_iter_last_element_offset: {resumed_request}"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_element_cursor_seek_requests_omit_element_params() {
    let server = MockServer::start().await;
    let filter = "element_filter(tags, tag == \"x\") and element_filter_query_iterator";
    // A fresh (non-resumed) element-filter iterator with an offset larger than one seek batch
    // performs multiple seek iterations. The offset seek uses `iterator=false`, and Milvus rejects
    // `query_iter_last_*` unless `iterator=true`, so seek requests must NOT carry the
    // element-resume params (only the later `iterator=true` data pages do).
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .offset(16_500)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let first = iterator
        .next()
        .await
        .expect("fetch first page")
        .expect("non-empty first page");
    assert!(!query_ids(&first).is_empty());

    // No `iterator=false` request may carry the element-resume params.
    let requests = server.service.request_texts("query");
    for request in requests
        .iter()
        .filter(|request| request.contains("key: \"iterator\", value: \"false\""))
    {
        assert!(
            !request.contains("query_iter_last_pk")
                && !request.contains("query_iter_last_element_offset"),
            "iterator=false seek request must not carry element-resume params: {request}"
        );
    }
    // The `iterator=true` data pages still carry them.
    assert!(
        requests.iter().any(|request| {
            request.contains("key: \"iterator\", value: \"true\"")
                && request.contains("query_iter_last_pk")
                && request.contains("query_iter_last_element_offset")
        }),
        "iterator=true data pages must carry element-resume params: {requests:?}"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_element_filter_does_not_stop_at_entity_batch_boundary() {
    let server = MockServer::start().await;
    let filter = "element_filter(tags, tag == \"x\") and multi_element_query_iterator";
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(4)
                .reduce_stop_for_best(false)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let mut ids = Vec::new();
    while let Some(page) = iterator.next().await.expect("fetch iterator page") {
        ids.extend(query_ids(&page));
    }
    // Each page returns 2 entities (4 elements) while batch_size is 4 entities; the iterator must
    // keep reading (judging exhaustion by element count) instead of stopping at the first page.
    assert_eq!(ids, [0, 1, 2, 3]);

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_element_filter_offset_seek_does_not_truncate() {
    let server = MockServer::start().await;
    let filter = "element_filter(tags, tag == \"x\") and multi_element_query_iterator";
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .offset(2)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let mut ids = Vec::new();
    while let Some(page) = iterator.next().await.expect("fetch iterator page") {
        ids.extend(query_ids(&page));
    }
    // The offset seek must not treat an element-capped page (fewer entities than the seek size)
    // as exhausted: the offset skips two entities and the remaining two are returned.
    assert_eq!(ids, [2, 3]);

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_element_cursor_resume_reenters_the_primary_key() {
    let server = MockServer::start().await;
    let filter = "element_filter(tags, tag == \"x\") and int64_reenter_element_query_iterator";
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let first = iterator
        .next()
        .await
        .expect("fetch first page")
        .expect("non-empty first page");
    assert_eq!(query_ids(&first), [0]);
    let cursor = iterator.cursor().expect("captured cursor");
    assert_eq!(cursor.get_pk(), &QueryCursorPk::Int64(0));
    assert_eq!(cursor.get_last_element_offset(), Some(1));

    // Resuming re-enters primary key 0 at element offset + 1 instead of skipping past it.
    let mut resumed = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(filter)
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .cursor(cursor)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create resumed query iterator");
    let resumed_page = resumed
        .next()
        .await
        .expect("fetch resumed page")
        .expect("non-empty resumed page");
    // The resumed page returns the cursor primary key again with its remaining elements.
    assert_eq!(query_ids(&resumed_page), [0]);
    assert_eq!(
        resumed
            .cursor()
            .expect("captured resumed cursor")
            .get_last_element_offset(),
        Some(3)
    );

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_var_char_cursor_resumes_from_quoted_predicate_and_raw_last_pk() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name("novels")
                .schema(
                    CollectionSchema::new().add_field(
                        FieldSchema::new()
                            .name("title")
                            .data_type(DataType::VarChar)
                            .primary_key(true)
                            .max_length(128),
                    ),
                )
                .build()
                .expect("valid collection request"),
        )
        .await
        .expect("create VarChar-PK collection");

    let filter = r#"element_filter(tags, tag == "x") and varchar_element_query_iterator"#;
    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("novels")
                        .filter(filter)
                        .output_fields(["title"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let first = iterator
        .next()
        .await
        .expect("fetch first page")
        .expect("non-empty first page");
    assert!(!query_var_char_titles(&first).is_empty());

    // The element-filter cursor carries a VarChar primary-key position and the element offset
    // within it, mirroring the Int64 cursor contract.
    let cursor = iterator.cursor().expect("captured VarChar element cursor");
    assert!(matches!(cursor.get_pk(), QueryCursorPk::VarChar(value) if value == "row-0"));
    assert_eq!(cursor.get_last_element_offset(), Some(0));

    let mut resumed = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("novels")
                        .filter(filter)
                        .output_fields(["title"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .cursor(cursor)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create resumed query iterator");
    let _ = resumed
        .next()
        .await
        .expect("fetch resumed page")
        .expect("non-empty resumed page");

    let requests = server.service.request_texts("query");
    let resumed_request = requests.last().expect("last query request");
    assert!(
        resumed_request.contains(r#"title >= \"row-0\""#),
        "resumed request must JSON-quote the VarChar cursor predicate: {resumed_request}"
    );
    assert!(
        resumed_request.contains(r#"key: "query_iter_last_pk", value: "row-0""#),
        "query_iter_last_pk must carry the raw VarChar string: {resumed_request}"
    );
    assert!(
        !resumed_request.contains(r#"key: "query_iter_last_pk", value: "\"row-0\"""#),
        "query_iter_last_pk must not carry the JSON-quoted form: {resumed_request}"
    );
    assert!(
        resumed_request.contains("query_iter_last_element_offset"),
        "missing query_iter_last_element_offset: {resumed_request}"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_plain_var_char_cursor_resumes_with_exclusive_predicate() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name("novels")
                .schema(
                    CollectionSchema::new().add_field(
                        FieldSchema::new()
                            .name("title")
                            .data_type(DataType::VarChar)
                            .primary_key(true)
                            .max_length(128),
                    ),
                )
                .build()
                .expect("valid collection request"),
        )
        .await
        .expect("create VarChar-PK collection");

    // A plain (non-element-filter) VarChar iterator: the exclusive `pk > "value"` predicate is
    // JSON-quoted, and resuming must not carry any element-resume params.
    let filter = "plain_varchar_query_iterator";
    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("novels")
                        .filter(filter)
                        .output_fields(["title"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let first = iterator
        .next()
        .await
        .expect("fetch first page")
        .expect("non-empty first page");
    assert!(!query_var_char_titles(&first).is_empty());
    let cursor = iterator.cursor().expect("captured VarChar cursor");
    assert!(matches!(cursor.get_pk(), QueryCursorPk::VarChar(value) if value == "row-0"));

    let mut resumed = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("novels")
                        .filter(filter)
                        .output_fields(["title"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .cursor(cursor)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create resumed query iterator");
    let _ = resumed
        .next()
        .await
        .expect("fetch resumed page")
        .expect("non-empty resumed page");

    let requests = server.service.request_texts("query");
    let resumed_request = requests.last().expect("last query request");
    assert!(
        resumed_request.contains(r#"title > \"row-0\""#),
        "resumed request must JSON-quote the exclusive VarChar cursor predicate: {resumed_request}"
    );
    assert!(
        !resumed_request.contains("query_iter_last_pk"),
        "plain VarChar resume must not carry element-resume params: {resumed_request}"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_rejects_negative_element_offset_cursor() {
    let server = MockServer::start().await;
    let error = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .cursor(QueryCursor::int64(300, 0).last_element_offset(-1))
                .build()
                .expect("valid request"),
        )
        .await
        .err()
        .expect("negative element offset must be rejected locally");
    assert!(error.to_string().contains("last_element_offset"));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_resume_with_session_ts_skips_the_probe() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("large_query_iterator")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(8_192)
                // A non-zero session_ts pins the MVCC snapshot from the original iterator and
                // makes the resumed iterator skip the server-side session probe entirely.
                .cursor(QueryCursor::int64(42_000, 8_191))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let mut row_count = 0;
    while let Some(page) = iterator.next().await.expect("fetch query iterator page") {
        row_count += page.results().get_output_fields()[0].len();
    }
    assert_eq!(row_count, 17_000 - 8_192);

    let requests = server.service.request_texts("query");
    // No probe request was sent — requests[0] is already the first data page.
    assert!(requests[0].contains("expr: \"id > 8191 and (large_query_iterator)\""));
    assert!(requests[1].contains("expr: \"id > 16383 and (large_query_iterator)\""));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_keeps_element_filter_last_after_cursor_predicate() {
    let server = MockServer::start().await;
    let original_filter = r#"element_filter(tags, tag == "element_filter_query_iterator")"#;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter(original_filter)
                        .output_fields(["id"])
                        .build()
                        .expect("valid element-filter query"),
                )
                .batch_size(1)
                .limit(3)
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create query iterator");

    let mut ids = Vec::new();
    while let Some(page) = iterator.next().await.expect("fetch iterator page") {
        ids.extend(query_ids(&page));
    }
    assert_eq!(ids, [0, 1, 2]);

    let requests = server.service.request_texts("query");
    assert_eq!(requests.len(), 4, "one probe and three data pages");
    for (request, cursor) in [(&requests[2], "id >= 0"), (&requests[3], "id >= 1")] {
        let cursor_position = request.find(cursor).expect("cursor predicate");
        let filter_position = request
            .find("element_filter(tags")
            .expect("original element filter");
        assert!(cursor_position < filter_position);
        assert!(request.contains(&format!("{cursor} and (element_filter")));
    }

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_caches_surplus_rows_and_commits_delivered_cursors() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("over_return_query_iterator")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    for expected in [[0, 1], [2, 3], [4, 5]] {
        let page = iterator
            .next()
            .await
            .expect("fetch cached iterator page")
            .expect("iterator page");
        assert_eq!(query_ids(&page), expected);
    }
    assert_eq!(
        server.service.call_count("query"),
        2,
        "timestamp probe plus one over-returned data request"
    );
    assert!(iterator
        .next()
        .await
        .expect("finish cached query iterator")
        .is_none());

    let requests = server.service.request_texts("query");
    assert!(requests[1].contains("KeyValuePair { key: \"limit\", value: \"2\" }"));
    assert!(requests
        .last()
        .unwrap()
        .contains("expr: \"id > 5 and (over_return_query_iterator)\""));
    assert!(requests[0].contains("output_fields: []"));
    assert!(requests[0].contains("partition_names: []"));
    assert!(requests
        .iter()
        .all(|request| request.contains("KeyValuePair { key: \"collection_id\", value: \"1\" }")));
    assert!(requests.iter().all(|request| request
        .contains("KeyValuePair { key: \"reduce_stop_for_best\", value: \"True\" }")));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_does_not_advance_cursor_when_decoding_fails() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("decode_failure_query_iterator")
                        .output_fields(["id", "invalid_json"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    assert!(iterator.next().await.is_err());
    assert!(iterator.next().await.is_err());

    let requests = server.service.request_texts("query");
    assert_eq!(requests.len(), 3);
    assert!(requests[1].contains("expr: \"decode_failure_query_iterator\""));
    assert!(requests[2].contains("expr: \"decode_failure_query_iterator\""));

    server.shutdown().await;
}

#[tokio::test]
async fn query_iterator_treats_negative_limit_as_unlimited() {
    let server = MockServer::start().await;
    let mut iterator = server
        .client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("unlimited_query_iterator")
                        .output_fields(["id"])
                        .limit(-1)
                        .build()
                        .expect("valid request"),
                )
                .batch_size(1)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create query iterator");

    let mut ids = Vec::new();
    while let Some(page) = iterator.next().await.expect("fetch unlimited iterator") {
        ids.extend(query_ids(&page));
    }
    assert_eq!(ids, [0, 1, 2]);

    server.shutdown().await;
}

fn query_ids(response: &milvus::v2::response::dql::QueryResponse) -> Vec<i64> {
    match response.results().get_output_field("id") {
        Some(FieldData::Int64 { values, .. }) => values.clone(),
        field => panic!("expected Int64 id field, got {field:?}"),
    }
}

fn query_var_char_titles(response: &milvus::v2::response::dql::QueryResponse) -> Vec<String> {
    match response.results().get_output_field("title") {
        Some(FieldData::VarChar { values, .. }) => values.clone(),
        field => panic!("expected VarChar title field, got {field:?}"),
    }
}

fn search_ids(response: &milvus::v2::response::dql::SearchResponse) -> Vec<i64> {
    match response.results().get_results()[0].get_ids() {
        milvus::v2::Ids::Int64(values) => values.clone(),
        ids => panic!("expected Int64 search IDs, got {ids:?}"),
    }
}

fn guarantee_timestamp(request: &str) -> u64 {
    request
        .split_once("guarantee_timestamp: ")
        .and_then(|(_, value)| {
            value
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .ok()
        })
        .expect("request contains a guarantee timestamp")
}

fn assert_hybrid_timestamp(timestamp: u64) {
    assert!(timestamp > 0);
    assert_eq!(timestamp & ((1_u64 << 18) - 1), 0);
}
