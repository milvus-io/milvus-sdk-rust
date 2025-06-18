//! Resource group management functionality for Milvus Rust SDK.
//!
//! This module provides comprehensive resource group operations including creation, deletion,
//! modification, and querying of resource groups in Milvus. Resource groups allow you to
//! manage and isolate computing resources for different workloads.
//!
//! # Examples
//!
//! ```rust
//! use milvus::client::Client;
//! use milvus::resource_group::CreateRgOptions;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("localhost:19530").await?;
//!     
//!     // Create a resource group with custom limits
//!     let options = CreateRgOptions::new()
//!         .limits(10)
//!         .requests(2);
//!     
//!     client.create_resource_group("my_rg", Some(options)).await?;
//!     
//!     // List all resource groups
//!     let groups = client.list_resource_groups().await?;
//!     println!("Available resource groups: {:?}", groups);
//!     
//!     // Transfer replicas between resource groups
//!     client.transfer_replica("source_rg", "target_rg", "my_collection", 1).await?;
//!     
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use crate::client::Client;
use crate::proto::common::{KeyValuePair, MsgBase, MsgType};
use crate::proto::rg::*;
use crate::utils::status_to_result;
use crate::{error::*, proto};

/// Configuration options for creating a new resource group.
///
/// This struct provides a builder pattern for configuring resource group properties
/// such as resource limits, requests, and transfer configurations.
///
/// # Examples
///
/// ```rust
/// use milvus::resource_group::CreateRgOptions;
///
/// let options = CreateRgOptions::new()
///     .limits(10)
///     .requests(2)
///     .transfer_from(vec!["other_rg"])
///     .transfer_to(vec!["target_rg"]);
/// ```
#[derive(Debug, Clone)]
pub struct CreateRgOptions {
    /// The resource group configuration
    config: ResourceGroupConfig,
}

/// Type alias for resource group update options, currently equivalent to CreateRgOptions
pub type UpdateRgOptions = CreateRgOptions;

impl Default for CreateRgOptions {
    fn default() -> Self {
        CreateRgOptions {
            config: ResourceGroupConfig {
                requests: None,
                limits: None,
                transfer_from: Vec::new(),
                transfer_to: Vec::new(),
                node_filter: None,
            },
        }
    }
}

impl CreateRgOptions {
    /// Creates a new `CreateRgOptions` instance with default values.
    ///
    /// All properties are set to default values, meaning they will use
    /// the server's default settings.
    ///
    /// # Returns
    ///
    /// A new `CreateRgOptions` instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the resource requests for the resource group.
    ///
    /// This specifies the minimum number of nodes that should be allocated
    /// to this resource group.
    ///
    /// # Arguments
    ///
    /// * `requests` - The number of nodes requested
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn requests(mut self, requests: i32) -> Self {
        self.config.requests = Some(ResourceGroupLimit { node_num: requests });
        self
    }

    /// Sets the resource limits for the resource group.
    ///
    /// This specifies the maximum number of nodes that can be allocated
    /// to this resource group.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum number of nodes allowed
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn limits(mut self, limit: i32) -> Self {
        self.config.limits = Some(ResourceGroupLimit { node_num: limit });
        self
    }

    /// Sets the source resource groups for transfers.
    ///
    /// This specifies which resource groups can transfer resources to this group.
    ///
    /// # Arguments
    ///
    /// * `transfer_from` - Vector of resource group names that can transfer to this group
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn transfer_from<S: Into<String>>(mut self, transfer_from: Vec<S>) -> Self {
        let transfer_from: Vec<ResourceGroupTransfer> = transfer_from
            .into_iter()
            .map(move |x| ResourceGroupTransfer {
                resource_group: x.into(),
            })
            .collect();
        self.config.transfer_from = transfer_from;
        self
    }

    /// Sets the target resource groups for transfers.
    ///
    /// This specifies which resource groups this group can transfer resources to.
    ///
    /// # Arguments
    ///
    /// * `transfer_to` - Vector of resource group names this group can transfer to
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn transfer_to<S: Into<String>>(mut self, transfer_to: Vec<S>) -> Self {
        let transfer_to: Vec<ResourceGroupTransfer> = transfer_to
            .into_iter()
            .map(|x| ResourceGroupTransfer {
                resource_group: x.into(),
            })
            .collect();
        self.config.transfer_to = transfer_to;
        self
    }

    /// Sets the node filter for the resource group.
    ///
    /// This specifies node labels that determine which nodes can be used
    /// by this resource group.
    ///
    /// # Arguments
    ///
    /// * `node_filter` - Vector of key-value pairs representing node labels
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn node_filter(mut self, node_filter: Vec<KeyValuePair>) -> Self {
        self.config.node_filter = Some(ResourceGroupNodeFilter {
            node_labels: node_filter,
        });
        self
    }
}

