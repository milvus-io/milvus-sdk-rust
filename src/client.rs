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

use crate::collection::{CollectionSchema, FieldSchema};
use crate::error::{Error, Result};
pub use crate::proto::common::ConsistencyLevel;
use crate::proto::common::ErrorCode;
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{CreateCollectionRequest, DropCollectionRequest, HasCollectionRequest};
use crate::proto::schema;
use prost::bytes::BytesMut;
use prost::Message;
use std::error::Error as _;
use tonic::codegen::StdError;
use tonic::transport::Channel;

pub struct Client {
    client: MilvusServiceClient<Channel>,
}

impl Client {
    pub async fn new<D>(dst: D) -> Result<Self>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        match MilvusServiceClient::connect(dst).await {
            Ok(i) => Ok(Self { client: i }),
            Err(e) => Err(Error::Communication(e)),
        }
    }

    pub async fn create_collection<S>(
        &self,
        name: S,
        description: S,
        cs: CollectionSchema,
        shards_num: i32,
        consistency_level: ConsistencyLevel,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let n = name.into();
        let schema = Self::convert_collection(cs, n.clone(), description.into());
        let mut buf = BytesMut::new();
        //TODO unwrap instead of panic
        schema.encode(&mut buf).unwrap();
        let status = match self
            .client
            .clone()
            .create_collection(CreateCollectionRequest {
                base: None,
                db_name: "".to_string(),
                collection_name: n,
                schema: buf.to_vec(),
                shards_num: shards_num,
                consistency_level: consistency_level as i32,
            })
            .await
        {
            Ok(i) => i.into_inner(),
            Err(e) => return Err(Error::from(e)),
        };
        match ErrorCode::from_i32(status.error_code) {
            Some(i) => match i {
                ErrorCode::Success => Ok(()),
                _ => Err(Error::from(status)),
            },
            None => Err(Error::Unknown()),
        }
    }

    pub async fn drop_collection<S>(&self, name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let status = match self
            .client
            .clone()
            .drop_collection(DropCollectionRequest {
                base: None,
                db_name: "".to_string(),
                collection_name: name.into(),
            })
            .await
        {
            Ok(i) => i.into_inner(),
            Err(e) => return Err(Error::from(e)),
        };
        match ErrorCode::from_i32(status.error_code) {
            Some(i) => match i {
                ErrorCode::Success => Ok(()),
                _ => Err(Error::from(status)),
            },
            None => Err(Error::Unknown()),
        }
    }
    pub async fn has_collection<S>(&self, name: S) -> Result<bool>
    where
        S: Into<String>,
    {
        let res = match self
            .client
            .clone()
            .has_collection(HasCollectionRequest {
                base: None,
                db_name: "".to_string(),
                collection_name: name.into(),
                time_stamp: 0,
            })
            .await
        {
            Ok(i) => i.into_inner(),
            Err(e) => return Err(Error::from(e)),
        };
        let status = match res.status {
            Some(s) => s,
            None => return Err(Error::Unknown()),
        };
        match ErrorCode::from_i32(status.error_code) {
            Some(i) => match i {
                ErrorCode::Success => Ok(res.value),
                _ => Err(Error::from(status)),
            },
            None => Err(Error::Unknown()),
        }
    }

    fn convert_collection(
        cs: CollectionSchema,
        name: String,
        des: String,
    ) -> schema::CollectionSchema {
        let (field, auto_id) = cs.unpack();
        let f = field.into_iter().map(FieldSchema::convert_field).collect();
        schema::CollectionSchema {
            name: name,
            description: des,
            auto_id: auto_id,
            fields: f,
        }
    }
}
