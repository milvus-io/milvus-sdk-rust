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

//! Request types for alias operations.

use crate::proto::milvus;
use crate::v2::error::Result;
use crate::v2::request::validation::required;

///////////////////////////////////////////////////////////////////////////////
// CreateAliasRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_alias operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateAliasRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) alias: String,
}

impl CreateAliasRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            alias: Default::default(),
        }
    }

    pub fn builder() -> CreateAliasRequestBuilder {
        CreateAliasRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateAliasRequestBuilder {
        CreateAliasRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::CreateAliasRequest {
        milvus::CreateAliasRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            alias: self.alias,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateAliasRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateAliasRequest.
#[derive(Debug, Clone)]
pub struct CreateAliasRequestBuilder {
    value: CreateAliasRequest,
}

impl CreateAliasRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn alias(mut self, value: impl Into<String>) -> Self {
        self.value.alias = value.into();
        self
    }

    pub fn build(self) -> Result<CreateAliasRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("alias", &self.value.alias)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterAliasRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_alias operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AlterAliasRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) alias: String,
}

impl AlterAliasRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            alias: Default::default(),
        }
    }

    pub fn builder() -> AlterAliasRequestBuilder {
        AlterAliasRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterAliasRequestBuilder {
        AlterAliasRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::AlterAliasRequest {
        milvus::AlterAliasRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            alias: self.alias,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterAliasRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterAliasRequest.
#[derive(Debug, Clone)]
pub struct AlterAliasRequestBuilder {
    value: AlterAliasRequest,
}

impl AlterAliasRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn alias(mut self, value: impl Into<String>) -> Self {
        self.value.alias = value.into();
        self
    }

    pub fn build(self) -> Result<AlterAliasRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("alias", &self.value.alias)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropAliasRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_alias operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropAliasRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) alias: String,
}

impl DropAliasRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            alias: Default::default(),
        }
    }

    pub fn builder() -> DropAliasRequestBuilder {
        DropAliasRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropAliasRequestBuilder {
        DropAliasRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DropAliasRequest {
        milvus::DropAliasRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            alias: self.alias,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropAliasRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropAliasRequest.
#[derive(Debug, Clone)]
pub struct DropAliasRequestBuilder {
    value: DropAliasRequest,
}

impl DropAliasRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    pub fn alias(mut self, value: impl Into<String>) -> Self {
        self.value.alias = value.into();
        self
    }

    pub fn build(self) -> Result<DropAliasRequest> {
        required("alias", &self.value.alias)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeAliasRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_alias operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeAliasRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) alias: String,
}

impl DescribeAliasRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            alias: Default::default(),
        }
    }

    pub fn builder() -> DescribeAliasRequestBuilder {
        DescribeAliasRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeAliasRequestBuilder {
        DescribeAliasRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DescribeAliasRequest {
        milvus::DescribeAliasRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            alias: self.alias,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeAliasRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeAliasRequest.
#[derive(Debug, Clone)]
pub struct DescribeAliasRequestBuilder {
    value: DescribeAliasRequest,
}

impl DescribeAliasRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    pub fn alias(mut self, value: impl Into<String>) -> Self {
        self.value.alias = value.into();
        self
    }

    pub fn build(self) -> Result<DescribeAliasRequest> {
        required("alias", &self.value.alias)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListAliasesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_aliases operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAliasesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListAliasesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    pub fn builder() -> ListAliasesRequestBuilder {
        ListAliasesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListAliasesRequestBuilder {
        ListAliasesRequestBuilder { value: self }
    }

    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ListAliasesRequest {
        milvus::ListAliasesRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListAliasesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListAliasesRequest.
#[derive(Debug, Clone)]
pub struct ListAliasesRequestBuilder {
    value: ListAliasesRequest,
}

impl ListAliasesRequestBuilder {
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    pub fn build(self) -> Result<ListAliasesRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_request_methods_and_conversions() {
        let create = CreateAliasRequest::builder()
            .database_name("db")
            .collection_name("books")
            .alias("featured")
            .build()
            .expect("valid request");
        assert_eq!(create.database_name().as_deref().to_owned(), Some("db"));
        assert_eq!(create.collection_name().to_owned(), "books");
        assert_eq!(create.alias().to_owned(), "featured");
        let proto = create.into_proto("default");
        assert_eq!(
            (
                proto.db_name.as_str(),
                proto.collection_name.as_str(),
                proto.alias.as_str()
            ),
            ("db", "books", "featured")
        );

        let alter = AlterAliasRequest::builder()
            .database_name("db")
            .collection_name("archive")
            .alias("featured")
            .build()
            .expect("valid request");
        assert_eq!(alter.database_name().as_deref().to_owned(), Some("db"));
        assert_eq!(alter.collection_name().to_owned(), "archive");
        assert_eq!(alter.alias().to_owned(), "featured");
        assert_eq!(alter.into_proto("default").collection_name, "archive");

        let drop = DropAliasRequest::builder()
            .database_name("db")
            .alias("featured")
            .build()
            .expect("valid request");
        assert_eq!(drop.database_name().as_deref().to_owned(), Some("db"));
        assert_eq!(drop.alias().to_owned(), "featured");
        assert_eq!(drop.into_proto("default").alias, "featured");

        let describe = DescribeAliasRequest::builder()
            .database_name("db")
            .alias("featured")
            .build()
            .expect("valid request");
        assert_eq!(describe.database_name().as_deref().to_owned(), Some("db"));
        assert_eq!(describe.alias().to_owned(), "featured");
        assert_eq!(describe.into_proto("default").db_name, "db");

        let list = ListAliasesRequest::builder()
            .database_name("db")
            .collection_name("books")
            .build()
            .expect("valid request");
        assert_eq!(list.database_name().as_deref().to_owned(), Some("db"));
        assert_eq!(list.collection_name().to_owned(), "books");
        assert_eq!(list.into_proto("default").collection_name, "books");

        assert_eq!(
            CreateAliasRequest::builder()
                .collection_name("books")
                .alias("featured")
                .build()
                .expect("valid request")
                .into_proto("default")
                .db_name,
            "default"
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
    fn create_alias_request_default_values() {
        let value = CreateAliasRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_alias: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.alias().to_owned(), expected_alias);
    }

    #[test]
    fn create_alias_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let alias = "alias-value".to_owned();
        let value = CreateAliasRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .alias(alias.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.alias().to_owned(), alias);
    }

    #[test]
    fn alter_alias_request_default_values() {
        let value = AlterAliasRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_alias: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.alias().to_owned(), expected_alias);
    }

    #[test]
    fn alter_alias_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let alias = "alias-value".to_owned();
        let value = AlterAliasRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .alias(alias.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.alias().to_owned(), alias);
    }

    #[test]
    fn drop_alias_request_default_values() {
        let value = DropAliasRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_alias: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.alias().to_owned(), expected_alias);
    }

    #[test]
    fn drop_alias_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let alias = "alias-value".to_owned();
        let value = DropAliasRequest::builder()
            .database_name(database_name.clone())
            .alias(alias.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.alias().to_owned(), alias);
    }

    #[test]
    fn describe_alias_request_default_values() {
        let value = DescribeAliasRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_alias: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.alias().to_owned(), expected_alias);
    }

    #[test]
    fn describe_alias_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let alias = "alias-value".to_owned();
        let value = DescribeAliasRequest::builder()
            .database_name(database_name.clone())
            .alias(alias.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.alias().to_owned(), alias);
    }

    #[test]
    fn list_aliases_request_default_values() {
        let value = ListAliasesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn list_aliases_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = ListAliasesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }
}
