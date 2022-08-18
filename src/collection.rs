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

use crate::error::{Error as SuperError, Result};
use crate::proto::common::{ConsistencyLevel, MsgType};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    CreateCollectionRequest, CreatePartitionRequest, DropCollectionRequest, FlushRequest,
    HasCollectionRequest, HasPartitionRequest, InsertRequest, LoadCollectionRequest, QueryRequest,
    ReleaseCollectionRequest, ShowCollectionsRequest, ShowPartitionsRequest, ShowType,
};
use crate::utils::{new_msg, status_to_result};
use crate::{config, schema};

use prost::bytes::BytesMut;
use prost::Message;
use std::borrow::Cow;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio::sync::Mutex;
use tonic::transport::Channel;

#[derive(Debug)]
pub struct Partition {
    pub name: Cow<'static, str>,
    pub percentage: i64,
}

#[derive(Debug)]
pub struct Collection<C> {
    client: MilvusServiceClient<Channel>,
    name: Cow<'static, str>,
    partitions: Mutex<HashSet<String>>,
    _m: PhantomData<C>,
}

impl<C> Collection<C> {
    pub fn new<N: Into<Cow<'static, str>>>(client: MilvusServiceClient<Channel>, name: N) -> Self {
        Self {
            client,
            name: name.into(),
            partitions: Mutex::new(Default::default()),
            _m: Default::default(),
        }
    }

    async fn load(&self, replica_number: i32) -> Result<()> {
        status_to_result(Some(
            self.client
                .clone()
                .load_collection(LoadCollectionRequest {
                    base: Some(new_msg(MsgType::LoadCollection)),
                    db_name: "".to_string(),
                    collection_name: self.name.to_string(),
                    replica_number,
                })
                .await?
                .into_inner(),
        ))
    }

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
                collection_names: vec![self.name.to_string()],
            })
            .await?
            .into_inner();

        status_to_result(response.status)?;

        let names = response.collection_names;
        let percent = response.in_memory_percentages;
        for i in 0..names.len() {
            if self.name == names[i] {
                return Ok(percent[i]);
            }
        }

        Err(SuperError::Unknown)
    }

    pub async fn load_blocked(&self, replica_number: i32) -> Result<()> {
        self.load(replica_number).await?;

        loop {
            if self.get_load_percent().await? >= 100 {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(config::WAIT_LOAD_DURATION_MS)).await;
        }
    }

    pub async fn is_load(&self) -> Result<bool> {
        Ok(self.get_load_percent().await? >= 100)
    }

    pub async fn release(&self) -> Result<()> {
        status_to_result(Some(
            self.client
                .clone()
                .release_collection(ReleaseCollectionRequest {
                    base: Some(new_msg(MsgType::ReleaseCollection)),
                    db_name: "".to_string(),
                    collection_name: self.name.to_string(),
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
                    collection_name: self.name.to_string(),
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
                collection_name: self.name.to_string(),
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
                collection_names: vec![self.name.to_string()],
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
                collection_name: self.name.to_string(),
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

    pub async fn create_partition(&self, name: String) -> Result<()> {
        let res = self
            .client
            .clone()
            .create_partition(CreatePartitionRequest {
                base: Some(new_msg(MsgType::ShowPartitions)),
                db_name: "".to_string(),
                collection_name: self.name.to_string(),
                partition_name: name,
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
                    collection_name: self.name.to_string(),
                    partition_name: p.as_ref().to_string(),
                })
                .await?
                .into_inner();

            status_to_result(res.status)?;

            Ok(res.value)
        }
    }
}

impl<E: schema::Entity> Collection<E> {
    pub async fn create(
        &self,
        shards_num: Option<i32>,
        consistency_level: Option<ConsistencyLevel>,
    ) -> Result<()> {
        let schema: crate::proto::schema::CollectionSchema = E::schema().into();

        let mut buf = BytesMut::new();
        schema.encode(&mut buf)?;

        let status = self
            .client
            .clone()
            .create_collection(CreateCollectionRequest {
                base: Some(new_msg(MsgType::CreateCollection)),
                db_name: "".to_string(),
                collection_name: schema.name.to_string(),
                schema: buf.to_vec(),
                shards_num: shards_num.unwrap_or(1),
                consistency_level: consistency_level.unwrap_or(ConsistencyLevel::Session) as i32,
            })
            .await?
            .into_inner();

        status_to_result(Some(status))
    }

    pub async fn query<'a, Exp, F, P>(&self, expr: Exp, partition_names: P) -> Result<F>
    where
        Exp: ToString,
        F: schema::Collection<'a, Entity = E> + schema::FromDataFields,
        P: IntoIterator,
        P::Item: ToString,
    {
        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(new_msg(MsgType::Retrieve)),
                db_name: "".to_string(),
                collection_name: self.name.to_string(),
                expr: expr.to_string(),
                output_fields: F::columns()
                    .into_iter()
                    .map(|x| x.name.to_string())
                    .collect(),
                partition_names: partition_names.into_iter().map(|x| x.to_string()).collect(),
                guarantee_timestamp: 0,
                travel_timestamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(res.status)?;

        Ok(F::from_data_fields(res.fields_data).unwrap())
    }

    pub async fn insert<'a, P: Into<String>, C: schema::Collection<'a, Entity = E>>(
        &self,
        fields_data: C,
        partition_name: Option<P>,
    ) -> Result<crate::proto::milvus::MutationResult> {
        let partition_name = if let Some(p) = partition_name {
            let p = p.into();

            if !self.has_partition(&p).await? {
                self.create_partition(p.clone()).await?;
            }

            p
        } else {
            String::new()
        };

        Ok(self
            .client
            .clone()
            .insert(InsertRequest {
                base: Some(new_msg(MsgType::Insert)),
                db_name: "".to_string(),
                collection_name: self.name.to_string(),
                partition_name,
                num_rows: fields_data.len() as _,
                fields_data: fields_data.into_data_fields(),
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
