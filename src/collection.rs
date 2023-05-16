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

use crate::{error::{Error as SuperError, Result}, proto::milvus::{DropAliasRequest, AlterAliasRequest}};
use crate::index::{IndexInfo, IndexParams, MetricType};
use crate::proto::common::{
    ConsistencyLevel, DslType, IndexState, KeyValuePair, MsgType, PlaceholderGroup,
    PlaceholderType, PlaceholderValue,
};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    CreateCollectionRequest, CreateIndexRequest, CreatePartitionRequest,
    DescribeCollectionResponse, DescribeIndexRequest, DropCollectionRequest, DropIndexRequest,
    FlushRequest, HasCollectionRequest, HasPartitionRequest, InsertRequest, LoadCollectionRequest,
    QueryRequest, ReleaseCollectionRequest, SearchRequest, ShowCollectionsRequest,
    ShowPartitionsRequest, ShowType,
};
use crate::proto::schema::i_ds::IdField::{IntId, StrId};
use crate::proto::schema::DataType;
use crate::schema::CollectionSchema;
use crate::types::*;
use crate::utils::{new_msg, status_to_result};
use crate::value::Value;
use crate::{client::AuthInterceptor, proto::milvus::CalcDistanceRequest};
use crate::{config, proto::milvus::DeleteRequest};
use crate::{data::FieldColumn, proto::milvus::CreateAliasRequest};

use prost::bytes::BytesMut;
use prost::Message;
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio::sync::Mutex;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

const STRONG_TIMESTAMP: u64 = 0;
const BOUNDED_TIMESTAMP: u64 = 2;
const EVENTUALLY_TIMESTAMP: u64 = 1;

#[derive(Debug)]
pub struct Partition {
    pub name: String,
    pub percentage: i64,
}

type ConcurrentHashMap<K, V> = tokio::sync::RwLock<std::collections::HashMap<K, V>>;

#[derive(Debug)]
pub struct Collection {
    client: MilvusServiceClient<InterceptedService<Channel, AuthInterceptor>>,
    info: DescribeCollectionResponse,
    schema: CollectionSchema,
    partitions: Mutex<HashSet<String>>,
    session_timestamps: ConcurrentHashMap<String, Timestamp>,
}

impl Collection {
    pub fn new(
        client: MilvusServiceClient<InterceptedService<Channel, AuthInterceptor>>,
        info: DescribeCollectionResponse,
    ) -> Self {
        let schema = info.schema.clone().unwrap();
        Self {
            client,
            info: info,
            schema: schema.into(),
            partitions: Mutex::new(Default::default()),
            session_timestamps: ConcurrentHashMap::new(HashMap::new()),
        }
    }

    pub fn schema(&self) -> &CollectionSchema {
        &self.schema
    }

    pub async fn get_load_percent(&self) -> Result<i64> {
        let response = self
            .client
            .clone()
            .show_collections(ShowCollectionsRequest {
                base: Some(new_msg(MsgType::ShowCollections)),
                db_name: "".to_string(),
                time_stamp: 0,
                r#type: ShowType::InMemory as i32,
                collection_names: vec![self.schema().name.to_string()],
            })
            .await?
            .into_inner();

        status_to_result(&response.status)?;

        let names = response.collection_names;
        let percent = response.in_memory_percentages;
        for i in 0..names.len() {
            if self.schema().name == names[i] {
                return Ok(percent[i]);
            }
        }

        Err(SuperError::Unexpected(
            "collection not exist in response".to_owned(),
        ))
    }

