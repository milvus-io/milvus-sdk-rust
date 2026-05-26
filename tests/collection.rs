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

use milvus::client::{Client, ConsistencyLevel};
use milvus::collection::{Collection, ParamValue};
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::mutate::{InsertOptions, UpsertOptions};
use milvus::options::LoadOptions;
use milvus::query::{QueryOptions, SearchOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

mod common;
use common::*;

use milvus::value::ValueVec;

#[tokio::test]
async fn manual_compaction_empty_collection() -> Result<()> {
    let collection_name = format!("manual_compaction_empty_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", DEFAULT_DIM))
        .build()?;
    client.create_collection(schema.clone(), None).await?;
    let _resp = client.manual_compaction(schema.name(), None).await?;
    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_upsert() -> Result<()> {
    let collection_name = format!("collection_upsert_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", DEFAULT_DIM))
        .build()?;
    client.create_collection(schema.clone(), None).await?;

    let row_count = 200;
    let pk_data = (0..row_count).map(|i| i as i64).collect::<Vec<_>>();
    let vec_data = gen_random_f32_vector_custom(row_count, DEFAULT_DIM);
    let pk_col = FieldColumn::new(schema.get_field("id").unwrap(), pk_data);
    let vec_col = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), vec_data);
    let result = client
        .upsert(schema.name(), vec![pk_col, vec_col], None::<UpsertOptions>)
        .await?;
    assert_eq!(result.upsert_cnt, row_count as i64, "{:?}", result);
    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_basic() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    let options = QueryOptions::default().limit(10);
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

    let index_params = IndexParams::new(
        DEFAULT_INDEX_NAME.to_owned(),
        IndexType::IvfFlat,
        milvus::index::MetricType::L2,
        HashMap::from([("nlist".to_owned(), "32".to_owned())]),
    );
    let index_list = client
        .describe_index(schema.name(), DEFAULT_VEC_FIELD)
        .await?;
    assert!(index_list.len() == 1, "{}", index_list.len());
    let index = &index_list[0];

    assert_eq!(index.params().name(), index_params.name());
    assert_eq!(
        index.params().extra_params().get("index_type"),
        Some(&"IVF_FLAT".to_string())
    );
    assert_eq!(
        index.params().extra_params().get("metric_type"),
        Some(&"L2".to_string())
    );

    client.drop_index(schema.name(), DEFAULT_VEC_FIELD).await?;
    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_search() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    sleep(Duration::from_millis(100)).await;

    let mut option = SearchOptions::with_limit(10).output_fields(vec!["id".to_owned()]);
    option = option.add_param("nprobe", "16");
    let query_vec = gen_random_f32_vector_custom(1, DEFAULT_DIM);

    let result = client
        .search(schema.name(), vec![query_vec.into()], Some(option))
        .await?;

    assert_eq!(result[0].size, 10);

    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_range_search() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    sleep(Duration::from_millis(100)).await;

    let radius_limit: f32 = 20.0;
    let range_filter_limit: f32 = 10.0;

    let mut option = SearchOptions::with_limit(5).output_fields(vec!["id".to_owned()]);
    option = option.add_param("nprobe", "16");
    option = option.radius(radius_limit);
    let query_vec = gen_random_f32_vector_custom(1, DEFAULT_DIM);

    let result = client
        .search(schema.name(), vec![query_vec.into()], Some(option))
        .await?;

    for record in &result {
        for value in &record.score {
            assert!(*value >= range_filter_limit && *value <= radius_limit);
        }
    }

    client.drop_collection(schema.name()).await?;
    Ok(())
}
