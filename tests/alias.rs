mod common;
use common::*;
use milvus::{collection::Collection, error::Result};

async fn clean_test_collection(collection: Collection) -> Result<()> {
    collection.drop().await?;
    Ok(())
}

#[tokio::test]
async fn test_create_alias() -> Result<()> {
    let alias = "test_create_alias";
    let collection = create_test_collection().await?;
    collection.create_alias(alias).await?;
    collection.drop_alias(alias).await?;
    clean_test_collection(collection).await?;
    Ok(())
}

#[tokio::test]
async fn test_alter_alias() -> Result<()> {
    let alias = "test_alter_alias";
    let collection1 = create_test_collection().await?;
    collection1.create_alias(alias).await?;
    let collection2 = create_test_collection().await?;
    collection2.alter_alias(alias).await?;
    collection2.drop_alias(alias).await?;
    clean_test_collection(collection1).await?;
    clean_test_collection(collection2).await?;
    Ok(())
}
