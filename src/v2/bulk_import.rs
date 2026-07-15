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

//! REST client and request/response types for bulk-import jobs.
//!
//! Bulk import is served by Milvus's HTTP API rather than its gRPC service.
//! Construct a [`BulkImport`] client from [`BulkImportConfig`], then create,
//! inspect, or list import jobs with validated request objects.
//!
//! ```rust,no_run
//! use milvus::v2::prelude::*;
//!
//! # async fn example() -> Result<()> {
//! let bulk_import = BulkImport::new(
//!     &BulkImportConfig::new()
//!         .url("http://localhost:19530")
//!         .api_key("root:Milvus"),
//! )?;
//! let response = bulk_import
//!     .bulk_import(
//!         BulkImportRequest::builder()
//!             .database_name("default")
//!             .collection_name("books")
//!             .file("imports/books.parquet")
//!             .build()?,
//!     )
//!     .await?;
//! println!("job ID: {:?}", response.job_id());
//! # Ok(())
//! # }
//! ```

use crate::v2::error::{Error, Result};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error as ThisError;

const CREATE_PATH: &str = "/v2/vectordb/jobs/import/create";
const LIST_PATH: &str = "/v2/vectordb/jobs/import/list";
const DESCRIBE_PATH: &str = "/v2/vectordb/jobs/import/describe";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_ERROR_BODY_CHARS: usize = 1_024;

///////////////////////////////////////////////////////////////////////////////
// BulkImportError
///////////////////////////////////////////////////////////////////////////////
/// Error produced by the bulk-import REST transport or server envelope.
#[derive(Debug, Clone, ThisError, PartialEq, Eq)]
#[non_exhaustive]
pub enum BulkImportError {
    #[error("bulk-import HTTP transport failed: {0}")]
    Transport(String),

    #[error("bulk-import HTTP request returned status {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("bulk-import server error (code={code}, message={message})")]
    Server { code: i64, message: String },
}

