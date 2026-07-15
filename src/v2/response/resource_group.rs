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

//! Response types returned by resource-group operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
pub use crate::v2::types::{ResourceGroupConfig, ResourceGroupDescription, ResourceGroupNode};

///////////////////////////////////////////////////////////////////////////////
// ListResourceGroupsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_resource_groups operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListResourceGroupsResponse {
    pub(crate) group_names: Vec<String>,
}

impl ListResourceGroupsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            group_names: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListResourceGroupsResponseBuilder {
        ListResourceGroupsResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn group_names(&self) -> &[String] {
        &self.group_names
    }

    pub(crate) fn from_proto(v: milvus::ListResourceGroupsResponse) -> Self {
        Self {
            group_names: v.resource_groups,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListResourceGroupsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListResourceGroupsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListResourceGroupsResponseBuilder {
    value: ListResourceGroupsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListResourceGroupsResponseBuilder {
    pub fn group_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.group_names = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> ListResourceGroupsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeResourceGroupResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_resource_group operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeResourceGroupResponse {
    pub(crate) description: ResourceGroupDescription,
}

impl DescribeResourceGroupResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            description: ResourceGroupDescription::new(),
        }
    }
}

impl DescribeResourceGroupResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeResourceGroupResponseBuilder {
        DescribeResourceGroupResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn description(&self) -> &ResourceGroupDescription {
        &self.description
    }

    pub(crate) fn from_proto(v: milvus::DescribeResourceGroupResponse) -> Result<Self> {
        let v = v.resource_group.ok_or_else(|| {
            Error::MalformedResponse("describe resource group returned no resource group".into())
        })?;
        Ok(Self {
            description: ResourceGroupDescription {
                name: v.name,
                capacity: v.capacity,
                available_nodes: v.num_available_node,
                loaded_replicas: v.num_loaded_replica,
                outgoing_nodes: v.num_outgoing_node,
                incoming_nodes: v.num_incoming_node,
                config: v
                    .config
                    .map_or_else(ResourceGroupConfig::new, ResourceGroupConfig::from_proto),
                nodes: v
                    .nodes
                    .into_iter()
                    .map(|n| ResourceGroupNode {
                        id: n.node_id,
                        address: n.address,
                        hostname: n.hostname,
                    })
                    .collect(),
            },
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeResourceGroupResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeResourceGroupResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeResourceGroupResponseBuilder {
    value: DescribeResourceGroupResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeResourceGroupResponseBuilder {
    pub fn description(mut self, value: ResourceGroupDescription) -> Self {
        self.value.description = value;
        self
    }

    pub fn build(self) -> DescribeResourceGroupResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::common;

    #[test]
    fn resource_group_response_methods_and_conversions() {
        let list = ListResourceGroupsResponse::builder()
            .group_names(["default", "analytics"])
            .build();
        assert_eq!(list.group_names().to_owned(), ["default", "analytics"]);
        let list = ListResourceGroupsResponse::from_proto(milvus::ListResourceGroupsResponse {
            resource_groups: vec!["batch".into()],
            ..Default::default()
        });
        assert_eq!(list.group_names().to_owned(), ["batch"]);

        let description = ResourceGroupDescription::new().name("analytics");
        let response = DescribeResourceGroupResponse::builder()
            .description(description)
            .build();
        assert_eq!(response.description().get_name().to_owned(), "analytics");

        let response =
            DescribeResourceGroupResponse::from_proto(milvus::DescribeResourceGroupResponse {
                resource_group: Some(milvus::ResourceGroup {
                    name: "batch".into(),
                    capacity: 4,
                    num_available_node: 2,
                    num_loaded_replica: std::collections::HashMap::from([("books".into(), 3)]),
                    num_outgoing_node: std::collections::HashMap::from([("backup".into(), 1)]),
                    num_incoming_node: std::collections::HashMap::from([("default".into(), 5)]),
                    nodes: vec![common::NodeInfo {
                        node_id: 10,
                        address: "127.0.0.1".into(),
                        hostname: "node".into(),
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })
            .expect("valid describe resource group response");
        let description = response.description();
        assert_eq!(description.get_name().to_owned(), "batch");
        assert_eq!(description.get_nodes()[0].get_id().to_owned(), 10);
    }

    #[test]
    fn describe_resource_group_rejects_a_missing_resource_group() {
        let error = DescribeResourceGroupResponse::from_proto(
            milvus::DescribeResourceGroupResponse::default(),
        )
        .unwrap_err();

        assert!(matches!(error, Error::MalformedResponse(_)));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn list_resource_groups_response_default_values() {
        let value = ListResourceGroupsResponse::builder().build();
        let expected_group_names: Vec<String> = Default::default();

        assert_eq!(value.group_names().to_owned(), expected_group_names);
    }

    #[test]
    fn list_resource_groups_response_populated_values() {
        let group_names = vec!["group_names-value".to_owned()];
        let value = ListResourceGroupsResponse::builder()
            .group_names(group_names.clone())
            .build();

        assert_eq!(value.group_names().to_owned(), group_names);
    }

    #[test]
    fn describe_resource_group_response_default_values() {
        let value = DescribeResourceGroupResponse::builder().build();
        let expected_description = ResourceGroupDescription::new();

        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn describe_resource_group_response_populated_values() {
        let description = ResourceGroupDescription::new();
        let value = DescribeResourceGroupResponse::builder()
            .description(description.clone())
            .build();

        assert_eq!(value.description().to_owned(), description);
    }
}
