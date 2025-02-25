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

use crate::config;
use crate::data::FieldColumn;
use crate::error::{Error as SuperError, Result};
use crate::index::{IndexInfo, IndexParams};
use crate::proto::milvus::{
    CreateCollectionRequest, CreateIndexRequest, DescribeIndexRequest, DropCollectionRequest,
    DropIndexRequest, FlushRequest, GetCompactionStateRequest, GetCompactionStateResponse,
    HasCollectionRequest, LoadCollectionRequest, ManualCompactionRequest, ManualCompactionResponse,
    ReleaseCollectionRequest, ShowCollectionsRequest,
};
use crate::proto::schema::DataType;
use crate::schema::CollectionSchema;
use crate::types::*;
use crate::utils::status_to_result;
use crate::value::Value;
use crate::{
    client::{AuthInterceptor, Client},
    options::{CreateCollectionOptions, GetLoadStateOptions, LoadOptions},
    proto::{
        self,
        common::{ConsistencyLevel, IndexState, MsgBase, MsgType},
        milvus::{milvus_service_client::MilvusServiceClient, DescribeCollectionRequest},
    },
};
use prost::bytes::BytesMut;
use prost::Message;
use serde_json;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error as ThisError;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub auto_id: bool,
    pub num_shards: usize,
    // pub num_partitions: usize,
    pub consistency_level: ConsistencyLevel,
    pub description: String,
    pub fields: Vec<Field>,
    // pub enable_dynamic_field: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CollectionCache {
    collections: dashmap::DashMap<String, Collection>,
    timestamps: dashmap::DashMap<String, Timestamp>,
    client: MilvusServiceClient<InterceptedService<Channel, AuthInterceptor>>,
}

impl CollectionCache {
    pub fn new(client: MilvusServiceClient<InterceptedService<Channel, AuthInterceptor>>) -> Self {
        Self {
            collections: dashmap::DashMap::new(),
            timestamps: dashmap::DashMap::new(),
            client: client,
        }
    }

    pub async fn get<'a>(&self, name: &str) -> Result<Collection> {
        if !self.local_exist(name) {
            let resp = self
                .client
                .clone()
                .describe_collection(DescribeCollectionRequest {
                    base: Some(MsgBase::new(MsgType::DescribeCollection)),
                    db_name: "".to_owned(),
                    collection_name: name.into(),
                    collection_id: 0,
                    time_stamp: 0,
                })
                .await?
                .into_inner();

            status_to_result(&resp.status)?;
            self.collections
                .insert(name.to_owned(), Collection::from(resp));
        }

        self.collections
            .get(name)
            .map(|v| v.value().clone())
            .ok_or(SuperError::Collection(Error::CollectionNotFound(
                name.to_owned(),
            )))
    }

    pub fn update_timestamp(&self, name: &str, timestamp: Timestamp) {
        self.timestamps
            .entry(name.to_owned())
            .and_modify(|t| {
                if *t < timestamp {
                    *t = timestamp;
                }
            })
            .or_insert(timestamp);
    }

    pub fn get_timestamp(&self, name: &str) -> Option<Timestamp> {
        self.timestamps.get(name).map(|v| v.value().clone())
    }

    fn local_exist(&self, name: &str) -> bool {
        self.collections.contains_key(name)
    }
}

impl From<proto::milvus::DescribeCollectionResponse> for Collection {
    fn from(value: proto::milvus::DescribeCollectionResponse) -> Self {
        let schema = value.schema.unwrap();
        Self {
            id: value.collection_id,
            name: value.collection_name,
            auto_id: schema.auto_id,
            num_shards: value.shards_num as usize,
            // num_partitions: value.partitions_num as usize,
            consistency_level: ConsistencyLevel::from_i32(value.consistency_level).unwrap(),
            description: schema.description,
            fields: schema.fields.into_iter().map(|f| Field::from(f)).collect(),
            // enable_dynamic_field: value.enable_dynamic_field,
        }
    }
}

#[derive(Debug)]
pub struct Partition {
    pub name: String,
    pub percentage: i64,
}

#[derive(Debug)]
pub struct CompactionInfo {
    pub id: i64,
    pub plan_count: i32,
}

impl From<ManualCompactionResponse> for CompactionInfo {
    fn from(value: proto::milvus::ManualCompactionResponse) -> Self {
        Self {
            id: value.compaction_id,
            plan_count: value.compaction_plan_count,
        }
    }
}

#[derive(Debug)]
pub struct CompactionState {
    pub state: crate::proto::common::CompactionState,
    pub executing_plan_num: i64,
    pub timeout_plan_num: i64,
    pub completed_plan_num: i64,
    pub failed_plan_num: i64,
}

impl From<GetCompactionStateResponse> for CompactionState {
    fn from(value: GetCompactionStateResponse) -> Self {
        Self {
            state: crate::proto::common::CompactionState::from_i32(value.state).unwrap(),
            executing_plan_num: value.executing_plan_no,
            timeout_plan_num: value.timeout_plan_no,
            completed_plan_num: value.completed_plan_no,
            failed_plan_num: value.failed_plan_no,
        }
    }
}

