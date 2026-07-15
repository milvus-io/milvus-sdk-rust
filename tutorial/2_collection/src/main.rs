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

use milvus::v2::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const TTL_SECONDS: &str = "collection.ttl.seconds";
const ID_FIELD: &str = "id";
const TITLE_FIELD: &str = "title";
const VECTOR_FIELD: &str = "embedding";

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".to_owned());
    let collection = tutorial_collection_name();
    let renamed_collection = format!("{collection}_renamed");

    let config = ConnectConfig::new().uri(uri).token(token);
    let client = ClientV2::new(&config).await?;

    print_collections(&client, "Collections before the tutorial").await?;

    // Capture the tutorial result so cleanup still runs if a later operation fails.
    let tutorial_result =
        demonstrate_collection_interfaces(&client, &collection, &renamed_collection).await;
    let cleanup_result = cleanup_collections(&client, [&collection, &renamed_collection]).await;

    if let Err(error) = &cleanup_result {
        eprintln!("Failed to clean up the tutorial collection: {error}");
    }
    tutorial_result?;
    cleanup_result?;

    print_collections(&client, "Collections after cleanup").await?;
    Ok(())
}

async fn demonstrate_collection_interfaces(
    client: &ClientV2,
    collection: &str,
    renamed_collection: &str,
) -> Result<()> {
    let schema = CollectionSchema::new()
        .description("Collection tutorial schema")
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name(TITLE_FIELD)
                .data_type(DataType::VarChar)
                .max_length(256),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR_FIELD)
                .data_type(DataType::FloatVector)
                .dimension(4),
        );
    let vector_index = IndexParam::new()
        .field_name(VECTOR_FIELD)
        .index_type(IndexType::AutoIndex)
        .metric_type(MetricType::Cosine);

    println!("\nCreating collection {collection:?}");
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection)
                .description("Collection created by tutorial/2_collection")
                .schema(schema)
                .index_param(vector_index)
                .consistency_level(ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;

    let exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("Collection exists: {}", exists.exists());

    describe_collection(client, collection).await?;

    println!("\nSetting {TTL_SECONDS}=3600");
    client
        .alter_collection_properties(
            AlterCollectionPropertiesRequest::builder()
                .collection_name(collection)
                .property(TTL_SECONDS, "3600")
                .build()?,
        )
        .await?;
    describe_property(client, collection).await?;

    println!("Removing {TTL_SECONDS}");
    client
        .drop_collection_properties(
            DropCollectionPropertiesRequest::builder()
                .collection_name(collection)
                .property_key(TTL_SECONDS)
                .build()?,
        )
        .await?;
    describe_property(client, collection).await?;

    println!("\nLoading the collection");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection)
                .sync(true)
                .timeout_ms(60_000)
                .build()?,
        )
        .await?;
    let load_state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!(
        "Load state: {:?}, progress={}%%",
        load_state.state(),
        load_state.progress()
    );

    let stats = client
        .get_collection_stats(
            GetCollectionStatsRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!(
        "Row count: {}",
        stats
            .statistics()
            .get("row_count")
            .map_or("<not returned>", String::as_str)
    );

    println!("\nReleasing the collection");
    client
        .release_collection(
            ReleaseCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;

    println!("Renaming {collection:?} to {renamed_collection:?}");
    client
        .rename_collection(
            RenameCollectionRequest::builder()
                .collection_name(collection)
                .new_collection_name(renamed_collection)
                .build()?,
        )
        .await?;

    let old_exists = has_collection(client, collection).await?;
    let new_exists = has_collection(client, renamed_collection).await?;
    println!("After rename: old name exists={old_exists}, new name exists={new_exists}");

    println!("Truncating {renamed_collection:?}");
    client
        .truncate_collection(
            TruncateCollectionRequest::builder()
                .collection_name(renamed_collection)
                .build()?,
        )
        .await?;
    println!("Truncate removes all entities but preserves the collection schema and indexes.");

    print_collections(client, "Collections before cleanup").await?;
    Ok(())
}

async fn describe_collection(client: &ClientV2, collection: &str) -> Result<()> {
    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    let description = response.description();
    println!(
        "Description: name={:?}, id={}, primary_field={:?}, vector_fields={:?}",
        description.get_collection_name(),
        description.get_collection_id(),
        description.get_primary_field_name(),
        description.get_vector_field_names()
    );
    Ok(())
}

async fn describe_property(client: &ClientV2, collection: &str) -> Result<()> {
    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!(
        "Property value: {}",
        response
            .description()
            .get_properties()
            .get(TTL_SECONDS)
            .map_or("<not set>", String::as_str)
    );
    Ok(())
}

async fn has_collection(client: &ClientV2, collection: &str) -> Result<bool> {
    Ok(client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?
        .exists())
}

async fn cleanup_collections<'a>(
    client: &ClientV2,
    collections: impl IntoIterator<Item = &'a String>,
) -> Result<()> {
    for collection in collections {
        if !has_collection(client, collection).await? {
            continue;
        }
        // Releasing an already released collection is harmless for tutorial cleanup.
        let _ = client
            .release_collection(
                ReleaseCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await;
        println!("\nDropping tutorial collection {collection:?}");
        client
            .drop_collection(
                DropCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await?;
    }
    Ok(())
}

async fn print_collections(client: &ClientV2, heading: &str) -> Result<()> {
    let response = client
        .list_collections(ListCollectionsRequest::builder().build()?)
        .await?;
    let names = if response.collection_names().is_empty() {
        "<none>".to_owned()
    } else {
        response.collection_names().join(", ")
    };
    println!("{heading}: {names}");
    Ok(())
}

fn tutorial_collection_name() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(
        "rust_sdk_collection_tutorial_{timestamp}_{}",
        std::process::id()
    )
}
