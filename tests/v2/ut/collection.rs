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
use milvus::v2::request::collection::*;
use milvus::v2::request::dml::InsertRequest;
use milvus::v2::request::dql::QueryRequest;
use milvus::v2::{
    CollectionSchema, ConsistencyLevel, DataType, FieldData, FieldSchema, Function, FunctionType,
    IndexParam, LoadState, RetryConfig,
};
use std::time::Duration;
use tonic::Code;

fn assert_collection_description(description: &milvus::v2::CollectionDesc) {
    assert_eq!(description.get_database_name().to_owned(), "default");
    assert_eq!(description.get_collection_name().to_owned(), "books");
    assert_eq!(
        description.get_description().to_owned(),
        "mock books collection"
    );
    assert_eq!(description.get_num_partitions().to_owned(), 0);
    assert_eq!(
        description.get_field_names().to_owned(),
        ["id", "text", "vector"]
    );
    assert_eq!(description.get_vector_field_names().to_owned(), ["vector"]);
    assert_eq!(description.get_primary_field_name().to_owned(), "id");
    assert!(description.is_dynamic_field_enabled());
    assert_eq!(description.get_auto_id().to_owned(), false);
    assert_eq!(description.get_num_shards().to_owned(), 1);
    assert_eq!(description.get_schema().get_fields().len().to_owned(), 3);
    assert_eq!(description.get_collection_id().to_owned(), 1);
    assert!(description.get_aliases().is_empty());
    assert_eq!(description.get_created_time().to_owned(), 102);
    assert_eq!(description.get_created_utc_time().to_owned(), 202);
    assert_eq!(description.get_update_time().to_owned(), 302);
    assert_eq!(
        description.get_consistency_level(),
        ConsistencyLevel::Bounded
    );
    assert!(description.get_properties().is_empty());
}

fn schema() -> CollectionSchema {
    CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("text")
                .data_type(DataType::VarChar)
                .max_length(128),
        )
        .add_field(
            FieldSchema::new()
                .name("vector")
                .data_type(DataType::FloatVector)
                .dimension(2),
        )
}

fn function() -> Function {
    Function::new()
        .name("bm25")
        .function_type(FunctionType::Bm25)
        .input_fields(["text"])
        .output_fields(["sparse"])
}

fn books_insert(id: i64) -> InsertRequest {
    InsertRequest::builder()
        .collection_name("books")
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![id],
            },
            FieldData::VarChar {
                name: "text".into(),
                values: vec![format!("book {id}")],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2]],
            },
        ])
        .build()
        .expect("valid books insert request")
}

fn auto_id_books_insert() -> InsertRequest {
    InsertRequest::builder()
        .collection_name("books")
        .columns(vec![
            FieldData::VarChar {
                name: "text".into(),
                values: vec!["auto-id book".into()],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2]],
            },
        ])
        .build()
        .expect("valid auto-ID books insert request")
}

#[tokio::test]
async fn explicit_describe_operations_do_not_populate_the_schema_cache() {
    let server = MockServer::start().await;

    server
        .client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid describe request"),
        )
        .await
        .expect("describe collection");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .client
        .insert(books_insert(1))
        .await
        .expect("insert performs its own schema load");
    assert_eq!(server.service.call_count("describe_collection"), 2);

    server
        .client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name("batch_books")
                .schema(
                    CollectionSchema::new().add_field(
                        FieldSchema::new()
                            .name("id")
                            .data_type(DataType::Int64)
                            .primary_key(true),
                    ),
                )
                .build()
                .expect("valid collection request"),
        )
        .await
        .expect("create batch-described collection");
    server
        .client
        .batch_describe_collections(
            BatchDescribeCollectionsRequest::builder()
                .collection_name("batch_books")
                .build()
                .expect("valid batch describe request"),
        )
        .await
        .expect("batch describe collection");
    server
        .client
        .insert(
            InsertRequest::builder()
                .collection_name("batch_books")
                .columns(vec![FieldData::Int64 {
                    name: "id".into(),
                    values: vec![1],
                }])
                .build()
                .expect("valid batch books insert request"),
        )
        .await
        .expect("insert performs its own schema load after batch describe");
    assert_eq!(server.service.call_count("batch_describe_collection"), 1);
    assert_eq!(server.service.call_count("describe_collection"), 3);

    server.shutdown().await;
}

