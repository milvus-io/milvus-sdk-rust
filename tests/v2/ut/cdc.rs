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

use super::common::MockServer;
use milvus::v2::error::Error;
use milvus::v2::request::cdc::*;
use milvus::v2::{CrossClusterTopology, ReplicateCluster, ReplicateConfiguration, WalName};
use std::time::Duration;
use tonic::Code;

#[tokio::test]
async fn cdc_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;
    let configuration = ReplicateConfiguration::new()
        .clusters(vec![ReplicateCluster::new()
            .cluster_id("source")
            .uri("http://source:19530")
            .token("token")
            .physical_channels(["channel"])])
        .topology(vec![CrossClusterTopology::new()
            .source_cluster_id("source")
            .target_cluster_id("target")]);

    client
        .update_replicate_configuration(
            UpdateReplicateConfigurationRequest::builder()
                .configuration(configuration)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let configuration = client
        .get_replicate_configuration(
            GetReplicateConfigurationRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let configuration = configuration.configuration();
    assert_eq!(configuration.get_clusters().len().to_owned(), 1);
    let cluster = &configuration.get_clusters()[0];
    assert_eq!(cluster.get_cluster_id().to_owned(), "source");
    assert_eq!(cluster.get_uri().to_owned(), "http://source:19530");
    assert_eq!(cluster.get_token().to_owned(), "token");
    assert_eq!(cluster.get_physical_channels().to_owned(), ["channel"]);
    assert_eq!(configuration.get_topology().len().to_owned(), 1);
    assert_eq!(
        configuration.get_topology()[0].get_source_cluster_id(),
        "source"
    );
    assert_eq!(
        configuration.get_topology()[0].get_target_cluster_id(),
        "target"
    );

    let info = client
        .get_replicate_info(
            GetReplicateInfoRequest::builder()
                .source_cluster_id("source")
                .target_physical_channel("channel")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let checkpoint = info.checkpoint();
    assert_eq!(checkpoint.get_cluster_id().to_owned(), "source");
    assert_eq!(checkpoint.get_physical_channel().to_owned(), "channel");
    assert_eq!(checkpoint.get_time_tick().to_owned(), 500);
    let message_id = checkpoint.get_message_id();
    assert_eq!(message_id.get_id().to_owned(), "message-1");
    assert_eq!(message_id.get_wal_name().to_owned(), WalName::Pulsar);
    let salvage = info
        .salvage_checkpoint()
        .expect("mock replicate info contains a salvage checkpoint");
    assert_eq!(salvage.get_time_tick().to_owned(), 400);

    let mut messages = 0;
    client
        .dump_messages(
            DumpMessagesRequest::builder()
                .physical_channel("channel")
                .start_message_id(
                    ReplicateMessageId::new()
                        .id("start-message")
                        .wal_name(WalName::Pulsar),
                )
                .build()
                .expect("valid request"),
            |message| {
                messages += 1;
                let message_id = message.get_message_id();
                assert_eq!(message_id.get_id().to_owned(), "message-1");
                assert_eq!(message_id.get_wal_name().to_owned(), WalName::Pulsar);
                assert_eq!(message.get_payload().to_owned(), [1, 2, 3]);
                assert_eq!(
                    message.get_properties().get("key").unwrap().to_owned(),
                    "value"
                );
                Ok(())
            },
        )
        .await
        .unwrap();
    assert_eq!(messages, 1);

    server.assert_request_contains(
        "update_replicate_configuration",
        &["cluster_id: \"source\"", "source_cluster_id: \"source\""],
    );
    server.assert_request_contains(
        "get_replicate_info",
        &[
            "source_cluster_id: \"source\"",
            "target_pchannel: \"channel\"",
        ],
    );
    server.assert_request_contains("dump_messages", &["pchannel: \"channel\""]);

    for rpc in [
        "update_replicate_configuration",
        "get_replicate_configuration",
        "get_replicate_info",
        "dump_messages",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn dump_messages_stream_outlives_the_unary_rpc_deadline() {
    let server = MockServer::start().await;
    server.client.set_rpc_deadline(Duration::from_millis(20));

    let mut messages = 0;
    tokio::time::timeout(
        Duration::from_secs(1),
        server.client.dump_messages(
            DumpMessagesRequest::builder()
                .physical_channel("delayed-channel")
                .start_message_id(
                    ReplicateMessageId::new()
                        .id("start-message")
                        .wal_name(WalName::Pulsar),
                )
                .build()
                .expect("valid request"),
            |_| {
                messages += 1;
                Ok(())
            },
        ),
    )
    .await
    .expect("delayed dump stream completes")
    .expect("dump stream is not cancelled by the unary deadline");

    assert_eq!(messages, 1);
    server.shutdown().await;
}

#[tokio::test]
async fn dump_messages_rejects_messages_without_ids_before_callback() {
    let server = MockServer::start().await;
    let mut delivered = 0;

    let error = server
        .client
        .dump_messages(
            DumpMessagesRequest::builder()
                .physical_channel("missing-message-id")
                .start_message_id(
                    ReplicateMessageId::new()
                        .id("start-message")
                        .wal_name(WalName::Pulsar),
                )
                .build()
                .expect("valid request"),
            |_| {
                delivered += 1;
                Ok(())
            },
        )
        .await
        .expect_err("a dumped message without an ID must fail decoding");

    assert!(matches!(
        error,
        Error::MalformedResponse(message) if message.contains("missing its message ID")
    ));
    assert_eq!(delivered, 0);
    server.shutdown().await;
}

#[tokio::test]
async fn dump_messages_does_not_retry_transport_failures() {
    let server = MockServer::start().await;
    server
        .service
        .fail_next_transport("dump_messages", Code::Unavailable);

    let error = server
        .client
        .dump_messages(
            DumpMessagesRequest::builder()
                .physical_channel("channel")
                .start_message_id(
                    ReplicateMessageId::new()
                        .id("start-message")
                        .wal_name(WalName::Pulsar),
                )
                .build()
                .expect("valid request"),
            |_| Ok(()),
        )
        .await
        .expect_err("dump stream establishment must not be retried");

    assert!(matches!(error, Error::Grpc(status) if status.code() == Code::Unavailable));
    assert_eq!(server.service.call_count("dump_messages"), 1);
    server.shutdown().await;
}
