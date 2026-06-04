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
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use base64::engine::general_purpose;
use base64::Engine;
use std::convert::TryInto;
use std::time::Duration;
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::Request;

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DbInterceptor {
    pub db_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CombinedInterceptor {
    pub auth: AuthInterceptor,
    pub db: DbInterceptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerVersion {
    pub version: String,
    pub build_time: String,
    pub git_commit: String,
    pub go_version: String,
    pub deploy_mode: String,
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

impl Interceptor for DbInterceptor {
    fn call(
        &mut self,
        mut req: Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        if let Some(ref db_name) = self.db_name {
            req.metadata_mut()
                .insert("dbname", db_name.parse().unwrap());
        }

        Ok(req)
    }
}

impl Interceptor for CombinedInterceptor {
    fn call(
        &mut self,
        mut req: Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        // Apply auth interceptor
        req = self.auth.call(req)?;
        // Apply db interceptor
        req = self.db.call(req)?;
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
    D::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
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
    pub(crate) client: MilvusServiceClient<InterceptedService<Channel, CombinedInterceptor>>,
    pub(crate) collection_cache: CollectionCache,
    pub(crate) db_name: Option<String>,
    pub(crate) channel: Channel,
    pub(crate) auth_interceptor: AuthInterceptor,
}

impl Client {
    pub async fn new<D>(dst: D) -> Result<Self>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<Box<dyn std::error::Error + Send + Sync>> + std::fmt::Debug,
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
        D::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        D::Error: std::fmt::Debug,
    {
        let mut dst: tonic::transport::Endpoint = dst.try_into().map_err(|err| {
            Error::InvalidParameter("url".to_owned(), format!("to parse {:?}", err))
        })?;

        dst = dst
            .timeout(timeout)
            .http2_keep_alive_interval(Duration::from_secs(10))
            .keep_alive_timeout(Duration::from_secs(5))
            .keep_alive_while_idle(true);

        let token = match (username, password) {
            (Some(username), Some(password)) => {
                let auth_token = format!("{}:{}", username, password);
                let auth_token = general_purpose::STANDARD.encode(auth_token);
                Some(auth_token)
            }
            _ => None,
        };

        let auth_interceptor = AuthInterceptor { token };
        let db_interceptor = DbInterceptor { db_name: None };
        let combined_interceptor = CombinedInterceptor {
            auth: auth_interceptor.clone(),
            db: db_interceptor,
        };

        let channel = dst.connect().await?;

        let client = MilvusServiceClient::with_interceptor(channel.clone(), combined_interceptor);

        Ok(Self {
            client: client.clone(),
            collection_cache: CollectionCache::new(client),
            db_name: None,
            channel,
            auth_interceptor,
        })
    }
}
