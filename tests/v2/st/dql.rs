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

use chrono::DateTime;
use milvus::v2::prelude::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

use super::common;

const CONSISTENCY_VECTOR_DIMENSION: u32 = 4;

#[tokio::test]
async fn query_decodes_advanced_types() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("query");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection_name)
                .filter("id in [1, 2]")
                .output_fields(common::advanced_load_fields())
                .timezone("Asia/Shanghai")
                .limit(2)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query advanced fields");

    let response_by_ids = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection_name)
                .ids(Ids::Int64(vec![1, 2]))
                .output_fields([common::ID_FIELD])
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid ID query request"),
        )
        .await
        .expect("query by primary-key IDs");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop query collection");

    let mut ids = response_by_ids
        .results()
        .rows()
        .expect("iterate ID query rows")
        .map(|row| row.get_i64(common::ID_FIELD).expect("Int64 primary key"))
        .collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!(ids, [1, 2]);

    common::assert_advanced_fields(response.results().get_output_fields(), 2);
    assert_all_array_types(response.results().get_output_fields(), 2);
    let borrowed_rows = response
        .results()
        .rows()
        .expect("iterate borrowed query rows")
        .collect::<Vec<_>>();
    assert_eq!(borrowed_rows.len(), 2);
    for row in borrowed_rows {
        assert_advanced_result_row(row, None);
    }
    let rows = response
        .results()
        .get_output_rows()
        .expect("materialize query rows");
    assert_advanced_output_rows(&rows, None, true);
    assert_eq!(
        response
            .results()
            .get_output_row(0)
            .expect("materialize first query row"),
        rows[0]
    );
}

#[tokio::test]
async fn session_and_strong_consistency_make_recent_writes_visible() {
    const ROW_COUNT: i64 = 20;

    let writer = common::client().await;
    let reader = common::client().await;

    for consistency_level in [ConsistencyLevel::Session, ConsistencyLevel::Strong] {
        let collection_name = common::unique_collection_name(match consistency_level {
            ConsistencyLevel::Session => "session_consistency",
            ConsistencyLevel::Strong => "strong_consistency",
            _ => unreachable!("the test covers session and strong consistency"),
        });
        let _cleanup = common::CollectionCleanup::new([&collection_name]);
        prepare_consistency_collection(&writer, &collection_name, consistency_level).await;

        for id in 0..ROW_COUNT {
            let vector = vec![id as f32, 1.0, 2.0, 3.0];
            writer
                .insert(
                    InsertRequest::builder()
                        .collection_name(&collection_name)
                        .columns(vec![
                            FieldData::Int64 {
                                name: common::ID_FIELD.into(),
                                values: vec![id],
                            },
                            FieldData::FloatVector {
                                name: common::VECTOR_FIELD.into(),
                                values: vec![vector.clone()],
                            },
                        ])
                        .build()
                        .expect("valid consistency insert"),
                )
                .await
                .expect("insert consistency row");

            let filter = format!("{} == {id}", common::ID_FIELD);
            if id % 3 == 0 {
                let response = reader
                    .query(
                        QueryRequest::builder()
                            .collection_name(&collection_name)
                            .filter(&filter)
                            .output_fields([common::ID_FIELD])
                            .limit(1)
                            .build()
                            .expect("valid consistency query"),
                    )
                    .await
                    .expect("query immediately after insert");
                let rows = response
                    .results()
                    .get_output_rows()
                    .expect("materialize consistency query rows");
                assert_single_consistency_row(&rows, id, consistency_level, "query");
            } else if id % 2 == 0 {
                let response = reader
                    .search(
                        SearchRequest::builder()
                            .collection_name(&collection_name)
                            .vector_field(common::VECTOR_FIELD)
                            .vectors(SearchVectors::Float(vec![vector.clone()]))
                            .filter(&filter)
                            .output_fields([common::ID_FIELD])
                            .limit(1)
                            .metric_type(MetricType::L2)
                            .build()
                            .expect("valid consistency search"),
                    )
                    .await
                    .expect("search immediately after insert");
                assert_eq!(response.results().len(), 1);
                let result = &response.results().get_results()[0];
                let rows = result
                    .get_output_rows()
                    .expect("materialize consistency search rows");
                assert_single_consistency_row(&rows, id, consistency_level, "search");
            } else {
                let sub_request = SubSearchRequest::builder()
                    .vector_field(common::VECTOR_FIELD)
                    .vectors(SearchVectors::Float(vec![vector]))
                    .filter(&filter)
                    .limit(1)
                    .metric_type(MetricType::L2)
                    .build()
                    .expect("valid consistency sub-search");
                let response = reader
                    .hybrid_search(
                        HybridSearchRequest::builder()
                            .collection_name(&collection_name)
                            .sub_requests(vec![sub_request])
                            .rerank(RRFRerank::new().k(20))
                            .output_fields([common::ID_FIELD])
                            .limit(1)
                            .build()
                            .expect("valid consistency hybrid search"),
                    )
                    .await
                    .expect("hybrid search immediately after insert");
                assert_eq!(response.results().len(), 1);
                let result = &response.results().get_results()[0];
                let rows = result
                    .get_output_rows()
                    .expect("materialize consistency hybrid-search rows");
                assert_single_consistency_row(&rows, id, consistency_level, "hybrid search");
            }
        }

        common::drop_collection(&writer, &collection_name)
            .await
            .expect("drop consistency collection");
    }
}

