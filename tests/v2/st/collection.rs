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

use milvus::v2::request::alias::{CreateAliasRequest, DropAliasRequest};
use milvus::v2::request::collection::{
    AddCollectionFieldRequest, AddCollectionFunctionRequest, AlterCollectionFieldPropertiesRequest,
    AlterCollectionFunctionRequest, AlterCollectionPropertiesRequest,
    BatchDescribeCollectionsRequest, CreateCollectionRequest, CreateSimpleCollectionRequest,
    DescribeCollectionRequest, DescribeReplicasRequest, DropCollectionFieldPropertiesRequest,
    DropCollectionFunctionRequest, DropCollectionPropertiesRequest, GetCollectionStatsRequest,
    GetLoadStateRequest, HasCollectionRequest, ListCollectionsRequest, LoadCollectionRequest,
    RefreshLoadRequest, ReleaseCollectionRequest, RenameCollectionRequest,
    TruncateCollectionRequest,
};
use milvus::v2::request::dml::InsertRequest;
use milvus::v2::request::index::CreateIndexRequest;
use milvus::v2::request::partition::ListPartitionsRequest;
use milvus::v2::{
    CollectionSchema, ConsistencyLevel, DataType, FieldSchema, Function, FunctionType, IndexParam,
    IndexType, LoadState, MetricType,
};
use std::collections::HashMap;

use super::common;