type ConcurrentHashMap<K, V> = tokio::sync::RwLock<std::collections::HashMap<K, V>>;

impl Client {
    /// Creates a new collection with the specified schema and options.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema of the collection.
    /// * `options` - Optional parameters for creating the collection.
    pub async fn create_collection(
        &self,
        schema: CollectionSchema,
        options: Option<CreateCollectionOptions>,
    ) -> Result<()> {
        let options = options.unwrap_or_default();
        let schema: crate::proto::schema::CollectionSchema = schema.into();
        let mut buf = BytesMut::new();

        schema.encode(&mut buf)?;

        let status = self
            .client
            .clone()
            .create_collection(CreateCollectionRequest {
                base: Some(MsgBase::new(MsgType::CreateCollection)),
                collection_name: schema.name.to_string(),
                schema: buf.to_vec(),
                shards_num: options.shard_num,
                consistency_level: options.consistency_level as i32,
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&Some(status))
    }

    /// Drops a collection with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection to drop.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn drop_collection<S>(&self, name: S) -> Result<()>
    where
        S: Into<String>,
    {
        status_to_result(&Some(
            self.client
                .clone()
                .drop_collection(DropCollectionRequest {
                    base: Some(MsgBase::new(MsgType::DropCollection)),
                    collection_name: name.into(),
                    ..Default::default()
                })
                .await?
                .into_inner(),
        ))
    }

    /// Retrieves a list of collections.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of collection names if successful, or an error if the operation fails.
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .clone()
            .show_collections(ShowCollectionsRequest {
                base: Some(MsgBase::new(MsgType::ShowCollections)),
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&response.status)?;
        Ok(response.collection_names)
    }

    /// Retrieves information about a collection.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Collection` information if successful, or an error if the collection does not exist or cannot be accessed.
    pub async fn describe_collection<S>(&self, name: S) -> Result<Collection>
    where
        S: Into<String>,
    {
        let resp = self
            .client
            .clone()
            .describe_collection(DescribeCollectionRequest {
                base: Some(MsgBase::new(MsgType::DescribeCollection)),
                db_name: "".to_owned(),
                collection_name: name.into(),
                collection_id: 0,
                time_stamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(&resp.status)?;

        Ok(resp.into())
    }

    /// Checks if a collection with the given name exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating whether the collection exists or not.
    pub async fn has_collection<S>(&self, name: S) -> Result<bool>
    where
        S: Into<String>,
    {
        let name = name.into();
        let res = self
            .client
            .clone()
            .has_collection(HasCollectionRequest {
                base: Some(MsgBase::new(MsgType::HasCollection)),
                db_name: "".to_string(),
                collection_name: name.clone(),
                time_stamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res.value)
    }

    // todo(yah01): implement this after the refactor done
    // pub async fn rename_collection<S>(&self, name: S, new_name: S) -> Result<()>
    // where
    //     S: Into<String>,
    // {
    //     let name = name.into();
    //     let new_name = new_name.into();
    //     let res = self
    //         .client
    //         .clone()
    //         .rename_collection(proto::milvus::RenameCollectionRequest {
    //             base: Some(MsgBase::new(MsgType::RenameCollection)),
    //             collection_name: name.clone(),
    //             new_collection_name: new_name.clone(),
    //         })
    //         .await?
    //         .into_inner();

    //     status_to_result(&Some(res))?;

    //     Ok(())
    // }

    /// Retrieves the statistics of a collection.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` with string keys and string values representing the collection statistics.
    pub async fn get_collection_stats(&self, name: &str) -> Result<HashMap<String, String>> {
        let res = self
            .client
            .clone()
            .get_collection_statistics(proto::milvus::GetCollectionStatisticsRequest {
                base: Some(MsgBase::new(MsgType::GetCollectionStatistics)),
                db_name: "".into(),
                collection_name: name.to_owned(),
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res.stats.into_iter().map(|s| (s.key, s.value)).collect())
    }

    /// Loads a collection with the given name and options.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to load.
    /// * `options` - Optional load options.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn load_collection<S>(
        &self,
        collection_name: S,
        options: Option<LoadOptions>,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let options = options.unwrap_or_default();
        let collection_name = collection_name.into();
        status_to_result(&Some(
            self.client
                .clone()
                .load_collection(LoadCollectionRequest {
                    base: Some(MsgBase::new(MsgType::LoadCollection)),
                    db_name: "".to_string(),
                    collection_name: collection_name.clone(),
                    replica_number: options.replica_number,
                    resource_groups: vec![],
                    refresh: false,
                })
                .await?
                .into_inner(),
        ))?;

        loop {
            match self.get_load_state(&collection_name, None).await? {
                proto::common::LoadState::NotExist => {
                    return Err(SuperError::Unexpected("collection not found".to_owned()))
                }
                proto::common::LoadState::Loading => (),
                proto::common::LoadState::Loaded => return Ok(()),
                proto::common::LoadState::NotLoad => {
                    return Err(SuperError::Unexpected("collection not loaded".to_owned()))
                }
            }

            tokio::time::sleep(Duration::from_millis(config::WAIT_LOAD_DURATION_MS)).await;
        }
    }

    /// Retrieves the load state of a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `options` - Optional parameters for retrieving the load state.
    ///
    /// # Returns
    ///
    /// The load state of the collection.
    ///
    /// # Errors
    ///
    /// Returns an error if the load state retrieval fails.
    pub async fn get_load_state<S>(
        &self,
        collection_name: S,
        options: Option<GetLoadStateOptions>,
    ) -> Result<crate::proto::common::LoadState>
    where
        S: Into<String>,
    {
        let options = options.unwrap_or_default();
        let res = self
            .client
            .clone()
            .get_load_state(proto::milvus::GetLoadStateRequest {
                base: Some(MsgBase::new(MsgType::Undefined)),
                db_name: "".into(),
                collection_name: collection_name.into(),
                partition_names: options.partition_names,
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res.state())
    }

    /// Releases a collection with the given name.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to release.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn release_collection<S>(&self, collection_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        status_to_result(&Some(
            self.client
                .clone()
                .release_collection(ReleaseCollectionRequest {
                    base: Some(MsgBase::new(MsgType::ReleaseCollection)),
                    db_name: "".to_string(),
                    collection_name: collection_name.into(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn flush<S>(&self, collection_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(MsgBase::new(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: vec![collection_name.into()],
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(())
    }

    async fn create_index_impl<S>(
        &self,
        collection_name: S,
        field_name: impl Into<String>,
        index_params: IndexParams,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let field_name = field_name.into();
        let status = self
            .client
            .clone()
            .create_index(CreateIndexRequest {
                base: Some(MsgBase::new(MsgType::CreateIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name,
                extra_params: index_params.extra_kv_params(),
                index_name: index_params.name().clone(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(status))
    }

    pub async fn create_index<S>(
        &self,
        collection_name: S,
        field_name: impl Into<String>,
        index_params: IndexParams,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let field_name = field_name.into();
        self.create_index_impl(
            collection_name.clone(),
            field_name.clone(),
            index_params.clone(),
        )
        .await?;

        loop {
            let index_infos = self
                .describe_index(collection_name.clone(), field_name.clone())
                .await?;

            let index_info = index_infos
                .iter()
                .find(|&x| x.params().name() == index_params.name());
            if index_info.is_none() {
                return Err(SuperError::Unexpected(
                    "failed to describe index".to_owned(),
                ));
            }
            match index_info.unwrap().state() {
                IndexState::Finished => return Ok(()),
                IndexState::Failed => return Err(SuperError::Collection(Error::IndexBuildFailed)),
                _ => (),
            };

            tokio::time::sleep(Duration::from_millis(config::WAIT_CREATE_INDEX_DURATION_MS)).await;
        }
    }

    pub async fn describe_index<S>(
        &self,
        collection_name: S,
        field_name: S,
    ) -> Result<Vec<IndexInfo>>
    where
        S: Into<String>,
    {
        let res = self
            .client
            .clone()
            .describe_index(DescribeIndexRequest {
                base: Some(MsgBase::new(MsgType::DescribeIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name: field_name.into(),
                index_name: "".to_string(),
                timestamp: 0,
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;

        Ok(res.index_descriptions.into_iter().map(Into::into).collect())
    }

    pub async fn drop_index<S>(&self, collection_name: S, field_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let status = self
            .client
            .clone()
            .drop_index(DropIndexRequest {
                base: Some(MsgBase::new(MsgType::DropIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name: field_name.into(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(status))
    }

    pub async fn manual_compaction<S>(&self, collection_name: S) -> Result<CompactionInfo>
    where
        S: Into<String>,
    {
        let collection = self.collection_cache.get(&collection_name.into()).await?;

        let resp = self
            .client
            .clone()
            .manual_compaction(ManualCompactionRequest {
                collection_id: collection.id,
                timetravel: 0,
            })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok(resp.into())
    }

    pub async fn get_compaction_state(&self, compaction_id: i64) -> Result<CompactionState> {
        let resp = self
            .client
            .clone()
            .get_compaction_state(GetCompactionStateRequest { compaction_id })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok(resp.into())
    }
}

pub type ParamValue = serde_json::Value;
pub use serde_json::json as ParamValue;

// search result for a single vector
#[derive(Debug)]
pub struct SearchResult<'a> {
    pub size: i64,
    pub id: Vec<Value<'a>>,
    pub field: Vec<FieldColumn>,
    pub score: Vec<f32>,
}

pub struct IndexProgress {
    pub total_rows: i64,
    pub indexed_rows: i64,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("collection {0} not found")]
    CollectionNotFound(String),

    #[error("type mismatched in {0:?}, available types: {1:?}")]
    IllegalType(String, Vec<DataType>),

    #[error("value mismatched in {0:?}, it must be {1:?}")]
    IllegalValue(String, String),

    #[error("index build failed")]
    IndexBuildFailed,
}
