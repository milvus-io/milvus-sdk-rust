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

const COLLECTION: &str = "RUST_V2_ADD_FIELD";
const VECTOR: &str = "vector";
const TEXT: &str = "text";
const DIMENSION: usize = 8;

async fn insert(client: &sdk::ClientV2, id: i64, text: Option<&str>) -> Result<()> {
    let mut row = json!({"id":id,VECTOR:float_vector(DIMENSION)});
    if let Some(text) = text {
        row[TEXT] = json!(text);
    }
    client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .row(row)
                .build()?,
        )
        .await?;
    Ok(())
}

async fn query(client: &sdk::ClientV2, id: i64) -> Result<()> {
    println!("\nQuery with id: {id}");
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(format!("id == {id}"))
                .output_fields(["*"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(response.results())?;
    println!("=============================================================");
    Ok(())
}

async fn describe(client: &sdk::ClientV2) -> Result<()> {
    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("\nCollection fields:");
    for field in response.description().get_schema().get_fields() {
        println!("  {}", field.get_name());
    }
    for function in response.description().get_schema().get_functions() {
        println!("  function: {}", function.get_name());
    }
    println!("=============================================================");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(VECTOR)
                .data_type(sdk::DataType::FloatVector)
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
                    sdk::IndexParam::new()
                        .field_name(VECTOR)
                        .index_type(sdk::IndexType::Flat)
                        .metric_type(sdk::MetricType::Cosine),
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
    println!("Collection created");
    insert(&client, 100, None).await?;
    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field(
                    sdk::FieldSchema::new()
                        .name(TEXT)
                        .data_type(sdk::DataType::VarChar)
                        .max_length(100)
                        .nullable(true),
                )
                .build()?,
        )
        .await?;
    describe(&client).await?;
    query(&client, 100).await?;
    insert(&client, 500, Some("this is a new row")).await?;
    query(&client, 500).await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
