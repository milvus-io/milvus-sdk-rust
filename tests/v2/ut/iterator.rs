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
    // The process-wide schema cache is keyed by endpoint, and MockServer binds an ephemeral port
    // that the OS may reuse, so an earlier test can already hold the schema for this endpoint. The
    // get therefore primes the cache only when it is cold: at most one describe_collection.
    let primed = server.service.call_count("describe_collection");
    assert!(
        primed <= 1,
        "get must not issue more than one describe_collection, got {primed}"
    );

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
    // The V2 search iterator always issues exactly one direct describe_collection (uncached),
    // bypassing the schema cache even when it is already warm.
    assert_eq!(server.service.call_count("describe_collection"), primed + 1);

    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_filters_the_full_page_before_capping_to_the_limit() {
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
                        .filter("filter_external_iterator")
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(10)
                .limit(2)
                .external_filter_func(|result| {
                    let keep = result
                        .get_scores()
                        .iter()
                        .enumerate()
                        .filter_map(|(index, score)| (*score >= 0.8).then_some(index))
                        .collect::<Vec<_>>();
                    result.filter_rows(&keep)
                })
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator");

    // The server page holds five hits (scores 0.2, 0.3, 0.9, 0.85, 0.95). The filter keeps only
    // the three hits above 0.8, which sit past the limit window, so the iterator must decode and
    // filter the whole page before capping the returned rows to the limit of two.
    let page = iterator
        .next()
        .await
        .expect("fetch search page")
        .expect("page has rows");
    let rows = page.results().get_results()[0]
        .get_output_rows()
        .expect("materialize rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["id"], 3);
    assert_eq!(rows[1]["id"], 4);
    assert!(iterator.next().await.expect("finish iterator").is_none());

    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pulls_the_next_page_when_the_whole_page_is_pruned() {
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
                        .filter("filter_external_iterator")
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(10)
                .limit(5)
                .external_filter_func(|result| {
                    // Keep nothing: the whole first page is pruned, forcing the iterator to pull
                    // the next server page, which the mock serves empty.
                    result.filter_rows(&[])
                })
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator");
    assert!(matches!(iterator, SearchIterator::V2(_)));

    assert!(iterator.next().await.expect("finish iterator").is_none());

    // The cached real first page was pruned, then a token-bearing request pulled the next page.
    let requests = server.service.request_texts("search");
    assert!(requests
        .iter()
        .any(|request| request.contains("search_iter_id")));
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_accumulates_filtered_rows_across_server_pages() {
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
                        .filter("filter_accumulate_iterator")
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(2)
                .limit(4)
                .external_filter_func(|result| {
                    let keep = result
                        .get_scores()
                        .iter()
                        .enumerate()
                        .filter_map(|(index, score)| (*score >= 0.8).then_some(index))
                        .collect::<Vec<_>>();
                    result.filter_rows(&keep)
                })
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create search iterator");
    assert!(matches!(iterator, SearchIterator::V2(_)));

    // Each server page contributes one qualifying hit, so a single next() must accumulate rows
    // across pages until the batch (size 2) is filled: [1] from the first page plus [3] from the
    // second, mirroring the C++/pymilvus batching.
    let first = iterator
        .next()
        .await
        .expect("fetch first batch")
        .expect("first batch has rows");
    assert_eq!(search_ids(&first), [1, 3]);
    let second = iterator
        .next()
        .await
        .expect("fetch second batch")
        .expect("second batch has rows");
    assert_eq!(search_ids(&second), [3, 3]);
    assert!(iterator.next().await.expect("finish iterator").is_none());

    server.shutdown().await;
}

#[tokio::test]
async fn legacy_search_iterator_filters_initial_cache_and_expands_past_pruned_rows() {
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
                        .filter("legacy_filter_iterator")
                        .build()
                        .expect("valid search request"),
                )
                .batch_size(2)
                .limit(3)
                .external_filter_func(|result| {
                    let keep = result
                        .get_scores()
                        .iter()
                        .enumerate()
                        .filter_map(|(index, score)| (*score >= 0.85).then_some(index))
                        .collect::<Vec<_>>();
                    result.filter_rows(&keep)
                })
                .build()
                .expect("valid iterator request"),
        )
        .await
        .expect("create legacy search iterator");
    assert!(matches!(&iterator, SearchIterator::V1(_)));

    // The initial cached page (ids 1,2 with scores 0.4,0.3) is fully pruned by the filter, so
    // the iterator must keep pulling: the radius-driven expansion fetches the band with ids 3,4
    // (scores 0.9,0.7) and returns only the qualifying 0.9 row. Every returned page must only
    // contain qualifying rows; the static mock re-serves the same expansion page, so the
    // remaining limit fills with the same qualifying id.
    let mut pages = Vec::new();
    for _ in 0..5 {
        match iterator.next().await.expect("poll legacy iterator") {
            Some(page) => {
                assert_eq!(search_ids(&page), [3]);
                pages.push(());
            }
            None => break,
        }
    }
    assert!(!pages.is_empty());
    assert!(iterator
        .next()
        .await
        .expect("finish legacy iterator")
        .is_none());

    let requests = server.service.request_texts("search");
    assert!(requests.iter().any(|request| request.contains("radius")));
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
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("search_iter_v2"));
    assert!(!requests[1].contains("search_iter_v2"));
    assert!(!requests[0].contains("metric_type"));
    assert!(requests[1].contains("key: \"metric_type\", value: \"COSINE\""));
    assert!(requests[1].contains("range_filter"));
    assert!(requests[1].contains("radius"));
    assert!(requests[1].contains("id not in [2]"));
    assert_eq!(guarantee_timestamp(&requests[1]), 301);
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
        .create_database(
            CreateDatabaseRequest::builder()
                .database_name("tenant")
                .build()
                .expect("valid database request"),
        )
        .await
        .expect("create tenant database");
    client
        .use_database("tenant")
        .await
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
async fn query_iterator_fallback_timestamp_and_legacy_search_live_reads() {
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
    assert_eq!(search_requests.len(), 2);
    assert_eq!(guarantee_timestamp(&search_requests[0]), 0);
    assert_eq!(
        guarantee_timestamp(&search_requests[1]),
        0,
        "old V2 without session_ts preserves live semantics"
    );
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_reuses_partial_first_page_before_empty_reply() {
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

    assert!(iterator.next().await.unwrap().is_some());
    assert!(iterator.next().await.unwrap().is_none());
    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 2);
    assert_eq!(guarantee_timestamp(&requests[1]), 301);
    assert!(requests[1].contains("search_iter_id"));

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

fn cursor_page(
    ids: Vec<i64>,
    scores: Vec<f32>,
    version: Option<&str>,
    session_ts: u64,
) -> milvus::proto::milvus::SearchResults {
    use milvus::proto::{common, milvus as pb, schema};
    let mut extra_info = std::collections::HashMap::new();
    if let Some(version) = version {
        extra_info.insert("search_iter_cursor_version".into(), version.into());
        if let Some(last_pk) = ids.last() {
            extra_info.insert("search_iter_last_pk_type".into(), "int64".into());
            extra_info.insert("search_iter_last_pk".into(), last_pk.to_string());
        }
    }
    pb::SearchResults {
        status: Some(common::Status {
            extra_info,
            ..Default::default()
        }),
        session_ts,
        results: Some(schema::SearchResultData {
            num_queries: 1,
            top_k: ids.len() as i64,
            topks: vec![ids.len() as i64],
            search_iterator_v2_results: Some(schema::SearchIteratorV2Results {
                token: "4ea6247d-4b47-4e95-a65c-3bca62bbf7c1".into(),
                last_bound: scores.last().copied().unwrap_or(0.0),
            }),
            scores,
            ids: Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                    data: ids,
                })),
            }),
            primary_field_name: "id".into(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn cursor_request(batch: usize, limit: usize) -> SearchIteratorRequest {
    SearchIteratorRequest::builder()
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .metric_type(MetricType::Cosine)
                .extra_params(std::collections::HashMap::from([(
                    "search_iter_cursor_version".into(),
                    "2".into(),
                )]))
                .build()
                .unwrap(),
        )
        .batch_size(batch)
        .limit(limit)
        .build()
        .unwrap()
}

#[tokio::test]
async fn search_iterator_pk_cursor_caches_first_batch_and_preserves_snapshot() {
    let server = MockServer::start().await;
    server.service.queue_search_response(cursor_page(
        vec![i64::MIN, i64::MAX],
        vec![0.9, 0.8],
        Some("2"),
        301,
    ));
    let mut iterator = server
        .client
        .search_iterator(cursor_request(2, 3))
        .await
        .unwrap();
    assert_eq!(server.service.call_count("search"), 1);
    let first = iterator.next().await.unwrap().unwrap();
    assert_eq!(search_ids(&first), [i64::MIN, i64::MAX]);
    assert_eq!(
        server.service.call_count("search"),
        1,
        "first Next must use cached real batch"
    );
    server
        .service
        .queue_search_response(cursor_page(vec![3, 4], vec![0.7, 0.6], Some("2"), 0));
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [3]);
    assert!(iterator.next().await.unwrap().is_none());
    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("key: \"topk\", value: \"2\""));
    assert!(!requests[0].contains("key: \"topk\", value: \"1\""));
    assert_eq!(guarantee_timestamp(&requests[1]), 301);
    assert!(requests[1].contains("key: \"search_iter_last_pk\", value: \"9223372036854775807\""));
    assert!(requests[1].contains("key: \"search_iter_last_pk_type\", value: \"int64\""));
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pk_cursor_distinct_rows_fill_batch_and_limit() {
    let server = MockServer::start().await;
    for (ids, scores, ts) in [
        (vec![1, 1], vec![0.9, 0.8], 301),
        (vec![1, 2], vec![0.7, 0.6], 0),
        (vec![2, 3], vec![0.5, 0.4], 0),
    ] {
        server
            .service
            .queue_search_response(cursor_page(ids, scores, Some("2"), ts));
    }
    let mut iterator = server
        .client
        .search_iterator(cursor_request(2, 3))
        .await
        .unwrap();
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [1, 2]);
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [3]);
    assert!(iterator.next().await.unwrap().is_none());
    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 3);
    assert!(requests[1].contains("key: \"search_iter_last_bound\", value: \"0.8\""));
    assert!(requests[2].contains("key: \"search_iter_last_pk\", value: \"2\""));
    assert!(requests[2].contains("key: \"search_iter_last_bound\", value: \"0.6\""));
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pk_cursor_deduplicates_exact_varchar_keys() {
    use milvus::proto::schema;
    let server = MockServer::start().await;
    server
        .service
        .set_search_primary_key_type(schema::DataType::VarChar);
    let quoted = "quoted\"\\中文";
    for (ids, scores, ts) in [
        (vec!["", quoted], vec![0.9, 0.8], 301),
        (vec![quoted, "last"], vec![0.7, 0.6], 0),
    ] {
        let mut page = cursor_page(vec![1, 2], scores, Some("2"), ts);
        page.results.as_mut().unwrap().ids = Some(schema::IDs {
            id_field: Some(schema::i_ds::IdField::StrId(schema::StringArray {
                data: ids.iter().map(|id| (*id).to_owned()).collect(),
            })),
        });
        let extra = &mut page.status.as_mut().unwrap().extra_info;
        extra.insert("search_iter_last_pk_type".into(), "varchar".into());
        extra.insert(
            "search_iter_last_pk".into(),
            ids.last().unwrap().to_string(),
        );
        server.service.queue_search_response(page);
    }
    let mut iterator = server
        .client
        .search_iterator(cursor_request(2, 3))
        .await
        .unwrap();
    assert_eq!(
        iterator
            .next()
            .await
            .unwrap()
            .unwrap()
            .results()
            .get_results()[0]
            .get_ids(),
        &Ids::VarChar(vec!["".into(), quoted.into()])
    );
    assert_eq!(
        iterator
            .next()
            .await
            .unwrap()
            .unwrap()
            .results()
            .get_results()[0]
            .get_ids(),
        &Ids::VarChar(vec!["last".into()])
    );
    assert!(iterator.next().await.unwrap().is_none());
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pk_cursor_filter_failure_keeps_pending_raw_page_and_seen_state() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let server = MockServer::start().await;
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 301));
    let calls = Arc::new(AtomicUsize::new(0));
    let filter_calls = calls.clone();
    let request = cursor_request(2, 2)
        .into_builder()
        .external_filter_func(move |result| {
            if filter_calls.fetch_add(1, Ordering::SeqCst) == 1 {
                result.filter_rows(&[])?;
                return Err(Error::MalformedResponse("filter failed".into()));
            }
            Ok(())
        })
        .build()
        .unwrap();
    let mut iterator = server.client.search_iterator(request).await.unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.8], Some("2"), 0));
    assert!(iterator.next().await.is_err());
    assert_eq!(server.service.call_count("search"), 2);
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [1, 2]);
    assert_eq!(
        server.service.call_count("search"),
        2,
        "filter retries decode the pending response"
    );
    assert!(iterator.next().await.unwrap().is_none());
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_distance_mode_keeps_duplicate_primary_keys() {
    let server = MockServer::start().await;
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], None, 301));
    let mut iterator = server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .unwrap();
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [1]);
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.8], None, 0));
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [1]);
    assert!(iterator.next().await.unwrap().is_none());
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pk_cursor_preserves_raw_quoted_varchar() {
    use milvus::proto::schema;
    let server = MockServer::start().await;
    server
        .service
        .set_search_primary_key_type(schema::DataType::VarChar);
    let last_pk = "quoted\"id\\next\n中文";
    let mut first = cursor_page(vec![1], vec![0.9], Some("2"), 301);
    first.results.as_mut().unwrap().ids = Some(schema::IDs {
        id_field: Some(schema::i_ds::IdField::StrId(schema::StringArray {
            data: vec![last_pk.into()],
        })),
    });
    let extra = &mut first.status.as_mut().unwrap().extra_info;
    extra.insert("search_iter_last_pk_type".into(), "varchar".into());
    extra.insert("search_iter_last_pk".into(), last_pk.into());
    server.service.queue_search_response(first);
    let mut iterator = server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .unwrap();
    assert!(iterator.next().await.unwrap().is_some());
    let mut empty = cursor_page(vec![], vec![], Some("2"), 0);
    empty.results.as_mut().unwrap().ids = Some(schema::IDs::default());
    server.service.queue_search_response(empty);
    assert!(iterator.next().await.unwrap().is_none());
    assert!(iterator.next().await.unwrap().is_none());
    let request = server.service.request_text("search");
    assert!(request.contains(&format!("key: \"search_iter_last_pk\", value: {last_pk:?}")));
    assert!(request.contains("key: \"search_iter_last_pk_type\", value: \"varchar\""));
    assert_eq!(server.service.call_count("search"), 2);
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pk_cursor_errors_do_not_advance_page() {
    let server = MockServer::start().await;
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 301));
    let mut iterator = server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .unwrap();
    assert!(iterator.next().await.unwrap().is_some());
    let mut wrong_pk = cursor_page(vec![2], vec![0.8], Some("2"), 0);
    wrong_pk
        .status
        .as_mut()
        .unwrap()
        .extra_info
        .insert("search_iter_last_pk".into(), "3".into());
    let mut bad_decode = cursor_page(vec![2], vec![0.8], Some("2"), 0);
    bad_decode.results.as_mut().unwrap().fields_data = vec![milvus::proto::schema::FieldData {
        r#type: milvus::proto::schema::DataType::Json as i32,
        field_name: "bad_json".into(),
        field: Some(milvus::proto::schema::field_data::Field::Scalars(
            milvus::proto::schema::ScalarField {
                data: Some(milvus::proto::schema::scalar_field::Data::JsonData(
                    milvus::proto::schema::JsonArray {
                        data: vec![b"not-json".to_vec()],
                    },
                )),
                ..Default::default()
            },
        )),
        ..Default::default()
    }];
    for response in [
        cursor_page(vec![2], vec![0.8], None, 0),
        cursor_page(vec![2], vec![0.8], Some("3"), 0),
        wrong_pk,
        bad_decode,
    ] {
        server.service.queue_search_response(response);
        assert!(iterator.next().await.is_err());
    }
    server
        .service
        .fail_next_transport("search", tonic::Code::Cancelled);
    assert!(iterator.next().await.is_err());
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.8], Some("2"), 0));
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [2]);
    let requests = server.service.request_texts("search");
    assert!(
        requests.len() >= 7,
        "centralized transport retry may replay a request"
    );
    for request in &requests[1..] {
        assert_eq!(
            request, &requests[1],
            "error must retry identical cursor and snapshot"
        );
    }
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_distance_mode_is_latched_and_empty_first_page_finishes() {
    let server = MockServer::start().await;
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], None, 301));
    let mut iterator = server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .unwrap();
    iterator.next().await.unwrap().unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![], vec![], None, 0));
    assert!(iterator.next().await.unwrap().is_none());
    let request = server.service.request_text("search");
    assert!(!request.contains("search_iter_cursor_version"));
    assert!(!request.contains("search_iter_last_pk"));
    assert_eq!(guarantee_timestamp(&request), 301);
    server
        .service
        .queue_search_response(cursor_page(vec![], vec![], Some("2"), 301));
    let mut empty = server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .unwrap();
    let calls = server.service.call_count("search");
    assert!(empty.next().await.unwrap().is_none());
    assert!(empty.next().await.unwrap().is_none());
    assert_eq!(server.service.call_count("search"), calls);
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 0));
    assert!(server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .is_err());
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_search_iterator_counts_hits_and_preserves_three_page_snapshot() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let mut iterator = client
        .search_iterator(
            "books",
            vec![vec![0.1_f32, 0.2].into()],
            milvus::v1::iterator::SearchIteratorOptions::default()
                .add_search_param("search_iter_cursor_version".into(), "2".into())
                .batch_size(2)
                .limit(5),
        )
        .await
        .unwrap();
    for (ids, scores, ts) in [
        (vec![1, 2], vec![0.9, 0.8], 301),
        (vec![3, 4], vec![0.7, 0.6], 0),
        (vec![5], vec![0.5], 0),
    ] {
        server
            .service
            .queue_search_response(cursor_page(ids.clone(), scores, Some("2"), ts));
        let result = iterator.next().await.unwrap().unwrap();
        assert_eq!(
            result.iter().map(|query| query.size).sum::<i64>(),
            ids.len() as i64
        );
    }
    assert!(iterator.next().await.unwrap().is_none());
    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 3);
    assert_eq!(guarantee_timestamp(&requests[0]), 0);
    assert_eq!(guarantee_timestamp(&requests[1]), 301);
    assert_eq!(guarantee_timestamp(&requests[2]), 301);
    assert!(requests[2].contains("key: \"search_iter_batch_size\", value: \"1\""));
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_search_iterator_rejects_missing_v2_and_retains_cursor_on_error() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let mut iterator = client
        .search_iterator(
            "books",
            vec![vec![0.1_f32, 0.2].into()],
            milvus::v1::iterator::SearchIteratorOptions::default()
                .add_search_param("search_iter_cursor_version".into(), "2".into())
                .batch_size(1)
                .limit(2),
        )
        .await
        .unwrap();
    let mut unsupported = cursor_page(vec![1], vec![0.9], None, 301);
    unsupported
        .results
        .as_mut()
        .unwrap()
        .search_iterator_v2_results = None;
    server.service.queue_search_response(unsupported);
    assert!(iterator
        .next()
        .await
        .unwrap_err()
        .to_string()
        .contains("does not support Search Iterator V2"));
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 301));
    iterator.next().await.unwrap().unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.8], None, 0));
    assert!(iterator.next().await.is_err());
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.8], Some("2"), 0));
    iterator.next().await.unwrap().unwrap();
    assert!(iterator.next().await.unwrap().is_none());
    let requests = server.service.request_texts("search");
    assert_eq!(requests[2], requests[3]);
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_pk_checkpoint_restores_snapshot_bound_and_consumed_hits() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let checkpoint =
        std::env::temp_dir().join(format!("rust-search-iterator-{}.cp", uuid::Uuid::new_v4()));
    let options = milvus::v1::iterator::SearchIteratorOptions::default()
        .add_search_param("search_iter_cursor_version".into(), "2".into())
        .batch_size(2)
        .limit(4)
        .iterator_cp_file(Some(checkpoint.to_string_lossy().into_owned()));
    let mut original = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options.clone())
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1, 2], vec![0.9, 0.8], Some("2"), 301));
    original.next().await.unwrap().unwrap();
    original.close();
    let checkpoint_text = std::fs::read_to_string(&checkpoint).unwrap();
    assert!(checkpoint_text.starts_with("301\n"));
    assert!(checkpoint_text.contains("\"format_version\":2"));
    // The previous SDK reader parsed exactly the first and last lines.
    let old_reader_lines: Vec<_> = checkpoint_text.lines().collect();
    assert_eq!(old_reader_lines.len(), 3);
    let old_snapshot = old_reader_lines[0].parse::<u64>().unwrap();
    let old_token = old_reader_lines[old_reader_lines.len() - 1];
    assert_eq!(old_snapshot, 301);
    assert_eq!(old_token, "4ea6247d-4b47-4e95-a65c-3bca62bbf7c1");
    assert!(uuid::Uuid::parse_str(old_token).is_ok());
    let mut resumed_options = options;
    resumed_options
        .search_params
        .remove("search_iter_cursor_version");
    let mut resumed = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], resumed_options)
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![3, 4], vec![0.7, 0.6], Some("2"), 0));
    resumed.next().await.unwrap().unwrap();
    assert_eq!(resumed.returned_count(), 4);
    assert!(resumed.next().await.unwrap().is_none());
    let request = server.service.request_text("search");
    assert_eq!(guarantee_timestamp(&request), 301);
    assert!(request.contains("key: \"search_iter_last_pk\", value: \"2\""));
    assert!(request.contains("key: \"search_iter_last_bound\", value: \"0.8\""));
    resumed.close();
    std::fs::remove_file(checkpoint).unwrap();
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_pk_checkpoint_preserves_distinct_keys_across_resume_and_duplicate_only_pages() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let checkpoint = std::env::temp_dir().join(format!("rust-dedup-{}.cp", uuid::Uuid::new_v4()));
    let options = milvus::v1::iterator::SearchIteratorOptions::default()
        .add_search_param("search_iter_cursor_version".into(), "2".into())
        .batch_size(1)
        .limit(2)
        .iterator_cp_file(Some(checkpoint.to_string_lossy().into_owned()));
    let mut first = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options.clone())
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 301));
    assert_eq!(first.next().await.unwrap().unwrap()[0].size, 1);
    first.close();
    let saved: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(&checkpoint)
            .unwrap()
            .lines()
            .nth(1)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(saved["accepted_pks"], serde_json::json!(["1"]));
    let original_checkpoint = std::fs::read_to_string(&checkpoint).unwrap();
    for missing in [false, true] {
        let mut invalid = saved.clone();
        if missing {
            invalid.as_object_mut().unwrap().remove("accepted_pks");
        } else {
            invalid["accepted_pks"] = serde_json::json!([]);
        }
        std::fs::write(
            &checkpoint,
            format!("301\n{invalid}\n4ea6247d-4b47-4e95-a65c-3bca62bbf7c1\n"),
        )
        .unwrap();
        let calls = server.service.call_count("search");
        let mut rejected = client
            .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options.clone())
            .await
            .unwrap();
        assert!(rejected.next().await.is_err());
        assert_eq!(
            server.service.call_count("search"),
            calls,
            "invalid accepted state must fail before Search"
        );
        rejected.close();
    }
    std::fs::write(&checkpoint, original_checkpoint).unwrap();
    let mut resumed = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options)
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.8], Some("2"), 0));
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.7], Some("2"), 0));
    let next = resumed.next().await.unwrap().unwrap();
    assert!(matches!(next[0].id[0], milvus::v1::value::Value::Long(2)));
    assert_eq!(next[0].score, [0.7]);
    assert_eq!(resumed.returned_count(), 2);
    assert!(resumed.next().await.unwrap().is_none());
    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 3);
    assert!(requests[2].contains("key: \"search_iter_last_bound\", value: \"0.8\""));
    resumed.close();
    std::fs::remove_file(checkpoint).unwrap();
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_pk_checkpoint_rejects_wrong_search_or_recreated_collection_before_search() {
    use milvus::proto::schema;

    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let checkpoint =
        std::env::temp_dir().join(format!("rust-search-identity-{}.cp", uuid::Uuid::new_v4()));
    let options = milvus::v1::iterator::SearchIteratorOptions::default()
        .add_search_param("search_iter_cursor_version".into(), "2".into())
        .add_search_param("metric_type".into(), "COSINE".into())
        .add_search_param(
            "params".into(),
            r#"{"radius":0.95,"range_filter":0.1,"nested":{"a":1,"b":2},"limit":50,"topk":7}"#
                .into(),
        )
        .filter("id > {limit}".into())
        .add_template_value(
            "limit".into(),
            schema::TemplateValue {
                val: Some(schema::template_value::Val::Int64Val(0)),
            },
        )
        .anns_field("vector".into())
        .batch_size(2)
        .iterator_cp_file(Some(checkpoint.to_string_lossy().into_owned()));
    let mut original = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options.clone())
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1, 2], vec![0.9, 0.8], Some("2"), 301));
    original.next().await.unwrap().unwrap();
    original.close();
    let before = std::fs::read_to_string(&checkpoint).unwrap();
    let saved: serde_json::Value = serde_json::from_str(before.lines().nth(1).unwrap()).unwrap();
    assert_eq!(saved["collection_id"], 1);
    assert_eq!(saved["request_fingerprint"].as_str().unwrap().len(), 64);
    let wrong_options = [
        options.clone().filter("id > 10".into()),
        options.clone().batch_size(1),
        options.clone().anns_field("other_vector".into()),
        options.clone().add_template_value(
            "limit".into(),
            schema::TemplateValue {
                val: Some(schema::template_value::Val::Int64Val(10)),
            },
        ),
        options.clone().add_search_param(
            "params".into(),
            r#"{"radius":0.95,"range_filter":0.1,"nested":{"a":1,"b":2},"limit":51,"topk":7}"#
                .into(),
        ),
        options.clone().add_search_param(
            "params".into(),
            r#"{"radius":0.8,"range_filter":0.1,"nested":{"a":1,"b":2},"limit":50,"topk":7}"#
                .into(),
        ),
    ];
    for wrong in wrong_options {
        let mut resumed = client
            .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], wrong)
            .await
            .unwrap();
        let calls = server.service.call_count("search");
        let error = resumed.next().await.unwrap_err().to_string();
        assert!(error.contains("search definition"), "{error}");
        assert_eq!(server.service.call_count("search"), calls);
        assert_eq!(std::fs::read_to_string(&checkpoint).unwrap(), before);
        resumed.close();
    }
    let mut wrong_vector = client
        .search_iterator("books", vec![vec![0.4_f32, 0.2].into()], options.clone())
        .await
        .unwrap();
    let calls = server.service.call_count("search");
    assert!(wrong_vector
        .next()
        .await
        .unwrap_err()
        .to_string()
        .contains("search definition"));
    assert_eq!(server.service.call_count("search"), calls);
    assert_eq!(std::fs::read_to_string(&checkpoint).unwrap(), before);
    wrong_vector.close();
    let mut wrong_snapshot = client
        .search_iterator(
            "books",
            vec![vec![0.1_f32, 0.2].into()],
            options.clone().guarantee_timestamp(302),
        )
        .await
        .unwrap();
    assert!(wrong_snapshot
        .next()
        .await
        .unwrap_err()
        .to_string()
        .contains("snapshot"));
    assert_eq!(server.service.call_count("search"), calls);
    assert_eq!(std::fs::read_to_string(&checkpoint).unwrap(), before);
    wrong_snapshot.close();
    server.service.set_search_collection_id(2);
    let mut recreated = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options.clone())
        .await
        .unwrap();
    assert!(recreated
        .next()
        .await
        .unwrap_err()
        .to_string()
        .contains("collection ID"));
    assert_eq!(server.service.call_count("search"), calls);
    assert_eq!(std::fs::read_to_string(&checkpoint).unwrap(), before);
    recreated.close();
    server.service.set_search_collection_id(1);
    let mut reordered = options.guarantee_timestamp(301).add_search_param(
        "params".into(),
        r#"{"topk":7,"limit":50,"nested":{"b":2,"a":1},"range_filter":0.1,"radius":0.95}"#.into(),
    );
    reordered.search_params.remove("search_iter_cursor_version");
    let mut matching = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], reordered)
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![3], vec![0.7], Some("2"), 0));
    matching.next().await.unwrap().unwrap();
    assert_eq!(matching.returned_count(), 3);
    assert_eq!(
        guarantee_timestamp(&server.service.request_text("search")),
        301
    );
    matching.close();
    std::fs::remove_file(checkpoint).unwrap();
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_pk_cursor_preserves_explicit_snapshot_when_response_omits_it() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let mut iterator = client
        .search_iterator(
            "books",
            vec![vec![0.1_f32, 0.2].into()],
            milvus::v1::iterator::SearchIteratorOptions::default()
                .add_search_param("search_iter_cursor_version".into(), "2".into())
                .batch_size(1)
                .limit(1)
                .guarantee_timestamp(444),
        )
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 0));
    iterator.next().await.unwrap().unwrap();
    assert_eq!(
        guarantee_timestamp(&server.service.request_text("search")),
        444
    );
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_tiny_distance_cursor_round_trips() {
    let server = MockServer::start().await;
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![1.0e-30], Some("2"), 301));
    let mut iterator = server
        .client
        .search_iterator(cursor_request(1, 2))
        .await
        .unwrap();
    iterator.next().await.unwrap().unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![], vec![], Some("2"), 0));
    assert!(iterator.next().await.unwrap().is_none());
    let request = server.service.request_text("search");
    assert!(request.contains(&format!(
        "key: \"search_iter_last_bound\", value: {:?}",
        1.0e-30_f32.to_string()
    )));
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_old_v2_latches_distance_mode() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let mut iterator = client
        .search_iterator(
            "books",
            vec![vec![0.1_f32, 0.2].into()],
            milvus::v1::iterator::SearchIteratorOptions::default()
                .batch_size(1)
                .limit(2),
        )
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], None, 301));
    iterator.next().await.unwrap().unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.8], None, 0));
    iterator.next().await.unwrap().unwrap();
    assert!(iterator.next().await.unwrap().is_none());
    let request = server.service.request_text("search");
    assert_eq!(guarantee_timestamp(&request), 301);
    assert!(!request.contains("search_iter_cursor_version"));
    assert!(!request.contains("search_iter_last_pk"));
    assert!(request.contains("search_iter_last_bound"));
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_pk_checkpoint_write_failure_does_not_advance_cursor() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let checkpoint =
        std::env::temp_dir().join(format!("rust-search-iterator-{}.cp", uuid::Uuid::new_v4()));
    std::fs::create_dir(&checkpoint).unwrap();
    let mut iterator = client
        .search_iterator(
            "books",
            vec![vec![0.1_f32, 0.2].into()],
            milvus::v1::iterator::SearchIteratorOptions::default()
                .add_search_param("search_iter_cursor_version".into(), "2".into())
                .batch_size(1)
                .limit(1)
                .iterator_cp_file(Some(checkpoint.to_string_lossy().into_owned())),
        )
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 301));
    assert!(iterator.next().await.is_err());
    assert_eq!(iterator.returned_count(), 0);
    std::fs::remove_dir(&checkpoint).unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1], vec![0.9], Some("2"), 301));
    iterator.next().await.unwrap().unwrap();
    let requests = server.service.request_texts("search");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0], requests[1]);
    std::fs::remove_file(checkpoint).unwrap();
    server.shutdown().await;
}

