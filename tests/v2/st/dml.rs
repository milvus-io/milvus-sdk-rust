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

use milvus::v2::prelude::*;
use serde_json::json;
use std::collections::BTreeMap;

use super::common;

#[tokio::test]
async fn insert_advanced_types_by_columns() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("dml_columns");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;

    let columns = common::advanced_columns();
    assert_advanced_insert_column_coverage(&columns);
    let request = InsertRequest::builder()
        .collection_name(&collection_name)
        .columns(columns)
        .build()
        .expect("build column-based insert request");
    let response = client.insert(request).await.expect("insert columns");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop column-insert collection");

    assert_eq!(response.insert_count().to_owned(), 2);
    assert!(response.failed_indices().is_empty());
}

fn assert_advanced_insert_column_coverage(columns: &[FieldData]) {
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Bool { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Int8 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Int16 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Int32 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Int64 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Float { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Double { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::VarChar { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Json { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Geometry { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Timestamptz { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayBool { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayInt8 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayInt16 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayInt32 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayInt64 { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayFloat { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayDouble { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::ArrayVarChar { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::Struct { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::FloatVector { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::BinaryVector { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::BFloat16Vector { .. })));
    assert!(columns
        .iter()
        .any(|field| matches!(field, FieldData::SparseFloatVector { .. })));
}

#[tokio::test]
async fn insert_float16_and_int8_vectors_by_columns() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("dml_remaining_vectors");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
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
                .name("float16")
                .data_type(DataType::Float16Vector)
                .dimension(4),
        )
        .add_field(
            FieldSchema::new()
                .name("int8")
                .data_type(DataType::Int8Vector)
                .dimension(4),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection_name)
                .schema(schema)
                .consistency_level(ConsistencyLevel::Strong)
                .index_param(
                    IndexParam::new()
                        .field_name("float16")
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                )
                .index_param(
                    IndexParam::new()
                        .field_name("int8")
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                )
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create remaining-vector collection");

    let request = InsertRequest::builder()
        .collection_name(&collection_name)
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::Float16Vector {
                name: "float16".into(),
                values: vec![milvus::v2::array_f32_to_f16(&[0.1, 0.2, 0.3, 0.4])],
            },
            FieldData::Int8Vector {
                name: "int8".into(),
                values: vec![vec![-128, -1, 1, 127]],
            },
        ])
        .build()
        .expect("build remaining-vector insert request");
    let response = client
        .insert(request)
        .await
        .expect("insert Float16 and Int8 vectors");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop remaining-vector collection");

    assert_eq!(response.insert_count().to_owned(), 1);
    assert!(response.failed_indices().is_empty());
}

#[tokio::test]
async fn insert_nullable_and_default_values_by_columns() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("dml_nullable_columns");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;

    let mut columns = common::advanced_columns();
    replace_column(
        &mut columns,
        FieldData::nullable(
            FieldData::Bool {
                name: "bool_value".into(),
                values: vec![false],
            },
            vec![true, false],
        )
        .expect("nullable bool column"),
    );
    replace_column(
        &mut columns,
        FieldData::nullable(
            FieldData::BinaryVector {
                name: common::BINARY_VECTOR_FIELD.into(),
                values: vec![vec![0b1010_1010]],
            },
            vec![true, false],
        )
        .expect("nullable binary vector column"),
    );
    replace_column(
        &mut columns,
        FieldData::nullable(
            FieldData::ArrayInt64 {
                name: "int64_array".into(),
                values: vec![vec![1, 2]],
            },
            vec![true, false],
        )
        .expect("nullable array column"),
    );
    replace_column(
        &mut columns,
        FieldData::nullable(
            FieldData::SparseFloatVector {
                name: common::SPARSE_VECTOR_FIELD.into(),
                values: vec![BTreeMap::from([(1, 0.5)])],
            },
            vec![true, false],
        )
        .expect("nullable sparse vector column"),
    );
    let request = InsertRequest::builder()
        .collection_name(&collection_name)
        .columns(columns)
        .build()
        .expect("build nullable column-based insert request");
    let response = client
        .insert(request)
        .await
        .expect("insert nullable columns");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop nullable column-insert collection");

    assert_eq!(response.insert_count().to_owned(), 2);
    assert!(response.failed_indices().is_empty());
}

fn replace_column(columns: &mut [FieldData], replacement: FieldData) {
    let name = replacement.name().to_owned();
    let column = columns
        .iter_mut()
        .find(|column| column.name() == name)
        .expect("column to replace");
    *column = replacement;
}

#[tokio::test]
async fn insert_advanced_types_by_rows() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("dml_rows");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;

    let request = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(common::advanced_rows())
        .build()
        .expect("build row-based insert request");
    let response = client.insert(request).await.expect("insert rows");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop row-insert collection");

    assert_eq!(response.insert_count().to_owned(), 2);
    assert!(response.failed_indices().is_empty());
}

#[tokio::test]
async fn insert_nullable_and_default_values_by_rows() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("dml_nullable_default_rows");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;

    let mut rows = common::advanced_rows();
    rows[0].extend([
        ("bool_value".into(), json!(false)),
        ("int8_value".into(), serde_json::Value::Null),
        ("int16_value".into(), serde_json::Value::Null),
        ("int32_value".into(), serde_json::Value::Null),
        ("float_value".into(), serde_json::Value::Null),
        ("double_value".into(), serde_json::Value::Null),
        ("varchar_value".into(), serde_json::Value::Null),
        ("int64_array".into(), json!([1, 2])),
        (common::BINARY_VECTOR_FIELD.into(), json!([170])),
        (common::SPARSE_VECTOR_FIELD.into(), json!({"1": 0.5})),
    ]);
    rows[1].extend([
        ("bool_value".into(), serde_json::Value::Null),
        ("int64_array".into(), serde_json::Value::Null),
        (common::BINARY_VECTOR_FIELD.into(), serde_json::Value::Null),
        (common::SPARSE_VECTOR_FIELD.into(), serde_json::Value::Null),
        (common::GEOMETRY_FIELD.into(), serde_json::Value::Null),
        (common::TIMESTAMPTZ_FIELD.into(), serde_json::Value::Null),
    ]);
    let request = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(rows)
        .build()
        .expect("build nullable/default row-based insert request");
    let response = client
        .insert(request)
        .await
        .expect("insert nullable/default rows");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop nullable/default row-insert collection");

    assert_eq!(response.insert_count().to_owned(), 2);
    assert!(response.failed_indices().is_empty());
}

#[tokio::test]
async fn upsert_existing_advanced_rows() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("upsert");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;

    let rows = common::advanced_rows();
    let insert = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(rows.clone())
        .build()
        .expect("build initial upsert data insert");
    client.insert(insert).await.expect("insert before upsert");

    let mut updated_rows = rows;
    updated_rows[0].insert(common::GEOMETRY_FIELD.to_owned(), json!("POINT (101 101)"));
    updated_rows[1].insert(
        common::TIMESTAMPTZ_FIELD.to_owned(),
        json!("2025-03-02T00:00:00+08:00"),
    );
    let upsert_data = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(updated_rows)
        .build()
        .expect("build row-based upsert data");
    let response = client
        .upsert(
            UpsertRequest::builder()
                .insert(upsert_data)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("upsert existing rows");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop upsert collection");

    assert_eq!(response.upsert_count().to_owned(), 2);
    assert!(response.failed_indices().is_empty());
}

#[tokio::test]
async fn partial_upsert_updates_only_supplied_normal_fields() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("partial_upsert");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
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
                .name("vector")
                .data_type(DataType::FloatVector)
                .dimension(2),
        )
        .add_field(
            FieldSchema::new()
                .name("note")
                .data_type(DataType::VarChar)
                .max_length(128),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection_name)
                .schema(schema)
                .consistency_level(ConsistencyLevel::Strong)
                .index_param(
                    IndexParam::new()
                        .field_name("vector")
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                )
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create partial-upsert collection");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(&collection_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load partial-upsert collection");

    let insert = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(vec![json!({
            "id": 1,
            "vector": [0.1, 0.2],
            "note": "before"
        })
        .as_object()
        .unwrap()
        .clone()])
        .build()
        .expect("build initial partial-upsert data");
    client
        .insert(insert)
        .await
        .expect("insert before partial upsert");

    let row = json!({
        "id": 1,
        "note": "after"
    })
    .as_object()
    .unwrap()
    .clone();
    let update = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(vec![row])
        .build()
        .expect("build partial-upsert row");
    let response = client
        .upsert(
            UpsertRequest::builder()
                .insert(update)
                .partial_update(true)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("partially upsert row");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection_name)
                .filter("id == 1")
                .output_fields(["vector", "note"])
                .limit(1)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query partially updated row");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop partial-upsert collection");

    assert_eq!(response.upsert_count().to_owned(), 1);
    assert!(response.failed_indices().is_empty());
    assert!(matches!(
        query
            .results()
            .get_output_fields()
            .iter()
            .find(|field| field.name() == "vector"),
        Some(FieldData::FloatVector { values, .. }) if values == &vec![vec![0.1, 0.2]]
    ));
    assert!(matches!(
        query
            .results()
            .get_output_fields()
            .iter()
            .find(|field| field.name() == "note"),
        Some(FieldData::VarChar { values, .. }) if values == &vec!["after".to_owned()]
    ));
}

#[tokio::test]
async fn per_field_upsert_appends_and_removes_array_elements() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("field_partial_update");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let upsert_array = |values: Vec<i64>, op_type| {
        UpsertRequest::builder()
            .insert(
                InsertRequest::builder()
                    .collection_name(&collection_name)
                    .columns(vec![
                        FieldData::Int64 {
                            name: common::ID_FIELD.to_owned(),
                            values: vec![1],
                        },
                        FieldData::array_int64(common::INT64_ARRAY_FIELD, vec![values]),
                    ])
                    .build()
                    .expect("valid field-operation payload"),
            )
            .add_field_op(
                FieldPartialUpdateOp::new()
                    .field_name(common::INT64_ARRAY_FIELD)
                    .op_type(op_type),
            )
            .build()
            .expect("valid field-operation upsert")
    };

    client
        .upsert(upsert_array(
            vec![3, 2],
            FieldPartialUpdateOpType::ArrayAppend,
        ))
        .await
        .expect("append array elements");
    let appended = query_int64_array(&client, &collection_name).await;
    assert_eq!(appended, vec![1, 2, 3, 2]);

    client
        .upsert(upsert_array(vec![2], FieldPartialUpdateOpType::ArrayRemove))
        .await
        .expect("remove array elements");
    let removed = query_int64_array(&client, &collection_name).await;
    assert_eq!(removed, vec![1, 3]);

    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop field-partial-update collection");
}

