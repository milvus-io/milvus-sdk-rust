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

//! ClientV2 alias operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Creates an alias that can be used in query and search requests in place of a collection name.
    ///
    /// The SDK also aligns schema and session-timestamp cache state with the aliased collection.
    pub async fn create_alias(&self, request: request::alias::CreateAliasRequest) -> Result<()> {
        let database = self.current_database();
        let raw = request.into_proto(&database);
        let database = raw.db_name.clone();
        let collection = raw.collection_name.clone();
        let alias = raw.alias.clone();
        let status = status_rpc_with_retry!(Idempotent, self, create_alias, raw)?;
        self.status(status)?;
        self.remove_collection_description(&database, &alias);
        self.copy_dml_timestamp(&database, &collection, &alias);
        Ok(())
    }

    /// Drops an alias without dropping the underlying collection.
    pub async fn drop_alias(&self, request: request::alias::DropAliasRequest) -> Result<()> {
        let database = self.current_database();
        let raw = request.into_proto(&database);
        let database = raw.db_name.clone();
        let alias = raw.alias.clone();
        let status = status_rpc_with_retry!(Idempotent, self, drop_alias, raw)?;
        self.status(status)?;
        self.remove_collection_cache(&database, &alias);
        Ok(())
    }

    /// Repoints an alias from its current collection to another collection.
    pub async fn alter_alias(&self, request: request::alias::AlterAliasRequest) -> Result<()> {
        let database = self.current_database();
        let raw = request.into_proto(&database);
        let database = raw.db_name.clone();
        let collection = raw.collection_name.clone();
        let alias = raw.alias.clone();
        let status = status_rpc_with_retry!(Idempotent, self, alter_alias, raw)?;
        self.status(status)?;
        self.remove_collection_description(&database, &alias);
        self.copy_dml_timestamp(&database, &collection, &alias);
        Ok(())
    }

    /// Resolves an alias to its database and canonical collection name.
    pub async fn describe_alias(
        &self,
        request: request::alias::DescribeAliasRequest,
    ) -> Result<response::alias::DescribeAliasResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, describe_alias, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::alias::DescribeAliasResponse::from_proto(response))
    }

    /// Lists aliases associated with a collection in the selected database.
    pub async fn list_aliases(
        &self,
        request: request::alias::ListAliasesRequest,
    ) -> Result<response::alias::ListAliasesResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, list_aliases, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::alias::ListAliasesResponse::from_proto(response))
    }
}
