#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::error::Result;

#[tokio::test]
async fn test_release_collection() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.release_collection(collection.name()).await?;
        Ok(())
    })
    .await
}