#[tokio::test]
async fn describe_collection() {
    let client = common::client().await;
    let collection = common::unique_collection_name("describe_collection");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    let alias = common::unique_name("describe_collection_alias");
    let schema = common::advanced_schema()
        .description("describe collection system test")
        .add_field(
            FieldSchema::new()
                .name("partition_key")
                .data_type(DataType::VarChar)
                .max_length(128)
                .partition_key(true),
        );
    let properties = HashMap::from([
        ("collection.ttl.seconds".to_owned(), "120".to_owned()),
        ("test.description".to_owned(), "complete".to_owned()),
    ]);

    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection)
                .description("describe collection system test")
                .schema(schema.clone())
                .num_partitions(2)
                .num_shards(2)
                .consistency_level(ConsistencyLevel::Strong)
                .properties(properties.clone())
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create describe collection");
    client
        .create_alias(
            CreateAliasRequest::builder()
                .collection_name(&collection)
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create describe collection alias");

    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe collection");
    let desc = response.description();

    let expected_field_names = schema
        .get_fields()
        .iter()
        .map(|field| field.get_name().to_owned())
        .chain(
            schema
                .get_struct_fields()
                .iter()
                .map(|field| field.get_name().to_owned()),
        )
        .collect::<Vec<_>>();
    let expected_vector_field_names = vec![
        common::VECTOR_FIELD.to_owned(),
        common::BINARY_VECTOR_FIELD.to_owned(),
        common::BFLOAT16_VECTOR_FIELD.to_owned(),
        common::SPARSE_VECTOR_FIELD.to_owned(),
    ];

    assert_eq!(
        desc.get_database_name().to_owned(),
        client.current_database()
    );
    assert_eq!(desc.get_collection_name().to_owned(), collection);
    assert_eq!(
        desc.get_description().to_owned(),
        "describe collection system test"
    );
    assert_eq!(desc.get_num_partitions().to_owned(), 2);
    assert_eq!(desc.get_field_names().to_owned(), expected_field_names);
    assert_eq!(
        desc.get_vector_field_names().to_owned(),
        expected_vector_field_names
    );
    assert_eq!(desc.get_primary_field_name().to_owned(), common::ID_FIELD);
    assert!(!desc.is_dynamic_field_enabled());
    assert!(!desc.get_auto_id());
    assert_eq!(desc.get_num_shards().to_owned(), 2);
    let returned_schema = desc.get_schema();
    assert_eq!(
        returned_schema.get_description().to_owned(),
        schema.get_description()
    );
    assert_eq!(
        returned_schema.is_dynamic_field_enabled(),
        schema.is_dynamic_field_enabled()
    );
    assert_eq!(
        returned_schema.get_fields().len(),
        schema.get_fields().len()
    );
    for expected in schema.get_fields() {
        let actual = returned_schema
            .get_fields()
            .iter()
            .find(|field| field.get_name() == expected.get_name())
            .expect("described schema field");
        assert_eq!(actual.get_data_type().to_owned(), expected.get_data_type());
        assert_eq!(
            actual.get_element_type().to_owned(),
            expected.get_element_type()
        );
        assert_eq!(
            actual.is_primary_key().to_owned(),
            expected.is_primary_key()
        );
        assert_eq!(actual.is_auto_id().to_owned(), expected.is_auto_id());
        assert_eq!(
            actual.is_partition_key().to_owned(),
            expected.is_partition_key()
        );
        assert_eq!(
            actual.is_clustering_key().to_owned(),
            expected.is_clustering_key()
        );
        assert_eq!(actual.is_nullable().to_owned(), expected.is_nullable());
        assert_eq!(actual.get_type_params(), expected.get_type_params());
        assert_eq!(actual.get_index_params(), expected.get_index_params());
        if expected.get_data_type() != DataType::Timestamptz {
            assert_eq!(actual.get_default_value(), expected.get_default_value());
        } else {
            assert!(actual.get_default_value().is_some());
        }
    }
    assert_eq!(
        returned_schema.get_struct_fields(),
        schema.get_struct_fields()
    );
    assert_eq!(
        returned_schema.get_functions().to_owned(),
        schema.get_functions()
    );
    for (key, value) in &properties {
        assert_eq!(
            returned_schema.get_properties().get(key).to_owned(),
            Some(value)
        );
    }
    assert!(desc.get_collection_id() > 0);
    assert_eq!(desc.get_aliases().to_owned(), [alias.clone()]);
    assert!(desc.get_created_time() > 0);
    assert!(desc.get_created_utc_time() > 0);
    assert!(
        desc.get_update_time() == 0 || desc.get_update_time() >= desc.get_created_time(),
        "update time must be unset or no earlier than creation time"
    );
    assert_eq!(
        desc.get_consistency_level().to_owned(),
        ConsistencyLevel::Strong
    );
    for (key, value) in &properties {
        assert_eq!(desc.get_properties().get(key).to_owned(), Some(value));
    }

    client
        .drop_alias(
            DropAliasRequest::builder()
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop describe collection alias");
    common::drop_collection(&client, &collection)
        .await
        .expect("drop described collection");
}

#[tokio::test]
async fn create_collection_variants_and_discovery() {
    let client = common::client().await;
    let full_collection = common::unique_collection_name("collection_full");
    let simple_collection = common::unique_collection_name("collection_simple");
    let _cleanup = common::CollectionCleanup::new([&full_collection, &simple_collection]);

    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&full_collection)
                .description("full collection system test")
                .schema(common::advanced_schema())
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create full collection");
    client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name(&simple_collection)
                .dimension(4)
                .primary_field("key")
                .primary_field_type(DataType::VarChar)
                .max_length(1_024)
                .vector_field("embedding")
                .enable_dynamic_field(false)
                .metric_type(MetricType::Cosine)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create simple collection");

    for collection in [&full_collection, &simple_collection] {
        let response = client
            .has_collection(
                HasCollectionRequest::builder()
                    .collection_name(collection)
                    .build()
                    .expect("valid request"),
            )
            .await
            .expect("check collection existence");
        assert!(response.exists());
    }

    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&full_collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe full collection");
    let full_description = description.description();
    assert_eq!(
        full_description.get_collection_name().to_owned(),
        full_collection
    );
    assert_eq!(
        full_description.get_description(),
        "full collection system test"
    );
    assert!(full_description.get_num_shards() > 0);
    assert!(full_description.get_num_partitions() > 0);
    assert!(full_description.get_collection_id() > 0);
    assert!(full_description.get_created_time() > 0);
    assert!(full_description.get_created_utc_time() > 0);
    assert_eq!(
        full_description.get_primary_field_name().to_owned(),
        common::ID_FIELD
    );
    assert!(!full_description.is_dynamic_field_enabled());
    assert!(!full_description.get_auto_id());
    assert_eq!(
        full_description.get_consistency_level(),
        ConsistencyLevel::Strong
    );
    assert!(full_description
        .get_field_names()
        .iter()
        .any(|name| name == common::STRUCT_FIELD));
    for vector_field in [
        common::VECTOR_FIELD,
        common::BINARY_VECTOR_FIELD,
        common::BFLOAT16_VECTOR_FIELD,
        common::SPARSE_VECTOR_FIELD,
    ] {
        assert!(full_description
            .get_vector_field_names()
            .iter()
            .any(|name| name == vector_field));
    }
    let schema = full_description.get_schema();
    assert!(schema
        .get_fields()
        .iter()
        .any(|field| field.get_data_type() == DataType::Geometry));
    assert!(schema
        .get_fields()
        .iter()
        .any(|field| field.get_data_type() == DataType::Timestamptz));
    assert!(schema
        .get_struct_fields()
        .iter()
        .any(|field| field.get_name() == common::STRUCT_FIELD));
    let expected_scalar_types = [
        DataType::Bool,
        DataType::Int8,
        DataType::Int16,
        DataType::Int32,
        DataType::Int64,
        DataType::Float,
        DataType::Double,
        DataType::VarChar,
        DataType::Json,
        DataType::Geometry,
        DataType::Timestamptz,
        DataType::Array,
    ];
    for data_type in expected_scalar_types {
        assert!(schema
            .get_fields()
            .iter()
            .any(|field| field.get_data_type() == data_type));
    }
    let array = schema
        .get_fields()
        .iter()
        .find(|field| field.get_name() == common::INT64_ARRAY_FIELD)
        .expect("int64 array field");
    assert_eq!(array.get_element_type().to_owned(), Some(DataType::Int64));
    let vector_types = schema
        .get_fields()
        .iter()
        .filter(|field| {
            matches!(
                field.get_data_type(),
                DataType::FloatVector
                    | DataType::BinaryVector
                    | DataType::BFloat16Vector
                    | DataType::SparseFloatVector
            )
        })
        .map(FieldSchema::get_data_type)
        .collect::<Vec<_>>();
    assert_eq!(vector_types.len(), 4);
    assert!(vector_types.contains(&DataType::FloatVector));
    assert!(vector_types.contains(&DataType::BinaryVector));
    assert!(vector_types.contains(&DataType::BFloat16Vector));
    assert!(vector_types.contains(&DataType::SparseFloatVector));

    let simple_description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&simple_collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe simple collection");
    let simple_schema = simple_description.description().get_schema();
    let primary_field = simple_schema
        .get_fields()
        .iter()
        .find(|field| field.is_primary_key())
        .expect("simple collection primary field");
    assert_eq!(primary_field.get_data_type().to_owned(), DataType::VarChar);
    assert_eq!(
        primary_field
            .get_type_params()
            .get("max_length")
            .map(String::as_str),
        Some("1024")
    );

    let collections = client
        .list_collections(
            ListCollectionsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list collections");
    assert!(collections.collection_names().contains(&full_collection));
    assert!(collections.collection_names().contains(&simple_collection));

    let statistics = client
        .get_collection_stats(
            GetCollectionStatsRequest::builder()
                .collection_name(&full_collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get collection statistics");
    assert!(statistics.statistics().contains_key("row_count"));

    let descriptions = client
        .batch_describe_collections(
            BatchDescribeCollectionsRequest::builder()
                .collection_names([&full_collection, &simple_collection])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("batch describe collections");
    assert!(descriptions
        .descriptions()
        .iter()
        .any(|description| { description.get_collection_name() == &full_collection }));
    assert!(descriptions
        .descriptions()
        .iter()
        .any(|description| { description.get_collection_name() == &simple_collection }));

    common::drop_collection(&client, &full_collection)
        .await
        .expect("drop full collection");
    common::drop_collection(&client, &simple_collection)
        .await
        .expect("drop simple collection");
    for collection in [&full_collection, &simple_collection] {
        let response = client
            .has_collection(
                HasCollectionRequest::builder()
                    .collection_name(collection)
                    .build()
                    .expect("valid request"),
            )
            .await
            .expect("check dropped collection");
        assert!(!response.exists());
    }
}

#[tokio::test]
async fn load_refresh_replica_state_and_release() {
    let client = common::client().await;
    let collection = common::unique_collection_name("collection_loading");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    common::create_advanced_collection(&client, &collection).await;
    let state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get initial collection load state");
    assert_eq!(state.state().to_owned(), LoadState::NotLoad);
    assert_eq!(state.progress().to_owned(), 0);
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(&collection)
                .index_param(
                    IndexParam::new()
                        .field_name(common::VECTOR_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::L2),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(common::BINARY_VECTOR_FIELD)
                        .index_type(IndexType::BinFlat)
                        .metric_type(MetricType::Hamming),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(common::BFLOAT16_VECTOR_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::L2),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(common::SPARSE_VECTOR_FIELD)
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Ip),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create index before loading");

    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(&collection)
                .load_fields(common::advanced_load_fields())
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load collection");
    let state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get collection load state");
    assert_eq!(state.state().to_owned(), LoadState::Loaded);
    assert_eq!(state.progress().to_owned(), 100);
    let partitions = client
        .list_partitions(
            ListPartitionsRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list loaded collection partitions");
    let partition = partitions
        .partition_names()
        .first()
        .expect("loaded collection partition");
    let partition_state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(&collection)
                .partition_name(partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get partition load state");
    assert_eq!(partition_state.state().to_owned(), LoadState::Loaded);
    assert_eq!(partition_state.progress().to_owned(), 100);

    client
        .refresh_load(
            RefreshLoadRequest::builder()
                .collection_name(&collection)
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("refresh loaded collection");
    let replicas = client
        .describe_replicas(
            DescribeReplicasRequest::builder()
                .collection_name(&collection)
                .with_shard_nodes(true)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe collection replicas");
    assert!(!replicas.replicas().is_empty());

    client
        .release_collection(
            ReleaseCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("release collection");
    common::drop_collection(&client, &collection)
        .await
        .expect("drop loading collection");
}

#[tokio::test]
async fn truncate_and_rename_collection() {
    let client = common::client().await;
    let collection = common::unique_collection_name("collection_truncate");
    let renamed_collection = common::unique_collection_name("collection_renamed");
    let _cleanup = common::CollectionCleanup::new([&collection, &renamed_collection]);
    common::create_advanced_collection(&client, &collection).await;
    let insert = InsertRequest::builder()
        .collection_name(&collection)
        .columns(common::advanced_columns())
        .build()
        .expect("build truncate insert request");
    client.insert(insert).await.expect("insert before truncate");

    client
        .truncate_collection(
            TruncateCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("truncate collection");
    client
        .rename_collection(
            RenameCollectionRequest::builder()
                .collection_name(&collection)
                .new_collection_name(&renamed_collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("rename collection");

    let old_exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("check old collection name");
    let new_exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(&renamed_collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("check new collection name");
    assert!(!old_exists.exists());
    assert!(new_exists.exists());

    common::drop_collection(&client, &renamed_collection)
        .await
        .expect("drop renamed collection");
}

#[tokio::test]
async fn alter_collection_and_field_properties() {
    let client = common::client().await;
    let collection = common::unique_collection_name("collection_properties");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("title")
                .data_type(DataType::VarChar)
                .max_length(128),
        )
        .add_field(
            FieldSchema::new()
                .name("vector")
                .data_type(DataType::FloatVector)
                .dimension(4),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection)
                .schema(schema)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create properties collection");

    client
        .alter_collection_properties(
            AlterCollectionPropertiesRequest::builder()
                .collection_name(&collection)
                .properties(HashMap::from([
                    ("collection.ttl.seconds".to_owned(), "60".to_owned()),
                    ("test.property".to_owned(), "enabled".to_owned()),
                ]))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("alter collection properties");
    client
        .alter_collection_field_properties(
            AlterCollectionFieldPropertiesRequest::builder()
                .collection_name(&collection)
                .field_name("title")
                .properties(HashMap::from([
                    ("max_length".to_owned(), "256".to_owned()),
                    ("mmap.enabled".to_owned(), "true".to_owned()),
                ]))
                .build()
                .expect("valid request"),
        )
        .await
        .expect("alter collection field properties");

    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe altered collection");
    assert_eq!(
        description
            .description()
            .get_properties()
            .get("collection.ttl.seconds"),
        Some(&"60".to_owned())
    );
    assert_eq!(
        description
            .description()
            .get_properties()
            .get("test.property"),
        Some(&"enabled".to_owned())
    );
    let title = description
        .description()
        .get_schema()
        .get_fields()
        .iter()
        .find(|field| field.get_name() == "title")
        .expect("title field");
    assert_eq!(
        title.get_type_params().get("max_length"),
        Some(&"256".to_owned())
    );
    assert_eq!(
        title.get_type_params().get("mmap.enabled"),
        Some(&"true".to_owned())
    );

    client
        .drop_collection_properties(
            DropCollectionPropertiesRequest::builder()
                .collection_name(&collection)
                .property_key("test.property")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop collection property");
    client
        .drop_collection_field_properties(
            DropCollectionFieldPropertiesRequest::builder()
                .collection_name(&collection)
                .field_name("title")
                .property_key("mmap.enabled")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop collection field property");

    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe collection after dropping properties");
    assert!(!description
        .description()
        .get_properties()
        .contains_key("test.property"));
    let title = description
        .description()
        .get_schema()
        .get_fields()
        .iter()
        .find(|field| field.get_name() == "title")
        .expect("title field after dropping property");
    assert!(!title.get_type_params().contains_key("mmap.enabled"));
    assert_eq!(
        title.get_type_params().get("max_length"),
        Some(&"256".to_owned())
    );

    common::drop_collection(&client, &collection)
        .await
        .expect("drop properties collection");
}

#[tokio::test]
async fn add_collection_field() {
    let client = common::client().await;
    let collection = common::unique_collection_name("collection_add_field");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    common::create_advanced_collection(&client, &collection).await;

    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name(&collection)
                .field(
                    FieldSchema::new()
                        .name("note")
                        .data_type(DataType::VarChar)
                        .max_length(256)
                        .nullable(true),
                )
                .build()
                .expect("valid request"),
        )
        .await
        .expect("add collection field");
    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe collection after adding field");
    let note = description
        .description()
        .get_schema()
        .get_fields()
        .iter()
        .find(|field| field.get_name() == "note")
        .expect("added note field");
    assert_eq!(note.get_data_type().to_owned(), DataType::VarChar);
    assert!(note.is_nullable());

    common::drop_collection(&client, &collection)
        .await
        .expect("drop add-field collection");
}

#[tokio::test]
async fn collection_function_lifecycle() {
    let client = common::client().await;
    let embedding_server = common::MockEmbeddingServer::start();
    let collection = common::unique_collection_name("collection_function");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    let function_name = common::unique_name("text_embedding_function");
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("document")
                .data_type(DataType::VarChar)
                .max_length(9_000),
        )
        .add_field(
            FieldSchema::new()
                .name("dense_vector")
                .data_type(DataType::FloatVector)
                .dimension(4),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection)
                .schema(schema)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create function collection");

    let function = Function::new()
        .name(&function_name)
        .function_type(FunctionType::TextEmbedding)
        .description("initial text embedding function")
        .input_fields(["document"])
        .output_fields(["dense_vector"])
        .param("provider", "tei")
        .param("endpoint", embedding_server.endpoint());
    client
        .add_collection_function(
            AddCollectionFunctionRequest::builder()
                .collection_name(&collection)
                .function(function)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("add collection function");
    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe collection with function");
    assert!(description
        .description()
        .get_schema()
        .get_functions()
        .iter()
        .any(|function| function.get_name() == function_name));

    let altered_function = Function::new()
        .name(&function_name)
        .function_type(FunctionType::TextEmbedding)
        .description("altered text embedding function")
        .input_fields(["document"])
        .output_fields(["dense_vector"])
        .param("provider", "tei")
        .param("endpoint", embedding_server.endpoint());
    client
        .alter_collection_function(
            AlterCollectionFunctionRequest::builder()
                .collection_name(&collection)
                .function(altered_function)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("alter collection function");
    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe altered collection function");
    assert!(description
        .description()
        .get_schema()
        .get_functions()
        .iter()
        .any(|function| {
            function.get_name() == function_name
                && function.get_description() == "altered text embedding function"
        }));

    client
        .drop_collection_function(
            DropCollectionFunctionRequest::builder()
                .collection_name(&collection)
                .function_name(&function_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop collection function");
    let description = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe collection after dropping function");
    assert!(!description
        .description()
        .get_schema()
        .get_functions()
        .iter()
        .any(|function| function.get_name() == function_name));

    common::drop_collection(&client, &collection)
        .await
        .expect("drop function collection");
}