    // load_blocked loads collection and returns when loading done
    pub async fn load(&self, replica_number: i32) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .load_collection(LoadCollectionRequest {
                    base: Some(new_msg(MsgType::LoadCollection)),
                    db_name: "".to_string(),
                    collection_name: self.schema().name.clone(),
                    replica_number,
                })
                .await?
                .into_inner(),
        ))?;

        loop {
            if self.get_load_percent().await? >= 100 {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(config::WAIT_LOAD_DURATION_MS)).await;
        }
    }

    /// Create a collection alias
    pub async fn create_alias<S: ToString>(&self, alias: S) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .create_alias(CreateAliasRequest {
                    base: Some(new_msg(MsgType::CreateAlias)),
                    db_name: "".to_string(),
                    collection_name: self.schema().name.to_string(),
                    alias: alias.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    /// Drop a collection alias
    pub async fn drop_alias<S: ToString>(&self, alias: S) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .drop_alias(DropAliasRequest {
                    base: Some(new_msg(MsgType::DropAlias)),
                    db_name: "".to_string(),
                    alias: alias.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    /// Alter a collection alias
    pub async fn alter_alias<S: ToString>(&self, alias: S) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .alter_alias(AlterAliasRequest {
                    base: Some(new_msg(MsgType::AlterAlias)),
                    db_name: "".to_string(),
                    collection_name: self.schema().name.to_string(),
                    alias: alias.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn is_loaded(&self) -> Result<bool> {
        Ok(self.get_load_percent().await? >= 100)
    }

    pub async fn release(&self) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .release_collection(ReleaseCollectionRequest {
                    base: Some(new_msg(MsgType::ReleaseCollection)),
                    db_name: "".to_string(),
                    collection_name: self.schema().name.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn delete<S: AsRef<str>>(&self, expr: S, partition_name: Option<&str>) -> Result<()> {
        status_to_result(
            &self
                .client
                .clone()
                .delete(DeleteRequest {
                    base: Some(new_msg(MsgType::Delete)),
                    db_name: "".to_string(),
                    collection_name: self.schema.name.to_string(),
                    partition_name: partition_name.unwrap_or_default().to_owned(),
                    expr: expr.as_ref().to_owned(),
                    hash_keys: vec![],
                })
                .await?
                .into_inner()
                .status,
        )
    }

    pub async fn drop(&self) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .drop_collection(DropCollectionRequest {
                    base: Some(new_msg(MsgType::DropCollection)),
                    db_name: "".to_string(),
                    collection_name: self.schema().name.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn exist(&self) -> Result<bool> {
        let res = self
            .client
            .clone()
            .has_collection(HasCollectionRequest {
                base: Some(new_msg(MsgType::HasCollection)),
                db_name: "".to_string(),
                collection_name: self.schema().name.clone(),
                time_stamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res.value)
    }

    pub async fn flush(&self) -> Result<()> {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(new_msg(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: vec![self.schema().name.to_string()],
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(())
    }

    pub async fn load_partition_list(&self) -> Result<()> {
        let res = self
            .client
            .clone()
            .show_partitions(ShowPartitionsRequest {
                base: Some(new_msg(MsgType::ShowPartitions)),
                db_name: "".to_string(),
                collection_name: self.schema().name.to_string(),
                collection_id: 0,
                partition_names: Vec::new(),
                r#type: 0,
            })
            .await?
            .into_inner();

        let mut partitions = HashSet::new();
        for name in res.partition_names {
            partitions.insert(name);
        }

        std::mem::swap(&mut *self.partitions.lock().await, &mut partitions);

        status_to_result(&res.status)?;

        Ok(())
    }

    pub async fn create_partition<S: AsRef<str>>(&self, name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .create_partition(CreatePartitionRequest {
                base: Some(new_msg(MsgType::ShowPartitions)),
                db_name: "".to_string(),
                collection_name: self.schema().name.to_string(),
                partition_name: name.as_ref().to_owned(),
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;

        Ok(())
    }

    pub async fn has_partition<P: AsRef<str>>(&self, p: P) -> Result<bool> {
        if self.partitions.lock().await.contains(p.as_ref()) {
            return Ok(true);
        } else {
            let res = self
                .client
                .clone()
                .has_partition(HasPartitionRequest {
                    base: Some(new_msg(MsgType::HasPartition)),
                    db_name: "".to_string(),
                    collection_name: self.schema().name.to_string(),
                    partition_name: p.as_ref().to_string(),
                })
                .await?
                .into_inner();

            status_to_result(&res.status)?;

            Ok(res.value)
        }
    }

    pub async fn create(
        &self,
        shards_num: Option<i32>,
        consistency_level: Option<ConsistencyLevel>,
    ) -> Result<()> {
        let schema: crate::proto::schema::CollectionSchema = self.schema().clone().into();

        let mut buf = BytesMut::new();
        schema.encode(&mut buf)?;

        let status = self
            .client
            .clone()
            .create_collection(CreateCollectionRequest {
                base: Some(new_msg(MsgType::CreateCollection)),
                db_name: "".to_string(),
                collection_name: self.schema().name.to_string(),
                schema: buf.to_vec(),
                shards_num: shards_num.unwrap_or(1),
                consistency_level: consistency_level.unwrap_or(ConsistencyLevel::Session) as i32,
                properties: vec![],
            })
            .await?
            .into_inner();

        status_to_result(&Some(status))
    }

    pub async fn query<Exp, P>(&self, expr: Exp, partition_names: P) -> Result<Vec<FieldColumn>>
    where
        Exp: Into<String>,
        P: IntoIterator,
        P::Item: Into<String>,
    {
        let consistency_level = self.info.consistency_level();

        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(new_msg(MsgType::Retrieve)),
                db_name: "".to_owned(),
                collection_name: self.schema().name.clone(),
                expr: expr.into(),
                output_fields: self
                    .schema()
                    .fields
                    .iter()
                    .map(|f| f.name.clone())
                    .collect(),
                partition_names: partition_names.into_iter().map(|x| x.into()).collect(),
                travel_timestamp: 0,
                guarantee_timestamp: self.get_gts_from_consistency(consistency_level).await,
                query_params: Vec::new(),
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res
            .fields_data
            .into_iter()
            .map(|f| FieldColumn::from(f))
            .collect())
    }

    pub async fn insert(
        &self,
        fields_data: Vec<FieldColumn>,
        partition_name: Option<&str>,
    ) -> Result<crate::proto::milvus::MutationResult> {
        let partition_name = partition_name.unwrap_or("_default").to_owned();
        let row_num = fields_data.first().map(|c| c.len()).unwrap_or(0);

        let result = self
            .client
            .clone()
            .insert(InsertRequest {
                base: Some(new_msg(MsgType::Insert)),
                db_name: "".to_string(),
                collection_name: self.schema().name.to_string(),
                partition_name,
                num_rows: row_num as u32,
                fields_data: fields_data.into_iter().map(|f| f.into()).collect(),
                hash_keys: Vec::new(),
            })
            .await?
            .into_inner();

        self.session_timestamps
            .write()
            .await
            .entry(self.info.collection_name.clone())
            .and_modify(|ts| {
                if *ts < result.timestamp {
                    *ts = result.timestamp;
                }
            })
            .or_insert(result.timestamp);

        Ok(result)
    }

    pub async fn search<S, I>(
        &self,
        data: Vec<Value<'_>>,
        vec_field: S,
        topk: i32,
        metric_type: MetricType,
        output_fields: I,
        option: &SearchOption,
    ) -> Result<Vec<SearchResult<'_>>>
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: Into<String>,
    {
        // check and prepare params
        if topk <= 0 {
            return Err(SuperError::from(Error::IllegalValue(
                "topk".to_owned(),
                "positive".to_owned(),
            )));
        }

        let search_params: Vec<KeyValuePair> = vec![
            KeyValuePair {
                key: "anns_field".to_owned(),
                value: vec_field.into(),
            },
            KeyValuePair {
                key: "topk".to_owned(),
                value: topk.to_string(),
            },
            KeyValuePair {
                key: "params".to_owned(),
                value: serde_json::to_string(&option.params)?,
            },
            KeyValuePair {
                key: "metric_type".to_owned(),
                value: metric_type.to_string(),
            },
            KeyValuePair {
                key: "round_decimal".to_owned(),
                value: "-1".to_owned(),
            },
        ];

        let mut consistency_level = self.info.consistency_level();
        if let Some(level) = option.consistency_level {
            consistency_level = level;
        }

        let res = self
            .client
            .clone()
            .search(SearchRequest {
                base: Some(new_msg(MsgType::Search)),
                db_name: "".to_string(),
                collection_name: self.schema().name.clone(),
                partition_names: option.partitions.clone().unwrap_or_default(),
                dsl: option.expr.clone().unwrap_or_default(),
                nq: data.len() as _,
                placeholder_group: get_place_holder_group(data)?,
                dsl_type: DslType::BoolExprV1 as _,
                output_fields: output_fields.into_iter().map(|f| f.into()).collect(),
                search_params,
                travel_timestamp: 0,
                guarantee_timestamp: self.get_gts_from_consistency(consistency_level).await,
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        let raw_data = res
            .results
            .ok_or(SuperError::Unexpected("no result for search".to_owned()))?;
        let mut result = Vec::new();
        let mut offset = 0;
        let fields_data = raw_data
            .fields_data
            .into_iter()
            .map(Into::into)
            .collect::<Vec<FieldColumn>>();
        let raw_id = raw_data.ids.unwrap().id_field.unwrap();

        for k in raw_data.topks {
            let k = k as usize;
            let mut score = Vec::new();
            score.extend_from_slice(&raw_data.scores[offset..offset + k]);
            let mut result_data = fields_data
                .iter()
                .map(FieldColumn::copy_with_metadata)
                .collect::<Vec<FieldColumn>>();
            for j in 0..fields_data.len() {
                for i in offset..offset + k {
                    result_data[j].push(fields_data[j].get(i).ok_or(SuperError::Unexpected(
                        "out of range while indexing field data".to_owned(),
                    ))?);
                }
            }

            let id = match raw_id {
                IntId(ref d) => {
                    Vec::<Value>::from_iter(d.data[offset..offset + k].iter().map(|&x| x.into()))
                }
                StrId(ref d) => Vec::<Value>::from_iter(
                    d.data[offset..offset + k].iter().map(|x| x.clone().into()),
                ),
            };

            result.push(SearchResult {
                size: k as i64,
                score,
                field: result_data,
                id,
            });

            offset += k;
        }

        Ok(result)
    }

    async fn create_index_impl(
        &self,
        field_name: impl Into<String>,
        index_params: IndexParams,
    ) -> Result<()> {
        let field_name = field_name.into();
        self.schema().is_valid_vector_field(&field_name)?;
        let status = self
            .client
            .clone()
            .create_index(CreateIndexRequest {
                base: Some(new_msg(MsgType::CreateIndex)),
                db_name: "".to_string(),
                collection_name: self.schema().name.clone(),
                field_name,
                extra_params: index_params.extra_kv_params(),
                index_name: index_params.name().clone(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(status))
    }

    pub async fn create_index(
        &self,
        field_name: impl Into<String>,
        index_params: IndexParams,
    ) -> Result<()> {
        let field_name = field_name.into();
        self.create_index_impl(field_name.clone(), index_params.clone())
            .await?;

        loop {
            let index_infos = self.describe_index(field_name.clone()).await?;

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

    pub async fn describe_index<S>(&self, field_name: S) -> Result<Vec<IndexInfo>>
    where
        S: Into<String>,
    {
        let res = self
            .client
            .clone()
            .describe_index(DescribeIndexRequest {
                base: Some(new_msg(MsgType::DescribeIndex)),
                db_name: "".to_string(),
                collection_name: self.schema().name.clone(),
                field_name: field_name.into(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;

        Ok(res.index_descriptions.into_iter().map(Into::into).collect())
    }

    pub async fn drop_index<S>(&self, field_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let status = self
            .client
            .clone()
            .drop_index(DropIndexRequest {
                base: Some(new_msg(MsgType::DropIndex)),
                db_name: "".to_string(),
                collection_name: self.schema().name.clone(),
                field_name: field_name.into(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(status))
    }

    async fn get_gts_from_consistency(&self, consistency_level: ConsistencyLevel) -> u64 {
        match consistency_level {
            ConsistencyLevel::Strong => STRONG_TIMESTAMP,
            ConsistencyLevel::Bounded => BOUNDED_TIMESTAMP,
            ConsistencyLevel::Eventually => EVENTUALLY_TIMESTAMP,
            ConsistencyLevel::Session => *self
                .session_timestamps
                .read()
                .await
                .get(&self.info.collection_name)
                .unwrap_or(&EVENTUALLY_TIMESTAMP),

            // This level not works for now
            ConsistencyLevel::Customized => 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct SearchOption {
    expr: Option<String>,
    partitions: Option<Vec<String>>,
    params: HashMap<String, String>,
    consistency_level: Option<ConsistencyLevel>,
}

impl SearchOption {
    pub fn new() -> Self {
        Self {
            expr: None,
            partitions: None,
            params: HashMap::new(),
            consistency_level: None,
        }
    }

    pub fn set_expr<S: Into<String>>(&mut self, expr: S) -> &mut Self {
        self.expr = Some(expr.into());
        self
    }

    pub fn set_partitions<I>(&mut self, partitions: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.partitions = Some(partitions.into_iter().map(Into::into).collect());
        self
    }

    pub fn add_param<S: Into<String>>(&mut self, key: S, value: S) -> &mut Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn set_consistency_level(&mut self, level: ConsistencyLevel) -> &mut Self {
        self.consistency_level = Some(level);
        self
    }
}

// search result for a single vector
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
    #[error("type mismatched in {0:?}, available types: {1:?}")]
    IllegalType(String, Vec<DataType>),

    #[error("value mismatched in {0:?}, it must be {1:?}")]
    IllegalValue(String, String),

    #[error("index build failed")]
    IndexBuildFailed,
}

fn get_place_holder_group(vectors: Vec<Value>) -> Result<Vec<u8>> {
    let group = PlaceholderGroup {
        placeholders: vec![get_place_holder_value(vectors)?],
    };
    let mut buf = BytesMut::new();
    group.encode(&mut buf).unwrap();
    return Ok(buf.to_vec());
}

fn get_place_holder_value(vectors: Vec<Value>) -> Result<PlaceholderValue> {
    let mut place_holder = PlaceholderValue {
        tag: "$0".to_string(),
        r#type: PlaceholderType::None as _,
        values: Vec::new(),
    };
    // if no vectors, return an empty one
    if vectors.len() == 0 {
        return Ok(place_holder);
    };

    match vectors[0] {
        Value::FloatArray(_) => place_holder.r#type = PlaceholderType::FloatVector as _,
        Value::Binary(_) => place_holder.r#type = PlaceholderType::BinaryVector as _,
        _ => {
            return Err(SuperError::from(Error::IllegalType(
                "place holder".to_string(),
                vec![DataType::BinaryVector, DataType::FloatVector],
            )))
        }
    };

    for v in &vectors {
        match (v, &vectors[0]) {
            (Value::FloatArray(d), Value::FloatArray(_)) => {
                let mut bytes = Vec::<u8>::with_capacity(d.len() * 4);
                for f in d.iter() {
                    bytes.extend_from_slice(&f.to_le_bytes());
                }
                place_holder.values.push(bytes)
            }
            (Value::Binary(d), Value::Binary(_)) => place_holder.values.push(d.to_vec()),
            _ => {
                return Err(SuperError::from(Error::IllegalType(
                    "place holder".to_string(),
                    vec![DataType::BinaryVector, DataType::FloatVector],
                )))
            }
        };
    }
    return Ok(place_holder);
}