async fn query_int64_array(client: &milvus::v2::ClientV2, collection_name: &str) -> Vec<i64> {
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(collection_name)
                .filter("id == 1")
                .output_fields([common::INT64_ARRAY_FIELD])
                .limit(1)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid array query"),
        )
        .await
        .expect("query array field");
    response
        .results()
        .get_output_field(common::INT64_ARRAY_FIELD)
        .and_then(FieldData::as_array_int64)
        .and_then(|values| values.first().cloned())
        .expect("int64 array output")
}

#[tokio::test]
async fn upsert_advanced_types_by_columns() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("upsert_columns");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;

    let insert = InsertRequest::builder()
        .collection_name(&collection_name)
        .columns(common::advanced_columns())
        .build()
        .expect("build column-based upsert data");
    let response = client
        .upsert(
            UpsertRequest::builder()
                .insert(insert)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("upsert advanced columns");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop column-upsert collection");

    assert_eq!(response.upsert_count().to_owned(), 2);
    assert!(response.failed_indices().is_empty());
}

#[tokio::test]
async fn insert_and_upsert_array_struct_sparse() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("dml_array_struct_sparse");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("tags")
                .data_type(DataType::Array)
                .element_type(DataType::Int64)
                .max_capacity(8),
        )
        .add_field(
            FieldSchema::new()
                .name("sparse")
                .data_type(DataType::SparseFloatVector),
        )
        .add_struct_field(
            StructFieldSchema::new()
                .name("items")
                .max_capacity(4)
                .add_field(
                    FieldSchema::new()
                        .name("name")
                        .data_type(DataType::VarChar)
                        .max_length(64),
                )
                .add_field(FieldSchema::new().name("score").data_type(DataType::Int32)),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection_name)
                .schema(schema)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create Array/Struct/SparseFloatVector collection");

    let insert = InsertRequest::builder()
        .collection_name(&collection_name)
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::ArrayInt64 {
                name: "tags".into(),
                values: vec![vec![1, 2]],
            },
            FieldData::SparseFloatVector {
                name: "sparse".into(),
                values: vec![BTreeMap::from([(2, 0.5), (10, 0.25)])],
            },
            FieldData::Struct {
                name: "items".into(),
                values: vec![vec![[
                    ("name".into(), json!("first")),
                    ("score".into(), json!(10)),
                ]
                .into_iter()
                .collect()]],
            },
        ])
        .build()
        .expect("build Array/Struct/SparseFloatVector insert");
    let response = client.insert(insert).await.expect("insert all three types");
    assert_eq!(response.insert_count().to_owned(), 1);

    let rows = vec![json!({
        "id": 1,
        "tags": [3, 4],
        "sparse": {"indices": [4, 11], "values": [0.6, 0.3]},
        "items": [{
            "name": "updated",
            "score": 20
        }]
    })
    .as_object()
    .unwrap()
    .clone()];
    let upsert = InsertRequest::builder()
        .collection_name(&collection_name)
        .rows(rows)
        .build()
        .expect("build Array/Struct/SparseFloatVector upsert");
    let response = client
        .upsert(
            UpsertRequest::builder()
                .insert(upsert)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("upsert all three types");
    assert_eq!(response.upsert_count().to_owned(), 1);

    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop Array/Struct/SparseFloatVector collection");
}

