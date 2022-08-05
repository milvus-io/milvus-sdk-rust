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
use crate::proto::common::{ErrorCode, MsgType};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    DescribeCollectionRequest, InsertRequest, LoadCollectionRequest, QueryRequest,
    ReleaseCollectionRequest, ShowCollectionsRequest, ShowType,
};
use crate::schema::{CollectionFieldsData, FieldData};
use crate::utils::{new_msg, status_to_result};
use crate::{config, schema};
use std::error::Error as _;
use std::time::Duration;
use std::{dbg, thread};
use thiserror::Error as ThisError;
use tonic::transport::Channel;

#[derive(Debug, Clone)]
pub struct Collection {
    client: MilvusServiceClient<Channel>,
    name: String,
}

impl Collection {
    pub fn new(client: MilvusServiceClient<Channel>, name: String) -> Self {
        Self { client, name }
    }

    async fn load(&self, replica_number: i32) -> Result<()> {
        dbg!("start load");
        let status = match self
            .client
            .clone()
            .load_collection(LoadCollectionRequest {
                base: Some(new_msg(MsgType::LoadCollection)),
                db_name: "".to_string(),
                collection_name: self.name.clone(),
                replica_number,
            })
            .await
        {
            Ok(i) => i.into_inner(),
            Err(e) => return Err(SuperError::from(e)),
        };

        match ErrorCode::from_i32(status.error_code) {
            Some(i) => match i {
                ErrorCode::Success => Ok(()),
                _ => Err(SuperError::from(status)),
            },
            None => Err(SuperError::Unknown()),
        }
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
                collection_names: vec![self.name.clone()],
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

        Err(SuperError::Unknown())
    }

    pub async fn load_blocked(&self, replica_number: i32) -> Result<()> {
        self.load(replica_number).await?;
        loop {
            if self.get_load_percent().await? >= 100 {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(config::WAIT_LOAD_DURATION_MS));
        }
    }

    pub async fn is_load(&self) -> Result<bool> {
        if self.get_load_percent().await? >= 100 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn release(&self) -> Result<()> {
        let status = self
            .client
            .clone()
            .release_collection(ReleaseCollectionRequest {
                base: Some(new_msg(MsgType::ReleaseCollection)),
                db_name: "".to_string(),
                collection_name: self.name.clone(),
            })
            .await?
            .into_inner();

        status_to_result(Some(status))
    }

    pub async fn create_insert_frame(&self) -> Result<schema::CollectionFieldsData> {
        let response = self
            .client
            .clone()
            .describe_collection(DescribeCollectionRequest {
                base: Some(new_msg(MsgType::DescribeCollection)),
                db_name: "".to_string(),
                time_stamp: 0,
                collection_name: self.name.clone(),
                collection_id: 0,
            })
            .await?
            .into_inner();

        status_to_result(response.status)?;

        let schema = response.schema.ok_or(SuperError::Unknown())?;
        let schema: schema::CollectionSchema = (&schema).into();

        Ok(CollectionFieldsData::new(&schema))
    }

    pub async fn insert<P: Into<String>>(
        &self,
        fields_data: schema::CollectionFieldsData,
        partition_name: Option<P>,
    ) -> Result<crate::proto::milvus::MutationResult> {
        Ok(self
            .client
            .clone()
            .insert(InsertRequest {
                base: Some(new_msg(MsgType::Insert)),
                db_name: "".to_string(),
                collection_name: self.name.clone(),
                partition_name: partition_name
                    .map(|pn| pn.into())
                    .unwrap_or_else(String::new),
                num_rows: fields_data.len() as _,
                fields_data: fields_data.convert(),
                hash_keys: Vec::new(),
            })
            .await?
            .into_inner())
    }

    pub async fn query<E, F, P>(
        &self,
        expr: E,
        output_fields: F,
        partition_names: P,
    ) -> Result<Vec<FieldData>>
    where
        E: ToString,
        F: IntoIterator,
        F::Item: ToString,
        P: IntoIterator,
        P::Item: ToString,
    {
        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(new_msg(MsgType::Retrieve)),
                db_name: "".to_string(),
                collection_name: self.name.clone(),
                expr: expr.to_string(),
                output_fields: output_fields.into_iter().map(|x| x.to_string()).collect(),
                partition_names: partition_names.into_iter().map(|x| x.to_string()).collect(),
                guarantee_timestamp: 0,
                travel_timestamp: 0,
            })
            .await?
            .into_inner();

        Ok(res.fields_data.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, ThisError)]
pub enum Error {
    // TODO
}
