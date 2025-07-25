use std::collections::HashMap;
use std::vec;

use milvus::client::Client;
use milvus::collection::SearchResult;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::query::*;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use rand::Rng;

const DIM: i64 = 8;
const NUM_ENTITIES: usize = 10000;
const PICTURE: &str = "picture";
const USER_ID: &str = "id";
const AGE: &str = "age";
const DEPOSIT: &str = "deposit";
const COLLECTION_NAME: &str = "test_query_search_collection";

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    prepare_data(&client).await?;
    query_test(&client).await?;
    search_test(&client).await?;
    get_test(&client).await?;
    Ok(())
}

// query test
async fn query_test(client: &Client) -> Result<()> {
    let options = QueryOptions::new()
        .limit(50)
        .output_fields(vec![USER_ID.to_string(), AGE.to_string()]);
    let res = client.query(COLLECTION_NAME, "10<age<20", &options).await?;
    println!("==========Query test begin==========");
    println!("Query result:");

    // Extract id and age columns from the result
    let id_column = res.iter().find(|col| col.name == USER_ID).unwrap();
    let age_column = res.iter().find(|col| col.name == AGE).unwrap();

    // Get the data vectors from the columns
    let ids: Vec<i64> = id_column.value.clone().try_into().unwrap();
    let ages: Vec<i64> = age_column.value.clone().try_into().unwrap();

    // Print the results in the requested format
    for (id, age) in ids.iter().zip(ages.iter()) {
        println!("id: {} age: {}", id, age);
    }
    println!("==========Query test end==========\n");
    Ok(())
}

// search test
async fn search_test(client: &Client) -> Result<()> {
    let vector_to_search = Value::from(
        (0..DIM as usize)
            .map(|_| rand::thread_rng().gen_range(0.0..1.0))
            .collect::<Vec<f32>>(),
    );
    println!("==========Search test begin==========");
    // Prepare search options
    let options = SearchOptions::new()
        .limit(10)
        .output_fields(vec![
            USER_ID.to_string(),
            AGE.to_string(),
            PICTURE.to_string(),
        ])
        .add_param("anns_field", "picture")
        .add_param("metric_type", "L2");

    // Search
    let res = client
        .search(COLLECTION_NAME, vec![vector_to_search], Some(options))
        .await?;

    println!("Search result:");
    print_search_results(&res);
    println!("==========Search test end==========\n");
    Ok(())
}

// get test
async fn get_test(client: &Client) -> Result<()> {
    // Prepare get options
    let options = GetOptions::new().output_fields(vec![
        USER_ID.to_string(),
        AGE.to_string(),
        DEPOSIT.to_string(),
        PICTURE.to_string(),
    ]);
    // Get
    let res = client
        .get(
            COLLECTION_NAME,
            IdType::Int64(vec![1, 2, 3, 4, 5]),
            Some(options),
        )
        .await?;
    println!("==========Get test begin==========");
    println!("Get result:");
    print_get_results(&res);
    println!("==========Get test end==========\n");
    Ok(())
}

