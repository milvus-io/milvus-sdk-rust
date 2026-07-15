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
use milvus::v2::error::{Error, Result};
use milvus::v2::prelude::*;
use serde_json::json;
use std::collections::HashSet;
use utils::*;

const COLLECTION: &str = "RUST_V2_ITERATOR_SEARCH";
const PRIMARY: &str = "user_id";
const NAME: &str = "user_name";
const AGE: &str = "user_age";
const FACE: &str = "user_face";
const DIMENSION: usize = 128;

async fn row_count(client: &ClientV2) -> Result<u64> {
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    let count = query_count(response.results())?;
    println!("count(*) = {count}");
    Ok(count)
}

async fn build_collection(client: &ClientV2, metric: MetricType) -> Result<()> {
    let schema = CollectionSchema::new()
        .enable_dynamic_field(true)
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .description("user id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(NAME)
                .data_type(DataType::VarChar)
                .max_length(100),
        )
        .add_field(FieldSchema::new().name(AGE).data_type(DataType::Int8))
        .add_field(
            FieldSchema::new()
                .name(FACE)
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        );
    drop_collection(client, COLLECTION).await;
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
                        .field_name(FACE)
                        .index_type(IndexType::Flat)
                        .metric_type(metric),
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
    for seed in [50_000i64, 10_000, 30_000, 90_000, 0] {
        let rows: Vec<_> = (0..10_000)
            .map(|offset| {
                let id = seed + offset;
                json!({PRIMARY: id, NAME: format!("my name is {id}"), AGE: offset % 100, FACE: float_vector(DIMENSION), "a": id, "b": format!("b is {id}")})
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
        println!("{} rows inserted.", insert.insert_count());
    }
    row_count(client).await?;
    Ok(())
}

async fn iterate(
    client: &ClientV2,
    batch_size: usize,
    limit: Option<usize>,
    filter: &str,
) -> Result<()> {
    println!("=====================================================");
    println!(
        "Iterate batch: {batch_size} limit: {} filter: {filter}",
        limit.map_or(-1, |v| v as i64)
    );
    let search = SearchRequest::builder()
        .collection_name(COLLECTION)
        .vector_field(FACE)
        .vectors(SearchVectors::Float(vec![vec![1.0; DIMENSION]]))
        .filter(filter)
        .output_fields([NAME, AGE, "b"])
        .limit(batch_size as i64)
        .consistency_level(sdk::ConsistencyLevel::Bounded)
        .build()?;
    let mut builder = SearchIteratorRequest::builder()
        .search(search)
        .batch_size(batch_size);
    if let Some(limit) = limit {
        builder = builder.limit(limit);
    }
    let mut iterator = client.search_iterator(builder.build()?).await?;
    let mut ids = HashSet::new();
    let mut pages = 0;
    let mut total = 0usize;
    while let Some(page) = iterator.next().await? {
        let result =
            page.results().iter().next().ok_or_else(|| {
                Error::Unexpected("search iterator returned no result set".into())
            })?;
        let rows: Vec<_> = result.rows()?.collect::<Vec<_>>();
        if rows.is_empty() {
            break;
        }
        pages += 1;
        total += rows.len();
        println!("No.{pages} page {} rows fetched", rows.len());
        println!(
            "\tthe first row: {:?}",
            rows.first().expect("non-empty page").to_entity_row()?
        );
        println!(
            "\tthe last row: {:?}",
            rows.last().expect("non-empty page").to_entity_row()?
        );
        for row in rows {
            let id = row.get_i64(PRIMARY)?;
            ids.insert(id);
        }
    }
    println!("search iteration finished");
    if filter.is_empty() {
        let count = row_count(client).await? as usize;
        let expected = limit.map_or(count, |limit| limit.min(count));
        if ids.len() != expected {
            println!(
                "Returned row count is unexpected: {} returned vs limit {expected}",
                ids.len()
            );
            println!("Possible reason: equal vector distances cannot be distinguished by the search engine");
        }
    }
    println!("Total fetched rows: {total}");
    println!("=====================================================");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    for metric in [MetricType::L2, MetricType::Cosine] {
        build_collection(&client, metric).await?;
        iterate(&client, 3000, Some(100_000), "").await?;
        iterate(&client, 25, Some(80), "").await?;
        iterate(&client, 5000, None, "").await?;
        iterate(&client, 100, None, "user_age == 8").await?;
        iterate(&client, 1000, Some(2500), "user_age > 30").await?;
        iterate(&client, 1000, Some(100_000), "user_age in [30, 40, 50]").await?;
    }
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
