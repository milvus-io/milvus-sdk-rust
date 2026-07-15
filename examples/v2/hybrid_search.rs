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
use std::collections::BTreeMap;
use utils::*;

fn sparse_vector(dimension: u32) -> SparseVector {
    let mut rng = rand::thread_rng();
    let mut vector = BTreeMap::new();
    while vector.len() < 8 {
        vector.insert(rng.gen_range(0..dimension), rng.gen_range(0.0..1.0));
    }
    vector
}

#[tokio::main]
async fn main() -> Result<()> {
    const COLLECTION: &str = "RUST_V2_HYBRID_SEARCH";
    const DENSE: &str = "dense";
    const SPARSE: &str = "sparse";
    const DIMENSION: usize = 128;
    let client = client().await?;
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name("id")
                .description("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("flag")
                .description("flag")
                .data_type(sdk::DataType::Int16),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name("text")
                .description("text")
                .data_type(sdk::DataType::VarChar)
                .max_length(1024),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(DENSE)
                .description("dense vector")
                .data_type(sdk::DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(SPARSE)
                .description("sparse vector")
                .data_type(sdk::DataType::SparseFloatVector),
        );
    drop_collection(&client, COLLECTION).await;
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .index_params(vec![
                    IndexParam::new()
                        .field_name(DENSE)
                        .index_type(IndexType::DiskAnn)
                        .metric_type(MetricType::Cosine),
                    IndexParam::new()
                        .field_name(SPARSE)
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Ip),
                ])
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
    let rows: Vec<_> = (0..1000).map(|id| json!({"id":id,"flag":id%8+1,"text":format!("text_{id}"),DENSE:float_vector(DIMENSION),SPARSE:sparse_vector(50)})).collect();
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
    let dense = SubSearchRequest::builder()
        .vector_field(DENSE)
        .vectors(SearchVectors::Float(vec![float_vector(DIMENSION)]))
        .filter("flag == 5")
        .limit(5)
        .build()?;
    let sparse = SubSearchRequest::builder()
        .vector_field(SPARSE)
        .vectors(SearchVectors::SparseFloat(vec![sparse_vector(50)]))
        .filter("flag in [1, 3]")
        .limit(15)
        .build()?;
    let response = client
        .hybrid_search(
            HybridSearchRequest::builder()
                .collection_name(COLLECTION)
                .sub_requests(vec![dense, sparse])
                .rerank(WeightedRerank::new().weights(vec![0.5, 0.5]))
                .output_fields(["flag", "text"])
                .limit(10)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    print_search_results(response.results())?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
