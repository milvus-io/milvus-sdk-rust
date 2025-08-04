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
    let index_params =
        IndexParams::new(
            "feature_index".to_string(),
            IndexType::Flat,
            MetricType::L2,
            HashMap::new(),
        );
    client
        .create_index(collection.name(), "feature", index_params)
        .await
        .unwrap();
    client.load_collection(collection.name(), None).await.unwrap();
    let result = client.release_collection(collection.name()).await;

    assert!(result.is_ok());
}
