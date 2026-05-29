#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::error::Result;

#[tokio::test]
async fn test_list_collections() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let collections = client.list_collections().await?;
        assert!(collections.contains(&collection.name().to_string()));
        Ok(())
    })
    .await
}
