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

const COLLECTION: &str = "RUST_V2_TIMESTAMPTZ_FIELD";
const VECTOR: &str = "vector";
const TIMESTAMP: &str = "tsz";
const DIMENSION: usize = 4;

async fn query(client: &sdk::ClientV2, timezone: &str) -> Result<()> {
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields([TIMESTAMP])
                .limit(3)
                .timezone(timezone)
                .build()?,
        )
        .await?;
    println!("\nQuery results:");
    print_query_results(response.results())
}

async fn search(client: &sdk::ClientV2, timezone: &str) -> Result<()> {
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .output_fields([TIMESTAMP])
                .limit(3)
                .timezone(timezone)
                .build()?,
        )
        .await?;
    println!("\nSearch results:");
    print_search_results(response.results())
}

async fn hybrid(client: &sdk::ClientV2, timezone: &str) -> Result<()> {
    let sub = SubSearchRequest::builder()
        .vector_field(VECTOR)
        .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
        .limit(5)
        .timezone(timezone)
        .build()?;
    let response = client
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name(COLLECTION)
                .sub_requests(vec![sub])
                .rerank(RRFRerank::new().k(5))
                .output_fields([TIMESTAMP])
                .limit(3)
                .build()?,
        )
        .await?;
    println!("\nHybridSearch results:");
    print_search_results(response.results())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let schema = sdk::CollectionSchema::new()
        .enable_dynamic_field(true)
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
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
                .name(TIMESTAMP)
                .data_type(sdk::DataType::Timestamptz),
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
                    sdk::IndexParam::new()
                        .field_name(VECTOR)
                        .index_type(sdk::IndexType::Hnsw)
                        .metric_type(sdk::MetricType::L2),
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
    println!("\nInsert timezones");
    let rows: Vec<_> = (0..10)
        .map(|id| {
            let timestamp = format!("2025-01-{:02}T00:00:00+08:00", id + 1);
            println!("\t{timestamp}");
            json!({"id":id,VECTOR:float_vector(DIMENSION),TIMESTAMP:timestamp})
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
    for timezone in [
        "Asia/Shanghai",
        "America/Havana",
        "Africa/Bangui",
        "Australia/Sydney",
    ] {
        println!("\n================== Query with timezone: {timezone} ==================");
        query(&client, timezone).await?;
        search(&client, timezone).await?;
        hybrid(&client, timezone).await?;
    }
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