#[tokio::test]
async fn manual_distance_cursor_continuation_does_not_opt_in_to_pk_mode() {
    let server = MockServer::start().await;
    let token = "4ea6247d-4b47-4e95-a65c-3bca62bbf7c1";
    let request = SearchIteratorRequest::builder()
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .metric_type(MetricType::Cosine)
                .extra_params(std::collections::HashMap::from([
                    ("search_iter_id".into(), token.into()),
                    ("search_iter_last_bound".into(), "0.5".into()),
                ]))
                .build()
                .unwrap(),
        )
        .batch_size(1)
        .limit(1)
        .build()
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.7], None, 301));
    let mut iterator = server.client.search_iterator(request).await.unwrap();
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [2]);
    let wire = server.service.request_text("search");
    assert!(wire.contains("search_iter_id"));
    assert!(wire.contains("key: \"search_iter_last_bound\", value: \"0.5\""));
    assert!(!wire.contains("search_iter_cursor_version"));
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let options = milvus::v1::iterator::SearchIteratorOptions::default()
        .batch_size(1)
        .limit(1)
        .add_search_param("search_iter_id".into(), token.into())
        .add_search_param("search_iter_last_bound".into(), "0.5".into());
    let mut legacy = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options)
        .await
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![2], vec![0.7], None, 301));
    legacy.next().await.unwrap().unwrap();
    let wire = server.service.request_text("search");
    assert!(wire.contains("key: \"search_iter_last_bound\", value: \"0.5\""));
    assert!(!wire.contains("search_iter_cursor_version"));
    server.shutdown().await;
}

