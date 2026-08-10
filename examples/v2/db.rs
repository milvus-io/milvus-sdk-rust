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
use std::collections::HashMap;
use std::time::Duration;
use utils::*;

#[tokio::main]
async fn main() -> Result<()> {
    const DATABASE: &str = "rust_v2_db";
    const COLLECTION: &str = "RUST_V2_DB";
    const PARTITION: &str = "Year_2022";
    const PRIMARY: &str = "user_id";
    const NAME: &str = "user_name";
    const AGE: &str = "user_age";
    const FACE: &str = "user_face";
    const DIMENSION: usize = 128;

    let client = client().await?;
    let databases = client
        .list_databases(ListDatabasesRequest::builder().build()?)
        .await?;
    println!("Databases: {}", databases.database_names().join(","));
    if !databases
        .database_names()
        .iter()
        .any(|name| name == DATABASE)
    {
        client
            .create_database(
                CreateDatabaseRequest::builder()
                    .database_name(DATABASE)
                    .properties(HashMap::from([(
                        "database.replica.number".into(),
                        "2".into(),
                    )]))
                    .build()?,
            )
            .await?;
    }
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(DATABASE)
                .build()?,
        )
        .await?;
    println!(
        "database.replica.number = {}",
        description
            .properties()
            .get("database.replica.number")
            .map_or("", String::as_str)
    );
    client.use_database(DATABASE)?;
    println!("Current in-used database: {}", client.current_database());

    drop_collection(&client, COLLECTION).await;
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
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .build()?,
        )
        .await?;
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_params(vec![
                    IndexParam::new()
                        .field_name(FACE)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::Cosine),
                    IndexParam::new()
                        .field_name(NAME)
                        .index_type(IndexType::Trie),
                    IndexParam::new()
                        .field_name(AGE)
                        .index_type(IndexType::StlSort),
                ])
                .build()?,
        )
        .await?;
    client
        .create_partition(
            sdk::request::partition::CreatePartitionRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION)
                .build()?,
        )
        .await?;
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    let mut rng = rand::thread_rng();
    let vectors = (0..1000)
        .map(|_| float_vector(DIMENSION))
        .collect::<Vec<_>>();
    let ages = (0..1000)
        .map(|_| rng.gen_range(1..100) as i8)
        .collect::<Vec<_>>();
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .partition_name(PARTITION)
                .columns(vec![
                    FieldData::int64(PRIMARY, (0..1000).collect()),
                    FieldData::varchar(NAME, (0..1000).map(|id| format!("user_{id}")).collect()),
                    FieldData::int8(AGE, ages.clone()),
                    FieldData::float_vector(FACE, vectors.clone()),
                ])
                .build()?,
        )
        .await?;
    println!("{} rows inserted.", insert.insert_count());
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
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("partition count(*) = {}", query_count(count.results())?);

    client.use_database("default")?;
    println!("Current in-used database: {}", client.current_database());
    let query = client
        .query(
            QueryRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .partition_names([PARTITION])
                .filter("user_id in [5, 10]")
                .output_fields([PRIMARY, NAME, AGE])
                .consistency_level(sdk::ConsistencyLevel::Eventually)
                .build()?,
        )
        .await?;
    println!("Query with expression: user_id in [5, 10]");
    print_query_results(query.results())?;

    println!("Searching the No.100 and No.800 with expression: user_age > 40");
    let search = client
        .search(
            SearchRequest::builder()
                .database_name(DATABASE)
                .collection_name(COLLECTION)
                .partition_names([PARTITION])
                .vector_field(FACE)
                .vectors(SearchVectors::Float(vec![
                    vectors[100].clone(),
                    vectors[800].clone(),
                ]))
                .filter("user_age > 40")
                .output_fields([NAME, AGE])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(search.results())?;

    client.use_database(DATABASE)?;
    println!("Current in-used database: {}", client.current_database());
    client
        .release_collection(
            sdk::request::collection::ReleaseCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    client
        .drop_index(
            sdk::request::index::DropIndexRequest::builder()
                .collection_name(COLLECTION)
                .field_name(FACE)
                .build()?,
        )
        .await?;
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
    client.use_database("")?;
    println!("Current in-used database: {}", client.current_database());
    client
        .drop_database(
            DropDatabaseRequest::builder()
                .database_name(DATABASE)
                .build()?,
        )
        .await?;
    Ok(())
}
