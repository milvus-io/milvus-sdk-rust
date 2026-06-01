#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::error::Result;

#[tokio::test]
async fn test_get_compaction_state() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let compaction_info = client.manual_compaction(collection.name(), None).await?;
        client.get_compaction_state(compaction_info.id).await?;
        Ok(())
    })
    .await
}
