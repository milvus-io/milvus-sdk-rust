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

//! ClientV2 collection lifecycle and schema operations.

use super::internal::ALLOW_INSERT_AUTO_ID;
use super::ClientV2;
use crate::proto::milvus;
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::{request, response, LoadState};
use std::time::Duration;
use tokio::time::{sleep, Instant};

const LOAD_POLL_INTERVAL: Duration = Duration::from_millis(500);

impl ClientV2 {
    /// Create a collection.
    pub async fn create_collection(
        &self,
        request: impl Into<request::collection::CreateCollectionRequest>,
    ) -> Result<()> {
        let mut request = request.into();
        let database = self.current_database();
        let effective_database = self.effective_database(request.database_name.as_deref());
        let collection_name = request.collection_name.clone();
        let index_params = std::mem::take(&mut request.index_params);
        let follow_up = if index_params.is_empty() {
            None
        } else {
            let index_request = request::index::CreateIndexRequest::builder()
                .database_name(effective_database.clone())
                .collection_name(collection_name.clone())
                .index_params(index_params)
                .sync(false)
                .build()?;
            let load_request = request::collection::LoadCollectionRequest::builder()
                .database_name(effective_database.clone())
                .collection_name(collection_name.clone())
                .sync(false)
                .build()?;
            Some((index_request, load_request))
        };
        let raw = request.into_proto(&database)?;
        self.status(status_rpc_with_retry!(
            Idempotent,
            self,
            create_collection,
            raw
        )?)?;
        self.remove_collection_description(&effective_database, &collection_name);

        if let Some((index_request, load_request)) = follow_up {
            self.create_index(index_request).await?;
            self.load_collection(load_request).await?;
        }
        Ok(())
    }

