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

//! Response types returned by change-data-capture and replication operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
pub use crate::v2::types::{
    CrossClusterTopology, DumpedMessage, ReplicateCheckpoint, ReplicateCluster,
    ReplicateConfiguration, ReplicateMessageId, WalName,
};

///////////////////////////////////////////////////////////////////////////////
// GetReplicateConfigurationResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_replicate_configuration operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetReplicateConfigurationResponse {
    pub(crate) configuration: ReplicateConfiguration,
}

impl GetReplicateConfigurationResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            configuration: ReplicateConfiguration::new(),
        }
    }
}

impl GetReplicateConfigurationResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetReplicateConfigurationResponseBuilder {
        GetReplicateConfigurationResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the configuration.
    pub fn configuration(&self) -> &ReplicateConfiguration {
        &self.configuration
    }

    pub(crate) fn from_proto(v: milvus::GetReplicateConfigurationResponse) -> Result<Self> {
        let configuration = v.configuration.ok_or_else(|| {
            Error::MalformedResponse("replicate configuration response has no configuration".into())
        })?;
        Ok(Self {
            configuration: ReplicateConfiguration::from_proto(configuration)?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateConfigurationResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetReplicateConfigurationResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetReplicateConfigurationResponseBuilder {
    value: GetReplicateConfigurationResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetReplicateConfigurationResponseBuilder {
    /// Sets the configuration and returns the updated value.
    pub fn configuration(mut self, value: ReplicateConfiguration) -> Self {
        self.value.configuration = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> GetReplicateConfigurationResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateInfoResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_replicate_info operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetReplicateInfoResponse {
    pub(crate) checkpoint: ReplicateCheckpoint,
    pub(crate) salvage_checkpoint: Option<ReplicateCheckpoint>,
}

impl GetReplicateInfoResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            checkpoint: ReplicateCheckpoint::new(),
            salvage_checkpoint: None,
        }
    }
}

impl GetReplicateInfoResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetReplicateInfoResponseBuilder {
        GetReplicateInfoResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the checkpoint.
    pub fn checkpoint(&self) -> &ReplicateCheckpoint {
        &self.checkpoint
    }

    /// Returns the salvage checkpoint.
    pub fn salvage_checkpoint(&self) -> Option<&ReplicateCheckpoint> {
        self.salvage_checkpoint.as_ref()
    }

    pub(crate) fn from_proto(v: milvus::GetReplicateInfoResponse) -> Result<Self> {
        let checkpoint = v.checkpoint.ok_or_else(|| {
            Error::MalformedResponse("replicate info response has no checkpoint".into())
        })?;
        Ok(Self {
            checkpoint: ReplicateCheckpoint::from_proto(checkpoint)?,
            salvage_checkpoint: v
                .salvage_checkpoint
                .map(ReplicateCheckpoint::from_proto)
                .transpose()?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateInfoResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetReplicateInfoResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetReplicateInfoResponseBuilder {
    value: GetReplicateInfoResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetReplicateInfoResponseBuilder {
    /// Sets the checkpoint and returns the updated value.
    pub fn checkpoint(mut self, value: ReplicateCheckpoint) -> Self {
        self.value.checkpoint = value;
        self
    }

    /// Sets the salvage checkpoint and returns the updated value.
    pub fn salvage_checkpoint(mut self, value: ReplicateCheckpoint) -> Self {
        self.value.salvage_checkpoint = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> GetReplicateInfoResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod decoding_tests {
    use super::{GetReplicateConfigurationResponse, GetReplicateInfoResponse};
    use crate::proto::{common, milvus};
    use crate::v2::error::Error;

    fn checkpoint() -> common::ReplicateCheckpoint {
        common::ReplicateCheckpoint {
            message_id: Some(common::MessageId {
                id: "message-1".into(),
                wal_name: common::WalName::Pulsar as i32,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn replicate_configuration_rejects_a_missing_configuration() {
        let error = GetReplicateConfigurationResponse::from_proto(
            milvus::GetReplicateConfigurationResponse::default(),
        )
        .unwrap_err();

        assert!(matches!(error, Error::MalformedResponse(_)));
    }

    #[test]
    fn replicate_info_requires_the_main_checkpoint_but_not_salvage() {
        let error =
            GetReplicateInfoResponse::from_proto(milvus::GetReplicateInfoResponse::default())
                .unwrap_err();
        assert!(matches!(error, Error::MalformedResponse(_)));

        let response = GetReplicateInfoResponse::from_proto(milvus::GetReplicateInfoResponse {
            checkpoint: Some(checkpoint()),
            salvage_checkpoint: None,
            ..Default::default()
        })
        .expect("salvage checkpoint is optional");
        assert!(response.salvage_checkpoint().is_none());
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod dump_message_tests {
    use super::{DumpedMessage, WalName};
    use crate::proto::common;
    use std::collections::HashMap;

    #[test]
    fn dumped_message_is_converted_to_domain_data() {
        let message = DumpedMessage::from_proto(common::ImmutableMessage {
            id: Some(common::MessageId {
                id: "message-1".into(),
                wal_name: common::WalName::Pulsar as i32,
            }),
            payload: vec![1, 2, 3],
            properties: HashMap::from([("key".into(), "value".into())]),
            ..Default::default()
        })
        .expect("dumped message has an ID");
        assert_eq!(message.message_id.wal_name, WalName::Pulsar);
        assert_eq!(message.payload, vec![1, 2, 3]);
        assert_eq!(
            message.properties.get("key").map(String::as_str),
            Some("value")
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
    fn get_replicate_configuration_response_default_values() {
        let value = GetReplicateConfigurationResponse::builder().build();
        let expected_configuration = ReplicateConfiguration::new();

        assert_eq!(value.configuration().to_owned(), expected_configuration);
    }

    #[test]
    fn get_replicate_configuration_response_populated_values() {
        let configuration = ReplicateConfiguration::new();
        let value = GetReplicateConfigurationResponse::builder()
            .configuration(configuration.clone())
            .build();

        assert_eq!(value.configuration().to_owned(), configuration);
    }

    #[test]
    fn get_replicate_info_response_default_values() {
        let value = GetReplicateInfoResponse::builder().build();
        let expected_checkpoint = ReplicateCheckpoint::new();

        assert_eq!(value.checkpoint().to_owned(), expected_checkpoint);
        assert!(value.salvage_checkpoint().is_none());
    }

    #[test]
    fn get_replicate_info_response_populated_values() {
        let checkpoint = ReplicateCheckpoint::new();
        let salvage_checkpoint = ReplicateCheckpoint::new();
        let value = GetReplicateInfoResponse::builder()
            .checkpoint(checkpoint.clone())
            .salvage_checkpoint(salvage_checkpoint.clone())
            .build();

        assert_eq!(value.checkpoint().to_owned(), checkpoint);
        assert_eq!(value.salvage_checkpoint(), Some(&salvage_checkpoint));
    }
}
