use milvus::client::{Client, ConsistencyLevel};
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::{CreateCollectionOptions, LoadOptions};
use milvus::query::{GetOptions, QueryOptions};
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const COLLECTION_NAME: &str = "rust_sdk_example_upsert_v1";
const ID_FIELD: &str = "pk";
const VECTOR_FIELD: &str = "vector";
const TEXT_FIELD: &str = "text_field";
const NULLABLE_FIELD: &str = "nullable_field";
const VECTOR_DIM: i64 = 4;

async fn cleanup(client: &Client) {
    if client.has_collection(COLLECTION_NAME).await.unwrap_or(false) {
        let _ = client.drop_collection(COLLECTION_NAME).await;
    }
}

fn create_vectors(row_count: usize) -> Vec<f32> {
    let mut vectors = Vec::with_capacity(row_count * VECTOR_DIM as usize);
    for i in 0..row_count {
        for j in 0..VECTOR_DIM as usize {
            vectors.push((i * VECTOR_DIM as usize + j) as f32 / 10.0);
        }
    }
    vectors
}

fn create_schema() -> Result<CollectionSchema, Error> {
    CollectionSchemaBuilder::new(COLLECTION_NAME, "upsert example")
        .add_field(FieldSchema::new_primary_int64(ID_FIELD, "", false))
        .add_field(FieldSchema::new_float_vector(VECTOR_FIELD, "", VECTOR_DIM))
        .add_field(FieldSchema::new_varchar(TEXT_FIELD, "", 100))
        .add_field(FieldSchema::new_int32(NULLABLE_FIELD, "").set_nullable(true))
        .build()
}

async fn create_collection(client: &Client, schema: &CollectionSchema) -> Result<(), Error> {
    cleanup(client).await;

    let indexes = IndexParams::new(
        "vector_idx".to_string(),
        IndexType::Flat,
        MetricType::COSINE,
        HashMap::new(),
    );

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                milvus::client::ConsistencyLevel::Bounded,
            )),
        )
        .await?;
    client
        .create_index(COLLECTION_NAME, VECTOR_FIELD, indexes)
        .await?;
    println!("Collection created");
    Ok(())
}

async fn insert_rows(client: &Client, schema: &CollectionSchema) -> Result<(), Error> {
    let ids: Vec<i64> = (0..100).collect();
    let vectors = create_vectors(100);
    let texts: Vec<String> = (0..100).map(|i| format!("text_{i}")).collect();
    let nullable: Vec<i32> = (0..100).collect();

    client
        .insert(
            COLLECTION_NAME,
            vec![
                FieldColumn::new(schema.get_field(ID_FIELD).unwrap(), ids),
                FieldColumn::new(schema.get_field(VECTOR_FIELD).unwrap(), vectors),
                FieldColumn::new(schema.get_field(TEXT_FIELD).unwrap(), texts),
                FieldColumn::new(schema.get_field(NULLABLE_FIELD).unwrap(), nullable),
            ],
            None,
        )
        .await?;
    client.flush(COLLECTION_NAME).await?;
    client
        .load_collection(COLLECTION_NAME, Some(LoadOptions::default()))
        .await?;
    println!("Inserted initial rows");
    Ok(())
}

async fn show_row(client: &Client, id: i64) -> Result<(), Error> {
    let result = client
        .get(
            COLLECTION_NAME,
            milvus::query::IdType::Int64(vec![id]),
            Some(GetOptions::new().output_fields(vec![
                ID_FIELD.to_string(),
                VECTOR_FIELD.to_string(),
                TEXT_FIELD.to_string(),
                NULLABLE_FIELD.to_string(),
            ]).consistency_level(ConsistencyLevel::Strong.into())),
        )
        .await?;
    println!("Row {id}:");
    for field in result {
        println!("  {} => {:?}", field.name, field.value);
    }
    Ok(())
}

async fn full_upsert(client: &Client, schema: &CollectionSchema, id: i64) -> Result<(), Error> {
    println!("------------------------------ full upsert ------------------------------");
    println!("Before full upsert:");
    show_row(client, id).await?;

    let upsert_vector = vec![1.0_f32, 1.0, 1.0, 1.0];
    let upsert_text = vec!["this field has been updated".to_string()];
    let upsert_nullable = vec![0_i32];

    client
        .upsert(
            COLLECTION_NAME,
            vec![
                FieldColumn::new(schema.get_field(ID_FIELD).unwrap(), vec![id]),
                FieldColumn::new(schema.get_field(VECTOR_FIELD).unwrap(), upsert_vector),
                FieldColumn::new(schema.get_field(TEXT_FIELD).unwrap(), upsert_text),
                FieldColumn::new(schema.get_field(NULLABLE_FIELD).unwrap(), upsert_nullable),
            ],
            None::<milvus::mutate::UpsertOptions>,
        )
        .await?;
    println!("After full upsert:");
    show_row(client, id).await?;
    Ok(())
}

async fn partial_upsert(
    client: &Client,
    schema: &CollectionSchema,
    ids: Vec<i64>,
) -> Result<(), Error> {
    println!("------------------------------ partial upsert ------------------------------");
    println!("Before partial upsert:");
    for id in &ids {
        show_row(client, *id).await?;
    }

    let new_texts: Vec<String> = ids
        .iter()
        .map(|_| "this row has been partially updated".to_string())
        .collect();

    client
        .upsert(
            COLLECTION_NAME,
            vec![
                FieldColumn::new(schema.get_field(ID_FIELD).unwrap(), ids.clone()),
                FieldColumn::new(schema.get_field(TEXT_FIELD).unwrap(), new_texts),
            ],
            Some(milvus::mutate::UpsertOptions::new().partial_update(true)),
        )
        .await?;

    println!("After partial upsert:");
    for id in &ids {
        show_row(client, *id).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new(URL).await?;
    let schema = create_schema()?;
    create_collection(&client, &schema).await?;
    insert_rows(&client, &schema).await?;

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
    println!("{persisted} rows persisted");

    full_upsert(&client, &schema, 2).await?;
    partial_upsert(&client, &schema, vec![3, 4]).await?;

    client.drop_collection(COLLECTION_NAME).await?;
    Ok(())
}
