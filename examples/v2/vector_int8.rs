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

fn primary_key(number: i64) -> String {
    format!("primary_key_{number}")
}

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_VECTOR_INT8";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "vector";
    const TEXT: &str = "text";
    const DIMENSION: usize = 16;

    let client = client().await?;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .data_type(DataType::VarChar)
                .primary_key(true)
                .max_length(128),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .data_type(DataType::Int8Vector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name(TEXT)
                .data_type(DataType::VarChar)
                .max_length(1024),
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
                        .index_type(IndexType::Hnsw)
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

    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .columns(vec![
                    FieldData::varchar(PRIMARY, vec![primary_key(10_000), primary_key(10_001)]),
                    FieldData::varchar(
                        TEXT,
                        vec!["column-based-1".into(), "column-based-2".into()],
                    ),
                    FieldData::int8_vector(
                        VECTOR,
                        vec![int8_vector(DIMENSION), int8_vector(DIMENSION)],
                    ),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    let vectors = (0..10).map(|_| int8_vector(DIMENSION)).collect::<Vec<_>>();
    let rows: Vec<_> = vectors
        .iter()
        .enumerate()
        .map(|(id, vector)| {
            json!({PRIMARY: primary_key(id as i64), TEXT: format!("this is text_{id}"), VECTOR: vector})
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
    println!("{} rows inserted by row-based.", insert.insert_count());

    let first = 3usize;
    let second = 4usize;
    let filter = format!(
        "pk in [\"{}\", \"{}\"]",
        primary_key(first as i64),
        primary_key(second as i64)
    );
    println!("Query with filter expression: {filter}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .output_fields([VECTOR, TEXT])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!(
        "Searching ID {} int8 vector: {:?}",
        primary_key(first as i64),
        vectors[first]
    );
    println!(
        "Searching ID {} int8 vector: {:?}",
        primary_key(second as i64),
        vectors[second]
    );
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .ids(Ids::VarChar(vec![
                    primary_key(first as i64),
                    primary_key(second as i64),
                ]))
                .output_fields([VECTOR, TEXT])
                .limit(3)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
