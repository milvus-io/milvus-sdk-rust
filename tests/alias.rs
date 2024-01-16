mod common;
use common::*;
use milvus::{client::Client, error::Result};

async fn clean_test_collection(client: Client, collection_name: &str) -> Result<()> {
    client.drop_collection(collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn test_create_alias() -> Result<()> {
    let alias = "test_create_alias";
    let (client, schema) = create_test_collection(true).await?;
    client.create_alias(schema.name(), alias).await?;
    client.drop_alias(alias).await?;
    clean_test_collection(client, schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn test_alter_alias() -> Result<()> {
    let alias = "test_alter_alias";
    let (client1, schema1) = create_test_collection(true).await?;
    client1.create_alias(schema1.name(), alias).await?;
    let (client2, schema2) = create_test_collection(true).await?;
    client2.alter_alias(schema2.name(), alias).await?;
    client2.drop_alias(alias).await?;
    clean_test_collection(client1, schema1.name()).await?;
    clean_test_collection(client2, schema2.name()).await?;
    Ok(())
}
