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
use rand::Rng;
use serde_json::json;
use std::collections::BTreeMap;
use utils::*;

fn sparse_vector(dimension: u32) -> SparseVector {
    let mut rng = rand::thread_rng();
    let mut vector = BTreeMap::new();
    while vector.len() < 10 {
        vector.insert(rng.gen_range(0..dimension), rng.gen_range(0.0..1.0));
    }
    vector
}

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_VECTOR_SPARSE";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "sparse";
    const TEXT: &str = "text";

    let client = client().await?;
    let schema = CollectionSchema::new()
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
                .description("user id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR)
                .data_type(DataType::SparseFloatVector),
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
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Ip),
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
                    FieldData::sparse_float_vector(
                        VECTOR,
                        vec![sparse_vector(100), sparse_vector(100)],
                    ),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    let vectors = (0..10).map(|_| sparse_vector(100)).collect::<Vec<_>>();
    let rows: Vec<_> = vectors
        .iter()
        .enumerate()
        .map(|(id, vector)| {
            json!({PRIMARY: id as i64, TEXT: format!("this is text_{id}"), VECTOR: vector})
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

    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields([VECTOR, TEXT])
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!("Searching the ID.1 sparse vector: {:?}", vectors[1]);
    println!("Searching the ID.8 sparse vector: {:?}", vectors[8]);
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::SparseFloat(vec![
                    vectors[1].clone(),
                    vectors[8].clone(),
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
