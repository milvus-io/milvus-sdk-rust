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
use milvus::v2::request::utility::*;
use milvus::v2::RefreshExternalCollectionStateCode;
use tonic::Code;

#[tokio::test]
async fn external_collection_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    let refresh = client
        .refresh_external_collection(
            RefreshExternalCollectionRequest::builder()
                .collection_name("books")
                .external_source("s3://bucket/path")
                .external_spec(serde_json::json!({"format": "parquet"}))
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(refresh.job_id(), 7);

    let progress = client
        .get_refresh_external_collection_progress(
            GetRefreshExternalCollectionProgressRequest::builder()
                .job_id(7)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let job_info = progress.job_info();
    assert_eq!(job_info.get_job_id(), 7);
    assert_eq!(job_info.get_collection_name(), "books");
    assert_eq!(
        job_info.get_state(),
        RefreshExternalCollectionStateCode::Completed
    );
    assert_eq!(job_info.get_progress(), 100);
    assert_eq!(job_info.get_external_source(), "s3://bucket/path");

    let jobs = client
        .list_refresh_external_collection_jobs(
            ListRefreshExternalCollectionJobsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(jobs.jobs().len(), 1);
    assert_eq!(
        jobs.jobs()[0].get_state(),
        RefreshExternalCollectionStateCode::InProgress
    );

    server.assert_request_contains(
        "refresh_external_collection",
        &[
            "collection_name: \"books\"",
            "external_source: \"s3://bucket/path\"",
            "external_spec: \"{\\\"format\\\":\\\"parquet\\\"}\"",
        ],
    );
    server.assert_request_contains("get_refresh_external_collection_progress", &["job_id: 7"]);
    server.assert_request_contains(
        "list_refresh_external_collection_jobs",
        &["collection_name: \"books\""],
    );

    for rpc in [
        "refresh_external_collection",
        "get_refresh_external_collection_progress",
        "list_refresh_external_collection_jobs",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn file_resource_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .add_file_resource(
            AddFileResourceRequest::builder()
                .name("data-files")
                .path("s3://bucket/path")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let resources = client
        .list_file_resources(
            ListFileResourcesRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(resources.resources().len(), 1);
    assert_eq!(resources.resources()[0].get_name(), "data-files");
    assert_eq!(resources.resources()[0].get_path(), "s3://bucket/path");
    client
        .remove_file_resource(
            RemoveFileResourceRequest::builder()
                .name("data-files")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();

    server.assert_request_contains(
        "add_file_resource",
        &["name: \"data-files\"", "path: \"s3://bucket/path\""],
    );
    server.assert_request_contains("remove_file_resource", &["name: \"data-files\""]);
    for rpc in [
        "add_file_resource",
        "remove_file_resource",
        "list_file_resources",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn refresh_and_file_resource_mutations_are_not_replayed_after_transport_failures() {
    let server = MockServer::start().await;
    server
        .service
        .fail_next_transport("refresh_external_collection", Code::Unavailable);
    server
        .service
        .fail_next_transport("add_file_resource", Code::Unavailable);

    let refresh_error = server
        .client
        .refresh_external_collection(
            RefreshExternalCollectionRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .expect_err("ambiguous transport failure must not be replayed");
    assert!(matches!(refresh_error, Error::Grpc(status) if status.code() == Code::Unavailable));
    assert_eq!(server.service.call_count("refresh_external_collection"), 1);

    let add_error = server
        .client
        .add_file_resource(
            AddFileResourceRequest::builder()
                .name("data-files")
                .path("s3://bucket/path")
                .build()
                .expect("valid request"),
        )
        .await
        .expect_err("ambiguous transport failure must not be replayed");
    assert!(matches!(add_error, Error::Grpc(status) if status.code() == Code::Unavailable));
    assert_eq!(server.service.call_count("add_file_resource"), 1);
    server.shutdown().await;
}
