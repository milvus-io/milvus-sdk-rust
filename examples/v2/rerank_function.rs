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

const COLLECTION: &str = "RUST_V2_RERANK_FUNCTION";
const VECTOR: &str = "vector";
const YEAR: &str = "year";
const DIMENSION: usize = 128;

fn decay(name: &str, function: &str, origin: i32) -> DecayRerank {
    let rerank = DecayRerank::new()
        .name(name)
        .decay_function(function)
        .origin(origin)
        .offset(20)
        .scale(50)
        .decay(0.5);
    let value = rerank.get_function().clone().input_fields([YEAR]);
    rerank.function(value)
}

async fn search(client: &sdk::ClientV2, rerank: Option<FunctionScore>, topk: i64) -> Result<()> {
    println!(
        "==================== Search {} function score ====================",
        if rerank.is_some() { "with" } else { "without" }
    );
    let mut builder = SearchRequest::builder()
        .collection_name(COLLECTION)
        .vector_field(VECTOR)
        .vectors(SearchVectors::Float(vec![vec![1.0; DIMENSION]]))
        .output_fields(["id", YEAR])
        .limit(topk)
        .consistency_level(sdk::ConsistencyLevel::Bounded);
    if let Some(rerank) = rerank {
        builder = builder.rerank(rerank);
    }
    let response = client.search(builder.build()?).await?;
    print_search_results(response.results())
}

#[tokio::main]
async fn main() -> Result<()> {
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
                .name(VECTOR)
                .data_type(sdk::DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(YEAR)
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
                        .field_name(VECTOR)
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
    let rows: Vec<_> = (0..1000)
        .map(|id| json!({"id":id,YEAR:id%125+1900,VECTOR:float_vector(DIMENSION)}))
        .collect();
    let insert = client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted", insert.insert_count());
    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("count(*) = {}", query_count(count.results())?);

    search(&client, None, 10).await?;
    let boost = BoostRerank::new()
        .name("boost on year")
        .filter("year >= 2000")
        .weight(5.0);
    let gauss = decay("gauss decay on year", "gauss", 1980);
    let exponential = decay("exponential decay on year", "exp", 1950);
    let linear = decay("linear decay on year", "linear", 1930);
    search(
        &client,
        Some(FunctionScore::new().add_function(boost.clone())),
        20,
    )
    .await?;
    search(
        &client,
        Some(FunctionScore::new().add_function(gauss.clone())),
        20,
    )
    .await?;
    search(
        &client,
        Some(FunctionScore::new().add_function(exponential)),
        20,
    )
    .await?;
    search(&client, Some(FunctionScore::new().add_function(linear)), 20).await?;
    search(
        &client,
        Some(FunctionScore::new().add_function(gauss).add_function(boost)),
        20,
    )
    .await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
