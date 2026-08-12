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
    // ClientV2::new connects to `uri` and authenticates with `token`; the same client is reused
    // for the complete collection lifecycle.
    println!("Calling ClientV2::new: connect to Milvus");
    let client = ClientV2::new(&config).await?;
    println!("ClientV2::new completed");

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
    // create_collection creates the named collection. `schema` declares its fields, `index_param`
    // creates the vector index, and `consistency_level` sets the default read consistency.
    println!("Calling create_collection: create {collection:?}");
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
    println!("create_collection completed");

    // has_collection reports whether `collection_name` exists in the selected database.
    println!("Calling has_collection: check {collection:?}");
    let exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("has_collection completed");
    println!("Collection exists: {}", exists.exists());

    describe_collection(client, collection).await?;

    println!("\nSetting {TTL_SECONDS}=3600");
    // alter_collection_properties adds or replaces key/value settings on the collection. This
    // property asks Milvus to expire entities after 3,600 seconds.
    println!("Calling alter_collection_properties: set {TTL_SECONDS}=3600");
    client
        .alter_collection_properties(
            AlterCollectionPropertiesRequest::builder()
                .collection_name(collection)
                .property(TTL_SECONDS, "3600")
                .build()?,
        )
        .await?;
    println!("alter_collection_properties completed");
    describe_property(client, collection).await?;

    println!("Removing {TTL_SECONDS}");
    // drop_collection_properties removes the named property key without changing other settings.
    println!("Calling drop_collection_properties: remove {TTL_SECONDS}");
    client
        .drop_collection_properties(
            DropCollectionPropertiesRequest::builder()
                .collection_name(collection)
                .property_key(TTL_SECONDS)
                .build()?,
        )
        .await?;
    println!("drop_collection_properties completed");
    describe_property(client, collection).await?;

    println!("\nLoading the collection");
    // load_collection prepares collection data and indexes for queries. `sync(true)` waits for
    // readiness, and `timeout_ms` limits that wait to 60 seconds.
    println!("Calling load_collection: load {collection:?}");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection)
                .sync(true)
                .timeout_ms(60_000)
                .build()?,
        )
        .await?;
    println!("load_collection completed");
    // get_load_state returns the current load state and progress for `collection_name`.
    println!("Calling get_load_state: inspect {collection:?}");
    let load_state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("get_load_state completed");
    println!(
        "Load state: {:?}, progress={}%%",
        load_state.state(),
        load_state.progress()
    );

    // get_collection_stats returns server statistics such as `row_count` for this collection.
    println!("Calling get_collection_stats: inspect {collection:?}");
    let stats = client
        .get_collection_stats(
            GetCollectionStatsRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("get_collection_stats completed");
    println!(
        "Row count: {}",
        stats
            .statistics()
            .get("row_count")
            .map_or("<not returned>", String::as_str)
    );

    println!("\nReleasing the collection");
    // release_collection removes the collection from serving memory without deleting its data.
    println!("Calling release_collection: release {collection:?}");
    client
        .release_collection(
            ReleaseCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("release_collection completed");

    println!("Renaming {collection:?} to {renamed_collection:?}");
    // rename_collection moves the collection from `collection_name` to `new_collection_name`.
    println!("Calling rename_collection: rename to {renamed_collection:?}");
    client
        .rename_collection(
            RenameCollectionRequest::builder()
                .collection_name(collection)
                .new_collection_name(renamed_collection)
                .build()?,
        )
        .await?;
    println!("rename_collection completed");

    let old_exists = has_collection(client, collection).await?;
    let new_exists = has_collection(client, renamed_collection).await?;
    println!("After rename: old name exists={old_exists}, new name exists={new_exists}");

    println!("Truncating {renamed_collection:?}");
    // truncate_collection deletes all entities but preserves the schema and indexes.
    println!("Calling truncate_collection: remove all entities from {renamed_collection:?}");
    client
        .truncate_collection(
            TruncateCollectionRequest::builder()
                .collection_name(renamed_collection)
                .build()?,
        )
        .await?;
    println!("truncate_collection completed");
    println!("Truncate removes all entities but preserves the collection schema and indexes.");

    print_collections(client, "Collections before cleanup").await?;
    Ok(())
}

async fn describe_collection(client: &ClientV2, collection: &str) -> Result<()> {
    // describe_collection fetches schema and metadata for the selected collection name.
    println!("Calling describe_collection: inspect {collection:?}");
    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("describe_collection completed");
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
    // describe_collection also exposes the current collection property map.
    println!("Calling describe_collection: inspect properties for {collection:?}");
    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("describe_collection completed");
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
    // has_collection is used before cleanup so a missing resource is not dropped twice.
    println!("Calling has_collection: check {collection:?}");
    let exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?
        .exists();
    println!("has_collection completed");
    Ok(exists)
}

async fn cleanup_collections<'a>(
    client: &ClientV2,
    collections: impl IntoIterator<Item = &'a String>,
) -> Result<()> {
    for collection in collections {
        if !has_collection(client, collection).await? {
            continue;
        }
        // release_collection is best-effort cleanup; releasing an already released collection is
        // harmless and makes the later drop independent of serving state.
        println!("Calling release_collection: cleanup {collection:?}");
        let release_result = client
            .release_collection(
                ReleaseCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await;
        println!(
            "release_collection completed: {}",
            if release_result.is_ok() {
                "ok"
            } else {
                "ignored error"
            }
        );
        println!("\nDropping tutorial collection {collection:?}");
        // drop_collection permanently removes the named collection, including data and indexes.
        println!("Calling drop_collection: remove {collection:?}");
        client
            .drop_collection(
                DropCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await?;
        println!("drop_collection completed");
    }
    Ok(())
}

async fn print_collections(client: &ClientV2, heading: &str) -> Result<()> {
    // list_collections returns all collection names visible in the client's selected database.
    println!("Calling list_collections: list the selected database");
    let response = client
        .list_collections(ListCollectionsRequest::builder().build()?)
        .await?;
    println!("list_collections completed");
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
