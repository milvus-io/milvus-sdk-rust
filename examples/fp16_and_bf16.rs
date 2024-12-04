use milvus::index::{IndexParams, IndexType};
use milvus::options::LoadOptions;
use milvus::query::SearchOptions;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder};
use milvus::{
    client::Client, data::FieldColumn, error::Error, schema::FieldSchema,
};

use half::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

const FP16_VEC_FIELD: &str = "float16_vector_field";
const BF16_VEC_FIELD: &str = "bfloat16_vector_field";

const DIM: i64 = 64;

#[tokio::main]
async fn main() -> Result<(), Error> {
    const URL: &str = "http://localhost:19530";

    let client = Client::new(URL).await?;

    let schema =
        CollectionSchemaBuilder::new("milvus_fp16", "fp16/bf16 example for milvus rust SDK")
            .add_field(FieldSchema::new_primary_int64(
                "id",
                "primary key field",
                true,
            ))
            .add_field(FieldSchema::new_float16_vector(
                FP16_VEC_FIELD,
                "fp16 feature field",
                DIM,
            ))
            .add_field(FieldSchema::new_bfloat16_vector(
                BF16_VEC_FIELD,
                "bf16 feature field",
                DIM,
            ))
            .build()?;
    client.create_collection(schema.clone(), None).await?;

    if let Err(err) = fp16_insert_and_query(&client, &schema).await {
        println!("failed to run hello milvus: {:?}", err);
    }
    client.drop_collection(schema.name()).await?;

    Ok(())
}

fn gen_random_f32_vector(n: i64) -> Vec<f32> {
    let mut data = Vec::<f32>::with_capacity(n as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        data.push(rng.gen());
    }
    data
}

async fn fp16_insert_and_query(
    client: &Client,
    collection: &CollectionSchema,
) -> Result<(), Error> {
    let mut embed_data = Vec::<f32>::new();
    for _ in 1..=DIM * 1000 {
        let mut rng = rand::thread_rng();
        let embed = rng.gen();
        embed_data.push(embed);
    }

    // fp16 or bf16 vector accept Vec<f32>, Vec<f64> or Vec<f16>/Vec<bf16> as input
    let bf16_column = FieldColumn::new(
        collection.get_field(BF16_VEC_FIELD).unwrap(),
        Vec::<bf16>::from_f32_slice(embed_data.as_slice()),
    )?;
    let fp16_column = FieldColumn::new(collection.get_field(FP16_VEC_FIELD).unwrap(), embed_data)?;

    let result = client
        .insert(collection.name(), vec![fp16_column, bf16_column], None)
        .await?;
    println!("insert cnt: {}", result.insert_cnt);
    client.flush(collection.name()).await?;

    let create_index_fut = [FP16_VEC_FIELD, BF16_VEC_FIELD].map(|field_name| {
        let index_params = IndexParams::new(
            field_name.to_string() + "_index",
            IndexType::IvfFlat,
            milvus::index::MetricType::L2,
            HashMap::from([("nlist".to_owned(), "32".to_owned())]),
        );
        client.create_index(collection.name(), field_name, index_params)
    });
    futures::future::try_join_all(create_index_fut).await?;
    client.flush(collection.name()).await?;
    client
        .load_collection(collection.name(), Some(LoadOptions::default()))
        .await?;

    // search
    let q1 = Vec::<f16>::from_f32_slice(&gen_random_f32_vector(DIM));
    let q2 = Vec::<f16>::from_f32_slice(&gen_random_f32_vector(DIM));
    let option = SearchOptions::with_limit(3)
        .metric_type(milvus::index::MetricType::L2)
        .output_fields(vec!["id".to_owned(), FP16_VEC_FIELD.to_owned()]);
    let result = client
        .search(
            collection.name(),
            vec![q1.into(), q2.into()],
            FP16_VEC_FIELD,
            &option,
        )
        .await?;

    println!("{:?}", result[0]);
    println!("result num: {}, {}", result[0].size, result[1].size);

    Ok(())
}
