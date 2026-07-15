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
use std::collections::HashMap;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const LANGUAGE_FIELD: &str = "language";
    const SPARSE_FIELD: &str = "sparse";

    let client = client().await?;
    let collection = "RUST_V2_MULTI_ANALYZER".to_owned();
    let multi_analyzer_params = json!({
        "analyzers": {
            "english": {"type": "english"},
            "chinese": {"tokenizer": "jieba", "filter": ["lowercase", "removepunct"]},
            "japanese": {"tokenizer": {"type": "lindera", "dict_kind": "ipadic"}},
            "default": {"tokenizer": "icu", "filter": ["lowercase", "removepunct", "asciifolding"]}
        },
        "by_field": LANGUAGE_FIELD,
        "alias": {"cn": "chinese", "en": "english", "jap": "japanese"}
    });
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            FieldSchema::new()
                .name(TEXT_FIELD)
                .data_type(DataType::VarChar)
                .max_length(65_535)
                .enable_analyzer(true)
                .multi_analyzer_params(multi_analyzer_params),
        )
        .add_field(
            FieldSchema::new()
                .name(LANGUAGE_FIELD)
                .data_type(DataType::VarChar)
                .max_length(100),
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
                .input_fields([TEXT_FIELD])
                .output_fields([SPARSE_FIELD]),
        );

    drop_collection(&client, &collection).await;
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(&collection)
                .schema(schema)
                .build()?,
        )
        .await?;
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(&collection)
                .index_param(
                    IndexParam::new()
                        .field_name(SPARSE_FIELD)
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Bm25),
                )
                .sync(true)
                .timeout_ms(60_000)
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

    let groups = [
        ("en", vec![
            "Milvus is an open-source vector database", "AI applications help people better life", "Will the electric car replace gas-powered car?", "LangChain is a composable framework to build with LLMs. Milvus is integrated into LangChain.", "RAG is the process of optimizing the output of a large language model", "Newton is one of the greatest scientist of human history", "Metric type L2 is Euclidean distance", "Embeddings represent real-world objects, like words, images, or videos, in a form that computers can process.", "The moon is 384,400 km distance away from earth", "Milvus supports L2 distance and IP similarity for float vector.",
        ]),
        ("cn", vec!["人工智能正在改变技术领域", "机器学习模型需要大型数据集", "Milvus 是一个高性能、可扩展的向量数据库！"]),
        ("jap", vec!["Milvusの新機能をご確認くださいこのページでは", "非構造化データやマルチモーダルデータを構造化されたコレクションに整理することができます", "主な利点はデータアクセスパターンにある"]),
        ("default", vec!["토큰화 도구는 소프트웨어 국제화를 위한 핵심 도구를 제공하는", "Les applications qui suivent le temps à travers les régions", "Sin embargo, esto puede aumentar la complejidad de las consultas y de la gestión", "المثال، يوضح الرمز التالي كيفية إضافة عامل تصفية الحقل القياسي إلى بحث متجه"]),
    ];
    let rows: Vec<_> = groups
        .into_iter()
        .flat_map(|(language, texts)| {
            texts
                .into_iter()
                .map(move |text| json!({"language": language, "text": text}))
        })
        .collect();
    client
        .insert(
            InsertRequest::builder()
                .collection_name(&collection)
                .rows(rows)
                .build()?,
        )
        .await?;
    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(&collection)
                .output_fields(["count(*)"])
                .consistency_level(milvus::v2::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("count(*) = {}", query_count(count.results())?);

    for (text, analyzer) in [
        ("Milvus vector database", "english"),
        ("人工智能与机器学习", "chinese"),
        ("非構造化データ", "japanese"),
        ("Gestion des applications", "default"),
    ] {
        println!("============================== {analyzer} =================================");
        println!("Search by text: {text}");
        let result = client
            .search(
                SearchRequest::builder()
                    .collection_name(&collection)
                    .vector_field(SPARSE_FIELD)
                    .vectors(SearchVectors::EmbeddedText(vec![text.to_owned()]))
                    .metric_type(MetricType::Bm25)
                    .extra_params(HashMap::from([(
                        "analyzer_name".to_owned(),
                        analyzer.to_owned(),
                    )]))
                    .output_fields([TEXT_FIELD, LANGUAGE_FIELD])
                    .limit(5)
                    .consistency_level(milvus::v2::ConsistencyLevel::Bounded)
                    .build()?,
            )
            .await?;
        print_search_results(result.results())?;
    }

    drop_collection(&client, &collection).await;
    Ok(())
}
