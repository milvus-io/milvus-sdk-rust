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

use crate::data::FieldColumn;
use crate::error::{Error as SuperError, Result};
use crate::proto::common::{
    ConsistencyLevel, DslType, IndexState, KeyValuePair, MsgType, PlaceholderGroup,
    PlaceholderType, PlaceholderValue,
};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    CreateCollectionRequest, CreateIndexRequest, CreatePartitionRequest, DescribeIndexRequest,
    DropCollectionRequest, DropIndexRequest, FlushRequest, GetIndexBuildProgressRequest,
    GetIndexStateRequest, HasCollectionRequest, HasPartitionRequest, InsertRequest,
    LoadCollectionRequest, QueryRequest, ReleaseCollectionRequest, SearchRequest,
    ShowCollectionsRequest, ShowPartitionsRequest, ShowType,
};
use crate::proto::schema::i_ds::IdField::{IntId, StrId};
use crate::proto::schema::DataType;
use crate::schema::CollectionSchema;
use crate::utils::{new_msg, status_to_result};
use crate::value::{Value, ValueVec};
use crate::{config, proto, schema};

use prost::bytes::BytesMut;
use prost::Message;
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio::sync::Mutex;
use tonic::transport::Channel;

#[derive(Debug)]
pub struct Partition {
    pub name: String,
    pub percentage: i64,
}

#[derive(Debug)]
pub struct Collection {
    client: MilvusServiceClient<Channel>,
    partitions: Mutex<HashSet<String>>,
    schema: CollectionSchema,
}

impl Collection {
    pub fn new(client: MilvusServiceClient<Channel>, schema: CollectionSchema) -> Self {
        Self {
            client,
            partitions: Mutex::new(Default::default()),
            schema: schema,
        }
    }

    async fn load(&self, replica_number: i32) -> Result<()> {
        status_to_result(Some(
            self.client
                .clone()
                .load_collection(LoadCollectionRequest {
                    base: Some(new_msg(MsgType::LoadCollection)),
                    db_name: "".to_string(),
                    collection_name: self.schema.name.clone(),
                    replica_number,
                })
                .await?
                .into_inner(),
        ))
    }

    pub fn schema(&self) -> &CollectionSchema {
        &self.schema
    }

