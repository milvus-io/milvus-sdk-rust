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
use serde_json::{json, Value};
use utils::*;

const COLLECTION: &str = "RUST_V2_TEXT_MATCH";
const VECTOR: &str = "vector";
const TEXT: &str = "text";
const DIMENSION: usize = 128;

async fn query(client: &sdk::ClientV2, filter: &str) -> Result<()> {
    println!("================================================================");
    println!("Query with filter: {filter}");
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .output_fields(["id", TEXT])
                .limit(50)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_query_results(response.results())
}

async fn search(client: &sdk::ClientV2, filter: &str) -> Result<()> {
    println!("================================================================");
    println!("Search with filter: {filter}");
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(VECTOR)
                .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
                .filter(filter)
                .output_fields(["id", TEXT])
                .limit(50)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(response.results())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let analyzer: Value = json!({"tokenizer":"standard","filter":[{"type":"stop","stop_words":["is","and","to","of","for"]}]});
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
                .max_length(1024)
                .enable_analyzer(true)
                .analyzer_params(analyzer)
                .enable_match(true),
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
                        .index_type(sdk::IndexType::IvfFlat)
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
    let texts = ["Milvus is an open-source vector database","AI applications help people better life","Will the electric car replace gas-powered car?","LangChain is a composable framework to build with LLMs. Milvus is integrated into LangChain.","RAG is the process of optimizing the output of a large language model","Newton is one of the greatest scientist of human history","Metric type L2 is Euclidean distance","Embeddings represent real-world objects, like words, images, or videos, in a form that computers can process.","The moon is 384,400 km distance away from earth","Milvus supports L2 distance and IP similarity for float vector."];
    let rows: Vec<_> = texts
        .iter()
        .enumerate()
        .map(|(id, text)| json!({"id":id,TEXT:text,VECTOR:float_vector(DIMENSION)}))
        .collect();
    client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("count(*) = {}", query_count(count.results())?);
    flush(&client, COLLECTION).await?;
    for filter in [
        r#"TEXT_MATCH(text, "distance")"#,
        r#"TEXT_MATCH(text, "Milvus") or TEXT_MATCH(text, "distance")"#,
        r#"TEXT_MATCH(text, "Euclidean") and TEXT_MATCH(text, "distance")"#,
    ] {
        query(&client, filter).await?;
    }
    for filter in [
        r#"TEXT_MATCH(text, "distance")"#,
        r#"TEXT_MATCH(text, "Euclidean distance")"#,
        r#"TEXT_MATCH(text, "vector database")"#,
    ] {
        search(&client, filter).await?;
    }
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