async fn prepare_consistency_collection(
    client: &milvus::v2::ClientV2,
    collection_name: &str,
    consistency_level: ConsistencyLevel,
) {
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(common::ID_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(common::VECTOR_FIELD)
                .data_type(DataType::FloatVector)
                .dimension(CONSISTENCY_VECTOR_DIMENSION),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection_name)
                .schema(schema)
                .consistency_level(consistency_level)
                .build()
                .expect("valid consistency collection"),
        )
        .await
        .expect("create consistency collection");
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(collection_name)
                .index_param(
                    IndexParam::new()
                        .field_name(common::VECTOR_FIELD)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::L2),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid consistency index"),
        )
        .await
        .expect("create consistency index");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection_name)
                .load_fields([common::ID_FIELD, common::VECTOR_FIELD])
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid consistency load"),
        )
        .await
        .expect("load consistency collection");
}

fn assert_single_consistency_row(
    rows: &[EntityRow],
    expected_id: i64,
    consistency_level: ConsistencyLevel,
    operation: &str,
) {
    assert_eq!(
        rows.len(),
        1,
        "{operation} with {consistency_level:?} consistency must see the recent insert"
    );
    assert_eq!(
        rows[0].get(common::ID_FIELD).and_then(Value::as_i64),
        Some(expected_id),
        "{operation} with {consistency_level:?} consistency returned the wrong row"
    );
}

fn assert_advanced_output_rows(
    rows: &[EntityRow],
    score_field_name: Option<&str>,
    require_all_fields: bool,
) {
    assert!(!rows.is_empty(), "search/query output must contain rows");
    let rows_by_id = rows_by_id(rows);
    assert_eq!(rows_by_id.len(), rows.len(), "output IDs must be unique");
    for (id, row) in rows_by_id {
        assert!([1, 2].contains(&id), "unexpected output row ID {id}");
        assert_advanced_output_row(row, id, score_field_name, require_all_fields);
    }
}

fn assert_advanced_output_row(
    row: &EntityRow,
    id: i64,
    score_field_name: Option<&str>,
    require_all_fields: bool,
) {
    let expected = expected_advanced_row(id);
    for (name, actual) in row {
        if score_field_name == Some(name.as_str()) {
            assert!(actual.is_number(), "search score {name} must be numeric");
            continue;
        }
        let expected_value = expected
            .get(name)
            .unwrap_or_else(|| panic!("unexpected output field {name} for inserted id {id}"));
        assert_json_value_eq(actual, expected_value, name);
    }

    if require_all_fields {
        assert_eq!(
            row.len(),
            expected.len() + usize::from(score_field_name.is_some()),
            "output row for inserted id {id} does not contain every requested field"
        );
    }
}

fn assert_advanced_result_row(row: ResultRow<'_>, score_field_name: Option<&str>) {
    assert!([1, 2].contains(&row.get_i64(common::ID_FIELD).unwrap()));
    row.get_bool("bool_value").expect("boolean result field");
    row.get_i8("int8_value").expect("int8 result field");
    row.get_i16("int16_value").expect("int16 result field");
    row.get_i32("int32_value").expect("int32 result field");
    row.get_f32("float_value").expect("float result field");
    row.get_f64("double_value").expect("double result field");
    row.get_str("varchar_value").expect("varchar result field");
    row.get_json("json_value").expect("JSON result field");
    row.get_array_bool(common::BOOL_ARRAY_FIELD)
        .expect("boolean-array result field");
    row.get_array_i8(common::INT8_ARRAY_FIELD)
        .expect("int8-array result field");
    row.get_array_i16(common::INT16_ARRAY_FIELD)
        .expect("int16-array result field");
    row.get_array_i32(common::INT32_ARRAY_FIELD)
        .expect("int32-array result field");
    row.get_array_i64(common::INT64_ARRAY_FIELD)
        .expect("int64-array result field");
    row.get_array_f32(common::FLOAT_ARRAY_FIELD)
        .expect("float-array result field");
    row.get_array_f64(common::DOUBLE_ARRAY_FIELD)
        .expect("double-array result field");
    row.get_array_varchar(common::VARCHAR_ARRAY_FIELD)
        .expect("varchar-array result field");
    row.get_float_vector(common::VECTOR_FIELD)
        .expect("float-vector result field");
    row.get_binary_vector(common::BINARY_VECTOR_FIELD)
        .expect("binary-vector result field");
    row.get_bfloat16_vector(common::BFLOAT16_VECTOR_FIELD)
        .expect("bfloat16-vector result field");
    row.get_sparse_float_vector(common::SPARSE_VECTOR_FIELD)
        .expect("sparse-vector result field");
    row.get_geometry(common::GEOMETRY_FIELD)
        .expect("geometry result field");
    row.get_timestamptz(common::TIMESTAMPTZ_FIELD)
        .expect("timestamptz result field");
    row.get_struct(common::STRUCT_FIELD)
        .expect("struct result field");
    if let Some(name) = score_field_name {
        row.get_f32(name).expect("search score");
    }
}

