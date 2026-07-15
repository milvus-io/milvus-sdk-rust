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

//! Domain types for change-data-capture and replication APIs.

use crate::proto::common;
use crate::v2::error::{Error, Result};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// ReplicateCluster
///////////////////////////////////////////////////////////////////////////////
/// Connection and identity information for a replication cluster.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReplicateCluster {
    pub(crate) cluster_id: String,
    pub(crate) uri: String,
    pub(crate) token: String,
    pub(crate) physical_channels: Vec<String>,
}

impl std::fmt::Debug for ReplicateCluster {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReplicateCluster")
            .field("cluster_id", &self.cluster_id)
            .field("uri", &self.uri)
            .field("token", &"[REDACTED]")
            .field("physical_channels", &self.physical_channels)
            .finish()
    }
}

impl ReplicateCluster {
    pub fn new() -> Self {
        Self {
            cluster_id: String::new(),
            uri: String::new(),
            token: String::new(),
            physical_channels: Vec::new(),
        }
    }

    pub fn cluster_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_id = value.into();
        self
    }

    pub fn set_cluster_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.cluster_id = value.into();
        self
    }

    pub fn get_cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn uri(mut self, value: impl Into<String>) -> Self {
        self.uri = value.into();
        self
    }

    pub fn set_uri(&mut self, value: impl Into<String>) -> &mut Self {
        self.uri = value.into();
        self
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn token(mut self, value: impl Into<String>) -> Self {
        self.token = value.into();
        self
    }

    pub fn set_token(&mut self, value: impl Into<String>) -> &mut Self {
        self.token = value.into();
        self
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn physical_channels(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.physical_channels = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn set_physical_channels(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.physical_channels = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn get_physical_channels(&self) -> &[String] {
        &self.physical_channels
    }

    pub fn add_physical_channel(mut self, value: impl Into<String>) -> Self {
        self.physical_channels.push(value.into());
        self
    }

    pub(crate) fn into_proto(self) -> common::MilvusCluster {
        common::MilvusCluster {
            cluster_id: self.cluster_id,
            connection_param: Some(common::ConnectionParam {
                uri: self.uri,
                token: self.token,
            }),
            pchannels: self.physical_channels,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CrossClusterTopology
///////////////////////////////////////////////////////////////////////////////
/// Source and target topology for cross-cluster replication.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrossClusterTopology {
    pub(crate) source_cluster_id: String,
    pub(crate) target_cluster_id: String,
}

impl CrossClusterTopology {
    pub fn new() -> Self {
        Self {
            source_cluster_id: String::new(),
            target_cluster_id: String::new(),
        }
    }

    pub fn source_cluster_id(mut self, value: impl Into<String>) -> Self {
        self.source_cluster_id = value.into();
        self
    }

    pub fn set_source_cluster_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.source_cluster_id = value.into();
        self
    }

    pub fn get_source_cluster_id(&self) -> &str {
        &self.source_cluster_id
    }

    pub fn target_cluster_id(mut self, value: impl Into<String>) -> Self {
        self.target_cluster_id = value.into();
        self
    }

    pub fn set_target_cluster_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.target_cluster_id = value.into();
        self
    }

    pub fn get_target_cluster_id(&self) -> &str {
        &self.target_cluster_id
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReplicateConfiguration
///////////////////////////////////////////////////////////////////////////////
/// Cross-cluster replication configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReplicateConfiguration {
    pub(crate) clusters: Vec<ReplicateCluster>,
    pub(crate) topology: Vec<CrossClusterTopology>,
}

impl ReplicateConfiguration {
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            topology: Vec::new(),
        }
    }

    pub(crate) fn from_proto(v: common::ReplicateConfiguration) -> Result<Self> {
        Ok(Self {
            clusters: v
                .clusters
                .into_iter()
                .map(|v| {
                    let c = v.connection_param.ok_or_else(|| {
                        Error::MalformedResponse(format!(
                            "replication cluster {:?} has no connection parameters",
                            v.cluster_id
                        ))
                    })?;
                    Ok(ReplicateCluster {
                        cluster_id: v.cluster_id,
                        uri: c.uri,
                        token: c.token,
                        physical_channels: v.pchannels,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            topology: v
                .cross_cluster_topology
                .into_iter()
                .map(|v| CrossClusterTopology {
                    source_cluster_id: v.source_cluster_id,
                    target_cluster_id: v.target_cluster_id,
                })
                .collect(),
        })
    }
}

impl ReplicateConfiguration {
    pub fn clusters(mut self, value: Vec<ReplicateCluster>) -> Self {
        self.clusters = value;
        self
    }

    pub fn set_clusters(&mut self, value: Vec<ReplicateCluster>) -> &mut Self {
        self.clusters = value;
        self
    }

    pub fn get_clusters(&self) -> &[ReplicateCluster] {
        &self.clusters
    }

    pub fn topology(mut self, value: Vec<CrossClusterTopology>) -> Self {
        self.topology = value;
        self
    }

    pub fn set_topology(&mut self, value: Vec<CrossClusterTopology>) -> &mut Self {
        self.topology = value;
        self
    }

    pub fn get_topology(&self) -> &[CrossClusterTopology] {
        &self.topology
    }

    pub fn add_cluster(mut self, value: ReplicateCluster) -> Self {
        self.clusters.push(value);
        self
    }

    pub fn add_topology(mut self, value: CrossClusterTopology) -> Self {
        self.topology.push(value);
        self
    }

    pub(crate) fn into_proto(self) -> common::ReplicateConfiguration {
        common::ReplicateConfiguration {
            clusters: self
                .clusters
                .into_iter()
                .map(ReplicateCluster::into_proto)
                .collect(),
            cross_cluster_topology: self
                .topology
                .into_iter()
                .map(|v| common::CrossClusterTopology {
                    source_cluster_id: v.source_cluster_id,
                    target_cluster_id: v.target_cluster_id,
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// WalName
///////////////////////////////////////////////////////////////////////////////
/// Write-ahead-log implementation used by a replication cluster.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum WalName {
    #[default]
    Unknown,
    RocksMq,
    Pulsar,
    Kafka,
    WoodPecker,
    Test,
}

impl WalName {
    fn from_proto(value: i32) -> Self {
        match common::WalName::try_from(value).ok() {
            Some(common::WalName::RocksMq) => Self::RocksMq,
            Some(common::WalName::Pulsar) => Self::Pulsar,
            Some(common::WalName::Kafka) => Self::Kafka,
            Some(common::WalName::WoodPecker) => Self::WoodPecker,
            Some(common::WalName::Test) => Self::Test,
            _ => Self::Unknown,
        }
    }
}

impl WalName {
    pub(crate) fn into_proto(self) -> common::WalName {
        match self {
            WalName::Unknown => common::WalName::Unknown,
            WalName::RocksMq => common::WalName::RocksMq,
            WalName::Pulsar => common::WalName::Pulsar,
            WalName::Kafka => common::WalName::Kafka,
            WalName::WoodPecker => common::WalName::WoodPecker,
            WalName::Test => common::WalName::Test,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReplicateMessageId
///////////////////////////////////////////////////////////////////////////////
/// Position of a message in a replication channel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReplicateMessageId {
    pub(crate) id: String,
    pub(crate) wal_name: WalName,
}

impl ReplicateMessageId {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            wal_name: WalName::Unknown,
        }
    }

    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = value.into();
        self
    }

    pub fn set_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.id = value.into();
        self
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn wal_name(mut self, value: WalName) -> Self {
        self.wal_name = value;
        self
    }

    pub fn set_wal_name(&mut self, value: WalName) -> &mut Self {
        self.wal_name = value;
        self
    }

    pub fn get_wal_name(&self) -> WalName {
        self.wal_name
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReplicateCheckpoint
///////////////////////////////////////////////////////////////////////////////
/// Replication checkpoint for a channel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReplicateCheckpoint {
    pub(crate) cluster_id: String,
    pub(crate) physical_channel: String,
    pub(crate) message_id: ReplicateMessageId,
    pub(crate) time_tick: u64,
}

impl ReplicateCheckpoint {
    pub fn new() -> Self {
        Self {
            cluster_id: String::new(),
            physical_channel: String::new(),
            message_id: ReplicateMessageId::new(),
            time_tick: 0,
        }
    }

    pub fn cluster_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_id = value.into();
        self
    }

    pub fn set_cluster_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.cluster_id = value.into();
        self
    }

    pub fn get_cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn physical_channel(mut self, value: impl Into<String>) -> Self {
        self.physical_channel = value.into();
        self
    }

    pub fn set_physical_channel(&mut self, value: impl Into<String>) -> &mut Self {
        self.physical_channel = value.into();
        self
    }

    pub fn get_physical_channel(&self) -> &str {
        &self.physical_channel
    }

    pub fn message_id(mut self, value: ReplicateMessageId) -> Self {
        self.message_id = value;
        self
    }

    pub fn set_message_id(&mut self, value: ReplicateMessageId) -> &mut Self {
        self.message_id = value;
        self
    }

    pub fn get_message_id(&self) -> &ReplicateMessageId {
        &self.message_id
    }

    pub fn time_tick(mut self, value: u64) -> Self {
        self.time_tick = value;
        self
    }

    pub fn set_time_tick(&mut self, value: u64) -> &mut Self {
        self.time_tick = value;
        self
    }

    pub fn get_time_tick(&self) -> u64 {
        self.time_tick
    }

    pub(crate) fn from_proto(v: common::ReplicateCheckpoint) -> Result<Self> {
        let message_id = v.message_id.ok_or_else(|| {
            Error::MalformedResponse("replication checkpoint has no message ID".into())
        })?;
        Ok(Self {
            cluster_id: v.cluster_id,
            physical_channel: v.pchannel,
            message_id: ReplicateMessageId {
                id: message_id.id,
                wal_name: WalName::from_proto(message_id.wal_name),
            },
            time_tick: v.time_tick,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DumpedMessage
///////////////////////////////////////////////////////////////////////////////
/// One WAL message returned by a CDC dump stream.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DumpedMessage {
    pub(crate) message_id: ReplicateMessageId,
    pub(crate) payload: Vec<u8>,
    pub(crate) properties: HashMap<String, String>,
}

impl DumpedMessage {
    pub fn new() -> Self {
        Self {
            message_id: ReplicateMessageId::new(),
            payload: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub fn message_id(mut self, value: ReplicateMessageId) -> Self {
        self.message_id = value;
        self
    }

    pub fn set_message_id(&mut self, value: ReplicateMessageId) -> &mut Self {
        self.message_id = value;
        self
    }

    pub fn get_message_id(&self) -> &ReplicateMessageId {
        &self.message_id
    }

    pub fn payload(mut self, value: Vec<u8>) -> Self {
        self.payload = value;
        self
    }

    pub fn set_payload(&mut self, value: Vec<u8>) -> &mut Self {
        self.payload = value;
        self
    }

    pub fn get_payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.properties = value;
        self
    }

    pub fn set_properties(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.properties = value;
        self
    }

    pub fn get_properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub fn add_payload(mut self, value: u8) -> Self {
        self.payload.push(value);
        self
    }

    pub(crate) fn from_proto(value: common::ImmutableMessage) -> Result<Self> {
        let id = value.id.ok_or_else(|| {
            Error::MalformedResponse("dumped WAL message is missing its message ID".into())
        })?;
        Ok(Self {
            message_id: ReplicateMessageId {
                id: id.id,
                wal_name: WalName::from_proto(id.wal_name),
            },
            payload: value.payload,
            properties: value.properties,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replicate_cluster_constructor_preserves_defaults() {
        let value = ReplicateCluster::new().cluster_id("cluster").uri("uri");

        assert_eq!(value.get_cluster_id(), "cluster");
        assert_eq!(value.get_uri(), "uri");
        assert!(value.get_token().is_empty());
        assert!(value.get_physical_channels().is_empty());

        let proto = value.into_proto();
        assert_eq!(proto.cluster_id, "cluster");
        assert!(proto.pchannels.is_empty());
        assert_eq!(proto.connection_param.unwrap().uri, "uri");
    }

    #[test]
    fn replicate_cluster_direct_setters_round_trip_values() {
        let value = ReplicateCluster::new()
            .cluster_id("source-cluster")
            .uri("http://localhost:19530")
            .token("token")
            .physical_channels(["channel-1", "channel-2"]);

        assert_eq!(value.get_cluster_id().to_owned(), "source-cluster");
        assert_eq!(value.get_uri().to_owned(), "http://localhost:19530");
        assert_eq!(value.get_token().to_owned(), "token");
        assert_eq!(
            value.get_physical_channels().to_owned(),
            ["channel-1", "channel-2"]
        );

        let constructed = ReplicateCluster::new()
            .cluster_id("source-cluster")
            .uri("http://localhost:19530");
        assert_eq!(constructed.get_cluster_id().to_owned(), "source-cluster");
        assert_eq!(constructed.get_uri().to_owned(), "http://localhost:19530");

        let proto = value.into_proto();
        assert_eq!(proto.cluster_id, "source-cluster");
        assert_eq!(proto.pchannels, ["channel-1", "channel-2"]);
        let connection = proto.connection_param.expect("connection parameters");
        assert_eq!(connection.uri, "http://localhost:19530");
        assert_eq!(connection.token, "token");
    }

    #[test]
    fn replication_debug_output_redacts_cluster_tokens() {
        let cluster = ReplicateCluster::new()
            .cluster_id("source-cluster")
            .uri("http://localhost:19530")
            .token("replication-token-secret");
        let cluster_debug = format!("{cluster:?}");
        assert!(cluster_debug.contains("source-cluster"));
        assert!(cluster_debug.contains("[REDACTED]"));
        assert!(!cluster_debug.contains("replication-token-secret"));

        let configuration_debug =
            format!("{:?}", ReplicateConfiguration::new().add_cluster(cluster));
        assert!(configuration_debug.contains("[REDACTED]"));
        assert!(!configuration_debug.contains("replication-token-secret"));
    }

    #[test]
    fn cross_cluster_topology_constructor_preserves_values() {
        let value = CrossClusterTopology::new()
            .source_cluster_id("source")
            .target_cluster_id("target");

        assert_eq!(value.get_source_cluster_id(), "source");
        assert_eq!(value.get_target_cluster_id(), "target");
    }

    #[test]
    fn cross_cluster_topology_direct_setters_round_trip_values() {
        let value = CrossClusterTopology::new()
            .source_cluster_id("source-cluster")
            .target_cluster_id("target-cluster");

        assert_eq!(value.get_source_cluster_id().to_owned(), "source-cluster");
        assert_eq!(value.get_target_cluster_id().to_owned(), "target-cluster");
        assert_eq!(
            CrossClusterTopology::new()
                .source_cluster_id("source-cluster")
                .target_cluster_id("target-cluster"),
            value
        );
    }

    #[test]
    fn replicate_configuration_default_preserves_defaults() {
        let value = ReplicateConfiguration::new();

        assert!(value.get_clusters().is_empty());
        assert!(value.get_topology().is_empty());
        assert_eq!(ReplicateConfiguration::new(), value);

        let proto = value.clone().into_proto();
        assert!(proto.clusters.is_empty());
        assert!(proto.cross_cluster_topology.is_empty());
        assert_eq!(ReplicateConfiguration::from_proto(proto).unwrap(), value);
    }

    #[test]
    fn replicate_configuration_direct_setters_round_trip_values() {
        let cluster = ReplicateCluster::new()
            .cluster_id("source-cluster")
            .uri("http://localhost:19530")
            .token("token")
            .physical_channels(["channel-1"]);
        let topology = CrossClusterTopology::new()
            .source_cluster_id("source-cluster")
            .target_cluster_id("target-cluster");
        let value = ReplicateConfiguration::new()
            .clusters(vec![cluster.clone()])
            .topology(vec![topology.clone()]);

        assert_eq!(value.get_clusters().to_owned(), vec![cluster]);
        assert_eq!(value.get_topology().to_owned(), vec![topology]);

        let proto = value.clone().into_proto();
        assert_eq!(ReplicateConfiguration::from_proto(proto).unwrap(), value);
    }

    #[test]
    fn wal_name_converts_all_supported_values() {
        let values = [
            (WalName::Unknown, common::WalName::Unknown),
            (WalName::RocksMq, common::WalName::RocksMq),
            (WalName::Pulsar, common::WalName::Pulsar),
            (WalName::Kafka, common::WalName::Kafka),
            (WalName::WoodPecker, common::WalName::WoodPecker),
            (WalName::Test, common::WalName::Test),
        ];

        for (sdk, proto) in values {
            assert_eq!(sdk.into_proto(), proto);
            assert_eq!(WalName::from_proto(proto as i32), sdk);
        }
        assert_eq!(WalName::from_proto(i32::MAX), WalName::Unknown);
    }

    #[test]
    fn replicate_message_id_constructor_preserves_values() {
        let value = ReplicateMessageId::new()
            .id("message")
            .wal_name(WalName::Unknown);

        assert_eq!(value.get_id(), "message");
        assert_eq!(value.get_wal_name().to_owned(), WalName::Unknown);
    }

    #[test]
    fn replicate_message_id_direct_setters_round_trip_values() {
        let value = ReplicateMessageId::new()
            .id("message-1")
            .wal_name(WalName::Pulsar);

        assert_eq!(value.get_id().to_owned(), "message-1");
        assert_eq!(value.get_wal_name().to_owned(), WalName::Pulsar);
        assert_eq!(
            ReplicateMessageId::new()
                .id("message-1")
                .wal_name(WalName::Pulsar),
            value
        );
    }

    #[test]
    fn replicate_checkpoint_default_preserves_defaults() {
        let value = ReplicateCheckpoint::new();

        assert!(value.get_cluster_id().is_empty());
        assert!(value.get_physical_channel().is_empty());
        assert_eq!(
            value.get_message_id().to_owned(),
            ReplicateMessageId::new().id("").wal_name(WalName::Unknown)
        );
        assert_eq!(value.get_time_tick().to_owned(), 0);
    }

    #[test]
    fn replicate_checkpoint_direct_setters_round_trip_values() {
        let message_id = ReplicateMessageId::new()
            .id("message-1")
            .wal_name(WalName::Kafka);
        let value = ReplicateCheckpoint::new()
            .cluster_id("source-cluster")
            .physical_channel("channel-1")
            .message_id(message_id.clone())
            .time_tick(100);

        assert_eq!(value.get_cluster_id().to_owned(), "source-cluster");
        assert_eq!(value.get_physical_channel().to_owned(), "channel-1");
        assert_eq!(value.get_message_id().to_owned(), message_id);
        assert_eq!(value.get_time_tick().to_owned(), 100);

        let converted = ReplicateCheckpoint::from_proto(common::ReplicateCheckpoint {
            cluster_id: "source-cluster".into(),
            pchannel: "channel-1".into(),
            message_id: Some(common::MessageId {
                id: "message-1".into(),
                wal_name: common::WalName::Kafka as i32,
            }),
            time_tick: 100,
        });
        assert_eq!(converted.unwrap(), value);
    }

    #[test]
    fn replication_types_reject_missing_required_nested_messages() {
        assert!(matches!(
            ReplicateCheckpoint::from_proto(common::ReplicateCheckpoint::default()),
            Err(Error::MalformedResponse(_))
        ));

        assert!(matches!(
            ReplicateConfiguration::from_proto(common::ReplicateConfiguration {
                clusters: vec![common::MilvusCluster {
                    cluster_id: "source".into(),
                    connection_param: None,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            Err(Error::MalformedResponse(_))
        ));
    }

    #[test]
    fn dumped_message_default_preserves_defaults() {
        let value = DumpedMessage::new();

        assert_eq!(
            value.get_message_id().to_owned(),
            ReplicateMessageId::new().id("").wal_name(WalName::Unknown)
        );
        assert!(value.get_payload().is_empty());
        assert!(value.get_properties().is_empty());
    }

    #[test]
    fn dumped_message_direct_setters_round_trip_values() {
        let message_id = ReplicateMessageId::new()
            .id("message-1")
            .wal_name(WalName::WoodPecker);
        let properties = HashMap::from([("key".to_owned(), "value".to_owned())]);
        let value = DumpedMessage::new()
            .message_id(message_id.clone())
            .payload(vec![1, 2, 3])
            .properties(properties.clone());

        assert_eq!(value.get_message_id().to_owned(), message_id);
        assert_eq!(value.get_payload().to_owned(), [1, 2, 3]);
        assert_eq!(value.get_properties().to_owned(), properties);

        let converted = DumpedMessage::from_proto(common::ImmutableMessage {
            id: Some(common::MessageId {
                id: "message-1".into(),
                wal_name: common::WalName::WoodPecker as i32,
            }),
            payload: vec![1, 2, 3],
            properties,
        })
        .expect("dumped message has an ID");
        assert_eq!(converted, value);
    }

    #[test]
    fn dumped_message_rejects_a_missing_message_id() {
        assert!(matches!(
            DumpedMessage::from_proto(common::ImmutableMessage {
                id: None,
                payload: vec![1, 2, 3],
                properties: HashMap::new(),
            }),
            Err(Error::MalformedResponse(message))
                if message.contains("missing its message ID")
        ));
    }
}