impl BulkImportError {
    /// Returns the HTTP status for an HTTP-level failure.
    pub fn http_status(&self) -> Option<u16> {
        match self {
            Self::HttpStatus { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Returns the REST envelope code for a server-level failure.
    pub fn server_code(&self) -> Option<i64> {
        match self {
            Self::Server { code, .. } => Some(*code),
            _ => None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// BulkImportConfig
///////////////////////////////////////////////////////////////////////////////
/// Connection settings for the bulk-import REST client.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BulkImportConfig {
    url: String,
    api_key: String,
    timeout: Duration,
    verify_tls: bool,
    ca_certificate_path: Option<PathBuf>,
    client_identity_path: Option<PathBuf>,
    client_certificate_path: Option<PathBuf>,
    client_private_key_path: Option<PathBuf>,
}

impl fmt::Debug for BulkImportConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BulkImportConfig")
            .field("url", &self.url)
            .field("api_key", &redacted(&self.api_key))
            .field("timeout", &self.timeout)
            .field("verify_tls", &self.verify_tls)
            .field("ca_certificate_path", &self.ca_certificate_path)
            .field("client_identity_path", &self.client_identity_path)
            .field("client_certificate_path", &self.client_certificate_path)
            .field(
                "client_private_key_path",
                &self.client_private_key_path.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl BulkImportConfig {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            api_key: String::new(),
            timeout: DEFAULT_TIMEOUT,
            verify_tls: true,
            ca_certificate_path: None,
            client_identity_path: None,
            client_certificate_path: None,
            client_private_key_path: None,
        }
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = value.into();
        self
    }

    pub fn set_url(&mut self, value: impl Into<String>) -> &mut Self {
        self.url = value.into();
        self
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn api_key(mut self, value: impl Into<String>) -> Self {
        self.api_key = value.into();
        self
    }

    pub fn set_api_key(&mut self, value: impl Into<String>) -> &mut Self {
        self.api_key = value.into();
        self
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    /// Sets the overall timeout for each HTTP request.
    ///
    /// A zero duration disables the client-side request timeout.
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }

    pub fn set_timeout(&mut self, value: Duration) -> &mut Self {
        self.timeout = value;
        self
    }

    pub fn get_timeout(&self) -> Duration {
        self.timeout
    }

    /// Enables or disables verification of the server's TLS certificate.
    pub fn verify_tls(mut self, value: bool) -> Self {
        self.verify_tls = value;
        self
    }

    pub fn set_verify_tls(&mut self, value: bool) -> &mut Self {
        self.verify_tls = value;
        self
    }

    pub fn get_verify_tls(&self) -> bool {
        self.verify_tls
    }

    /// Uses a PEM CA certificate to verify the server.
    ///
    /// This corresponds to passing a certificate path as PyMilvus's `verify` argument.
    pub fn ca_certificate_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.ca_certificate_path = Some(value.into());
        self
    }

    pub fn set_ca_certificate_path(&mut self, value: impl Into<PathBuf>) -> &mut Self {
        self.ca_certificate_path = Some(value.into());
        self
    }

    pub fn get_ca_certificate_path(&self) -> Option<&PathBuf> {
        self.ca_certificate_path.as_ref()
    }

    /// Uses one PEM file containing the client certificate and private key.
    ///
    /// This corresponds to passing a string as PyMilvus's `cert` argument.
    pub fn client_identity_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.client_identity_path = Some(value.into());
        self
    }

    pub fn set_client_identity_path(&mut self, value: impl Into<PathBuf>) -> &mut Self {
        self.client_identity_path = Some(value.into());
        self
    }

    pub fn get_client_identity_path(&self) -> Option<&PathBuf> {
        self.client_identity_path.as_ref()
    }

    /// Uses a PEM client certificate together with [`Self::client_private_key_path`].
    pub fn client_certificate_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.client_certificate_path = Some(value.into());
        self
    }

    pub fn set_client_certificate_path(&mut self, value: impl Into<PathBuf>) -> &mut Self {
        self.client_certificate_path = Some(value.into());
        self
    }

    pub fn get_client_certificate_path(&self) -> Option<&PathBuf> {
        self.client_certificate_path.as_ref()
    }

    /// Uses a PEM client private key together with [`Self::client_certificate_path`].
    pub fn client_private_key_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.client_private_key_path = Some(value.into());
        self
    }

    pub fn set_client_private_key_path(&mut self, value: impl Into<PathBuf>) -> &mut Self {
        self.client_private_key_path = Some(value.into());
        self
    }

    pub fn get_client_private_key_path(&self) -> Option<&PathBuf> {
        self.client_private_key_path.as_ref()
    }
}

///////////////////////////////////////////////////////////////////////////////
// BulkImport
///////////////////////////////////////////////////////////////////////////////
/// Async client for Milvus bulk-import REST endpoints.
#[derive(Clone)]
pub struct BulkImport {
    client: Client,
    base_url: String,
    api_key: String,
}

impl fmt::Debug for BulkImport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BulkImport")
            .field("base_url", &self.base_url)
            .field("api_key", &redacted(&self.api_key))
            .finish_non_exhaustive()
    }
}