    // load_unblocked loads collection and returns when request committed
    pub async fn load_unblocked(&self, replica_number: i32) -> Result<()> {
        dbg!("start load_unblocked");
        // TODO wrap the error
        // let rt = Builder::new_current_thread().enable_all().build().unwrap();
        // rt.block_on(self.load(replica_number))
        self.load(replica_number).await
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
                collection_names: vec![self.schema.name.to_string()],
            })
            .await?
            .into_inner();

        status_to_result(response.status)?;

        let names = response.collection_names;
        let percent = response.in_memory_percentages;
        for i in 0..names.len() {
            if self.schema.name == names[i] {
                return Ok(percent[i]);
            }
        }

        Err(SuperError::Unknown)
    }

    // load_blocked loads collection and returns when loading done
    pub async fn load_blocked(&self, replica_number: i32) -> Result<()> {
        self.load(replica_number).await?;

        loop {
            if self.get_load_percent().await? >= 100 {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(config::WAIT_LOAD_DURATION_MS)).await;
        }
    }

    pub async fn is_loaded(&self) -> Result<bool> {
        Ok(self.get_load_percent().await? >= 100)
    }

    pub async fn release(&self) -> Result<()> {
        status_to_result(Some(
            self.client
                .clone()
                .release_collection(ReleaseCollectionRequest {
                    base: Some(new_msg(MsgType::ReleaseCollection)),
                    db_name: "".to_string(),
                    collection_name: self.schema.name.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn drop(&self) -> Result<()> {
        status_to_result(Some(
            self.client
                .clone()
                .drop_collection(DropCollectionRequest {
                    base: Some(new_msg(MsgType::DropCollection)),
                    db_name: "".to_string(),
                    collection_name: self.schema.name.to_string(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn exists(&self) -> Result<bool> {
        let res = self
            .client
            .clone()
            .has_collection(HasCollectionRequest {
                base: Some(new_msg(MsgType::HasCollection)),
                db_name: "".to_string(),
                collection_name: self.schema.name.to_string(),
                time_stamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(res.status)?;

        Ok(res.value)
    }

    pub async fn flush(&self) -> Result<()> {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(new_msg(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: vec![self.schema.name.to_string()],
            })
            .await?
            .into_inner();

        status_to_result(res.status)?;

        Ok(())
    }

    pub async fn load_partition_list(&self) -> Result<()> {
        let res = self
            .client
            .clone()
            .show_partitions(ShowPartitionsRequest {
                base: Some(new_msg(MsgType::ShowPartitions)),
                db_name: "".to_string(),
                collection_name: self.schema.name.to_string(),
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

        status_to_result(res.status)?;

        Ok(())
    }

    pub async fn create_partition<S: AsRef<str>>(&self, name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .create_partition(CreatePartitionRequest {
                base: Some(new_msg(MsgType::ShowPartitions)),
                db_name: "".to_string(),
                collection_name: self.schema.name.to_string(),
                partition_name: name.as_ref().to_owned(),
            })
            .await?
            .into_inner();

        status_to_result(Some(res))?;

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
                    collection_name: self.schema.name.to_string(),
                    partition_name: p.as_ref().to_string(),
                })
                .await?
                .into_inner();

            status_to_result(res.status)?;

            Ok(res.value)
        }
    }

    pub async fn create(
        &self,
        shards_num: Option<i32>,
        consistency_level: Option<ConsistencyLevel>,
    ) -> Result<()> {
        let schema: crate::proto::schema::CollectionSchema = self.schema.clone().into();

        let mut buf = BytesMut::new();
        schema.encode(&mut buf)?;

        let status = self
            .client
            .clone()
            .create_collection(CreateCollectionRequest {
                base: Some(new_msg(MsgType::CreateCollection)),
                db_name: "".to_string(),
                collection_name: self.schema.name.to_string(),
                schema: buf.to_vec(),
                shards_num: shards_num.unwrap_or(1),
                consistency_level: consistency_level.unwrap_or(ConsistencyLevel::Session) as i32,
            })
            .await?
            .into_inner();

        status_to_result(Some(status))
    }

    pub async fn query<Exp, P>(&self, expr: Exp, partition_names: P) -> Result<Vec<FieldColumn>>
    where
        Exp: ToString,
        P: IntoIterator,
        P::Item: ToString,
    {
        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(new_msg(MsgType::Retrieve)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                expr: expr.to_string(),
                output_fields: self.schema.fields.iter().map(|f| f.name.clone()).collect(),
                partition_names: partition_names.into_iter().map(|x| x.to_string()).collect(),
                travel_timestamp: 0,
                guarantee_timestamp: 0,
                query_params: Vec::new(),
                consistency_level: 0,
                use_default_consistency: true,
            })
            .await?
            .into_inner();

        status_to_result(res.status)?;

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

        Ok(self
            .client
            .clone()
            .insert(InsertRequest {
                base: Some(new_msg(MsgType::Insert)),
                db_name: "".to_string(),
                collection_name: self.schema.name.to_string(),
                partition_name,
                num_rows: row_num as u32,
                fields_data: fields_data.into_iter().map(|f| f.into()).collect(),
                hash_keys: Vec::new(),
            })
            .await?
            .into_inner())
    }

    pub async fn search<S, I>(
        &self,
        data: Vec<Value<'_>>,
        vec_field: S,
        top_k: i32,
        expr: Option<S>,
        partition_names: I,
        metric_type: MetricType,
        output_fields: I,
        params: HashMap<String, String>,
        consistency_level: Option<ConsistencyLevel>,
    ) -> Result<Vec<SearchResult<'_>>>
    where
        S: ToString,
        I: IntoIterator,
        I::Item: ToString,
    {
        // check and prepare params

        if top_k <= 0 {
            return Err(SuperError::from(Error::IllegalValue(
                "top_k".to_string(),
                "positive".to_string(),
            )));
        }

        let mut search_params = Vec::new();
        search_params.push(KeyValuePair {
            key: "anns_field".to_string(),
            value: vec_field.to_string(),
        });
        search_params.push(KeyValuePair {
            key: "topk".to_string(),
            value: format!("{top_k}").to_string(),
        });
        search_params.push(KeyValuePair {
            key: "params".to_string(),
            value: serde_json::to_string(&params).unwrap(),
        });
        search_params.push(KeyValuePair {
            key: "metric_type".to_string(),
            value: metric_type.to_string(),
        });
        search_params.push(KeyValuePair {
            key: "round_demical".to_string(),
            value: "-1".to_string(),
        });
        let res = self
            .client
            .clone()
            .search(SearchRequest {
                base: Some(new_msg(MsgType::Search)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                partition_names: partition_names.into_iter().map(|x| x.to_string()).collect(),
                dsl: expr.map_or("".to_string(), |x| x.to_string()),
                nq: data.len() as _,
                placeholder_group: get_place_holder_group(data)?,
                dsl_type: DslType::BoolExprV1 as _,
                output_fields: output_fields.into_iter().map(|f| f.to_string()).collect(),
                search_params,
                travel_timestamp: 0,
                guarantee_timestamp: 0,
                consistency_level: consistency_level.unwrap_or_default() as _,
                use_default_consistency: true,
            })
            .await?
            .into_inner();
        status_to_result(res.status)?;
        let raw_data = res.results.ok_or(SuperError::Unknown)?;
        let mut result = Vec::new();
        let mut offset = 0;
        let fields_data = raw_data
            .fields_data
            .into_iter()
            .map(Into::into)
            .collect::<Vec<FieldColumn>>();
        let raw_id = raw_data.ids.unwrap().id_field.unwrap();

        for k in raw_data.topks {
            let mut score = Vec::new();
            for i in offset..offset + k {
                score.push(raw_data.scores[i as usize]);
            }
            let mut result_data = fields_data
                .iter()
                .map(FieldColumn::copy_with_metadata)
                .collect::<Vec<FieldColumn>>();
            for j in 0..fields_data.len() {
                for i in offset..offset + k {
                    result_data[j].push(fields_data[j].get(i as _).ok_or(SuperError::Unknown)?);
                }
            }

            let id = match raw_id {
                IntId(ref d) => {
                    let mut tmp_id = Vec::<Value>::new();
                    for i in offset..offset + k {
                        tmp_id.push(d.data[i as usize].into());
                    }
                    tmp_id
                }
                StrId(ref d) => {
                    let mut tmp_id = Vec::<Value>::new();
                    for i in offset..offset + k {
                        tmp_id.push(d.data[i as usize].clone().into());
                    }
                    tmp_id
                }
            };

            result.push(SearchResult {
                size: k,
                score,
                field: result_data,
                id,
            });

            offset += k;
        }

        Ok(result)
    }

    pub async fn create_index_unblocked<S>(
        &self,
        field_name: S,
        params: HashMap<String, String>,
    ) -> Result<()>
    where
        S: ToString,
    {
        let mut extra_params = HashMap::new();
        let mut params = params.clone();
        extra_params.insert(
            "index_type",
            params.remove("index_type").ok_or(SuperError::Unknown)?,
        );
        extra_params.insert(
            "metric_type",
            params.remove("metric_type").ok_or(SuperError::Unknown)?,
        );
        extra_params.insert("params", serde_json::to_string(&params).unwrap());

        let field_name = field_name.to_string();
        self.schema.is_valid_vector_field(field_name.clone())?;
        let status = self
            .client
            .clone()
            .create_index(CreateIndexRequest {
                base: Some(new_msg(MsgType::CreateIndex)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                field_name,
                extra_params: extra_params
                    .into_iter()
                    .map(|(key, value)| KeyValuePair {
                        key: key.to_string(),
                        value,
                    })
                    .collect::<Vec<KeyValuePair>>(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(Some(status))
    }

    pub async fn get_index_state<S>(&self, field_name: S) -> Result<IndexState>
    where
        S: ToString,
    {
        let res = self
            .client
            .clone()
            .get_index_state(GetIndexStateRequest {
                base: Some(new_msg(MsgType::GetIndexState)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                field_name: field_name.to_string(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(res.status)?;
        Ok(IndexState::from_i32(res.state).unwrap())
    }
    pub async fn create_index_blocked<S>(
        &self,
        field_name: S,
        params: HashMap<String, String>,
    ) -> Result<()>
    where
        S: ToString,
    {
        let field_name = field_name.to_string();
        self.create_index_unblocked(field_name.clone(), params)
            .await?;
        loop {
            match self.get_index_state(field_name.clone()).await? {
                IndexState::Finished => return Ok(()),
                IndexState::Failed => return Err(SuperError::from(Error::IndexBuildFailed)),
                _ => (),
            }

            tokio::time::sleep(Duration::from_millis(config::WAIT_CREATE_INDEX_DURATION_MS)).await;
        }
    }

    pub async fn describe_index<S>(&self, field_name: S) -> Result<Vec<IndexInfo>>
    where
        S: ToString,
    {
        let res = self
            .client
            .clone()
            .describe_index(DescribeIndexRequest {
                base: Some(new_msg(MsgType::DescribeIndex)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                field_name: field_name.to_string(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(res.status)?;

        Ok(res
            .index_descriptions
            .into_iter()
            .map(|x| IndexInfo::new(x.index_name, x.params))
            .collect())
    }

    pub async fn get_index_build_progress<S>(&self, field_name: S) -> Result<IndexProgress>
    where
        S: ToString,
    {
        let res = self
            .client
            .clone()
            .get_index_build_progress(GetIndexBuildProgressRequest {
                base: Some(new_msg(MsgType::GetIndexBuildProgress)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                field_name: field_name.to_string(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(res.status)?;
        Ok(IndexProgress {
            total_rows: res.total_rows,
            indexed_rows: res.indexed_rows,
        })
    }

    pub async fn drop_index<S>(&self, field_name: S) -> Result<()>
    where
        S: ToString,
    {
        let status = self
            .client
            .clone()
            .drop_index(DropIndexRequest {
                base: Some(new_msg(MsgType::DropIndex)),
                db_name: "".to_string(),
                collection_name: self.schema.name.clone(),
                field_name: field_name.to_string(),
                index_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(Some(status))
    }
}

pub enum MetricType {
    L2,
    IP,
    HAMMING,
    JACCARD,
    TANIMOTO,
    SUBSTRUCTURE,
    SUPERSTRUCTURE,
}

impl ToString for MetricType {
    fn to_string(&self) -> String {
        match self {
            MetricType::L2 => "L2",
            MetricType::IP => "IP",
            MetricType::HAMMING => "HAMMING",
            MetricType::JACCARD => "JACCARD",
            MetricType::TANIMOTO => "TANIMOTO",
            MetricType::SUBSTRUCTURE => "SUBSTRUCTURE",
            MetricType::SUPERSTRUCTURE => "SUPERSTRUCTURE",
        }
        .to_string()
    }
}

// search result for a single vector
pub struct SearchResult<'a> {
    pub size: i64,
    pub id: Vec<Value<'a>>,
    pub field: Vec<FieldColumn>,
    pub score: Vec<f32>,
}

pub struct IndexInfo {
    pub name: String,
    pub params: HashMap<String, String>,
}

impl IndexInfo {
    pub fn new<S>(name: S, params: Vec<KeyValuePair>) -> Self
    where
        S: ToString,
    {
        let mut p = HashMap::new();
        for kv in params.into_iter() {
            if kv.key == "index_type".to_string() {
                p.insert(kv.key, kv.value);
            } else if kv.key == "metric_type".to_string() {
                p.insert(kv.key, kv.value);
            } else {
                let map: HashMap<String, String> = serde_json::from_str(&kv.value).unwrap();
                p.extend(map);
            }
        }
        Self {
            name: name.to_string(),
            params: p,
        }
    }
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
