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
use std::collections::HashMap;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_PARTITION_KEY";
    const PRIMARY: &str = "id";
    const NAME: &str = "name";
    const VECTOR: &str = "vector";
    const DIMENSION: usize = 128;

    let client = client().await?;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .data_type(DataType::Int64)
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            FieldSchema::new()
                .name(NAME)
                .description("partition key")
                .data_type(DataType::VarChar)
                .max_length(100)
                .partition_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .description("embedding")
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        );
    drop_collection(&client, COLLECTION).await;
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .num_partitions(8)
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
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::Ip)
                        .extra_params(HashMap::from([
                            ("M".into(), "64".into()),
                            ("efConstruction".into(), "100".into()),
                        ])),
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

    let partitions = client
        .list_partitions(
            sdk::request::partition::ListPartitionsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("\nPartitions of {COLLECTION}:");
    for name in partitions.partition_names() {
        println!("\t{name}");
    }

    for group in 0..10 {
        let rows: Vec<_> = (0..1000)
            .map(
                |row| json!({NAME: format!("name_{group}_{row}"), VECTOR: float_vector(DIMENSION)}),
            )
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

    println!("\nQuery with expression: name == \"name_3_500\"");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("name == \"name_3_500\"")
                .output_fields([PRIMARY, NAME])
                .consistency_level(sdk::ConsistencyLevel::Eventually)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!("\nSearching with expression: name == \"name_3_500\"");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .filter("name == \"name_3_500\"")
                .output_fields([PRIMARY, NAME])
                .extra_params(HashMap::from([("ef".into(), "10".into())]))
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
