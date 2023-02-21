// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use milvus::client::ConsistencyLevel;
use milvus::collection::{Collection, SearchOption};
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use std::collections::HashMap;

mod common;
use common::*;

async fn clean_test_collection(collection: Collection) -> Result<()> {
    collection.drop().await?;
    Ok(())
}

#[tokio::test]
async fn collection_basic() -> Result<()> {
    let collection = create_test_collection().await?;

    let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);

    let embed_column = FieldColumn::new(
        collection.schema().get_field(DEFAULT_VEC_FIELD).unwrap(),
        embed_data,
    );

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;
    let index_params = IndexParams::new(
        DEFAULT_INDEX_NAME.to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    collection
        .create_index(DEFAULT_VEC_FIELD, index_params)
        .await?;
    collection.load(1).await?;

    let result = collection.query::<_, [&str; 0]>("id > 0", []).await?;

    println!(
        "result num: {}",
        result.first().map(|c| c.len()).unwrap_or(0),
    );

    clean_test_collection(collection).await?;
    Ok(())
}

#[tokio::test]
async fn collection_index() -> Result<()> {
    let collection = create_test_collection().await?;

    let feature = gen_random_f32_vector(DEFAULT_DIM * 2000);

    let feature_column = FieldColumn::new(
        collection.schema().get_field(DEFAULT_VEC_FIELD).unwrap(),
        feature,
    );

    collection.insert(vec![feature_column], None).await?;
    collection.flush().await?;

    let index_params = IndexParams::new(
        DEFAULT_INDEX_NAME.to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    collection
        .create_index(DEFAULT_VEC_FIELD, index_params.clone())
        .await?;
    let index_list = collection.describe_index(DEFAULT_VEC_FIELD).await?;
    assert!(index_list.len() == 1, "{}", index_list.len());
    let index = &index_list[0];

    assert_eq!(index.params().name(), index_params.name());
    assert_eq!(index.params().extra_params(), index_params.extra_params());

    collection.drop_index(DEFAULT_VEC_FIELD).await?;

    clean_test_collection(collection).await?;
    Ok(())
}

#[tokio::test]
async fn collection_search() -> Result<()> {
    let collection = create_test_collection().await?;

    let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
    let embed_column = FieldColumn::new(
        collection.schema().get_field(DEFAULT_VEC_FIELD).unwrap(),
        embed_data,
    );

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;
    let index_params = IndexParams::new(
        "ivf_flat".to_owned(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from_iter([("nlist".to_owned(), 32.to_string())]),
    );
    collection
        .create_index(DEFAULT_VEC_FIELD, index_params)
        .await?;
    collection.flush().await?;
    collection.load(1).await?;

    let query_vec = gen_random_f32_vector(DEFAULT_DIM);
    let result = collection
        .search(
            vec![query_vec.into()],
            DEFAULT_VEC_FIELD,
            10,
            MetricType::L2,
            vec!["id"],
            &SearchOption::default(),
        )
        .await?;

    assert_eq!(result[0].size, 10);

    clean_test_collection(collection).await?;
    Ok(())
}

#[tokio::test]
async fn session_consistency() -> Result<()> {
    let collection = create_test_collection().await?;

    let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
    let embed_column = FieldColumn::new(
        collection.schema().get_field(DEFAULT_VEC_FIELD).unwrap(),
        embed_data,
    );

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;
    let index_params = IndexParams::new(
        "ivf_flat".to_owned(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from_iter([("nlist".to_owned(), 32.to_string())]),
    );
    collection
        .create_index(DEFAULT_VEC_FIELD, index_params)
        .await?;
    collection.flush().await?;
    collection.load(1).await?;

    let query_vec = gen_random_f32_vector(DEFAULT_DIM);
    let mut options = SearchOption::default();
    options.set_consistency_level(ConsistencyLevel::Session);
    let result = collection
        .search(
            vec![query_vec.into()],
            DEFAULT_VEC_FIELD,
            10,
            MetricType::L2,
            vec!["id"],
            &options,
        )
        .await?;

    assert_eq!(result[0].size, 10);

    clean_test_collection(collection).await?;
    Ok(())
}
