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

// Lists the Search, HybridSearch and Query requests running in a cluster, and cancels one by id.
//
// The example starts a deliberately heavy search (many query vectors, brute-force FLAT index) in a
// background task, finds it with list_running_requests, cancels it with cancel_requests, and shows
// that the search's own client fails with server error code 3002 (request canceled).
//
// Requires a Milvus server that supports ListRunningRequests / CancelRequests (Milvus 3.0+). With
// authorization enabled, listing and canceling other users' requests requires the
// ListRunningRequests / CancelRequests privileges; any user may list and cancel their own.
// MILVUS_URI and MILVUS_TOKEN override the default connection settings; EXAMPLE_NQ controls how
// heavy the search is.

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::{Error, Result};
use milvus::v2::prelude::*;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use utils::*;

const COLLECTION: &str = "RUST_V2_RUNNING_REQUESTS";
const PRIMARY: &str = "id";
const VECTOR: &str = "vector";
const DIMENSION: usize = 128;
const ROW_COUNT: usize = 100_000;
const BATCH_SIZE: usize = 5_000;

fn nq() -> usize {
    std::env::var("EXAMPLE_NQ")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(5_000)
}

async fn prepare_collection(client: &ClientV2) -> Result<()> {
    drop_collection(client, COLLECTION).await;

    // A FLAT index makes every search a brute-force scan, so a search with many query vectors runs
    // long enough to be seen and canceled.
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(
                    sdk::CollectionSchema::new()
                        .add_field(
                            sdk::FieldSchema::new()
                                .name(PRIMARY)
                                .data_type(sdk::DataType::Int64)
                                .primary_key(true),
                        )
                        .add_field(
                            sdk::FieldSchema::new()
                                .name(VECTOR)
                                .data_type(sdk::DataType::FloatVector)
                                .dimension(DIMENSION as u32),
                        ),
                )
                .build()?,
        )
        .await?;
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_params(vec![IndexParam::new()
                    .field_name(VECTOR)
                    .index_type(IndexType::Flat)
                    .metric_type(MetricType::L2)])
                .build()?,
        )
        .await?;

    for start in (0..ROW_COUNT).step_by(BATCH_SIZE) {
        let rows: Vec<_> = (0..BATCH_SIZE)
            .map(|index| json!({PRIMARY: start + index, VECTOR: float_vector(DIMENSION)}))
            .collect();
        client
            .insert(
                sdk::request::dml::InsertRequest::builder()
                    .collection_name(COLLECTION)
                    .rows(rows)
                    .build()?,
            )
            .await?;
    }
    flush(client, COLLECTION).await?;
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!(
        "Collection '{COLLECTION}' has {} rows",
        query_count(count.results())?
    );
    Ok(())
}

fn print_node_results(node_results: &[RunningRequestNodeResult]) {
    for node in node_results {
        if node.is_success() {
            println!("  proxy {} answered", node.get_node_id());
        } else {
            println!(
                "  proxy {} did not answer (unimplemented={}): {}",
                node.get_node_id(),
                node.is_unimplemented(),
                node.get_reason()
            );
        }
    }
}

async fn list_all(client: &ClientV2) -> Result<()> {
    let response = client
        .list_running_requests(ListRunningRequestsRequest::builder().build()?)
        .await?;
    println!(
        "\n{} request(s) running in the cluster",
        response.requests().len()
    );
    print_node_results(response.node_results());
    Ok(())
}

async fn cancel_heavy_search(client: &ClientV2) -> Result<()> {
    let vectors: Vec<Vec<f32>> = (0..nq()).map(|_| float_vector(DIMENSION)).collect();
    let search_client = client.clone();
    let (done_tx, done_rx) = oneshot::channel();
    let search_finished = Arc::new(AtomicBool::new(false));
    let finished = Arc::clone(&search_finished);
    tokio::spawn(async move {
        let outcome = match search_client
            .search(
                SearchRequest::builder()
                    .collection_name(COLLECTION)
                    .vector_field(VECTOR)
                    .vectors(SearchVectors::Float(vectors))
                    .limit(100)
                    .build()
                    .expect("valid search request"),
            )
            .await
        {
            Ok(_) => "the search finished before it was canceled".to_owned(),
            Err(Error::Server(server_error)) => format!(
                "the search failed with server error code {}: {}",
                server_error.code(),
                server_error.reason()
            ),
            Err(error) => format!("the search failed: {error}"),
        };
        finished.store(true, Ordering::SeqCst);
        let _ = done_tx.send(outcome);
    });

    // Find the search by its collection, then cancel it by id.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut target = None;
    while target.is_none() && !search_finished.load(Ordering::SeqCst) && Instant::now() < deadline {
        let listed = client
            .list_running_requests(
                ListRunningRequestsRequest::builder()
                    .collection_name(COLLECTION)
                    .build()?,
            )
            .await?;
        for info in listed.requests() {
            if info.get_type() == "Search" {
                target = Some(info.clone());
                break;
            }
        }
        if target.is_none() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    let Some(target) = target else {
        println!("\nThe search was not seen running; raise EXAMPLE_NQ to make it slower");
        return Ok(());
    };
    println!(
        "\nFound the search: id={} user={} nq={} state={} elapsed={}ms",
        target.get_request_id(),
        target.get_user(),
        target.get_nq(),
        target.get_state(),
        target.get_elapsed_ms()
    );

    let canceled = client
        .cancel_requests(
            CancelRequestsRequest::builder()
                .request_ids([target.get_request_id()])
                .reason("canceled by RunningRequestsExample")
                .build()?,
        )
        .await?;
    for info in canceled.canceled() {
        println!(
            "Canceled {} after {}ms (state before the cancel: {})",
            info.get_request_id(),
            info.get_elapsed_ms(),
            info.get_state()
        );
    }
    println!("Not found: {:?}", canceled.not_found());
    println!("Undetermined: {:?}", canceled.undetermined());

    let outcome = done_rx.await;
    println!(
        "Search client: {}",
        outcome.unwrap_or_else(|_| "search task ended without an outcome".to_owned())
    );
    Ok(())
}

async fn cancel_unknown_id(client: &ClientV2) -> Result<()> {
    // An id that no proxy holds is reported in not_found once every proxy has answered.
    let response = client
        .cancel_requests(
            CancelRequestsRequest::builder()
                .request_ids(["0-0-1"])
                .build()?,
        )
        .await?;
    println!(
        "\nCancel an unknown id, not found: {:?}",
        response.not_found()
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    prepare_collection(&client).await?;
    list_all(&client).await?;
    cancel_heavy_search(&client).await?;
    cancel_unknown_id(&client).await?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
