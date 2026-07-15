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

//! Request types for database operations.

use crate::proto::{common, milvus};
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::required;
use std::collections::{HashMap, HashSet};

fn properties(values: HashMap<String, String>) -> Vec<common::KeyValuePair> {
    values
        .into_iter()
        .map(|(key, value)| common::KeyValuePair { key, value })
        .collect()
}

///////////////////////////////////////////////////////////////////////////////
// CreateDatabaseRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_database operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateDatabaseRequest {
    pub(crate) database_name: String,
    pub(crate) properties: HashMap<String, String>,
}

impl CreateDatabaseRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            properties: Default::default(),
        }
    }

    pub fn builder() -> CreateDatabaseRequestBuilder {
        CreateDatabaseRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateDatabaseRequestBuilder {
        CreateDatabaseRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn into_proto(self) -> milvus::CreateDatabaseRequest {
        milvus::CreateDatabaseRequest {
            base: None,
            db_name: self.database_name,
            properties: properties(self.properties),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateDatabaseRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateDatabaseRequest.
#[derive(Debug, Clone)]
pub struct CreateDatabaseRequestBuilder {
    value: CreateDatabaseRequest,
}

impl CreateDatabaseRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    pub fn build(self) -> Result<CreateDatabaseRequest> {
        required("database_name", &self.value.database_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropDatabaseRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_database operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropDatabaseRequest {
    pub(crate) database_name: String,
}

impl DropDatabaseRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
        }
    }

    pub fn builder() -> DropDatabaseRequestBuilder {
        DropDatabaseRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropDatabaseRequestBuilder {
        DropDatabaseRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub(crate) fn into_proto(self) -> milvus::DropDatabaseRequest {
        milvus::DropDatabaseRequest {
            base: None,
            db_name: self.database_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropDatabaseRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropDatabaseRequest.
#[derive(Debug, Clone)]
pub struct DropDatabaseRequestBuilder {
    value: DropDatabaseRequest,
}

impl DropDatabaseRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn build(self) -> Result<DropDatabaseRequest> {
        required("database_name", &self.value.database_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListDatabasesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_databases operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListDatabasesRequest;

impl ListDatabasesRequest {
    pub fn builder() -> ListDatabasesRequestBuilder {
        ListDatabasesRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListDatabasesRequestBuilder {
        ListDatabasesRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::ListDatabasesRequest {
        milvus::ListDatabasesRequest::default()
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListDatabasesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListDatabasesRequest.
#[derive(Debug, Clone, Copy)]
pub struct ListDatabasesRequestBuilder;

impl ListDatabasesRequestBuilder {
    pub fn build(self) -> Result<ListDatabasesRequest> {
        Ok(ListDatabasesRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterDatabasePropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_database_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AlterDatabasePropertiesRequest {
    pub(crate) database_name: String,
    pub(crate) properties: HashMap<String, String>,
}

impl AlterDatabasePropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            properties: Default::default(),
        }
    }

    pub fn builder() -> AlterDatabasePropertiesRequestBuilder {
        AlterDatabasePropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterDatabasePropertiesRequestBuilder {
        AlterDatabasePropertiesRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn into_proto(self) -> milvus::AlterDatabaseRequest {
        milvus::AlterDatabaseRequest {
            base: None,
            db_name: self.database_name,
            db_id: String::new(),
            properties: properties(self.properties),
            delete_keys: Vec::new(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterDatabasePropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterDatabasePropertiesRequest.
#[derive(Debug, Clone)]
pub struct AlterDatabasePropertiesRequestBuilder {
    value: AlterDatabasePropertiesRequest,
}

impl AlterDatabasePropertiesRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.value.properties.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<AlterDatabasePropertiesRequest> {
        required("database_name", &self.value.database_name)?;
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
// DropDatabasePropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_database_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropDatabasePropertiesRequest {
    pub(crate) database_name: String,
    pub(crate) property_keys: HashSet<String>,
}

impl DropDatabasePropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            property_keys: Default::default(),
        }
    }

    pub fn builder() -> DropDatabasePropertiesRequestBuilder {
        DropDatabasePropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropDatabasePropertiesRequestBuilder {
        DropDatabasePropertiesRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn property_keys(&self) -> &HashSet<String> {
        &self.property_keys
    }

    pub(crate) fn into_proto(self) -> milvus::AlterDatabaseRequest {
        milvus::AlterDatabaseRequest {
            base: None,
            db_name: self.database_name,
            db_id: String::new(),
            properties: Vec::new(),
            delete_keys: self.property_keys.into_iter().collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropDatabasePropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropDatabasePropertiesRequest.
#[derive(Debug, Clone)]
pub struct DropDatabasePropertiesRequestBuilder {
    value: DropDatabasePropertiesRequest,
}

impl DropDatabasePropertiesRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn property_keys(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.property_keys = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn property_key(mut self, value: impl Into<String>) -> Self {
        self.value.property_keys.insert(value.into());
        self
    }

    pub fn build(self) -> Result<DropDatabasePropertiesRequest> {
        required("database_name", &self.value.database_name)?;
        if self.value.property_keys.is_empty() {
            return Err(Error::validation(
                "property_keys".into(),
                "must contain at least one property key".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeDatabaseRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_database operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeDatabaseRequest {
    pub(crate) database_name: String,
}

impl DescribeDatabaseRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
        }
    }

    pub fn builder() -> DescribeDatabaseRequestBuilder {
        DescribeDatabaseRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeDatabaseRequestBuilder {
        DescribeDatabaseRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub(crate) fn into_proto(self) -> milvus::DescribeDatabaseRequest {
        milvus::DescribeDatabaseRequest {
            base: None,
            db_name: self.database_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeDatabaseRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeDatabaseRequest.
#[derive(Debug, Clone)]
pub struct DescribeDatabaseRequestBuilder {
    value: DescribeDatabaseRequest,
}

impl DescribeDatabaseRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn build(self) -> Result<DescribeDatabaseRequest> {
        required("database_name", &self.value.database_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod database_property_request_tests {
    use super::{AlterDatabasePropertiesRequest, DropDatabasePropertiesRequest};

    #[test]
    fn alter_and_drop_database_properties_encode_separate_operations() {
        let alter = AlterDatabasePropertiesRequest::builder()
            .database_name("catalog")
            .property("database.replica.number", "1")
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(alter.db_name, "catalog");
        assert_eq!(alter.properties.len(), 1);
        assert!(alter.delete_keys.is_empty());

        let drop = DropDatabasePropertiesRequest::builder()
            .database_name("catalog")
            .property_keys(["test.one", "test.two"])
            .property_key("test.one")
            .build()
            .expect("valid request")
            .into_proto();
        assert!(drop.properties.is_empty());
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
    fn list_databases_request_default_values() {
        assert_eq!(
            ListDatabasesRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::ListDatabasesRequest::default()
        );
    }

    #[test]
    fn list_databases_request_populated_values() {
        let value = ListDatabasesRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(value.into_proto(), milvus::ListDatabasesRequest::default());
    }

    #[test]
    fn create_database_request_default_values() {
        let value = CreateDatabaseRequest::empty();
        let expected_database_name: String = String::new();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn create_database_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = CreateDatabaseRequest::builder()
            .database_name(database_name.clone())
            .properties(properties.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.properties().to_owned(), properties);
    }

    #[test]
    fn drop_database_request_default_values() {
        let value = DropDatabaseRequest::empty();
        let expected_database_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
    }

    #[test]
    fn drop_database_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let value = DropDatabaseRequest::builder()
            .database_name(database_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), database_name);
    }

    #[test]
    fn alter_database_properties_request_default_values() {
        let value = AlterDatabasePropertiesRequest::empty();
        let expected_database_name: String = String::new();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn alter_database_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = AlterDatabasePropertiesRequest::builder()
            .database_name(database_name.clone())
            .properties(properties.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.properties().to_owned(), properties);
    }

    #[test]
    fn drop_database_properties_request_default_values() {
        let value = DropDatabasePropertiesRequest::empty();
        let expected_database_name: String = String::new();
        let expected_property_keys: HashSet<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.property_keys().to_owned(), expected_property_keys);
    }

    #[test]
    fn drop_database_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let property_keys = HashSet::from(["property_keys-value".to_owned()]);
        let value = DropDatabasePropertiesRequest::builder()
            .database_name(database_name.clone())
            .property_keys(property_keys.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.property_keys().to_owned(), property_keys);
    }

    #[test]
    fn describe_database_request_default_values() {
        let value = DescribeDatabaseRequest::empty();
        let expected_database_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
    }

    #[test]
    fn describe_database_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let value = DescribeDatabaseRequest::builder()
            .database_name(database_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), database_name);
    }
}
