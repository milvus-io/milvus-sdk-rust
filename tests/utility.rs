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
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn get_server_version() -> Result<()> {
    let client = Client::new(URL).await?;
    let version = client.get_server_version().await?;
    println!("server version: {version:?}");
    assert!(!version.version.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_flush_collections() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.flush_collections(vec![collection.name()]).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn flush_all_and_get_state() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let flush_all_ts = client.flush_all().await?;
        client.get_flush_all_state(flush_all_ts).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn list_persistent_segments() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.flush(collection.name()).await?;
        let segments = client.list_persistent_segments(collection.name()).await?;
        assert!(!segments.is_empty());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn list_loaded_segments() -> Result<()> {
    let collection_name = format!("list_loaded_segments_{}", gen_random_name());
    let client = Client::new(URL).await?;

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let schema = CollectionSchemaBuilder::new(&collection_name, "")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(FieldSchema::new_float_vector(
                DEFAULT_VEC_FIELD,
                "",
                DEFAULT_DIM,
            ))
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let feature_data = gen_random_f32_vector_custom(ENTITYNUM, DEFAULT_DIM);
        let feature_column =
            FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), feature_data);
        client
            .insert(schema.name(), vec![feature_column], None)
            .await?;
        client.flush(schema.name()).await?;

        let mut persistent_segments = Vec::new();
        for _ in 0..150 {
            persistent_segments = client.list_persistent_segments(schema.name()).await?;
            if !persistent_segments.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        assert!(!persistent_segments.is_empty());

        client
            .create_index(
                schema.name(),
                DEFAULT_VEC_FIELD,
                IndexParams::new(
                    DEFAULT_INDEX_NAME.to_string(),
                    IndexType::HNSW,
                    MetricType::COSINE,
                    HashMap::from([
                        ("M".to_string(), "64".to_string()),
                        ("efConstruction".to_string(), "200".to_string()),
                    ]),
                ),
            )
            .await?;
        client.load_collection(schema.name(), None).await?;

        let segments = client.list_loaded_segments(schema.name()).await?;
        assert!(!segments.is_empty());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn manual_compaction_empty_collection() -> Result<()> {
    let collection_name = format!("manual_compaction_empty_{}", gen_random_name());
    let client = Client::new(URL).await?;

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
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
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_get_compaction_state() -> Result<()> {
    let (client, collection) = create_test_collection(true).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let compaction_info = client.manual_compaction(collection.name(), None).await?;
        client.get_compaction_state(compaction_info.id).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn run_analyzer_returns_tokens() -> Result<()> {
    let client = Client::new(URL).await?;
    let results = client
        .run_analyzer(
            vec![
                "Milvus is a vector database".to_string(),
                "Rust SDK integration tests".to_string(),
            ],
            "{\"type\": \"standard\"}",
        )
        .await?;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|result| !result.tokens.is_empty()));

    Ok(())
}
