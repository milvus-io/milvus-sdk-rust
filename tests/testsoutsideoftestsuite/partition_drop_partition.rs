#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_drop_partition() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    client
        .create_partition(collection.name().to_string(), "test_partition".to_string())
        .await
        .unwrap();
    let result = client
        .drop_partition(collection.name().to_string(), "test_partition".to_string())
        .await;

    assert!(result.is_ok());
}
