use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::{CreateCollectionOptions, LoadOptions};
use milvus::query::{QueryOptions, SearchOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use rand::Rng;
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const COLLECTION_NAME: &str = "rust_sdk_example_timestampz_v1";
const ID_FIELD: &str = "id";
const VECTOR_FIELD: &str = "vector";
const TIMESTAMPTZ_FIELD: &str = "tsz";
const DIM: i64 = 128;

async fn cleanup(client: &Client) {
    if client.has_collection(COLLECTION_NAME).await.unwrap_or(false) {
        let _ = client.drop_collection(COLLECTION_NAME).await;
    }
}

fn generate_float_vector(dim: i64) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new(URL).await?;
    cleanup(&client).await;

    let schema = CollectionSchemaBuilder::new(COLLECTION_NAME, "timestamptz example")
        .add_field(FieldSchema::new_primary_int64(ID_FIELD, "", false))
        .add_field(FieldSchema::new_float_vector(VECTOR_FIELD, "", DIM))
        .add_field(FieldSchema::new_timestamptz(TIMESTAMPTZ_FIELD, "timestamp with timezone"))
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
    let mut vectors = Vec::new();
    for _ in 0..10 {
        vectors.extend(generate_float_vector(DIM));
    }
    let timestamps: Vec<String> = (0..10)
        .map(|i| format!("2025-01-{:02}T00:00:00+08:00", i + 1))
        .collect();

    client
        .insert(
            COLLECTION_NAME,
            vec![
                FieldColumn::new(schema.get_field(ID_FIELD).unwrap(), ids),
                FieldColumn::new(schema.get_field(VECTOR_FIELD).unwrap(), vectors),
                FieldColumn::new(schema.get_field(TIMESTAMPTZ_FIELD).unwrap(), timestamps),
            ],
            None,
        )
        .await?;
    client.flush(COLLECTION_NAME).await?;

    let index_params = IndexParams::new(
        "vector_idx".to_owned(),
        IndexType::AutoIndex,
        MetricType::COSINE,
        HashMap::new(),
    );
    client
        .create_index(COLLECTION_NAME, VECTOR_FIELD, index_params)
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
    println!("\n{persisted} rows persisted");

    let query_ret = client
        .query(
            COLLECTION_NAME,
            "id <= 3",
            &QueryOptions::new().output_fields(vec![TIMESTAMPTZ_FIELD.to_string()]),
        )
        .await?;
    println!("\nQuery results:");
    for field in &query_ret {
        println!("{} => {:?}", field.name, field.value);
    }

    let search_ret = client
        .search(
            COLLECTION_NAME,
            vec![Value::from(generate_float_vector(DIM))],
            Some(
                SearchOptions::with_limit(10)
                    .filter("id <= 3".to_string())
                    .output_fields(vec![TIMESTAMPTZ_FIELD.to_string()]),
            ),
        )
        .await?;
    println!("\nSearch results:");
    for result in &search_ret {
        for i in 0..result.size as usize {
            let hit_id = match &result.id[i] {
                Value::Long(id) => *id,
                other => {
                    return Err(Error::Unexpected(format!(
                        "unexpected id type in search result: {:?}",
                        other
                    )))
                }
            };
            println!("ID: {} Score: {}", hit_id, result.score[i]);
        }
        for field in &result.field {
            println!("{:?}", field);
        }
    }

    client.drop_collection(COLLECTION_NAME).await?;
    Ok(())
}