impl BulkImport {
    /// Creates a reusable bulk-import REST client.
    pub fn new(config: &BulkImportConfig) -> Result<Self> {
        required("url", config.get_url())?;
        let url = Url::parse(config.get_url()).map_err(|error| {
            Error::validation("url".into(), format!("must be a valid HTTP URL: {error}"))
        })?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::validation(
                "url".into(),
                "scheme must be http or https".into(),
            ));
        }
        if url.host().is_none() {
            return Err(Error::validation(
                "url".into(),
                "must contain a host".into(),
            ));
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(Error::validation(
                "url".into(),
                "must not contain a query string or fragment".into(),
            ));
        }

        validate_tls_config(config)?;

        let mut builder = Client::builder().danger_accept_invalid_certs(!config.verify_tls);
        if let Some(path) = &config.ca_certificate_path {
            let certificate = reqwest::Certificate::from_pem(&read_certificate(path, "verify")?)
                .map_err(|error| {
                    Error::validation(
                        "ca_certificate_path".into(),
                        format!("must contain a valid PEM CA certificate: {error}"),
                    )
                })?;
            builder = builder.add_root_certificate(certificate);
        }
        if let Some(path) = &config.client_identity_path {
            let identity =
                reqwest::Identity::from_pem(&read_certificate(path, "cert")?).map_err(|error| {
                    Error::validation(
                        "client_identity_path".into(),
                        format!("must contain a valid PEM client certificate and key: {error}"),
                    )
                })?;
            builder = builder.identity(identity);
        } else if let (Some(certificate_path), Some(private_key_path)) = (
            &config.client_certificate_path,
            &config.client_private_key_path,
        ) {
            let mut identity_pem = read_certificate(certificate_path, "cert")?;
            if !identity_pem.ends_with(b"\n") {
                identity_pem.push(b'\n');
            }
            identity_pem.extend(read_certificate(private_key_path, "cert")?);
            let identity = reqwest::Identity::from_pem(&identity_pem).map_err(|error| {
                Error::validation(
                    "client_certificate_path".into(),
                    format!("certificate and private key must be valid PEM data: {error}"),
                )
            })?;
            builder = builder.identity(identity);
        }
        if !config.timeout.is_zero() {
            builder = builder
                .connect_timeout(config.timeout)
                .timeout(config.timeout);
        }
        let client = builder
            .build()
            .map_err(|error| BulkImportError::Transport(error.to_string()))?;
        Ok(Self {
            client,
            base_url: config.url.trim_end_matches('/').to_owned(),
            api_key: config.api_key.clone(),
        })
    }

    /// Creates a bulk-import job.
    pub async fn bulk_import(&self, request: BulkImportRequest) -> Result<BulkImportResponse> {
        let database_name = request.database_name.clone();
        self.post(CREATE_PATH, &database_name, request.into_body())
            .await
    }

    /// Lists bulk-import jobs visible to the target deployment.
    pub async fn list_import_jobs(
        &self,
        request: ListImportJobsRequest,
    ) -> Result<BulkImportResponse> {
        let database_name = request.database_name.clone();
        self.post(LIST_PATH, &database_name, request.into_body())
            .await
    }

    /// Gets the current state and progress of one bulk-import job.
    pub async fn get_import_progress(
        &self,
        request: GetImportProgressRequest,
    ) -> Result<BulkImportResponse> {
        let database_name = request.database_name.clone();
        self.post(DESCRIBE_PATH, &database_name, request.into_body())
            .await
    }

    async fn post(
        &self,
        path: &str,
        database_name: &str,
        body: Value,
    ) -> Result<BulkImportResponse> {
        let endpoint = format!("{}{path}", self.base_url);
        let mut request = self
            .client
            .post(&endpoint)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&body);
        request = request.header(AUTHORIZATION, format!("Bearer {}", self.api_key));
        if !database_name.is_empty() {
            request = request.header("DB-Name", database_name);
        }

        let response = request.send().await.map_err(http_transport_error)?;
        let status = response.status();
        let response_body = response.text().await.map_err(http_transport_error)?;
        if status != StatusCode::OK {
            return Err(BulkImportError::HttpStatus {
                status: status.as_u16(),
                body: truncate_error_body(&response_body),
            }
            .into());
        }

        let envelope: RestEnvelope = serde_json::from_str(&response_body).map_err(|error| {
            Error::MalformedResponse(format!(
                "bulk-import response from {endpoint} is not valid JSON: {error}"
            ))
        })?;
        if envelope.code != 0 {
            return Err(BulkImportError::Server {
                code: envelope.code,
                message: envelope.message,
            }
            .into());
        }
        Ok(BulkImportResponse {
            code: envelope.code,
            message: envelope.message,
            data: envelope.data,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// BulkImportRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for creating a bulk-import job.
#[derive(Clone, PartialEq)]
#[non_exhaustive]
pub struct BulkImportRequest {
    database_name: String,
    collection_name: String,
    partition_name: String,
    files: Vec<Vec<String>>,
    object_url: String,
    object_urls: Vec<Vec<String>>,
    cluster_id: String,
    project_id: String,
    region_id: String,
    access_key: String,
    secret_key: String,
    token: String,
    volume_name: String,
    data_paths: Vec<Vec<String>>,
    options: HashMap<String, Value>,
}

impl fmt::Debug for BulkImportRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BulkImportRequest")
            .field("database_name", &self.database_name)
            .field("collection_name", &self.collection_name)
            .field("partition_name", &self.partition_name)
            .field("files", &self.files)
            .field("object_url", &self.object_url)
            .field("object_urls", &self.object_urls)
            .field("cluster_id", &self.cluster_id)
            .field("project_id", &self.project_id)
            .field("region_id", &self.region_id)
            .field("access_key", &redacted(&self.access_key))
            .field("secret_key", &redacted(&self.secret_key))
            .field("token", &redacted(&self.token))
            .field("volume_name", &self.volume_name)
            .field("data_paths", &self.data_paths)
            .field("options", &self.options)
            .finish()
    }
}

