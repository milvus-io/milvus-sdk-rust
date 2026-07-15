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
use milvus::data::FieldColumn;
use milvus::database::CreateDbOptions;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::CreateCollectionOptions;
use milvus::proto::common::KeyValuePair;
use milvus::query::{
    AnnSearchRequest, GetOptions, HybridSearchOptions, IdType, QueryOptions, RrfRanker,
    SearchOptions, WeightedRanker,
};
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use std::collections::HashMap;

use super::common::*;

const DIM: i64 = 8;
const ROW_COUNT: i64 = 80;

#[tokio::test]
async fn query_search_hybrid_search_and_get() -> Result<()> {
    let client = Client::new(URL).await?;
    let collection_name = format!("test_query_{}", gen_random_name());

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let schema = CollectionSchemaBuilder::new(&collection_name, "query test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(FieldSchema::new_int64("age", ""))
            .add_field(FieldSchema::new_double("deposit", ""))
            .add_field(FieldSchema::new_float_vector("picture", "", DIM))
            .add_field(FieldSchema::new_float_vector("face", "", DIM))
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let ids: Vec<i64> = (0..ROW_COUNT).collect();
        let ages: Vec<i64> = (0..ROW_COUNT).map(|i| i % 100).collect();
        let deposits: Vec<f64> = (0..ROW_COUNT).map(|i| i as f64).collect();
        let pictures = gen_random_f32_vector_custom(ROW_COUNT, DIM);
        let faces = gen_random_f32_vector_custom(ROW_COUNT, DIM);

        client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("id").unwrap(), ids),
                    FieldColumn::new(schema.get_field("age").unwrap(), ages),
                    FieldColumn::new(schema.get_field("deposit").unwrap(), deposits),
                    FieldColumn::new(schema.get_field("picture").unwrap(), pictures),
                    FieldColumn::new(schema.get_field("face").unwrap(), faces),
                ],
                None,
            )
            .await?;
        client.flush(&collection_name).await?;

        for (field, index_name) in [("picture", "picture_index"), ("face", "face_index")] {
            let index_params = IndexParams::new(
                index_name.to_string(),
                IndexType::IvfFlat,
                MetricType::L2,
                HashMap::from([("nlist".to_string(), "32".to_string())]),
            );
            client
                .create_index(&collection_name, field, index_params)
                .await?;
        }
        client.load_collection(&collection_name, None).await?;

        let query_options = QueryOptions::new()
            .limit(10)
            .output_fields(vec!["id".to_string(), "age".to_string()]);
        let query_result = client
            .query(&collection_name, "age >= 10 and age < 20", &query_options)
            .await?;
        assert!(query_result.iter().any(|field| field.name == "id"));
        assert!(query_result.iter().any(|field| field.name == "age"));
        assert!(query_result.first().is_some_and(|field| field.len() > 0));

        let get_options =
            GetOptions::new().output_fields(vec!["id".to_string(), "deposit".to_string()]);
        let get_result = client
            .get(
                &collection_name,
                IdType::Int64(vec![1, 2, 3]),
                Some(get_options),
            )
            .await?;
        assert!(get_result.iter().any(|field| field.name == "id"));
        assert!(get_result.first().is_some_and(|field| field.len() == 3));

        let search_vector = gen_random_f32_vector_custom(1, DIM);
        let search_options = SearchOptions::with_limit(5)
            .output_fields(vec!["id".to_string(), "age".to_string()])
            .anns_field(vec!["picture".to_string()])
            .add_param("metric_type", "L2")
            .add_param("nprobe", "10");
        let search_result = client
            .search(
                &collection_name,
                vec![Value::FloatArray(search_vector.clone().into())],
                Some(search_options),
            )
            .await?;
        assert_eq!(search_result.len(), 1);
        assert!(search_result[0].size > 0);

        let picture_req = AnnSearchRequest::new(
            vec![Value::FloatArray(search_vector.into())],
            "picture".to_string(),
            vec![
                KeyValuePair {
                    key: "metric_type".to_string(),
                    value: "L2".to_string(),
                },
                KeyValuePair {
                    key: "nprobe".to_string(),
                    value: "10".to_string(),
                },
            ],
            5,
        );
        let face_req = AnnSearchRequest::new(
            vec![Value::FloatArray(
                gen_random_f32_vector_custom(1, DIM).into(),
            )],
            "face".to_string(),
            vec![
                KeyValuePair {
                    key: "metric_type".to_string(),
                    value: "L2".to_string(),
                },
                KeyValuePair {
                    key: "nprobe".to_string(),
                    value: "10".to_string(),
                },
            ],
            5,
        );
        let hybrid_options = SearchOptions::with_limit(5).output_fields(vec!["id".to_string()]);
        let hybrid_result = client
            .hybrid_search(
                &collection_name,
                vec![picture_req, face_req],
                Box::new(WeightedRanker::new(vec![0.5, 0.5])),
                Some(hybrid_options),
            )
            .await?;
        assert_eq!(hybrid_result.len(), 1);
        assert!(hybrid_result[0].size > 0);

        Ok(())
    })
    .await
}