fn expected_advanced_row(id: i64) -> EntityRow {
    let value = match id {
        1 => json!({
            (common::ID_FIELD): 1,
            "bool_value": true,
            "int8_value": 8,
            "int16_value": 16,
            "int32_value": 32,
            "float_value": 1.5_f32,
            "double_value": 10.25,
            "varchar_value": "first",
            "json_value": {"rank": 1},
            (common::BOOL_ARRAY_FIELD): [true, false],
            (common::INT8_ARRAY_FIELD): [-8, 8],
            (common::INT16_ARRAY_FIELD): [-16, 16],
            (common::INT32_ARRAY_FIELD): [-32, 32],
            (common::INT64_ARRAY_FIELD): [1, 2],
            (common::FLOAT_ARRAY_FIELD): [1.25_f32, 2.5_f32],
            (common::DOUBLE_ARRAY_FIELD): [10.25, 20.5],
            (common::VARCHAR_ARRAY_FIELD): ["first", "second"],
            (common::VECTOR_FIELD): [0.1_f32, 0.2_f32, 0.3_f32, 0.4_f32],
            (common::BINARY_VECTOR_FIELD): [0b1010_1010_u8],
            (common::BFLOAT16_VECTOR_FIELD): common::bfloat16_vector([0.1, 0.2, 0.3, 0.4]),
            (common::SPARSE_VECTOR_FIELD): {"1": 0.5_f32, "7": 0.25_f32},
            (common::GEOMETRY_FIELD): "POINT (1 1)",
            (common::TIMESTAMPTZ_FIELD): "2025-01-01T00:00:00+08:00",
            (common::STRUCT_FIELD): [{"label": "created", "score": 10}],
        }),
        2 => json!({
            (common::ID_FIELD): 2,
            "bool_value": false,
            "int8_value": 9,
            "int16_value": 17,
            "int32_value": 33,
            "float_value": 2.5_f32,
            "double_value": 20.5,
            "varchar_value": "second",
            "json_value": {"rank": 2},
            (common::BOOL_ARRAY_FIELD): [false, true],
            (common::INT8_ARRAY_FIELD): [-9, 9],
            (common::INT16_ARRAY_FIELD): [-17, 17],
            (common::INT32_ARRAY_FIELD): [-33, 33],
            (common::INT64_ARRAY_FIELD): [3, 4],
            (common::FLOAT_ARRAY_FIELD): [3.75_f32, 4.5_f32],
            (common::DOUBLE_ARRAY_FIELD): [30.75, 40.125],
            (common::VARCHAR_ARRAY_FIELD): ["third", "fourth"],
            (common::VECTOR_FIELD): [0.4_f32, 0.3_f32, 0.2_f32, 0.1_f32],
            (common::BINARY_VECTOR_FIELD): [0b0101_0101_u8],
            (common::BFLOAT16_VECTOR_FIELD): common::bfloat16_vector([0.4, 0.3, 0.2, 0.1]),
            (common::SPARSE_VECTOR_FIELD): {"2": 0.75_f32, "9": 0.125_f32},
            (common::GEOMETRY_FIELD): "POINT (2 2)",
            (common::TIMESTAMPTZ_FIELD): "2025-01-02T00:00:00+08:00",
            (common::STRUCT_FIELD): [
                {"label": "created", "score": 20},
                {"label": "updated", "score": 21}
            ],
        }),
        _ => panic!("no advanced inserted data for id {id}"),
    };
    value
        .as_object()
        .expect("expected advanced row is a JSON object")
        .clone()
}

fn rows_by_id(rows: &[EntityRow]) -> HashMap<i64, &EntityRow> {
    rows.iter()
        .map(|row| {
            let id = row
                .get(common::ID_FIELD)
                .and_then(Value::as_i64)
                .expect("output row contains an integer id");
            (id, row)
        })
        .collect()
}

