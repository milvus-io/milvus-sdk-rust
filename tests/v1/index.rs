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
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::CreateCollectionOptions;
use milvus::proto::common::KeyValuePair;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

use super::common::*;

#[tokio::test]
async fn index_management_lifecycle() -> Result<()> {
    let client = Client::new(URL).await?;
    let collection_name = format!("test_index_{}", gen_random_name());

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let vector_field = "embedding".to_string();
        let scalar_field = "category".to_string();
        let vector_index = "embedding_index".to_string();
        let scalar_index = "category_index".to_string();

        let schema = CollectionSchemaBuilder::new(&collection_name, "index test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(FieldSchema::new_float_vector("embedding", "", 8))
            .add_field(FieldSchema::new_varchar("category", "", 128))
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let row_count = 30;
        let ids: Vec<i64> = (0..row_count).collect();
        let vectors = gen_random_f32_vector_custom(row_count, 8);
        let categories: Vec<String> = (0..row_count).map(|i| format!("cat_{}", i % 3)).collect();

        client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("id").unwrap(), ids),
                    FieldColumn::new(schema.get_field("embedding").unwrap(), vectors),
                    FieldColumn::new(schema.get_field("category").unwrap(), categories),
                ],
                None,
            )
            .await?;
        client.flush(&collection_name).await?;

        let vector_params = IndexParams::new(
            vector_index.clone(),
            IndexType::IvfFlat,
            MetricType::L2,
            HashMap::from([("nlist".to_string(), "32".to_string())]),
        );
        client
            .create_index(collection_name.clone(), vector_field.clone(), vector_params)
            .await?;

        let scalar_params = IndexParams::new(
            scalar_index.clone(),
            IndexType::Trie,
            MetricType::L2,
            HashMap::new(),
        );
        client
            .create_index(collection_name.clone(), scalar_field, scalar_params)
            .await?;

        let all_indexes = client.list_indexes(&collection_name, None).await?;
        assert!(all_indexes.contains(&vector_index));
        assert!(all_indexes.contains(&scalar_index));

        let vector_indexes = client
            .list_indexes(collection_name.clone(), Some(vector_field.clone()))
            .await?;
        assert_eq!(vector_indexes, vec![vector_index.clone()]);

        let descriptions = client
            .describe_index(collection_name.clone(), vector_field)
            .await?;
        assert!(descriptions
            .iter()
            .any(|index| index.params().name() == &vector_index));

        client
            .alter_index_properties(
                collection_name.clone(),
                vector_index.clone(),
                vec![KeyValuePair {
                    key: "mmap.enabled".to_string(),
                    value: "false".to_string(),
                }],
            )
            .await?;
        client
            .drop_index_properties(
                collection_name.clone(),
                vector_index.clone(),
                vec!["mmap.enabled".to_string()],
            )
            .await?;

        client
            .drop_index(collection_name.clone(), scalar_index)
            .await?;
        client
            .drop_index(collection_name.clone(), vector_index)
            .await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn index_type_inverted() -> Result<()> {
    let collection_name = format!("test_inverted_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let mut cleanup = CollectionCleanup::new([&collection_name]);

    let schema = CollectionSchemaBuilder::new(&collection_name, "inverted index test")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .add_field(FieldSchema::new_varchar("category", "", 128))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Create INVERTED index on varchar field
    let index_params = IndexParams::new(
        "category_idx".to_owned(),
        IndexType::Inverted,
        MetricType::L2, // metric type not used for scalar index
        HashMap::new(),
    );
    client
        .create_index(&collection_name, "category", index_params)
        .await?;

    client.drop_collection(&collection_name).await?;
    cleanup.disarm();
    Ok(())
}
