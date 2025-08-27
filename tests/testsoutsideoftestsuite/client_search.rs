#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::{
    client::*,
    collection::*,
    data::FieldColumn,
    error::Result,
    index::{IndexParams, IndexType, MetricType},
    mutate::InsertOptions,
    query::SearchOptions,
    schema::CollectionSchema,
    value::Value,
};
use std::{borrow::Cow, collections::HashMap};

fn gen_f32_data(size: i64) -> Vec<f32> {
    let mut data = Vec::<f32>::with_capacity(size as usize);
    for i in 0..size {
        data.push(i as f32);
    }
    data
}

fn gen_i64_data(size: i64) -> Vec<i64> {
    let mut data = Vec::<i64>::with_capacity(size as usize);
    for i in 0..size {
        data.push(i as i64);
    }
    data
}

async fn insert_data(
    client: &Client,
    collection: &CollectionSchema,
    count: i64,
) -> Result<(Vec<i64>, Vec<f32>)> {
    let ids = gen_i64_data(count);
    let vectors = gen_f32_data(count * DEFAULT_DIM);

    let mut fields = Vec::new();
    fields.push(FieldColumn::new(
        collection.get_field("id").unwrap(),
        ids.clone(),
    ));
    fields.push(FieldColumn::new(
        collection.get_field(DEFAULT_VEC_FIELD).unwrap(),
        vectors.clone(),
    ));

    client.insert(collection.name(), fields, None).await?;
    client.flush(collection.name()).await?;

    Ok((ids, vectors))
}

#[tokio::test]
async fn test_search() {
    let (client, collection) = create_test_collection(false).await.unwrap();

    let (_ids, _vectors) = insert_data(&client, &collection, 10).await.unwrap();

    let index_params = IndexParams::new(
        "feature_index".to_string(),
        IndexType::Flat,
        MetricType::L2,
        HashMap::new(),
    );
    client
        .create_index(collection.name(), "feature", index_params)
        .await
        .unwrap();

    client
        .load_collection(collection.name(), None)
        .await
        .unwrap();

    let search_vectors = vec![Value::FloatArray(Cow::Owned(vec![
        0.0;
        DEFAULT_DIM as usize
    ]))];

    let result = client
        .search(
            collection.name(),
            search_vectors,
            DEFAULT_VEC_FIELD,
            &SearchOptions::default().limit(5),
        )
        .await;

    assert!(result.is_ok());

    let search_result = result.unwrap();
    assert_eq!(search_result.len(), 1);
    assert_eq!(search_result[0].score.len(), 5);
}
