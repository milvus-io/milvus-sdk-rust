#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_create_partition() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    let result = client
        .create_partition(collection.name().to_string(), "test_partition".to_string())
        .await;

    assert!(result.is_ok());
}