fn assert_json_value_eq(actual: &Value, expected: &Value, path: &str) {
    if path.rsplit('.').next() == Some(common::TIMESTAMPTZ_FIELD) {
        if let (Some(actual), Some(expected)) = (actual.as_str(), expected.as_str()) {
            let actual = DateTime::parse_from_rfc3339(actual)
                .unwrap_or_else(|error| panic!("invalid actual timestamp at {path}: {error}"));
            let expected = DateTime::parse_from_rfc3339(expected)
                .unwrap_or_else(|error| panic!("invalid expected timestamp at {path}: {error}"));
            assert_eq!(actual, expected, "mismatched timestamp at {path}");
            return;
        }
    }
    match (actual, expected) {
        (Value::Number(actual), Value::Number(expected)) => {
            if let (Some(actual), Some(expected)) = (actual.as_i64(), expected.as_i64()) {
                assert_eq!(actual, expected, "mismatched integer at {path}");
            } else if let (Some(actual), Some(expected)) = (actual.as_u64(), expected.as_u64()) {
                assert_eq!(actual, expected, "mismatched unsigned integer at {path}");
            } else {
                let actual = actual.as_f64().expect("actual JSON number is finite");
                let expected = expected.as_f64().expect("expected JSON number is finite");
                assert!(
                    (actual - expected).abs() <= 1e-6,
                    "mismatched floating-point value at {path}: expected {expected}, got {actual}"
                );
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            assert_eq!(
                actual.len(),
                expected.len(),
                "mismatched array size at {path}"
            );
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                assert_json_value_eq(actual, expected, &format!("{path}[{index}]"));
            }
        }
        (Value::Object(actual), Value::Object(expected)) => {
            assert_eq!(
                actual.len(),
                expected.len(),
                "mismatched object size at {path}"
            );
            for (name, expected) in expected {
                let actual = actual
                    .get(name)
                    .unwrap_or_else(|| panic!("missing object member {path}.{name}"));
                assert_json_value_eq(actual, expected, &format!("{path}.{name}"));
            }
        }
        _ => assert_eq!(actual, expected, "mismatched value at {path}"),
    }
}

fn assert_all_array_types(fields: &[FieldData], expected_rows: usize) {
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::BOOL_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayBool { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::INT8_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayInt8 { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::INT16_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayInt16 { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::INT32_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayInt32 { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::INT64_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayInt64 { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::FLOAT_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayFloat { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::DOUBLE_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayDouble { values, .. }) if values.len() == expected_rows
    ));
    assert!(matches!(
        fields
            .iter()
            .find(|field| field.name() == common::VARCHAR_ARRAY_FIELD)
            .map(FieldData::inner),
        Some(FieldData::ArrayVarChar { values, .. }) if values.len() == expected_rows
    ));
}

#[tokio::test]
async fn query_preserves_nullable_and_defaulted_fields() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("nullable_defaults");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;
    let mut rows = common::advanced_rows();
    rows[0].extend([
        ("json_value".into(), json!({"present": true})),
        ("int64_array".into(), json!([1, 2])),
    ]);
    rows[1].extend([
        ("json_value".into(), serde_json::Value::Null),
        ("int64_array".into(), serde_json::Value::Null),
        (common::GEOMETRY_FIELD.into(), serde_json::Value::Null),
        (common::TIMESTAMPTZ_FIELD.into(), serde_json::Value::Null),
    ]);
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&collection_name)
                .rows(rows)
                .build()
                .expect("build nullable/default row insert"),
        )
        .await
        .expect("insert nullable/default rows");
    client
        .flush(
            FlushRequest::builder()
                .collection_names([&collection_name])
                .wait_flushed_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("flush nullable/default rows");

    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection_name)
                .filter("id in [11, 12]")
                .output_fields([
                    common::ID_FIELD,
                    "bool_value",
                    "int8_value",
                    "int16_value",
                    "int32_value",
                    "float_value",
                    "double_value",
                    "varchar_value",
                    "json_value",
                    "int64_array",
                    common::GEOMETRY_FIELD,
                    common::TIMESTAMPTZ_FIELD,
                ])
                .limit(2)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query nullable and defaulted fields");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop nullable/default collection");

    for name in [
        "bool_value",
        "int8_value",
        "int16_value",
        "int32_value",
        "float_value",
        "double_value",
        "varchar_value",
        common::GEOMETRY_FIELD,
        common::TIMESTAMPTZ_FIELD,
    ] {
        let field = response
            .results()
            .get_output_fields()
            .iter()
            .find(|field| field.name() == name)
            .unwrap_or_else(|| panic!("missing defaulted field {name}"));
        assert_eq!(field.len(), 2);
        assert!(!field.is_null(0));
        assert!(!field.is_null(1));
        assert_eq!(field.inner().len(), 2);
    }

    for name in ["json_value", "int64_array"] {
        let field = response
            .results()
            .get_output_fields()
            .iter()
            .find(|field| field.name() == name)
            .unwrap_or_else(|| panic!("missing nullable field {name}"));
        assert_eq!(field.valid_data(), Some([true, false].as_slice()));
    }
    let array = response
        .results()
        .get_output_fields()
        .iter()
        .find(|field| field.name() == "int64_array")
        .unwrap();
    assert!(
        matches!(array.inner(), FieldData::ArrayInt64 { values, .. } if values == &vec![vec![1, 2]])
    );

    let output_rows = response
        .results()
        .get_output_rows()
        .expect("materialize nullable/default query rows");
    let output_rows = rows_by_id(&output_rows);
    let expected_rows = HashMap::from([
        (
            11,
            json!({
                (common::ID_FIELD): 11,
                "bool_value": true,
                "int8_value": 18,
                "int16_value": 116,
                "int32_value": 1116,
                "float_value": 1.5_f32,
                "double_value": 2.5,
                "varchar_value": "default",
                "json_value": {"present": true},
                "int64_array": [1, 2],
                (common::GEOMETRY_FIELD): "POINT (11 11)",
                (common::TIMESTAMPTZ_FIELD): "2025-02-01T00:00:00+08:00",
            }),
        ),
        (
            12,
            json!({
                (common::ID_FIELD): 12,
                "bool_value": true,
                "int8_value": 19,
                "int16_value": 117,
                "int32_value": 1117,
                "float_value": 1.5_f32,
                "double_value": 2.5,
                "varchar_value": "default",
                "json_value": null,
                "int64_array": null,
                (common::GEOMETRY_FIELD): "POINT (0 0)",
                (common::TIMESTAMPTZ_FIELD): "2025-01-01T00:00:00+08:00",
            }),
        ),
    ]);
    for (id, expected) in expected_rows {
        let actual = output_rows
            .get(&id)
            .unwrap_or_else(|| panic!("missing nullable/default output row for id {id}"));
        assert_json_value_eq(
            &Value::Object((*actual).clone()),
            &expected,
            &format!("row[{id}]"),
        );
    }
}

