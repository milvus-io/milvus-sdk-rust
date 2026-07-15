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
use milvus::v2::array_f32_to_f16;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use serde_json::json;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_VECTOR_FP16";
    const PRIMARY: &str = "pk";
    const FLOAT16: &str = "vector_fp16";
    const BFLOAT16: &str = "vector_bf16";
    const TEXT: &str = "text";
    const DIMENSION: usize = 4;

    let client = client().await?;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .description("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(FLOAT16)
                .data_type(DataType::Float16Vector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name(BFLOAT16)
                .data_type(DataType::BFloat16Vector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name(TEXT)
                .data_type(DataType::VarChar)
                .max_length(100),
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
                .index_params(vec![
                    IndexParam::new()
                        .field_name(FLOAT16)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                    IndexParam::new()
                        .field_name(BFLOAT16)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                ])
                .build()?,
        )
        .await?;

    let mut fp16_sources = Vec::new();
    let mut bf16_sources = Vec::new();
    let rows: Vec<_> = (0..100)
        .map(|id| {
            let fp16 = float_vector(DIMENSION);
            let bf16 = float_vector(DIMENSION);
            fp16_sources.push(fp16.clone());
            bf16_sources.push(bf16.clone());
            json!({PRIMARY: id, TEXT: format!("hello world {id}"), FLOAT16: fp16, BFLOAT16: bf16})
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

    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    let first = 10usize;
    let second = 50usize;
    for id in [first, second] {
        println!("Original {FLOAT16} No.{id}: {:?}", fp16_sources[id]);
        println!("Original {BFLOAT16} No.{id}: {:?}", bf16_sources[id]);
    }
    let filter = format!("pk in [{first},{second}]");
    println!("Query with expression: {filter}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .output_fields([PRIMARY, TEXT, FLOAT16, BFLOAT16])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!("Searching the No.{first} and No.{second} vectors.");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(FLOAT16)
                .vectors(SearchVectors::Float16(vec![
                    array_f32_to_f16(&fp16_sources[first]),
                    array_f32_to_f16(&fp16_sources[second]),
                ]))
                .output_fields([FLOAT16])
                .limit(3)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
