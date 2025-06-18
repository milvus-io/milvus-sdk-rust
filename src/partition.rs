use std::collections::HashMap;

use crate::error::*;
use crate::{
    client::Client,
    proto::{
        self,
        common::{MsgBase, MsgType},
    },
    utils::status_to_result,
};

/// load_partitions' waitting time
const WAIT_LOAD_DURATION_MS: u64 = 100;

#[derive(Debug, Clone)]
pub struct LoadPartitionsOption {
    resource_groups: Vec<String>,
    refresh: bool,
    load_fields: Vec<String>,
    skip_load_dynamic_field: bool,
    load_params: HashMap<String, String>,
}

impl Default for LoadPartitionsOption {
    fn default() -> Self {
        LoadPartitionsOption {
            resource_groups: Vec::new(),
            refresh: false,
            load_fields: Vec::new(),
            skip_load_dynamic_field: false,
            load_params: HashMap::new(),
        }
    }
}

impl Client {
    /// Creates a new partition in the specified collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection where the partition will be created.
    /// * `partition_name` - The name of the partition to be created.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn create_partition(
        &self,
        collection_name: String,
        partition_name: String,
    ) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .create_partition(crate::proto::milvus::CreatePartitionRequest {
                    base: Some(MsgBase::new(MsgType::CreatePartition)),
                    db_name: "".to_string(), // reserved
                    collection_name,
                    partition_name,
                })
                .await?
                .into_inner(),
        ))
    }

    /// Drops a partition from the specified collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection containing the partition.
    /// * `partition_name` - The name of the partition to be dropped.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn drop_partition(
        &self,
        collection_name: String,
        partition_name: String,
    ) -> Result<()> {
        status_to_result(&Some(
            self.client
                .clone()
                .drop_partition(crate::proto::milvus::DropPartitionRequest {
                    base: Some(MsgBase::new(MsgType::DropPartition)),
                    db_name: "".to_string(), // reserved
                    collection_name,
                    partition_name,
                })
                .await?
                .into_inner(),
        ))
    }

    /// Retrieves a list of all partitions in the specified collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to list partitions for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of partition names if successful, or an error if the operation fails.
    pub async fn list_partitions(&self, collection_name: String) -> Result<Vec<String>> {
        let res = self
            .client
            .clone()
            .show_partitions(crate::proto::milvus::ShowPartitionsRequest {
                base: Some(MsgBase::new(MsgType::ShowPartitions)),
                db_name: "".to_string(), // reserved
                collection_name,
                collection_id: 0,        // reserved
                partition_names: vec![], // reserved
                r#type: 0,               // reserved
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res.partition_names)
    }

    /// Checks if a partition exists in the specified collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to check.
    /// * `partition_name` - The name of the partition to check for existence.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a boolean indicating whether the partition exists.
    pub async fn has_partition(
        &self,
        collection_name: String,
        partition_name: String,
    ) -> Result<bool> {
        let res = self
            .client
            .clone()
            .has_partition(crate::proto::milvus::HasPartitionRequest {
                base: Some(MsgBase::new(MsgType::HasPartition)),
                db_name: "".to_string(), // reserved
                collection_name,
                partition_name,
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res.value)
    }

    /// Retrieves statistics for a specific partition.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection containing the partition.
    /// * `partition_name` - The name of the partition to get statistics for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a HashMap of statistics key-value pairs.
    pub async fn get_partition_stats(
        &self,
        collection_name: String,
        partition_name: String,
    ) -> Result<HashMap<String, String>> {
        let res = self
            .client
            .clone()
            .get_partition_statistics(crate::proto::milvus::GetPartitionStatisticsRequest {
                base: Some(MsgBase::new(MsgType::GetPartitionStatistics)),
                db_name: "".to_string(), // reserved
                collection_name,
                partition_name,
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;

        Ok(res.stats.into_iter().map(|s| (s.key, s.value)).collect())
    }

    /// Gets the loading progress for specified partitions.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection.
    /// * `partition_names` - An iterator of partition names to check progress for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the loading progress percentage (0-100).
    async fn get_loading_progress<'a, S, I>(
        &self,
        collection_name: S,
        partition_names: I,
    ) -> Result<i64>
    where
        S: Into<String>,
        I: IntoIterator<Item = &'a String>,
    {
        let partition_names: Vec<String> = partition_names.into_iter().map(|x| x.into()).collect();
        let resp = self
            .client
            .clone()
            .get_loading_progress(crate::proto::milvus::GetLoadingProgressRequest {
                base: Some(MsgBase::new(MsgType::LoadPartitions)),
                db_name: "".to_string(),
                collection_name: collection_name.into(),
                partition_names: partition_names,
            })
            .await?
            .into_inner();

        status_to_result(&resp.status)?;
        Ok(resp.progress)
    }

    /// Loads partitions into memory with configurable options.
    ///
    /// This method loads the specified partitions into memory and waits for the loading
    /// process to complete. The method polls the loading progress until it reaches 100%.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection containing the partitions.
    /// * `partition_names` - An iterator of partition names to load.
    /// * `replica_number` - The number of replicas to load.
    /// * `options` - Optional configuration for the loading process.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure. The method will wait until
    /// all partitions are fully loaded before returning.
    pub async fn load_partitions<S: Into<String> + Copy, I: IntoIterator<Item = S>>(
        &self,
        collection_name: S,
        partition_names: I,
        replica_number: i32,
        options: Option<LoadPartitionsOption>,
    ) -> Result<()> {
        let names: Vec<String> = partition_names.into_iter().map(|x| x.into()).collect();
        let options = options.unwrap_or_default();

        status_to_result(&Some(
            self.client
                .clone()
                .load_partitions(proto::milvus::LoadPartitionsRequest {
                    base: Some(MsgBase::new(MsgType::LoadPartitions)),
                    db_name: "".to_string(),
                    collection_name: collection_name.into(),
                    replica_number,
                    partition_names: names.clone(),
                    resource_groups: options.resource_groups,
                    refresh: options.refresh,
                    load_fields: options.load_fields,
                    skip_load_dynamic_field: options.skip_load_dynamic_field,
                    load_params: options.load_params,
                })
                .await?
                .into_inner(),
        ))?;

        loop {
            if self.get_loading_progress(collection_name, &names).await? >= 100 {
                return Ok(());
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(WAIT_LOAD_DURATION_MS)).await;
        }
    }

    /// Releases partitions from memory.
    ///
    /// This method releases the specified partitions from memory, freeing up
    /// system resources. After releasing, the partitions will no longer be
    /// available for queries until they are loaded again.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection containing the partitions.
    /// * `partition_names` - An iterator of partition names to release.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    pub async fn release_partitions<S: Into<String>, I: IntoIterator<Item = S>>(
        &self,
        collection_name: S,
        partition_names: I,
    ) -> Result<()> {
        let names: Vec<String> = partition_names.into_iter().map(|x| x.into()).collect();
        status_to_result(&Some(
            self.client
                .clone()
                .release_partitions(crate::proto::milvus::ReleasePartitionsRequest {
                    base: Some(MsgBase::new(MsgType::ReleasePartitions)),
                    db_name: "".to_string(),
                    collection_name: collection_name.into(),
                    partition_names: names,
                })
                .await?
                .into_inner(),
        ))
    }
}
