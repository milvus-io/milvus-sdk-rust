#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_get_collection_stats() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    let result = client.get_collection_stats(collection.name()).await;

    assert!(result.is_ok());

    let stats = result.unwrap();
    assert!(stats.contains_key("row_count"));
}
