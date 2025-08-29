use milvus::{
    client::Client,
    data::FieldColumn,
    index::{IndexParams, IndexType, MetricType},
    options::LoadOptions,
    query::QueryOptions,
    schema::{CollectionSchemaBuilder, FieldSchema},
};
use rand::Rng;
use std::collections::HashMap;

const DIM: i64 = 8;
const COLLECTION_NAME: &str = "hello_milvus";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://localhost:19530").await?;

    if client.has_collection(COLLECTION_NAME).await? {
        println!("Dropping existing collection: {}", COLLECTION_NAME);
        client.drop_collection(COLLECTION_NAME).await?;
    }

    let mut schema = CollectionSchemaBuilder::new(COLLECTION_NAME, "Hello Milvus collection");
    schema
        .add_field(FieldSchema::new_primary_int64("id", "", true)) // is_primary = true
        .add_field(FieldSchema::new_float_vector("embeddings", "", DIM))
        .add_field(FieldSchema::new_varchar("title", "", 64))
        .enable_dynamic_field();

    let schema = schema.build()?;

    println!("Creating collection");
    client.create_collection(schema.clone(), None).await?;

    println!("Start inserting entities");
    let mut rng = rand::thread_rng();

    let mut embeddings_data = Vec::new();
    let mut title_data = Vec::new();

    for i in 1..=6 {
        embeddings_data.extend((0..DIM).map(|_| rng.gen::<f32>()));
        title_data.push(format!("t{}", i));
    }

    let embeddings_col = FieldColumn::new(schema.get_field("embeddings").unwrap(), embeddings_data);
    let title_col = FieldColumn::new(schema.get_field("title").unwrap(), title_data);

    let insert_result = client
        .insert(COLLECTION_NAME, vec![embeddings_col, title_col], None)
        .await?;
    println!("Inserting entities done");
    println!("Insert result: {:?}", insert_result);

    client.flush(COLLECTION_NAME).await?;

    println!("Start create index for embeddings");
    let index_params = IndexParams::new(
        "embeddings_index".to_string(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from([("nlist".to_string(), "32".to_string())]),
    );
    client
        .create_index(COLLECTION_NAME, "embeddings", index_params)
        .await?;

    println!("Start create index for title");
    let title_index_params = IndexParams::new(
        "my_trie".to_string(),
        IndexType::Trie,
        MetricType::L2,
        HashMap::new(),
    );
    client
        .create_index(COLLECTION_NAME, "title", title_index_params)
        .await?;

    let index_names = client.list_indexes(COLLECTION_NAME, None).await?;
    println!("Index names for {}: {:?}", COLLECTION_NAME, index_names);

    for index_name in &index_names {
        let index_info = client.describe_index(COLLECTION_NAME, index_name).await?;
        println!("Index info for index {}: {:?}", index_name, index_info);
    }

    println!("Start load collection");
    client
        .load_collection(COLLECTION_NAME, Some(LoadOptions::default()))
        .await?;

    println!("Start query by specifying primary keys");
    let query_options = QueryOptions::default();
    let query_results = client
        .query(COLLECTION_NAME, "id == 2", &query_options)
        .await?;
    if let Some(result) = query_results.first() {
        println!("Query result: {:?}", result);
    }

    println!("Start query by specifying filtering expression");
    let query_results = client
        .query(COLLECTION_NAME, "title == 't2'", &query_options)
        .await?;
    for ret in query_results {
        println!("Query result: {:?}", ret);
    }

    let field_index_names = client
        .list_indexes(COLLECTION_NAME, Some("embeddings"))
        .await?;
    println!(
        "Index names for {}'s field embeddings: {:?}",
        COLLECTION_NAME, field_index_names
    );

    println!("Try to drop index");
    client.release_collection(COLLECTION_NAME).await?;

    match client.drop_index(COLLECTION_NAME, "my_trie").await {
        Ok(_) => println!("Successfully dropped index for title field"),
        Err(e) => println!("Caught error when dropping index: {}", e),
    }

    match client.drop_index(COLLECTION_NAME, "my_trie").await {
        Ok(_) => println!("Successfully dropped index for title field"),
        Err(e) => println!("Caught error when dropping index: {}", e),
    }

    client.drop_collection(COLLECTION_NAME).await?;

    println!("Example completed successfully!");
    Ok(())
}
