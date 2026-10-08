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

const COLLECTION: &str = "RUST_V2_TEXT_FIELD";
const TEXT: &str = "text";
const TEXT_VECTOR: &str = "text_vector";

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let analyzer = json!({
        "tokenizer": "standard",
        "filter": [{"type": "stop", "stop_words": ["is", "and", "to", "of", "for"]}]
    });
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(TEXT)
                .data_type(sdk::DataType::Text)
                .enable_analyzer(true)
                .analyzer_params(analyzer)
                .enable_match(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(TEXT_VECTOR)
                .data_type(sdk::DataType::SparseFloatVector),
        )
        .add_function(
            Function::new()
                .name("function_bm25")
                .function_type(FunctionType::Bm25)
                .input_fields([TEXT])
                .output_fields([TEXT_VECTOR]),
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
                        .field_name(TEXT_VECTOR)
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Bm25),
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

    // Column-based insert: Text schema fields accept VarChar column data.
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .columns(vec![sdk::FieldData::varchar(
                    TEXT,
                    vec!["column-based text row".into(), "another column text".into()],
                )])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    // Row-based insert with JSON values.
    let texts = [
        "Milvus is an open-source vector database",
        "AI applications help people better life",
        "RAG is the process of optimizing the output of a large language model",
        "The moon is 384,400 km distance away from earth",
    ];
    let rows: Vec<_> = texts.iter().map(|text| json!({TEXT: text})).collect();
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by row-based.", insert.insert_count());

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

    // Full-text search over the Text field through the BM25 function vector.
    for query_text in ["Milvus vector database", "moon and earth distance"] {
        println!("================================================================");
        println!("Search by text: {query_text}");
        let response = client
            .search(
                SearchRequest::builder()
                    .collection_name(COLLECTION)
                    .vector_field(TEXT_VECTOR)
                    .vectors(SearchVectors::EmbeddedText(vec![query_text.into()]))
                    .output_fields(["id", TEXT])
                    .limit(5)
                    .consistency_level(sdk::ConsistencyLevel::Bounded)
                    .build()?,
            )
            .await?;
        print_search_results(response.results())?;
    }

    // The Text field decodes back to plain strings in query results.
    // TEXT_MATCH runs against the sealed-segment text index, so flush first.
    flush(&client, COLLECTION).await?;
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(r#"TEXT_MATCH(text, "Milvus")"#)
                .output_fields(["id", TEXT])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    print_query_results(response.results())?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