#[tokio::test]
async fn search_decodes_advanced_output_fields() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("search");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let float_response = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .vector_field(common::VECTOR_FIELD)
                .vectors(SearchVectors::Float(vec![
                    vec![0.1, 0.2, 0.3, 0.4],
                    vec![0.4, 0.3, 0.2, 0.1],
                ]))
                .output_fields(common::advanced_load_fields())
                .limit(2)
                .metric_type(MetricType::L2)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search advanced output fields");

    let binary_response = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .vector_field(common::BINARY_VECTOR_FIELD)
                .vectors(SearchVectors::Binary(vec![vec![0b1010_1010]]))
                .output_fields([common::BINARY_VECTOR_FIELD])
                .limit(2)
                .metric_type(MetricType::Hamming)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search binary vectors");

    let sparse_response = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .vector_field(common::SPARSE_VECTOR_FIELD)
                .vectors(SearchVectors::SparseFloat(vec![BTreeMap::from([
                    (1, 0.5),
                    (7, 0.25),
                ])]))
                .output_fields([common::SPARSE_VECTOR_FIELD])
                .limit(2)
                .metric_type(MetricType::Ip)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search sparse vectors");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop search collection");

    let float_results = float_response.results();
    assert_eq!(float_results.len(), 2);
    for result in float_results.get_results() {
        assert_eq!(result.len(), 2);
        common::assert_advanced_fields(result.get_output_fields(), 2);
        assert_all_array_types(result.get_output_fields(), 2);
        let borrowed_rows = result
            .rows()
            .expect("iterate borrowed search rows")
            .collect::<Vec<_>>();
        assert_eq!(borrowed_rows.len(), 2);
        for row in borrowed_rows {
            assert_advanced_result_row(row, Some(result.get_score_field_name()));
        }
        let rows = result.get_output_rows().expect("materialize search rows");
        assert_advanced_output_rows(&rows, Some(result.get_score_field_name()), true);
        assert_eq!(
            result
                .get_output_row(0)
                .expect("materialize first search row"),
            rows[0]
        );
    }
    assert!(matches!(
        binary_response.results().get_results()[0]
            .get_output_fields()
            .iter()
            .find(|field| field.name() == common::BINARY_VECTOR_FIELD)
            .map(FieldData::inner),
        Some(FieldData::BinaryVector { values, .. }) if values.len() == 2
    ));
    let binary_result = &binary_response.results().get_results()[0];
    let binary_rows = binary_result
        .get_output_rows()
        .expect("materialize binary search rows");
    assert_advanced_output_rows(
        &binary_rows,
        Some(binary_result.get_score_field_name()),
        false,
    );
    let sparse_result = &sparse_response.results().get_results()[0];
    assert!(!sparse_result.is_empty());
    assert!(matches!(
        sparse_result
            .get_output_fields()
            .iter()
            .find(|field| field.name() == common::SPARSE_VECTOR_FIELD)
            .map(FieldData::inner),
        Some(FieldData::SparseFloatVector { values, .. }) if values.len() == sparse_result.len()
    ));
    let sparse_rows = sparse_result
        .get_output_rows()
        .expect("materialize sparse search rows");
    assert_advanced_output_rows(
        &sparse_rows,
        Some(sparse_result.get_score_field_name()),
        false,
    );
}