impl BulkImportRequest {
    pub fn builder() -> BulkImportRequestBuilder {
        BulkImportRequestBuilder {
            value: Self::empty(),
        }
    }

    pub fn into_builder(self) -> BulkImportRequestBuilder {
        BulkImportRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    pub fn files(&self) -> &[Vec<String>] {
        &self.files
    }

    /// Returns the deprecated singular object URL.
    ///
    /// Prefer [`Self::object_urls`] for new applications.
    pub fn object_url(&self) -> &str {
        &self.object_url
    }

    pub fn object_urls(&self) -> &[Vec<String>] {
        &self.object_urls
    }

    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn region_id(&self) -> &str {
        &self.region_id
    }

    pub fn access_key(&self) -> &str {
        &self.access_key
    }

    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn volume_name(&self) -> &str {
        &self.volume_name
    }

    pub fn data_paths(&self) -> &[Vec<String>] {
        &self.data_paths
    }

    pub fn options(&self) -> &HashMap<String, Value> {
        &self.options
    }

    fn empty() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            partition_name: String::new(),
            files: Vec::new(),
            object_url: String::new(),
            object_urls: Vec::new(),
            cluster_id: String::new(),
            project_id: String::new(),
            region_id: String::new(),
            access_key: String::new(),
            secret_key: String::new(),
            token: String::new(),
            volume_name: String::new(),
            data_paths: Vec::new(),
            options: HashMap::new(),
        }
    }

    fn into_body(self) -> Value {
        let mut body = Map::new();
        insert_string(&mut body, "dbName", self.database_name, true);
        insert_string(&mut body, "collectionName", self.collection_name, true);
        insert_string(&mut body, "partitionName", self.partition_name, false);
        insert_groups(&mut body, "files", self.files);
        insert_string(&mut body, "objectUrl", self.object_url, false);
        insert_groups(&mut body, "objectUrls", self.object_urls);
        insert_string(&mut body, "clusterId", self.cluster_id, false);
        insert_string(&mut body, "projectId", self.project_id, false);
        insert_string(&mut body, "regionId", self.region_id, false);
        insert_string(&mut body, "accessKey", self.access_key, false);
        insert_string(&mut body, "secretKey", self.secret_key, false);
        insert_string(&mut body, "token", self.token, false);
        insert_string(&mut body, "volumeName", self.volume_name, false);
        insert_groups(&mut body, "dataPaths", self.data_paths);
        if !self.options.is_empty() {
            body.insert(
                "options".into(),
                Value::Object(self.options.into_iter().collect()),
            );
        }
        Value::Object(body)
    }
}

///////////////////////////////////////////////////////////////////////////////
// BulkImportRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for [`BulkImportRequest`].
#[derive(Debug, Clone)]
pub struct BulkImportRequestBuilder {
    value: BulkImportRequest,
}

