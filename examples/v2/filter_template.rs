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
    const COLLECTION: &str = "RUST_V2_FILTER_TEMPLATE";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "vector";
    const TEXT: &str = "text";
    const DIMENSION: usize = 4;
    const ROW_COUNT: usize = 10_000;

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
                .name(VECTOR)
                .data_type(DataType::FloatVector)
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
                        .index_type(IndexType::Flat)
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

    let rows: Vec<_> = (0..ROW_COUNT)
        .map(|i| json!({TEXT: format!("text_{i}"), VECTOR: float_vector(DIMENSION)}))
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
    let ids = match insert.ids() {
        Ids::Int64(ids) => ids,
        _ => return Err(Error::Unexpected("expected integer auto IDs".into())),
    };

    let filter = "pk in {my_ids}";
    println!("Query with filter expression: {filter}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .filter_templates(HashMap::from([(
                    "my_ids".into(),
                    json!(ids[500..600].to_vec()),
                )]))
                .output_fields([TEXT])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("Query with filter template:");
    print_query_results(query.results())?;

    let texts = (300..500).map(|i| format!("text_{i}")).collect::<Vec<_>>();
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![
                    float_vector(DIMENSION),
                    float_vector(DIMENSION),
                ]))
                .filter("text in {my_texts}")
                .filter_templates(HashMap::from([("my_texts".into(), json!(texts))]))
                .output_fields([TEXT])
                .limit(200)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    println!("Search with filter template:");
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
