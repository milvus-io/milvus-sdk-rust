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

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_NULLABLE_FIELD";
    const PARTITION_1: &str = "partition_1";
    const PARTITION_2: &str = "partition_2";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "vector";
    const NAME: &str = "name";
    const AGE: &str = "age";
    const ARRAY: &str = "array";
    const DIMENSION: usize = 4;

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
                .name(VECTOR)
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name(NAME)
                .data_type(DataType::VarChar)
                .max_length(1024)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(AGE)
                .data_type(DataType::Int8)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(ARRAY)
                .data_type(DataType::Array)
                .element_type(DataType::Float)
                .max_capacity(10)
                .nullable(true),
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
    for partition in [PARTITION_1, PARTITION_2] {
        client
            .create_partition(
                sdk::request::partition::CreatePartitionRequest::builder()
                    .collection_name(COLLECTION)
                    .partition_name(partition)
                    .build()?,
            )
            .await?;
    }
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

    let rows: Vec<_> = (0..10)
        .map(|id| {
            if id % 2 == 0 {
                json!({PRIMARY: id, VECTOR: float_vector(DIMENSION), NAME: format!("row_{id}"), AGE: id, ARRAY: [id as f32 + 0.1, id as f32 + 0.2, id as f32 + 0.3]})
            } else {
                json!({PRIMARY: id, VECTOR: float_vector(DIMENSION), NAME: null})
            }
        })
        .collect();
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION_1)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by row-based.", insert.insert_count());

    let names = FieldData::varchar(NAME, (0..5).map(|i| format!("column_{}", i * 2)).collect())
        .with_validity((0..10).map(|i| i % 2 == 0).collect())?;
    let ages = FieldData::int8(AGE, (0..5).map(|i| (i * 2) as i8).collect())
        .with_validity((0..10).map(|i| i % 2 == 0).collect())?;
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION_2)
                .columns(vec![
                    FieldData::int64(PRIMARY, (10..20).collect()),
                    FieldData::float_vector(
                        VECTOR,
                        (0..10).map(|_| float_vector(DIMENSION)).collect(),
                    ),
                    names,
                    ages,
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    println!("\nQuery with filter: name is null in {PARTITION_1}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .partition_names([PARTITION_1])
                .filter("name is null")
                .output_fields(["*"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!("\nSearch with filter: age is not null in {PARTITION_2}");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .partition_names([PARTITION_2])
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![
                    float_vector(DIMENSION),
                    float_vector(DIMENSION),
                ]))
                .filter("age is not null")
                .output_fields([NAME, AGE, ARRAY])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
