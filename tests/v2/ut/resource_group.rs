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
use milvus::v2::request::resource_group::*;
use milvus::v2::{ResourceGroupConfig, RetryConfig};
use std::collections::HashMap;
use std::time::Duration;
use tonic::Code;

const AMBIGUOUS_TRANSPORT_CODES: [Code; 5] = [
    Code::Unavailable,
    Code::Unknown,
    Code::Internal,
    Code::Aborted,
    Code::Cancelled,
];

#[tokio::test]
async fn delta_transfers_do_not_retry_ambiguous_transport_errors() {
    let server = MockServer::start().await;
    server.client.set_retry_param(
        RetryConfig::new()
            .max_attempts(3)
            .initial_backoff(Duration::ZERO)
            .max_backoff(Duration::ZERO),
    );

    for (attempt, code) in AMBIGUOUS_TRANSPORT_CODES.into_iter().enumerate() {
        server.service.fail_next_transport("transfer_node", code);
        let error = server
            .client
            .transfer_node(
                TransferNodeRequest::builder()
                    .source_group("default")
                    .target_group("rg")
                    .node_count(1)
                    .build()
                    .expect("valid transfer-node request"),
            )
            .await
            .expect_err("ambiguous transfer-node failure must be returned");
        assert!(matches!(error, Error::Grpc(status) if status.code() == code));
        assert_eq!(server.service.call_count("transfer_node"), attempt + 1);
    }

    for (attempt, code) in AMBIGUOUS_TRANSPORT_CODES.into_iter().enumerate() {
        server.service.fail_next_transport("transfer_replica", code);
        let error = server
            .client
            .transfer_replica(
                TransferReplicaRequest::builder()
                    .database_name("default")
                    .collection_name("books")
                    .source_group("default")
                    .target_group("rg")
                    .replica_count(1)
                    .build()
                    .expect("valid transfer-replica request"),
            )
            .await
            .expect_err("ambiguous transfer-replica failure must be returned");
        assert!(matches!(error, Error::Grpc(status) if status.code() == code));
        assert_eq!(server.service.call_count("transfer_replica"), attempt + 1);
    }

    server.shutdown().await;
}

#[tokio::test]
async fn resource_group_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;
    let config = ResourceGroupConfig::new().requested_nodes(1);

    client
        .create_resource_group(
            CreateResourceGroupRequest::builder()
                .name("rg")
                .config(config.clone())
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .update_resource_groups(
            UpdateResourceGroupsRequest::builder()
                .groups(HashMap::from([("rg".into(), config)]))
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .transfer_node(
            TransferNodeRequest::builder()
                .source_group("default")
                .target_group("rg")
                .node_count(1)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .transfer_replica(
            TransferReplicaRequest::builder()
                .database_name("default")
                .collection_name("books")
                .source_group("default")
                .target_group("rg")
                .replica_count(1)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let groups = client
        .list_resource_groups(
            ListResourceGroupsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(groups.group_names().to_owned(), ["default", "rg"]);

    let group = client
        .describe_resource_group(
            DescribeResourceGroupRequest::builder()
                .group_name("rg")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = group.description();
    assert_eq!(description.get_name().to_owned(), "rg");
    assert_eq!(description.get_capacity().to_owned(), 2);
    assert_eq!(description.get_available_nodes().to_owned(), 1);
    assert_eq!(
        description.get_loaded_replicas().get("books").to_owned(),
        Some(&1)
    );
    assert_eq!(
        description.get_outgoing_nodes().get("default").to_owned(),
        Some(&1)
    );
    assert_eq!(
        description.get_incoming_nodes().get("backup").to_owned(),
        Some(&1)
    );
    let config = description.get_config();
    assert_eq!(config.get_requested_nodes().to_owned(), 1);
    assert_eq!(config.get_node_limit().to_owned(), 0);
    assert!(config.get_transfer_from().is_empty());
    assert!(config.get_transfer_to().is_empty());
    assert!(config.get_node_labels().is_empty());
    assert_eq!(description.get_nodes().len().to_owned(), 1);
    assert_eq!(description.get_nodes()[0].get_id().to_owned(), 8);
    assert_eq!(
        description.get_nodes()[0].get_address().to_owned(),
        "127.0.0.1:21123"
    );
    assert_eq!(
        description.get_nodes()[0].get_hostname().to_owned(),
        "query-node"
    );
    client
        .drop_resource_group(
            DropResourceGroupRequest::builder()
                .group_name("rg")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let groups = client
        .list_resource_groups(
            ListResourceGroupsRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(groups.group_names().to_owned(), ["default"]);

    server.assert_request_contains("create_resource_group", &["resource_group: \"rg\""]);
    server.assert_request_contains("update_resource_groups", &["resource_groups:", "rg"]);
    server.assert_request_contains(
        "transfer_node",
        &[
            "source_resource_group: \"default\"",
            "target_resource_group: \"rg\"",
        ],
    );
    server.assert_request_contains("transfer_replica", &["collection_name: \"books\""]);
    server.assert_request_contains("drop_resource_group", &["resource_group: \"rg\""]);

    for rpc in [
        "create_resource_group",
        "update_resource_groups",
        "transfer_node",
        "transfer_replica",
        "list_resource_groups",
        "describe_resource_group",
        "drop_resource_group",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}
