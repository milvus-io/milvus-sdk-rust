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

use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use serde_json::json;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const TITLE_FIELD: &str = "title";
    const SPARSE_FIELD: &str = "sparse";

    let client = client().await?;
    let collection = "RUST_V2_HIGHLIGHTER".to_owned();
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(TITLE_FIELD)
                .data_type(DataType::VarChar)
                .max_length(512)
                .enable_analyzer(true)
                .enable_match(true),
        )
        .add_field(
            FieldSchema::new()
                .name(TEXT_FIELD)
                .data_type(DataType::VarChar)
                .max_length(65_535)
                .enable_analyzer(true)
                .enable_match(true),
        )
        .add_field(
            FieldSchema::new()
                .name(SPARSE_FIELD)
                .data_type(DataType::SparseFloatVector),
        )
        .add_function(
            Function::new()
                .name("bm25")
                .function_type(FunctionType::Bm25)
                .input_fields([TITLE_FIELD])
                .output_fields([SPARSE_FIELD]),
        );

    drop_collection(&client, &collection).await;
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection)
                .schema(schema)
                .index_param(
                    IndexParam::new()
                        .field_name(SPARSE_FIELD)
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Bm25),
                )
                .consistency_level(milvus::v2::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(&collection)
                .sync(true)
                .timeout_ms(60_000)
                .build()?,
        )
        .await?;

    let rows: Vec<_> = vec![
        json!({"id": 0, "title": "Milvus for scale", "text": "Milvus is an open-source vector database built for scale. This paragraph is intentionally long so the keyword search appears much later in the same text fragment. Search is a core capability for information retrieval systems."}),
        json!({"id": 1, "title": "Full text search", "text": "Milvus supports full text search with analyzers and BM25. This sentence adds enough spacing and extra wording to separate the two highlighted terms into different regions for the lexical highlighter example."}),
        json!({"id": 2, "title": "RAG systems", "text": "Vector databases help retrieval augmented generation systems."}),
        json!({"id": 3, "title": "Milvus users", "text": "This example demonstrates highlighted snippets for modern applications. The word search is placed here with a lot of filler text before Milvus appears again near the end of the document to encourage multiple fragments in highlighter output for Milvus users."}),
    ];
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(&collection)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("Inserted {} rows", insert.insert_count());
    flush(&client, &collection).await?;

    let query_texts = ["milvus users", "text search"];
    let highlighter = LexicalHighlighter::new()
        .highlight_queries(
            query_texts
                .map(|text| {
                    HighlightQuery::new()
                        .query_type("TextMatch")
                        .field(TEXT_FIELD)
                        .text(text)
                })
                .to_vec(),
        )
        .pre_tags(["<em>"])
        .post_tags(["</em>"])
        .fragment_size(40)
        .num_of_fragments(10);
    let result = client
        .search(
            SearchRequest::builder()
                .collection_name(&collection)
                .vector_field(SPARSE_FIELD)
                .vectors(SearchVectors::EmbeddedText(
                    query_texts.iter().map(|text| (*text).to_owned()).collect(),
                ))
                .filter(format!(
                    "TEXT_MATCH({TEXT_FIELD}, '{}')",
                    query_texts.join(" ")
                ))
                .metric_type(MetricType::Bm25)
                .highlighter(highlighter)
                .output_fields([TITLE_FIELD, TEXT_FIELD])
                .limit(3)
                .build()?,
        )
        .await?;
    println!("\nSearch with lexical highlighter: {:?}", query_texts);
    for result in result.results() {
        for (row, highlights) in result.rows()?.zip(result.get_highlight_results()) {
            println!(
                "\n-----------------------------------------------------------------------------"
            );
            println!("{:?}", row.to_entity_row()?);
            for field in [TEXT_FIELD, TITLE_FIELD] {
                if let Some(highlight) = highlights.get(field) {
                    println!("  highlighted field: {}", highlight.get_field_name());
                    println!("  fragments: {:?}", highlight.get_fragments());
                    println!("  scores: {:?}", highlight.get_scores());
                }
            }
            println!(
                "  title: {}",
                row.get_str(TITLE_FIELD).unwrap_or("<missing>")
            );
            println!("  text: {}", row.get_str(TEXT_FIELD).unwrap_or("<missing>"));
        }
    }

    drop_collection(&client, &collection).await;
    Ok(())
}
