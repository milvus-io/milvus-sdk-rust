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

//! ClientV2 index-management operations.

use super::ClientV2;
use crate::proto::milvus;
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::types::IndexStateCode;
use crate::v2::{request, response};
use std::time::Duration;
use tokio::time::{sleep, timeout, Instant};

const INDEX_POLL_INTERVAL: Duration = Duration::from_millis(500);

impl ClientV2 {
    /// Creates one or more vector or scalar indexes for a collection.
    ///
    /// Index creation is asynchronous on the server. Set the request's synchronization and timeout
    /// options when the caller must wait until the index is ready; otherwise this method returns
    /// after the create request is accepted.
    pub async fn create_index(&self, request: request::index::CreateIndexRequest) -> Result<()> {
        let database = self.current_database();
        let (database, collection, index_params, sync, timeout_ms) = request.into_parts(&database);
        for index_param in index_params {
            let field_name = index_param.field_name.clone();
            let index_name = index_param.index_name.clone();
            let raw = index_param.into_proto(database.clone(), collection.clone());
            let status = status_rpc_with_retry!(Idempotent, self, create_index, raw)?;
            self.status(status)?;
            if sync {
                self.wait_for_index(&database, &collection, &field_name, &index_name, timeout_ms)
                    .await?;
            }
        }
        Ok(())
    }

    /// Retrieves index definitions and server-side parameters for a collection field.
    pub async fn describe_index(
        &self,
        request: request::index::DescribeIndexRequest,
    ) -> Result<response::index::DescribeIndexResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, describe_index, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        response::index::DescribeIndexResponse::from_proto(response)
    }

    /// Drops the index associated with a collection field.
    pub async fn drop_index(&self, request: request::index::DropIndexRequest) -> Result<()> {
        let database = self.current_database();
        let status =
            status_rpc_with_retry!(Idempotent, self, drop_index, request.into_proto(&database))?;
        self.status(status)
    }

    /// Lists index names and metadata associated with a collection.
    pub async fn list_indexes(
        &self,
        request: request::index::ListIndexesRequest,
    ) -> Result<response::index::ListIndexesResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, get_index_statistics, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        response::index::ListIndexesResponse::from_proto(response)
    }

    /// Updates mutable properties of an existing index.
    pub async fn alter_index_properties(
        &self,
        request: request::index::AlterIndexPropertiesRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status =
            status_rpc_with_retry!(Idempotent, self, alter_index, request.into_proto(&database))?;
        self.status(status)
    }

    /// Removes the requested mutable properties from an index.
    pub async fn drop_index_properties(
        &self,
        request: request::index::DropIndexPropertiesRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status =
            status_rpc_with_retry!(Idempotent, self, alter_index, request.into_proto(&database))?;
        self.status(status)
    }

    async fn wait_for_index(
        &self,
        database: &str,
        collection: &str,
        field_name: &str,
        index_name: &str,
        timeout_ms: i64,
    ) -> Result<()> {
        let deadline =
            (timeout_ms > 0).then(|| Instant::now() + Duration::from_millis(timeout_ms as u64));
        let timestamp = wait_for_index_poll(deadline, async {
            rpc_with_retry!(
                self,
                alloc_timestamp,
                milvus::AllocTimestampRequest {
                    base: None,
                    ..Default::default()
                }
            )
        })
        .await?;
        status_to_result(&timestamp.status)?;

        loop {
            let response = wait_for_index_poll(deadline, async {
                rpc_with_retry!(
                    self,
                    describe_index,
                    milvus::DescribeIndexRequest {
                        base: None,
                        db_name: database.to_owned(),
                        collection_name: collection.to_owned(),
                        field_name: field_name.to_owned(),
                        index_name: index_name.to_owned(),
                        timestamp: timestamp.timestamp,
                        ..Default::default()
                    }
                )
            })
            .await?;
            status_to_result(&response.status)?;

            let indexes: Vec<_> = response
                .index_descriptions
                .into_iter()
                .filter(|index| {
                    (!index_name.is_empty() && index.index_name == index_name)
                        || (index_name.is_empty() && index.field_name == field_name)
                })
                .collect();
            if indexes.is_empty() {
                return Err(Error::MalformedResponse(format!(
                    "created index cannot be described for field {field_name:?} and index {index_name:?}"
                )));
            }

            let mut finished = true;
            for index in indexes {
                let state = IndexStateCode::from_proto(index.state);
                trace_debug!(
                    target: "milvus_sdk::polling",
                    operation = "create_index",
                    database,
                    collection,
                    field = %index.field_name,
                    index = %index.index_name,
                    state = ?state,
                    "index polling state"
                );
                match state {
                    IndexStateCode::Finished => {}
                    IndexStateCode::Failed => {
                        return Err(Error::Unexpected(format!(
                            "index creation failed: {}",
                            index.index_state_fail_reason
                        )));
                    }
                    _ => finished = false,
                }
            }
            if finished {
                return Ok(());
            }
            sleep(next_index_poll_delay(deadline)?).await;
        }
    }
}

fn next_index_poll_delay(deadline: Option<Instant>) -> Result<Duration> {
    match deadline {
        Some(deadline) => {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                Err(index_wait_timeout())
            } else {
                Ok(INDEX_POLL_INTERVAL.min(remaining))
            }
        }
        None => Ok(INDEX_POLL_INTERVAL),
    }
}

async fn wait_for_index_poll<T>(
    deadline: Option<Instant>,
    poll: impl std::future::Future<Output = Result<T>>,
) -> Result<T> {
    if let Some(deadline) = deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(index_wait_timeout());
        }
        match timeout(remaining, poll).await {
            Ok(result) => result,
            Err(_) => Err(index_wait_timeout()),
        }
    } else {
        poll.await
    }
}

fn index_wait_timeout() -> Error {
    Error::Timeout("creating index".into())
}
