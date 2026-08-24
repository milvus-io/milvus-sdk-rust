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

//! Request types for resource-group operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::{positive_i32, positive_i64, required};
pub use crate::v2::types::ResourceGroupConfig;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// CreateResourceGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_resource_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateResourceGroupRequest {
    pub(crate) name: String,
    pub(crate) config: ResourceGroupConfig,
}

impl CreateResourceGroupRequest {
    fn empty() -> Self {
        Self {
            name: String::new(),
            config: ResourceGroupConfig::new(),
        }
    }
}

impl CreateResourceGroupRequest {
    /// Creates a builder for this request.
    pub fn builder() -> CreateResourceGroupRequestBuilder {
        CreateResourceGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateResourceGroupRequestBuilder {
        CreateResourceGroupRequestBuilder { value: self }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the config.
    pub fn config(&self) -> &ResourceGroupConfig {
        &self.config
    }

    pub(crate) fn into_proto(self) -> milvus::CreateResourceGroupRequest {
        milvus::CreateResourceGroupRequest {
            base: None,
            resource_group: self.name,
            config: Some(self.config.into_proto()),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateResourceGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateResourceGroupRequest.
#[derive(Debug, Clone)]
pub struct CreateResourceGroupRequestBuilder {
    value: CreateResourceGroupRequest,
}

impl CreateResourceGroupRequestBuilder {
    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.value.name = value.into();
        self
    }

    /// Sets the config and returns the updated value.
    pub fn config(mut self, value: ResourceGroupConfig) -> Self {
        self.value.config = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateResourceGroupRequest> {
        required("name", &self.value.name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropResourceGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_resource_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropResourceGroupRequest {
    pub(crate) group_name: String,
}

impl DropResourceGroupRequest {
    fn empty() -> Self {
        Self {
            group_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropResourceGroupRequestBuilder {
        DropResourceGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropResourceGroupRequestBuilder {
        DropResourceGroupRequestBuilder { value: self }
    }

    /// Returns the group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    pub(crate) fn into_proto(self) -> milvus::DropResourceGroupRequest {
        milvus::DropResourceGroupRequest {
            base: None,
            resource_group: self.group_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropResourceGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropResourceGroupRequest.
#[derive(Debug, Clone)]
pub struct DropResourceGroupRequestBuilder {
    value: DropResourceGroupRequest,
}

impl DropResourceGroupRequestBuilder {
    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.value.group_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropResourceGroupRequest> {
        required("group_name", &self.value.group_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateResourceGroupsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 update_resource_groups operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpdateResourceGroupsRequest {
    pub(crate) groups: HashMap<String, ResourceGroupConfig>,
}

impl UpdateResourceGroupsRequest {
    fn empty() -> Self {
        Self {
            groups: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> UpdateResourceGroupsRequestBuilder {
        UpdateResourceGroupsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UpdateResourceGroupsRequestBuilder {
        UpdateResourceGroupsRequestBuilder { value: self }
    }

    /// Returns the groups.
    pub fn groups(&self) -> &HashMap<String, ResourceGroupConfig> {
        &self.groups
    }

    pub(crate) fn into_proto(self) -> milvus::UpdateResourceGroupsRequest {
        milvus::UpdateResourceGroupsRequest {
            base: None,
            resource_groups: self
                .groups
                .into_iter()
                .map(|(n, c)| (n, c.into_proto()))
                .collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateResourceGroupsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UpdateResourceGroupsRequest.
#[derive(Debug, Clone)]
pub struct UpdateResourceGroupsRequestBuilder {
    value: UpdateResourceGroupsRequest,
}

impl UpdateResourceGroupsRequestBuilder {
    /// Sets the groups and returns the updated value.
    pub fn groups(mut self, value: HashMap<String, ResourceGroupConfig>) -> Self {
        self.value.groups = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<UpdateResourceGroupsRequest> {
        if self.value.groups.is_empty() {
            return Err(Error::validation(
                "groups".into(),
                "must contain at least one resource group".into(),
            ));
        }
        if self.value.groups.keys().any(String::is_empty) {
            return Err(Error::validation(
                "groups".into(),
                "resource group names must not be empty".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// TransferNodeRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 transfer_node operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransferNodeRequest {
    pub(crate) source_group: String,
    pub(crate) target_group: String,
    pub(crate) node_count: i32,
}

impl TransferNodeRequest {
    fn empty() -> Self {
        Self {
            source_group: Default::default(),
            target_group: Default::default(),
            node_count: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> TransferNodeRequestBuilder {
        TransferNodeRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> TransferNodeRequestBuilder {
        TransferNodeRequestBuilder { value: self }
    }

    /// Returns the source group.
    pub fn source_group(&self) -> &str {
        &self.source_group
    }

    /// Returns the target group.
    pub fn target_group(&self) -> &str {
        &self.target_group
    }

    /// Returns the node count.
    pub fn node_count(&self) -> i32 {
        self.node_count
    }

    pub(crate) fn into_proto(self) -> milvus::TransferNodeRequest {
        milvus::TransferNodeRequest {
            base: None,
            source_resource_group: self.source_group,
            target_resource_group: self.target_group,
            num_node: self.node_count,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// TransferNodeRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for TransferNodeRequest.
#[derive(Debug, Clone)]
pub struct TransferNodeRequestBuilder {
    value: TransferNodeRequest,
}

impl TransferNodeRequestBuilder {
    /// Sets the source group and returns the updated value.
    pub fn source_group(mut self, value: impl Into<String>) -> Self {
        self.value.source_group = value.into();
        self
    }

    /// Sets the target group and returns the updated value.
    pub fn target_group(mut self, value: impl Into<String>) -> Self {
        self.value.target_group = value.into();
        self
    }

    /// Sets the node count and returns the updated value.
    pub fn node_count(mut self, value: i32) -> Self {
        self.value.node_count = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<TransferNodeRequest> {
        required("source_group", &self.value.source_group)?;
        required("target_group", &self.value.target_group)?;
        positive_i32("node_count", self.value.node_count)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// TransferReplicaRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 transfer_replica operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransferReplicaRequest {
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) source_group: String,
    pub(crate) target_group: String,
    pub(crate) replica_count: i64,
}

impl TransferReplicaRequest {
    /// Creates a builder for this request.
    pub fn builder() -> TransferReplicaRequestBuilder {
        TransferReplicaRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> TransferReplicaRequestBuilder {
        TransferReplicaRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the source group.
    pub fn source_group(&self) -> &str {
        &self.source_group
    }

    /// Returns the target group.
    pub fn target_group(&self) -> &str {
        &self.target_group
    }

    /// Returns the replica count.
    pub fn replica_count(&self) -> i64 {
        self.replica_count
    }

    pub(crate) fn into_proto(self) -> milvus::TransferReplicaRequest {
        milvus::TransferReplicaRequest {
            base: None,
            source_resource_group: self.source_group,
            target_resource_group: self.target_group,
            collection_name: self.collection_name,
            num_replica: self.replica_count,
            db_name: self.database_name,
            ..Default::default()
        }
    }
}

impl TransferReplicaRequest {
    fn empty() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            source_group: String::new(),
            target_group: String::new(),
            replica_count: 1,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// TransferReplicaRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for TransferReplicaRequest.
#[derive(Debug, Clone)]
pub struct TransferReplicaRequestBuilder {
    value: TransferReplicaRequest,
}

impl TransferReplicaRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the source group and returns the updated value.
    pub fn source_group(mut self, value: impl Into<String>) -> Self {
        self.value.source_group = value.into();
        self
    }

    /// Sets the target group and returns the updated value.
    pub fn target_group(mut self, value: impl Into<String>) -> Self {
        self.value.target_group = value.into();
        self
    }

    /// Sets the replica count and returns the updated value.
    pub fn replica_count(mut self, value: i64) -> Self {
        self.value.replica_count = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<TransferReplicaRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("source_group", &self.value.source_group)?;
        required("target_group", &self.value.target_group)?;
        positive_i64("replica_count", self.value.replica_count)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListResourceGroupsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_resource_groups operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListResourceGroupsRequest;

impl ListResourceGroupsRequest {
    /// Creates a builder for this request.
    pub fn builder() -> ListResourceGroupsRequestBuilder {
        ListResourceGroupsRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListResourceGroupsRequestBuilder {
        ListResourceGroupsRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::ListResourceGroupsRequest {
        milvus::ListResourceGroupsRequest::default()
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListResourceGroupsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListResourceGroupsRequest.
#[derive(Debug, Clone, Copy)]
pub struct ListResourceGroupsRequestBuilder;

impl ListResourceGroupsRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListResourceGroupsRequest> {
        Ok(ListResourceGroupsRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeResourceGroupRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_resource_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeResourceGroupRequest {
    pub(crate) group_name: String,
}

impl DescribeResourceGroupRequest {
    fn empty() -> Self {
        Self {
            group_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DescribeResourceGroupRequestBuilder {
        DescribeResourceGroupRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeResourceGroupRequestBuilder {
        DescribeResourceGroupRequestBuilder { value: self }
    }

    /// Returns the group name.
    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    pub(crate) fn into_proto(self) -> milvus::DescribeResourceGroupRequest {
        milvus::DescribeResourceGroupRequest {
            base: None,
            resource_group: self.group_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeResourceGroupRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeResourceGroupRequest.
#[derive(Debug, Clone)]
pub struct DescribeResourceGroupRequestBuilder {
    value: DescribeResourceGroupRequest,
}

impl DescribeResourceGroupRequestBuilder {
    /// Sets the group name and returns the updated value.
    pub fn group_name(mut self, value: impl Into<String>) -> Self {
        self.value.group_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeResourceGroupRequest> {
        required("group_name", &self.value.group_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ResourceGroupConfig {
        ResourceGroupConfig::new()
            .requested_nodes(2)
            .node_limit(4)
            .transfer_from(["default"])
            .transfer_to(["backup"])
            .node_labels(HashMap::from([("zone".into(), "east".into())]))
    }

    #[test]
    fn resource_group_request_methods_and_conversions() {
        let create = CreateResourceGroupRequest::builder()
            .name("analytics")
            .config(config())
            .build()
            .expect("valid request");
        assert_eq!(create.name().to_owned(), "analytics");
        assert_eq!(create.config().get_requested_nodes().to_owned(), 2);
        let proto = create.into_proto();
        assert_eq!(proto.resource_group, "analytics");
        assert_eq!(proto.config.unwrap().requests.unwrap().node_num, 2);

        let drop = DropResourceGroupRequest::builder()
            .group_name("old")
            .build()
            .expect("valid request");
        assert_eq!(drop.group_name().to_owned(), "old");
        assert_eq!(drop.into_proto().resource_group, "old");

        let update = UpdateResourceGroupsRequest::builder()
            .groups(HashMap::from([("analytics".into(), config())]))
            .build()
            .expect("valid request");
        assert!(update.groups().contains_key("analytics"));
        assert!(update
            .into_proto()
            .resource_groups
            .contains_key("analytics"));

        let nodes = TransferNodeRequest::builder()
            .source_group("default")
            .target_group("analytics")
            .node_count(3)
            .build()
            .expect("valid request");
        assert_eq!(nodes.source_group().to_owned(), "default");
        assert_eq!(nodes.target_group().to_owned(), "analytics");
        assert_eq!(nodes.node_count().to_owned(), 3);
        assert_eq!(nodes.into_proto().num_node, 3);

        let replicas = TransferReplicaRequest::builder()
            .database_name("db")
            .collection_name("books")
            .source_group("default")
            .target_group("analytics")
            .replica_count(2)
            .build()
            .expect("valid request");
        assert_eq!(replicas.database_name().to_owned(), "db");
        assert_eq!(replicas.collection_name().to_owned(), "books");
        assert_eq!(replicas.source_group().to_owned(), "default");
        assert_eq!(replicas.target_group().to_owned(), "analytics");
        assert_eq!(replicas.replica_count().to_owned(), 2);
        assert_eq!(replicas.into_proto().num_replica, 2);
        assert_eq!(
            TransferReplicaRequest::empty().replica_count().to_owned(),
            1
        );

        let list = ListResourceGroupsRequest::builder()
            .build()
            .expect("valid request")
            .into_proto();
        assert!(list.base.is_none());

        let describe = DescribeResourceGroupRequest::builder()
            .group_name("analytics")
            .build()
            .expect("valid request");
        assert_eq!(describe.group_name().to_owned(), "analytics");
        assert_eq!(describe.into_proto().resource_group, "analytics");
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn list_resource_groups_request_default_values() {
        assert_eq!(
            ListResourceGroupsRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::ListResourceGroupsRequest::default()
        );
    }

    #[test]
    fn list_resource_groups_request_populated_values() {
        let value = ListResourceGroupsRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(
            value.into_proto(),
            milvus::ListResourceGroupsRequest::default()
        );
    }

    #[test]
    fn create_resource_group_request_default_values() {
        let value = CreateResourceGroupRequest::empty();
        let expected_name: String = String::new();
        let expected_config = ResourceGroupConfig::new();

        assert_eq!(value.name().to_owned(), expected_name);
        assert_eq!(value.config().to_owned(), expected_config);
        let proto = value.into_proto();
        let config = proto.config.unwrap();
        assert_eq!(config.requests.unwrap().node_num, 0);
        assert_eq!(config.limits.unwrap().node_num, 0);
    }

    #[test]
    fn create_resource_group_request_populated_values() {
        let name = "name-value".to_owned();
        let config = ResourceGroupConfig::new();
        let value = CreateResourceGroupRequest::builder()
            .name(name.clone())
            .config(config.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.name().to_owned(), name);
        assert_eq!(value.config().to_owned(), config);
    }

    #[test]
    fn drop_resource_group_request_default_values() {
        let value = DropResourceGroupRequest::empty();
        let expected_group_name: String = String::new();

        assert_eq!(value.group_name().to_owned(), expected_group_name);
    }

    #[test]
    fn drop_resource_group_request_populated_values() {
        let group_name = "group_name-value".to_owned();
        let value = DropResourceGroupRequest::builder()
            .group_name(group_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.group_name().to_owned(), group_name);
    }

    #[test]
    fn update_resource_groups_request_default_values() {
        let value = UpdateResourceGroupsRequest::empty();
        let expected_groups: HashMap<String, ResourceGroupConfig> = Default::default();

        assert_eq!(value.groups().to_owned(), expected_groups);
    }

    #[test]
    fn update_resource_groups_request_populated_values() {
        let groups = HashMap::from([("key-value".to_owned(), ResourceGroupConfig::new())]);
        let value = UpdateResourceGroupsRequest::builder()
            .groups(groups.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.groups().to_owned(), groups);
    }

    #[test]
    fn transfer_node_request_default_values() {
        let value = TransferNodeRequest::empty();
        let expected_source_group: String = String::new();
        let expected_target_group: String = String::new();
        let expected_node_count: i32 = 0;

        assert_eq!(value.source_group().to_owned(), expected_source_group);
        assert_eq!(value.target_group().to_owned(), expected_target_group);
        assert_eq!(value.node_count().to_owned(), expected_node_count);
    }

    #[test]
    fn transfer_node_request_populated_values() {
        let source_group = "source_group-value".to_owned();
        let target_group = "target_group-value".to_owned();
        let node_count = 7;
        let value = TransferNodeRequest::builder()
            .source_group(source_group.clone())
            .target_group(target_group.clone())
            .node_count(node_count.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.source_group().to_owned(), source_group);
        assert_eq!(value.target_group().to_owned(), target_group);
        assert_eq!(value.node_count().to_owned(), node_count);
    }

    #[test]
    fn transfer_replica_request_default_values() {
        let value = TransferReplicaRequest::empty();
        let expected_database_name: String = String::new();
        let expected_collection_name: String = String::new();
        let expected_source_group: String = String::new();
        let expected_target_group: String = String::new();
        let expected_replica_count: i64 = 1;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.source_group().to_owned(), expected_source_group);
        assert_eq!(value.target_group().to_owned(), expected_target_group);
        assert_eq!(value.replica_count().to_owned(), expected_replica_count);
    }

    #[test]
    fn transfer_replica_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let source_group = "source_group-value".to_owned();
        let target_group = "target_group-value".to_owned();
        let replica_count = 7;
        let value = TransferReplicaRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .source_group(source_group.clone())
            .target_group(target_group.clone())
            .replica_count(replica_count.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), database_name);
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.source_group().to_owned(), source_group);
        assert_eq!(value.target_group().to_owned(), target_group);
        assert_eq!(value.replica_count().to_owned(), replica_count);
    }

    #[test]
    fn describe_resource_group_request_default_values() {
        let value = DescribeResourceGroupRequest::empty();
        let expected_group_name: String = String::new();

        assert_eq!(value.group_name().to_owned(), expected_group_name);
    }

    #[test]
    fn describe_resource_group_request_populated_values() {
        let group_name = "group_name-value".to_owned();
        let value = DescribeResourceGroupRequest::builder()
            .group_name(group_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.group_name().to_owned(), group_name);
    }
}
