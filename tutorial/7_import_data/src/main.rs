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

use milvus::v2::prelude::*;
use std::time::{Duration, Instant};

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);
const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(10 * 60);

#[tokio::main]
async fn main() -> Result<()> {
    let url = env_or("MILVUS_URI", "http://localhost:19530");
    let api_key = env_or("MILVUS_TOKEN", "root:Milvus");
    let database = env_or("MILVUS_DATABASE", "");
    let collection = required_env("MILVUS_COLLECTION")?;
    let files = parse_file_groups(&required_env("MILVUS_IMPORT_FILES")?)?;

    let bulk_import = BulkImport::new(
        &BulkImportConfig::new()
            .url(url)
            .api_key(api_key)
            .timeout(Duration::from_secs(20)),
    )?;

    let mut import_builder = BulkImportRequest::builder()
        .database_name(&database)
        .collection_name(&collection)
        .files(files);
    if let Ok(partition) = std::env::var("MILVUS_PARTITION") {
        if !partition.is_empty() {
            import_builder = import_builder.partition_name(partition);
        }
    }

    let response = bulk_import.bulk_import(import_builder.build()?).await?;
    let job_id = response
        .job_id()
        .ok_or_else(|| Error::MalformedResponse("bulk-import response has no jobId".into()))?
        .to_owned();
    println!("Created import job {job_id}");

    let jobs = bulk_import
        .list_import_jobs(
            ListImportJobsRequest::builder()
                .database_name(&database)
                .collection_name(&collection)
                .page_size(10)
                .current_page(1)
                .build()?,
        )
        .await?;
    println!("Recent import jobs: {}", jobs.data());

    wait_for_import(&bulk_import, &database, &job_id, DEFAULT_WAIT_TIMEOUT).await?;

    println!(
        "Import completed. Load the collection, or refresh it if it was already loaded, before querying the new data."
    );
    Ok(())
}

async fn wait_for_import(
    bulk_import: &BulkImport,
    database: &str,
    job_id: &str,
    timeout: Duration,
) -> Result<()> {
    let started = Instant::now();
    loop {
        let response = bulk_import
            .get_import_progress(
                GetImportProgressRequest::builder()
                    .database_name(database)
                    .job_id(job_id)
                    .build()?,
            )
            .await?;
        let state = response
            .state()
            .ok_or_else(|| Error::MalformedResponse(format!("import job {job_id} has no state")))?;
        let progress = response.progress().unwrap_or_default();
        println!("job={job_id}, state={state}, progress={progress}%");

        match state {
            "Completed" => return Ok(()),
            "Failed" => {
                return Err(Error::Unexpected(format!(
                    "import job {job_id} failed: {}",
                    response.reason().unwrap_or("no reason returned")
                )))
            }
            _ => {}
        }

        let remaining = timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(Error::Timeout(format!("waiting for import job {job_id}")));
        }
        tokio::time::sleep(DEFAULT_POLL_INTERVAL.min(remaining)).await;
    }
}

fn parse_file_groups(value: &str) -> Result<Vec<Vec<String>>> {
    let groups = value
        .split(';')
        .map(|group| {
            group
                .split(',')
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|group| !group.is_empty())
        .collect::<Vec<_>>();

    if groups.is_empty() {
        return Err(Error::Unexpected(
            "MILVUS_IMPORT_FILES must contain at least one object key from the S3-compatible bucket configured for Milvus"
                .into(),
        ));
    }
    Ok(groups)
}

fn required_env(name: &str) -> Result<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::Unexpected(format!("environment variable {name} must be set")))
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}