    /// Check existence of a collection.
    pub async fn has_collection(
        &self,
        request: request::collection::HasCollectionRequest,
    ) -> Result<response::collection::HasCollectionResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, has_collection, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::collection::HasCollectionResponse(response.value))
    }

    /// Drop a collection, with all its partitions, index and segments.
    pub async fn drop_collection(
        &self,
        request: request::collection::DropCollectionRequest,
    ) -> Result<()> {
        let default_db = self.current_database();
        let raw = request.into_proto(&default_db);
        let db = raw.db_name.clone();
        let name = raw.collection_name.clone();
        self.status(status_rpc_with_retry!(
            Idempotent,
            self,
            drop_collection,
            raw
        )?)?;
        self.remove_collection_cache(&db, &name);
        Ok(())
    }

    /// Load collection data into CPU memory of query node.
    pub async fn load_collection(
        &self,
        mut request: request::collection::LoadCollectionRequest,
    ) -> Result<()> {
        let sync = request.sync;
        let timeout_ms = request.timeout_ms;
        let refresh = request.refresh;
        let database = self.effective_database(request.database_name.as_deref());
        request.database_name = Some(database.clone());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            load_collection,
            request.into_proto(&database)
        )?;
        self.status(status)?;

        if !sync {
            return Ok(());
        }
        self.wait_for_collection_loading(
            &database,
            &collection,
            &[],
            refresh,
            timeout_ms,
            "load collection timed out",
        )
        .await
    }

    /// Refresh loaded collection data in query node.
    pub async fn refresh_load(
        &self,
        mut request: request::collection::RefreshLoadRequest,
    ) -> Result<()> {
        let sync = request.sync;
        let timeout_ms = request.timeout_ms;
        let database = self.effective_database(request.database_name.as_deref());
        request.database_name = Some(database.clone());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            load_collection,
            request.into_proto(&database)
        )?;
        self.status(status)?;

        if !sync {
            return Ok(());
        }

        self.wait_for_collection_loading(
            &database,
            &collection,
            &[],
            true,
            timeout_ms,
            "refresh load timed out",
        )
        .await
    }

    pub(super) async fn wait_for_collection_loading(
        &self,
        database: &str,
        collection: &str,
        partition_names: &[String],
        refresh: bool,
        timeout_ms: i64,
        timeout_message: &str,
    ) -> Result<()> {
        let deadline =
            (timeout_ms > 0).then(|| Instant::now() + Duration::from_millis(timeout_ms as u64));
        loop {
            let poll = async {
                rpc_with_retry!(
                    self,
                    get_loading_progress,
                    milvus::GetLoadingProgressRequest {
                        base: None,
                        collection_name: collection.to_owned(),
                        partition_names: partition_names.to_vec(),
                        db_name: database.to_owned(),
                    }
                )
            };
            let response = if let Some(deadline) = deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(Error::Timeout(
                        timeout_message.trim_end_matches(" timed out").into(),
                    ));
                }
                match tokio::time::timeout(remaining, poll).await {
                    Ok(response) => response?,
                    Err(_) => {
                        return Err(Error::Timeout(
                            timeout_message.trim_end_matches(" timed out").into(),
                        ))
                    }
                }
            } else {
                poll.await?
            };
            status_to_result(&response.status)?;
            let progress = if refresh {
                response.refresh_progress
            } else {
                response.progress
            };
            if progress >= 100 {
                return Ok(());
            }

            let delay = if let Some(deadline) = deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(Error::Timeout(
                        timeout_message.trim_end_matches(" timed out").into(),
                    ));
                }
                LOAD_POLL_INTERVAL.min(remaining)
            } else {
                LOAD_POLL_INTERVAL
            };
            sleep(delay).await;
        }
    }

    /// Release collection data from query node.
    pub async fn release_collection(
        &self,
        request: request::collection::ReleaseCollectionRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            release_collection,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Get collection description, including its schema and properties.
    pub async fn describe_collection(
        &self,
        request: request::collection::DescribeCollectionRequest,
    ) -> Result<response::collection::DescribeCollectionResponse> {
        let raw = request.into_proto(&self.current_database());
        let response = rpc_with_retry!(self, describe_collection, raw)?;
        status_to_result(&response.status)?;
        response::collection::DescribeCollectionResponse::from_proto(response)
    }

    /// List all collections brief information.
    pub async fn list_collections(
        &self,
        request: request::collection::ListCollectionsRequest,
    ) -> Result<response::collection::ListCollectionsResponse> {
        let response = rpc_with_retry!(self, show_collections, request.into_proto())?;
        status_to_result(&response.status)?;
        response::collection::ListCollectionsResponse::from_proto(response)
    }

    /// Get collection statistics, currently only return row count.
    pub async fn get_collection_stats(
        &self,
        request: request::collection::GetCollectionStatsRequest,
    ) -> Result<response::collection::GetCollectionStatsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(
            self,
            get_collection_statistics,
            request.into_proto(&database)
        )?;
        status_to_result(&response.status)?;
        Ok(response::collection::GetCollectionStatsResponse::from_proto(response))
    }

    /// Describe multiple collections.
    pub async fn batch_describe_collections(
        &self,
        request: request::collection::BatchDescribeCollectionsRequest,
    ) -> Result<response::collection::BatchDescribeCollectionsResponse> {
        let response = rpc_with_retry!(self, batch_describe_collection, request.into_proto())?;
        status_to_result(&response.status)?;
        response::collection::BatchDescribeCollectionsResponse::from_proto(response)
    }

    /// Describe replicas of a collection.
    pub async fn describe_replicas(
        &self,
        request: request::collection::DescribeReplicasRequest,
    ) -> Result<response::collection::DescribeReplicasResponse> {
        let response = rpc_with_retry!(self, get_replicas, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::collection::DescribeReplicasResponse::from_proto(
            response,
        ))
    }

    /// Get load state of collection or partitions.
    pub async fn get_load_state(
        &self,
        request: request::collection::GetLoadStateRequest,
    ) -> Result<response::collection::GetLoadStateResponse> {
        let database = self.current_database();
        let request = request.into_proto(&database);
        let progress_request = milvus::GetLoadingProgressRequest {
            base: None,
            collection_name: request.collection_name.clone(),
            partition_names: request.partition_names.clone(),
            db_name: request.db_name.clone(),
        };
        let response = rpc_with_retry!(self, get_load_state, request)?;
        status_to_result(&response.status)?;
        let state = LoadState::from_proto(response.state);
        let progress = match state {
            LoadState::Loading => {
                let progress = rpc_with_retry!(self, get_loading_progress, progress_request)?;
                status_to_result(&progress.status)?;
                progress.progress
            }
            LoadState::Loaded => 100,
            LoadState::NotExist | LoadState::NotLoad => 0,
            LoadState::Unknown => {
                return Err(crate::v2::error::Error::MalformedResponse(format!(
                    "get load state returned unknown state {}",
                    response.state
                )));
            }
        };
        Ok(response::collection::GetLoadStateResponse::from_proto(
            response, progress,
        ))
    }

    /// Truncate a collection, removing all data while keeping the collection structure.
    pub async fn truncate_collection(
        &self,
        request: request::collection::TruncateCollectionRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let request = request.into_proto(&database);
        let database = request.db_name.clone();
        let collection = request.collection_name.clone();
        let response = rpc_with_retry!(NonIdempotent, self, truncate_collection, request)?;
        status_to_result(&response.status)?;
        self.remove_dml_timestamp(&database, &collection);
        Ok(())
    }

    /// RenameCollection rename a collection.
    pub async fn rename_collection(
        &self,
        request: request::collection::RenameCollectionRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let raw = request.into_proto(&database);
        let old_database = raw.db_name.clone();
        let new_database = raw.new_db_name.clone();
        let old_name = raw.old_name.clone();
        let new_name = raw.new_name.clone();
        let status = status_rpc_with_retry!(Idempotent, self, rename_collection, raw)?;
        self.status(status)?;
        self.rename_collection_cache(&old_database, &old_name, &new_database, &new_name);
        Ok(())
    }

    /// Alter a collection's properties.
    pub async fn alter_collection_properties(
        &self,
        request: request::collection::AlterCollectionPropertiesRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let invalidates_schema = request.properties.contains_key(ALLOW_INSERT_AUTO_ID);
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            alter_collection,
            request.into_proto(&database)
        )?;
        self.status(status)?;
        if invalidates_schema {
            self.remove_collection_description(&database, &collection);
        }
        Ok(())
    }

    /// Drop a collection's properties.
    pub async fn drop_collection_properties(
        &self,
        request: request::collection::DropCollectionPropertiesRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let invalidates_schema = request.property_keys.contains(ALLOW_INSERT_AUTO_ID);
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            alter_collection,
            request.into_proto(&database)
        )?;
        self.status(status)?;
        if invalidates_schema {
            self.remove_collection_description(&database, &collection);
        }
        Ok(())
    }

    /// Alter a field's properties.
    pub async fn alter_collection_field_properties(
        &self,
        request: request::collection::AlterCollectionFieldPropertiesRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            alter_collection_field,
            request.into_proto(&database)
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Drop a field's properties.
    pub async fn drop_collection_field_properties(
        &self,
        request: request::collection::DropCollectionFieldPropertiesRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            alter_collection_field,
            request.into_proto(&database)
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Add a field to an existing collection.
    pub async fn add_collection_field(
        &self,
        request: request::collection::AddCollectionFieldRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            add_collection_field,
            request.into_proto()?
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Add a function to an existing collection.
    pub async fn add_collection_function(
        &self,
        request: request::collection::AddCollectionFunctionRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            add_collection_function,
            request.into_proto()
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Alter a function of an existing collection.
    pub async fn alter_collection_function(
        &self,
        request: request::collection::AlterCollectionFunctionRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            alter_collection_function,
            request.into_proto()
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Drop a function of an existing collection.
    pub async fn drop_collection_function(
        &self,
        request: request::collection::DropCollectionFunctionRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            drop_collection_function,
            request.into_proto()
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }
}
