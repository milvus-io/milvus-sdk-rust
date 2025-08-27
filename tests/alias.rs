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

#[tokio::test]
async fn test_describe_alias() -> Result<()> {
    let alias = "test_describe_alias";
    let (client, schema) = create_test_collection(true).await?;

    client.create_alias(schema.name(), alias).await?;
    let (alias, collection, db_name) = client.describe_alias(alias).await?;
    assert_eq!(alias, "test_describe_alias");
    assert_eq!(collection, schema.name());
    assert_eq!(db_name, "default");
    client.drop_alias(alias).await?;
    clean_test_collection(client, schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn list_aliases() -> Result<()> {
    let alias1 = "test_list_alias_1";
    let alias2 = "test_list_alias_2";
    let (client, schema) = create_test_collection(true).await?;
    client.create_alias(schema.name(), alias1).await?;
    client.create_alias(schema.name(), alias2).await?;

    let (db_name, collection_name, aliases) = client.list_aliases(schema.name()).await?;

    assert_eq!(db_name, "default");
    assert_eq!(collection_name, schema.name());

    // the result is not in order,so transfer to hashset
    let set1: std::collections::HashSet<_> = aliases.iter().collect();
    let vec = vec![alias1.to_string(), alias2.to_string()];
    let set2: std::collections::HashSet<_> = vec.iter().collect();
    assert_eq!(set1, set2);

    client.drop_alias(alias1).await?;
    client.drop_alias(alias2).await?;
    clean_test_collection(client, schema.name()).await?;
    Ok(())
}
