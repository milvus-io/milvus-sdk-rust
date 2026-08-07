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

use milvus::v2::request::utility::{
    CheckHealthRequest, GetServerVersionRequest, ListPersistentSegmentsRequest,
    ListQuerySegmentsRequest,
};

use super::common;

/// Requires a Milvus server. Run explicitly with:
/// `cargo test --test v2_st -- --nocapture`
#[tokio::test]
async fn connect_and_check_health() {
    let client = common::client().await;
    let response = client
        .check_health(
            CheckHealthRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("check health");
    assert!(response.is_healthy(), "{}", response.reasons().join(", "));

    let version = client
        .server_version(
            GetServerVersionRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get server version");
    assert!(!version.version().is_empty());
    assert_eq!(version.build_time(), None);
    assert_eq!(version.git_commit(), None);
    assert_eq!(version.go_version(), None);
    assert_eq!(version.deploy_mode(), None);

    let details = client
        .server_version(
            GetServerVersionRequest::builder()
                .detail(true)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get detailed server version");
    assert!(!details.version().is_empty());
    assert!(details.build_time().is_some_and(|value| !value.is_empty()));
    assert!(details.git_commit().is_some_and(|value| !value.is_empty()));
    assert!(details.go_version().is_some_and(|value| !value.is_empty()));
    assert!(details.deploy_mode().is_some_and(|value| !value.is_empty()));
}

#[tokio::test]
async fn list_segments_preserves_collection_name() {
    let client = common::client().await;
    let collection_name = common::unique_collection_name("segments");
    let _cleanup = common::CollectionCleanup::new([&collection_name]);
    common::prepare_loaded_collection(&client, &collection_name).await;

    let persistent = client
        .list_persistent_segments(
            ListPersistentSegmentsRequest::builder()
                .collection_name(&collection_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list persistent segments");
    assert!(!persistent.segments().is_empty());
    assert!(persistent
        .segments()
        .iter()
        .all(|segment| segment.get_collection_name() == &collection_name));

    let query = client
        .list_query_segments(
            ListQuerySegmentsRequest::builder()
                .collection_name(&collection_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list query segments");
    assert!(!query.segments().is_empty());
    assert!(query
        .segments()
        .iter()
        .all(|segment| segment.get_collection_name() == &collection_name));

    common::drop_collection(&client, &collection_name)
        .await
        .expect("drop segment collection");
}
