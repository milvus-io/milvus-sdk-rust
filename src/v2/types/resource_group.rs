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

//! Resource-group configuration and status types.

use crate::proto::{common, rg};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// ResourceGroupConfig
///////////////////////////////////////////////////////////////////////////////
/// Configuration for a Milvus resource group.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResourceGroupConfig {
    pub(crate) requested_nodes: i32,
    pub(crate) node_limit: i32,
    pub(crate) transfer_from: Vec<String>,
    pub(crate) transfer_to: Vec<String>,
    pub(crate) node_labels: HashMap<String, String>,
}

impl ResourceGroupConfig {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            requested_nodes: 0,
            node_limit: 0,
            transfer_from: Vec::new(),
            transfer_to: Vec::new(),
            node_labels: HashMap::new(),
        }
    }

    pub(crate) fn from_proto(v: rg::ResourceGroupConfig) -> Self {
        Self {
            requested_nodes: v.requests.map_or(0, |v| v.node_num),
            node_limit: v.limits.map_or(0, |v| v.node_num),
            transfer_from: v
                .transfer_from
                .into_iter()
                .map(|v| v.resource_group)
                .collect(),
            transfer_to: v
                .transfer_to
                .into_iter()
                .map(|v| v.resource_group)
                .collect(),
            node_labels: v
                .node_filter
                .map(|v| {
                    v.node_labels
                        .into_iter()
                        .map(|p| (p.key, p.value))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

impl ResourceGroupConfig {
    /// Sets the requested nodes and returns the updated value.
    pub fn requested_nodes(mut self, value: i32) -> Self {
        self.requested_nodes = value;
        self
    }

    /// Sets the requested nodes and returns this value for further mutation.
    pub fn set_requested_nodes(&mut self, value: i32) -> &mut Self {
        self.requested_nodes = value;
        self
    }

    /// Returns the configured requested nodes.
    pub fn get_requested_nodes(&self) -> i32 {
        self.requested_nodes
    }

    /// Sets the node limit and returns the updated value.
    pub fn node_limit(mut self, value: i32) -> Self {
        self.node_limit = value;
        self
    }

    /// Sets the node limit and returns this value for further mutation.
    pub fn set_node_limit(&mut self, value: i32) -> &mut Self {
        self.node_limit = value;
        self
    }

    /// Returns the configured node limit.
    pub fn get_node_limit(&self) -> i32 {
        self.node_limit
    }

    /// Sets the transfer from and returns the updated value.
    pub fn transfer_from(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.transfer_from = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the transfer from and returns this value for further mutation.
    pub fn set_transfer_from(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.transfer_from = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured transfer from.
    pub fn get_transfer_from(&self) -> &[String] {
        &self.transfer_from
    }

    /// Sets the transfer to and returns the updated value.
    pub fn transfer_to(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.transfer_to = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the transfer to and returns this value for further mutation.
    pub fn set_transfer_to(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.transfer_to = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured transfer to.
    pub fn get_transfer_to(&self) -> &[String] {
        &self.transfer_to
    }

    /// Sets the node labels and returns the updated value.
    pub fn node_labels(mut self, value: HashMap<String, String>) -> Self {
        self.node_labels = value;
        self
    }

    /// Sets the node labels and returns this value for further mutation.
    pub fn set_node_labels(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.node_labels = value;
        self
    }

    /// Returns the configured node labels.
    pub fn get_node_labels(&self) -> &HashMap<String, String> {
        &self.node_labels
    }

    /// Adds one add transfer from to the existing values.
    pub fn add_transfer_from(mut self, value: impl Into<String>) -> Self {
        self.transfer_from.push(value.into());
        self
    }

    /// Adds one add transfer to to the existing values.
    pub fn add_transfer_to(mut self, value: impl Into<String>) -> Self {
        self.transfer_to.push(value.into());
        self
    }

    pub(crate) fn into_proto(self) -> rg::ResourceGroupConfig {
        rg::ResourceGroupConfig {
            requests: Some(rg::ResourceGroupLimit {
                node_num: self.requested_nodes,
            }),
            limits: Some(rg::ResourceGroupLimit {
                node_num: self.node_limit,
            }),
            transfer_from: self
                .transfer_from
                .into_iter()
                .map(|resource_group| rg::ResourceGroupTransfer { resource_group })
                .collect(),
            transfer_to: self
                .transfer_to
                .into_iter()
                .map(|resource_group| rg::ResourceGroupTransfer { resource_group })
                .collect(),
            node_filter: if self.node_labels.is_empty() {
                None
            } else {
                Some(rg::ResourceGroupNodeFilter {
                    node_labels: self
                        .node_labels
                        .into_iter()
                        .map(|(key, value)| common::KeyValuePair {
                            key,
                            value,
                            ..Default::default()
                        })
                        .collect(),
                })
            },
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ResourceGroupNode
///////////////////////////////////////////////////////////////////////////////
/// A query node assigned to a resource group.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResourceGroupNode {
    pub(crate) id: i64,
    pub(crate) address: String,
    pub(crate) hostname: String,
}

impl ResourceGroupNode {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            id: 0,
            address: String::new(),
            hostname: String::new(),
        }
    }

    /// Sets the id and returns the updated value.
    pub fn id(mut self, value: i64) -> Self {
        self.id = value;
        self
    }

    /// Sets the id and returns this value for further mutation.
    pub fn set_id(&mut self, value: i64) -> &mut Self {
        self.id = value;
        self
    }

    /// Returns the configured id.
    pub fn get_id(&self) -> i64 {
        self.id
    }

    /// Sets the address and returns the updated value.
    pub fn address(mut self, value: impl Into<String>) -> Self {
        self.address = value.into();
        self
    }

    /// Sets the address and returns this value for further mutation.
    pub fn set_address(&mut self, value: impl Into<String>) -> &mut Self {
        self.address = value.into();
        self
    }

    /// Returns the configured address.
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Sets the hostname and returns the updated value.
    pub fn hostname(mut self, value: impl Into<String>) -> Self {
        self.hostname = value.into();
        self
    }

    /// Sets the hostname and returns this value for further mutation.
    pub fn set_hostname(&mut self, value: impl Into<String>) -> &mut Self {
        self.hostname = value.into();
        self
    }

    /// Returns the configured hostname.
    pub fn get_hostname(&self) -> &str {
        &self.hostname
    }
}

///////////////////////////////////////////////////////////////////////////////
// ResourceGroupDescription
///////////////////////////////////////////////////////////////////////////////
/// Current configuration and capacity of a resource group.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResourceGroupDescription {
    pub(crate) name: String,
    pub(crate) capacity: i32,
    pub(crate) available_nodes: i32,
    pub(crate) loaded_replicas: HashMap<String, i32>,
    pub(crate) outgoing_nodes: HashMap<String, i32>,
    pub(crate) incoming_nodes: HashMap<String, i32>,
    pub(crate) config: ResourceGroupConfig,
    pub(crate) nodes: Vec<ResourceGroupNode>,
}

impl ResourceGroupDescription {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            capacity: 0,
            available_nodes: 0,
            loaded_replicas: HashMap::new(),
            outgoing_nodes: HashMap::new(),
            incoming_nodes: HashMap::new(),
            config: ResourceGroupConfig::new(),
            nodes: Vec::new(),
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Returns the configured name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the capacity and returns the updated value.
    pub fn capacity(mut self, value: i32) -> Self {
        self.capacity = value;
        self
    }

    /// Sets the capacity and returns this value for further mutation.
    pub fn set_capacity(&mut self, value: i32) -> &mut Self {
        self.capacity = value;
        self
    }

    /// Returns the configured capacity.
    pub fn get_capacity(&self) -> i32 {
        self.capacity
    }

    /// Sets the available nodes and returns the updated value.
    pub fn available_nodes(mut self, value: i32) -> Self {
        self.available_nodes = value;
        self
    }

    /// Sets the available nodes and returns this value for further mutation.
    pub fn set_available_nodes(&mut self, value: i32) -> &mut Self {
        self.available_nodes = value;
        self
    }

    /// Returns the configured available nodes.
    pub fn get_available_nodes(&self) -> i32 {
        self.available_nodes
    }

    /// Sets the loaded replicas and returns the updated value.
    pub fn loaded_replicas(mut self, value: HashMap<String, i32>) -> Self {
        self.loaded_replicas = value;
        self
    }

    /// Sets the loaded replicas and returns this value for further mutation.
    pub fn set_loaded_replicas(&mut self, value: HashMap<String, i32>) -> &mut Self {
        self.loaded_replicas = value;
        self
    }

    /// Returns the configured loaded replicas.
    pub fn get_loaded_replicas(&self) -> &HashMap<String, i32> {
        &self.loaded_replicas
    }

    /// Sets the outgoing nodes and returns the updated value.
    pub fn outgoing_nodes(mut self, value: HashMap<String, i32>) -> Self {
        self.outgoing_nodes = value;
        self
    }

    /// Sets the outgoing nodes and returns this value for further mutation.
    pub fn set_outgoing_nodes(&mut self, value: HashMap<String, i32>) -> &mut Self {
        self.outgoing_nodes = value;
        self
    }

    /// Returns the configured outgoing nodes.
    pub fn get_outgoing_nodes(&self) -> &HashMap<String, i32> {
        &self.outgoing_nodes
    }

    /// Sets the incoming nodes and returns the updated value.
    pub fn incoming_nodes(mut self, value: HashMap<String, i32>) -> Self {
        self.incoming_nodes = value;
        self
    }

    /// Sets the incoming nodes and returns this value for further mutation.
    pub fn set_incoming_nodes(&mut self, value: HashMap<String, i32>) -> &mut Self {
        self.incoming_nodes = value;
        self
    }

    /// Returns the configured incoming nodes.
    pub fn get_incoming_nodes(&self) -> &HashMap<String, i32> {
        &self.incoming_nodes
    }

    /// Sets the config and returns the updated value.
    pub fn config(mut self, value: ResourceGroupConfig) -> Self {
        self.config = value;
        self
    }

    /// Sets the config and returns this value for further mutation.
    pub fn set_config(&mut self, value: ResourceGroupConfig) -> &mut Self {
        self.config = value;
        self
    }

    /// Returns the configured config.
    pub fn get_config(&self) -> &ResourceGroupConfig {
        &self.config
    }

    /// Sets the nodes and returns the updated value.
    pub fn nodes(mut self, value: Vec<ResourceGroupNode>) -> Self {
        self.nodes = value;
        self
    }

    /// Sets the nodes and returns this value for further mutation.
    pub fn set_nodes(&mut self, value: Vec<ResourceGroupNode>) -> &mut Self {
        self.nodes = value;
        self
    }

    /// Returns the configured nodes.
    pub fn get_nodes(&self) -> &[ResourceGroupNode] {
        &self.nodes
    }

    /// Adds one add node to the existing values.
    pub fn add_node(mut self, value: ResourceGroupNode) -> Self {
        self.nodes.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn resource_group_config_default_values() {
        let value = ResourceGroupConfig::new();
        let expected_requested_nodes: i32 = 0;
        let expected_node_limit: i32 = 0;
        let expected_transfer_from: Vec<String> = Default::default();
        let expected_transfer_to: Vec<String> = Default::default();
        let expected_node_labels: HashMap<String, String> = Default::default();

        assert_eq!(
            value.get_requested_nodes().to_owned(),
            expected_requested_nodes
        );
        assert_eq!(value.get_node_limit().to_owned(), expected_node_limit);
        assert_eq!(value.get_transfer_from().to_owned(), expected_transfer_from);
        assert_eq!(value.get_transfer_to().to_owned(), expected_transfer_to);
        assert_eq!(value.get_node_labels().to_owned(), expected_node_labels);
        assert_eq!(ResourceGroupConfig::new(), value);
    }

    #[test]
    fn resource_group_config_populated_values() {
        let requested_nodes = 7;
        let node_limit = 7;
        let transfer_from = vec!["transfer_from-value".to_owned()];
        let transfer_to = vec!["transfer_to-value".to_owned()];
        let node_labels = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = ResourceGroupConfig::new()
            .requested_nodes(requested_nodes.clone())
            .node_limit(node_limit.clone())
            .transfer_from(transfer_from.clone())
            .transfer_to(transfer_to.clone())
            .node_labels(node_labels.clone());

        assert_eq!(value.get_requested_nodes().to_owned(), requested_nodes);
        assert_eq!(value.get_node_limit().to_owned(), node_limit);
        assert_eq!(value.get_transfer_from().to_owned(), transfer_from);
        assert_eq!(value.get_transfer_to().to_owned(), transfer_to);
        assert_eq!(value.get_node_labels().to_owned(), node_labels);
    }

    #[test]
    fn resource_group_node_default_values() {
        let value = ResourceGroupNode::new();
        let expected_id: i64 = 0;
        let expected_address: String = String::new();
        let expected_hostname: String = String::new();

        assert_eq!(value.get_id().to_owned(), expected_id);
        assert_eq!(value.get_address().to_owned(), expected_address);
        assert_eq!(value.get_hostname().to_owned(), expected_hostname);
    }

    #[test]
    fn resource_group_node_populated_values() {
        let id = 7;
        let address = "address-value".to_owned();
        let hostname = "hostname-value".to_owned();
        let value = ResourceGroupNode::new()
            .id(id.clone())
            .address(address.clone())
            .hostname(hostname.clone());

        assert_eq!(value.get_id().to_owned(), id);
        assert_eq!(value.get_address().to_owned(), address);
        assert_eq!(value.get_hostname().to_owned(), hostname);
    }

    #[test]
    fn resource_group_description_default_values() {
        let value = ResourceGroupDescription::new();
        let expected_name: String = String::new();
        let expected_capacity: i32 = 0;
        let expected_available_nodes: i32 = 0;
        let expected_loaded_replicas: HashMap<String, i32> = Default::default();
        let expected_outgoing_nodes: HashMap<String, i32> = Default::default();
        let expected_incoming_nodes: HashMap<String, i32> = Default::default();
        let expected_config = ResourceGroupConfig::new();
        let expected_nodes: Vec<ResourceGroupNode> = Default::default();

        assert_eq!(value.get_name().to_owned(), expected_name);
        assert_eq!(value.get_capacity().to_owned(), expected_capacity);
        assert_eq!(
            value.get_available_nodes().to_owned(),
            expected_available_nodes
        );
        assert_eq!(
            value.get_loaded_replicas().to_owned(),
            expected_loaded_replicas
        );
        assert_eq!(
            value.get_outgoing_nodes().to_owned(),
            expected_outgoing_nodes
        );
        assert_eq!(
            value.get_incoming_nodes().to_owned(),
            expected_incoming_nodes
        );
        assert_eq!(value.get_config().to_owned(), expected_config);
        assert_eq!(value.get_nodes().to_owned(), expected_nodes);
    }

    #[test]
    fn resource_group_description_populated_values() {
        let name = "name-value".to_owned();
        let capacity = 7;
        let available_nodes = 7;
        let loaded_replicas = HashMap::from([("key-value".to_owned(), 7)]);
        let outgoing_nodes = HashMap::from([("key-value".to_owned(), 7)]);
        let incoming_nodes = HashMap::from([("key-value".to_owned(), 7)]);
        let config = ResourceGroupConfig::new();
        let nodes = vec![ResourceGroupNode::new()];
        let value = ResourceGroupDescription::new()
            .name(name.clone())
            .capacity(capacity.clone())
            .available_nodes(available_nodes.clone())
            .loaded_replicas(loaded_replicas.clone())
            .outgoing_nodes(outgoing_nodes.clone())
            .incoming_nodes(incoming_nodes.clone())
            .config(config.clone())
            .nodes(nodes.clone());

        assert_eq!(value.get_name().to_owned(), name);
        assert_eq!(value.get_capacity().to_owned(), capacity);
        assert_eq!(value.get_available_nodes().to_owned(), available_nodes);
        assert_eq!(value.get_loaded_replicas().to_owned(), loaded_replicas);
        assert_eq!(value.get_outgoing_nodes().to_owned(), outgoing_nodes);
        assert_eq!(value.get_incoming_nodes().to_owned(), incoming_nodes);
        assert_eq!(value.get_config().to_owned(), config);
        assert_eq!(value.get_nodes().to_owned(), nodes);
    }
}