#[tokio::test]
async fn explicit_pk_opt_in_rejects_partial_manual_distance_cursor_before_search() {
    let server = MockServer::start().await;
    let token = "4ea6247d-4b47-4e95-a65c-3bca62bbf7c1";
    let request = SearchIteratorRequest::builder()
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .extra_params(std::collections::HashMap::from([
                    ("search_iter_cursor_version".into(), "2".into()),
                    ("search_iter_id".into(), token.into()),
                    ("search_iter_last_bound".into(), "0.5".into()),
                ]))
                .build()
                .unwrap(),
        )
        .batch_size(1)
        .build()
        .unwrap();
    assert!(server.client.search_iterator(request).await.is_err());
    assert_eq!(server.service.call_count("search"), 0);
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    let options = milvus::v1::iterator::SearchIteratorOptions::default()
        .add_search_param("search_iter_cursor_version".into(), "2".into())
        .add_search_param("search_iter_id".into(), token.into())
        .add_search_param("search_iter_last_bound".into(), "0.5".into());
    let mut legacy = client
        .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options)
        .await
        .unwrap();
    assert!(legacy.next().await.is_err());
    assert_eq!(server.service.call_count("search"), 0);
    server.shutdown().await;
}

#[tokio::test]
async fn legacy_api_latches_declined_pk_mode_and_replaces_manual_distance_cursor() {
    let server = MockServer::start().await;
    let client = milvus::v1::client::Client::new(server.uri.clone())
        .await
        .unwrap();
    for manual in [false, true] {
        let mut options = milvus::v1::iterator::SearchIteratorOptions::default()
            .batch_size(1)
            .limit(2)
            // These dynamic controls are iterator-owned despite generic options.
            .add_search_param("topk".into(), "100".into())
            .add_search_param("search_iter_batch_size".into(), "100".into());
        if manual {
            options = options
                .add_search_param(
                    "search_iter_id".into(),
                    "4ea6247d-4b47-4e95-a65c-3bca62bbf7c1".into(),
                )
                .add_search_param("search_iter_last_bound".into(), "0.5".into());
        } else {
            options = options.add_search_param("search_iter_cursor_version".into(), "2".into());
        }
        let mut iterator = client
            .search_iterator("books", vec![vec![0.1_f32, 0.2].into()], options)
            .await
            .unwrap();
        server
            .service
            .queue_search_response(cursor_page(vec![1], vec![0.9], None, 301));
        iterator.next().await.unwrap().unwrap();
        let first = server.service.request_text("search");
        assert_eq!(first.contains("search_iter_cursor_version"), !manual);
        assert!(first.contains("key: \"topk\", value: \"1\""));
        assert!(!first.contains("value: \"100\""));
        server
            .service
            .queue_search_response(cursor_page(vec![2], vec![0.8], None, 0));
        iterator.next().await.unwrap().unwrap();
        let second = server.service.request_text("search");
        assert_eq!(guarantee_timestamp(&second), 301);
        assert!(!second.contains("search_iter_cursor_version"));
        assert_eq!(second.matches("key: \"search_iter_last_bound\"").count(), 1);
        assert!(second.contains("key: \"search_iter_last_bound\", value: \"0.9\""));
        assert!(!second.contains("value: \"0.5\""));
        let calls = server.service.call_count("search");
        assert!(iterator.next().await.unwrap().is_none());
        assert_eq!(server.service.call_count("search"), calls);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn empty_v2_token_without_pk_marker_uses_real_legacy_first_page() {
    let server = MockServer::start().await;
    let mut reply = cursor_page(vec![1], vec![0.9], None, 301);
    reply
        .results
        .as_mut()
        .unwrap()
        .search_iterator_v2_results
        .as_mut()
        .unwrap()
        .token
        .clear();
    server.service.queue_search_response(reply);
    let mut iterator = server
        .client
        .search_iterator(cursor_request(1, 1))
        .await
        .unwrap();
    assert!(matches!(iterator, SearchIterator::V1(_)));
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [1]);
    assert_eq!(server.service.call_count("search"), 1);
    let mut bad = cursor_page(vec![1], vec![0.9], Some("2"), 301);
    bad.results
        .as_mut()
        .unwrap()
        .search_iterator_v2_results
        .as_mut()
        .unwrap()
        .token
        .clear();
    server.service.queue_search_response(bad);
    assert!(server
        .client
        .search_iterator(cursor_request(1, 1))
        .await
        .is_err());
    server.shutdown().await;
}

#[tokio::test]
async fn search_iterator_pk_cursor_requires_explicit_opt_in() {
    let server = MockServer::start().await;
    let request = SearchIteratorRequest::builder()
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .metric_type(MetricType::Cosine)
                .build()
                .unwrap(),
        )
        .batch_size(2)
        .limit(2)
        .build()
        .unwrap();
    server
        .service
        .queue_search_response(cursor_page(vec![1, 2], vec![0.9, 0.8], None, 301));
    let mut iterator = server
        .client
        .search_iterator(request.clone())
        .await
        .unwrap();
    assert_eq!(search_ids(&iterator.next().await.unwrap().unwrap()), [1, 2]);
    assert_eq!(server.service.call_count("search"), 1);
    let wire = server.service.request_text("search");
    assert!(!wire.contains("search_iter_cursor_version"));
    assert!(wire.contains("key: \"topk\", value: \"2\""));
    server
        .service
        .queue_search_response(cursor_page(vec![1, 2], vec![0.9, 0.8], Some("2"), 301));
    assert!(server.client.search_iterator(request).await.is_err());
    let unsupported = SearchIteratorRequest::builder()
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .metric_type(MetricType::Cosine)
                .extra_params(std::collections::HashMap::from([(
                    "search_iter_cursor_version".into(),
                    "3".into(),
                )]))
                .build()
                .unwrap(),
        )
        .batch_size(1)
        .build()
        .unwrap();
    let calls = server.service.call_count("search");
    assert!(server.client.search_iterator(unsupported).await.is_err());
    assert_eq!(server.service.call_count("search"), calls);
    server.shutdown().await;
}
