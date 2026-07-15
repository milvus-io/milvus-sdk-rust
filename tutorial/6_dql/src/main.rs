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

use milvus::v2::prelude::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const DIMENSION: usize = 4;

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".to_owned());
    let collection = tutorial_collection_name();
    let client = ClientV2::new(&ConnectConfig::new().uri(uri).token(token)).await?;

    let tutorial_result = demonstrate_dql(&client, &collection).await;
    let cleanup_result = cleanup_collection(&client, &collection).await;
    if let Err(error) = &cleanup_result {
        eprintln!("Failed to clean up {collection:?}: {error}");
    }
    tutorial_result?;
    cleanup_result
}

async fn demonstrate_dql(client: &ClientV2, collection: &str) -> Result<()> {
    create_load_and_insert(client, collection).await?;
    run_query(client, collection).await?;
    run_search(client, collection).await?;
    run_hybrid_search(client, collection).await?;
    run_query_iterator(client, collection).await?;
    run_search_iterator(client, collection).await
}

async fn create_load_and_insert(client: &ClientV2, collection: &str) -> Result<()> {
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("title")
                .data_type(DataType::VarChar)
                .max_length(256),
        )
        .add_field(
            FieldSchema::new()
                .name("category")
                .data_type(DataType::Int32),
        )
        .add_field(
            FieldSchema::new()
                .name("dense")
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name("sparse")
                .data_type(DataType::SparseFloatVector),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection)
                .schema(schema)
                .index_params(vec![
                    IndexParam::new()
                        .field_name("dense")
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::Cosine),
                    IndexParam::new()
                        .field_name("sparse")
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Ip),
                ])
                .build()?,
        )
        .await?;
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection)
                .sync(true)
                .timeout_ms(60_000)
                .build()?,
        )
        .await?;

    let rows = (0..12)
        .map(|id| {
            json!({
                "id": id,
                "title": format!("document_{id}"),
                "category": id % 3,
                "dense": dense_vector(id),
                "sparse": sparse_vector(id),
            })
        })
        .collect::<Vec<_>>();
    let response = client
        .insert(
            InsertRequest::builder()
                .collection_name(collection)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("Inserted {} tutorial rows", response.insert_count());
    Ok(())
}

async fn run_query(client: &ClientV2, collection: &str) -> Result<()> {
    println!("\nquery: category == 1");
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(collection)
                .filter("category == 1")
                .output_fields(["id", "title", "category"])
                .limit(5)
                .consistency_level(ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_rows(response.results())
}

async fn run_search(client: &ClientV2, collection: &str) -> Result<()> {
    println!("\nsearch: nearest neighbors in the dense field");
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(collection)
                .vector_field("dense")
                .vectors(SearchVectors::Float(vec![dense_vector(2)]))
                .filter("category >= 0")
                .output_fields(["title", "category"])
                .limit(3)
                .consistency_level(ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_search_rows(response.results())
}

async fn run_hybrid_search(client: &ClientV2, collection: &str) -> Result<()> {
    println!("\nhybrid_search: combine dense and sparse similarities");
    let dense = SubSearchRequest::builder()
        .vector_field("dense")
        .vectors(SearchVectors::Float(vec![dense_vector(2)]))
        .limit(6)
        .build()?;
    let sparse = SubSearchRequest::builder()
        .vector_field("sparse")
        .vectors(SearchVectors::SparseFloat(vec![sparse_vector(2)]))
        .limit(6)
        .build()?;
    let response = client
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name(collection)
                .sub_requests(vec![dense, sparse])
                .rerank(WeightedRerank::new().weights(vec![0.7, 0.3]))
                .output_fields(["title", "category"])
                .limit(4)
                .consistency_level(ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_search_rows(response.results())
}

async fn run_query_iterator(client: &ClientV2, collection: &str) -> Result<()> {
    println!("\nquery_iterator: batch_size=3, limit=7");
    let query = QueryRequest::builder()
        .collection_name(collection)
        .filter("id >= 0")
        .output_fields(["id", "title", "category"])
        .consistency_level(ConsistencyLevel::Strong)
        .build()?;
    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(query)
                .batch_size(3)
                .limit(7)
                .build()?,
        )
        .await?;
    let mut page_number = 0;
    while let Some(page) = iterator.next().await? {
        page_number += 1;
        println!("  page {page_number}");
        print_query_rows(page.results())?;
    }
    Ok(())
}

async fn run_search_iterator(client: &ClientV2, collection: &str) -> Result<()> {
    println!("\nsearch_iterator: batch_size=3, limit=7");
    let search = SearchRequest::builder()
        .collection_name(collection)
        .vector_field("dense")
        .vectors(SearchVectors::Float(vec![dense_vector(2)]))
        .output_fields(["title", "category"])
        .limit(3)
        .consistency_level(ConsistencyLevel::Strong)
        .build()?;
    let mut iterator = client
        .search_iterator(
            SearchIteratorRequest::builder()
                .search(search)
                .batch_size(3)
                .limit(7)
                .build()?,
        )
        .await?;
    let mut page_number = 0;
    while let Some(page) = iterator.next().await? {
        page_number += 1;
        println!("  page {page_number}");
        print_search_rows(page.results())?;
    }
    Ok(())
}

fn print_query_rows(results: &QueryResults) -> Result<()> {
    for row in results.rows()? {
        println!(
            "  id={}, title={:?}, category={}, generic_title={:?}",
            row.get_i64("id")?,
            row.get_str("title")?,
            row.get_i32("category")?,
            row.get("title")?
        );
    }
    Ok(())
}

fn print_search_rows(results: &SearchResults) -> Result<()> {
    for (query_index, result) in results.iter().enumerate() {
        println!("  result set for query vector {query_index}");
        for row in result.rows()? {
            println!(
                "    id={}, score={:.4}, title={:?}, category={}",
                row.get_i64("id")?,
                row.get_f32("score")?,
                row.get_str("title")?,
                row.get_i32("category")?
            );
        }
    }
    Ok(())
}

fn dense_vector(id: i64) -> Vec<f32> {
    let base = id as f32 / 10.0;
    vec![base, base + 0.1, base + 0.2, base + 0.3]
}

fn sparse_vector(id: i64) -> SparseVector {
    let mut vector = BTreeMap::new();
    vector.insert((id % 8) as u32, 1.0);
    vector.insert(((id + 3) % 8) as u32, 0.5);
    vector
}

async fn cleanup_collection(client: &ClientV2, collection: &str) -> Result<()> {
    let exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?
        .exists();
    if exists {
        let _ = client
            .release_collection(
                ReleaseCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await;
        client
            .drop_collection(
                DropCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await?;
    }
    Ok(())
}

fn tutorial_collection_name() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("RUST_V2_DQL_{timestamp}_{}", std::process::id())
}