impl Client {
    /// Creates a new resource group with specified configuration.
    ///
    /// This method creates a new resource group in Milvus with the given name and
    /// optional configuration properties. If no options are provided, the
    /// resource group will be created with default settings.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource group to create
    /// * `options` - Optional configuration properties for the resource group
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use milvus::resource_group::CreateRgOptions;
    ///
    /// // Create resource group with default settings
    /// client.create_resource_group("my_rg", None).await?;
    ///
    /// // Create resource group with custom properties
    /// let options = CreateRgOptions::new()
    ///     .limits(10)
    ///     .requests(2);
    ///     
    /// client.create_resource_group("my_rg", Some(options)).await?;
    /// ```
    pub async fn create_resource_group<S: Into<String>>(
        &self,
        name: S,
        options: Option<CreateRgOptions>,
    ) -> Result<()> {
        let resource_group: String = name.into();
        let config = options.unwrap_or_default().config;
        let res = self
            .client
            .clone()
            .create_resource_group(proto::milvus::CreateResourceGroupRequest {
                base: Some(MsgBase::new(MsgType::CreateResourceGroup)),
                resource_group,
                config: Some(config),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Describes a resource group and returns its detailed information.
    ///
    /// This method retrieves comprehensive information about a resource group including
    /// its configuration, limits, requests, and transfer settings.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource group to describe
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `Option<ResourceGroup>` with resource group details.
    /// Returns `None` if the resource group doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// if let Some(rg_info) = client.describe_resource_group("my_rg").await? {
    ///     println!("Resource group: {:?}", rg_info);
    /// } else {
    ///     println!("Resource group doesn't exist!");
    /// }
    /// ```
    pub async fn describe_resource_group<S: Into<String>>(
        &self,
        name: S,
    ) -> Result<Option<proto::milvus::ResourceGroup>> {
        let res = self
            .client
            .clone()
            .describe_resource_group(proto::milvus::DescribeResourceGroupRequest {
                base: Some(MsgBase::new(MsgType::DescribeResourceGroup)),
                resource_group: name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res.resource_group)
    }

    /// Drops (deletes) a resource group.
    ///
    /// This method permanently removes a resource group. All resources allocated
    /// to this group will be released.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource group to drop
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// client.drop_resource_group("my_rg").await?;
    /// ```
    ///
    /// # Warning
    ///
    /// This operation will release all resources allocated to the resource group.
    pub async fn drop_resource_group<S: Into<String>>(&self, name: S) -> Result<()> {
        let res = self
            .client
            .clone()
            .drop_resource_group(proto::milvus::DropResourceGroupRequest {
                base: Some(MsgBase::new(MsgType::DropResourceGroup)),
                resource_group: name.into(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Lists all resource groups in the Milvus instance.
    ///
    /// This method retrieves a list of all resource group names that exist in
    /// the current Milvus instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of resource group names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let groups = client.list_resource_groups().await?;
    /// println!("Available resource groups: {:?}", groups);
    /// ```
    pub async fn list_resource_groups(&self) -> Result<Vec<String>> {
        let res = self
            .client
            .clone()
            .list_resource_groups(proto::milvus::ListResourceGroupsRequest {
                base: Some(MsgBase::new(MsgType::ListResourceGroups)),
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res.resource_groups)
    }

    /// Transfers replicas between resource groups.
    ///
    /// This method moves a specified number of replicas from one resource group
    /// to another. This is useful for load balancing and resource optimization.
    ///
    /// # Arguments
    ///
    /// * `source_group` - The source resource group name
    /// * `target_group` - The target resource group name
    /// * `collection_name` - The name of the collection whose replicas to transfer
    /// * `num_replicas` - The number of replicas to transfer
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Transfer 2 replicas from source_rg to target_rg for my_collection
    /// client.transfer_replica("source_rg", "target_rg", "my_collection", 2).await?;
    /// ```
    pub async fn transfer_replica<S: Into<String>>(
        &self,
        source_group: S,
        target_group: S,
        collection_name: S,
        num_replicas: i64,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .transfer_replica(proto::milvus::TransferReplicaRequest {
                base: Some(MsgBase::new(MsgType::TransferReplica)),
                source_resource_group: source_group.into(),
                target_resource_group: target_group.into(),
                collection_name: collection_name.into(),
                num_replica: num_replicas,
                db_name: "".to_string(),
            })
            .await?
            .into_inner();
        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Transfers nodes between resource groups.
    ///
    /// This method moves a specified number of nodes from one resource group
    /// to another. This affects the computing resources available to each group.
    ///
    /// # Arguments
    ///
    /// * `source` - The source resource group name
    /// * `target` - The target resource group name
    /// * `num_node` - The number of nodes to transfer
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Transfer 3 nodes from source_rg to target_rg
    /// client.transfer_node("source_rg", "target_rg", 3).await?;
    /// ```
    pub async fn transfer_node<S: Into<String>>(
        &self,
        source: S,
        target: S,
        num_node: i32,
    ) -> Result<()> {
        let res = self
            .client
            .clone()
            .transfer_node(proto::milvus::TransferNodeRequest {
                base: Some(MsgBase::new(MsgType::TransferNode)),
                source_resource_group: source.into(),
                target_resource_group: target.into(),
                num_node,
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;
        Ok(())
    }

    /// Updates multiple resource groups with new configurations.
    ///
    /// This method allows you to update the configuration of multiple resource
    /// groups in a single operation. This is more efficient than updating
    /// each group individually.
    ///
    /// # Arguments
    ///
    /// * `configs` - A HashMap mapping resource group names to their new configurations
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use milvus::resource_group::UpdateRgOptions;
    ///
    /// let mut configs = HashMap::new();
    /// configs.insert("rg1".to_string(), UpdateRgOptions::new().limits(5).requests(2));
    /// configs.insert("rg2".to_string(), UpdateRgOptions::new().limits(10).requests(3));
    ///
    /// client.update_resource_groups(configs).await?;
    /// ```
    pub async fn update_resource_groups<S: Into<String>>(
        &self,
        configs: HashMap<S, UpdateRgOptions>,
    ) -> Result<()> {
        let config: HashMap<String, ResourceGroupConfig> = configs
            .into_iter()
            .map(|(name, options)| {
                let name: String = name.into();
                let config = options.config;
                (name, config)
            })
            .collect();

        let res = self
            .client
            .clone()
            .update_resource_groups(proto::milvus::UpdateResourceGroupsRequest {
                base: Some(MsgBase::new(MsgType::UpdateResourceGroups)),
                resource_groups: config,
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;

        Ok(())
    }
}
