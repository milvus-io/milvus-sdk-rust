#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::client::*;

#[tokio::test]
async fn test_flush_collections() {
    let (client, collection) = create_test_collection(true).await.unwrap();
    let result = client.flush_collections(vec![collection.name()]).await;

    assert!(result.is_ok());
}
