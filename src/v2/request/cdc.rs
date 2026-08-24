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

//! Request types for change-data-capture and replication operations.

use crate::proto::{common, milvus};
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::required;
pub use crate::v2::types::{
    CrossClusterTopology, ReplicateCluster, ReplicateConfiguration, ReplicateMessageId, WalName,
};

///////////////////////////////////////////////////////////////////////////////
// UpdateReplicateConfigurationRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 update_replicate_configuration operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UpdateReplicateConfigurationRequest {
    pub(crate) configuration: ReplicateConfiguration,
    pub(crate) force_promote: bool,
}

impl UpdateReplicateConfigurationRequest {
    fn empty() -> Self {
        Self {
            configuration: ReplicateConfiguration::new(),
            force_promote: false,
        }
    }
}

impl UpdateReplicateConfigurationRequest {
    /// Creates a builder for this request.
    pub fn builder() -> UpdateReplicateConfigurationRequestBuilder {
        UpdateReplicateConfigurationRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UpdateReplicateConfigurationRequestBuilder {
        UpdateReplicateConfigurationRequestBuilder { value: self }
    }

    /// Returns the configuration.
    pub fn configuration(&self) -> &ReplicateConfiguration {
        &self.configuration
    }

    /// Returns whether the request should force promote.
    pub fn should_force_promote(&self) -> bool {
        self.force_promote
    }

    pub(crate) fn into_proto(self) -> milvus::UpdateReplicateConfigurationRequest {
        milvus::UpdateReplicateConfigurationRequest {
            replicate_configuration: Some(self.configuration.into_proto()),
            force_promote: self.force_promote,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateReplicateConfigurationRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UpdateReplicateConfigurationRequest.
#[derive(Debug, Clone)]
pub struct UpdateReplicateConfigurationRequestBuilder {
    value: UpdateReplicateConfigurationRequest,
}

impl UpdateReplicateConfigurationRequestBuilder {
    /// Sets the configuration and returns the updated value.
    pub fn configuration(mut self, value: ReplicateConfiguration) -> Self {
        self.value.configuration = value;
        self
    }

    /// Sets the force promote and returns the updated value.
    pub fn force_promote(mut self, value: bool) -> Self {
        self.value.force_promote = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<UpdateReplicateConfigurationRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateConfigurationRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_replicate_configuration operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetReplicateConfigurationRequest;

impl GetReplicateConfigurationRequest {
    /// Creates a builder for this request.
    pub fn builder() -> GetReplicateConfigurationRequestBuilder {
        GetReplicateConfigurationRequestBuilder
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetReplicateConfigurationRequestBuilder {
        GetReplicateConfigurationRequestBuilder
    }

    pub(crate) fn into_proto(self) -> milvus::GetReplicateConfigurationRequest {
        milvus::GetReplicateConfigurationRequest {
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateConfigurationRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetReplicateConfigurationRequest.
#[derive(Debug, Clone, Copy)]
pub struct GetReplicateConfigurationRequestBuilder;

impl GetReplicateConfigurationRequestBuilder {
    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetReplicateConfigurationRequest> {
        Ok(GetReplicateConfigurationRequest)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateInfoRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_replicate_info operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetReplicateInfoRequest {
    pub(crate) source_cluster_id: String,
    pub(crate) target_physical_channel: String,
}

impl GetReplicateInfoRequest {
    fn empty() -> Self {
        Self {
            source_cluster_id: Default::default(),
            target_physical_channel: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetReplicateInfoRequestBuilder {
        GetReplicateInfoRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetReplicateInfoRequestBuilder {
        GetReplicateInfoRequestBuilder { value: self }
    }

    /// Returns the source cluster id.
    pub fn source_cluster_id(&self) -> &str {
        &self.source_cluster_id
    }

    /// Returns the target physical channel.
    pub fn target_physical_channel(&self) -> &str {
        &self.target_physical_channel
    }

    pub(crate) fn into_proto(self) -> milvus::GetReplicateInfoRequest {
        milvus::GetReplicateInfoRequest {
            source_cluster_id: self.source_cluster_id,
            target_pchannel: self.target_physical_channel,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetReplicateInfoRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetReplicateInfoRequest.
#[derive(Debug, Clone)]
pub struct GetReplicateInfoRequestBuilder {
    value: GetReplicateInfoRequest,
}

impl GetReplicateInfoRequestBuilder {
    /// Sets the source cluster id and returns the updated value.
    pub fn source_cluster_id(mut self, value: impl Into<String>) -> Self {
        self.value.source_cluster_id = value.into();
        self
    }

    /// Sets the target physical channel and returns the updated value.
    pub fn target_physical_channel(mut self, value: impl Into<String>) -> Self {
        self.value.target_physical_channel = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetReplicateInfoRequest> {
        required("source_cluster_id", &self.value.source_cluster_id)?;
        required(
            "target_physical_channel",
            &self.value.target_physical_channel,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DumpMessagesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 dump_messages operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DumpMessagesRequest {
    pub(crate) physical_channel: String,
    pub(crate) start_message_id: ReplicateMessageId,
    pub(crate) start_time_tick: u64,
    pub(crate) end_time_tick: u64,
    pub(crate) include_start_message: bool,
}

impl DumpMessagesRequest {
    fn empty() -> Self {
        Self {
            physical_channel: String::new(),
            start_message_id: ReplicateMessageId::new(),
            start_time_tick: 0,
            end_time_tick: 0,
            include_start_message: false,
        }
    }
}

impl DumpMessagesRequest {
    /// Creates a builder for this request.
    pub fn builder() -> DumpMessagesRequestBuilder {
        DumpMessagesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DumpMessagesRequestBuilder {
        DumpMessagesRequestBuilder { value: self }
    }

    /// Returns the physical channel.
    pub fn physical_channel(&self) -> &str {
        &self.physical_channel
    }

    /// Returns the start message id.
    pub fn start_message_id(&self) -> &ReplicateMessageId {
        &self.start_message_id
    }

    /// Returns the start time tick.
    pub fn start_time_tick(&self) -> u64 {
        self.start_time_tick
    }

    /// Returns the end time tick.
    pub fn end_time_tick(&self) -> u64 {
        self.end_time_tick
    }

    /// Returns whether the request should include start message.
    pub fn should_include_start_message(&self) -> bool {
        self.include_start_message
    }

    pub(crate) fn into_proto(self) -> milvus::DumpMessagesRequest {
        milvus::DumpMessagesRequest {
            pchannel: self.physical_channel,
            start_message_id: Some(common::MessageId {
                id: self.start_message_id.id,
                wal_name: self.start_message_id.wal_name.into_proto() as i32,
            }),
            start_timetick: self.start_time_tick,
            end_timetick: self.end_time_tick,
            include_start_message: self.include_start_message,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DumpMessagesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DumpMessagesRequest.
#[derive(Debug, Clone)]
pub struct DumpMessagesRequestBuilder {
    value: DumpMessagesRequest,
}

impl DumpMessagesRequestBuilder {
    /// Sets the physical channel and returns the updated value.
    pub fn physical_channel(mut self, value: impl Into<String>) -> Self {
        self.value.physical_channel = value.into();
        self
    }

    /// Sets the start message id and returns the updated value.
    pub fn start_message_id(mut self, value: ReplicateMessageId) -> Self {
        self.value.start_message_id = value;
        self
    }

    /// Sets the start time tick and returns the updated value.
    pub fn start_time_tick(mut self, value: u64) -> Self {
        self.value.start_time_tick = value;
        self
    }

    /// Sets the end time tick and returns the updated value.
    pub fn end_time_tick(mut self, value: u64) -> Self {
        self.value.end_time_tick = value;
        self
    }

    /// Sets the include start message and returns the updated value.
    pub fn include_start_message(mut self, value: bool) -> Self {
        self.value.include_start_message = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DumpMessagesRequest> {
        required("physical_channel", &self.value.physical_channel)?;
        required("start_message_id.id", self.value.start_message_id.get_id())?;
        if self.value.start_message_id.get_wal_name() == WalName::Unknown {
            return Err(Error::validation(
                "start_message_id.wal_name".into(),
                "must be specified".into(),
            ));
        }
        if self.value.end_time_tick != 0 && self.value.end_time_tick < self.value.start_time_tick {
            return Err(Error::validation(
                "end_time_tick".into(),
                "must be zero or greater than or equal to start_time_tick".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{DumpMessagesRequest, ReplicateMessageId, WalName};

    #[test]
    fn dump_messages_request_owns_only_sdk_fields() {
        let proto = DumpMessagesRequest::builder()
            .physical_channel("by-dev-rootcoord-dml_0")
            .start_message_id(
                ReplicateMessageId::new()
                    .id("message-1")
                    .wal_name(WalName::Kafka),
            )
            .start_time_tick(10)
            .end_time_tick(20)
            .build()
            .expect("valid request")
            .into_proto();
        assert_eq!(proto.pchannel, "by-dev-rootcoord-dml_0");
        assert_eq!(proto.start_timetick, 10);
        assert_eq!(proto.end_timetick, 20);
        assert!(!proto.include_start_message);
        assert_eq!(proto.start_message_id.unwrap().id, "message-1");
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn get_replicate_configuration_request_default_values() {
        assert_eq!(
            GetReplicateConfigurationRequest::builder()
                .build()
                .expect("valid request")
                .into_proto(),
            milvus::GetReplicateConfigurationRequest::default()
        );
    }

    #[test]
    fn get_replicate_configuration_request_populated_values() {
        let value = GetReplicateConfigurationRequest::builder()
            .build()
            .expect("valid request");
        assert_eq!(
            value.into_proto(),
            milvus::GetReplicateConfigurationRequest::default()
        );
    }

    #[test]
    fn update_replicate_configuration_request_default_values() {
        let value = UpdateReplicateConfigurationRequest::empty();
        let expected_configuration = ReplicateConfiguration::new();
        let expected_force_promote: bool = false;

        assert_eq!(value.configuration().to_owned(), expected_configuration);
        assert_eq!(
            value.should_force_promote().to_owned(),
            expected_force_promote
        );
        let proto = value.into_proto();
        assert_eq!(
            proto.replicate_configuration,
            Some(expected_configuration.into_proto())
        );
    }

    #[test]
    fn update_replicate_configuration_request_populated_values() {
        let configuration = ReplicateConfiguration::new();
        let force_promote = true;
        let value = UpdateReplicateConfigurationRequest::builder()
            .configuration(configuration.clone())
            .force_promote(force_promote.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.configuration().to_owned(), configuration);
        assert_eq!(value.should_force_promote().to_owned(), force_promote);
    }

    #[test]
    fn get_replicate_info_request_default_values() {
        let value = GetReplicateInfoRequest::empty();
        let expected_source_cluster_id: String = String::new();
        let expected_target_physical_channel: String = String::new();

        assert_eq!(
            value.source_cluster_id().to_owned(),
            expected_source_cluster_id
        );
        assert_eq!(
            value.target_physical_channel(),
            &expected_target_physical_channel
        );
    }

    #[test]
    fn get_replicate_info_request_populated_values() {
        let source_cluster_id = "source_cluster_id-value".to_owned();
        let target_physical_channel = "target_physical_channel-value".to_owned();
        let value = GetReplicateInfoRequest::builder()
            .source_cluster_id(source_cluster_id.clone())
            .target_physical_channel(target_physical_channel.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.source_cluster_id().to_owned(), source_cluster_id);
        assert_eq!(value.target_physical_channel(), &target_physical_channel);
    }

    #[test]
    fn dump_messages_request_default_values() {
        let value = DumpMessagesRequest::empty();
        let expected_physical_channel: String = String::new();
        let expected_start_message_id = ReplicateMessageId::new().id("").wal_name(WalName::Unknown);
        let expected_start_time_tick: u64 = 0;
        let expected_end_time_tick: u64 = 0;
        let expected_include_start_message = false;

        assert_eq!(
            value.physical_channel().to_owned(),
            expected_physical_channel
        );
        assert_eq!(
            value.start_message_id().to_owned(),
            expected_start_message_id
        );
        assert_eq!(value.start_time_tick().to_owned(), expected_start_time_tick);
        assert_eq!(value.end_time_tick().to_owned(), expected_end_time_tick);
        assert_eq!(
            value.should_include_start_message(),
            expected_include_start_message
        );
        let proto = value.into_proto();
        assert_eq!(proto.start_message_id, Some(common::MessageId::default()));
        assert!(!proto.include_start_message);
    }

    #[test]
    fn dump_messages_request_populated_values() {
        let physical_channel = "physical_channel-value".to_owned();
        let start_message_id = ReplicateMessageId::new()
            .id("message")
            .wal_name(WalName::Pulsar);
        let start_time_tick = 7;
        let end_time_tick = 7;
        let include_start_message = true;
        let value = DumpMessagesRequest::builder()
            .physical_channel(physical_channel.clone())
            .start_message_id(start_message_id.clone())
            .start_time_tick(start_time_tick.clone())
            .end_time_tick(end_time_tick.clone())
            .include_start_message(include_start_message)
            .build()
            .expect("valid request");

        assert_eq!(value.physical_channel().to_owned(), physical_channel);
        assert_eq!(value.start_message_id().to_owned(), start_message_id);
        assert_eq!(value.start_time_tick().to_owned(), start_time_tick);
        assert_eq!(value.end_time_tick().to_owned(), end_time_tick);
        assert_eq!(value.should_include_start_message(), include_start_message);
    }
}
