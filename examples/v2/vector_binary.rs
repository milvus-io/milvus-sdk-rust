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
use std::collections::HashMap;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_VECTOR_BINARY";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "vector";
    const TEXT: &str = "text";
    const DIMENSION: usize = 128;

    let client = client().await?;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .data_type(DataType::BinaryVector)
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
                        .index_type(IndexType::BinIvfFlat)
                        .metric_type(MetricType::Hamming)
                        .extra_params(HashMap::from([("nlist".into(), "5".into())])),
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
                    FieldData::int64(PRIMARY, vec![10_000, 10_001]),
                    FieldData::varchar(
                        TEXT,
                        vec!["column-based-1".into(), "column-based-2".into()],
                    ),
                    FieldData::binary_vector(
                        VECTOR,
                        vec![binary_vector(DIMENSION), binary_vector(DIMENSION)],
                    ),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    let vectors = (0..10)
        .map(|_| binary_vector(DIMENSION))
        .collect::<Vec<_>>();
    let rows: Vec<_> = vectors
        .iter()
        .enumerate()
        .map(|(id, vector)| {
            json!({PRIMARY: id as i64, TEXT: format!("row-based-{id}"), VECTOR: vector})
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

    let query_ids = [1usize, 8usize];
    let filter = format!("pk in [{}, {}]", query_ids[0], query_ids[1]);
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
    for row in query.results().rows()? {
        println!("\tRow: {:?}", row.to_entity_row()?);
        let id = row.get_i64(PRIMARY)?;
        let output = row.get_binary_vector(VECTOR)?;
        if output != vectors[id as usize] {
            return Err(Error::Unexpected(
                "output binary vector differs from inserted vector".into(),
            ));
        }
    }

    for id in query_ids {
        println!("Searching the ID.{id} binary vector: {:?}", vectors[id]);
    }
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Binary(vec![
                    vectors[query_ids[0]].clone(),
                    vectors[query_ids[1]].clone(),
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
