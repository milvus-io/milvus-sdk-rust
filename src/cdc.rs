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

//! Change Data Capture (CDC) APIs for Milvus Rust SDK.

use crate::client::Client;
use crate::error::Result;
use crate::proto::common::{
    MessageId as ProtoMessageId, ReplicateCheckpoint as ProtoReplicateCheckpoint,
    ReplicateConfiguration as ProtoReplicateConfiguration, WalName,
};
use crate::proto::milvus::{
    GetReplicateConfigurationRequest, GetReplicateInfoRequest,
    UpdateReplicateConfigurationRequest,
};
use crate::utils::status_to_result;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplicateCheckpoint {
    pub cluster_id: String,
    pub pchannel: String,
    pub message_id: Option<MessageId>,
    pub time_tick: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageId {
    pub id: String,
    pub wal_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplicateConfiguration {
    pub clusters: Vec<MilvusCluster>,
    pub cross_cluster_topologies: Vec<CrossClusterTopology>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MilvusCluster {
    pub cluster_id: String,
    pub uri: String,
    pub token: String,
    pub pchannels: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrossClusterTopology {
    pub source_cluster_id: String,
    pub target_cluster_id: String,
}

impl From<ProtoMessageId> for MessageId {
    fn from(value: ProtoMessageId) -> Self {
        Self {
            id: value.id,
            wal_name: WalName::try_from(value.wal_name)
                .map(|name| name.as_str_name().to_string())
                .unwrap_or_else(|_| "Unknown".to_string()),
        }
    }
}

impl From<MessageId> for ProtoMessageId {
    fn from(value: MessageId) -> Self {
        let wal_name = WalName::from_str_name(&value.wal_name).unwrap_or(WalName::Unknown) as i32;
        Self {
            id: value.id,
            wal_name,
        }
    }
}

impl From<ProtoReplicateCheckpoint> for ReplicateCheckpoint {
    fn from(value: ProtoReplicateCheckpoint) -> Self {
        Self {
            cluster_id: value.cluster_id,
            pchannel: value.pchannel,
            message_id: value.message_id.map(Into::into),
            time_tick: value.time_tick,
        }
    }
}

impl From<ReplicateCheckpoint> for ProtoReplicateCheckpoint {
    fn from(value: ReplicateCheckpoint) -> Self {
        Self {
            cluster_id: value.cluster_id,
            pchannel: value.pchannel,
            message_id: value.message_id.map(Into::into),
            time_tick: value.time_tick,
        }
    }
}

impl From<crate::proto::common::MilvusCluster> for MilvusCluster {
    fn from(value: crate::proto::common::MilvusCluster) -> Self {
        let (uri, token) = match value.connection_param {
            Some(connection_param) => (connection_param.uri, connection_param.token),
            None => (String::new(), String::new()),
        };

        Self {
            cluster_id: value.cluster_id,
            uri,
            token,
            pchannels: value.pchannels,
        }
    }
}

impl From<MilvusCluster> for crate::proto::common::MilvusCluster {
    fn from(value: MilvusCluster) -> Self {
        let connection_param = if value.uri.is_empty() && value.token.is_empty() {
            None
        } else {
            Some(crate::proto::common::ConnectionParam {
                uri: value.uri,
                token: value.token,
            })
        };

        Self {
            cluster_id: value.cluster_id,
            connection_param,
            pchannels: value.pchannels,
        }
    }
}

impl From<crate::proto::common::CrossClusterTopology> for CrossClusterTopology {
    fn from(value: crate::proto::common::CrossClusterTopology) -> Self {
        Self {
            source_cluster_id: value.source_cluster_id,
            target_cluster_id: value.target_cluster_id,
        }
    }
}

impl From<CrossClusterTopology> for crate::proto::common::CrossClusterTopology {
    fn from(value: CrossClusterTopology) -> Self {
        Self {
            source_cluster_id: value.source_cluster_id,
            target_cluster_id: value.target_cluster_id,
        }
    }
}

impl From<ProtoReplicateConfiguration> for ReplicateConfiguration {
    fn from(value: ProtoReplicateConfiguration) -> Self {
        Self {
            clusters: value.clusters.into_iter().map(Into::into).collect(),
            cross_cluster_topologies: value
                .cross_cluster_topology
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<ReplicateConfiguration> for ProtoReplicateConfiguration {
    fn from(value: ReplicateConfiguration) -> Self {
        Self {
            clusters: value.clusters.into_iter().map(Into::into).collect(),
            cross_cluster_topology: value
                .cross_cluster_topologies
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl Client {
    /// Retrieves replication checkpoint metadata for a target pchannel.
    pub async fn get_replicate_info<S1: Into<String>, S2: Into<String>>(
        &self,
        source_cluster_id: S1,
        target_pchannel: S2,
    ) -> Result<(Option<ReplicateCheckpoint>, Option<ReplicateCheckpoint>)> {
        let resp = self
            .client
            .clone()
            .get_replicate_info(GetReplicateInfoRequest {
                source_cluster_id: source_cluster_id.into(),
                target_pchannel: target_pchannel.into(),
            })
            .await?
            .into_inner();

        Ok((resp.checkpoint.map(Into::into), None))
    }

    /// Retrieves the current replication topology configuration.
    pub async fn get_replicate_configuration(&self) -> Result<ReplicateConfiguration> {
        let resp = self
            .client
            .clone()
            .get_replicate_configuration(GetReplicateConfigurationRequest {})
            .await?
            .into_inner();

        status_to_result(&resp.status)?;
        Ok(resp.configuration.unwrap_or_default().into())
    }

    /// Replaces the current replication topology configuration.
    pub async fn update_replicate_configuration(
        &self,
        replicate_configuration: ReplicateConfiguration,
        force_promote: bool,
    ) -> Result<()> {
        let resp = self
            .client
            .clone()
            .update_replicate_configuration(UpdateReplicateConfigurationRequest {
                replicate_configuration: Some(replicate_configuration.into()),
                force_promote,
            })
            .await?
            .into_inner();

        status_to_result(&Some(resp))?;
        Ok(())
    }
}