#[tokio::test]
async fn delete_by_primary_key_filter() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("delete");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::create_advanced_collection(&client, &collection_name).await;
    let insert = InsertRequest::builder()
        .collection_name(&collection_name)
        .columns(common::advanced_columns())
        .build()
        .expect("build delete data insert");
    client.insert(insert).await.expect("insert before delete");

    let response = client
        .delete(
            DeleteRequest::builder()
                .collection_name(&collection_name)
                .filter("id in [1]")
                .build()
                .expect("build filter delete request"),
        )
        .await
        .expect("delete by primary key filter");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop delete collection");

    assert_eq!(response.delete_count().to_owned(), 1);
    assert!(response.timestamp() > 0);
}

#[tokio::test]
async fn delete_by_int64_and_varchar_primary_keys() {
    let client = common::client().await;

    let int_collection = common::unique_collection_name("delete_int_ids");
    let _int_cleanup = common::CollectionCleanup::new([&int_collection]);
    common::create_advanced_collection(&client, &int_collection).await;
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&int_collection)
                .columns(common::advanced_columns())
                .build()
                .expect("build Int64 delete data insert"),
        )
        .await
        .expect("insert Int64 delete data");
    let int_response = client
        .delete(
            DeleteRequest::builder()
                .collection_name(&int_collection)
                .ids(Ids::Int64(vec![1]))
                .build()
                .expect("build Int64 ID delete request"),
        )
        .await
        .expect("delete by Int64 IDs");
    common::drop_collection(&client, &int_collection)
        .await
        .expect("drop Int64 ID delete collection");
    assert_eq!(int_response.delete_count().to_owned(), 1);

    let string_collection = common::unique_collection_name("delete_string_ids");
    let _string_cleanup = common::CollectionCleanup::new([&string_collection]);
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name("key")
                .data_type(DataType::VarChar)
                .max_length(64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("vector")
                .data_type(DataType::FloatVector)
                .dimension(2),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&string_collection)
                .schema(schema)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create VarChar ID delete collection");
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&string_collection)
                .rows(vec![json!({
                    "key": "book-1",
                    "vector": [0.1, 0.2]
                })
                .as_object()
                .unwrap()
                .clone()])
                .build()
                .expect("build VarChar delete data insert"),
        )
        .await
        .expect("insert VarChar delete data");
    let string_response = client
        .delete(
            DeleteRequest::builder()
                .collection_name(&string_collection)
                .ids(Ids::VarChar(vec!["book-1".into()]))
                .build()
                .expect("build VarChar ID delete request"),
        )
        .await
        .expect("delete by VarChar IDs");
    common::drop_collection(&client, &string_collection)
        .await
        .expect("drop VarChar ID delete collection");
    assert_eq!(string_response.delete_count().to_owned(), 1);
}
