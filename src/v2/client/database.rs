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

//! ClientV2 database operations and database selection.

use super::{normalize_database, ClientV2};
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Selects the database used by subsequent operations on this client and its clones.
    ///
    /// This changes client-side routing metadata; it does not create the database or verify that
    /// it exists. Use [`ClientV2::describe_database`] or a database operation to validate it.
    pub fn use_database(&self, database: impl Into<String>) -> Result<()> {
        let database = normalize_database(database.into())?;

        *self.database.write() = database;
        Ok(())
    }

    /// Returns the database currently selected by this client.
    pub fn current_database(&self) -> String {
        let database = self.database.read();
        if database.is_empty() {
            "default".to_owned()
        } else {
            database.clone()
        }
    }

    /// Creates a database in the connected Milvus instance.
    pub async fn create_database(
        &self,
        request: request::database::CreateDatabaseRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(Idempotent, self, create_database, request.into_proto())?;
        self.status(status)
    }

    /// Drops a database and the resources owned by it according to server policy.
    pub async fn drop_database(
        &self,
        request: request::database::DropDatabaseRequest,
    ) -> Result<()> {
        let raw = request.into_proto();
        let name = raw.db_name.clone();
        let status = status_rpc_with_retry!(Idempotent, self, drop_database, raw)?;
        self.status(status)?;
        self.clear_database_cache(&name);
        Ok(())
    }

    /// Lists databases visible to the authenticated user.
    pub async fn list_databases(
        &self,
        request: request::database::ListDatabasesRequest,
    ) -> Result<response::database::ListDatabasesResponse> {
        let response = rpc_with_retry!(self, list_databases, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::database::ListDatabasesResponse::from_proto(
            response,
        ))
    }

    /// Updates mutable properties of a database.
    pub async fn alter_database_properties(
        &self,
        request: request::database::AlterDatabasePropertiesRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(Idempotent, self, alter_database, request.into_proto())?;
        self.status(status)
    }

    /// Removes the requested mutable database properties.
    pub async fn drop_database_properties(
        &self,
        request: request::database::DropDatabasePropertiesRequest,
    ) -> Result<()> {
        let status =
            status_rpc_with_retry!(Idempotent, self, alter_database, request.into_proto())?;
        self.status(status)
    }

    /// Retrieves a database's metadata and properties.
    pub async fn describe_database(
        &self,
        request: request::database::DescribeDatabaseRequest,
    ) -> Result<response::database::DescribeDatabaseResponse> {
        let response = rpc_with_retry!(self, describe_database, request.into_proto())?;
        status_to_result(&response.status)?;
        Ok(response::database::DescribeDatabaseResponse::from_proto(
            response,
        ))
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::milvus;
    use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
    use crate::v2::client::cache::SCHEMA_CACHE;
    use crate::v2::error::Error;
    use crate::v2::types::RetryConfig;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::time::Duration;
    use tonic::transport::Endpoint;

    fn client() -> ClientV2 {
        let database = Arc::new(RwLock::new("default".to_owned()));
        let channel = Endpoint::from_static("http://127.0.0.1:19530").connect_lazy();
        let interceptor = super::super::V2Interceptor {
            token: None,
            database: Arc::clone(&database),
        };
        ClientV2 {
            service: Arc::new(RwLock::new(MilvusServiceClient::with_interceptor(
                channel,
                interceptor,
            ))),
            database,
            rpc_timeout: Arc::new(RwLock::new(Duration::from_secs(1))),
            retry: Arc::new(RwLock::new(RetryConfig::new())),
            cache_endpoint: Arc::new("database-tests".to_owned()),
            schema_load_scope: Arc::new(super::super::cache::SchemaLoadScope::new()),
            global_cluster: None,
        }
    }

    #[tokio::test]
    async fn use_database_updates_shared_selection_without_clearing_global_schema_cache() {
        let client = client();
        let clone = client.clone();
        SCHEMA_CACHE.set(
            &client.cache_endpoint,
            "default",
            "books",
            milvus::DescribeCollectionResponse::default(),
        );

        client.use_database("catalog").expect("switch database");

        assert_eq!(client.current_database(), "catalog");
        assert_eq!(clone.current_database(), "catalog");
        assert!(SCHEMA_CACHE
            .get(&client.cache_endpoint, "default", "books")
            .is_some());
        SCHEMA_CACHE.invalidate(&client.cache_endpoint, "default", "books");
    }

    #[tokio::test]
    async fn use_database_normalizes_default_and_rejects_invalid_metadata() {
        let client = client();

        *client.database.write() = String::new();
        assert_eq!(client.current_database(), "default");

        client.use_database("").expect("select default database");
        assert_eq!(client.current_database(), "default");

        let error = client
            .use_database("invalid\nname")
            .expect_err("reject invalid database metadata");
        assert!(matches!(error, Error::Validation(_)));
        assert_eq!(client.current_database(), "default");
    }
}