#[tokio::test]
async fn search_with_ids_uses_stored_vectors_as_targets() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("search_ids");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .ids(Ids::Int64(vec![1, 2]))
                .vector_field(common::VECTOR_FIELD)
                .limit(1)
                .metric_type(MetricType::L2)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid search-with-IDs request"),
        )
        .await
        .expect("search using stored vectors selected by primary key");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop search-with-IDs collection");

    let results = response.results().get_results();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].get_ids(), &Ids::Int64(vec![1]));
    assert_eq!(results[1].get_ids(), &Ids::Int64(vec![2]));
}

#[tokio::test]
async fn get_by_primary_key_decodes_advanced_fields() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("get");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let response = client
        .get(
            GetRequest::builder()
                .collection_name(&collection_name)
                .ids(Ids::Int64(vec![1]))
                .output_fields(common::advanced_load_fields())
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get advanced fields by primary key");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop get collection");

    common::assert_advanced_fields(response.results().get_output_fields(), 1);
    assert_all_array_types(response.results().get_output_fields(), 1);
    let rows = response
        .results()
        .get_output_rows()
        .expect("materialize get rows");
    assert_eq!(rows.len(), 1);
    assert_advanced_output_row(&rows[0], 1, None, true);
}

#[tokio::test]
async fn search_and_query_decode_float16_and_int8_vectors() {
    const FLOAT16_FIELD: &str = "float16_vector";
    const INT8_FIELD: &str = "int8_vector";

    let client = common::client().await;
    let collection_name = common::unique_collection_name("dql_remaining_vectors");
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
                .name(FLOAT16_FIELD)
                .data_type(DataType::Float16Vector)
                .dimension(4),
        )
        .add_field(
            FieldSchema::new()
                .name(INT8_FIELD)
                .data_type(DataType::Int8Vector)
                .dimension(4),
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
        .expect("create Float16/Int8 vector collection");

    let float16_values = milvus::v2::array_f32_to_f16(&[0.1, 0.2, 0.3, 0.4]);
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&collection_name)
                .columns(vec![
                    FieldData::Int64 {
                        name: "id".into(),
                        values: vec![1],
                    },
                    FieldData::Float16Vector {
                        name: FLOAT16_FIELD.into(),
                        values: vec![float16_values.clone()],
                    },
                    FieldData::Int8Vector {
                        name: INT8_FIELD.into(),
                        values: vec![vec![-8, -1, 1, 8]],
                    },
                ])
                .build()
                .expect("build Float16/Int8 vector insert"),
        )
        .await
        .expect("insert Float16/Int8 vectors");
    client
        .flush(
            FlushRequest::builder()
                .collection_names([&collection_name])
                .wait_flushed_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("flush Float16/Int8 vectors");
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(&collection_name)
                .index_param(
                    IndexParam::new()
                        .field_name(FLOAT16_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(INT8_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("index Float16/Int8 vectors");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(&collection_name)
                .load_fields(["id", FLOAT16_FIELD, INT8_FIELD])
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load Float16/Int8 vector collection");

    let float16_search = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .vector_field(FLOAT16_FIELD)
                .vectors(SearchVectors::Float16(vec![float16_values]))
                .output_fields([FLOAT16_FIELD, INT8_FIELD])
                .limit(1)
                .metric_type(MetricType::Cosine)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search Float16 vectors");
    let int8_search = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .vector_field(INT8_FIELD)
                .vectors(SearchVectors::Int8(vec![vec![-8, -1, 1, 8]]))
                .output_fields([FLOAT16_FIELD, INT8_FIELD])
                .limit(1)
                .metric_type(MetricType::Cosine)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search Int8 vectors");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection_name)
                .filter("id == 1")
                .output_fields(["id", FLOAT16_FIELD, INT8_FIELD])
                .limit(1)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query Float16/Int8 vector fields");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop Float16/Int8 vector collection");

    for fields in [
        float16_search.results().get_results()[0].get_output_fields(),
        int8_search.results().get_results()[0].get_output_fields(),
        query.results().get_output_fields(),
    ] {
        assert!(matches!(
            fields.iter().find(|field| field.name() == FLOAT16_FIELD),
            Some(FieldData::Float16Vector { values, .. }) if values.len() == 1
        ));
        assert!(matches!(
            fields.iter().find(|field| field.name() == INT8_FIELD),
            Some(FieldData::Int8Vector { values, .. }) if values.len() == 1
        ));
    }

    let expected = json!({
        "id": 1,
        (FLOAT16_FIELD): milvus::v2::array_f32_to_f16(&[0.1, 0.2, 0.3, 0.4]),
        (INT8_FIELD): [-8, -1, 1, 8],
    });
    let query_rows = query
        .results()
        .get_output_rows()
        .expect("materialize Float16/Int8 query rows");
    assert_eq!(query_rows.len(), 1);
    assert_json_value_eq(
        &Value::Object(query_rows[0].clone()),
        &expected,
        "Float16/Int8 query row",
    );
    for result in [
        &float16_search.results().get_results()[0],
        &int8_search.results().get_results()[0],
    ] {
        let rows = result
            .get_output_rows()
            .expect("materialize Float16/Int8 search rows");
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0]
                .get(result.get_score_field_name())
                .is_some_and(Value::is_number),
            "Float16/Int8 search score must be numeric"
        );
        let mut expected = expected
            .as_object()
            .expect("expected vector row is an object")
            .clone();
        expected.insert(
            result.get_score_field_name().to_owned(),
            rows[0][result.get_score_field_name()].clone(),
        );
        assert_json_value_eq(
            &Value::Object(rows[0].clone()),
            &Value::Object(expected),
            "Float16/Int8 search row",
        );
    }
}