#[tokio::test]
async fn unrelated_collection_property_mutations_preserve_cached_schema() {
    let server = MockServer::start().await;

    server
        .client
        .insert(books_insert(1))
        .await
        .expect("prime schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .client
        .alter_collection_properties(
            AlterCollectionPropertiesRequest::builder()
                .collection_name("books")
                .property("retention", "7200")
                .build()
                .expect("valid alter properties request"),
        )
        .await
        .expect("alter unrelated collection property");
    server
        .client
        .insert(books_insert(2))
        .await
        .expect("reuse schema after altering unrelated property");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .client
        .drop_collection_properties(
            DropCollectionPropertiesRequest::builder()
                .collection_name("books")
                .property_key("retention")
                .build()
                .expect("valid drop properties request"),
        )
        .await
        .expect("drop unrelated collection property");
    server
        .client
        .insert(books_insert(3))
        .await
        .expect("reuse schema after dropping unrelated property");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server.shutdown().await;
}

#[tokio::test]
async fn collection_property_mutations_invalidate_cached_auto_id_policy() {
    let server = MockServer::start().await;
    server
        .service
        .set_collection_auto_id("default", "books", true);

    server
        .client
        .insert(auto_id_books_insert())
        .await
        .expect("prime schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .client
        .alter_collection_properties(
            AlterCollectionPropertiesRequest::builder()
                .collection_name("books")
                .property("allow_insert_auto_id", "true")
                .build()
                .expect("valid alter properties request"),
        )
        .await
        .expect("alter collection properties");
    server
        .client
        .insert(books_insert(1))
        .await
        .expect("refreshed schema permits an explicit auto ID");
    assert_eq!(server.service.call_count("describe_collection"), 2);

    server
        .client
        .drop_collection_properties(
            DropCollectionPropertiesRequest::builder()
                .collection_name("books")
                .property_key("allow_insert_auto_id")
                .build()
                .expect("valid drop properties request"),
        )
        .await
        .expect("drop collection properties");
    server
        .client
        .insert(auto_id_books_insert())
        .await
        .expect("refreshed schema still permits server-generated IDs");
    assert_eq!(server.service.call_count("describe_collection"), 3);

    let error = server
        .client
        .insert(books_insert(2))
        .await
        .expect_err("dropped property must reject an explicit auto ID");
    assert!(
        error
            .to_string()
            .contains("field must not be supplied for this operation"),
        "unexpected explicit auto-ID error: {error}"
    );
    assert_eq!(server.service.call_count("insert"), 3);

    server.shutdown().await;
}

#[tokio::test]
async fn collection_function_mutations_invalidate_the_cached_schema() {
    let server = MockServer::start().await;

    server
        .client
        .insert(books_insert(1))
        .await
        .expect("prime schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .client
        .add_collection_function(
            AddCollectionFunctionRequest::builder()
                .collection_name("books")
                .function(function())
                .build()
                .expect("valid add function request"),
        )
        .await
        .expect("add collection function");
    server
        .client
        .insert(books_insert(2))
        .await
        .expect("reload schema after adding a collection function");
    assert_eq!(server.service.call_count("describe_collection"), 2);

    server
        .client
        .alter_collection_function(
            AlterCollectionFunctionRequest::builder()
                .collection_name("books")
                .function(function())
                .build()
                .expect("valid alter function request"),
        )
        .await
        .expect("alter collection function");
    server
        .client
        .insert(books_insert(3))
        .await
        .expect("reload schema after altering a collection function");
    assert_eq!(server.service.call_count("describe_collection"), 3);

    server
        .client
        .drop_collection_function(
            DropCollectionFunctionRequest::builder()
                .collection_name("books")
                .function_name("bm25")
                .build()
                .expect("valid drop function request"),
        )
        .await
        .expect("drop collection function");
    server
        .client
        .insert(books_insert(4))
        .await
        .expect("reload schema after dropping a collection function");
    assert_eq!(server.service.call_count("describe_collection"), 4);

    server.shutdown().await;
}

#[tokio::test]
async fn collection_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name("books")
                .description("mock books collection")
                .schema(schema())
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name("simple_books")
                .dimension(2)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let has_collection = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(has_collection.exists());
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name("books")
                .sync(true)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .refresh_load(
            RefreshLoadRequest::builder()
                .collection_name("books")
                .sync(true)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .release_collection(
            ReleaseCollectionRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_collection_description(description.description());

    let collections = client
        .list_collections(
            ListCollectionsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(collections.collection_names(), &["books", "simple_books"]);
    assert_eq!(collections.collections().len().to_owned(), 2);
    let info = &collections.collections()[0];
    assert_eq!(info.get_name().to_owned(), "books");
    assert_eq!(info.get_id().to_owned(), 1);
    assert_eq!(info.get_created_timestamp().to_owned(), 102);
    assert_eq!(info.get_created_utc_timestamp().to_owned(), 202);
    assert_eq!(info.get_query_service_available(), Some(false));
    assert_eq!(info.get_shard_count(), Some(1));

    let statistics = client
        .get_collection_stats(
            GetCollectionStatsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(
        statistics.statistics().get("row_count").unwrap().to_owned(),
        "1"
    );

    let descriptions = client
        .batch_describe_collections(
            BatchDescribeCollectionsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(descriptions.descriptions().len().to_owned(), 1);
    assert_collection_description(&descriptions.descriptions()[0]);

    let replicas = client
        .describe_replicas(
            DescribeReplicasRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(replicas.replicas().len().to_owned(), 1);
    let replica = &replicas.replicas()[0];
    assert_eq!(replica.get_replica_id().to_owned(), 7);
    assert_eq!(replica.get_collection_id().to_owned(), 1);
    assert_eq!(replica.get_partition_ids().to_owned(), [1]);
    assert_eq!(replica.get_node_ids().to_owned(), [8, 9]);
    assert_eq!(replica.get_resource_group().to_owned(), "default");
    assert_eq!(
        replica.get_outbound_nodes().get("backup").to_owned(),
        Some(&1)
    );
    assert_eq!(replica.get_shards().len().to_owned(), 1);
    let shard = &replica.get_shards()[0];
    assert_eq!(shard.get_leader_id().to_owned(), 8);
    assert_eq!(shard.get_leader_address().to_owned(), "127.0.0.1:21123");
    assert_eq!(shard.get_channel_name().to_owned(), "channel-1");
    assert_eq!(shard.get_node_ids().to_owned(), [8, 9]);

    let load_state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(load_state.state().to_owned(), LoadState::NotLoad);
    assert_eq!(load_state.progress().to_owned(), 0);
    client
        .truncate_collection(
            TruncateCollectionRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    client
        .alter_collection_properties(
            AlterCollectionPropertiesRequest::builder()
                .collection_name("books")
                .property("key", "value")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .drop_collection_properties(
            DropCollectionPropertiesRequest::builder()
                .collection_name("books")
                .property_key("key")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .alter_collection_field_properties(
            AlterCollectionFieldPropertiesRequest::builder()
                .collection_name("books")
                .field_name("text")
                .property("key", "value")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .drop_collection_field_properties(
            DropCollectionFieldPropertiesRequest::builder()
                .collection_name("books")
                .field_name("text")
                .property_key("key")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name("books")
                .field(
                    FieldSchema::new()
                        .name("extra")
                        .data_type(DataType::Int64)
                        .nullable(true),
                )
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .add_collection_function(
            AddCollectionFunctionRequest::builder()
                .collection_name("books")
                .function(function())
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .alter_collection_function(
            AlterCollectionFunctionRequest::builder()
                .collection_name("books")
                .function(function())
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .drop_collection_function(
            DropCollectionFunctionRequest::builder()
                .collection_name("books")
                .function_name("bm25")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .rename_collection(
            RenameCollectionRequest::builder()
                .collection_name("books")
                .new_collection_name("renamed_books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let renamed = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name("renamed_books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(renamed.description().get_collection_name(), "renamed_books");
    client
        .drop_collection(
            DropCollectionRequest::builder()
                .collection_name("renamed_books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let has_renamed = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name("renamed_books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(!has_renamed.exists());

    server.assert_any_request_contains(
        "create_collection",
        &["collection_name: \"books\"", "schema:"],
    );
    server.assert_any_request_contains("create_collection", &["collection_name: \"simple_books\""]);
    server.assert_any_request_contains(
        "load_collection",
        &["collection_name: \"books\"", "refresh: false"],
    );
    server.assert_any_request_contains(
        "load_collection",
        &["collection_name: \"books\"", "refresh: true"],
    );
    server.assert_request_contains("release_collection", &["collection_name: \"books\""]);
    server.assert_request_contains("truncate_collection", &["collection_name: \"books\""]);
    server.assert_request_contains(
        "rename_collection",
        &["old_name: \"books\"", "new_name: \"renamed_books\""],
    );
    server.assert_request_contains("add_collection_field", &["schema:"]);
    server.assert_request_contains("add_collection_function", &["name: \"bm25\""]);
    server.assert_request_contains("alter_collection_function", &["function_name: \"bm25\""]);
    server.assert_request_contains("drop_collection_function", &["function_name: \"bm25\""]);
    server.assert_any_request_contains(
        "alter_collection",
        &["collection_name: \"books\"", "properties:"],
    );
    server.assert_any_request_contains(
        "alter_collection",
        &["collection_name: \"books\"", "delete_keys:"],
    );
    server.assert_any_request_contains(
        "alter_collection_field",
        &["field_name: \"text\"", "properties:"],
    );
    server.assert_any_request_contains(
        "alter_collection_field",
        &["field_name: \"text\"", "delete_keys:"],
    );
    server.assert_request_contains("drop_collection", &["collection_name: \"renamed_books\""]);

    for rpc in [
        "create_collection",
        "create_index",
        "has_collection",
        "load_collection",
        "get_loading_progress",
        "release_collection",
        "describe_collection",
        "show_collections",
        "get_collection_statistics",
        "batch_describe_collection",
        "get_replicas",
        "get_load_state",
        "truncate_collection",
        "rename_collection",
        "alter_collection",
        "alter_collection_field",
        "add_collection_field",
        "add_collection_function",
        "alter_collection_function",
        "drop_collection_function",
        "drop_collection",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn loading_wait_timeout_bounds_stalled_progress_rpc() {
    let server = MockServer::start().await;

    let collection_result = tokio::time::timeout(
        Duration::from_secs(1),
        server.client.load_collection(
            LoadCollectionRequest::builder()
                .collection_name("stalled_loading_progress")
                .timeout_ms(25)
                .build()
                .expect("valid load request"),
        ),
    )
    .await
    .expect("load collection must honor its operation timeout");
    let collection_error = collection_result.expect_err("stalled collection load must time out");
    assert!(matches!(
        collection_error,
        Error::Timeout(message) if message == "load collection"
    ));

    let partition_result = tokio::time::timeout(
        Duration::from_secs(1),
        server.client.load_partitions(
            milvus::v2::request::partition::LoadPartitionsRequest::builder()
                .collection_name("stalled_loading_progress")
                .partition_name("p1")
                .timeout_ms(25)
                .build()
                .expect("valid load request"),
        ),
    )
    .await
    .expect("load partitions must honor its operation timeout");
    let partition_error = partition_result.expect_err("stalled partition load must time out");
    assert!(matches!(
        partition_error,
        Error::Timeout(message) if message == "load partitions"
    ));

    assert_eq!(server.service.call_count("get_loading_progress"), 2);
    server.shutdown().await;
}

#[tokio::test]
async fn create_collection_validates_index_follow_up_before_rpc() {
    let server = MockServer::start().await;

    let result = server
        .client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name("invalid_index_collection")
                .schema(schema())
                .index_param(IndexParam::new().field_name("vector"))
                .build()
                .expect("valid collection request"),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(server.service.call_count("create_collection"), 0);
    assert_eq!(server.service.call_count("create_index"), 0);
    server.shutdown().await;
}

#[tokio::test]
async fn idempotent_create_preserves_session_timestamp_and_invalidates_schema() {
    let server = MockServer::start().await;
    let client = &server.client;
    let database = "idempotent_create_db";
    let collection = "idempotent_create_books";
    let create_request = || {
        CreateCollectionRequest::builder()
            .database_name(database)
            .collection_name(collection)
            .schema(schema())
            .build()
            .expect("valid idempotent create request")
    };
    let insert_request = |id| {
        InsertRequest::builder()
            .database_name(database)
            .collection_name(collection)
            .columns(vec![
                FieldData::Int64 {
                    name: "id".into(),
                    values: vec![id],
                },
                FieldData::VarChar {
                    name: "text".into(),
                    values: vec![format!("book {id}")],
                },
                FieldData::FloatVector {
                    name: "vector".into(),
                    values: vec![vec![0.1, 0.2]],
                },
            ])
            .build()
            .expect("valid idempotent create insert request")
    };

    client
        .create_collection(create_request())
        .await
        .expect("create collection");
    client
        .insert(insert_request(1))
        .await
        .expect("record DML timestamp and prime schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    client
        .create_collection(create_request())
        .await
        .expect("idempotent create of the existing collection");
    client
        .query(
            QueryRequest::builder()
                .database_name(database)
                .collection_name(collection)
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid session query"),
        )
        .await
        .expect("session query after idempotent create");
    server.assert_request_contains(
        "query",
        &[
            "collection_name: \"idempotent_create_books\"",
            "guarantee_timestamp: 10",
        ],
    );

    client
        .insert(insert_request(2))
        .await
        .expect("reload schema after idempotent create");
    assert_eq!(server.service.call_count("describe_collection"), 2);
    assert_eq!(server.service.call_count("create_collection"), 2);

    server.shutdown().await;
}

#[tokio::test]
async fn truncate_clears_the_collection_session_timestamp() {
    let server = MockServer::start().await;
    let client = &server.client;
    let database = "truncate_gts_db";
    let collection = "truncate_gts_books";
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .database_name(database)
                .collection_name(collection)
                .schema(schema())
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create collection for truncate timestamp test");
    let insert = InsertRequest::builder()
        .database_name(database)
        .collection_name(collection)
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
        .expect("build truncate timestamp insert");
    client
        .insert(insert)
        .await
        .expect("record DML timestamp before truncate");

    client
        .query(
            QueryRequest::builder()
                .database_name(database)
                .collection_name(collection)
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query before truncate");
    server.assert_request_contains("query", &["guarantee_timestamp: 10"]);

    client
        .truncate_collection(
            TruncateCollectionRequest::builder()
                .database_name(database)
                .collection_name(collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("truncate collection");
    client
        .query(
            QueryRequest::builder()
                .database_name(database)
                .collection_name(collection)
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query after truncate");
    server.assert_request_contains("query", &["guarantee_timestamp: 1"]);
    server.shutdown().await;
}

#[tokio::test]
async fn truncate_does_not_retry_ambiguous_transport_errors() {
    let server = MockServer::start().await;
    server.client.set_retry_param(
        RetryConfig::new()
            .max_attempts(3)
            .initial_backoff(Duration::ZERO)
            .max_backoff(Duration::ZERO),
    );

    for (attempt, code) in [
        Code::Unavailable,
        Code::Unknown,
        Code::Internal,
        Code::Aborted,
        Code::Cancelled,
    ]
    .into_iter()
    .enumerate()
    {
        server
            .service
            .fail_next_transport("truncate_collection", code);
        let error = server
            .client
            .truncate_collection(
                TruncateCollectionRequest::builder()
                    .collection_name("books")
                    .build()
                    .expect("valid truncate request"),
            )
            .await
            .expect_err("ambiguous truncate failure must be returned");
        assert!(matches!(error, Error::Grpc(status) if status.code() == code));
        assert_eq!(
            server.service.call_count("truncate_collection"),
            attempt + 1
        );
    }

    server.shutdown().await;
}

#[tokio::test]
async fn rename_collection_preserves_the_session_timestamp() {
    let server = MockServer::start().await;
    let client = &server.client;
    let database = "rename_gts_db";
    let old_collection = "old_books";
    let new_collection = "new_books";
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .database_name(database)
                .collection_name(old_collection)
                .schema(schema())
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create collection for rename timestamp test");
    client
        .insert(
            InsertRequest::builder()
                .database_name(database)
                .collection_name(old_collection)
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
        .expect("record DML timestamp before rename");

    client
        .rename_collection(
            RenameCollectionRequest::builder()
                .database_name(database)
                .collection_name(old_collection)
                .new_collection_name(new_collection)
                .build()
                .expect("valid rename request"),
        )
        .await
        .expect("rename collection");
    client
        .query(
            QueryRequest::builder()
                .database_name(database)
                .collection_name(new_collection)
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid session query"),
        )
        .await
        .expect("query renamed collection");
    server.assert_request_contains(
        "query",
        &["collection_name: \"new_books\"", "guarantee_timestamp: 10"],
    );
    server.assert_request_contains(
        "rename_collection",
        &[
            "db_name: \"rename_gts_db\"",
            "old_name: \"old_books\"",
            "new_name: \"new_books\"",
            "new_db_name: \"rename_gts_db\"",
        ],
    );

    server.shutdown().await;
}
