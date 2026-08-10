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
    const COLLECTION: &str = "RUST_V2_DYNAMIC_FIELD";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "vector";
    const TEXT: &str = "text";
    const DIMENSION: usize = 4;
    const ROW_COUNT: i64 = 10;

    let client = client().await?;
    let schema = sdk::CollectionSchema::new()
        .enable_dynamic_field(true)
        .add_field(
            sdk::FieldSchema::new()
                .name(PRIMARY)
                .description("user id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(VECTOR)
                .data_type(sdk::DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(TEXT)
                .data_type(sdk::DataType::VarChar)
                .max_length(1024),
        );
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
                        .index_type(IndexType::IvfSq8)
                        .metric_type(MetricType::Ip),
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

    let ids = (0..ROW_COUNT).collect::<Vec<_>>();
    let texts = (0..ROW_COUNT).map(|i| format!("text_{i}")).collect();
    let vectors = (0..ROW_COUNT).map(|_| float_vector(DIMENSION)).collect();
    let dynamics = (0..ROW_COUNT)
        .map(|i| {
            if i % 2 == 0 {
                json!({"a": i, "b": format!("column-based insert value is {i}")})
            } else {
                json!({"a": i})
            }
        })
        .collect();
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .columns(vec![
                    FieldData::int64(PRIMARY, ids),
                    FieldData::varchar(TEXT, texts),
                    FieldData::float_vector(VECTOR, vectors),
                    FieldData::json("$meta", dynamics),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    let rows: Vec<_> = (0..ROW_COUNT)
        .map(|i| {
            let id = ROW_COUNT + i;
            json!({
                PRIMARY: id,
                TEXT: format!("this is text_{i}"),
                VECTOR: float_vector(DIMENSION),
                "a": id,
                "b": format!("row-based insert value is {id}"),
            })
        })
        .collect();
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by row-based.", insert.insert_count());

    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("pk == 2")
                .output_fields(["*"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .filter("a in [4, 7, 13, 18]")
                .output_fields([TEXT, "a", "b"])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
