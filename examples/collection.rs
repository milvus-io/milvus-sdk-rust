use milvus::index::{IndexParams, IndexType};
use milvus::options::LoadOptions;
use milvus::query::QueryOptions;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder};
use milvus::{
    client::Client, collection::Collection, data::FieldColumn, error::Error, schema::FieldSchema,
};
use std::collections::HashMap;

use rand::prelude::*;

const FP32_VEC_FIELD: &str = "float32_vector_field";

const DIM: i64 = 256;

#[tokio::main]
async fn main() -> Result<(), Error> {
    const URL: &str = "http://localhost:19530";

    let client = Client::new(URL).await?;

    let schema =
        CollectionSchemaBuilder::new("hello_milvus", "a guide example for milvus rust SDK")
            .add_field(FieldSchema::new_primary_int64(
                "id",
                "primary key field",
                true,
            ))
            .add_field(FieldSchema::new_float_vector(
                FP32_VEC_FIELD,
                "fp32 feature field",
                DIM,
            ))
            .build()?;
    client.create_collection(schema.clone(), None).await?;

    if let Err(err) = hello_milvus(&client, &schema).await {
        println!("failed to run hello milvus: {:?}", err);
    }
    client.drop_collection(schema.name()).await?;

    Ok(())
}

async fn hello_milvus(client: &Client, collection: &CollectionSchema) -> Result<(), Error> {
    let mut embed_data = Vec::<f32>::new();
    for _ in 1..=DIM * 1000 {
        let mut rng = rand::thread_rng();
        let embed = rng.gen();
        embed_data.push(embed);
    }
    let embed_column = FieldColumn::new(collection.get_field(FP32_VEC_FIELD).unwrap(), embed_data)?;

    client
        .insert(collection.name(), vec![embed_column], None)
        .await?;
    client.flush(collection.name()).await?;
    let index_params = IndexParams::new(
        "feature_index".to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    client
        .create_index(collection.name(), FP32_VEC_FIELD, index_params)
        .await?;
    client
        .load_collection(collection.name(), Some(LoadOptions::default()))
        .await?;

    let options = QueryOptions::default();
    let result = client.query(collection.name(), "id > 0", &options).await?;

    println!(
        "result num: {}",
        result.first().map(|c| c.len()).unwrap_or(0),
    );

    Ok(())
}
