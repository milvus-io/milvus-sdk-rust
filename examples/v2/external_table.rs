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

//! Demonstrates creating and refreshing an external collection over object-storage data.
//!
//! Requires a Milvus deployment with external-table support and access to the configured
//! external storage. Object-storage credentials are read from the `EXTERNAL_ACCESS_KEY` and
//! `EXTERNAL_SECRET_KEY` environment variables; the example falls back to the well-known
//! MinIO defaults (`minioadmin`/`minioadmin`) for local development only. Run against the
//! local default endpoint:
//!
//! ```shell
//! cargo run --example v2_external_table
//! ```

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use std::time::Duration;
use utils::*;

const COLLECTION: &str = "RUST_V2_EXTERNAL_TABLE";
const PRIMARY_FIELD: &str = "product_id";
const VECTOR_FIELD: &str = "embedding";
const DIMENSION: u32 = 128;
const EXTERNAL_SOURCE: &str = "s3://minio:9000/a-bucket/external_table_example_data/";

fn external_storage_spec() -> serde_json::Value {
    let access_key =
        std::env::var("EXTERNAL_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_owned());
    let secret_key =
        std::env::var("EXTERNAL_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_owned());
    serde_json::json!({
        "format": "parquet",
        "extfs": {
            "access_key_id": access_key,
            "access_key_value": secret_key,
            "region": "us-east-1",
            "use_ssl": "false",
            "use_virtual_host": "false",
            "cloud_provider": "minio",
        },
    })
}

async fn wait_refresh_complete(
    client: &ClientV2,
    job_id: i64,
) -> Result<RefreshExternalCollectionJobInfo> {
    // Poll the asynchronous refresh job until it completes, fails, or times out.
    for _ in 0..60 {
        let progress = client
            .get_refresh_external_collection_progress(
                sdk::request::utility::GetRefreshExternalCollectionProgressRequest::builder()
                    .job_id(job_id)
                    .build()?,
            )
            .await?;
        let job_info = progress.job_info();
        println!(
            "Refresh job {}: state={}, progress={}%",
            job_info.get_job_id(),
            job_info.get_state().as_str(),
            job_info.get_progress()
        );
        match job_info.get_state() {
            RefreshExternalCollectionStateCode::Completed => return Ok(job_info.clone()),
            RefreshExternalCollectionStateCode::Failed => {
                return Err(milvus::v2::error::Error::Unexpected(format!(
                    "refresh external collection failed: {}",
                    job_info.get_reason()
                )));
            }
            _ => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    Err(milvus::v2::error::Error::Unexpected(
        "refresh external collection did not complete within 60 seconds".into(),
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;

    // Build an external-collection schema: each data field maps to a source field
    // through `external_field`, and the schema carries the object-storage path and
    // format spec. Credentials come from environment variables (see above).
    let schema = CollectionSchema::new()
        .external_source(EXTERNAL_SOURCE)
        .external_spec(external_storage_spec())
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(PRIMARY_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true)
                .external_field("id"),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR_FIELD)
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION)
                .external_field("vector"),
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
    println!("External collection '{COLLECTION}' created");

    // Refresh the external collection so its data becomes queryable.
    let refresh = client
        .refresh_external_collection(
            sdk::request::utility::RefreshExternalCollectionRequest::builder()
                .collection_name(COLLECTION)
                .external_source(EXTERNAL_SOURCE)
                .build()?,
        )
        .await?;
    println!(
        "Refresh external collection job submitted, job_id={}",
        refresh.job_id()
    );

    let job_info = wait_refresh_complete(&client, refresh.job_id()).await?;
    println!(
        "Refresh job state: state={}, progress={}, reason={}",
        job_info.get_state().as_str(),
        job_info.get_progress(),
        job_info.get_reason()
    );

    let jobs = client
        .list_refresh_external_collection_jobs(
            sdk::request::utility::ListRefreshExternalCollectionJobsRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Refresh jobs: {}", jobs.jobs().len());

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
