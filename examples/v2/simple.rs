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
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let collection = "RUST_V2_SIMPLE";
    const DIMENSION: usize = 128;
    const ROW_COUNT: i64 = 100;
    const PRIMARY_FIELD: &str = "pk";
    const VECTOR_FIELD: &str = "embedding";
    drop_collection(&client, collection).await;
    client
        .create_collection(
            sdk::request::collection::CreateSimpleCollectionRequest::builder()
                .collection_name(collection)
                .primary_field(PRIMARY_FIELD)
                .vector_field(VECTOR_FIELD)
                .dimension(DIMENSION as u32)
                .build()?,
        )
        .await?;
    let rows: Vec<_> = (0..ROW_COUNT)
        .map(|id| serde_json::json!({PRIMARY_FIELD: id, VECTOR_FIELD: float_vector(DIMENSION)}))
        .collect();
    let insert = client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(collection)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by row-based.", insert.insert_count());

    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(collection)
                .vector_field(VECTOR_FIELD)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .output_fields([VECTOR_FIELD])
                .limit(3)
                .consistency_level(ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("Search results:");
    for (query_index, result) in search.results().iter().enumerate() {
        println!("  Query vector {query_index}:");
        for row in result.rows()? {
            let id = match row.get(PRIMARY_FIELD)? {
                ResultValue::Int64(value) => value,
                value => {
                    println!("    unexpected primary-key value: {value:?}");
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
            let dimension = match row.get(VECTOR_FIELD)? {
                ResultValue::FloatVector(value) => value.len(),
                value => {
                    println!("    unexpected vector value: {value:?}");
                    continue;
                }
            };
            println!("    id={id}, score={score}, vector_dimension={dimension}");
        }
    }

    let get = client
        .get(
            GetRequest::builder()
                .collection_name(collection)
                .ids(Ids::Int64(vec![5, 1, 10, 8]))
                .output_fields([PRIMARY_FIELD, VECTOR_FIELD])
                .build()?,
        )
        .await?;
    println!("Query results:");
    for row in get.results().rows()? {
        for field_name in [PRIMARY_FIELD, VECTOR_FIELD] {
            match row.get(field_name)? {
                ResultValue::Int64(value) => println!("  {field_name}={value}"),
                ResultValue::FloatVector(value) => {
                    println!("  {field_name}=FloatVector(dimension={})", value.len())
                }
                ResultValue::Null => println!("  {field_name}=null"),
                value => println!("  {field_name}={value:?}"),
            }
        }
        println!();
    }

    drop_collection(&client, collection).await;
    Ok(())
}
