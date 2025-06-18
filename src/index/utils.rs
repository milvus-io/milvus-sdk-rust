//! Index utilities for Milvus.
//!
//! This module provides utilities for creating, describing, dropping, and altering indexes in Milvus.
//!
//! # Examples
//!
//! # Examples
//!
//! ```rust
//! use milvus_sdk_rust::index::utils;
//! ```
//!
use crate::collection::Error;
use crate::error::{Error as SuperError, Result};
use crate::index::IndexInfo;
use crate::proto::common::{IndexState, KeyValuePair, MsgBase, MsgType};
use crate::proto::milvus::{CreateIndexRequest, DescribeIndexRequest, DropIndexRequest};
use crate::utils::status_to_result;
use crate::{client::Client, index::IndexParams};
use crate::{config, proto};
use std::time::Duration;

impl Client {

    async fn create_index_impl<S>(
        &self,
        collection_name: S,
        field_name: impl Into<String>,
        index_params: IndexParams,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let field_name = field_name.into();
        let status = self
            .client
            .clone()
            .create_index(CreateIndexRequest {
                base: Some(MsgBase::new(MsgType::CreateIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name,
                extra_params: index_params.extra_kv_params(),
                index_name: index_params.name().clone(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(status))
    }

    /// Creates an index for a specified field in a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `field_name` - The name of the field to create an index for.
    /// * `index_params` - The parameters for the index.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `()` if successful, or an error if the index creation fails.
    pub async fn create_index<S>(
        &self,
        collection_name: S,
        field_name: impl Into<String>,
        index_params: IndexParams,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let field_name = field_name.into();
        self.create_index_impl(
            collection_name.clone(),
            field_name.clone(),
            index_params.clone(),
        )
        .await?;

        loop {
            let index_infos = self
                .describe_index(collection_name.clone(), field_name.clone())
                .await?;

            let index_info = index_infos
                .iter()
                .find(|&x| x.params().name() == index_params.name());
            if index_info.is_none() {
                return Err(SuperError::Unexpected(
                    "failed to describe index".to_owned(),
                ));
            }
            match index_info.unwrap().state() {
                IndexState::Finished => return Ok(()),
                IndexState::Failed => return Err(SuperError::Collection(Error::IndexBuildFailed)),
                _ => (),
            };

            tokio::time::sleep(Duration::from_millis(config::WAIT_CREATE_INDEX_DURATION_MS)).await;
        }
    }

    /// Retrieves information about the indexes for a specified field in a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `field_name` - The name of the field to describe the indexes for.
    ///
    /// # Returns
    /// Returns a `Result` containing a vector of `IndexInfo` if successful, or an error if the index description fails.
    pub async fn describe_index<S>(
        &self,
        collection_name: S,
        field_name: S,
    ) -> Result<Vec<IndexInfo>>
    where
        S: Into<String>,
    {
        let res = self
            .client
            .clone()
            .describe_index(DescribeIndexRequest {
                base: Some(MsgBase::new(MsgType::DescribeIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name: field_name.into(),
                index_name: "".to_string(),
                timestamp: 0,
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;

        Ok(res.index_descriptions.into_iter().map(Into::into).collect())
    }

    /// Drops an index for a specified field in a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `field_name` - The name of the field to drop the index for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `()` if successful, or an error if the index drop fails.
    pub async fn drop_index<S>(&self, collection_name: S, index_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let status = self
            .client
            .clone()
            .drop_index(DropIndexRequest {
                base: Some(MsgBase::new(MsgType::DropIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                field_name: "".to_string(),
                index_name: index_name.into()
            })
            .await?
            .into_inner();
        status_to_result(&Some(status))
    }

    /// Alters the properties of an index for a specified field in a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `index_name` - The name of the index to alter.
    /// * `properties` - The properties to alter.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `()` if successful, or an error if the index alteration fails.
    pub async fn alter_index_properties<S: Into<String>>(
        &self,
        collection_name: S,
        index_name: S,
        properties: Vec<KeyValuePair>,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .alter_index(proto::milvus::AlterIndexRequest {
                base: Some(MsgBase::new(MsgType::AlterIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                index_name: index_name.into(),
                extra_params: properties,
                delete_keys: Vec::new(),
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Drops the properties of an index for a specified field in a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `index_name` - The name of the index to drop the properties for.
    /// * `property_keys` - The keys of the properties to drop.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `()` if successful, or an error if the index properties drop fails.
    pub async fn drop_index_properties<S: Into<String>>(
        &self,
        collection_name: S,
        index_name: S,
        property_keys: Vec<S>,
    ) -> Result<()> {
        let delete_keys: Vec<String> = property_keys.into_iter().map(|x| x.into()).collect();
        let res = self
            .client
            .clone()
            .alter_index(proto::milvus::AlterIndexRequest {
                base: Some(MsgBase::new(MsgType::AlterIndex)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                index_name: index_name.into(),
                extra_params: Vec::new(),
                delete_keys,
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Lists the indexes for a specified field in a collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `field_name` - The name of the field to list the indexes for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of index names if successful, or an error if the index listing fails.
    pub async fn list_indexes<S: Into<String>>(
        &self,
        collection_name: S,
        field_name: Option<S>,
    ) -> Result<Vec<String>> {
        let res = if let Some(field_name) = field_name {
            self.describe_index(collection_name, field_name)
                .await?
                .into_iter()
                .map(|x| x.index_name)
                .collect()
        } else {
            self.describe_index(collection_name.into(), "".to_string())
                .await?
                .into_iter()
                .map(|x| x.index_name)
                .collect()
        };
        Ok(res)
    }
}
