#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_list_partitions() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    client
        .create_partition(collection.name().to_string(), "test_partition".to_string())
        .await
        .unwrap();

    let result = client.list_partitions(collection.name().to_string()).await;

    assert!(result.is_ok());
    let partitions = result.unwrap();
    assert!(partitions.contains(&"test_partition".to_string()));
}
