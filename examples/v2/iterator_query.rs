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

const COLLECTION: &str = "RUST_V2_ITERATOR_QUERY";
const PRIMARY: &str = "user_id";
const NAME: &str = "user_name";
const AGE: &str = "user_age";
const FACE: &str = "user_face";
const TAGS: &str = "user_tags";
const DIMENSION: usize = 128;

async fn iterate(
    client: &ClientV2,
    row_count: u64,
    batch_size: usize,
    offset: i64,
    limit: i64,
    filter: &str,
) -> Result<()> {
    println!("=====================================================");
    println!("Iterate batch: {batch_size} offset: {offset} limit: {limit} filter: {filter}");
    let query = QueryRequest::builder()
        .collection_name(COLLECTION)
        .filter(filter)
        .output_fields([NAME, AGE, "a"])
        .offset(offset)
        .consistency_level(sdk::ConsistencyLevel::Bounded)
        .build()?;
    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(query)
                .batch_size(batch_size)
                .limit(limit)
                .build()?,
        )
        .await?;

    let mut ids = HashSet::new();
    let mut pages = 0;
    let mut total = 0usize;
    while let Some(page) = iterator.next().await? {
        let rows: Vec<_> = page.results().rows()?.collect::<Vec<_>>();
        if rows.is_empty() {
            break;
        }
        pages += 1;
        total += rows.len();
        println!("No.{pages} page {} rows fetched", rows.len());
        println!(
            "\tthe first row: {}",
            serde_json::Value::Object(rows.first().expect("non-empty page").to_entity_row()?)
        );
        println!(
            "\tthe last row: {}",
            serde_json::Value::Object(rows.last().expect("non-empty page").to_entity_row()?)
        );
        for row in rows {
            let id = row.get_i64(PRIMARY)?;
            ids.insert(id);
        }
    }
    println!("query iteration finished");
    if filter.is_empty() {
        let available = row_count.saturating_sub(offset as u64);
        let expected = if limit < 0 {
            available
        } else {
            available.min(limit as u64)
        };
        if ids.len() as u64 != expected {
            return Err(Error::Unexpected(format!(
                "returned row count is unexpected: {} returned vs expected {expected}",
                ids.len()
            )));
        }
    }
    println!("Total fetched rows: {total}");
    println!("=====================================================");
    Ok(())
}

async fn query_iterator_with_cursor(
    client: &ClientV2,
    batch_size: usize,
    limit: i64,
) -> Result<()> {
    println!("=====================================================");
    println!("Query iterator with resumable cursor, batch size: {batch_size} limit: {limit}");
    let query = QueryRequest::builder()
        .collection_name(COLLECTION)
        .output_fields([PRIMARY, NAME, AGE])
        .consistency_level(sdk::ConsistencyLevel::Bounded)
        .build()?;

    // Read the first page, capture the cursor, then resume a brand-new iterator from it.
    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(query.clone())
                .batch_size(batch_size)
                .limit(limit)
                .build()?,
        )
        .await?;
    let first_page = iterator
        .next()
        .await?
        .ok_or_else(|| Error::Unexpected("query iterator returned no first page".into()))?;
    let first_count = first_page.results().rows()?.count();
    println!("First page fetched {first_count} rows.");
    let cursor = iterator
        .cursor()
        .ok_or_else(|| Error::Unexpected("query iterator has no cursor".into()))?;

    let mut resumed = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(query)
                .batch_size(batch_size)
                .limit(limit)
                .cursor(cursor)
                .build()?,
        )
        .await?;
    let resumed_page = resumed
        .next()
        .await?
        .ok_or_else(|| Error::Unexpected("resumed query iterator returned no page".into()))?;
    let resumed_count = resumed_page.results().rows()?.count();
    println!("Resumed iterator fetched {resumed_count} rows.");
    println!("=====================================================");
    Ok(())
}

async fn query_iterator_element_filter(client: &ClientV2, batch_size: usize) -> Result<()> {
    println!("=====================================================");
    println!("Iterate element_filter(user_tags, tag == \"news\")");
    // element_filter evaluates its predicate per element of the array field and the query results
    // expand into one row per matched element, each carrying the element offset within the parent
    // array field.
    let filter = "element_filter(user_tags, tag == \"news\")";
    let query = QueryRequest::builder()
        .collection_name(COLLECTION)
        .filter(filter)
        .output_fields([PRIMARY, TAGS])
        .consistency_level(sdk::ConsistencyLevel::Bounded)
        .build()?;
    let mut iterator = client
        .query_iterator(
            QueryIteratorRequest::builder()
                .query(query)
                .batch_size(batch_size)
                .build()?,
        )
        .await?;

    let mut pages = 0;
    let mut total = 0usize;
    while let Some(page) = iterator.next().await? {
        let rows: Vec<_> = page.results().rows()?.collect::<Vec<_>>();
        pages += 1;
        total += rows.len();
        print!("No.{pages} page {} matched elements: ", rows.len());
        for row in rows {
            let id = row.get_i64(PRIMARY)?;
            let tags = row.get_array_varchar(TAGS)?;
            let offset = row
                .element_offset()
                .expect("element filter rows carry an element offset");
            print!("[id={id} tags={tags:?} offset={offset}] ");
        }
        println!();
    }
    println!("element-filter iteration finished");
    println!("Total fetched matched elements: {total}");
    println!("=====================================================");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
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
                .name(TAGS)
                .description("user tags")
                .data_type(DataType::Array)
                .element_type(DataType::VarChar)
                .max_capacity(10)
                .max_length(100),
        )
        .add_field(
            FieldSchema::new()
                .name(FACE)
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
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
                        .field_name(FACE)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::L2),
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
                // Some rows carry a "news" tag (some even twice) so the element_filter demo below
                // has matching elements to iterate over.
                let mut tags = vec!["sports".to_owned()];
                if offset % 3 == 0 {
                    tags.push("news".to_owned());
                }
                if offset % 7 == 0 {
                    tags.push("news".to_owned());
                }
                json!({PRIMARY: id, NAME: format!("my name is {id}"), AGE: offset % 100, FACE: float_vector(DIMENSION), "a": id, "b": format!("b is {id}"), TAGS: tags})
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

    iterate(&client, count, 3000, 25_000, 100_000, "").await?;
    iterate(&client, count, 25, 100, 80, "").await?;
    iterate(&client, count, 5000, 0, -1, "").await?;
    iterate(&client, count, 100, 0, -1, "user_age == 8").await?;
    iterate(&client, count, 1000, 15_000, 2500, "user_age > 30").await?;
    iterate(&client, count, 1000, 0, 100_000, "user_age in [30, 40, 50]").await?;
    query_iterator_with_cursor(&client, 100, 50).await?;
    query_iterator_element_filter(&client, 100).await?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