#[tokio::test]
async fn insert_query_and_search_struct_vector_subfield() {
    const STRUCT_FIELD: &str = "events";
    const LABEL_FIELD: &str = "label";
    const EMBEDDING_FIELD: &str = "embedding";
    const STRUCT_VECTOR_FIELD: &str = "events[embedding]";

    let client = common::client().await;
    let collection_name = common::unique_collection_name("struct_vector_search");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_struct_field(
            StructFieldSchema::new()
                .name(STRUCT_FIELD)
                .max_capacity(4)
                .add_field(
                    FieldSchema::new()
                        .name(LABEL_FIELD)
                        .data_type(DataType::VarChar)
                        .max_length(64),
                )
                .add_field(
                    FieldSchema::new()
                        .name(EMBEDDING_FIELD)
                        .data_type(DataType::FloatVector)
                        .dimension(4),
                ),
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
        .expect("create struct-vector collection");

    let struct_item = |label: &str, embedding: [f32; 4]| {
        json!({
            (LABEL_FIELD): label,
            (EMBEDDING_FIELD): embedding,
        })
        .as_object()
        .expect("struct item is a JSON object")
        .clone()
    };
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&collection_name)
                .columns(vec![
                    FieldData::Int64 {
                        name: "id".into(),
                        values: vec![1],
                    },
                    FieldData::Struct {
                        name: STRUCT_FIELD.into(),
                        values: vec![vec![
                            struct_item("column-first", [1.0, 0.0, 0.0, 0.0]),
                            struct_item("column-second", [0.0, 1.0, 0.0, 0.0]),
                        ]],
                    },
                ])
                .build()
                .expect("build column struct-vector insert"),
        )
        .await
        .expect("insert struct vectors by columns");
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&collection_name)
                .rows(vec![json!({
                    "id": 2,
                    (STRUCT_FIELD): [
                        {
                            (LABEL_FIELD): "row-first",
                            (EMBEDDING_FIELD): [0.9, 0.1, 0.0, 0.0]
                        },
                        {
                            (LABEL_FIELD): "row-second",
                            (EMBEDDING_FIELD): [0.0, 0.9, 0.1, 0.0]
                        }
                    ]
                })
                .as_object()
                .expect("row struct item is a JSON object")
                .clone()])
                .build()
                .expect("build row struct-vector insert"),
        )
        .await
        .expect("insert struct vectors by rows");
    client
        .flush(
            FlushRequest::builder()
                .collection_names([&collection_name])
                .wait_flushed_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("flush struct vectors");
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(&collection_name)
                .index_param(
                    IndexParam::new()
                        .field_name(STRUCT_VECTOR_FIELD)
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::MaxSimCosine)
                        .extra_params(HashMap::from([
                            ("M".into(), "16".into()),
                            ("efConstruction".into(), "200".into()),
                        ])),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("index struct vector subfield");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(&collection_name)
                .load_fields(["id", STRUCT_FIELD])
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load struct-vector collection");

    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection_name)
                .filter("id in [1, 2]")
                .output_fields(["id", STRUCT_FIELD])
                .limit(2)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("query struct vector subfield");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection_name)
                .vector_field(STRUCT_VECTOR_FIELD)
                .vectors(SearchVectors::EmbeddingLists(vec![EmbeddingList::new()
                    .vectors(vec![
                        vec![1.0, 0.0, 0.0, 0.0],
                        vec![0.0, 1.0, 0.0, 0.0],
                    ])]))
                .output_fields([STRUCT_FIELD])
                .limit(2)
                .metric_type(MetricType::MaxSimCosine)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("search struct vector subfield");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop struct-vector collection");

    assert_struct_vector_values(
        query.results().get_output_fields(),
        2,
        STRUCT_FIELD,
        EMBEDDING_FIELD,
    );
    assert_eq!(search.results().len().to_owned(), 1);
    assert_struct_vector_values(
        search.results().get_results()[0].get_output_fields(),
        2,
        STRUCT_FIELD,
        EMBEDDING_FIELD,
    );

    let expected_struct_rows = HashMap::from([
        (
            1,
            json!([
                {"label": "column-first", "embedding": [1.0_f32, 0.0, 0.0, 0.0]},
                {"label": "column-second", "embedding": [0.0_f32, 1.0, 0.0, 0.0]}
            ]),
        ),
        (
            2,
            json!([
                {"label": "row-first", "embedding": [0.9_f32, 0.1, 0.0, 0.0]},
                {"label": "row-second", "embedding": [0.0_f32, 0.9, 0.1, 0.0]}
            ]),
        ),
    ]);
    let query_rows = query
        .results()
        .get_output_rows()
        .expect("materialize struct-vector query rows");
    assert_struct_output_rows(&query_rows, &expected_struct_rows, None);
    let search_result = &search.results().get_results()[0];
    let search_rows = search_result
        .get_output_rows()
        .expect("materialize struct-vector search rows");
    assert_struct_output_rows(
        &search_rows,
        &expected_struct_rows,
        Some(search_result.get_score_field_name()),
    );
}

