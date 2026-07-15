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
use serde_json::json;
use utils::*;

async fn search_group_by(
    client: &sdk::ClientV2,
    group_field: Option<&str>,
    limit: i64,
    group_size: i64,
    strict: bool,
) -> Result<()> {
    const COLLECTION: &str = "RUST_V2_GROUP_BY";
    let mut builder = SearchRequest::builder()
        .collection_name(COLLECTION)
        .vector_field("vector")
        .vectors(SearchVectors::Float(vec![vec![
            0.145292, 0.914725, 0.796505, 0.700925, 0.560520,
        ]]))
        .output_fields(["docId"])
        .limit(limit)
        .consistency_level(sdk::ConsistencyLevel::Session);
    if let Some(field) = group_field {
        builder = builder
            .group_by_field(field)
            .group_size(group_size)
            .strict_group_size(strict);
    }
    println!(
        "\nSearch with group by field: {}, group size: {group_size}, strict: {strict}, limit: {limit}",
        group_field.unwrap_or("null")
    );
    let response = client.search(builder.build()?).await?;
    print_search_results(response.results())
}

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_GROUP_BY";
    let client = client().await?;
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("vector")
                .data_type(sdk::DataType::FloatVector)
                .dimension(5),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("chunk")
                .data_type(sdk::DataType::VarChar)
                .max_length(128),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("docId")
                .data_type(sdk::DataType::Int32),
        );
    drop_collection(&client, COLLECTION).await;
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
                .index_param(
                    sdk::IndexParam::new()
                        .field_name("vector")
                        .index_type(sdk::IndexType::Flat)
                        .metric_type(sdk::MetricType::Cosine),
                )
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
    let rows: Vec<_> = vec![
        json!({"id":0,"vector":[0.358037,-0.602349,0.184140,-0.262862,0.902943],"chunk":"pink_8682","docId":1}),
        json!({"id":1,"vector":[0.198868,0.060235,0.697696,0.261447,0.838729],"chunk":"red_7025","docId":5}),
        json!({"id":2,"vector":[0.437421,-0.559750,0.645788,0.789405,0.207857],"chunk":"orange_6781","docId":2}),
        json!({"id":3,"vector":[0.317200,0.971904,-0.369811,0.120690,-0.144627],"chunk":"yellow_4222","docId":4}),
        json!({"id":4,"vector":[0.837197,-0.015764,-0.310629,-0.562666,-0.898494],"chunk":"red_9392","docId":1}),
        json!({"id":5,"vector":[-0.33445,-0.256713,0.898753,0.940299,0.537806],"chunk":"grey_8510","docId":2}),
        json!({"id":6,"vector":[0.395247,0.400025,-0.589050,-0.865050,-0.614036],"chunk":"white_9381","docId":5}),
        json!({"id":7,"vector":[0.571828,0.240703,-0.373791,-0.067269,-0.6980531],"chunk":"purple_4976","docId":3}),
    ];
    for row in rows {
        client
            .insert(
                sdk::request::dml::InsertRequest::builder()
                    .collection_name(COLLECTION)
                    .row(row)
                    .build()?,
            )
            .await?;
    }
    println!("8 rows inserted.");
    search_group_by(&client, None, 3, 1, false).await?;
    search_group_by(&client, Some("docId"), 3, 1, false).await?;
    search_group_by(&client, Some("docId"), 3, 2, false).await?;
    search_group_by(&client, Some("docId"), 3, 2, true).await?;
    search_group_by(&client, Some("docId"), 4, 3, false).await?;
    search_group_by(&client, Some("docId"), 4, 3, true).await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
