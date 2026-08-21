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

//! ClientV2 change-data-capture and replication operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Updates the CDC replication configuration for the current database.
    pub async fn update_replicate_configuration(
        &self,
        request: request::cdc::UpdateReplicateConfigurationRequest,
    ) -> Result<()> {
        let status = status_rpc_with_retry!(
            Idempotent,
            self,
            update_replicate_configuration,
            request.into_proto()
        )?;
        self.status(status)
    }

    /// Retrieves the current CDC replication configuration.
    pub async fn get_replicate_configuration(
        &self,
        request: request::cdc::GetReplicateConfigurationRequest,
    ) -> Result<response::cdc::GetReplicateConfigurationResponse> {
        let response = rpc_with_retry!(self, get_replicate_configuration, request.into_proto())?;
        status_to_result(&response.status)?;
        response::cdc::GetReplicateConfigurationResponse::from_proto(response)
    }

    /// Retrieves replication progress and metadata for a CDC channel.
    pub async fn get_replicate_info(
        &self,
        request: request::cdc::GetReplicateInfoRequest,
    ) -> Result<response::cdc::GetReplicateInfoResponse> {
        let response = self
            .retry_transport(
                request.into_proto(),
                true,
                |mut service, request| async move { service.get_replicate_info(request).await },
            )
            .await?;
        response::cdc::GetReplicateInfoResponse::from_proto(response)
    }

    /// Consume dumped WAL messages with an SDK-domain callback.
    ///
    /// This streaming operation is not retried. Reissuing it after an ambiguous
    /// transport failure could replay messages that the server already delivered.
    ///
    /// Returning an error from the callback stops iteration and drops the
    /// underlying gRPC stream.
    pub async fn dump_messages<F>(
        &self,
        request: request::cdc::DumpMessagesRequest,
        mut on_message: F,
    ) -> Result<()>
    where
        F: FnMut(&response::cdc::DumpedMessage) -> Result<()>,
    {
        use crate::proto::milvus::dump_messages_response::Response;

        let mut service = self.service.read().clone();
        let mut stream = service
            .dump_messages(tonic::Request::new(request.into_proto()))
            .await?
            .into_inner();
        while let Some(value) = stream.message().await? {
            match value.response {
                Some(Response::Message(message)) => {
                    let message = response::cdc::DumpedMessage::from_proto(message)?;
                    on_message(&message)?;
                }
                Some(Response::Status(status)) => status_to_result(&Some(status))?,
                None => {
                    return Err(crate::v2::error::Error::MalformedResponse(
                        "dump messages returned an empty stream item".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
