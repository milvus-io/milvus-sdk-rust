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
use crate::config::RPC_TIMEOUT;
use crate::error::{Error, Result};
use crate::options::CreateCollectionOptions;
pub use crate::proto::common::ConsistencyLevel;
use crate::proto::common::{MsgBase, MsgType};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::{
    CreateCollectionRequest, DescribeCollectionRequest, DropCollectionRequest, FlushRequest,
    HasCollectionRequest, ShowCollectionsRequest,
};
use crate::schema::CollectionSchema;
use crate::utils::status_to_result;
use base64::engine::general_purpose;
use base64::Engine;
use prost::bytes::BytesMut;
use prost::Message;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;
use tonic::codegen::{InterceptedService, StdError};
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::Request;

#[derive(Clone)]
pub struct AuthInterceptor {
    token: Option<String>,
}

impl Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut req: Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        if let Some(ref token) = self.token {
            let header_value = format!("{}", token);
            req.metadata_mut()
                .insert("authorization", header_value.parse().unwrap());
        }

        Ok(req)
    }
}

#[derive(Clone)]
pub struct ClientBuilder<D> {
    dst: D,
    username: Option<String>,
    password: Option<String>,
}

impl<D> ClientBuilder<D>
where
    D: TryInto<tonic::transport::Endpoint> + Clone,
    D::Error: Into<StdError>,
    D::Error: std::fmt::Debug,
{
    pub fn new(dst: D) -> Self {
        Self {
            dst,
            username: None,
            password: None,
        }
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_owned());
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    pub async fn build(self) -> Result<Client> {
        Client::with_timeout(self.dst, RPC_TIMEOUT, self.username, self.password).await
    }
}

#[derive(Clone)]
pub struct Client {
    client: MilvusServiceClient<InterceptedService<Channel, AuthInterceptor>>,
}

impl Client {
    pub async fn new<D>(dst: D) -> Result<Self>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
        D::Error: std::fmt::Debug,
    {
        Self::with_timeout(dst, RPC_TIMEOUT, None, None).await
    }

    pub async fn with_timeout<D>(
        dst: D,
        timeout: Duration,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Self>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
        D::Error: std::fmt::Debug,
    {
        let mut dst: tonic::transport::Endpoint = dst.try_into().map_err(|err| {
            Error::InvalidParameter("url".to_owned(), format!("to parse {:?}", err))
        })?;

        dst = dst.timeout(timeout);

        let token = match (username, password) {
            (Some(username), Some(password)) => {
                let auth_token = format!("{}:{}", username, password);
                let auth_token = general_purpose::STANDARD.encode(auth_token);
                Some(auth_token)
            }
            _ => None,
        };

        let auth_interceptor = AuthInterceptor { token };

        let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;

        let client = MilvusServiceClient::with_interceptor(conn, auth_interceptor);

        Ok(Self { client })
    }

    pub async fn create_collection(
        &self,
        schema: CollectionSchema,
        options: Option<CreateCollectionOptions>,
    ) -> Result<Collection> {
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

        status_to_result(&Some(status))?;

        Ok(self.get_collection(&schema.name).await?)
    }

    pub async fn get_collection(&self, collection_name: &str) -> Result<Collection> {
        let resp = self
            .client
            .clone()
            .describe_collection(DescribeCollectionRequest {
                base: Some(MsgBase::new(MsgType::DescribeCollection)),
                db_name: "".to_owned(),
                collection_name: collection_name.to_owned(),
                collection_id: 0,
                time_stamp: 0,
            })
            .await?
            .into_inner();

        status_to_result(&resp.status)?;

        Ok(Collection::new(self.client.clone(), resp))
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

    pub async fn flush_collections<C>(&self, collections: C) -> Result<HashMap<String, Vec<i64>>>
    where
        C: IntoIterator,
        C::Item: ToString,
    {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(MsgBase::new(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: collections.into_iter().map(|x| x.to_string()).collect(),
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res
            .coll_seg_i_ds
            .into_iter()
            .map(|(k, v)| (k, v.data))
            .collect())
    }
}
