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

//! Request types for index-management operations.

use crate::proto::{common, milvus};
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::{required, required_slice};
use crate::v2::types::{IndexParam, IndexType};
use std::collections::{HashMap, HashSet};
fn pairs(v: HashMap<String, String>) -> Vec<common::KeyValuePair> {
    v.into_iter()
        .map(|(key, value)| common::KeyValuePair {
            key,
            value,
            ..Default::default()
        })
        .collect()
}

///////////////////////////////////////////////////////////////////////////////
// CreateIndexRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_index operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateIndexRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) index_params: Vec<IndexParam>,
    pub(crate) sync: bool,
    pub(crate) timeout_ms: i64,
}

impl CreateIndexRequest {
    /// Creates a builder for this request.
    pub fn builder() -> CreateIndexRequestBuilder {
        CreateIndexRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateIndexRequestBuilder {
        CreateIndexRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the index params.
    pub fn index_params(&self) -> &[IndexParam] {
        &self.index_params
    }

    /// Returns whether sync.
    pub fn is_sync(&self) -> bool {
        self.sync
    }

    /// Returns the timeout ms.
    pub fn timeout_ms(&self) -> i64 {
        self.timeout_ms
    }

    pub(crate) fn into_parts(self, db: &str) -> (String, String, Vec<IndexParam>, bool, i64) {
        (
            self.database_name.unwrap_or_else(|| db.to_owned()),
            self.collection_name,
            self.index_params,
            self.sync,
            self.timeout_ms,
        )
    }
}

impl CreateIndexRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            index_params: Vec::new(),
            sync: true,
            timeout_ms: 60_000,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateIndexRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateIndexRequest.
#[derive(Debug, Clone)]
pub struct CreateIndexRequestBuilder {
    value: CreateIndexRequest,
}

impl CreateIndexRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the index params and returns the updated value.
    pub fn index_params(mut self, value: Vec<IndexParam>) -> Self {
        self.value.index_params = value;
        self
    }

    /// Sets the index param and returns the updated value.
    pub fn index_param(mut self, value: IndexParam) -> Self {
        self.value.index_params.push(value);
        self
    }

    /// Sets the sync and returns the updated value.
    pub fn sync(mut self, value: bool) -> Self {
        self.value.sync = value;
        self
    }

    /// Sets the timeout ms and returns the updated value.
    pub fn timeout_ms(mut self, value: i64) -> Self {
        self.value.timeout_ms = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateIndexRequest> {
        validate_index_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required_slice("index_params", &self.value.index_params)?;
        for index in &self.value.index_params {
            required("index_params.field_name", index.get_field_name())?;
            if index.get_index_type() == IndexType::Invalid {
                return Err(Error::validation(
                    "index_params.index_type".into(),
                    format!("must be specified for field {:?}", index.get_field_name()),
                ));
            }
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeIndexRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_index operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeIndexRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: String,
    pub(crate) index_name: String,
    pub(crate) timestamp: u64,
}

impl DescribeIndexRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            index_name: Default::default(),
            timestamp: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DescribeIndexRequestBuilder {
        DescribeIndexRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeIndexRequestBuilder {
        DescribeIndexRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the index name.
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn into_proto(self, db: &str) -> milvus::DescribeIndexRequest {
        milvus::DescribeIndexRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| db.to_owned()),
            collection_name: self.collection_name,
            field_name: self.field_name,
            index_name: self.index_name,
            timestamp: self.timestamp,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeIndexRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeIndexRequest.
#[derive(Debug, Clone)]
pub struct DescribeIndexRequestBuilder {
    value: DescribeIndexRequest,
}

impl DescribeIndexRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = value.into();
        self
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.value.index_name = value.into();
        self
    }

    /// Sets the timestamp and returns the updated value.
    pub fn timestamp(mut self, value: u64) -> Self {
        self.value.timestamp = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeIndexRequest> {
        validate_index_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListIndexesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_indexes operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListIndexesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: String,
    pub(crate) index_name: String,
    pub(crate) timestamp: u64,
}

impl ListIndexesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            index_name: Default::default(),
            timestamp: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListIndexesRequestBuilder {
        ListIndexesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListIndexesRequestBuilder {
        ListIndexesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name filter.
    ///
    /// When non-empty, only indexes built on this field are returned.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the index name.
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn into_proto(self, db: &str) -> milvus::GetIndexStatisticsRequest {
        milvus::GetIndexStatisticsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| db.to_owned()),
            collection_name: self.collection_name,
            index_name: self.index_name,
            timestamp: self.timestamp,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListIndexesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListIndexesRequest.
#[derive(Debug, Clone)]
pub struct ListIndexesRequestBuilder {
    value: ListIndexesRequest,
}

impl ListIndexesRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the field name filter and returns the updated value.
    ///
    /// When non-empty, only indexes built on this field are returned.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = value.into();
        self
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.value.index_name = value.into();
        self
    }

    /// Sets the timestamp and returns the updated value.
    pub fn timestamp(mut self, value: u64) -> Self {
        self.value.timestamp = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListIndexesRequest> {
        validate_index_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropIndexRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_index operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropIndexRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: String,
    pub(crate) index_name: String,
}

impl DropIndexRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            index_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropIndexRequestBuilder {
        DropIndexRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropIndexRequestBuilder {
        DropIndexRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the index name.
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub(crate) fn into_proto(self, db: &str) -> milvus::DropIndexRequest {
        let index_name = if self.index_name.is_empty() {
            self.field_name
        } else {
            self.index_name
        };
        milvus::DropIndexRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| db.to_owned()),
            collection_name: self.collection_name,
            // Milvus uses index_name to accept either an index name or a field
            // name. This matches the C++ V2 request conversion.
            field_name: String::new(),
            index_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropIndexRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropIndexRequest.
#[derive(Debug, Clone)]
pub struct DropIndexRequestBuilder {
    value: DropIndexRequest,
}

impl DropIndexRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = value.into();
        self
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.value.index_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropIndexRequest> {
        validate_index_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterIndexPropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_index_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AlterIndexPropertiesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) index_name: String,
    pub(crate) properties: HashMap<String, String>,
}

impl AlterIndexPropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            index_name: Default::default(),
            properties: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AlterIndexPropertiesRequestBuilder {
        AlterIndexPropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterIndexPropertiesRequestBuilder {
        AlterIndexPropertiesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the index name.
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    /// Returns the properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn into_proto(self, db: &str) -> milvus::AlterIndexRequest {
        milvus::AlterIndexRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| db.to_owned()),
            collection_name: self.collection_name,
            index_name: self.index_name,
            extra_params: pairs(self.properties),
            delete_keys: Vec::new(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterIndexPropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterIndexPropertiesRequest.
#[derive(Debug, Clone)]
pub struct AlterIndexPropertiesRequestBuilder {
    value: AlterIndexPropertiesRequest,
}

impl AlterIndexPropertiesRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.value.index_name = value.into();
        self
    }

    /// Sets the properties and returns the updated value.
    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    /// Sets the property and returns the updated value.
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.value.properties.insert(key.into(), value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AlterIndexPropertiesRequest> {
        validate_index_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("index_name", &self.value.index_name)?;
        if self.value.properties.is_empty() {
            return Err(Error::validation(
                "properties".into(),
                "must contain at least one property".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropIndexPropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_index_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropIndexPropertiesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) index_name: String,
    pub(crate) property_keys: HashSet<String>,
}

impl DropIndexPropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            index_name: Default::default(),
            property_keys: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropIndexPropertiesRequestBuilder {
        DropIndexPropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropIndexPropertiesRequestBuilder {
        DropIndexPropertiesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the index name.
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    /// Returns the property keys.
    pub fn property_keys(&self) -> &HashSet<String> {
        &self.property_keys
    }

    pub(crate) fn into_proto(self, db: &str) -> milvus::AlterIndexRequest {
        milvus::AlterIndexRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| db.to_owned()),
            collection_name: self.collection_name,
            index_name: self.index_name,
            extra_params: Vec::new(),
            delete_keys: self.property_keys.into_iter().collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropIndexPropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropIndexPropertiesRequest.
#[derive(Debug, Clone)]
pub struct DropIndexPropertiesRequestBuilder {
    value: DropIndexPropertiesRequest,
}

impl DropIndexPropertiesRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.value.index_name = value.into();
        self
    }

    /// Sets the property keys and returns the updated value.
    pub fn property_keys(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.property_keys = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the property key and returns the updated value.
    pub fn property_key(mut self, value: impl Into<String>) -> Self {
        self.value.property_keys.insert(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropIndexPropertiesRequest> {
        validate_index_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("index_name", &self.value.index_name)?;
        if self.value.property_keys.is_empty() {
            return Err(Error::validation(
                "property_keys".into(),
                "must contain at least one property key".into(),
            ));
        }
        Ok(self.value)
    }
}

fn validate_index_collection(_database_name: Option<&str>, collection_name: &str) -> Result<()> {
    required("collection_name", collection_name)
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod create_index_tests {
    use super::CreateIndexRequest;
    use crate::v2::types::{IndexParam, IndexType, MetricType};

    #[test]
    fn create_index_owns_multiple_creation_only_params() {
        let request = CreateIndexRequest::builder()
            .collection_name("books")
            .index_param(
                IndexParam::new()
                    .field_name("embedding")
                    .index_type(IndexType::Hnsw)
                    .metric_type(MetricType::Cosine)
                    .index_name("embedding_idx"),
            )
            .index_param(
                IndexParam::new()
                    .field_name("title")
                    .index_type(IndexType::Inverted)
                    .metric_type(MetricType::Default),
            )
            .build()
            .expect("valid request");

        assert_eq!(request.index_params().len().to_owned(), 2);
        assert!(request.is_sync());
        assert_eq!(request.timeout_ms().to_owned(), 60_000);

        let (database, collection, params, sync, timeout_ms) = request.into_parts("default");
        assert_eq!(database, "default");
        assert_eq!(collection, "books");
        assert_eq!(params.len(), 2);
        assert!(sync);
        assert_eq!(timeout_ms, 60_000);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod index_property_request_tests {
    use super::{AlterIndexPropertiesRequest, DropIndexPropertiesRequest};

    #[test]
    fn alter_and_drop_index_properties_encode_separate_operations() {
        let alter = AlterIndexPropertiesRequest::builder()
            .collection_name("books")
            .index_name("embedding_idx")
            .property("mmap.enabled", "true")
            .build()
            .expect("valid request")
            .into_proto("catalog");
        assert_eq!(alter.db_name, "catalog");
        assert_eq!(alter.extra_params.len(), 1);
        assert!(alter.delete_keys.is_empty());

        let drop = DropIndexPropertiesRequest::builder()
            .collection_name("books")
            .index_name("embedding_idx")
            .property_keys(["test.one", "test.two"])
            .property_key("test.one")
            .build()
            .expect("valid request")
            .into_proto("catalog");
        assert!(drop.extra_params.is_empty());
        assert_eq!(drop.delete_keys.len(), 2);
        assert!(drop.delete_keys.contains(&"test.one".to_owned()));
        assert!(drop.delete_keys.contains(&"test.two".to_owned()));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn create_index_request_default_values() {
        let value = CreateIndexRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_index_params: Vec<IndexParam> = Default::default();
        let expected_sync: bool = true;
        let expected_timeout_ms: i64 = 60_000;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.index_params().to_owned(), expected_index_params);
        assert_eq!(value.is_sync().to_owned(), expected_sync);
        assert_eq!(value.timeout_ms().to_owned(), expected_timeout_ms);
    }

    #[test]
    fn create_index_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let index_params = vec![IndexParam::new()
            .field_name("field")
            .index_type(crate::v2::IndexType::AutoIndex)
            .metric_type(crate::v2::MetricType::Cosine)
            .index_name("index")];
        let sync = true;
        let timeout_ms = 7;
        let value = CreateIndexRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .index_params(index_params.clone())
            .sync(sync.clone())
            .timeout_ms(timeout_ms.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.index_params().to_owned(), index_params);
        assert_eq!(value.is_sync().to_owned(), sync);
        assert_eq!(value.timeout_ms().to_owned(), timeout_ms);
    }

    #[test]
    fn describe_index_request_default_values() {
        let value = DescribeIndexRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field_name: String = String::new();
        let expected_index_name: String = String::new();
        let expected_timestamp: u64 = 0;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field_name().to_owned(), expected_field_name);
        assert_eq!(value.index_name().to_owned(), expected_index_name);
        assert_eq!(value.timestamp().to_owned(), expected_timestamp);
    }

    #[test]
    fn describe_index_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field_name = "field_name-value".to_owned();
        let index_name = "index_name-value".to_owned();
        let timestamp = 7;
        let value = DescribeIndexRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field_name(field_name.clone())
            .index_name(index_name.clone())
            .timestamp(timestamp.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field_name().to_owned(), field_name);
        assert_eq!(value.index_name().to_owned(), index_name);
        assert_eq!(value.timestamp().to_owned(), timestamp);
    }

    #[test]
    fn list_indexes_request_default_values() {
        let value = ListIndexesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field_name: String = String::new();
        let expected_index_name: String = String::new();
        let expected_timestamp: u64 = 0;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field_name().to_owned(), expected_field_name);
        assert_eq!(value.index_name().to_owned(), expected_index_name);
        assert_eq!(value.timestamp().to_owned(), expected_timestamp);
    }

    #[test]
    fn list_indexes_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field_name = "field_name-value".to_owned();
        let index_name = "index_name-value".to_owned();
        let timestamp = 7;
        let value = ListIndexesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field_name(field_name.clone())
            .index_name(index_name.clone())
            .timestamp(timestamp.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field_name().to_owned(), field_name);
        assert_eq!(value.index_name().to_owned(), index_name);
        assert_eq!(value.timestamp().to_owned(), timestamp);
    }

    #[test]
    fn drop_index_request_default_values() {
        let value = DropIndexRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field_name: String = String::new();
        let expected_index_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field_name().to_owned(), expected_field_name);
        assert_eq!(value.index_name().to_owned(), expected_index_name);
    }

    #[test]
    fn drop_index_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field_name = "field_name-value".to_owned();
        let index_name = "index_name-value".to_owned();
        let value = DropIndexRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field_name(field_name.clone())
            .index_name(index_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field_name().to_owned(), field_name);
        assert_eq!(value.index_name().to_owned(), index_name);
    }

    #[test]
    fn drop_index_uses_field_name_as_the_implicit_index_name() {
        let value = DropIndexRequest::builder()
            .collection_name("books")
            .field_name("embedding")
            .build()
            .expect("valid request")
            .into_proto("default");

        assert!(value.field_name.is_empty());
        assert_eq!(value.index_name, "embedding");
    }

    #[test]
    fn alter_index_properties_request_default_values() {
        let value = AlterIndexPropertiesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_index_name: String = String::new();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.index_name().to_owned(), expected_index_name);
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn alter_index_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let index_name = "index_name-value".to_owned();
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = AlterIndexPropertiesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .index_name(index_name.clone())
            .properties(properties.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.index_name().to_owned(), index_name);
        assert_eq!(value.properties().to_owned(), properties);
    }

    #[test]
    fn drop_index_properties_request_default_values() {
        let value = DropIndexPropertiesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_index_name: String = String::new();
        let expected_property_keys: HashSet<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.index_name().to_owned(), expected_index_name);
        assert_eq!(value.property_keys().to_owned(), expected_property_keys);
    }

    #[test]
    fn drop_index_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let index_name = "index_name-value".to_owned();
        let property_keys = HashSet::from(["property_keys-value".to_owned()]);
        let value = DropIndexPropertiesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .index_name(index_name.clone())
            .property_keys(property_keys.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.index_name().to_owned(), index_name);
        assert_eq!(value.property_keys().to_owned(), property_keys);
    }
}