// prepare data
async fn prepare_data(client: &Client) -> Result<()> {
    // Prepare data
    if client.has_collection(COLLECTION_NAME).await? {
        client.drop_collection(COLLECTION_NAME).await?;
    }
    println!("==========Prepare data begin==========");
    // 1. create collection
    let schema = CollectionSchemaBuilder::new(COLLECTION_NAME, "test_query_search_collection")
        .add_field(FieldSchema::new_primary_int64(USER_ID, "user if", false))
        .add_field(FieldSchema::new_int64(AGE, "age of user"))
        .add_field(FieldSchema::new_double(DEPOSIT, ""))
        .add_field(FieldSchema::new_float_vector(PICTURE, "", DIM))
        .build()?;

    client.create_collection(schema.clone(), None).await?;
    // 2. insert data
    let ids = (0..NUM_ENTITIES).map(|i| i as i64).collect::<Vec<_>>();
    let age = (0..NUM_ENTITIES)
        .map(|i| (i % 100) as i64)
        .collect::<Vec<_>>();
    let deposit = (0..NUM_ENTITIES).map(|i| i as f64).collect::<Vec<_>>();
    let picture = (0..NUM_ENTITIES * DIM as usize)
        .map(|_| rand::thread_rng().gen_range(0.0..1.0))
        .collect::<Vec<f32>>();

    let id_column = FieldColumn::new(schema.get_field(USER_ID).unwrap(), ids);
    let age_column = FieldColumn::new(schema.get_field(AGE).unwrap(), age);
    let deposit_column = FieldColumn::new(schema.get_field(DEPOSIT).unwrap(), deposit);
    let picture_column = FieldColumn::new(schema.get_field(PICTURE).unwrap(), picture);

    client
        .insert(
            COLLECTION_NAME,
            vec![id_column, age_column, deposit_column, picture_column],
            None,
        )
        .await?;
    client.flush(COLLECTION_NAME).await?;
    println!("Finish flush collections:{}", COLLECTION_NAME);

    // 3. create index
    let index_params = IndexParams::new(
        "picture_index".to_string(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from([("nlist".to_string(), "1024".to_string())]),
    );
    client
        .create_index(COLLECTION_NAME, PICTURE, index_params)
        .await?;
    client.load_collection(COLLECTION_NAME, None).await?;
    println!("==========Prepare data end==========\n");
    Ok(())
}

// Print functions.
// You can ignore this part.

fn print_search_results(res: &Vec<SearchResult<'_>>) {
    let id_column = res
        .iter()
        .map(|col| {
            col.field
                .iter()
                .find(|x| x.name == USER_ID)
                .unwrap()
                .value
                .clone()
        })
        .collect::<Vec<_>>();
    let age_column = res
        .iter()
        .map(|col| {
            col.field
                .iter()
                .find(|x| x.name == AGE)
                .unwrap()
                .value
                .clone()
        })
        .collect::<Vec<_>>();
    let picture_column = res
        .iter()
        .map(|col| {
            col.field
                .iter()
                .find(|x| x.name == PICTURE)
                .unwrap()
                .value
                .clone()
        })
        .collect::<Vec<_>>();
    let score_column = res.iter().map(|col| col.score.clone()).collect::<Vec<_>>();
    for (ids, ages, pictures, scores) in id_column
        .iter()
        .zip(age_column.iter())
        .zip(picture_column.iter())
        .zip(score_column.iter())
        .map(|(((id, age), picture), score)| {
            (id.clone(), age.clone(), picture.clone(), score.clone())
        })
    {
        let id_column: Vec<i64> = ids.clone().try_into().unwrap();
        let age_column: Vec<i64> = ages.clone().try_into().unwrap();
        let picture_column: Vec<f32> = pictures.clone().try_into().unwrap();
        let score_column: Vec<f32> = scores.clone().try_into().unwrap();
        for (id, age, picture, score) in id_column
            .iter()
            .zip(age_column.iter())
            .zip(picture_column.chunks(DIM as usize))
            .zip(score_column.iter())
            .map(|(((id, age), picture), score)| {
                (id.clone(), age.clone(), picture.to_vec(), score.clone())
            })
        {
            println!(
                "id: {} age: {} picture: {:?} score: {}",
                id, age, picture, score
            );
        }
    }
}

fn print_get_results(res: &Vec<FieldColumn>) {
    let id_column = res.iter().find(|col| col.name == USER_ID).unwrap();
    let age_column = res.iter().find(|col| col.name == AGE).unwrap();
    let deposit_column = res.iter().find(|col| col.name == DEPOSIT).unwrap();
    let picture_column = res.iter().find(|col| col.name == PICTURE).unwrap();

    let ids: Vec<i64> = id_column.value.clone().try_into().unwrap();
    let ages: Vec<i64> = age_column.value.clone().try_into().unwrap();
    let deposits: Vec<f64> = deposit_column.value.clone().try_into().unwrap();
    let pictures: Vec<f32> = picture_column.value.clone().try_into().unwrap();
    for (id, age, deposit, picture) in ids
        .iter()
        .zip(ages.iter())
        .zip(deposits.iter())
        .zip(pictures.chunks(DIM as usize))
        .map(|(((id, age), deposit), picture)| {
            (id.clone(), age.clone(), deposit.clone(), picture.to_vec())
        })
    {
        println!(
            "id: {} age: {} deposit: {} picture: {:?}",
            id, age, deposit, picture
        );
    }
}
