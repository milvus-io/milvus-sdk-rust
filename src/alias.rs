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

use crate::client::Client;
use crate::error::Result;
use crate::proto::common::{MsgBase, MsgType};
use crate::proto::milvus::{
    AlterAliasRequest, CreateAliasRequest, DescribeAliasRequest, DropAliasRequest,
    ListAliasesRequest,
};
use crate::utils::status_to_result;

impl Client {
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
                .create_alias(CreateAliasRequest {
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
                .drop_alias(DropAliasRequest {
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
                .alter_alias(AlterAliasRequest {
                    base: Some(MsgBase::new(MsgType::AlterAlias)),
                    db_name: "".to_string(), // reserved
                    collection_name,
                    alias,
                })
                .await?
                .into_inner(),
        ))
    }

    /// Describe an alias.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias to be described.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the alias, collection name, and database name.
    pub async fn describe_alias(&self, alias: &str) -> Result<(String, String, String)> {
        let resp = self
            .client
            .clone()
            .describe_alias(DescribeAliasRequest {
                base: Some(MsgBase::new(MsgType::DescribeAlias)),
                db_name: "".to_string(),
                alias: alias.to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok((resp.alias, resp.collection, resp.db_name))
    }

    /// List a collection's aliases
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the database name, collection name, and aliases.
    pub async fn list_aliases(
        &self,
        collection_name: &str,
    ) -> Result<(String, String, Vec<String>)> {
        let resp = self
            .client
            .clone()
            .list_aliases(ListAliasesRequest {
                base: Some(MsgBase::new(MsgType::ListAliases)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok((resp.db_name, resp.collection_name, resp.aliases))
    }
}
