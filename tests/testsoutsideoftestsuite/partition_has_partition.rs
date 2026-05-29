#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::error::Result;

#[tokio::test]
async fn test_has_partition() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client
            .create_partition(collection.name().to_string(), "test_partition".to_string())
            .await?;

        let has_partition = client
            .has_partition(collection.name().to_string(), "test_partition".to_string())
            .await?;
        assert!(has_partition);
        Ok(())
    })
    .await
}
