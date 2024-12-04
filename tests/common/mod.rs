use milvus::client::*;
use milvus::error::Result;
use milvus::options::CreateCollectionOptions;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use rand::Rng;

pub const DEFAULT_DIM: i64 = 128;
pub const DEFAULT_VEC_FIELD: &str = "float32_vec_field";
pub const FP16_VEC_FIELD: &str = "float16_vec_field";
pub const BF16_VEC_FIELD: &str = "bfloat16_vec_field";
pub const DEFAULT_INDEX_NAME: &str = "float32_vec_field_index";
pub const FP16_VEC_INDEX_NAME: &str = "float16_vec_field_index";
pub const BF16_VEC_INDEX_NAME: &str = "bfloat16_vec_field_index";

pub const URL: &str = "http://localhost:19530";

pub async fn create_test_collection(autoid: bool) -> Result<(Client, CollectionSchema)> {
    let collection_name = gen_random_name();
    let collection_name = format!("{}_{}", "test_collection", collection_name);
    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", autoid))
        .add_field(FieldSchema::new_float_vector(
            DEFAULT_VEC_FIELD,
            "",
            DEFAULT_DIM,
        ))
        .build()?;
    create_test_collection_with_schema(schema).await
}

pub async fn create_test_fp16_bf16_collection(autoid: bool) -> Result<(Client, CollectionSchema)> {
    let collection_name = gen_random_name();
    let collection_name = format!("{}_{}", "test_collection", collection_name);
    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", autoid))
        .add_field(FieldSchema::new_float16_vector(
            FP16_VEC_FIELD,
            "fp16 vector field",
            DEFAULT_DIM,
        ))
        .add_field(FieldSchema::new_bfloat16_vector(
            BF16_VEC_FIELD,
            "bf16 vector field",
            DEFAULT_DIM,
        ))
        .build()?;
    create_test_collection_with_schema(schema).await
}

async fn create_test_collection_with_schema(
    schema: CollectionSchema,
) -> Result<(Client, CollectionSchema)> {
    let client = Client::new(URL).await?;
    if client.has_collection(schema.name()).await? {
        client.drop_collection(schema.name()).await?;
    }
    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Eventually,
            )),
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
