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

use futures::FutureExt;
use milvus::client::*;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::IndexType;
use milvus::options::CreateCollectionOptions;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use rand::Rng;
use std::future::Future;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Duration;

pub struct CollectionCleanup {
    collections: Vec<(String, String)>,
    armed: bool,
}

impl CollectionCleanup {
    pub fn new(collection_names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::in_database("default", collection_names)
    }

    pub fn in_database(
        database_name: impl Into<String>,
        collection_names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let database_name = database_name.into();
        Self {
            collections: collection_names
                .into_iter()
                .map(|name| (database_name.clone(), name.into()))
                .collect(),
            armed: true,
        }
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for CollectionCleanup {
    fn drop(&mut self) {
        if !self.armed || self.collections.is_empty() {
            return;
        }
        let collections = std::mem::take(&mut self.collections);
        let cleanup = std::thread::Builder::new().spawn(move || {
            let Ok(runtime) = tokio::runtime::Runtime::new() else {
                return;
            };
            runtime.block_on(async move {
                let Ok(mut client) = ClientBuilder::new(URL)
                    .timeout(Duration::from_secs(60))
                    .build()
                    .await
                else {
                    return;
                };
                for (database_name, collection_name) in collections.into_iter().rev() {
                    let _ = client.using_database(&database_name).await;
                    if let Ok((_, _, aliases)) = client.list_aliases(&collection_name).await {
                        for alias in aliases {
                            let _ = client.drop_alias(alias).await;
                        }
                    }
                    let _ = client.drop_collection(&collection_name).await;
                }
            });
        });
        if let Ok(cleanup) = cleanup {
            let _ = cleanup.join();
        }
    }
}

pub const DEFAULT_DIM: i64 = 128;
pub const DEFAULT_VEC_FIELD: &str = "feature";
pub const DEFAULT_INDEX_NAME: &str = "feature_index";
pub const URL: &str = "http://localhost:29830";
pub const ENTITYNUM: i64 = 1000;

pub async fn run_with_collection_cleanup<F, Fut>(
    client: &Client,
    collection_names: Vec<String>,
    body: F,
) -> Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let mut fallback_cleanup = CollectionCleanup::new(collection_names.clone());
    let body_result = match catch_unwind(AssertUnwindSafe(body)) {
        Ok(future) => AssertUnwindSafe(future).catch_unwind().await,
        Err(payload) => Err(payload),
    };
    let mut cleanup_error = None;

    for collection_name in collection_names.iter().rev() {
        match client.has_collection(collection_name).await {
            Ok(true) => {
                if let Ok((_, _, aliases)) = client.list_aliases(collection_name).await {
                    for alias in aliases {
                        if let Err(error) = client.drop_alias(alias).await {
                            if cleanup_error.is_none() {
                                cleanup_error = Some(error);
                            }
                        }
                    }
                }

                if let Err(error) = client.drop_collection(collection_name).await {
                    if cleanup_error.is_none() {
                        cleanup_error = Some(error);
                    }
                }
            }
            Ok(false) => {}
            Err(error) => {
                if cleanup_error.is_none() {
                    cleanup_error = Some(error);
                }
            }
        }
    }

    if cleanup_error.is_none() {
        fallback_cleanup.disarm();
    }

    match body_result {
        Ok(Ok(())) => cleanup_error.map_or(Ok(()), Err),
        Ok(Err(error)) => {
            if let Some(cleanup_error) = cleanup_error {
                eprintln!("collection cleanup failed after test error: {cleanup_error}");
            }
            Err(error)
        }
        Err(payload) => {
            if let Some(cleanup_error) = cleanup_error {
                eprintln!("collection cleanup failed after panic: {cleanup_error}");
            }
            resume_unwind(payload)
        }
    }
}

pub async fn create_test_collection(autoid: bool) -> Result<(Client, CollectionSchema)> {
    create_test_collection_with_data(autoid, ENTITYNUM).await
}

pub async fn create_empty_test_collection(autoid: bool) -> Result<(Client, CollectionSchema)> {
    create_empty_test_collection_custom(autoid, DEFAULT_DIM, DEFAULT_VEC_FIELD).await
}

pub async fn create_empty_test_collection_custom(
    autoid: bool,
    dimension: i64,
    vector_field_name: &str,
) -> Result<(Client, CollectionSchema)> {
    let collection_name = gen_random_name();
    let collection_name = format!("{}_{}", "test_collection", collection_name);
    let client = Client::new(URL).await?;
    let mut cleanup = CollectionCleanup::new([&collection_name]);
    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", autoid))
        .add_field(FieldSchema::new_float_vector(
            vector_field_name,
            "",
            dimension,
        ))
        .build()?;
    if client.has_collection(&collection_name).await? {
        client.drop_collection(&collection_name).await?;
    }
    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Eventually,
            )),
        )
        .await?;
    cleanup.disarm();
    Ok((client, schema))
}

pub async fn create_test_collection_with_data(
    autoid: bool,
    entity_count: i64,
) -> Result<(Client, CollectionSchema)> {
    create_test_collection_custom(autoid, entity_count, DEFAULT_DIM, DEFAULT_VEC_FIELD).await
}

pub async fn create_test_collection_custom(
    autoid: bool,
    entity_count: i64,
    dimension: i64,
    vector_field_name: &str,
) -> Result<(Client, CollectionSchema)> {
    let (client, schema) =
        create_empty_test_collection_custom(autoid, dimension, vector_field_name).await?;
    let mut cleanup = CollectionCleanup::new([schema.name()]);

    let feature_data = gen_random_f32_vector_custom(entity_count, dimension);
    let feature_column =
        FieldColumn::new(schema.get_field(vector_field_name).unwrap(), feature_data);

    let columns = if autoid {
        vec![feature_column]
    } else {
        let id_data = gen_random_int64_vector(entity_count);
        let id_column = FieldColumn::new(schema.get_field("id").unwrap(), id_data);
        vec![id_column, feature_column]
    };

    let setup = async {
        client.insert(schema.name(), columns, None).await?;
        client.flush(schema.name()).await?;

        client
            .create_index(
                schema.name(),
                vector_field_name,
                milvus::index::IndexParams::new(
                    DEFAULT_INDEX_NAME.to_string(),
                    IndexType::IvfFlat,
                    milvus::index::MetricType::L2,
                    std::collections::HashMap::new(),
                ),
            )
            .await?;

        client.load_collection(schema.name(), None).await
    }
    .await;
    setup?;
    cleanup.disarm();
    Ok((client, schema))
}

pub fn gen_random_name() -> String {
    format!(
        "r{}",
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect::<String>(),
    )
}

pub fn gen_random_int64_vector(n: i64) -> Vec<i64> {
    let mut data: Vec<i64> = Vec::with_capacity(n as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        data.push(rng.gen());
    }
    data
}

pub fn gen_random_f32_vector_custom(n: i64, dimension: i64) -> Vec<f32> {
    let mut data = Vec::<f32>::with_capacity((n * dimension) as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..n * dimension {
        data.push(rng.gen());
    }
    data
}
