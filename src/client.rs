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

use crate::collection::Collection;
use crate::error::{Error, Result};
pub use crate::proto::common::ConsistencyLevel;
use crate::proto::common::MsgType;
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    CreateCollectionRequest, DropCollectionRequest, FlushRequest, HasCollectionRequest,
};
use crate::schema::{self, CollectionSchema};
use crate::utils::{new_msg, status_to_result};
use prost::bytes::BytesMut;
use prost::Message;
use std::collections::HashMap;
use std::convert::TryInto;
use tonic::codegen::StdError;
use tonic::transport::Channel;

pub struct Client {
    client: MilvusServiceClient<Channel>,
}

impl Client {
    pub async fn new<D>(dst: D) -> Result<Self>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        match MilvusServiceClient::connect(dst).await {
            Ok(c) => Ok(Self { client: c }),
            Err(e) => Err(Error::Communication(e)),
        }
    }

    pub async fn create_collection(
        &self,
        schema: CollectionSchema,
        shards_num: i32,
        consistency_level: ConsistencyLevel,
    ) -> Result<Collection> {
        let schema: crate::proto::schema::CollectionSchema = schema.into();
        let mut buf = BytesMut::new();

        //TODO unwrap instead of panic
        schema.encode(&mut buf).unwrap();

        let status = self
            .client
            .clone()
            .create_collection(CreateCollectionRequest {
                base: Some(new_msg(MsgType::CreateCollection)),
                db_name: "".to_string(),
                collection_name: schema.name.to_string(),
                schema: buf.to_vec(),
                shards_num,
                consistency_level: consistency_level as i32,
            })
            .await?
            .into_inner();

        status_to_result(Some(status))?;

        Ok(Collection::new(self.client.clone(), schema.into()))
    }

    pub async fn drop_collection<S>(&self, name: S) -> Result<()>
    where
        S: Into<String>,
    {
        status_to_result(Some(
            self.client
                .clone()
                .drop_collection(DropCollectionRequest {
                    base: Some(new_msg(MsgType::DropCollection)),
                    db_name: "".to_string(),
                    collection_name: name.into(),
                })
                .await?
                .into_inner(),
        ))
    }

    pub async fn has_collection<S>(&self, name: S) -> Result<bool>
    where
        S: Into<String>,
    {
        let name = name.into();
        let res = self
            .client
            .clone()
            .has_collection(HasCollectionRequest {
                base: Some(new_msg(MsgType::HasCollection)),
                db_name: "".to_string(),
                collection_name: name.clone(),
                time_stamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(res.status)?;

        Ok(res.value)
    }

    // pub async fn get_collection<E: schema::Schema>(&self,name: &str) -> Result<Collection<E>> {
    //     Ok(Collection::new(self.client.clone(), E::NAME))
    // }

    pub async fn flush_collections<C>(&self, collections: C) -> Result<HashMap<String, Vec<i64>>>
    where
        C: IntoIterator,
        C::Item: ToString,
    {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(new_msg(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: collections.into_iter().map(|x| x.to_string()).collect(),
            })
            .await?
            .into_inner();

        status_to_result(res.status)?;

        Ok(res
            .coll_seg_i_ds
            .into_iter()
            .map(|(k, v)| (k, v.data))
            .collect())
    }
}
