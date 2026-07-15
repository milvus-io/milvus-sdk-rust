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

use milvus::error::Result;

use super::common::*;

fn partition_test_name() -> String {
    "test_partition".to_string()
}

async fn create_partition_only_collection(
) -> Result<(milvus::client::Client, milvus::schema::CollectionSchema)> {
    create_empty_test_collection(true).await
}

async fn create_loaded_partition_collection(
) -> Result<(milvus::client::Client, milvus::schema::CollectionSchema)> {
    create_test_collection(true).await
}

async fn create_partition_and_verify(
    client: &milvus::client::Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<()> {
    client
        .create_partition(collection_name.to_string(), partition_name.to_string())
        .await?;
    Ok(())
}

async fn create_partition_with_retry(
    client: &milvus::client::Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<()> {
    create_partition_and_verify(client, collection_name, partition_name).await?;
    for _ in 0..20 {
        if client
            .has_partition(collection_name.to_string(), partition_name.to_string())
            .await
            .unwrap_or(false)
        {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    create_partition_and_verify(client, collection_name, partition_name).await
}

async fn wait_for_partition_visible(
    client: &milvus::client::Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<()> {
    for _ in 0..20 {
        if client
            .has_partition(collection_name.to_string(), partition_name.to_string())
            .await?
        {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err(milvus::error::Error::Unexpected(format!(
        "partition {partition_name} not visible in {collection_name}"
    )))
}

async fn partition_stats_row_count(
    client: &milvus::client::Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<std::collections::HashMap<String, String>> {
    client
        .get_partition_stats(collection_name.to_string(), partition_name.to_string())
        .await
}

async fn wait_for_partition_stats(
    client: &milvus::client::Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<std::collections::HashMap<String, String>> {
    for _ in 0..20 {
        let stats = partition_stats_row_count(client, collection_name, partition_name).await?;
        if stats.contains_key("row_count") {
            return Ok(stats);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    partition_stats_row_count(client, collection_name, partition_name).await
}

async fn wait_for_partition_listed(
    client: &milvus::client::Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<Vec<String>> {
    for _ in 0..20 {
        let partitions = client.list_partitions(collection_name.to_string()).await?;
        if partitions.iter().any(|name| name == partition_name) {
            return Ok(partitions);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    client.list_partitions(collection_name.to_string()).await
}

async fn with_partition_test_collection<F, Fut>(body: F) -> Result<()>
where
    F: FnOnce(milvus::client::Client, milvus::schema::CollectionSchema) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let (client, schema) = create_partition_only_collection().await?;
    let cleanup_client = client.clone();
    let collection_name = schema.name().to_string();
    run_with_collection_cleanup(&cleanup_client, vec![collection_name], || {
        body(client, schema)
    })
    .await
}

async fn with_loaded_partition_test_collection<F, Fut>(body: F) -> Result<()>
where
    F: FnOnce(milvus::client::Client, milvus::schema::CollectionSchema) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let (client, schema) = create_loaded_partition_collection().await?;
    let cleanup_client = client.clone();
    let collection_name = schema.name().to_string();
    run_with_collection_cleanup(&cleanup_client, vec![collection_name], || {
        body(client, schema)
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn load_release_partitions() -> Result<()> {
    with_loaded_partition_test_collection(|client, schema| async move {
        create_partition_with_retry(&client, schema.name(), "partition_A").await?;

        client.release_collection(schema.name()).await?;

        client
            .load_partitions(schema.name(), vec!["partition_A"], 0, None)
            .await?;

        let mut status = client.get_load_state(schema.name(), None).await?;

        assert_eq!(status, milvus::proto::common::LoadState::Loaded);

        status = client
            .get_load_state(
                schema.name(),
                Some(milvus::options::GetLoadStateOptions::with_partition_names(
                    vec!["partition_A".to_string()],
                )),
            )
            .await?;
        assert_eq!(status, milvus::proto::common::LoadState::Loaded);

        client
            .release_partitions(schema.name(), vec!["partition_A"])
            .await?;
        status = client.get_load_state(schema.name(), None).await?;
        assert_eq!(status, milvus::proto::common::LoadState::NotLoad);

        Ok(())
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_partition() -> Result<()> {
    with_partition_test_collection(|client, collection| async move {
        let partition_name = partition_test_name();
        create_partition_with_retry(&client, collection.name(), &partition_name).await
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn test_drop_partition() -> Result<()> {
    with_partition_test_collection(|client, collection| async move {
        let partition_name = partition_test_name();
        create_partition_with_retry(&client, collection.name(), &partition_name).await?;
        client.release_collection(collection.name()).await?;
        client
            .drop_partition(collection.name().to_string(), partition_name)
            .await?;

        Ok(())
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_partitions() -> Result<()> {
    with_partition_test_collection(|client, collection| async move {
        let partition_name = partition_test_name();
        create_partition_with_retry(&client, collection.name(), &partition_name).await?;

        let partitions =
            wait_for_partition_listed(&client, collection.name(), &partition_name).await?;
        assert!(partitions.contains(&partition_name));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn test_has_partition() -> Result<()> {
    with_partition_test_collection(|client, collection| async move {
        let partition_name = partition_test_name();
        create_partition_with_retry(&client, collection.name(), &partition_name).await?;
        wait_for_partition_visible(&client, collection.name(), &partition_name).await?;
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_partition_stats() -> Result<()> {
    with_partition_test_collection(|client, collection| async move {
        let partition_name = partition_test_name();
        create_partition_with_retry(&client, collection.name(), &partition_name).await?;

        let stats = wait_for_partition_stats(&client, collection.name(), &partition_name).await?;
        assert!(stats.contains_key("row_count"));
        Ok(())
    })
    .await
}
