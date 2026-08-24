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

//! Request types for partition operations.

use crate::proto::milvus;
use crate::v2::error::Result;
use crate::v2::request::validation::{non_empty_strings, positive_i32, required, required_slice};

///////////////////////////////////////////////////////////////////////////////
// CreatePartitionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_partition operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreatePartitionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_name: String,
}

impl CreatePartitionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CreatePartitionRequestBuilder {
        CreatePartitionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreatePartitionRequestBuilder {
        CreatePartitionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition name.
    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::CreatePartitionRequest {
        milvus::CreatePartitionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_name: self.partition_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreatePartitionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreatePartitionRequest.
#[derive(Debug, Clone)]
pub struct CreatePartitionRequestBuilder {
    value: CreatePartitionRequest,
}

impl CreatePartitionRequestBuilder {
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

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreatePartitionRequest> {
        validate_partition_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
            &self.value.partition_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropPartitionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_partition operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropPartitionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_name: String,
}

impl DropPartitionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropPartitionRequestBuilder {
        DropPartitionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropPartitionRequestBuilder {
        DropPartitionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition name.
    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DropPartitionRequest {
        milvus::DropPartitionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_name: self.partition_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropPartitionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropPartitionRequest.
#[derive(Debug, Clone)]
pub struct DropPartitionRequestBuilder {
    value: DropPartitionRequest,
}

impl DropPartitionRequestBuilder {
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

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropPartitionRequest> {
        validate_partition_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
            &self.value.partition_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasPartitionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 has_partition operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HasPartitionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_name: String,
}

impl HasPartitionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> HasPartitionRequestBuilder {
        HasPartitionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> HasPartitionRequestBuilder {
        HasPartitionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition name.
    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::HasPartitionRequest {
        milvus::HasPartitionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_name: self.partition_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasPartitionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for HasPartitionRequest.
#[derive(Debug, Clone)]
pub struct HasPartitionRequestBuilder {
    value: HasPartitionRequest,
}

impl HasPartitionRequestBuilder {
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

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<HasPartitionRequest> {
        validate_partition_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
            &self.value.partition_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetPartitionStatsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_partition_stats operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetPartitionStatsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_name: String,
}

impl GetPartitionStatsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetPartitionStatsRequestBuilder {
        GetPartitionStatsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetPartitionStatsRequestBuilder {
        GetPartitionStatsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition name.
    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetPartitionStatisticsRequest {
        milvus::GetPartitionStatisticsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_name: self.partition_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetPartitionStatsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetPartitionStatsRequest.
#[derive(Debug, Clone)]
pub struct GetPartitionStatsRequestBuilder {
    value: GetPartitionStatsRequest,
}

impl GetPartitionStatsRequestBuilder {
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

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetPartitionStatsRequest> {
        validate_partition_target(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
            &self.value.partition_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPartitionsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_partitions operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPartitionsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListPartitionsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListPartitionsRequestBuilder {
        ListPartitionsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListPartitionsRequestBuilder {
        ListPartitionsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ShowPartitionsRequest {
        let mut value = milvus::ShowPartitionsRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPartitionsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListPartitionsRequest.
#[derive(Debug, Clone)]
pub struct ListPartitionsRequestBuilder {
    value: ListPartitionsRequest,
}

impl ListPartitionsRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListPartitionsRequest> {
        validate_partition_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// LoadPartitionsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 load_partitions operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoadPartitionsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
    pub(crate) sync: bool,
    pub(crate) replica_number: i32,
    /// Overall loading wait timeout in milliseconds. A value less than or
    /// equal to zero waits indefinitely.
    pub(crate) timeout_ms: i64,
    pub(crate) refresh: bool,
    pub(crate) load_fields: Vec<String>,
    pub(crate) skip_load_dynamic_field: bool,
    pub(crate) resource_groups: Vec<String>,
}

impl LoadPartitionsRequest {
    /// Creates a builder for this request.
    pub fn builder() -> LoadPartitionsRequestBuilder {
        LoadPartitionsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> LoadPartitionsRequestBuilder {
        LoadPartitionsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition names.
    pub fn partition_names(&self) -> &[String] {
        &self.partition_names
    }

    /// Returns whether sync.
    pub fn is_sync(&self) -> bool {
        self.sync
    }

    /// Returns the replica number.
    pub fn replica_number(&self) -> i32 {
        self.replica_number
    }

    /// Returns the timeout ms.
    pub fn timeout_ms(&self) -> i64 {
        self.timeout_ms
    }

    /// Returns whether the request should refresh.
    pub fn should_refresh(&self) -> bool {
        self.refresh
    }

    /// Returns the load fields.
    pub fn load_fields(&self) -> &[String] {
        &self.load_fields
    }

    /// Returns whether the request should skip load dynamic field.
    pub fn should_skip_load_dynamic_field(&self) -> bool {
        self.skip_load_dynamic_field
    }

    /// Returns the resource groups.
    pub fn resource_groups(&self) -> &[String] {
        &self.resource_groups
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::LoadPartitionsRequest {
        let mut value = milvus::LoadPartitionsRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value.partition_names = self.partition_names;
        value.replica_number = self.replica_number;
        value.refresh = self.refresh;
        value.resource_groups = self.resource_groups;
        value.load_fields = self.load_fields;
        value.skip_load_dynamic_field = self.skip_load_dynamic_field;
        value
    }
}

impl LoadPartitionsRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            partition_names: Vec::new(),
            sync: true,
            replica_number: 1,
            timeout_ms: 60_000,
            refresh: false,
            load_fields: Vec::new(),
            skip_load_dynamic_field: false,
            resource_groups: Vec::new(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// LoadPartitionsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for LoadPartitionsRequest.
#[derive(Debug, Clone)]
pub struct LoadPartitionsRequestBuilder {
    value: LoadPartitionsRequest,
}

impl LoadPartitionsRequestBuilder {
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

    /// Sets the partition names and returns the updated value.
    pub fn partition_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.partition_names.clear();
        for value in values.into_iter().map(Into::into) {
            if !self.value.partition_names.contains(&value) {
                self.value.partition_names.push(value);
            }
        }
        self
    }

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !self.value.partition_names.contains(&value) {
            self.value.partition_names.push(value);
        }
        self
    }

    /// Sets the sync and returns the updated value.
    pub fn sync(mut self, value: bool) -> Self {
        self.value.sync = value;
        self
    }

    /// Sets the replica number and returns the updated value.
    pub fn replica_number(mut self, value: i32) -> Self {
        self.value.replica_number = value;
        self
    }

    /// Sets the timeout ms and returns the updated value.
    pub fn timeout_ms(mut self, value: i64) -> Self {
        self.value.timeout_ms = value;
        self
    }

    /// Sets the refresh and returns the updated value.
    pub fn refresh(mut self, value: bool) -> Self {
        self.value.refresh = value;
        self
    }

    /// Sets the load fields and returns the updated value.
    pub fn load_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.load_fields.clear();
        for value in values.into_iter().map(Into::into) {
            if !self.value.load_fields.contains(&value) {
                self.value.load_fields.push(value);
            }
        }
        self
    }

    /// Sets the load field and returns the updated value.
    pub fn load_field(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !self.value.load_fields.contains(&value) {
            self.value.load_fields.push(value);
        }
        self
    }

    /// Sets the skip load dynamic field and returns the updated value.
    pub fn skip_load_dynamic_field(mut self, value: bool) -> Self {
        self.value.skip_load_dynamic_field = value;
        self
    }

    /// Sets the resource groups and returns the updated value.
    pub fn resource_groups(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.resource_groups.clear();
        for value in values.into_iter().map(Into::into) {
            if !self.value.resource_groups.contains(&value) {
                self.value.resource_groups.push(value);
            }
        }
        self
    }

    /// Sets the resource group and returns the updated value.
    pub fn resource_group(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !self.value.resource_groups.contains(&value) {
            self.value.resource_groups.push(value);
        }
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<LoadPartitionsRequest> {
        validate_partition_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required_slice("partition_names", &self.value.partition_names)?;
        non_empty_strings("partition_names", &self.value.partition_names)?;
        positive_i32("replica_number", self.value.replica_number)?;
        non_empty_strings("load_fields", &self.value.load_fields)?;
        non_empty_strings("resource_groups", &self.value.resource_groups)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReleasePartitionsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 release_partitions operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReleasePartitionsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
}

impl ReleasePartitionsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_names: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ReleasePartitionsRequestBuilder {
        ReleasePartitionsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ReleasePartitionsRequestBuilder {
        ReleasePartitionsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition names.
    pub fn partition_names(&self) -> &[String] {
        &self.partition_names
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ReleasePartitionsRequest {
        milvus::ReleasePartitionsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_names: self.partition_names,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReleasePartitionsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ReleasePartitionsRequest.
#[derive(Debug, Clone)]
pub struct ReleasePartitionsRequestBuilder {
    value: ReleasePartitionsRequest,
}

impl ReleasePartitionsRequestBuilder {
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

    /// Sets the partition names and returns the updated value.
    pub fn partition_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.partition_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ReleasePartitionsRequest> {
        validate_partition_collection(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required_slice("partition_names", &self.value.partition_names)?;
        non_empty_strings("partition_names", &self.value.partition_names)?;
        Ok(self.value)
    }
}

fn validate_partition_collection(
    _database_name: Option<&str>,
    collection_name: &str,
) -> Result<()> {
    required("collection_name", collection_name)
}

fn validate_partition_target(
    database_name: Option<&str>,
    collection_name: &str,
    partition_name: &str,
) -> Result<()> {
    validate_partition_collection(database_name, collection_name)?;
    required("partition_name", partition_name)
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::LoadPartitionsRequest;

    #[test]
    fn load_partitions_defaults_match_sync_wait_behavior() {
        let request = LoadPartitionsRequest::builder()
            .collection_name("books")
            .partition_names(["default"])
            .build()
            .expect("valid request");
        assert!(request.is_sync());
        assert_eq!(request.replica_number().to_owned(), 1);
        assert_eq!(request.timeout_ms().to_owned(), 60_000);
        assert!(!request.should_refresh());
        assert!(!request.should_skip_load_dynamic_field());
    }

    #[test]
    fn load_partitions_converts_wire_fields_and_keeps_wait_fields_sdk_only() {
        let request = LoadPartitionsRequest::builder()
            .collection_name("books")
            .partition_names(["history", "science", "history"])
            .partition_name("fiction")
            .sync(false)
            .replica_number(2)
            .timeout_ms(120_000)
            .refresh(true)
            .load_fields(["id", "vector", "id"])
            .load_field("title")
            .skip_load_dynamic_field(true)
            .resource_groups(["rg1", "rg2", "rg1"])
            .resource_group("rg3")
            .build()
            .expect("valid request");

        assert!(!request.is_sync());
        assert_eq!(request.timeout_ms().to_owned(), 120_000);
        assert_eq!(
            request.partition_names(),
            &["history", "science", "fiction"]
        );
        assert_eq!(request.load_fields().to_owned(), ["id", "vector", "title"]);
        assert_eq!(request.resource_groups().to_owned(), ["rg1", "rg2", "rg3"]);

        let proto = request.into_proto("catalog");
        assert_eq!(proto.db_name, "catalog");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.partition_names, ["history", "science", "fiction"]);
        assert_eq!(proto.replica_number, 2);
        assert!(proto.refresh);
        assert_eq!(proto.load_fields, ["id", "vector", "title"]);
        assert!(proto.skip_load_dynamic_field);
        assert_eq!(proto.resource_groups, ["rg1", "rg2", "rg3"]);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn create_partition_request_default_values() {
        let value = CreatePartitionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_name().to_owned(), expected_partition_name);
    }

    #[test]
    fn create_partition_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_name = "partition_name-value".to_owned();
        let value = CreatePartitionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_name(partition_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_name().to_owned(), partition_name);
    }

    #[test]
    fn drop_partition_request_default_values() {
        let value = DropPartitionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_name().to_owned(), expected_partition_name);
    }

    #[test]
    fn drop_partition_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_name = "partition_name-value".to_owned();
        let value = DropPartitionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_name(partition_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_name().to_owned(), partition_name);
    }

    #[test]
    fn has_partition_request_default_values() {
        let value = HasPartitionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_name().to_owned(), expected_partition_name);
    }

    #[test]
    fn has_partition_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_name = "partition_name-value".to_owned();
        let value = HasPartitionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_name(partition_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_name().to_owned(), partition_name);
    }

    #[test]
    fn get_partition_stats_request_default_values() {
        let value = GetPartitionStatsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_name().to_owned(), expected_partition_name);
    }

    #[test]
    fn get_partition_stats_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_name = "partition_name-value".to_owned();
        let value = GetPartitionStatsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_name(partition_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_name().to_owned(), partition_name);
    }

    #[test]
    fn list_partitions_request_default_values() {
        let value = ListPartitionsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn list_partitions_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = ListPartitionsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn load_partitions_request_default_values() {
        let value = LoadPartitionsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_names: Vec<String> = Default::default();
        let expected_sync: bool = true;
        let expected_replica_number: i32 = 1;
        let expected_refresh: bool = false;
        let expected_load_fields: Vec<String> = Default::default();
        let expected_skip_load_dynamic_field: bool = false;
        let expected_resource_groups: Vec<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
        assert_eq!(value.is_sync().to_owned(), expected_sync);
        assert_eq!(value.replica_number().to_owned(), expected_replica_number);
        assert_eq!(value.should_refresh().to_owned(), expected_refresh);
        assert_eq!(value.load_fields().to_owned(), expected_load_fields);
        assert_eq!(
            value.should_skip_load_dynamic_field(),
            expected_skip_load_dynamic_field
        );
        assert_eq!(value.resource_groups().to_owned(), expected_resource_groups);
    }

    #[test]
    fn load_partitions_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_names = vec!["partition_names-value".to_owned()];
        let sync = true;
        let replica_number = 7;
        let refresh = true;
        let load_fields = vec!["load_fields-value".to_owned()];
        let skip_load_dynamic_field = true;
        let resource_groups = vec!["resource_groups-value".to_owned()];
        let value = LoadPartitionsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_names(partition_names.clone())
            .sync(sync.clone())
            .replica_number(replica_number.clone())
            .refresh(refresh.clone())
            .load_fields(load_fields.clone())
            .skip_load_dynamic_field(skip_load_dynamic_field.clone())
            .resource_groups(resource_groups.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_names().to_owned(), partition_names);
        assert_eq!(value.is_sync().to_owned(), sync);
        assert_eq!(value.replica_number().to_owned(), replica_number);
        assert_eq!(value.should_refresh().to_owned(), refresh);
        assert_eq!(value.load_fields().to_owned(), load_fields);
        assert_eq!(
            value.should_skip_load_dynamic_field(),
            skip_load_dynamic_field
        );
        assert_eq!(value.resource_groups().to_owned(), resource_groups);
    }

    #[test]
    fn release_partitions_request_default_values() {
        let value = ReleasePartitionsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_names: Vec<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
    }

    #[test]
    fn release_partitions_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_names = vec!["partition_names-value".to_owned()];
        let value = ReleasePartitionsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_names(partition_names.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_names().to_owned(), partition_names);
    }
}
