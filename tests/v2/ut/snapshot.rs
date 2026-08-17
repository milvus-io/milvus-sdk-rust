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
use milvus::v2::request::snapshot::*;
use milvus::v2::RestoreSnapshotStateCode;
use tonic::Code;

#[tokio::test]
async fn snapshot_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_snapshot(
            CreateSnapshotRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .description("backup")
                .compaction_protection_seconds(300)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    client
        .drop_snapshot(
            DropSnapshotRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    let snapshots = client
        .list_snapshots(
            ListSnapshotsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(
        snapshots.snapshots(),
        &["snap-1".to_owned(), "snap-2".to_owned()]
    );

    let snapshot = client
        .describe_snapshot(
            DescribeSnapshotRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(snapshot.name(), "snap-1");
    assert_eq!(snapshot.description(), "backup");
    assert_eq!(snapshot.collection_name(), "books");
    assert_eq!(
        snapshot.partition_names(),
        &["p1".to_owned(), "p2".to_owned()]
    );
    assert_eq!(snapshot.create_ts(), 123);
    assert_eq!(snapshot.s3_location(), "s3://bucket/export");

    let restore = client
        .restore_snapshot(
            RestoreSnapshotRequest::builder()
                .snapshot_name("snap-1")
                .source_collection_name("books")
                .target_collection_name("books_restored")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(restore.job_id(), 7);

    let state = client
        .get_restore_snapshot_state(
            GetRestoreSnapshotStateRequest::builder()
                .job_id(7)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let job_info = state.job_info();
    assert_eq!(job_info.get_job_id(), 7);
    assert_eq!(job_info.get_snapshot_name(), "snap-1");
    assert_eq!(job_info.get_collection_name(), "books");
    assert_eq!(job_info.get_state(), RestoreSnapshotStateCode::Executing);
    assert_eq!(job_info.get_progress(), 50);
    assert_eq!(job_info.get_time_cost(), 5);

    let jobs = client
        .list_restore_snapshot_jobs(
            ListRestoreSnapshotJobsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(jobs.jobs().len(), 1);
    assert_eq!(
        jobs.jobs()[0].get_state(),
        RestoreSnapshotStateCode::Completed
    );

    let pin = client
        .pin_snapshot_data(
            PinSnapshotDataRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .ttl_seconds(3600)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(pin.pin_id(), 42);
    client
        .unpin_snapshot_data(
            UnpinSnapshotDataRequest::builder()
                .pin_id(42)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    server.assert_request_contains(
        "create_snapshot",
        &[
            "name: \"snap-1\"",
            "description: \"backup\"",
            "collection_name: \"books\"",
            "compaction_protection_seconds: 300",
        ],
    );
    server.assert_request_contains(
        "restore_snapshot",
        &[
            "name: \"snap-1\"",
            "collection_name: \"books\"",
            "target_collection_name: \"books_restored\"",
        ],
    );
    server.assert_request_contains(
        "pin_snapshot_data",
        &["name: \"snap-1\"", "ttl_seconds: 3600"],
    );
    server.assert_request_contains("unpin_snapshot_data", &["pin_id: 42"]);

    for rpc in [
        "create_snapshot",
        "drop_snapshot",
        "list_snapshots",
        "describe_snapshot",
        "restore_snapshot",
        "get_restore_snapshot_state",
        "list_restore_snapshot_jobs",
        "pin_snapshot_data",
        "unpin_snapshot_data",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn snapshot_requests_use_the_selected_database() {
    let server = MockServer::start().await;
    server.client.use_database("analytics").unwrap();

    server
        .client
        .list_snapshots(
            ListSnapshotsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    server
        .client
        .restore_snapshot(
            RestoreSnapshotRequest::builder()
                .snapshot_name("snap-1")
                .source_collection_name("books")
                .target_collection_name("books_restored")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    server.assert_request_contains("list_snapshots", &["db_name: \"analytics\""]);
    server.assert_request_contains(
        "restore_snapshot",
        &["db_name: \"analytics\"", "target_db_name: \"analytics\""],
    );
    server.shutdown().await;
}

#[tokio::test]
async fn snapshot_mutations_are_not_replayed_after_ambiguous_transport_failures() {
    let server = MockServer::start().await;
    server
        .service
        .fail_next_transport("create_snapshot", Code::Unavailable);

    let error = server
        .client
        .create_snapshot(
            CreateSnapshotRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .build()
                .expect("valid request"),
        )
        .await
        .expect_err("ambiguous transport failure must not be replayed");

    assert!(matches!(error, Error::Grpc(status) if status.code() == Code::Unavailable));
    assert_eq!(server.service.call_count("create_snapshot"), 1);
    server.shutdown().await;
}

#[tokio::test]
async fn restore_and_pin_mutations_are_not_replayed_after_ambiguous_transport_failures() {
    let server = MockServer::start().await;
    server
        .service
        .fail_next_transport("restore_snapshot", Code::Unavailable);
    server
        .service
        .fail_next_transport("pin_snapshot_data", Code::Unavailable);

    let restore_error = server
        .client
        .restore_snapshot(
            RestoreSnapshotRequest::builder()
                .snapshot_name("snap-1")
                .source_collection_name("books")
                .target_collection_name("books_restored")
                .build()
                .expect("valid request"),
        )
        .await
        .expect_err("ambiguous transport failure must not be replayed");
    assert!(matches!(restore_error, Error::Grpc(status) if status.code() == Code::Unavailable));
    assert_eq!(server.service.call_count("restore_snapshot"), 1);

    let pin_error = server
        .client
        .pin_snapshot_data(
            PinSnapshotDataRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .build()
                .expect("valid request"),
        )
        .await
        .expect_err("ambiguous transport failure must not be replayed");
    assert!(matches!(pin_error, Error::Grpc(status) if status.code() == Code::Unavailable));
    assert_eq!(server.service.call_count("pin_snapshot_data"), 1);
    server.shutdown().await;
}

#[tokio::test]
async fn snapshot_reads_are_retried_after_transport_failures() {
    let server = MockServer::start().await;
    server
        .service
        .fail_next_transport("describe_snapshot", Code::Unavailable);

    let snapshot = server
        .client
        .describe_snapshot(
            DescribeSnapshotRequest::builder()
                .collection_name("books")
                .snapshot_name("snap-1")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("idempotent read is retried after a transport failure");
    assert_eq!(snapshot.name(), "snap-1");
    assert!(server.service.call_count("describe_snapshot") > 1);
    server.shutdown().await;
}
