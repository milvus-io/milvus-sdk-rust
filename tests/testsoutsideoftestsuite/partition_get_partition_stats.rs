#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::error::Result;

#[tokio::test]
async fn test_get_partition_stats() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client
            .create_partition(collection.name().to_string(), "test_partition".to_string())
            .await?;

        let stats = client
            .get_partition_stats(collection.name().to_string(), "test_partition".to_string())
            .await?;
        assert!(stats.contains_key("row_count"));
        Ok(())
    })
    .await
}
