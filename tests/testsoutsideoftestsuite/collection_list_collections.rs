#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_list_collections() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    let result = client.list_collections().await;

    assert!(result.is_ok());

    let collections = result.unwrap();
    assert!(collections.contains(&collection.name().to_string()));
}
