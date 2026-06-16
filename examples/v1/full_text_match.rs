use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::{CreateCollectionOptions, LoadOptions};
use milvus::query::{QueryOptions, SearchOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const COLLECTION_NAME: &str = "rust_sdk_example_text_match_v1";
const ID_FIELD: &str = "id";
const TEXT_FIELD: &str = "text";
const VECTOR_FIELD: &str = "vector";

async fn cleanup(client: &Client) {
    if client.has_collection(COLLECTION_NAME).await.unwrap_or(false) {
        let _ = client.drop_collection(COLLECTION_NAME).await;
    }
}

async fn query_by_text(client: &Client, text: &str) -> Result<(), Error> {
    let results = client
        .search(
            COLLECTION_NAME,
            vec![text.into()],
            Some(
                SearchOptions::new()
                    .anns_field(vec![VECTOR_FIELD.to_string()])
                    .limit(3)
                    .output_fields(vec![TEXT_FIELD.to_string()]),
            ),
        )
        .await?;

    println!("\nSearch by text: {text}");
    for result in &results {
        let text_field = result
            .field
            .iter()
            .find(|field| field.name == TEXT_FIELD)
            .unwrap();
        println!(
            "score: {:?}, {} => {:?}",
            result.score, text_field.name, text_field.value
        );
    }
    println!("=============================================================");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new(URL).await?;
    cleanup(&client).await;

    let bm25_function = milvus::proto::schema::FunctionSchema {
        name: "function_bm25".to_string(),
        r#type: milvus::proto::schema::FunctionType::Bm25 as i32,
        input_field_names: vec![TEXT_FIELD.to_string()],
        output_field_names: vec![VECTOR_FIELD.to_string()],
        ..Default::default()
    };

    let schema = CollectionSchemaBuilder::new(COLLECTION_NAME, "full text match example")
        .add_field(FieldSchema::new_primary_int64(ID_FIELD, "", false))
        .add_field(
            FieldSchema::new_varchar(TEXT_FIELD, "document text", 65535)
                .add_type_param("enable_analyzer", "true")
                .add_type_param("enable_match", "true")
                .add_type_param("analyzer_params", "{\"type\": \"standard\"}"),
        )
        .add_field(FieldSchema::new_sparse_float_vector(VECTOR_FIELD, "BM25 output"))
        .add_function(bm25_function)
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                milvus::client::ConsistencyLevel::Strong,
            )),
        )
        .await?;
    println!("Collection created");

    let ids: Vec<i64> = (0..10).collect();
    let texts: Vec<String> = vec![
        "Milvus is an open-source vector database".into(),
        "AI applications help people better life".into(),
        "Will the electric car replace gas-powered car?".into(),
        "LangChain is a composable framework to build with LLMs. Milvus is integrated into LangChain.".into(),
        "RAG is the process of optimizing the output of a large language model".into(),
        "Newton is one of the greatest scientist of human history".into(),
        "Metric type L2 is Euclidean distance".into(),
        "Embeddings represent real-world objects, like words, images, or videos, in a form that computers can process.".into(),
        "The moon is 384,400 km distance away from earth".into(),
        "Milvus supports L2 distance and IP similarity for float vector.".into(),
    ];

    client
        .insert(
            COLLECTION_NAME,
            vec![
                FieldColumn::new(schema.get_field(ID_FIELD).unwrap(), ids),
                FieldColumn::new(schema.get_field(TEXT_FIELD).unwrap(), texts),
            ],
            None,
        )
        .await?;
    client.flush(COLLECTION_NAME).await?;

    let sparse_idx = IndexParams::new(
        "sparse_idx".to_owned(),
        IndexType::SparseInvertedIndex,
        MetricType::BM25,
        HashMap::new(),
    );
    client
        .create_index(COLLECTION_NAME, VECTOR_FIELD, sparse_idx)
        .await?;
    client
        .load_collection(COLLECTION_NAME, Some(LoadOptions::default()))
        .await?;

    let count_result = client
        .query(
            COLLECTION_NAME,
            "",
            &QueryOptions::default()
                .output_fields(vec!["count(*)".to_string()])
                .consistency_level(milvus::client::ConsistencyLevel::Strong.into()),
        )
        .await?;
    let persisted = count_result[0]
        .value
        .clone()
        .try_into()
        .map(|v: Vec<i64>| v[0])
        .unwrap_or(0);
    println!("{persisted} rows in collection");

    query_by_text(&client, "moon and earth distance").await?;
    query_by_text(&client, "Milvus vector database").await?;

    client.drop_collection(COLLECTION_NAME).await?;
    Ok(())
}
