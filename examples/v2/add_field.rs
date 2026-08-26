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
const NOTE: &str = "note";
const SPARSE: &str = "sparse";
const FUNCTION: &str = "bm25";
const DIMENSION: usize = 8;

async fn insert(
    client: &sdk::ClientV2,
    id: i64,
    text: Option<&str>,
    note: Option<&str>,
) -> Result<()> {
    let mut row = json!({"id":id,VECTOR:float_vector(DIMENSION)});
    if let Some(text) = text {
        row[TEXT] = json!(text);
    }
    if let Some(note) = note {
        row[NOTE] = json!(note);
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
    // Milvus 3.0 requires dataCoord.compaction.bumpSchemaVersion.enabled=true to add a
    // function field to a collection that already contains segments.
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
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(TEXT)
                .data_type(sdk::DataType::VarChar)
                .max_length(100)
                .enable_analyzer(true)
                .enable_match(true)
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
    insert(&client, 100, None, None).await?;
    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field(
                    sdk::FieldSchema::new()
                        .name(NOTE)
                        .data_type(sdk::DataType::VarChar)
                        .max_length(100)
                        .nullable(true),
                )
                .build()?,
        )
        .await?;
    describe(&client).await?;
    query(&client, 100).await?;
    insert(&client, 500, Some("this is a new row"), Some("new note")).await?;
    query(&client, 500).await?;

    // Drop the field added above. Existing data in that field is removed with the schema.
    client
        .drop_collection_field(
            DropCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field_name(NOTE)
                .build()?,
        )
        .await?;
    println!("Field '{NOTE}' dropped");
    describe(&client).await?;

    // Add a function-backed sparse field, its BM25 function, and its index in one request.
    // The collection's original text field is the BM25 input field.
    client
        .add_function_field(
            AddFunctionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field(
                    sdk::FieldSchema::new()
                        .name(SPARSE)
                        .data_type(sdk::DataType::SparseFloatVector),
                )
                .function(
                    sdk::Function::new()
                        .name(FUNCTION)
                        .function_type(sdk::FunctionType::Bm25)
                        .input_fields([TEXT])
                        .output_fields([SPARSE]),
                )
                .index(
                    sdk::IndexParam::new()
                        .field_name(SPARSE)
                        .index_name("sparse_idx")
                        .index_type(sdk::IndexType::SparseInvertedIndex)
                        .metric_type(sdk::MetricType::Bm25),
                )
                .build()?,
        )
        .await?;
    println!("Function-backed field '{SPARSE}' added");
    describe(&client).await?;

    // Drop the function and its output field together.
    client
        .drop_function_field(
            DropFunctionFieldRequest::builder()
                .collection_name(COLLECTION)
                .function_name(FUNCTION)
                .build()?,
        )
        .await?;
    println!("Function-backed field '{SPARSE}' dropped");
    describe(&client).await?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
