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

use milvus::client::{self, *};
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::IndexType;
use milvus::options::CreateCollectionOptions;
use milvus::proto::schema;
use milvus::query::{IdType, QueryOptions};
use milvus::{collection, schema::*};
use rand::Rng;
use std::collections::HashMap;
mod common;
use common::*;

#[tokio::test]
async fn create_client() -> Result<()> {
    match Client::new(URL).await {
        Ok(_) => Result::<()>::Ok(()),
        Err(e) => panic!("Error is {}.", e),
    }
}

#[tokio::test]
async fn create_client_wrong_url() -> Result<()> {
    const URL: &str = "http://localhost:9999";
    match Client::new(URL).await {
        Ok(_) => panic!("Should fail due to wrong url."),
        Err(_) => Result::<()>::Ok(()),
    }
}

#[tokio::test]
async fn create_client_wrong_fmt() -> Result<()> {
    const URL: &str = "9999";
    match Client::new(URL).await {
        Ok(_) => panic!("Should fail due to wrong format url."),
        Err(_) => Result::<()>::Ok(()),
    }
}

#[tokio::test]
async fn has_collection() -> Result<()> {
    const NAME: &str = "qwerty";
    let client = Client::new(URL).await?;
    match client.has_collection(NAME).await {
        Ok(has) => {
            if has {
                panic!("Expect no such collection.");
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

#[tokio::test]
async fn create_has_drop_collection() -> Result<()> {
    const NAME: &str = "create_has_drop_collection";

    let client = Client::new(URL).await?;
    // let client = ClientBuilder::new(URL).username("username").password("password").build().await?;

    let mut schema = CollectionSchemaBuilder::new(NAME, "hello world");
    let schema = schema
        .add_field(FieldSchema::new_int64("i64_field", ""))
        .add_field(FieldSchema::new_bool("bool_field", ""))
        .add_field(FieldSchema::new_float_vector("floate_vec", "", 128))
        .set_primary_key("i64_field")?
        .enable_auto_id()?
        .build()?;

    if client.has_collection(NAME).await? {
        client.drop_collection(NAME).await?;
    }

    let collection = client
        .create_collection(
            schema,
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Session,
            )),
        )
        .await?;

    assert!(client.has_collection(NAME).await?);

    client.drop_collection(NAME).await?;
    assert!(!client.has_collection(NAME).await?);

    Ok(())
}

#[tokio::test]
async fn create_alter_drop_alias() -> Result<()> {
    let alias0 = gen_random_name();
    let alias1 = gen_random_name();

    let client = Client::new(URL).await?;

    let (_, schema1) = create_test_collection(true).await?;
    let (_, schema2) = create_test_collection(true).await?;

    client.create_alias(schema1.name(), &alias0).await?;
    assert!(client.has_collection(alias0).await?);

    client.create_alias(schema2.name(), &alias1).await?;

    client.alter_alias(schema1.name(), &alias1).await?;

    client.drop_collection(schema2.name()).await?;
    assert!(client.has_collection(alias1).await?);

    Ok(())
}

#[tokio::test]
async fn test_alter_collection_field() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let collection_name = "test_alter_collection_field";

    // Create a collection with different field types
    let schema = CollectionSchemaBuilder::new(collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_varchar("varchar_field", "", 100))
        .add_field(FieldSchema::new_float_vector("vector_field", "", 128))
        .build()?;

    if client.has_collection(collection_name).await? {
        client.drop_collection(collection_name).await?;
    }
    // Create collection
    client.create_collection(schema, None).await.unwrap();

    // Test cases for different field types
    let test_cases = vec![
        // Test VARCHAR field - alter max_length
        (
            "varchar_field",
            HashMap::from([("max_length".to_string(), "200".to_string())]),
        ),
        // Test vector field - alter mmap_enabled
        (
            "vector_field",
            HashMap::from([("mmap_enabled".to_string(), "true".to_string())]),
        ),
    ];

    // Execute test cases
    for (field_name, params) in test_cases {
        let result = client
            .alter_collection_field(collection_name, field_name, params)
            .await;
        assert!(
            result.is_ok(),
            "Failed to alter field {}: {:?}",
            field_name,
            result
        );
    }

    // Cleanup
    client.drop_collection(collection_name).await.unwrap();
    Ok(())
}

#[tokio::test]
async fn test_alter_collection() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let collection_name = "test_alter_collection";

    // Create a collection with different field types
    let schema = CollectionSchemaBuilder::new(collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_varchar("varchar_field", "", 100))
        .add_field(FieldSchema::new_float_vector("vector_field", "", 128))
        .build()?;

    if client.has_collection(collection_name).await? {
        client.drop_collection(collection_name).await?;
    }

    // Create collection
    client.create_collection(schema, None).await?;
    let mut test_case = HashMap::new();
    test_case.insert("collection.ttl.second".to_string(), "10".to_string());
    test_case.insert("mmap.enabled".to_string(), "true".to_string());
    test_case.insert("partitionkey.isolation".to_string(), "false".to_string());

    // Alter collection properties
    assert!(client
        .alter_collection_properties(collection_name, test_case)
        .await
        .is_ok());

    // Cleanup
    client.drop_collection(collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn create_schema() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let mut schema = client.create_schema("test_schema").await?;
    let schema = schema
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("administrator", "", 32))
        .build()?;

    client
        .create_collection(
            schema,
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Session,
            )),
        )
        .await?;
    assert!(client.has_collection("test_schema").await?);
    client.drop_collection("test_schema").await?;
    Ok(())
}

#[tokio::test]
async fn test_drop_collection_properties() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let collection_name = "test_drop_collection_properties";
    let schema = CollectionSchemaBuilder::new(collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("administrator", "", 32))
        .build()?;
    client.create_collection(schema, None).await?;
    client
        .alter_collection_properties(
            collection_name,
            HashMap::from([("collection.ttl.second".to_string(), "10".to_string())]),
        )
        .await?;
    client
        .drop_collection_properties(collection_name, vec!["collection.ttl.second".to_string()])
        .await?;
    client.drop_collection(collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn rename_collection() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    let rename = format!("{}_{}", schema.name(), "rename");

    client
        .rename_collection(schema.name(), rename.as_str(), None)
        .await?;

    let collection = client.describe_collection(rename.clone()).await?;
    assert_eq!(collection.collection_name, rename);
    client.drop_collection(collection.collection_name).await?;
    Ok(())
}

