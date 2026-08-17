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

//! Demonstrates snapshot creation, listing, description, restore, pinning, and cleanup.
//!
//! Requires a Milvus deployment that supports snapshots. Run against the local default endpoint:
//!
//! ```shell
//! cargo run --example v2_snapshot
//! ```

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use std::time::Duration;
use utils::*;

const COLLECTION: &str = "RUST_V2_SNAPSHOT";
const RESTORE_COLLECTION: &str = "RUST_V2_SNAPSHOT_RESTORE";
const SNAPSHOT_NAME: &str = "rust_sdk_example_snapshot_backup";
const PRIMARY_FIELD: &str = "id";
const VECTOR_FIELD: &str = "vector";
const DIMENSION: usize = 4;
const ROW_COUNT: i64 = 100;

async fn query_row_count(client: &ClientV2, collection: &str) -> Result<u64> {
    // Query the persisted row count of a collection before and after restore.
    let results = client
        .query(
            sdk::request::dql::QueryRequest::builder()
                .collection_name(collection)
                .output_fields(["count(*)"])
                .consistency_level(ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    query_count(results.results())
}

async fn wait_restore_complete(client: &ClientV2, job_id: i64) -> Result<RestoreSnapshotJobInfo> {
    // Poll the asynchronous restore job until it completes, fails, or times out.
    for _ in 0..60 {
        let state = client
            .get_restore_snapshot_state(
                sdk::request::snapshot::GetRestoreSnapshotStateRequest::builder()
                    .job_id(job_id)
                    .build()?,
            )
            .await?;
        let job_info = state.job_info();
        match job_info.get_state() {
            RestoreSnapshotStateCode::Completed => return Ok(job_info.clone()),
            RestoreSnapshotStateCode::Failed => {
                return Err(milvus::v2::error::Error::Unexpected(format!(
                    "restore snapshot failed: {}",
                    job_info.get_reason()
                )));
            }
            _ => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    Err(milvus::v2::error::Error::Unexpected(
        "restore snapshot did not complete within 60 seconds".into(),
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;

    // Start from a clean slate so the example is idempotent.
    drop_collection(&client, RESTORE_COLLECTION).await;
    drop_collection(&client, COLLECTION).await;
    client
        .create_collection(
            sdk::request::collection::CreateSimpleCollectionRequest::builder()
                .collection_name(COLLECTION)
                .primary_field(PRIMARY_FIELD)
                .vector_field(VECTOR_FIELD)
                .dimension(DIMENSION as u32)
                .build()?,
        )
        .await?;
    println!("Collection '{COLLECTION}' created");

    let rows: Vec<_> = (0..ROW_COUNT)
        .map(|id| {
            serde_json::json!({
                PRIMARY_FIELD: id,
                VECTOR_FIELD: vec![
                    id as f32,
                    id as f32 / 2.0,
                    id as f32 / 3.0,
                    id as f32 / 4.0,
                ],
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

    // Flush so the inserted data is persisted and covered by the snapshot.
    flush(&client, COLLECTION).await?;
    println!(
        "Collection '{COLLECTION}' flushed with {} rows",
        query_row_count(&client, COLLECTION).await?
    );

    // Create a snapshot of the collection; compaction_protection_seconds keeps
    // the referenced segments from being compacted for the given duration.
    client
        .create_snapshot(
            sdk::request::snapshot::CreateSnapshotRequest::builder()
                .collection_name(COLLECTION)
                .snapshot_name(SNAPSHOT_NAME)
                .description("Snapshot example backup")
                .build()?,
        )
        .await?;
    println!("Snapshot '{SNAPSHOT_NAME}' created");

    let snapshots = client
        .list_snapshots(
            sdk::request::snapshot::ListSnapshotsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Snapshots:");
    for name in snapshots.snapshots() {
        println!("\t{name}");
    }

    let description = client
        .describe_snapshot(
            sdk::request::snapshot::DescribeSnapshotRequest::builder()
                .collection_name(COLLECTION)
                .snapshot_name(SNAPSHOT_NAME)
                .build()?,
        )
        .await?;
    println!(
        "Snapshot detail: name={}, collection={}, partitions={:?}, create_ts={}, s3_location={}",
        description.name(),
        description.collection_name(),
        description.partition_names(),
        description.create_ts(),
        description.s3_location(),
    );

    // Pin the snapshot data so it is not reclaimed; 0 means never expire, here we
    // use 3600 seconds. The returned pin id is required by unpin_snapshot_data.
    let pin = client
        .pin_snapshot_data(
            sdk::request::snapshot::PinSnapshotDataRequest::builder()
                .collection_name(COLLECTION)
                .snapshot_name(SNAPSHOT_NAME)
                .ttl_seconds(3600)
                .build()?,
        )
        .await?;
    println!("Snapshot data pinned, pin_id={}", pin.pin_id());

    // Restore the snapshot into a brand new collection.
    let restore = client
        .restore_snapshot(
            sdk::request::snapshot::RestoreSnapshotRequest::builder()
                .snapshot_name(SNAPSHOT_NAME)
                .source_collection_name(COLLECTION)
                .target_collection_name(RESTORE_COLLECTION)
                .build()?,
        )
        .await?;
    println!(
        "Restore snapshot job submitted, job_id={}",
        restore.job_id()
    );

    let job_info = wait_restore_complete(&client, restore.job_id()).await?;
    println!(
        "Restore job state: state={:?}, progress={}, reason={}",
        job_info.get_state(),
        job_info.get_progress(),
        job_info.get_reason()
    );

    // Load the restored collection before querying it.
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(RESTORE_COLLECTION)
                .build()?,
        )
        .await?;
    println!("Restored collection '{RESTORE_COLLECTION}' loaded");

    let jobs = client
        .list_restore_snapshot_jobs(
            sdk::request::snapshot::ListRestoreSnapshotJobsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Restore jobs: {}", jobs.jobs().len());

    let source_count = query_row_count(&client, COLLECTION).await?;
    let target_count = query_row_count(&client, RESTORE_COLLECTION).await?;
    println!(
        "Source persisted row count={source_count}, target persisted row count={target_count}"
    );
    if source_count != target_count {
        return Err(milvus::v2::error::Error::Unexpected(format!(
            "restored row count mismatch: source={source_count}, target={target_count}"
        )));
    }

    client
        .unpin_snapshot_data(
            sdk::request::snapshot::UnpinSnapshotDataRequest::builder()
                .pin_id(pin.pin_id())
                .build()?,
        )
        .await?;
    println!("Snapshot data unpinned, pin_id={}", pin.pin_id());

    client
        .drop_snapshot(
            sdk::request::snapshot::DropSnapshotRequest::builder()
                .collection_name(COLLECTION)
                .snapshot_name(SNAPSHOT_NAME)
                .build()?,
        )
        .await?;
    println!("Snapshot '{SNAPSHOT_NAME}' dropped");

    drop_collection(&client, RESTORE_COLLECTION).await;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