fn assert_struct_output_rows(
    rows: &[EntityRow],
    expected_struct_rows: &HashMap<i64, Value>,
    score_field_name: Option<&str>,
) {
    assert_eq!(rows.len(), expected_struct_rows.len());
    for (id, row) in rows_by_id(rows) {
        assert_eq!(
            row.len(),
            2 + usize::from(score_field_name.is_some()),
            "unexpected fields in struct-vector output row for id {id}"
        );
        if let Some(score_field_name) = score_field_name {
            assert!(
                row.get(score_field_name).is_some_and(Value::is_number),
                "struct-vector search score must be numeric"
            );
        }
        assert_json_value_eq(
            row.get("events")
                .expect("struct output row contains events"),
            expected_struct_rows
                .get(&id)
                .unwrap_or_else(|| panic!("unexpected struct-vector row id {id}")),
            &format!("events for id {id}"),
        );
    }
}

fn assert_struct_vector_values(
    fields: &[FieldData],
    expected_rows: usize,
    struct_field: &str,
    embedding_field: &str,
) {
    let field = fields
        .iter()
        .find(|field| field.name() == struct_field)
        .expect("struct output field");
    let FieldData::Struct { values, .. } = field else {
        panic!("expected struct field data")
    };
    assert_eq!(values.len(), expected_rows);
    assert!(values.iter().all(|row| {
        !row.is_empty()
            && row.iter().all(|item| {
                item.get(embedding_field)
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|embedding| embedding.len() == 4)
            })
    }));
}

#[tokio::test]
async fn hybrid_search_combines_sub_searches() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("hybrid_search");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let first_search = SubSearchRequest::builder()
        .vector_field(common::VECTOR_FIELD)
        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2, 0.3, 0.4]]))
        .limit(2)
        .metric_type(MetricType::L2)
        .build()
        .expect("valid request");
    let second_search = SubSearchRequest::builder()
        .vector_field(common::BFLOAT16_VECTOR_FIELD)
        .vectors(SearchVectors::BFloat16(vec![common::bfloat16_vector([
            0.4, 0.3, 0.2, 0.1,
        ])]))
        .limit(2)
        .metric_type(MetricType::L2)
        .build()
        .expect("valid request");
    let response = client
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name(&collection_name)
                .sub_requests(vec![first_search, second_search])
                .rerank(RRFRerank::new().k(60))
                .limit(2)
                .output_fields(common::advanced_load_fields())
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("hybrid search with RRF ranking");
    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop hybrid-search collection");

    let results = response.results();
    assert_eq!(results.len(), 1);
    let result = &results.get_results()[0];
    common::assert_advanced_fields(result.get_output_fields(), 2);
    assert_all_array_types(result.get_output_fields(), 2);
    let rows = result
        .get_output_rows()
        .expect("materialize hybrid-search rows");
    assert_advanced_output_rows(&rows, Some(result.get_score_field_name()), true);
}
