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

impl Client {
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

    // pub async fn load_partitions<S: Into<String>, I: IntoIterator<Item = S>>(
    //     &self,
    //     collection_name: S,
    //     partition_names: I,
    //     replica_number: i32,
    // ) -> Result<()> {
    //     let names: Vec<String> = partition_names.into_iter().map(|x| x.to_string()).collect();
    //     status_to_result(&Some(
    //         self.client
    //             .clone()
    //             .load_partitions(proto::milvus::LoadPartitionsRequest {
    //                 base: Some(MsgBase::new(MsgType::LoadPartitions)),
    //                 db_name: "".to_string(),
    //                 collection_name: collection_name.into(),
    //                 replica_number,
    //                 partition_names: names.clone(),
    //             })
    //             .await?
    //             .into_inner(),
    //     ))?;

    //     loop {
    //         if self.get_loading_progress(&names).await? >= 100 {
    //             return Ok(());
    //         }

    //         tokio::time::sleep(Duration::from_millis(config::WAIT_LOAD_DURATION_MS)).await;
    //     }
    // }

    // pub async fn release_partitions<S: ToString, I: IntoIterator<Item = S>>(
    //     &self,
    //     partition_names: I,
    // ) -> Result<()> {
    //     status_to_result(&Some(
    //         self.client
    //             .clone()
    //             .release_partitions(ReleasePartitionsRequest {
    //                 base: Some(MsgBase::new(MsgType::ReleasePartitions)),
    //                 db_name: "".to_string(),
    //                 collection_name: self.schema().name.to_string(),
    //                 partition_names: partition_names.into_iter().map(|x| x.to_string()).collect(),
    //             })
    //             .await?
    //             .into_inner(),
    //     ))
    // }
}
