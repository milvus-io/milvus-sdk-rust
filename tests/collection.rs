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

use milvus::client::*;
use milvus::collection::{Collection, MetricType, SearchOption};
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use rand::Rng;
use std::collections::HashMap;

async fn create_test_collection(collection_name: &str) -> Result<Collection> {
    const URL: &str = "http://localhost:19530";

    let client = Client::new(URL).await?;
    let schema =
        CollectionSchemaBuilder::new(collection_name, "a guide example for milvus rust SDK")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(FieldSchema::new_float_vector("embed", "", 32))
            .build()?;
    if client.has_collection(collection_name).await? {
        client.drop_collection(collection_name).await?;
    }
    client
        .create_collection(schema.clone(), 2, ConsistencyLevel::Eventually)
        .await
}

async fn clean_test_collection(collection: Collection) -> Result<()> {
    collection.drop().await?;
    Ok(())
}

fn gen_random_f32_data(number: i32) -> Vec<f32> {
    let mut data = Vec::<f32>::new();
    let mut rng = rand::thread_rng();
    for _ in 0..number {
        data.push(rng.gen());
    }
    data
}

#[tokio::test]
async fn collection_basic() -> Result<()> {
    let collection = create_test_collection("collection_basic").await?;

    let embed_data = gen_random_f32_data(32 * 100);

    let embed_column =
        FieldColumn::new(collection.schema().get_field("embed").unwrap(), embed_data);

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;
    collection.load_blocked(1).await?;

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
    let collection = create_test_collection("collection_index").await?;

    let embed_data = gen_random_f32_data(32 * 100);

    let embed_column =
        FieldColumn::new(collection.schema().get_field("embed").unwrap(), embed_data);

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;

    let params = HashMap::from([
        ("index_type".to_string(), "IVF_FLAT".to_string()),
        ("metric_type".to_string(), "L2".to_string()),
        ("nlist".to_string(), 32.to_string()),
    ]);
    collection
        .create_index_blocked("embed", params.clone())
        .await?;
    let index_list = collection.describe_index("embed").await?;
    assert!(index_list.len() == 1, "{}", index_list.len());
    let index = &index_list[0];

    assert!(
        index.name == "_default_idx".to_string(),
        "index name is {}",
        index.name
    );

    assert!(
        index.params == params,
        "index params are {:?}",
        index.params
    );

    collection.drop_index("embed").await?;

    clean_test_collection(collection).await?;
    Ok(())
}

#[tokio::test]
async fn collection_search() -> Result<()> {
    let collection = create_test_collection("collection_search").await?;

    let mut embed_data = Vec::<f32>::new();
    for i in 0..32 {
        for j in 0..32 {
            if i == j {
                embed_data.push(1.0);
            } else {
                embed_data.push(0.0);
            }
        }
    }

    let embed_column =
        FieldColumn::new(collection.schema().get_field("embed").unwrap(), embed_data);

    collection.insert(vec![embed_column], None).await?;
    collection.flush().await?;
    let params = HashMap::from([
        ("index_type".to_string(), "IVF_FLAT".to_string()),
        ("metric_type".to_string(), "L2".to_string()),
        ("nlist".to_string(), 32.to_string()),
    ]);
    collection
        .create_index_blocked("embed", params.clone())
        .await?;
    collection.flush().await?;
    collection.load_blocked(1).await?;

    let mut search_vec = Vec::new();
    for i in 0..32 {
        if i == 15 {
            search_vec.push(1.0);
        } else {
            search_vec.push(0.0);
        }
    }

    let mut data: Vec<Value> = Vec::new();
    data.push(search_vec.clone().into());

    let result = collection
        .search(
            data,
            "embed",
            1,
            Vec::new(),
            MetricType::L2,
            vec!["id"],
            &SearchOption::new(),
        )
        .await?;

    assert!(result[0].size == 1);

    clean_test_collection(collection).await?;
    Ok(())
}
