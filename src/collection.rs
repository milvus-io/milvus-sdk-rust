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
use crate::proto::milvus::{
    AlterCollectionFieldRequest, AlterCollectionRequest, CreateCollectionRequest,
    DropCollectionRequest, FlushRequest, GetCompactionStateRequest, GetCompactionStateResponse,
    HasCollectionRequest, LoadCollectionRequest, ManualCompactionRequest, ManualCompactionResponse,
    ReleaseCollectionRequest, ShowCollectionsRequest,
};
use crate::proto::schema::DataType;
use crate::schema::{CollectionSchema, CollectionSchemaBuilder};
use crate::types::*;
use crate::utils::status_to_result;
use crate::value::Value;
use crate::{
    client::{Client, CombinedInterceptor},
    options::{CreateCollectionOptions, GetLoadStateOptions, LoadOptions},
    proto::{
        self,
        common::{ConsistencyLevel, MsgBase, MsgType},
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

/// Return type for describe collection,containing enough messages
pub struct DescribeCollection {
    pub collection_name: String,
    pub collection_id: i64,
    pub shards_num: i32,
    pub aliases: Vec<String>,
    pub consistency_level: i32,
    pub properties: Vec<proto::common::KeyValuePair>,
    pub num_partitions: i64,
    pub schema: crate::proto::schema::CollectionSchema,
}

#[derive(Debug, Clone)]
pub(crate) struct CollectionCache {
    collections: dashmap::DashMap<String, Collection>,
    timestamps: dashmap::DashMap<String, Timestamp>,
    client: MilvusServiceClient<InterceptedService<Channel, CombinedInterceptor>>,
}

impl CollectionCache {
    pub fn new(
        client: MilvusServiceClient<InterceptedService<Channel, CombinedInterceptor>>,
    ) -> Self {
        Self {
            collections: dashmap::DashMap::new(),
            timestamps: dashmap::DashMap::new(),
            client: client,
        }
    }

    pub fn clear(&self) {
        self.collections.clear();
        self.timestamps.clear();
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

//type ConcurrentHashMap<K, V> = tokio::sync::RwLock<std::collections::HashMap<K, V>>;

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
    pub async fn describe_collection<S>(&self, name: S) -> Result<DescribeCollection>
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

        Ok(DescribeCollection {
            collection_name: resp.collection_name,
            collection_id: resp.collection_id,
            shards_num: resp.shards_num,
            aliases: resp.aliases,
            consistency_level: resp.consistency_level,
            properties: resp.properties,
            num_partitions: resp.num_partitions,
            schema: resp.schema.unwrap_or_default(),
        })
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

    pub async fn rename_collection<S>(
        &self,
        name: S,
        new_name: S,
        options: Option<(String, String)>,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let name = name.into();
        let new_name = new_name.into();
        let (db_name, new_db_name) = match options {
            Some((db_name, new_db_name)) => (db_name, new_db_name),
            None => ("".to_string(), "".to_string()),
        };
        let res = self
            .client
            .clone()
            .rename_collection(crate::proto::milvus::RenameCollectionRequest {
                base: Some(MsgBase::new(MsgType::RenameCollection)),
                db_name: db_name,
                old_name: name,
                new_name: new_name,
                new_db_name: new_db_name,
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;

        Ok(())
    }

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
                    resource_groups: options.resource_groups,
                    refresh: options.refresh,
                    load_fields: options.load_fields,
                    skip_load_dynamic_field: options.skip_load_dynamic_field,
                    load_params: options.load_params,
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

    pub async fn refresh_load<S: Into<String>>(&self, collection_name: S) -> Result<()> {
        let options = LoadOptions::new().refresh(true);
        self.load_collection(collection_name, Some(options)).await?;
        Ok(())
    }

    /// Releases a collection with the given name from memory.
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

    /// Alters the field of a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `field_name` - The name of the field to alter.
    /// * `field_params` - A `HashMap` containing the parameters to alter the field.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn alter_collection_field<S>(
        &self,
        collection_name: S,
        field_name: S,
        field_params: HashMap<String, String>,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        //collect field_params into a vec
        let properties: Vec<crate::proto::common::KeyValuePair> = field_params
            .iter()
            .map(|(k, v)| crate::proto::common::KeyValuePair {
                key: k.clone(),
                value: v.clone(),
            })
            .collect();

        let resp = self
            .client
            .clone()
            .alter_collection_field(AlterCollectionFieldRequest {
                base: Some(MsgBase::new(MsgType::AlterCollectionField)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name: field_name.into(),
                properties: properties,
                delete_keys: vec![],
            })
            .await?
            .into_inner();
        status_to_result(&Some(resp))
    }

    /// alter a collection
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of a collection
    /// * `properties` - A HashMap containing the properties need to be alter
    ///
    /// # Returns
    ///
    /// Retrun a `Result` indicating success or failure.
    pub async fn alter_collection_properties<S>(
        &self,
        collection_name: S,
        properties: HashMap<String, String>,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let properties: Vec<crate::proto::common::KeyValuePair> = properties
            .iter()
            .map(|(k, v)| crate::proto::common::KeyValuePair {
                key: k.clone(),
                value: v.clone(),
            })
            .collect();

        let resp = self
            .client
            .clone()
            .alter_collection(AlterCollectionRequest {
                base: Some(MsgBase::new(MsgType::AlterCollection)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                collection_id: 0,
                properties: properties,
                delete_keys: Vec::new(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(resp))
    }

    /// Drop properties of a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `delet_keys` - The keys of the properties to be deleted.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn drop_collection_properties<S>(
        &self,
        collection_name: S,
        delet_keys: Vec<String>,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let resp = self
            .client
            .clone()
            .alter_collection(AlterCollectionRequest {
                base: Some(MsgBase::new(MsgType::AlterCollection)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                collection_id: 0,
                properties: Vec::new(),
                delete_keys: delet_keys,
            })
            .await?
            .into_inner();
        status_to_result(&Some(resp))?;
        Ok(())
    }

    /// create a schema
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the schema
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CollectionSchemaBuilder` if successful, or an error if the schema creation fails.
    pub async fn create_schema(&self, name: &str) -> Result<CollectionSchemaBuilder> {
        let schema = CollectionSchemaBuilder::new(name, "");
        Ok(schema)
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

    /// manual compaction
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection
    /// * `is_clustering` - Whether to perform clustering compaction
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CompactionInfo` if successful, or an error if the compaction fails.
    pub async fn manual_compaction<S>(
        &self,
        collection_name: S,
        is_clustering: Option<bool>,
    ) -> Result<CompactionInfo>
    where
        S: Into<String>,
    {
        let collection = self.collection_cache.get(&collection_name.into()).await?;
        let major_compaction = is_clustering.unwrap_or(false);

        let resp = self
            .client
            .clone()
            .manual_compaction(ManualCompactionRequest {
                collection_id: collection.id,
                timetravel: 0,
                major_compaction,
                collection_name: collection.name,
                db_name: "".to_string(),
                partition_id: 0,
                segment_ids: vec![],
                channel: "".to_string(),
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
#[derive(Clone, Debug)]
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
