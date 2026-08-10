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

//! Response types returned by database operations.

use crate::proto::milvus;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// ListDatabasesResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_databases operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListDatabasesResponse {
    pub(crate) database_names: Vec<String>,
}

impl ListDatabasesResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            database_names: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListDatabasesResponseBuilder {
        ListDatabasesResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn database_names(&self) -> &[String] {
        &self.database_names
    }

    pub(crate) fn from_proto(value: milvus::ListDatabasesResponse) -> Self {
        Self {
            database_names: value.db_names,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListDatabasesResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListDatabasesResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListDatabasesResponseBuilder {
    value: ListDatabasesResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListDatabasesResponseBuilder {
    pub fn database_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.database_names = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> ListDatabasesResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeDatabaseResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_database operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeDatabaseResponse {
    pub(crate) database_name: String,
    pub(crate) database_id: i64,
    pub(crate) created_timestamp: u64,
    pub(crate) properties: HashMap<String, String>,
}

impl DescribeDatabaseResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            database_name: String::new(),
            database_id: 0,
            created_timestamp: 0,
            properties: HashMap::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeDatabaseResponseBuilder {
        DescribeDatabaseResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn database_id(&self) -> i64 {
        self.database_id
    }

    pub fn created_timestamp(&self) -> u64 {
        self.created_timestamp
    }

    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn from_proto(value: milvus::DescribeDatabaseResponse) -> Self {
        Self {
            database_name: value.db_name,
            database_id: value.db_id,
            created_timestamp: value.created_timestamp,
            properties: value
                .properties
                .into_iter()
                .map(|pair| (pair.key, pair.value))
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeDatabaseResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeDatabaseResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeDatabaseResponseBuilder {
    value: DescribeDatabaseResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeDatabaseResponseBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn database_id(mut self, value: i64) -> Self {
        self.value.database_id = value;
        self
    }

    pub fn created_timestamp(mut self, value: u64) -> Self {
        self.value.created_timestamp = value;
        self
    }

    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    pub fn build(self) -> DescribeDatabaseResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod describe_database_response_tests {
    use super::DescribeDatabaseResponse;
    use crate::proto::{common, milvus};

    #[test]
    fn describe_database_fields_are_exposed_directly() {
        let response = DescribeDatabaseResponse::from_proto(milvus::DescribeDatabaseResponse {
            db_name: "database".into(),
            db_id: 42,
            created_timestamp: 100,
            properties: vec![common::KeyValuePair {
                key: "key".into(),
                value: "value".into(),
            }],
            ..Default::default()
        });

        assert_eq!(response.database_name().to_owned(), "database");
        assert_eq!(response.database_id().to_owned(), 42);
        assert_eq!(response.created_timestamp().to_owned(), 100);
        assert_eq!(
            response.properties().get("key").map(String::as_str),
            Some("value")
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn list_databases_response_default_values() {
        let value = ListDatabasesResponse::builder().build();
        let expected_database_names: Vec<String> = Default::default();

        assert_eq!(value.database_names().to_owned(), expected_database_names);
    }

    #[test]
    fn list_databases_response_populated_values() {
        let database_names = vec!["database_names-value".to_owned()];
        let value = ListDatabasesResponse::builder()
            .database_names(database_names.clone())
            .build();

        assert_eq!(value.database_names().to_owned(), database_names);
    }

    #[test]
    fn describe_database_response_default_values() {
        let value = DescribeDatabaseResponse::builder().build();
        let expected_database_name: String = String::new();
        let expected_database_id: i64 = 0;
        let expected_created_timestamp: u64 = 0;
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.database_id().to_owned(), expected_database_id);
        assert_eq!(
            value.created_timestamp().to_owned(),
            expected_created_timestamp
        );
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn describe_database_response_populated_values() {
        let database_name = "database_name-value".to_owned();
        let database_id = 7;
        let created_timestamp = 7;
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = DescribeDatabaseResponse::builder()
            .database_name(database_name.clone())
            .database_id(database_id.clone())
            .created_timestamp(created_timestamp.clone())
            .properties(properties.clone())
            .build();

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.database_id().to_owned(), database_id);
        assert_eq!(value.created_timestamp().to_owned(), created_timestamp);
        assert_eq!(value.properties().to_owned(), properties);
    }
}
