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
use milvus::collection::{Collection, ParamValue};
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::mutate::InsertOptions;
use milvus::options::LoadOptions;
use milvus::query::{QueryOptions, SearchOptions};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

mod common;
use common::*;

use milvus::value::ValueVec;

#[tokio::test]
async fn manual_compaction_empty_collection() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;
    let resp = client.manual_compaction(schema.name()).await?;
    assert_eq!(0, resp.plan_count);
    Ok(())
}

#[tokio::test]
async fn collection_upsert() -> Result<()> {
    let (client, schema) = create_test_collection(false).await?;
    let pk_data = gen_random_int64_vector(2000);
    let vec_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
    let pk_col = FieldColumn::new(schema.get_field("id").unwrap(), pk_data);
    let vec_col = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), vec_data);
    client
        .upsert(schema.name(), vec![pk_col, vec_col], None)
        .await?;
    let index_params = IndexParams::new(
        DEFAULT_INDEX_NAME.to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    client
        .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
        .await?;
    client
        .load_collection(schema.name(), Some(LoadOptions::default()))
        .await?;

    let options = QueryOptions::default();
    let options = options.output_fields(vec![String::from("count(*)")]);
    let result = client.query(schema.name(), "", &options).await?;
    if let ValueVec::Long(vec) = &result[0].value {
        assert_eq!(2000, vec[0]);
    } else {
        panic!("invalid result");
    }
    Ok(())
}

#[tokio::test]
async fn collection_basic() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);

    let embed_column = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), embed_data);

    client
        .insert(schema.name(), vec![embed_column], None)
        .await?;
    client.flush(schema.name()).await?;
    let index_params = IndexParams::new(
        DEFAULT_INDEX_NAME.to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    client
        .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
        .await?;
    client
        .load_collection(schema.name(), Some(LoadOptions::default()))
        .await?;

    let options = QueryOptions::default();
    let result = client.query(schema.name(), "id > 0", &options).await?;

    println!(
        "result num: {}",
        result.first().map(|c| c.len()).unwrap_or(0),
    );

    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_index() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    let feature = gen_random_f32_vector(DEFAULT_DIM * 2000);

    let feature_column = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), feature);

    client
        .insert(schema.name(), vec![feature_column], None)
        .await?;
    client.flush(schema.name()).await?;

    let index_params = IndexParams::new(
        DEFAULT_INDEX_NAME.to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    client
        .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params.clone())
        .await?;
    let index_list = client
        .describe_index(schema.name(), DEFAULT_VEC_FIELD)
        .await?;
    assert!(index_list.len() == 1, "{}", index_list.len());
    let index = &index_list[0];

    assert_eq!(index.params().name(), index_params.name());
    assert_eq!(index.params().extra_params(), index_params.extra_params());

    client.drop_index(schema.name(), DEFAULT_VEC_FIELD).await?;
    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_search() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
    let embed_column = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), embed_data);

    client
        .insert(schema.name(), vec![embed_column], None)
        .await?;
    client.flush(schema.name()).await?;
    let index_params = IndexParams::new(
        "ivf_flat".to_owned(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from_iter([("nlist".to_owned(), 32.to_string())]),
    );
    client
        .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
        .await?;
    client
        .load_collection(schema.name(), Some(LoadOptions::default()))
        .await?;

    sleep(Duration::from_millis(100)).await;

    let mut option = SearchOptions::with_limit(10)
        .metric_type(MetricType::L2)
        .output_fields(vec!["id".to_owned()]);
    option = option.add_param("nprobe", ParamValue!(16));
    let query_vec = gen_random_f32_vector(DEFAULT_DIM);

    let result = client
        .search(
            schema.name(),
            vec![query_vec.into()],
            DEFAULT_VEC_FIELD,
            &option,
        )
        .await?;

    assert_eq!(result[0].size, 10);

    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_range_search() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
    let embed_column = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), embed_data);

    client
        .insert(schema.name(), vec![embed_column], None)
        .await?;
    client.flush(schema.name()).await?;
    let index_params = IndexParams::new(
        "ivf_flat".to_owned(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from_iter([("nlist".to_owned(), 32.to_string())]),
    );
    client
        .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
        .await?;
    client
        .load_collection(schema.name(), Some(LoadOptions::default()))
        .await?;

    sleep(Duration::from_millis(100)).await;

    let radius_limit: f32 = 20.0;
    let range_filter_limit: f32 = 10.0;

    let mut option = SearchOptions::with_limit(5)
        .metric_type(MetricType::L2)
        .output_fields(vec!["id".to_owned()]);
    option = option.add_param("nprobe", ParamValue!(16));
    option = option.radius(radius_limit).range_filter(range_filter_limit);
    let query_vec = gen_random_f32_vector(DEFAULT_DIM);

    let result = client
        .search(
            schema.name(),
            vec![query_vec.into()],
            DEFAULT_VEC_FIELD,
            &option,
        )
        .await?;

    for record in &result {
        for value in &record.score {
            assert!(*value >= range_filter_limit && *value <= radius_limit);
        }
    }

    client.drop_collection(schema.name()).await?;
    Ok(())
}
