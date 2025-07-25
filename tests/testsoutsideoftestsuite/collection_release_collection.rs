#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::{
    client::*,
    index::{IndexParams, IndexType, MetricType},
};
use std::collections::HashMap;

#[tokio::test]
async fn test_release_collection() {
    let (client, collection) = create_test_collection(true).await.unwrap();

    let result = client.release_collection(collection.name()).await;

    assert!(result.is_ok());
}
