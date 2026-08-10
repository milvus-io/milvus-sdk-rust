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
use std::collections::HashMap;
use std::time::{Duration, Instant};
use utils::*;

const COLLECTION: &str = "RUST_V2_OPTIMIZE";
const PRIMARY: &str = "id";
const VECTOR: &str = "vector";
const DIMENSION: usize = 512;
const TOTAL_ROWS: usize = 1_000_000;
const BATCH_SIZE: usize = 10_000;
const SEGMENT_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(120);
const SEGMENT_POLL_INTERVAL: Duration = Duration::from_secs(1);

async fn print_segment_info(client: &ClientV2) -> Result<usize> {
    let response = client
        .list_query_segments(
            ListQuerySegmentsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("  Total segments: {}", response.segments().len());
    let mut total_rows = 0;
    for segment in response.segments() {
        println!(
            "    Segment {}: rows={}, state={:?}, index={}",
            segment.get_segment_id(),
            segment.get_row_count(),
            segment.get_state(),
            segment.get_index_name()
        );
        total_rows += segment.get_row_count();
    }
    println!("  Total rows across segments: {total_rows}");
    Ok(response.segments().len())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    println!("========== Step 1: Create collection ==========");
    drop_collection(&client, COLLECTION).await;
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name(PRIMARY)
                .data_type(sdk::DataType::Int64)
                .primary_key(true)
                .auto_id(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(VECTOR)
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

    println!("========== Step 2: Insert 1,000,000 rows ==========");
    let mut total_inserted = 0;
    for batch in 0..TOTAL_ROWS / BATCH_SIZE {
        let rows: Vec<_> = (0..BATCH_SIZE)
            .map(|_| json!({VECTOR: float_vector(DIMENSION)}))
            .collect();
        let insert = client
            .insert(
                sdk::request::dml::InsertRequest::builder()
                    .collection_name(COLLECTION)
                    .rows(rows)
                    .build()?,
            )
            .await?;
        total_inserted += insert.insert_count();
        if (batch + 1) % 10 == 0 {
            println!("  Inserted {total_inserted} / {TOTAL_ROWS} rows");
        }
    }
    flush(&client, COLLECTION).await?;
    println!("Total inserted: {total_inserted} rows");

    println!("========== Step 3: Create IVF_FLAT index ==========");
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR)
                        .index_type(IndexType::IvfFlat)
                        .metric_type(MetricType::L2)
                        .extra_params(HashMap::from([("nlist".into(), "32".into())])),
                )
                .timeout_ms(100_000)
                .build()?,
        )
        .await?;

    println!("========== Step 4: Load collection ==========");
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    println!("========== Step 5: Query segment info (before optimize) ==========");
    let initial_segment_count = print_segment_info(&client).await?;

    println!("========== Step 6: Optimize (targetSize=4GB, async) ==========");
    let start = Instant::now();
    let task = client
        .optimize(
            OptimizeRequest::builder()
                .collection_name(COLLECTION)
                .target_size("4GB")
                .async_mode(true)
                .build()?,
        )
        .await?;
    let mut last_progress = None;
    while !task.is_done() {
        let progress = task.current_progress();
        if progress.is_some() && progress != last_progress {
            println!(
                "  Optimize progress [{}s]: {}",
                start.elapsed().as_secs(),
                progress.as_deref().unwrap_or_default()
            );
            last_progress = progress;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let result = task.get_result(0).await?;
    println!(
        "Optimize completed in {:.3} seconds",
        start.elapsed().as_secs_f64()
    );
    println!("  Status: {}", result.status_text());
    println!("  Compaction ID: {}", result.compaction_id());
    println!("  Progress: {}", result.progress_history().join(" "));

    println!("========== Step 7: Query segment info (after optimize) ==========");
    let step7 = Instant::now();
    let deadline = step7 + SEGMENT_OBSERVATION_TIMEOUT;
    let mut final_segment_count = initial_segment_count;
    let mut reached_single_segment = false;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        final_segment_count =
            match tokio::time::timeout(remaining, print_segment_info(&client)).await {
                Ok(result) => result?,
                Err(_) => break,
            };
        if final_segment_count == 1 {
            println!("Optimization successful, only one segment remains");
            reached_single_segment = true;
            break;
        }
        println!("Waiting for optimized segments to become observable...");
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        tokio::time::sleep(SEGMENT_POLL_INTERVAL.min(remaining)).await;
    }
    if !reached_single_segment {
        println!(
            "Stopped waiting after {:.3} seconds; optimization may legitimately leave multiple segments",
            step7.elapsed().as_secs_f64()
        );
    }
    println!("Final observed segment count: {final_segment_count}");
    println!(
        "Step 7 completed in {:.3} seconds",
        step7.elapsed().as_secs_f64()
    );

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
