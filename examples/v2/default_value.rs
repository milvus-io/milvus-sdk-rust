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
    const COLLECTION: &str = "RUST_V2_DEFAULT_VALUE";
    const PARTITION_1: &str = "partition_1";
    const PARTITION_2: &str = "partition_2";
    const PRIMARY: &str = "pk";
    const VECTOR: &str = "vector";
    const NAME: &str = "name";
    const PRICE: &str = "price";
    const DIMENSION: usize = 4;

    let client = client().await?;
    let schema = CollectionSchema::new()
        .enable_dynamic_field(true)
        .add_field(
            FieldSchema::new()
                .name(PRIMARY)
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
                .default_value(DefaultValue::String("No Name".into())),
        )
        .add_field(
            FieldSchema::new()
                .name(PRICE)
                .data_type(DataType::Float)
                .default_value(DefaultValue::Float(0.123456)),
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
            let mut row = json!({PRIMARY: id, VECTOR: float_vector(DIMENSION)});
            if id % 2 == 0 {
                row[NAME] = json!(format!("row_{id}"));
                row[PRICE] = json!(id as f32 / 4.0);
            }
            row
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
                    FieldData::varchar(NAME, (0..10).map(|i| format!("column_{i}")).collect()),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    println!("\nQuery with filter: price < 0.5 in {PARTITION_1}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .partition_names([PARTITION_1])
                .filter("price < 0.5")
                .output_fields(["*"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;

    println!("\nSearch with filter: name != \"No Name\"");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![
                    float_vector(DIMENSION),
                    float_vector(DIMENSION),
                ]))
                .filter("name != \"No Name\"")
                .output_fields([NAME, PRICE])
                .limit(20)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
