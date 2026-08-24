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
use milvus::v2::request::dql::*;
use milvus::v2::SearchVectors;

#[tokio::test]
async fn session_routes_dql_requests_to_the_target_cluster() {
    let server = MockServer::start().await;
    let session = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");
    assert_eq!(session.cluster_id(), "cluster-a");

    session
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .output_fields(["id"])
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    session
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .output_fields(["id"])
                .limit(3)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    session
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name("books")
                .sub_requests(vec![SubSearchRequest::builder()
                    .vector_field("vector")
                    .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                    .build()
                    .expect("valid request")])
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    session
        .get(
            GetRequest::builder()
                .collection_name("books")
                .ids(Ids::Int64(vec![1]))
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    // `session.query` is the first "query" RPC captured; `session.get` also issues a "query" RPC
    // (get_with_cluster), so assert on the first query request to target the session.query call.
    let query_requests = server.service.request_texts("query");
    assert!(
        query_requests
            .first()
            .is_some_and(|request| request.contains(r#"key: "cluster_id", value: "cluster-a""#)),
        "the session.query request must carry the cluster_id: {:?}",
        query_requests
    );
    server.assert_request_contains("search", &["search_params", "cluster_id", "cluster-a"]);
    server.assert_request_contains("hybrid_search", &["rank_params", "cluster_id", "cluster-a"]);
    for request in query_requests {
        assert!(
            request.contains("cluster_id"),
            "every captured query request must carry a cluster id: {request}"
        );
    }

    for rpc in ["query", "search", "hybrid_search"] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn session_requests_never_carry_another_sessions_cluster_id() {
    let server = MockServer::start().await;
    let session_a = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");
    let session_b = server
        .client
        .session("cluster-b")
        .expect("valid cluster id");

    session_a
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    session_b
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    let query_requests = server.service.request_texts("query");
    assert_eq!(query_requests.len(), 2, "two query requests captured");
    assert!(
        query_requests[0].contains(r#"key: "cluster_id", value: "cluster-a""#),
        "first request must route to cluster-a: {}",
        query_requests[0]
    );
    assert!(
        query_requests[1].contains(r#"key: "cluster_id", value: "cluster-b""#),
        "second request must route to cluster-b: {}",
        query_requests[1]
    );
    assert!(
        !query_requests[0].contains("cluster-b") && !query_requests[1].contains("cluster-a"),
        "each request must only carry its own session's cluster id: {:?}",
        query_requests
    );
    server.shutdown().await;
}

#[tokio::test]
async fn session_iterators_route_dql_requests_to_the_target_cluster() {
    let server = MockServer::start().await;
    let session = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");

    let mut query_iterator = session
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("id >= 0")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query iterator");
    while let Some(_rows) = query_iterator.next().await.expect("iterator page") {}

    let mut search_iterator = session
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .build()
                        .expect("valid request"),
                )
                .batch_size(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search iterator");
    while let Some(_rows) = search_iterator.next().await.expect("iterator page") {}

    for request in server.service.request_texts("query") {
        assert!(
            request.contains(r#"key: "cluster_id", value: "cluster-a""#),
            "every query iterator request must carry the session cluster id: {request}"
        );
    }
    for request in server.service.request_texts("search") {
        assert!(
            request.contains(r#"key: "cluster_id", value: "cluster-a""#),
            "every search iterator request must carry the session cluster id: {request}"
        );
    }
    server.shutdown().await;
}

#[tokio::test]
async fn session_rejects_an_empty_cluster_id() {
    let server = MockServer::start().await;
    let result = server.client.session("");
    assert!(matches!(result, Err(Error::Validation(_))));
    server.shutdown().await;
}

#[tokio::test]
async fn plain_client_requests_do_not_carry_a_cluster_id() {
    let server = MockServer::start().await;
    server
        .client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let request = server.service.request_text("query");
    assert!(!request.contains("cluster_id"));
    server.shutdown().await;
}

#[tokio::test]
async fn plain_client_requests_keep_a_user_supplied_cluster_param() {
    let server = MockServer::start().await;
    let mut params = std::collections::HashMap::new();
    params.insert("cluster_id".to_owned(), "user-routed".to_owned());
    server
        .client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .extra_params(params)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let request = server.service.request_text("query");
    assert!(
        request.contains(r#"key: "cluster_id", value: "user-routed""#),
        "plain request must preserve the user-supplied cluster_id: {request}"
    );
    server.shutdown().await;
}

#[tokio::test]
async fn closed_session_rejects_further_calls_even_from_clones() {
    let server = MockServer::start().await;
    let session = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");
    let clone = session.clone();

    session.close();
    for closed in [&session, &clone] {
        let error = closed
            .query(
                QueryRequest::builder()
                    .collection_name("books")
                    .filter("id > 0")
                    .build()
                    .expect("valid request"),
            )
            .await
            .expect_err("closed session must reject queries");
        assert!(matches!(
            error,
            Error::Unexpected(message) if message.contains("session is closed")
        ));
    }
    server.shutdown().await;
}

#[tokio::test]
async fn session_cluster_id_overwrites_a_user_supplied_cluster_param() {
    let server = MockServer::start().await;
    let session = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");

    let mut params = std::collections::HashMap::new();
    params.insert("cluster_id".to_owned(), "user-specified".to_owned());
    session
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .extra_params(params)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    let request = server.service.request_text("query");
    assert_eq!(
        request.matches(r#"key: "cluster_id""#).count(),
        1,
        "request must contain exactly one cluster_id param: {request}"
    );
    assert!(
        request.contains(r#"key: "cluster_id", value: "cluster-a""#),
        "session cluster id must win: {request}"
    );
    assert!(
        !request.contains("user-specified"),
        "user cluster id must be erased: {request}"
    );
    server.shutdown().await;
}

#[tokio::test]
async fn closing_a_session_stops_iterators_created_before_the_close() {
    let server = MockServer::start().await;
    let session = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");

    let mut query_iterator = session
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(
                    QueryRequest::builder()
                        .collection_name("books")
                        .filter("id >= 0")
                        .output_fields(["id"])
                        .build()
                        .expect("valid request"),
                )
                .batch_size(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query iterator");

    let mut search_iterator = session
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(
                    SearchRequest::builder()
                        .collection_name("books")
                        .vector_field("vector")
                        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                        .build()
                        .expect("valid request"),
                )
                .batch_size(2)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search iterator");

    session.close();

    let error = query_iterator
        .next()
        .await
        .expect_err("closed session query iterator");
    assert!(
        matches!(error, Error::Unexpected(ref message) if message.contains("session is closed")),
        "query iterator must fail after the session closes: {error}"
    );
    let error = search_iterator
        .next()
        .await
        .expect_err("closed session search iterator");
    assert!(
        matches!(error, Error::Unexpected(ref message) if message.contains("session is closed")),
        "search iterator must fail after the session closes: {error}"
    );
    server.shutdown().await;
}

#[tokio::test]
async fn global_cluster_connect_discovers_the_primary_and_serves_dql() {
    let topology = r#"{"code":0,"data":{"version":1,"clusters":[
        {"clusterId":"primary","endpoint":"{endpoint}","capability":3},
        {"clusterId":"replica","endpoint":"{endpoint}","capability":1}
    ]}}"#
        .to_owned();
    let server = MockServer::start_global(topology).await;

    // The client connected through the global-cluster endpoint; the DQL calls below prove it
    // discovered the primary and routed to the mock gRPC server.

    // A plain DQL request routes to the primary (the mock gRPC server) without a cluster_id param.
    server
        .client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .filter("id > 0")
                .output_fields(["id"])
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let request = server.service.request_text("query");
    assert!(
        !request.contains("cluster_id"),
        "plain request must not carry a cluster_id: {request}"
    );

    // A session pins DQL to a specific cluster id while still routing through the primary.
    let session = server
        .client
        .session("cluster-a")
        .expect("valid cluster id");
    session
        .search(
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("vector")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    server.assert_request_contains("search", &["cluster_id", "cluster-a"]);

    server.shutdown().await;
}
