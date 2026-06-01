#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::error::Result;

#[tokio::test]
async fn test_get_collection_stats() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let stats = client.get_collection_stats(collection.name()).await?;
        assert!(stats.contains_key("row_count"));
        Ok(())
    })
    .await
}
