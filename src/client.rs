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

use crate::collection::CollectionCache;
use crate::config::RPC_TIMEOUT;
use crate::error::{Error, Result};
pub use crate::proto::common::ConsistencyLevel;
use crate::proto::common::{MsgBase, MsgType};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::proto::milvus::FlushRequest;
use crate::utils::status_to_result;
use base64::engine::general_purpose;
use base64::Engine;
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
    timeout: Option<Duration>,
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
            timeout: None,
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

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn build(self) -> Result<Client> {
        Client::with_timeout(
            self.dst,
            self.timeout.unwrap_or(RPC_TIMEOUT),
            self.username,
            self.password,
        )
        .await
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) client: MilvusServiceClient<InterceptedService<Channel, AuthInterceptor>>,
    pub(crate) collection_cache: CollectionCache,
}

impl Client {
    pub async fn new<D>(dst: D) -> Result<Self>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError> + std::fmt::Debug,
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

        Ok(Self {
            client: client.clone(),
            collection_cache: CollectionCache::new(client),
        })
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

    // alias related:

    /// Creates an alias for a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `alias` - The alias to be created.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn create_alias(
        &self,
        collection_name: impl Into<String>,
        alias: impl Into<String>,
    ) -> Result<()> {
        let collection_name = collection_name.into();
        let alias = alias.into();
        status_to_result(&Some(
            self.client
                .clone()
                .create_alias(crate::proto::milvus::CreateAliasRequest {
                    base: Some(MsgBase::new(MsgType::CreateAlias)),
                    db_name: "".to_string(), // reserved
                    collection_name,
                    alias,
                })
                .await?
                .into_inner(),
        ))
    }

    /// Drops an alias.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias to be dropped.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn drop_alias<S>(&self, alias: S) -> Result<()>
    where
        S: Into<String>,
    {
        let alias = alias.into();
        status_to_result(&Some(
            self.client
                .clone()
                .drop_alias(crate::proto::milvus::DropAliasRequest {
                    base: Some(MsgBase::new(MsgType::DropAlias)),
                    db_name: "".to_string(), // reserved
                    alias,
                })
                .await?
                .into_inner(),
        ))
    }

    /// Alter the alias of a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `alias` - The new alias for the collection.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn alter_alias(
        &self,
        collection_name: impl Into<String>,
        alias: impl Into<String>,
    ) -> Result<()> {
        let collection_name = collection_name.into();
        let alias = alias.into();
        status_to_result(&Some(
            self.client
                .clone()
                .alter_alias(crate::proto::milvus::AlterAliasRequest {
                    base: Some(MsgBase::new(MsgType::AlterAlias)),
                    db_name: "".to_string(), // reserved
                    collection_name,
                    alias,
                })
                .await?
                .into_inner(),
        ))
    }
}