#[tokio::test]
async fn count_api() -> Result<()> {
    let entity_count: i64 = 300;
    let (client, schema) =
        create_empty_test_collection_custom(false, DEFAULT_DIM, DEFAULT_VEC_FIELD).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        // Insert deterministic sequential IDs so the filtered count can be asserted exactly.
        let ids: Vec<i64> = (0..entity_count).collect();
        let feature_data = gen_random_f32_vector_custom(entity_count, DEFAULT_DIM);
        let id_column = FieldColumn::new(schema.get_field("id").unwrap(), ids);
        let feature_column =
            FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), feature_data);
        client
            .insert(&collection_name, vec![id_column, feature_column], None)
            .await?;
        client.flush(&collection_name).await?;
        client
            .create_index(
                &collection_name,
                DEFAULT_VEC_FIELD,
                IndexParams::new(
                    DEFAULT_INDEX_NAME.to_string(),
                    IndexType::IvfFlat,
                    MetricType::L2,
                    HashMap::new(),
                ),
            )
            .await?;
        client.load_collection(&collection_name, None).await?;

        // Wait for asynchronous loading before issuing aggregate queries.
        for _ in 0..20 {
            if client.get_load_state(&collection_name, None).await?
                == milvus::proto::common::LoadState::Loaded
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        let whole = client.count(&collection_name).await?;
        assert_eq!(whole, entity_count, "whole-collection count");

        let filtered = client
            .count_with_expr(
                &collection_name,
                "id < 150",
                &milvus::query::QueryOptions::new(),
            )
            .await?;
        assert_eq!(filtered, 150, "filtered count matches strict subset");

        Ok(())
    })
    .await
}

async fn cleanup_consistency_case(
    query_client: &mut Client,
    db_name: &str,
    collection_name: &str,
) -> Result<()> {
    if db_name != "default" {
        query_client.using_database(db_name).await?;
    }
    if query_client
        .has_collection(collection_name)
        .await
        .unwrap_or(false)
    {
        query_client.drop_collection(collection_name).await?;
    }
    if db_name != "default" {
        query_client.using_database("default").await?;
        let databases = query_client.list_databases().await?;
        if databases.iter().any(|db| db == db_name) {
            query_client.drop_database(db_name).await?;
        }
    }
    Ok(())
}

async fn create_consistency_collection(
    query_client: &mut Client,
    db_name: &str,
    collection_name: &str,
    consistency_level: ConsistencyLevel,
) -> Result<CollectionSchema> {
    if db_name != "default" {
        let databases = query_client.list_databases().await?;
        if !databases.iter().any(|db| db == db_name) {
            query_client
                .create_database(db_name, Some(CreateDbOptions::new()))
                .await?;
        }
        query_client.using_database(db_name).await?;
    }

    let schema = CollectionSchemaBuilder::new(collection_name, "consistency test")
        .add_field(FieldSchema::new_primary_int64("pk", "", false))
        .add_field(FieldSchema::new_float_vector("vector", "", 4))
        .build()?;

    query_client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                consistency_level,
            )),
        )
        .await?;
    query_client
        .create_index(
            collection_name,
            "vector",
            IndexParams::new(
                "vector_idx".to_string(),
                IndexType::Flat,
                MetricType::L2,
                HashMap::new(),
            ),
        )
        .await?;
    query_client.load_collection(collection_name, None).await?;

    Ok(schema)
}

