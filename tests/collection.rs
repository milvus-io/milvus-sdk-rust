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

use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType};
use milvus::mutate::UpsertOptions;
use milvus::options::GetLoadStateOptions;
use milvus::proto::common::KeyValuePair;
use milvus::proto::schema::{FunctionSchema, FunctionType};
use milvus::query::{QueryOptions, SearchOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

mod common;
use common::*;

#[tokio::test]
async fn manual_compaction_empty_collection() -> Result<()> {
    let collection_name = format!("manual_compaction_empty_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector(
            DEFAULT_VEC_FIELD,
            "",
            DEFAULT_DIM,
        ))
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
        .add_field(FieldSchema::new_float_vector(
            DEFAULT_VEC_FIELD,
            "",
            DEFAULT_DIM,
        ))
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
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let options = QueryOptions::default().limit(10);
        let result = client.query(schema.name(), "id > 0", &options).await?;

        println!(
            "result num: {}",
            result.first().map(|c| c.len()).unwrap_or(0),
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn collection_index() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
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
        Ok(())
    })
    .await
}

#[tokio::test]
async fn collection_search() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        sleep(Duration::from_millis(100)).await;

        let mut option = SearchOptions::with_limit(10).output_fields(vec!["id".to_owned()]);
        option = option.add_param("nprobe", "16");
        let query_vec = gen_random_f32_vector_custom(1, DEFAULT_DIM);

        let result = client
            .search(schema.name(), vec![query_vec.into()], Some(option))
            .await?;

        assert_eq!(result[0].size, 10);

        Ok(())
    })
    .await
}

#[tokio::test]
async fn collection_range_search() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
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

        Ok(())
    })
    .await
}

#[tokio::test]
async fn collection_refresh_load() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.refresh_load(schema.name()).await?;
        let state = client
            .get_load_state(
                schema.name(),
                Some(GetLoadStateOptions::default().partition_names(Vec::<String>::new())),
            )
            .await?;
        assert!(matches!(
            state,
            milvus::proto::common::LoadState::Loaded | milvus::proto::common::LoadState::Loading
        ));

        Ok(())
    })
    .await
}

#[tokio::test]
async fn collection_drop_field_properties() -> Result<()> {
    let collection_name = format!("test_collection_field_properties_{}", gen_random_name());
    let client = Client::new(URL).await?;

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let schema = CollectionSchemaBuilder::new(&collection_name, "field properties test")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(
                FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", DEFAULT_DIM)
                    .add_type_param("mmap.enabled", "true"),
            )
            .build()?;
        client.create_collection(schema, None).await?;

        let described = client.describe_collection(&collection_name).await?;
        let vector_field = described
            .schema
            .fields
            .iter()
            .find(|field| field.name == DEFAULT_VEC_FIELD)
            .expect("vector field should exist");
        assert!(vector_field
            .type_params
            .iter()
            .any(|param| param.key == "mmap.enabled" && param.value == "true"));

        client
            .drop_collection_field_properties(
                &collection_name,
                DEFAULT_VEC_FIELD,
                vec!["mmap.enabled".to_string()],
            )
            .await?;

        let described = client.describe_collection(&collection_name).await?;
        let vector_field = described
            .schema
            .fields
            .iter()
            .find(|field| field.name == DEFAULT_VEC_FIELD)
            .expect("vector field should exist");
        assert!(!vector_field
            .type_params
            .iter()
            .any(|param| param.key == "mmap.enabled"));

        Ok(())
    })
    .await
}

#[tokio::test]
async fn collection_bm25_function_in_schema() -> Result<()> {
    let collection_name = format!("test_collection_function_{}", gen_random_name());
    let client = Client::new(URL).await?;

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let function = FunctionSchema {
            name: "bm25_fn".to_string(),
            description: "BM25 text to sparse".to_string(),
            r#type: FunctionType::Bm25 as i32,
            input_field_names: vec!["text".to_string()],
            output_field_names: vec!["sparse".to_string()],
            ..Default::default()
        };
        let schema = CollectionSchemaBuilder::new(&collection_name, "collection function test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(
                FieldSchema::new_varchar("text", "", 4096)
                    .add_type_param("enable_analyzer", "true")
                    .add_type_param("enable_match", "true")
                    .add_type_param("analyzer_params", "{\"type\": \"standard\"}"),
            )
            .add_field(FieldSchema::new_sparse_float_vector("sparse", ""))
            .add_function(function)
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let described = client.describe_collection(&collection_name).await?;
        assert!(described
            .schema
            .functions
            .iter()
            .any(|function| function.name == "bm25_fn"));

        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore = "requires a configured text embedding provider"]
async fn collection_function_lifecycle() -> Result<()> {
    let collection_name = format!("test_collection_function_rpc_{}", gen_random_name());
    let client = Client::new(URL).await?;

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let function = FunctionSchema {
            name: "openai".to_string(),
            description: "OpenAI text embedding".to_string(),
            r#type: FunctionType::TextEmbedding as i32,
            input_field_names: vec!["document".to_string()],
            output_field_names: vec!["dense".to_string()],
            params: vec![
                KeyValuePair {
                    key: "provider".to_string(),
                    value: "openai".to_string(),
                },
                KeyValuePair {
                    key: "model_name".to_string(),
                    value: "text-embedding-3-small".to_string(),
                },
            ],
            ..Default::default()
        };
        let schema = CollectionSchemaBuilder::new(&collection_name, "collection function RPC test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(FieldSchema::new_varchar("document", "", 9000))
            .add_field(FieldSchema::new_float_vector("dense", "", 1536))
            .add_function(function.clone())
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let altered = FunctionSchema {
            params: vec![
                KeyValuePair {
                    key: "provider".to_string(),
                    value: "openai".to_string(),
                },
                KeyValuePair {
                    key: "model_name".to_string(),
                    value: "text-embedding-3-small".to_string(),
                },
                KeyValuePair {
                    key: "user".to_string(),
                    value: "user123".to_string(),
                },
            ],
            ..function.clone()
        };
        client
            .alter_collection_function(&collection_name, "openai", altered)
            .await?;
        client
            .drop_collection_function(&collection_name, "openai")
            .await?;

        let added = FunctionSchema {
            params: vec![
                KeyValuePair {
                    key: "provider".to_string(),
                    value: "openai".to_string(),
                },
                KeyValuePair {
                    key: "model_name".to_string(),
                    value: "text-embedding-3-small".to_string(),
                },
                KeyValuePair {
                    key: "user".to_string(),
                    value: "user1234".to_string(),
                },
            ],
            ..function
        };
        client
            .add_collection_function(&collection_name, added)
            .await?;

        Ok(())
    })
    .await
}
