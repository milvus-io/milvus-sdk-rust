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
use crate::proto::common::{ConsistencyLevel, MsgType};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    CreateCollectionRequest, CreatePartitionRequest, DropCollectionRequest, FlushRequest,
    HasCollectionRequest, HasPartitionRequest, InsertRequest, LoadCollectionRequest, QueryRequest,
    ReleaseCollectionRequest, ShowCollectionsRequest, ShowPartitionsRequest, ShowType,
};
use crate::schema::CollectionSchema;
use crate::utils::{new_msg, status_to_result};
use crate::{config, proto, schema};

use prost::bytes::BytesMut;
use prost::Message;
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
}

#[derive(Debug, ThisError)]
pub enum Error {
    // TODO
}