async fn run_consistency_visibility_case(
    query_client: &mut Client,
    db_name: &str,
    consistency_level: ConsistencyLevel,
) -> Result<()> {
    let collection_name = format!("test_consistency_{}", gen_random_name());
    let mut cleanup = CollectionCleanup::in_database(db_name, [&collection_name]);
    let schema =
        create_consistency_collection(query_client, db_name, &collection_name, consistency_level)
            .await?;

    let mut insert_client = query_client.clone();
    if db_name != "default" {
        insert_client.using_database(db_name).await?;
    }
    if db_name == "default" {
        insert_client.using_database("default").await?;
    }

    for i in 0..20_i64 {
        let row_vector = vec![i as f32, i as f32 + 0.1, i as f32 + 0.2, i as f32 + 0.3];
        insert_client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("pk").unwrap(), vec![i]),
                    FieldColumn::new(schema.get_field("vector").unwrap(), row_vector.clone()),
                ],
                None,
            )
            .await?;

        let filter = format!("pk == {}", i);
        if i % 3 == 0 {
            let query_resp = query_client
                .query(
                    &collection_name,
                    &filter,
                    &QueryOptions::new().output_fields(vec!["pk".to_string()]),
                )
                .await?;
            let pk_column = query_resp.iter().find(|field| field.name == "pk").unwrap();
            assert_eq!(pk_column.len(), 1);
        } else if i % 2 == 0 {
            let search_resp = query_client
                .search(
                    &collection_name,
                    vec![Value::FloatArray(row_vector.clone().into())],
                    Some(
                        SearchOptions::with_limit(10)
                            .anns_field(vec!["vector".to_string()])
                            .filter(filter.clone())
                            .add_param("metric_type", "L2"),
                    ),
                )
                .await?;
            assert_eq!(search_resp.len(), 1);
            assert_eq!(search_resp[0].size, 1);
        } else {
            let sub_req = AnnSearchRequest::new(
                vec![Value::FloatArray(row_vector.into())],
                "vector".to_string(),
                vec![KeyValuePair {
                    key: "metric_type".to_string(),
                    value: "L2".to_string(),
                }],
                7,
            )
            .with_expr(filter.clone());
            let search_resp = query_client
                .hybrid_search(
                    &collection_name,
                    vec![sub_req],
                    Box::new(RrfRanker::new(20.0)),
                    Some(HybridSearchOptions::with_limit(5)),
                )
                .await?;
            assert_eq!(search_resp.len(), 1);
            assert_eq!(search_resp[0].size, 1);
        }
    }

    cleanup_consistency_case(query_client, db_name, &collection_name).await?;
    cleanup.disarm();
    Ok(())
}

#[tokio::test]
async fn consistency_level_visibility_parity() -> Result<()> {
    let mut client = Client::new(URL).await?;
    let temp_db_name = format!("test_level_db_{}", gen_random_name());

    let databases = client.list_databases().await?;
    if !databases.iter().any(|db| db == &temp_db_name) {
        client
            .create_database(&temp_db_name, Some(CreateDbOptions::new()))
            .await?;
    }

    let test_result: Result<()> = async {
        run_consistency_visibility_case(&mut client, "default", ConsistencyLevel::Session).await?;
        run_consistency_visibility_case(&mut client, &temp_db_name, ConsistencyLevel::Session)
            .await?;
        run_consistency_visibility_case(&mut client, "default", ConsistencyLevel::Strong).await?;
        run_consistency_visibility_case(&mut client, &temp_db_name, ConsistencyLevel::Strong)
            .await?;
        Ok(())
    }
    .await;

    client.using_database("default").await?;
    let databases = client.list_databases().await?;
    if databases.iter().any(|db| db == &temp_db_name) {
        client.drop_database(&temp_db_name).await?;
    }

    test_result
}

#[tokio::test]
async fn search_with_cosine_metric() -> Result<()> {
    let collection_name = format!("test_cosine_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let mut cleanup = CollectionCleanup::new([&collection_name]);
    let dim: i64 = 16;

    let schema = CollectionSchemaBuilder::new(&collection_name, "cosine test")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("embedding", "", dim))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Insert data
    let embed_data = gen_random_f32_vector_custom(100, dim);
    let embed_col = FieldColumn::new(schema.get_field("embedding").unwrap(), embed_data);
    client
        .insert(&collection_name, vec![embed_col], None)
        .await?;
    client.flush(&collection_name).await?;

    // Create index with COSINE metric
    let index_params = IndexParams::new(
        "cosine_idx".to_owned(),
        IndexType::HNSW,
        MetricType::COSINE,
        HashMap::from([
            ("M".to_owned(), "16".to_owned()),
            ("efConstruction".to_owned(), "64".to_owned()),
        ]),
    );
    client
        .create_index(&collection_name, "embedding", index_params)
        .await?;

    client.load_collection(&collection_name, None).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Search with COSINE
    let query_vec = gen_random_f32_vector_custom(1, dim);
    let option = SearchOptions::with_limit(5)
        .output_fields(vec!["id".to_owned()])
        .add_param("ef", "64");

    let result = client
        .search(&collection_name, vec![query_vec.into()], Some(option))
        .await?;

    assert!(!result.is_empty());
    assert!(result[0].size > 0);

    client.drop_collection(&collection_name).await?;
    cleanup.disarm();
    Ok(())
}
