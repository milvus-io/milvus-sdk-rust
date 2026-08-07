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
use milvus::v2::request::partition::*;

#[tokio::test]
async fn partition_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_partition(
            CreatePartitionRequest::builder()
                .collection_name("books")
                .partition_name("p1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let has_partition = client
        .has_partition(
            HasPartitionRequest::builder()
                .collection_name("books")
                .partition_name("p1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(has_partition.exists());

    let partitions = client
        .list_partitions(
            ListPartitionsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(partitions.partition_names().to_owned(), ["p1"]);
    assert_eq!(partitions.partitions().len().to_owned(), 1);
    let partition = &partitions.partitions()[0];
    assert_eq!(partition.get_name().to_owned(), "p1");
    assert_eq!(partition.get_id().to_owned(), 1);
    assert_eq!(partition.get_created_timestamp().to_owned(), 201);
    assert_eq!(partition.get_created_utc_timestamp().to_owned(), 202);
    client
        .load_partitions(
            LoadPartitionsRequest::builder()
                .collection_name("books")
                .partition_name("p1")
                .sync(true)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .release_partitions(
            ReleasePartitionsRequest::builder()
                .collection_name("books")
                .partition_names(["p1"])
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let statistics = client
        .get_partition_stats(
            GetPartitionStatsRequest::builder()
                .collection_name("books")
                .partition_name("p1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(
        statistics.statistics().get("row_count").unwrap().to_owned(),
        "1"
    );
    client
        .drop_partition(
            DropPartitionRequest::builder()
                .collection_name("books")
                .partition_name("p1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let has_partition = client
        .has_partition(
            HasPartitionRequest::builder()
                .collection_name("books")
                .partition_name("p1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(!has_partition.exists());

    server.assert_request_contains(
        "create_partition",
        &["collection_name: \"books\"", "partition_name: \"p1\""],
    );
    server.assert_request_contains("load_partitions", &["partition_names: [\"p1\"]"]);
    server.assert_request_contains("release_partitions", &["partition_names: [\"p1\"]"]);
    server.assert_request_contains("drop_partition", &["partition_name: \"p1\""]);

    for rpc in [
        "create_partition",
        "has_partition",
        "show_partitions",
        "load_partitions",
        "get_loading_progress",
        "release_partitions",
        "get_partition_statistics",
        "drop_partition",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}
