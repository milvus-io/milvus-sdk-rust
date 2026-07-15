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

const DIMENSION: usize = 8;

async fn insert_null_vectors(client: &ClientV2) -> Result<()> {
    const COLLECTION: &str = "RUST_V2_NULLABLE_VECTOR_INSERT";
    const PRIMARY: &str = "id";
    const NAME: &str = "name";
    const VECTOR: &str = "embedding";
    println!("=== Demo 1: Insert null vectors ===");
    drop_collection(client, COLLECTION).await;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(NAME)
                .data_type(DataType::VarChar)
                .max_length(100),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32)
                .nullable(true),
        );
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
            CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::L2),
                )
                .build()?,
        )
        .await?;
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    let rows: Vec<_> = (0..100)
        .map(|id| {
            json!({PRIMARY: id, NAME: format!("item_{id}"), VECTOR: if id % 2 == 0 { json!(float_vector(DIMENSION)) } else { serde_json::Value::Null }})
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
    println!("Inserted {} rows: 50 valid, 50 null", insert.insert_count());

    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id >= 0")
                .output_fields([PRIMARY, VECTOR])
                .limit(110)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    let mut null_count = 0;
    let mut row_count = 0;
    println!("Query results:");
    for row in query.results().rows()? {
        row_count += 1;
        if row.is_null(VECTOR)? {
            null_count += 1;
        }
        println!("  {:?}", row.to_entity_row()?);
    }
    println!(
        "Query result: {} valid, {null_count} null",
        row_count - null_count
    );

    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .output_fields([PRIMARY, VECTOR])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;
    let hits = search.results().iter().next().map_or(0, SingleResult::len);
    println!("Search returned {hits} hits (only non-null vectors)");
    drop_collection(client, COLLECTION).await;
    println!();
    Ok(())
}

async fn add_nullable_vector_field(client: &ClientV2) -> Result<()> {
    const COLLECTION: &str = "RUST_V2_NULLABLE_VECTOR_ADD_FIELD";
    const PRIMARY: &str = "id";
    const NAME: &str = "name";
    const VECTOR_V1: &str = "embedding_v1";
    const VECTOR_V2: &str = "embedding_v2";
    println!("=== Demo 2: Add nullable vector field ===");
    drop_collection(client, COLLECTION).await;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(NAME)
                .data_type(DataType::VarChar)
                .max_length(100),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR_V1)
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        );
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
            CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR_V1)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::L2),
                )
                .build()?,
        )
        .await?;
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    let rows: Vec<_> = (0..10)
        .map(|id| {
            json!({PRIMARY: id, NAME: format!("item_{id}"), VECTOR_V1: float_vector(DIMENSION)})
        })
        .collect();
    client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    client
        .release_collection(
            ReleaseCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field(
                    FieldSchema::new()
                        .name(VECTOR_V2)
                        .data_type(DataType::FloatVector)
                        .dimension(DIMENSION as u32)
                        .nullable(true),
                )
                .build()?,
        )
        .await?;
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR_V2)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::L2),
                )
                .build()?,
        )
        .await?;
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id >= 0")
                .output_fields([PRIMARY, VECTOR_V1, VECTOR_V2])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("Old rows after adding {VECTOR_V2}:");
    for row in query.results().rows()? {
        println!(
            "  id={}, {VECTOR_V1}={}, {VECTOR_V2}={}",
            row.get_i64(PRIMARY)?,
            if !row.is_null(VECTOR_V1)? {
                "has value"
            } else {
                "null"
            },
            if !row.is_null(VECTOR_V2)? {
                "has value"
            } else {
                "null"
            },
        );
    }
    drop_collection(client, COLLECTION).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    insert_null_vectors(&client).await?;
    add_nullable_vector_field(&client).await?;
    println!("Done!");
    Ok(())
}
