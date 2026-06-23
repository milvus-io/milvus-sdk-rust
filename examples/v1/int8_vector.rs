use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::{CreateCollectionOptions, LoadOptions};
use milvus::query::{GetOptions, QueryOptions, SearchOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use rand::Rng;
use std::borrow::Cow;
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const COLLECTION_NAME: &str = "rust_sdk_example_int8_vector_v1";
const ID_FIELD: &str = "id";
const VECTOR_FIELD: &str = "vector";
const VECTOR_DIM: usize = 128;
const ROW_COUNT: usize = 200;

fn generate_int8_vectors(count: usize) -> Vec<Vec<u8>> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            (0..VECTOR_DIM)
                .map(|_| (rng.gen_range(-128_i16..=127_i16) as i8) as u8)
                .collect()
        })
        .collect()
}

async fn cleanup(client: &Client) {
    if client.has_collection(COLLECTION_NAME).await.unwrap_or(false) {
        let _ = client.drop_collection(COLLECTION_NAME).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new(URL).await?;
    cleanup(&client).await;

    let schema = CollectionSchemaBuilder::new(COLLECTION_NAME, "int8 vector example")
        .add_field(FieldSchema::new_primary_int64(ID_FIELD, "", false))
        .add_field(FieldSchema::new_int8_vector(
            VECTOR_FIELD,
            "int8 embedding",
            VECTOR_DIM as i64,
        ))
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

    let ids: Vec<i64> = (0..ROW_COUNT as i64).collect();
    let vectors = generate_int8_vectors(ROW_COUNT);
    let flat_vectors: Vec<u8> = vectors.iter().flat_map(|v| v.iter().copied()).collect();

    client
        .insert(
            COLLECTION_NAME,
            vec![
                FieldColumn::new(schema.get_field(ID_FIELD).unwrap(), ids),
                FieldColumn::new(schema.get_field(VECTOR_FIELD).unwrap(), flat_vectors),
            ],
            None,
        )
        .await?;
    client.flush(COLLECTION_NAME).await?;

    let index_params = IndexParams::new(
        "int8_hnsw".to_string(),
        IndexType::HNSW,
        MetricType::L2,
        HashMap::from([
            ("M".to_string(), "64".to_string()),
            ("efConstruction".to_string(), "200".to_string()),
        ]),
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
            &QueryOptions::default().output_fields(vec!["count(*)".to_string()]),
        )
        .await?;
    let persisted = count_result[0]
        .value
        .clone()
        .try_into()
        .map(|v: Vec<i64>| v[0])
        .unwrap_or(0);
    println!("{persisted} rows persisted");

    let target_id = rand::thread_rng().gen_range(0..ROW_COUNT);
    let target_vector = vectors[target_id].clone();
    let search_resp = client
        .search(
            COLLECTION_NAME,
            vec![Value::Int8Vector(Cow::Owned(target_vector.clone()))],
            Some(
                SearchOptions::with_limit(3)
                    .output_fields(vec![VECTOR_FIELD.to_string()])
                    .add_param("ef", "200"),
            ),
        )
        .await?;

    let result = &search_resp[0];
    println!("\nSearch results for row {target_id}:");
    for hit_idx in 0..result.size as usize {
        let hit_id = match &result.id[hit_idx] {
            Value::Long(id) => *id,
            other => {
                return Err(Error::Unexpected(format!(
                    "unexpected id type in search result: {:?}",
                    other
                )))
            }
        };
        println!("id: {} score: {}", hit_id, result.score[hit_idx]);
    }

    let top1_id = match &result.id[0] {
        Value::Long(id) => *id,
        other => {
            return Err(Error::Unexpected(format!(
                "unexpected id type in top1 search result: {:?}",
                other
            )))
        }
    };
    if top1_id != target_id as i64 {
        return Err(Error::Unexpected(format!(
            "The top1 ID {} is not equal to target vector's ID {}",
            top1_id, target_id
        )));
    }
    println!("Search result is correct");

    let query_resp = client
        .get(
            COLLECTION_NAME,
            milvus::query::IdType::Int64(vec![target_id as i64]),
            Some(GetOptions::new().output_fields(vec![VECTOR_FIELD.to_string()])),
        )
        .await?;
    let vector_field = query_resp
        .iter()
        .find(|field| field.name == VECTOR_FIELD)
        .ok_or_else(|| Error::Unexpected("vector field not found in get response".to_string()))?;
    let stored_vectors: Vec<u8> = vector_field.value.clone().try_into().unwrap();
    if stored_vectors != target_vector {
        return Err(Error::Unexpected(
            "The query result is incorrect".to_string(),
        ));
    }
    println!("Query result is correct");

    client.drop_collection(COLLECTION_NAME).await?;
    Ok(())
}
