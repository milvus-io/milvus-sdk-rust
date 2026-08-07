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

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use rand::Rng;
use serde_json::json;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_ARRAY";
    const DIMENSION: usize = 128;
    const VECTOR: &str = "vector";
    const ARRAY_BOOL: &str = "field_array_bool";
    const ARRAY_INT8: &str = "field_array_int8";
    const ARRAY_INT16: &str = "array_int16_field";
    const ARRAY_INT32: &str = "field_array_int32";
    const ARRAY_INT64: &str = "field_array_int64";
    const ARRAY_FLOAT: &str = "field_array_float";
    const ARRAY_DOUBLE: &str = "field_array_double";
    const ARRAY_VARCHAR: &str = "field_array_varchar";

    let client = client().await?;
    let mut schema = CollectionSchema::new().enable_dynamic_field(false);
    schema = schema
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .description("user id")
                .data_type(DataType::VarChar)
                .primary_key(true)
                .max_length(64),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .description("face signature")
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        );
    for (name, element_type, description, capacity) in [
        (ARRAY_BOOL, DataType::Bool, "bool array", 10),
        (ARRAY_INT8, DataType::Int8, "int8 array", 10),
        (ARRAY_INT16, DataType::Int16, "int16 array", 10),
        (ARRAY_INT32, DataType::Int32, "int32 array", 10),
        (ARRAY_INT64, DataType::Int64, "int64 array", 10),
        (ARRAY_FLOAT, DataType::Float, "float array", 10),
        (ARRAY_DOUBLE, DataType::Double, "double array", 10),
        (ARRAY_VARCHAR, DataType::VarChar, "string array", 100),
    ] {
        let mut field = FieldSchema::new()
            .name(name)
            .description(description)
            .data_type(DataType::Array)
            .element_type(element_type)
            .max_capacity(capacity);
        if element_type == DataType::VarChar {
            field = field.max_length(1024);
        }
        schema = schema.add_field(field);
    }

    drop_collection(&client, COLLECTION).await;
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .build()?,
        )
        .await?;
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::Cosine),
                )
                .build()?,
        )
        .await?;
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    let mut rows = Vec::new();
    for i in 0..10 {
        let capacity = rng.gen_range(1..5);
        let vector = float_vector(DIMENSION);
        vectors.push(vector.clone());
        rows.push(
            json!({
                ID_FIELD: format!("user_{i}"),
                VECTOR: vector,
                ARRAY_BOOL: (0..capacity).map(|_| rng.gen()).collect::<Vec<bool>>(),
                ARRAY_INT8: (0..capacity).map(|_| rng.gen_range(0..100)).collect::<Vec<i8>>(),
                ARRAY_INT16: (0..capacity).map(|_| rng.gen_range(0..1000)).collect::<Vec<i16>>(),
                ARRAY_INT32: (0..capacity).map(|_| rng.gen_range(0..10000)).collect::<Vec<i32>>(),
                ARRAY_INT64: (0..capacity).map(|_| rng.gen_range(0..100000)).collect::<Vec<i64>>(),
                ARRAY_FLOAT: (0..capacity).map(|_| rng.gen_range(0.0..1.0)).collect::<Vec<f32>>(),
                ARRAY_DOUBLE: (0..capacity).map(|_| rng.gen_range(0.0..10.0)).collect::<Vec<f64>>(),
                ARRAY_VARCHAR: (0..capacity).map(|_| format!("varchar_{}", i * 10000 + rng.gen_range(0..100))).collect::<Vec<_>>(),
            }),
        );
    }
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted.", insert.insert_count());

    let output_fields = [
        ID_FIELD,
        ARRAY_BOOL,
        ARRAY_INT8,
        ARRAY_INT16,
        ARRAY_INT32,
        ARRAY_INT64,
        ARRAY_FLOAT,
        ARRAY_DOUBLE,
        ARRAY_VARCHAR,
    ];
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(output_fields)
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!("Searching the No.1 and No.8");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![
                    vectors[1].clone(),
                    vectors[8].clone(),
                ]))
                .output_fields(output_fields)
                .limit(3)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
