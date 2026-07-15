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
use serde_json::json;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_STRUCT_FIELD";
    const CLIPS: &str = "clips";
    const SIMPLE: &str = "simplify_clips";
    const FLOAT: &str = "clip_float_embedding";
    let client = client().await?;
    let clips = StructFieldSchema::new()
        .name(CLIPS)
        .max_capacity(10)
        .add_field(
            sdk::FieldSchema::new()
                .name("frame_number")
                .data_type(sdk::DataType::Int32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("clip_desc")
                .data_type(sdk::DataType::VarChar)
                .max_length(1024),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(FLOAT)
                .data_type(sdk::DataType::FloatVector)
                .dimension(16),
        );
    let simple = StructFieldSchema::new()
        .name(SIMPLE)
        .max_capacity(10)
        .add_field(
            sdk::FieldSchema::new()
                .name(FLOAT)
                .data_type(sdk::DataType::FloatVector)
                .dimension(32),
        );
    let schema = sdk::CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
                .description("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("film_name")
                .data_type(sdk::DataType::VarChar)
                .max_length(1024),
        )
        .add_struct_field(clips)
        .add_struct_field(simple);
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
                .index_params(vec![
                    IndexParam::new()
                        .field_name(format!("{CLIPS}[{FLOAT}]"))
                        .index_name("index_float")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::MaxSimIp),
                    IndexParam::new()
                        .field_name(format!("{SIMPLE}[{FLOAT}]"))
                        .index_name("index_simplify")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::L2),
                ])
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

    let mut float_lists = Vec::new();
    let mut simple_vectors = Vec::new();
    let rows: Vec<_> = (0..100)
        .map(|id| {
            let clips = (0..5)
                .map(|frame| {
                    let vector = float_vector(16);
                    if id < 10 {
                        float_lists.push((id, vector.clone()));
                    }
                    json!({"frame_number":frame,"clip_desc":format!("clip_description_{id}"),FLOAT:vector})
                })
                .collect::<Vec<_>>();
            let simplified = (0..2)
                .map(|_| {
                    let vector = float_vector(32);
                    if id == 5 {
                        simple_vectors.push(vector.clone());
                    }
                    json!({FLOAT:vector})
                })
                .collect::<Vec<_>>();
            json!({"id":id,"film_name":format!("film_{id}"),CLIPS:clips,SIMPLE:simplified})
        })
        .collect();
    let insert = client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by row-based.", insert.insert_count());
    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("count(*) = {}", query_count(count.results())?);
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id in [0, 5]")
                .output_fields(["id", CLIPS, SIMPLE])
                .limit(3)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    println!("\nQuery results with filter: id in [0, 5]");
    let mut query_row_count = 0;
    for row in query.results().rows()? {
        query_row_count += 1;
        let id = match row.get("id")? {
            ResultValue::Int64(value) => value,
            value => {
                println!("  unexpected id value: {value:?}");
                continue;
            }
        };
        println!("  id={id}");
        for field_name in [CLIPS, SIMPLE] {
            match row.get(field_name)? {
                ResultValue::Struct(values) => {
                    println!("    {field_name}: {} elements", values.len());
                    for (index, value) in values.iter().enumerate() {
                        let vector_dimension = value
                            .get(FLOAT)
                            .and_then(serde_json::Value::as_array)
                            .map_or(0, Vec::len);
                        println!(
                            "      element={index}, frame_number={:?}, clip_desc={:?}, vector_dimension={vector_dimension}",
                            value.get("frame_number"),
                            value.get("clip_desc")
                        );
                    }
                }
                ResultValue::Null => println!("    {field_name}=null"),
                value => println!("    {field_name} has unexpected value: {value:?}"),
            }
        }
    }
    let lists = [0i64, 5]
        .into_iter()
        .map(|id| {
            EmbeddingList::new().vectors(
                float_lists
                    .iter()
                    .filter(|(row, _)| *row == id)
                    .map(|(_, vector)| vector.clone())
                    .collect(),
            )
        })
        .collect();
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(format!("{CLIPS}[{FLOAT}]"))
                .vectors(SearchVectors::EmbeddingLists(lists))
                .output_fields(["film_name", "clips[frame_number]", "clips[clip_desc]"])
                .limit(3)
                .build()?,
        )
        .await?;
    println!("\nEmbeddingList search on struct field's {CLIPS}[{FLOAT}]");
    for (query_index, result) in response.results().iter().enumerate() {
        println!("  Query embedding list {query_index}:");
        for row in result.rows()? {
            let id = match row.get("id")? {
                ResultValue::Int64(value) => value,
                value => {
                    println!("    unexpected id value: {value:?}");
                    continue;
                }
            };
            let score = match row.get("score")? {
                ResultValue::Float(value) => value,
                value => {
                    println!("    unexpected score value: {value:?}");
                    continue;
                }
            };
            println!("    id={id}, score={score}");
            for field in result.get_output_fields() {
                let field_name = field.name();
                match row.get(field_name)? {
                    ResultValue::Int32(value) => println!("      {field_name}={value}"),
                    ResultValue::String(value) => println!("      {field_name}={value}"),
                    ResultValue::ArrayInt32(values) => {
                        println!("      {field_name}={values:?}")
                    }
                    ResultValue::ArrayVarChar(values) => {
                        println!("      {field_name}={values:?}")
                    }
                    ResultValue::Null => println!("      {field_name}=null"),
                    value => println!("      {field_name}={value:?}"),
                }
            }
        }
    }
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(format!("{SIMPLE}[{FLOAT}]"))
                .vectors(SearchVectors::Float(simple_vectors))
                .output_fields(["film_name"])
                .limit(3)
                .build()?,
        )
        .await?;
    println!("\nElement-level search on {SIMPLE}[{FLOAT}]");
    for (query_index, result) in response.results().iter().enumerate() {
        println!("  Query vector {query_index}:");
        for row in result.rows()? {
            let id = match row.get("id")? {
                ResultValue::Int64(value) => value,
                value => {
                    println!("    unexpected id value: {value:?}");
                    continue;
                }
            };
            let score = match row.get("score")? {
                ResultValue::Float(value) => value,
                value => {
                    println!("    unexpected score value: {value:?}");
                    continue;
                }
            };
            let film_name = match row.get("film_name")? {
                ResultValue::String(value) => value,
                value => {
                    println!("    unexpected film_name value: {value:?}");
                    continue;
                }
            };
            println!(
                "    id={id}, score={score}, element_offset={:?}, film_name={film_name}",
                row.element_offset()
            );
        }
    }
    println!("Decoded {query_row_count} query rows.");

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
