use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::iterator::{QueryIteratorOptions, SearchIteratorOptions};
use milvus::options::LoadOptions;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use rand::Rng;
use std::collections::HashMap;

const COLLECTION_NAME: &str = "test_rust_iterator";
const USER_ID: &str = "id";
const AGE: &str = "age";
const DEPOSIT: &str = "deposit";
const PICTURE: &str = "picture";
const DIM: i64 = 8;
const NUM_ENTITIES: i64 = 10000;

async fn test_search_iterator(client: &Client) -> Result<(), Error> {
    // Generate a random vector for search
    let mut rng = rand::thread_rng();
    let search_vector: Vec<f32> = (0..DIM).map(|_| rng.gen()).collect();

    // Create search iterator options
    let search_options = SearchIteratorOptions::new()
        .batch_size(50)
        .anns_field(PICTURE.to_string())
        .add_search_param("metric_type".to_string(), "L2".to_string())
        .add_search_param("params".to_string(), "{\"nprobe\": 10}".to_string())
        .reduce_stop_for_best(true)
        .limit(10);

    println!("Testing search iterator...");
    println!("Search vector: {:?}", search_vector);

    // Create search iterator
    let mut search_iterator = client
        .search_iterator(
            COLLECTION_NAME,
            vec![Value::FloatArray(search_vector.into())],
            search_options,
        )
        .await
        .expect("Failed to create search iterator");

    let mut page_idx = 0;
    loop {
        let res = search_iterator.next().await?;
        match res {
            Some(results) => {
                if results.is_empty() {
                    println!("search iteration finished, close");
                    search_iterator.close();
                    break;
                }

                // Print each search result in Python-like format
                for (i, result) in results.iter().enumerate() {
                    println!("Search result {}: {{", i);
                    println!("  size: {}", result.size);
                    println!("  scores: {:?}", result.score);
                    println!("  ids: {:?}", result.id);

                    // Print field data in entity format
                    if !result.field.is_empty() {
                        println!("  entity: {{");
                        for field in &result.field {
                            println!("    {}: {:?}", field.name, field.value);
                        }
                        println!("  }}");
                    }
                    println!("}}");
                }

                page_idx += 1;
                println!("page{}-------------------------", page_idx);
            }
            None => {
                println!("search iteration finished, close");
                search_iterator.close();
                break;
            }
        }
    }

    Ok(())
}

async fn test_query_iterator(client: &Client) -> Result<(), Error> {
    // Create query iterator options
    let query_options = QueryIteratorOptions::new()
        .batch_size(50)
        .filter(format!("10 <= {} <= 25", AGE))
        .output_fields(vec![USER_ID.to_string(), AGE.to_string()])
        .limit(1000);

    println!("Testing query iterator...");

    // Create query iterator
    let mut query_iterator = client
        .query_iterator(COLLECTION_NAME, query_options)
        .await
        .expect("Failed to create query iterator");

    let mut page_idx = 0;
    loop {
        let res = query_iterator.next().await?;
        match res {
            Some(fields) => {
                if fields.is_empty() {
                    println!("query iteration finished, close");
                    query_iterator.close();
                    break;
                }

                // Print each query result in Python-like format
                let num_records = if fields.is_empty() {
                    0
                } else {
                    fields[0].len()
                };
                for i in 0..num_records {
                    println!("Query result {}: {{", i);
                    for field in &fields {
                        if let Some(value) = field.get(i) {
                            println!("  {}: {:?}", field.name, value);
                        }
                    }
                    println!("}}");
                }

                page_idx += 1;
                println!("page{}-------------------------", page_idx);
            }
            None => {
                println!("query iteration finished, close");
                query_iterator.close();
                break;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new("http://localhost:19530").await?;

    if client.has_collection(COLLECTION_NAME).await? {
        client.drop_collection(COLLECTION_NAME).await?;
        println!("Dropped existed collection: {}", COLLECTION_NAME);
    }

    let schema = CollectionSchemaBuilder::new(COLLECTION_NAME, "Rust iterator example")
        .add_field(FieldSchema::new_primary_int64(USER_ID, "user id", false))
        .add_field(FieldSchema::new_int64(AGE, "age"))
        .add_field(FieldSchema::new_double(DEPOSIT, "deposit"))
        .add_field(FieldSchema::new_float_vector(PICTURE, "picture", DIM))
        .build()?;
    client.create_collection(schema.clone(), None).await?;

    let mut user_ids = Vec::with_capacity(NUM_ENTITIES as usize);
    let mut ages = Vec::with_capacity(NUM_ENTITIES as usize);
    let mut deposits = Vec::with_capacity(NUM_ENTITIES as usize);
    let mut pictures = Vec::with_capacity((NUM_ENTITIES * DIM) as usize);

    let mut rng = rand::thread_rng();
    for i in 0..NUM_ENTITIES {
        user_ids.push(i);
        ages.push((i % 100) as i64);
        deposits.push(i as f64);
        for _ in 0..DIM {
            pictures.push(rng.gen::<f32>());
        }
    }

    let columns = vec![
        FieldColumn::new(schema.get_field(USER_ID).unwrap(), user_ids),
        FieldColumn::new(schema.get_field(AGE).unwrap(), ages),
        FieldColumn::new(schema.get_field(DEPOSIT).unwrap(), deposits),
        FieldColumn::new(schema.get_field(PICTURE).unwrap(), pictures),
    ];
    client.insert(COLLECTION_NAME, columns, None).await?;
    client.flush(COLLECTION_NAME).await?;
    println!("Inserted and flushed {} entities.", NUM_ENTITIES);

    let index_params = IndexParams::new(
        "picture_index".to_owned(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    client
        .create_index(COLLECTION_NAME, PICTURE, index_params)
        .await?;
    client
        .load_collection(COLLECTION_NAME, Some(LoadOptions::default()))
        .await?;
    println!("Collection loaded and indexed.");

    println!("=== Testing Query Iterator ===");
    test_query_iterator(&client).await?;

    println!("\n=== Testing Search Iterator ===");
    test_search_iterator(&client).await?;

    client.drop_collection(COLLECTION_NAME).await?;

    Ok(())
}
