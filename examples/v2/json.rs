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
    const COLLECTION: &str = "RUST_V2_JSON";
    const VECTOR: &str = "vector";
    const JSON_FIELD: &str = "json_field";
    const DIMENSION: usize = 128;

    let client = client().await?;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .description("user id")
                .data_type(DataType::Int64)
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .description("face signature")
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name(JSON_FIELD)
                .description("properties")
                .data_type(DataType::Json),
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
    let rows: Vec<_> = (0..10)
        .map(|i| {
            let vector = float_vector(DIMENSION);
            vectors.push(vector.clone());
            json!({
                JSON_FIELD: {"age": rng.gen_range(1..100), "name": format!("user_{i}")},
                VECTOR: vector,
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
    println!("{} rows inserted", insert.insert_count());

    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields([ID_FIELD, JSON_FIELD])
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("Successfully query.");
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
                .output_fields([ID_FIELD, JSON_FIELD])
                .limit(3)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