impl BulkImportRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Sets local/object-storage paths visible to an open-source Milvus deployment.
    pub fn files<I, G, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = G>,
        G: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.value.files = string_groups(values);
        self
    }

    /// Adds one independently imported file group.
    pub fn file_group<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.value
            .files
            .push(values.into_iter().map(Into::into).collect());
        self
    }

    /// Adds one JSON, JSONL, CSV, or Parquet file as its own import group.
    pub fn file(mut self, value: impl Into<String>) -> Self {
        self.value.files.push(vec![value.into()]);
        self
    }

    /// Sets the deprecated singular object URL accepted by Milvus 2.6.
    ///
    /// Prefer [`Self::object_urls`] for new applications.
    pub fn object_url(mut self, value: impl Into<String>) -> Self {
        self.value.object_url = value.into();
        self
    }

    pub fn object_urls<I, G, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = G>,
        G: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.value.object_urls = string_groups(values);
        self
    }

    pub fn object_url_group<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.value
            .object_urls
            .push(values.into_iter().map(Into::into).collect());
        self
    }

    pub fn cluster_id(mut self, value: impl Into<String>) -> Self {
        self.value.cluster_id = value.into();
        self
    }

    pub fn project_id(mut self, value: impl Into<String>) -> Self {
        self.value.project_id = value.into();
        self
    }

    pub fn region_id(mut self, value: impl Into<String>) -> Self {
        self.value.region_id = value.into();
        self
    }

    pub fn access_key(mut self, value: impl Into<String>) -> Self {
        self.value.access_key = value.into();
        self
    }

    pub fn secret_key(mut self, value: impl Into<String>) -> Self {
        self.value.secret_key = value.into();
        self
    }

    pub fn token(mut self, value: impl Into<String>) -> Self {
        self.value.token = value.into();
        self
    }

    pub fn volume_name(mut self, value: impl Into<String>) -> Self {
        self.value.volume_name = value.into();
        self
    }

    pub fn data_paths<I, G, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = G>,
        G: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.value.data_paths = string_groups(values);
        self
    }

    pub fn data_path_group<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.value
            .data_paths
            .push(values.into_iter().map(Into::into).collect());
        self
    }

    pub fn options(mut self, value: HashMap<String, Value>) -> Self {
        self.value.options = value;
        self
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.value.options.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<BulkImportRequest> {
        required("collection_name", &self.value.collection_name)?;
        validate_cloud_target(
            &self.value.cluster_id,
            &self.value.project_id,
            &self.value.region_id,
        )?;
        validate_groups("files", &self.value.files)?;
        validate_groups("object_urls", &self.value.object_urls)?;
        validate_groups("data_paths", &self.value.data_paths)?;

        let source_count = [
            !self.value.files.is_empty(),
            !self.value.object_url.is_empty(),
            !self.value.object_urls.is_empty(),
            !self.value.data_paths.is_empty(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if source_count != 1 {
            return Err(Error::validation(
                "data_source".into(),
                "exactly one of files, object_url, object_urls, or data_paths must be specified"
                    .into(),
            ));
        }
        if !self.value.data_paths.is_empty() && self.value.volume_name.is_empty() {
            return Err(Error::validation(
                "volume_name".into(),
                "must be specified when data_paths are used".into(),
            ));
        }
        if self.value.access_key.is_empty() != self.value.secret_key.is_empty() {
            return Err(Error::validation(
                "storage_credentials".into(),
                "access_key and secret_key must be specified together".into(),
            ));
        }
        if self.value.options.keys().any(|key| key.is_empty()) {
            return Err(Error::validation(
                "options".into(),
                "option keys must not be empty".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListImportJobsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for listing bulk-import jobs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListImportJobsRequest {
    database_name: String,
    collection_name: String,
    cluster_id: String,
    project_id: String,
    region_id: String,
    page_size: u32,
    current_page: u32,
}

impl ListImportJobsRequest {
    pub fn builder() -> ListImportJobsRequestBuilder {
        ListImportJobsRequestBuilder {
            value: Self::empty(),
        }
    }

    pub fn into_builder(self) -> ListImportJobsRequestBuilder {
        ListImportJobsRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn region_id(&self) -> &str {
        &self.region_id
    }

    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    pub fn current_page(&self) -> u32 {
        self.current_page
    }

    fn empty() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            cluster_id: String::new(),
            project_id: String::new(),
            region_id: String::new(),
            page_size: 10,
            current_page: 1,
        }
    }

    fn into_body(self) -> Value {
        let mut body = Map::new();
        insert_string(&mut body, "dbName", self.database_name, true);
        insert_string(&mut body, "collectionName", self.collection_name, true);
        insert_string(&mut body, "clusterId", self.cluster_id, false);
        insert_string(&mut body, "projectId", self.project_id, false);
        insert_string(&mut body, "regionId", self.region_id, false);
        body.insert("pageSize".into(), Value::from(self.page_size));
        body.insert("currentPage".into(), Value::from(self.current_page));
        Value::Object(body)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListImportJobsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for [`ListImportJobsRequest`].
#[derive(Debug, Clone)]
pub struct ListImportJobsRequestBuilder {
    value: ListImportJobsRequest,
}

impl ListImportJobsRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn cluster_id(mut self, value: impl Into<String>) -> Self {
        self.value.cluster_id = value.into();
        self
    }

    pub fn project_id(mut self, value: impl Into<String>) -> Self {
        self.value.project_id = value.into();
        self
    }

    pub fn region_id(mut self, value: impl Into<String>) -> Self {
        self.value.region_id = value.into();
        self
    }

    pub fn page_size(mut self, value: u32) -> Self {
        self.value.page_size = value;
        self
    }

    pub fn current_page(mut self, value: u32) -> Self {
        self.value.current_page = value;
        self
    }

    pub fn build(self) -> Result<ListImportJobsRequest> {
        validate_cloud_target(
            &self.value.cluster_id,
            &self.value.project_id,
            &self.value.region_id,
        )?;
        if self.value.page_size == 0 {
            return Err(Error::validation(
                "page_size".into(),
                "must be greater than zero".into(),
            ));
        }
        if self.value.current_page == 0 {
            return Err(Error::validation(
                "current_page".into(),
                "must be greater than zero".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetImportProgressRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters identifying one import job whose progress should be described.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetImportProgressRequest {
    database_name: String,
    job_id: String,
    cluster_id: String,
    project_id: String,
    region_id: String,
}

impl GetImportProgressRequest {
    pub fn builder() -> GetImportProgressRequestBuilder {
        GetImportProgressRequestBuilder {
            value: Self::empty(),
        }
    }

    pub fn into_builder(self) -> GetImportProgressRequestBuilder {
        GetImportProgressRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn region_id(&self) -> &str {
        &self.region_id
    }

    fn empty() -> Self {
        Self {
            database_name: String::new(),
            job_id: String::new(),
            cluster_id: String::new(),
            project_id: String::new(),
            region_id: String::new(),
        }
    }

    fn into_body(self) -> Value {
        let mut body = Map::new();
        insert_string(&mut body, "jobId", self.job_id, true);
        insert_string(&mut body, "clusterId", self.cluster_id, false);
        insert_string(&mut body, "projectId", self.project_id, false);
        insert_string(&mut body, "regionId", self.region_id, false);
        Value::Object(body)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetImportProgressRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for [`GetImportProgressRequest`].
#[derive(Debug, Clone)]
pub struct GetImportProgressRequestBuilder {
    value: GetImportProgressRequest,
}

impl GetImportProgressRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn job_id(mut self, value: impl Into<String>) -> Self {
        self.value.job_id = value.into();
        self
    }

    pub fn cluster_id(mut self, value: impl Into<String>) -> Self {
        self.value.cluster_id = value.into();
        self
    }

    pub fn project_id(mut self, value: impl Into<String>) -> Self {
        self.value.project_id = value.into();
        self
    }

    pub fn region_id(mut self, value: impl Into<String>) -> Self {
        self.value.region_id = value.into();
        self
    }

    pub fn build(self) -> Result<GetImportProgressRequest> {
        required("job_id", &self.value.job_id)?;
        validate_cloud_target(
            &self.value.cluster_id,
            &self.value.project_id,
            &self.value.region_id,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// BulkImportResponse
///////////////////////////////////////////////////////////////////////////////
/// Successful REST envelope returned by a bulk-import operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct BulkImportResponse {
    code: i64,
    message: String,
    data: Value,
}

impl BulkImportResponse {
    pub fn code(&self) -> i64 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the endpoint-specific response payload.
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Returns one field from the endpoint-specific response payload.
    pub fn get_data_field(&self, name: &str) -> Option<&Value> {
        self.data.get(name)
    }

    /// Returns the created or described import job ID when supplied.
    pub fn job_id(&self) -> Option<&str> {
        self.get_data_field("jobId").and_then(Value::as_str)
    }

    /// Returns the import state when supplied by the describe endpoint.
    pub fn state(&self) -> Option<&str> {
        self.get_data_field("state").and_then(Value::as_str)
    }

    /// Returns the progress percentage when supplied by the describe endpoint.
    pub fn progress(&self) -> Option<i64> {
        self.get_data_field("progress").and_then(Value::as_i64)
    }

    /// Returns the failure reason when supplied by the describe endpoint.
    pub fn reason(&self) -> Option<&str> {
        self.get_data_field("reason").and_then(Value::as_str)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RestEnvelope
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
struct RestEnvelope {
    code: i64,
    #[serde(default)]
    message: String,
    #[serde(default)]
    data: Value,
}

fn required(parameter: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        Err(Error::validation(
            parameter.into(),
            "must be specified".into(),
        ))
    } else {
        Ok(())
    }
}

fn validate_cloud_target(cluster_id: &str, project_id: &str, region_id: &str) -> Result<()> {
    if project_id.is_empty() != region_id.is_empty() {
        return Err(Error::validation(
            "project_target".into(),
            "project_id and region_id must be specified together".into(),
        ));
    }
    if !cluster_id.is_empty() && !project_id.is_empty() {
        return Err(Error::validation(
            "cloud_target".into(),
            "cluster_id cannot be combined with project_id and region_id".into(),
        ));
    }
    Ok(())
}

fn validate_tls_config(config: &BulkImportConfig) -> Result<()> {
    if !config.verify_tls && config.ca_certificate_path.is_some() {
        return Err(Error::validation(
            "ca_certificate_path".into(),
            "cannot be combined with disabled TLS verification".into(),
        ));
    }
    if config.client_identity_path.is_some()
        && (config.client_certificate_path.is_some() || config.client_private_key_path.is_some())
    {
        return Err(Error::validation(
            "client_identity_path".into(),
            "cannot be combined with separate client certificate or private-key paths".into(),
        ));
    }
    if config.client_certificate_path.is_some() != config.client_private_key_path.is_some() {
        return Err(Error::validation(
            "client_certificate".into(),
            "client_certificate_path and client_private_key_path must be specified together".into(),
        ));
    }
    Ok(())
}

fn read_certificate(path: &PathBuf, parameter: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| {
        Error::validation(
            parameter.into(),
            format!("cannot read certificate file {}: {error}", path.display()),
        )
    })
}

fn validate_groups(parameter: &str, groups: &[Vec<String>]) -> Result<()> {
    for (group_index, group) in groups.iter().enumerate() {
        if group.is_empty() {
            return Err(Error::validation(
                format!("{parameter}[{group_index}]"),
                "must contain at least one path".into(),
            ));
        }
        if let Some(path_index) = group.iter().position(String::is_empty) {
            return Err(Error::validation(
                format!("{parameter}[{group_index}][{path_index}]"),
                "path must not be empty".into(),
            ));
        }
    }
    Ok(())
}

fn string_groups<I, G, S>(values: I) -> Vec<Vec<String>>
where
    I: IntoIterator<Item = G>,
    G: IntoIterator<Item = S>,
    S: Into<String>,
{
    values
        .into_iter()
        .map(|group| group.into_iter().map(Into::into).collect())
        .collect()
}

fn insert_string(body: &mut Map<String, Value>, key: &str, value: String, include_empty: bool) {
    if include_empty || !value.is_empty() {
        body.insert(key.into(), Value::String(value));
    }
}

fn insert_groups(body: &mut Map<String, Value>, key: &str, values: Vec<Vec<String>>) {
    if !values.is_empty() {
        body.insert(
            key.into(),
            Value::Array(
                values
                    .into_iter()
                    .map(|group| Value::Array(group.into_iter().map(Value::String).collect()))
                    .collect(),
            ),
        );
    }
}

fn truncate_error_body(value: &str) -> String {
    let mut chars = value.chars();
    let truncated = chars
        .by_ref()
        .take(MAX_ERROR_BODY_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn http_transport_error(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::Timeout("bulk-import HTTP request".into())
    } else {
        BulkImportError::Transport(error.to_string()).into()
    }
}

fn redacted(value: &str) -> &str {
    if value.is_empty() {
        ""
    } else {
        "<redacted>"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_request_serializes_open_source_files() {
        let request = BulkImportRequest::builder()
            .database_name("books_db")
            .collection_name("books")
            .partition_name("archive")
            .file("one.parquet")
            .file_group(["id.npy", "embedding.npy"])
            .option("auto_commit", json!(false))
            .build()
            .expect("valid request");

        assert_eq!(
            request.into_body(),
            json!({
                "dbName": "books_db",
                "collectionName": "books",
                "partitionName": "archive",
                "files": [["one.parquet"], ["id.npy", "embedding.npy"]],
                "options": {"auto_commit": false}
            })
        );
    }

    #[test]
    fn create_request_serializes_cloud_and_volume_sources() {
        let deprecated_object_url = BulkImportRequest::builder()
            .collection_name("books")
            .object_url("s3://bucket/books.parquet")
            .build()
            .expect("valid singular object URL request");
        assert_eq!(
            deprecated_object_url.into_body(),
            json!({
                "dbName": "",
                "collectionName": "books",
                "objectUrl": "s3://bucket/books.parquet"
            })
        );

        let cloud = BulkImportRequest::builder()
            .collection_name("books")
            .cluster_id("cluster-1")
            .object_urls([["s3://bucket/books.parquet"]])
            .access_key("access")
            .secret_key("secret")
            .token("session")
            .build()
            .expect("valid cloud request");
        assert_eq!(
            cloud.into_body(),
            json!({
                "dbName": "",
                "collectionName": "books",
                "objectUrls": [["s3://bucket/books.parquet"]],
                "clusterId": "cluster-1",
                "accessKey": "access",
                "secretKey": "secret",
                "token": "session"
            })
        );

        let volume = BulkImportRequest::builder()
            .collection_name("books")
            .project_id("project-1")
            .region_id("aws-us-west-2")
            .volume_name("volume-1")
            .data_paths([["books/part-1.parquet"]])
            .build()
            .expect("valid volume request");
        assert_eq!(
            volume.into_body(),
            json!({
                "dbName": "",
                "collectionName": "books",
                "projectId": "project-1",
                "regionId": "aws-us-west-2",
                "volumeName": "volume-1",
                "dataPaths": [["books/part-1.parquet"]]
            })
        );
    }

    #[test]
    fn request_builders_reject_ambiguous_or_incomplete_input() {
        assert!(BulkImportRequest::builder()
            .collection_name("books")
            .build()
            .is_err());
        assert!(BulkImportRequest::builder()
            .collection_name("books")
            .file("books.parquet")
            .object_urls([["s3://bucket/books.parquet"]])
            .build()
            .is_err());
        assert!(BulkImportRequest::builder()
            .collection_name("books")
            .data_paths([["books.parquet"]])
            .build()
            .is_err());
        assert!(GetImportProgressRequest::builder().build().is_err());
        assert!(ListImportJobsRequest::builder()
            .page_size(0)
            .build()
            .is_err());
    }

    #[test]
    fn response_exposes_common_import_fields() {
        let response = BulkImportResponse {
            code: 0,
            message: "success".into(),
            data: json!({
                "jobId": "job-1",
                "state": "Importing",
                "progress": 42,
                "reason": ""
            }),
        };
        assert_eq!(response.job_id(), Some("job-1"));
        assert_eq!(response.state(), Some("Importing"));
        assert_eq!(response.progress(), Some(42));
        assert_eq!(response.reason(), Some(""));
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let config = BulkImportConfig::new()
            .url("https://example.com")
            .api_key("secret-api-key");
        let request = BulkImportRequest::builder()
            .collection_name("books")
            .object_urls([["s3://bucket/books.parquet"]])
            .access_key("secret-access-key")
            .secret_key("secret-storage-key")
            .token("secret-token")
            .build()
            .expect("valid request");

        let config_debug = format!("{config:?}");
        let request_debug = format!("{request:?}");
        assert!(!config_debug.contains("secret-api-key"));
        assert!(!request_debug.contains("secret-access-key"));
        assert!(!request_debug.contains("secret-storage-key"));
        assert!(!request_debug.contains("secret-token"));
    }

    #[test]
    fn config_defaults_and_tls_validation_match_pymilvus_inputs() {
        let config = BulkImportConfig::new();
        assert_eq!(config.get_timeout(), Duration::from_secs(20));
        assert!(config.get_verify_tls());

        let error = BulkImport::new(
            &BulkImportConfig::new()
                .url("https://example.com")
                .verify_tls(false)
                .ca_certificate_path("ca.pem"),
        )
        .expect_err("a CA path conflicts with disabled verification");
        assert!(matches!(error, Error::Validation(_)));

        let error = BulkImport::new(
            &BulkImportConfig::new()
                .url("https://example.com")
                .client_certificate_path("client.pem"),
        )
        .expect_err("separate client certificate requires its private key");
        assert!(matches!(error, Error::Validation(_)));
    }
}
