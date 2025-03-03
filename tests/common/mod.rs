use milvus::client::*;
use milvus::error::Result;
use milvus::options::CreateCollectionOptions;
use milvus::proto::schema::DataType;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchemaBuilder};
use rand::Rng;

pub const DEFAULT_DIM: i64 = 128;
pub const DEFAULT_VEC_FIELD: &str = "feature";
pub const DEFAULT_INDEX_NAME: &str = "feature_index";
pub const URL: &str = "http://127.0.0.1:19530";

pub type CollectionSchemaFn = fn(&str) -> Result<CollectionSchema>;

pub async fn create_test_collection(
    autoid: bool,
    schema: Option<CollectionSchemaFn>,
) -> Result<(Client, CollectionSchema)> {
    let collection_name = format!("{}_{}", "test_collection", gen_random_name());
    let client = Client::new(URL).await?;
    let schema = match schema {
        Some(schema_fn) => schema_fn(&collection_name)?,
        None => CollectionSchemaBuilder::new(&collection_name, "")
            .add_field(
                FieldSchemaBuilder::new()
                    .with_name("id")
                    .with_dtype(DataType::Int64)
                    .with_primary(true)
                    .with_auto_id(autoid)
                    .build(),
            )
            .add_field(
                FieldSchemaBuilder::new()
                    .with_name(DEFAULT_VEC_FIELD)
                    .with_dtype(DataType::FloatVector)
                    .with_dim(DEFAULT_DIM)
                    .build(),
            )
            .build()?,
    };
    if client.has_collection(&collection_name).await? {
        client.drop_collection(&collection_name).await?;
    }
    client
        .create_collection(
            schema.clone(),
            Some(
                CreateCollectionOptions::with_consistency_level(ConsistencyLevel::Eventually)
                    .add_property("collection.insertRate.max.mb", "2000000"),
            ),
        )
        .await?;
    Ok((client, schema))
}

pub fn gen_random_name() -> String {
    format!(
        "r{}",
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect::<String>(),
    )
}

pub fn gen_random_int64_vector(n: i64) -> Vec<i64> {
    let mut data: Vec<i64> = Vec::with_capacity(n as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        data.push(rng.gen());
    }
    data
}

pub fn gen_random_f32_vector(n: i64) -> Vec<f32> {
    let mut data = Vec::<f32>::with_capacity(n as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        data.push(rng.gen());
    }
    data
}
