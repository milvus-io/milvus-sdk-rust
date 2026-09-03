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
use milvus::v2::request::index::*;
use milvus::v2::{IndexParam, IndexStateCode, IndexType, MetricType};
use std::time::Duration;

fn assert_index(index: &milvus::v2::IndexDesc) {
    assert_eq!(index.get_index_name().to_owned(), "vector_idx");
    assert_eq!(index.get_index_id().to_owned(), 10);
    assert_eq!(index.get_field_name().to_owned(), "vector");
    assert_eq!(index.get_index_type().to_owned(), IndexType::Hnsw);
    assert_eq!(index.get_metric_type().to_owned(), MetricType::Cosine);
    assert!(index.get_extra_params().is_empty());
    assert_eq!(index.get_indexed_rows().to_owned(), 1);
    assert_eq!(index.get_total_rows().to_owned(), 1);
    assert_eq!(index.get_pending_rows().to_owned(), 0);
    assert_eq!(index.get_state().to_owned(), IndexStateCode::Finished);
    assert!(index.get_failure_reason().is_empty());
    assert_eq!(index.get_min_version().to_owned(), 0);
    assert_eq!(index.get_max_version().to_owned(), 0);
}

#[tokio::test]
async fn index_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name("books")
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::Cosine)
                        .index_name("vector_idx"),
                )
                .sync(true)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = client
        .describe_index(
            DescribeIndexRequest::builder()
                .collection_name("books")
                .index_name("vector_idx")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(description.indexes().len().to_owned(), 1);
    assert_index(&description.indexes()[0]);

    let indexes = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(indexes.index_names().to_owned(), ["vector_idx"]);
    assert_eq!(indexes.indexes().len().to_owned(), 1);
    assert_index(&indexes.indexes()[0]);
    client
        .alter_index_properties(
            AlterIndexPropertiesRequest::builder()
                .collection_name("books")
                .index_name("vector_idx")
                .property("key", "value")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    client
        .drop_index_properties(
            DropIndexPropertiesRequest::builder()
                .collection_name("books")
                .index_name("vector_idx")
                .property_key("key")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    server.assert_request_contains(
        "create_index",
        &[
            "collection_name: \"books\"",
            "field_name: \"vector\"",
            "index_name: \"vector_idx\"",
            "HNSW",
            "COSINE",
        ],
    );
    server.assert_any_request_contains(
        "alter_index",
        &["index_name: \"vector_idx\"", "extra_params:"],
    );
    server.assert_any_request_contains(
        "alter_index",
        &["index_name: \"vector_idx\"", "delete_keys:"],
    );
    client
        .drop_index(
            DropIndexRequest::builder()
                .collection_name("books")
                .field_name("vector_idx")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    server.assert_request_contains("drop_index", &["index_name: \"vector_idx\""]);
    let indexes = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(indexes.index_names().is_empty());
    assert!(indexes.indexes().is_empty());

    for rpc in [
        "create_index",
        "describe_index",
        "get_index_statistics",
        "alter_index",
        "drop_index",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn index_wait_timeout_bounds_sleep_and_stalled_poll() {
    let delayed_server = MockServer::start().await;
    let delayed_result = tokio::time::timeout(
        Duration::from_secs(1),
        delayed_server.client.create_index(
            CreateIndexRequest::builder()
                .collection_name("delayed_index_progress")
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::Hnsw),
                )
                .timeout_ms(25)
                .build()
                .expect("valid delayed index request"),
        ),
    )
    .await
    .expect("index wait must honor its operation timeout");
    let delayed_error = delayed_result.expect_err("poll delay must not outlive index timeout");
    assert!(matches!(
        delayed_error,
        Error::Timeout(message) if message == "creating index"
    ));
    assert_eq!(delayed_server.service.call_count("describe_index"), 1);
    delayed_server.shutdown().await;

    let stalled_server = MockServer::start().await;
    let stalled_result = tokio::time::timeout(
        Duration::from_secs(1),
        stalled_server.client.create_index(
            CreateIndexRequest::builder()
                .collection_name("stalled_index_poll")
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::Hnsw),
                )
                .timeout_ms(25)
                .build()
                .expect("valid stalled index request"),
        ),
    )
    .await
    .expect("stalled describe-index poll must honor the operation timeout");
    let stalled_error = stalled_result.expect_err("stalled index poll must time out");
    assert!(matches!(
        stalled_error,
        Error::Timeout(message) if message == "creating index"
    ));
    assert_eq!(stalled_server.service.call_count("describe_index"), 1);
    stalled_server.shutdown().await;
}

#[tokio::test]
async fn list_indexes_filters_by_field_name() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name("books")
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::Cosine)
                        .index_name("vector_idx"),
                )
                .index_param(
                    IndexParam::new()
                        .field_name("title")
                        .index_type(IndexType::Inverted)
                        .metric_type(MetricType::Default)
                        .index_name("title_idx"),
                )
                .sync(false)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    let all = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let mut all_names = all.index_names().to_vec();
    all_names.sort();
    assert_eq!(all_names, ["title_idx", "vector_idx"]);

    let vector_only = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("books")
                .field_name("vector")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(vector_only.index_names().to_owned(), ["vector_idx"]);
    assert_eq!(vector_only.indexes().len().to_owned(), 1);
    assert_eq!(
        vector_only.indexes()[0].get_field_name().to_owned(),
        "vector"
    );

    let title_only = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("books")
                .field_name("title")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(title_only.index_names().to_owned(), ["title_idx"]);

    let none = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("books")
                .field_name("missing")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(none.index_names().is_empty());
    assert!(none.indexes().is_empty());

    server.shutdown().await;
}

#[tokio::test]
async fn list_indexes_combines_index_and_field_filters_and_keeps_names_in_sync() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name("combined_indexes")
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::Cosine)
                        .index_name("vector_idx_a"),
                )
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::IvfFlat)
                        .metric_type(MetricType::Cosine)
                        .index_name("vector_idx_b"),
                )
                .index_param(
                    IndexParam::new()
                        .field_name("title")
                        .index_type(IndexType::Inverted)
                        .metric_type(MetricType::Default)
                        .index_name("title_idx"),
                )
                .sync(false)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    let by_field = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("combined_indexes")
                .field_name("vector")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let mut by_field_names = by_field.index_names().to_vec();
    by_field_names.sort();
    assert_eq!(by_field_names, ["vector_idx_a", "vector_idx_b"]);
    assert_eq!(by_field.indexes().len().to_owned(), 2);
    let desc_names: Vec<String> = by_field
        .indexes()
        .iter()
        .map(|index| index.get_index_name().to_owned())
        .collect();
    let mut desc_names = desc_names;
    desc_names.sort();
    assert_eq!(desc_names, by_field_names);

    let combined = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("combined_indexes")
                .field_name("vector")
                .index_name("vector_idx_a")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(combined.index_names().to_owned(), ["vector_idx_a"]);
    assert_eq!(combined.indexes().len().to_owned(), 1);
    assert_eq!(combined.indexes()[0].get_field_name().to_owned(), "vector");

    let field_mismatch = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name("combined_indexes")
                .field_name("title")
                .index_name("vector_idx_a")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(field_mismatch.index_names().is_empty());
    assert!(field_mismatch.indexes().is_empty());

    server.shutdown().await;
}
