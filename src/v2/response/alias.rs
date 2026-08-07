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

//! Response types returned by alias operations.

use crate::proto::milvus;

///////////////////////////////////////////////////////////////////////////////
// DescribeAliasResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_alias operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeAliasResponse {
    pub(crate) database_name: String,
    pub(crate) alias: String,
    pub(crate) collection_name: String,
}

impl DescribeAliasResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            database_name: String::new(),
            alias: String::new(),
            collection_name: String::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeAliasResponseBuilder {
        DescribeAliasResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn from_proto(value: milvus::DescribeAliasResponse) -> Self {
        Self {
            database_name: value.db_name,
            alias: value.alias,
            collection_name: value.collection,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeAliasResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeAliasResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeAliasResponseBuilder {
    value: DescribeAliasResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeAliasResponseBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn alias(mut self, value: impl Into<String>) -> Self {
        self.value.alias = value.into();
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn build(self) -> DescribeAliasResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListAliasesResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_aliases operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAliasesResponse {
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) aliases: Vec<String>,
}

impl ListAliasesResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            aliases: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListAliasesResponseBuilder {
        ListAliasesResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    pub(crate) fn from_proto(value: milvus::ListAliasesResponse) -> Self {
        Self {
            database_name: value.db_name,
            collection_name: value.collection_name,
            aliases: value.aliases,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListAliasesResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListAliasesResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListAliasesResponseBuilder {
    value: ListAliasesResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListAliasesResponseBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn aliases(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.aliases = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> ListAliasesResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod describe_alias_response_tests {
    use super::DescribeAliasResponse;
    use crate::proto::milvus;

    #[test]
    fn describe_alias_fields_are_exposed_directly() {
        let response = DescribeAliasResponse::from_proto(milvus::DescribeAliasResponse {
            db_name: "database".into(),
            alias: "alias".into(),
            collection: "collection".into(),
            ..Default::default()
        });

        assert_eq!(response.database_name().to_owned(), "database");
        assert_eq!(response.alias().to_owned(), "alias");
        assert_eq!(response.collection_name().to_owned(), "collection");
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn describe_alias_response_default_values() {
        let value = DescribeAliasResponse::builder().build();
        let expected_database_name: String = String::new();
        let expected_alias: String = String::new();
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.alias().to_owned(), expected_alias);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn describe_alias_response_populated_values() {
        let database_name = "database_name-value".to_owned();
        let alias = "alias-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = DescribeAliasResponse::builder()
            .database_name(database_name.clone())
            .alias(alias.clone())
            .collection_name(collection_name.clone())
            .build();

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.alias().to_owned(), alias);
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn list_aliases_response_default_values() {
        let value = ListAliasesResponse::builder().build();
        let expected_database_name: String = String::new();
        let expected_collection_name: String = String::new();
        let expected_aliases: Vec<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.aliases().to_owned(), expected_aliases);
    }

    #[test]
    fn list_aliases_response_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let aliases = vec!["aliases-value".to_owned()];
        let value = ListAliasesResponse::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .aliases(aliases.clone())
            .build();

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.aliases().to_owned(), aliases);
    }
}
