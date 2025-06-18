use milvus::error::Result;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::{client::Client, database::CreateDbOptions};

const TEST_NAME_A: &str = "test_database_A";
const TEST_NAME_B: &str = "test_database_B";

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("http://localhost:19530").await?;

    //create database with properties
    let db_properties = CreateDbOptions::new().replica_number(1).max_collections(3);
    client
        .create_database(TEST_NAME_A, Some(db_properties))
        .await?;
    let db_properties = CreateDbOptions::new().replica_number(1).max_collections(3);
    client
        .create_database(TEST_NAME_B, Some(db_properties))
        .await?;

    //list database
    let res = client.list_databases().await?;
    println!("After create databases:{:?}", res);

    //describe database
    let res = client.describe_database(TEST_NAME_A).await?;
    println!("Describe database_A : \n{:#?}", res);

    //alter database properties
    let options = CreateDbOptions::new().replica_number(0).max_collections(2);
    client
        .alter_database_properties(TEST_NAME_A, options)
        .await?;
    let res = client.describe_database(TEST_NAME_A).await?;
    println!("After alter database_A properties:\n{:#?}\n", res);

    //using database
    let collection_name_a = "collection_A";
    let collection_name_b = "collection_B";
    let schema_a = CollectionSchemaBuilder::new(collection_name_a, "For database test")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("vector_A", "", 128))
        .build()?;
    let schema_b = CollectionSchemaBuilder::new(collection_name_b, "For database test")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("vector_B", "", 128))
        .build()?;
    //use db_A
    client.using_database(TEST_NAME_A).await?;
    client.create_collection(schema_a, None).await?;
    let res = client.list_collections().await?;
    println!("Database A collections: {:?} ", res);
    client.drop_collection(collection_name_a).await?;
    //use db_B
    client.using_database(TEST_NAME_B).await?;
    client.create_collection(schema_b, None).await?;
    let res = client.list_collections().await?;
    println!("Database B collections: {:?}", res);
    //drop database
    client.drop_collection(collection_name_b).await?;

    client.drop_database(TEST_NAME_A).await?;
    client.drop_database(TEST_NAME_B).await?;
    let res = client.list_databases().await?;
    println!("After drop database A and B there are databases: {:?}", res);
    Ok(())
}
