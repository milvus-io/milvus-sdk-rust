#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_get_compaction_state() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    let compaction_info = client
        .manual_compaction(collection.name(), None)
        .await
        .unwrap();
    let result = client.get_compaction_state(compaction_info.id).await;

    assert!(result.is_ok());
}
