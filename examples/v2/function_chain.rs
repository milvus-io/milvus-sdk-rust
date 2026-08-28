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

const COLLECTION: &str = "RUST_V2_FUNCTION_CHAIN";
const VECTOR: &str = "vector";
const YEAR: &str = "year";
const CATEGORY: &str = "category";
const DIMENSION: usize = 128;

/// Builds a rerank function chain: score recency with a decay function, blend the
/// decayed score with the vector score, round it, sort by it, and keep the top 10.
fn rerank_chain() -> FunctionChain {
    FunctionChain::new()
        .stage(FunctionChainStage::L2Rerank)
        .name("fresh_popular_rerank")
        .map(
            "freshness",
            fn_::decay(col(YEAR), "exp", 1980.0, 50.0, None, None),
        )
        .map(
            "$score",
            fn_::num_combine(
                vec![col("$score"), col("freshness")],
                "weighted",
                Some(vec![0.7, 0.3]),
            ),
        )
        .map("$score", fn_::round_decimal(col("$score"), 4))
        .sort("$score", true, Some("id".to_owned()))
        .limit(10, 0)
}

fn print_bucket(bucket: &AggregationBucket, depth: usize) {
    let indent = "  ".repeat(depth);
    let key = bucket
        .get_key()
        .iter()
        .map(|entry| format!("{}={:?}", entry.get_field_name(), entry.get_value()))
        .collect::<Vec<_>>()
        .join(", ");
    println!("{indent}{{key: [{key}] count: {}", bucket.get_count());
    for (alias, value) in bucket.get_metrics() {
        println!("{indent}  metric {alias} = {value:?}");
    }
    for hit in bucket.get_hits() {
        println!(
            "{indent}  hit pk={:?} score={} fields={:?}",
            hit.get_pk(),
            hit.get_score(),
            hit.get_fields()
        );
    }
    println!("{indent}}}");
    for sub in bucket.get_sub_groups() {
        print_bucket(sub, depth + 1);
    }
}

fn print_buckets(buckets: &[Vec<AggregationBucket>]) {
    for (query_index, groups) in buckets.iter().enumerate() {
        println!("Aggregation buckets for query vector {query_index}:");
        for bucket in groups {
            print_bucket(bucket, 1);
        }
    }
}

async fn search_with_chain(client: &sdk::ClientV2) -> Result<()> {
    println!("==================== Search with function chain ====================");
    let request = SearchRequest::builder()
        .collection_name(COLLECTION)
        .vector_field(VECTOR)
        .vectors(SearchVectors::Float(vec![vec![1.0; DIMENSION]]))
        .output_fields(["id", YEAR, CATEGORY])
        .limit(10)
        .consistency_level(sdk::ConsistencyLevel::Bounded)
        .function_chains([rerank_chain()])
        .build()?;
    let response = client.search(request).await?;
    print_search_results(response.results())?;
    Ok(())
}

async fn search_with_aggregation(client: &sdk::ClientV2) -> Result<()> {
    println!("==================== Search with aggregation ====================");
    let aggregation = SearchAggregation::new()
        .fields([CATEGORY])
        .size(5)
        .add_metric(
            "avg_year",
            MetricSpec::new().op(MetricOp::Avg).field_name(YEAR),
        )
        .add_metric(
            "count",
            MetricSpec::new().op(MetricOp::Count).field_name("*"),
        )
        .add_order(
            OrderSpec::new()
                .key("avg_year")
                .direction(SortDirection::Desc),
        )
        .top_hits(
            TopHitsSpec::new().size(3).sort([SortSpec::new()
                .field_name(YEAR)
                .direction(SortDirection::Desc)]),
        );
    let request = SearchRequest::builder()
        .collection_name(COLLECTION)
        .vector_field(VECTOR)
        .vectors(SearchVectors::Float(vec![vec![1.0; DIMENSION]]))
        .output_fields(["id", YEAR, CATEGORY])
        .limit(10)
        .consistency_level(sdk::ConsistencyLevel::Bounded)
        .search_aggregation(aggregation)
        .build()?;
    let response = client.search(request).await?;
    print_buckets(response.results().get_agg_buckets());
    Ok(())
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
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(CATEGORY)
                .data_type(sdk::DataType::VarChar)
                .max_length(32),
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
    let categories = ["electronics", "books", "clothing", "sports"];
    let rows: Vec<_> = (0..1000)
        .map(|id| {
            json!({
                "id": id,
                YEAR: id % 125 + 1900,
                CATEGORY: categories[id % categories.len()],
                VECTOR: float_vector(DIMENSION),
            })
        })
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

    search_with_chain(&client).await?;
    search_with_aggregation(&client).await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
