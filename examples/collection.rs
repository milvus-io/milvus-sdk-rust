use milvus::index::{IndexParams, IndexType};
use milvus::schema::CollectionSchemaBuilder;
use milvus::{
    client::Client, collection::Collection, data::FieldColumn, error::Error, schema::FieldSchema,
};
use std::collections::HashMap;

use rand::prelude::*;

const DEFAULT_VEC_FIELD: &str = "embed";

#[tokio::main]
async fn main() -> Result<(), Error> {
    const URL: &str = "http://localhost:19530";

    let client = Client::new(URL).await?;

    let schema =
        CollectionSchemaBuilder::new("hello_milvus", "a guide example for milvus rust SDK")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", 256))
            .build()?;
    let collection = client.create_collection(schema.clone(), None).await?;

    if let Err(err) = hello_milvus(&collection).await {
        println!("failed to run hello milvus: {:?}", err);
    }
    collection.drop().await?;

    Ok(())
}

async fn hello_milvus(collection: &Collection) -> Result<(), Error> {
    let mut embed_data = Vec::<f32>::new();
    for _ in 1..=256 * 1000 {
        let mut rng = rand::thread_rng();
        let embed = rng.gen();
        embed_data.push(embed);
    }
    let embed_column = FieldColumn::new(
        collection.schema().get_field(DEFAULT_VEC_FIELD).unwrap(),
        embed_data,
    );

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;
    let index_params = IndexParams::new(
        "feature_index".to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    collection
        .create_index(DEFAULT_VEC_FIELD, index_params)
        .await?;
    collection.load(1).await?;

    let result = collection.query::<_, [&str; 0]>("id > 0", []).await?;

    println!(
        "result num: {}",
        result.first().map(|c| c.len()).unwrap_or(0),
    );

    Ok(())
}
