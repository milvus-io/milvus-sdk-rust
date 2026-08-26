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
    /// Creates a collection and, optionally, its indexes and load state.
    ///
    /// When the request contains index parameters, this method creates the collection first,
    /// creates those indexes, and starts loading the collection as asynchronous follow-up
    /// operations. The call therefore does not wait for index-building or loading completion. A
    /// successful lifecycle change invalidates the client's cached collection description.
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

    /// Checks whether a collection exists in the request's database.
    pub async fn has_collection(
        &self,
        request: request::collection::HasCollectionRequest,
    ) -> Result<response::collection::HasCollectionResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, has_collection, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::collection::HasCollectionResponse(response.value))
    }

    /// Drops a collection together with its partitions, indexes, and stored segments.
    ///
    /// This is destructive and cannot be undone. The collection's schema and DML timestamp cache
    /// entries are removed after the server confirms success.
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

    /// Loads collection data into query-node memory.
    ///
    /// With `sync = false`, the method returns after the load request is accepted. With `sync =
    /// true`, it polls load state until success or the request's operation timeout expires; that
    /// timeout covers the polling workflow, not only one RPC attempt.
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

    /// Refreshes a loaded collection or its partitions on query nodes.
    ///
    /// Synchronous requests poll until the refreshed load is complete. Use this after new data or
    /// index changes when the existing loaded view must be updated.
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
                        ..Default::default()
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
            trace_debug!(
                target: "milvus_sdk::polling",
                operation = if refresh { "refresh_load" } else { "load_collection" },
                database,
                collection,
                progress,
                "collection load polling progress"
            );
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

    /// Releases a collection's loaded data from query nodes while retaining its definition.
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

    /// Retrieves a collection's schema, functions, and collection properties.
    ///
    /// This is a direct metadata read; schema-aware DML operations maintain their own shared cache
    /// and should not be assumed to populate it from this call.
    pub async fn describe_collection(
        &self,
        request: request::collection::DescribeCollectionRequest,
    ) -> Result<response::collection::DescribeCollectionResponse> {
        let raw = request.into_proto(&self.current_database());
        let response = rpc_with_retry!(self, describe_collection, raw)?;
        status_to_result(&response.status)?;
        response::collection::DescribeCollectionResponse::from_proto(response)
    }

    /// Lists collections visible in the selected database.
    pub async fn list_collections(
        &self,
        request: request::collection::ListCollectionsRequest,
    ) -> Result<response::collection::ListCollectionsResponse> {
        let response = rpc_with_retry!(self, show_collections, request.into_proto())?;
        status_to_result(&response.status)?;
        response::collection::ListCollectionsResponse::from_proto(response)
    }

    /// Returns collection statistics, currently including the server-reported row count.
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

    /// Retrieves descriptions for multiple collections in one request.
    pub async fn batch_describe_collections(
        &self,
        request: request::collection::BatchDescribeCollectionsRequest,
    ) -> Result<response::collection::BatchDescribeCollectionsResponse> {
        let response = rpc_with_retry!(self, batch_describe_collection, request.into_proto())?;
        status_to_result(&response.status)?;
        response::collection::BatchDescribeCollectionsResponse::from_proto(response)
    }

    /// Describes the query replicas serving a collection and their resource assignments.
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

    /// Returns the current load state for a collection or selected partitions.
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
            ..Default::default()
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

    /// Removes all entities while retaining the collection schema, indexes, and properties.
    ///
    /// Truncation is destructive and non-idempotent. A successful operation updates the shared DML
    /// timestamp state used by Session-consistency reads.
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

    /// Renames a collection within the selected database.
    ///
    /// The SDK moves the collection's session timestamp state to the new name and invalidates
    /// affected schema-cache entries after success.
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
    #[deprecated(
        note = "Milvus 3.0 and later do not support adding a function separately; use add_function_field instead"
    )]
    #[allow(deprecated)]
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
    #[deprecated(
        note = "Milvus 3.0 and later do not support dropping a function separately; use drop_function_field instead"
    )]
    #[allow(deprecated)]
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

    /// Add a nullable struct field to an existing collection.
    pub async fn add_collection_struct_field(
        &self,
        request: request::collection::AddCollectionStructFieldRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            add_collection_struct_field,
            request.into_proto()?
        )?;
        self.status(status)?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Add a function-backed field (e.g. BM25 sparse vector) to an existing collection.
    ///
    /// The request commits the new output field together with the function definition and the
    /// index bound to that output field in one schema change.
    pub async fn add_function_field(
        &self,
        request: request::collection::AddFunctionFieldRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let proto = request.into_proto()?;
        let response = self
            .retry_rpc(
                || Ok(proto.clone()),
                super::RetrySemantics::Idempotent,
                |mut service, request| async move { service.alter_collection_schema(request).await },
                |response| response.alter_status.clone(),
            )
            .await?;
        self.status(response.alter_status.unwrap_or_default())?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Drop a function and its output field from an existing collection.
    pub async fn drop_function_field(
        &self,
        request: request::collection::DropFunctionFieldRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let proto = request.into_proto()?;
        let response = self
            .retry_rpc(
                || Ok(proto.clone()),
                super::RetrySemantics::Idempotent,
                |mut service, request| async move { service.alter_collection_schema(request).await },
                |response| response.alter_status.clone(),
            )
            .await?;
        self.status(response.alter_status.unwrap_or_default())?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }

    /// Drop a field from an existing collection.
    pub async fn drop_collection_field(
        &self,
        request: request::collection::DropCollectionFieldRequest,
    ) -> Result<()> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let proto = request.into_proto()?;
        let response = self
            .retry_rpc(
                || Ok(proto.clone()),
                super::RetrySemantics::Idempotent,
                |mut service, request| async move { service.alter_collection_schema(request).await },
                |response| response.alter_status.clone(),
            )
            .await?;
        self.status(response.alter_status.unwrap_or_default())?;
        self.remove_collection_description(&database, &collection);
        Ok(())
    }
}
