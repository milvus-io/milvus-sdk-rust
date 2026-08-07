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

const COLLECTION: &str = "RUST_V2_GEOMETRY_FIELD";
const VECTOR: &str = "vector";
const GEO: &str = "geo";
const DIMENSION: usize = 4;

async fn query(client: &sdk::ClientV2, filter: &str) -> Result<()> {
    println!("\n========= Query with filter: {filter}");
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .output_fields(["*"])
                .build()?,
        )
        .await?;
    print_query_results(response.results())
}

async fn search(client: &sdk::ClientV2, filter: &str) -> Result<()> {
    println!("\n========= Search with filter: {filter}");
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .filter(filter)
                .output_fields([GEO])
                .limit(20)
                .build()?,
        )
        .await?;
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
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(VECTOR)
                .data_type(sdk::DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(GEO)
                .data_type(sdk::DataType::Geometry),
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
    for geometry in [
        "POINT (1 1)",
        "LINESTRING (10 10, 10 30, 40 40)",
        "POLYGON ((0 100, 100 100, 100 50, 0 50, 0 100))",
    ] {
        let insert = client
            .insert(
                sdk::request::dml::InsertRequest::builder()
                    .collection_name(COLLECTION)
                    .row(json!({VECTOR:float_vector(DIMENSION),GEO:geometry}))
                    .build()?,
            )
            .await?;
        println!("{} rows inserted by row-based.", insert.insert_count());
    }
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
    let filters = [
        "ST_EQUALS(geo, 'POINT (1 1)')",
        "ST_TOUCHES(geo, 'LINESTRING (0 50, 0 100)')",
        "ST_CONTAINS(geo, 'POINT (70 70)')",
        "ST_CROSSES(geo, 'LINESTRING (20 0, 20 100)')",
        "ST_WITHIN(geo, 'POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))')",
    ];
    for filter in filters {
        query(&client, filter).await?;
    }
    for filter in filters {
        search(&client, filter).await?;
    }
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
