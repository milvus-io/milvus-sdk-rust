#[path = "../common/mod.rs"]
mod common;

use common::*;

#[tokio::test]
async fn test_drop_partition() -> milvus::error::Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client
            .create_partition(collection.name().to_string(), "test_partition".to_string())
            .await?;
        client.release_collection(collection.name()).await?;
        client
            .drop_partition(collection.name().to_string(), "test_partition".to_string())
            .await?;

        Ok(())
    })
    .await
}
