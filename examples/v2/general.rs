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

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use rand::Rng;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use utils::*;

const DATABASE: &str = "rust_v2_general_db";
const COLLECTION: &str = "RUST_V2_GENERAL";
const PARTITION: &str = "Year_2022";
const PRIMARY: &str = "user_id";
const NAME: &str = "user_name";
const AGE: &str = "user_age";
const FACE: &str = "user_face";
const DIMENSION: usize = 128;

async fn describe_collection(client: &ClientV2) -> Result<()> {
    let description = client
        .describe_collection(
            sdk::request::collection::DescribeCollectionRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Collection: {:?}", description.description());
    let load = client
        .get_load_state(
            sdk::request::collection::GetLoadStateRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Collection load state: {:?}", load.state());
    Ok(())
}

async fn describe_index(client: &ClientV2, index_name: &str) -> Result<()> {
    let response = client
        .describe_index(
            sdk::request::index::DescribeIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_name(index_name)
                .build()?,
        )
        .await?;
    for index in response.indexes() {
        println!("Index: {index:?}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let health = client
        .check_health(sdk::request::utility::CheckHealthRequest::builder().build()?)
        .await?;
    if health.is_healthy() {
        println!("The milvus server is healthy");
    } else {
        println!(
            "The milvus server is unhealthy, reasons: {:?}, quota states: {:?}",
            health.reasons(),
            health.quota_states()
        );
    }
    client.set_rpc_deadline(Duration::from_secs(10));
    println!(
        "The milvus server version is: {}",
        client
            .server_version(sdk::request::utility::GetServerVersionRequest::builder().build()?,)
            .await?
            .version()
    );
    println!("The Rust SDK version is: {}", client.sdk_version());

    let databases = client
        .list_databases(sdk::request::database::ListDatabasesRequest::builder().build()?)
        .await?;
    let created_database = !databases
        .database_names()
        .iter()
        .any(|name| name == DATABASE);
    if created_database {
        client
            .create_database(
                sdk::request::database::CreateDatabaseRequest::builder()
                    .database_name(DATABASE)
                    .build()?,
            )
            .await?;
    }
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name(PRIMARY)
                .description("user id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(NAME)
                .description("user name")
                .data_type(sdk::DataType::VarChar)
                .max_length(100),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(AGE)
                .description("user age")
                .data_type(sdk::DataType::Int8),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(FACE)
                .description("face signature")
                .data_type(sdk::DataType::FloatVector)
                .dimension(DIMENSION as u32),
        );
    let _ = client
        .drop_collection(
            sdk::request::collection::DropCollectionRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .build()?,
        )
        .await;
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .description("my collection")
                .num_shards(1)
                .schema(schema)
                .index_params(vec![
                    IndexParam::new()
                        .field_name(FACE)
                        .index_type(IndexType::IvfFlat)
                        .metric_type(MetricType::Cosine)
                        .extra_params(HashMap::from([("nlist".into(), "100".into())])),
                    IndexParam::new()
                        .field_name(AGE)
                        .index_type(IndexType::StlSort),
                    IndexParam::new()
                        .field_name(NAME)
                        .index_type(IndexType::Trie),
                ])
                .properties(HashMap::from([
                    ("my_prop".into(), "dummy".into()),
                    ("collection.ttl.seconds".into(), "60".into()),
                ]))
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    describe_collection(&client).await?;
    client
        .alter_collection_properties(
            sdk::request::collection::AlterCollectionPropertiesRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .properties(HashMap::from([(
                    "collection.ttl.seconds".into(),
                    "20".into(),
                )]))
                .build()?,
        )
        .await?;
    describe_collection(&client).await?;
    client
        .drop_collection_properties(
            sdk::request::collection::DropCollectionPropertiesRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .property_keys(["collection.ttl.seconds"])
                .build()?,
        )
        .await?;
    describe_collection(&client).await?;
    client
        .create_partition(
            sdk::request::partition::CreatePartitionRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .partition_name(PARTITION)
                .build()?,
        )
        .await?;
    let collections = client
        .list_collections(
            sdk::request::collection::ListCollectionsRequest::builder()
                .database_name(DATABASE)
                .build()?,
        )
        .await?;
    println!("\nCollections: {:?}", collections.collection_names());
    let partitions = client
        .list_partitions(
            sdk::request::partition::ListPartitionsRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!(
        "\nPartitions of {COLLECTION}: {:?}",
        partitions.partition_names()
    );
    client.use_database(DATABASE)?;

    let mut rng = rand::thread_rng();
    let ids = (0..2000).collect::<Vec<i64>>();
    let names = ids
        .iter()
        .map(|id| format!("user_{id}"))
        .collect::<Vec<_>>();
    let ages = ids
        .iter()
        .map(|_| rng.gen_range(1..100) as i8)
        .collect::<Vec<_>>();
    let vectors = ids
        .iter()
        .map(|_| float_vector(DIMENSION))
        .collect::<Vec<_>>();
    let inserted = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION)
                .columns(vec![
                    FieldData::int64(PRIMARY, ids[..1000].to_vec()),
                    FieldData::varchar(NAME, names[..1000].to_vec()),
                    FieldData::int8(AGE, ages[..1000].to_vec()),
                    FieldData::float_vector(FACE, vectors[..1000].to_vec()),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", inserted.insert_count());
    for chunk in (1000..2000).collect::<Vec<_>>().chunks(300) {
        let rows: Vec<_> = chunk.iter().map(|index| json!({PRIMARY:ids[*index],NAME:names[*index],AGE:ages[*index],FACE:vectors[*index]})).collect();
        let inserted = client
            .insert(
                InsertRequest::builder()
                    .collection_name(COLLECTION)
                    .partition_name(PARTITION)
                    .rows(rows)
                    .build()?,
            )
            .await?;
        println!("{} rows inserted by row-based.", inserted.insert_count());
    }
    client
        .delete(
            DeleteRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION)
                .filter("user_id == 5")
                .build()?,
        )
        .await?;
    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .partition_names([PARTITION])
                .output_fields(["count(*)"])
                .build()?,
        )
        .await?;
    println!("partition count(*) = {}", query_count(count.results())?);
    flush(&client, COLLECTION).await?;
    describe_index(&client, FACE).await?;
    println!("\nQuery with expression: user_id in [1, 5, 10]");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .partition_names([PARTITION])
                .filter("user_id in [1, 5, 10]")
                .output_fields([PRIMARY, NAME, AGE])
                .consistency_level(sdk::ConsistencyLevel::Eventually)
                .build()?,
        )
        .await?;
    print_query_results(query.results())?;
    println!("\nSearching the No.100 and No.1800 with expression: user_age > 40");
    let search = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .partition_names([PARTITION])
                .vector_field(FACE)
                .vectors(SearchVectors::Float(vec![
                    vectors[100].clone(),
                    vectors[1800].clone(),
                ]))
                .filter("user_age > 40")
                .output_fields([NAME, AGE])
                .extra_params(HashMap::from([("nprobe".into(), "10".into())]))
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;
    describe_collection(&client).await?;
    client
        .release_collection(
            sdk::request::collection::ReleaseCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    describe_collection(&client).await?;
    client
        .drop_index(
            sdk::request::index::DropIndexRequest::builder()
                .collection_name(COLLECTION)
                .field_name(FACE)
                .build()?,
        )
        .await?;
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    IndexParam::new()
                        .field_name(FACE)
                        .index_name("vector_index_name")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::L2)
                        .extra_params(HashMap::from([
                            ("M".into(), "32".into()),
                            ("efConstruction".into(), "100".into()),
                        ])),
                )
                .sync(true)
                .build()?,
        )
        .await?;
    client
        .alter_index_properties(
            sdk::request::index::AlterIndexPropertiesRequest::builder()
                .collection_name(COLLECTION)
                .index_name("vector_index_name")
                .properties(HashMap::from([("mmap.enabled".into(), "true".into())]))
                .build()?,
        )
        .await?;
    describe_index(&client, "vector_index_name").await?;
    client
        .drop_index_properties(
            sdk::request::index::DropIndexPropertiesRequest::builder()
                .collection_name(COLLECTION)
                .index_name("vector_index_name")
                .property_keys(["mmap.enabled"])
                .build()?,
        )
        .await?;
    describe_index(&client, "vector_index_name").await?;
    client
        .drop_partition(
            sdk::request::partition::DropPartitionRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION)
                .build()?,
        )
        .await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    let stats = client
        .get_collection_stats(
            sdk::request::collection::GetCollectionStatsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!(
        "Collection {COLLECTION} row count: {}",
        stats
            .statistics()
            .get("row_count")
            .map_or("0", String::as_str)
    );
    drop_collection(&client, COLLECTION).await;
    client.use_database("default")?;
    if created_database {
        client
            .drop_database(
                sdk::request::database::DropDatabaseRequest::builder()
                    .database_name(DATABASE)
                    .build()?,
            )
            .await?;
    }
    Ok(())
}
