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
use milvus::v2::request::collection::{
    CreateSimpleCollectionRequest, LoadCollectionRequest, RefreshLoadRequest,
};
use milvus::v2::request::dml::InsertRequest;
use milvus::v2::request::partition::LoadPartitionsRequest;
use milvus::v2::request::utility::*;
use milvus::v2::{CompactionStateCode, FieldData, SegmentLevel, SegmentState};
use std::time::{Duration, Instant};

#[tokio::test]
async fn compact_and_optimize_direct_describe_bypass_the_schema_cache() {
    let server = MockServer::start().await;

    server
        .client
        .insert(
            InsertRequest::builder()
                .collection_name("books")
                .columns(vec![
                    FieldData::Int64 {
                        name: "id".into(),
                        values: vec![1],
                    },
                    FieldData::VarChar {
                        name: "text".into(),
                        values: vec!["book".into()],
                    },
                    FieldData::FloatVector {
                        name: "vector".into(),
                        values: vec![vec![0.1, 0.2]],
                    },
                ])
                .build()
                .expect("valid insert request"),
        )
        .await
        .expect("prime schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .client
        .compact(
            CompactRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid compact request"),
        )
        .await
        .expect("compact collection");
    assert_eq!(server.service.call_count("describe_collection"), 2);
    server.assert_request_contains("manual_compaction", &["collection_id: 1"]);

    server
        .client
        .optimize(
            OptimizeRequest::builder()
                .collection_name("books")
                .target_size("2MB")
                .build()
                .expect("valid optimize request"),
        )
        .await
        .expect("optimize collection");
    assert_eq!(server.service.call_count("describe_collection"), 3);

    server.shutdown().await;
}

#[tokio::test]
async fn empty_database_name_uses_selected_database_for_workflow_rpcs() {
    let server = MockServer::start().await;
    let client = &server.client;
    client.use_database("tenant").expect("select database");

    client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name("tenant_books")
                .dimension(2)
                .build()
                .expect("valid create request"),
        )
        .await
        .expect("create tenant collection");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .database_name("")
                .collection_name("tenant_books")
                .sync(false)
                .build()
                .expect("valid load request"),
        )
        .await
        .expect("load tenant collection");
    client
        .refresh_load(
            RefreshLoadRequest::builder()
                .database_name("")
                .collection_name("tenant_books")
                .sync(false)
                .build()
                .expect("valid refresh request"),
        )
        .await
        .expect("refresh tenant collection");
    client
        .load_partitions(
            LoadPartitionsRequest::builder()
                .database_name("")
                .collection_name("tenant_books")
                .partition_name("p1")
                .sync(false)
                .build()
                .expect("valid load partitions request"),
        )
        .await
        .expect("load tenant partitions");
    client
        .flush(
            FlushRequest::builder()
                .database_name("")
                .collection_names(["tenant_books"])
                .wait_flushed_ms(1_000)
                .build()
                .expect("valid flush request"),
        )
        .await
        .expect("flush tenant collection");
    client
        .flush_all(
            FlushAllRequest::builder()
                .database_name("")
                .wait_flushed_ms(1_000)
                .build()
                .expect("valid flush-all request"),
        )
        .await
        .expect("flush tenant database");
    client
        .optimize(
            OptimizeRequest::builder()
                .database_name("")
                .collection_name("tenant_books")
                .target_size("2MB")
                .build()
                .expect("valid optimize request"),
        )
        .await
        .expect("optimize tenant collection");

    for rpc in [
        "load_collection",
        "load_partitions",
        "flush",
        "flush_all",
        "manual_compaction",
    ] {
        server.assert_request_contains(rpc, &["db_name: \"tenant\""]);
    }
    let load_requests = server.service.request_texts("load_collection");
    assert!(
        load_requests.len() >= 2
            && load_requests
                .iter()
                .all(|request| request.contains("db_name: \"tenant\"")),
        "load and refresh requests should use the selected database: {load_requests:?}"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn flush_wait_timeout_bounds_stalled_polls_and_visibility_delay() {
    let server = MockServer::start().await;

    let flush_started = Instant::now();
    let flush_result = tokio::time::timeout(
        Duration::from_secs(2),
        server.client.flush(
            FlushRequest::builder()
                .collection_names(["stalled_flush_state"])
                .wait_flushed_ms(1_050)
                .build()
                .expect("valid flush request"),
        ),
    )
    .await
    .expect("flush must honor its operation timeout");
    let flush_error = flush_result.expect_err("stalled flush-state poll must time out");
    assert!(matches!(
        flush_error,
        Error::Timeout(message) if message == "waiting for flush completion"
    ));
    assert!(flush_started.elapsed() < Duration::from_millis(1_500));

    let flush_all_started = Instant::now();
    let flush_all_result = tokio::time::timeout(
        Duration::from_secs(2),
        server.client.flush_all(
            FlushAllRequest::builder()
                .database_name("stalled_flush_all_state")
                .wait_flushed_ms(550)
                .build()
                .expect("valid flush-all request"),
        ),
    )
    .await
    .expect("flush-all must honor its operation timeout");
    let flush_all_error = flush_all_result.expect_err("stalled flush-all-state poll must time out");
    assert!(matches!(
        flush_all_error,
        Error::Timeout(message) if message == "waiting for flush completion"
    ));
    assert!(flush_all_started.elapsed() < Duration::from_secs(1));

    let visibility_started = Instant::now();
    server
        .client
        .flush(
            FlushRequest::builder()
                .collection_names(["short_flush_visibility_delay"])
                .wait_flushed_ms(1_050)
                .build()
                .expect("valid flush request"),
        )
        .await
        .expect("successful flush uses only the remaining visibility delay");
    assert!(visibility_started.elapsed() < Duration::from_millis(1_500));

    assert_eq!(server.service.call_count("get_flush_state"), 2);
    assert_eq!(server.service.call_count("get_flush_all_state"), 1);
    server.shutdown().await;
}

#[tokio::test]
async fn flush_rejects_missing_collection_timestamp() {
    let server = MockServer::start().await;

    let error = server
        .client
        .flush(
            FlushRequest::builder()
                .collection_names(["missing_flush_timestamp"])
                .wait_flushed_ms(1_000)
                .build()
                .expect("valid flush request"),
        )
        .await
        .expect_err("flush response without a collection timestamp must fail");

    assert!(matches!(
        error,
        Error::MalformedResponse(message)
            if message.contains("missing_flush_timestamp")
                && message.contains("no flush timestamp")
    ));
    assert_eq!(server.service.call_count("get_flush_state"), 0);
    server.shutdown().await;
}

#[tokio::test]
async fn optimize_treats_zero_compaction_plans_as_a_successful_noop() {
    let server = MockServer::start().await;
    let task = server
        .client
        .optimize(
            OptimizeRequest::builder()
                .collection_name("books")
                .target_size("2MB")
                .build()
                .expect("valid optimize request"),
        )
        .await
        .expect("start optimize task");

    let result = task
        .get_result(1_000)
        .await
        .expect("zero-plan optimize succeeds");
    assert_eq!(result.status_text(), "success");
    assert_eq!(result.compaction_id(), -1);
    assert!(result
        .progress_history()
        .iter()
        .any(|progress| progress == "no compaction required"));
    assert_eq!(server.service.call_count("get_compaction_state"), 0);

    server.shutdown().await;
}

#[tokio::test]
async fn utility_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    assert!(!client.sdk_version().is_empty());
    let version = client
        .server_version(
            GetServerVersionRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(version.version(), "mock-2.6");
    assert_eq!(version.build_time(), None);

    let detailed = client
        .server_version(
            GetServerVersionRequest::builder()
                .detail(true)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(detailed.version(), "mock-2.6-detail");
    assert_eq!(detailed.build_time(), Some("2026-07-29"));
    assert_eq!(detailed.git_commit(), Some("abcdef"));
    assert_eq!(detailed.go_version(), Some("go1.24"));
    assert_eq!(detailed.deploy_mode(), Some("STANDALONE"));
    let health = client
        .check_health(
            CheckHealthRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(health.is_healthy().to_owned(), true);
    assert!(health.reasons().is_empty());
    assert_eq!(health.quota_states().to_owned(), ["ReadLimited"]);

    let flush = client
        .flush(
            FlushRequest::builder()
                .collection_names(["books"])
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(flush.database_name().to_owned(), "default");
    assert!(flush.segment_ids().is_empty());
    assert_eq!(flush.flush_timestamps().get("books").to_owned(), Some(&400));

    let flush_all = client
        .flush_all(
            FlushAllRequest::builder()
                .wait_flushed_ms(1_000)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(flush_all.flush_all_timestamp().to_owned(), 401);

    let flush_state = client
        .get_flush_all_state(
            GetFlushAllStateRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(flush_state.is_flushed().to_owned(), true);

    let persistent = client
        .list_persistent_segments(
            ListPersistentSegmentsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(persistent.segments().len().to_owned(), 1);
    let segment = &persistent.segments()[0];
    assert_eq!(segment.get_segment_id().to_owned(), 40);
    assert_eq!(segment.get_collection_id().to_owned(), 1);
    assert_eq!(segment.get_partition_id().to_owned(), 1);
    assert_eq!(segment.get_row_count().to_owned(), 1);
    assert_eq!(segment.get_state().to_owned(), SegmentState::Flushed);
    assert_eq!(segment.get_collection_name().to_owned(), "books");
    assert_eq!(segment.get_level().to_owned(), SegmentLevel::L1);
    assert_eq!(segment.get_sorted().to_owned(), true);
    assert_eq!(segment.get_storage_version().to_owned(), 2);

    let query_segments = client
        .list_query_segments(
            ListQuerySegmentsRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(query_segments.segments().len().to_owned(), 1);
    let segment = &query_segments.segments()[0];
    assert_eq!(segment.get_segment_id().to_owned(), 41);
    assert_eq!(segment.get_collection_id().to_owned(), 1);
    assert_eq!(segment.get_partition_id().to_owned(), 1);
    assert_eq!(segment.get_memory_size().to_owned(), 1024);
    assert_eq!(segment.get_row_count().to_owned(), 1);
    assert_eq!(segment.get_index_name().to_owned(), "vector_idx");
    assert_eq!(segment.get_index_id().to_owned(), 10);
    assert_eq!(segment.get_node_ids().to_owned(), [8, 9]);
    assert_eq!(segment.get_state().to_owned(), SegmentState::Flushed);
    assert_eq!(segment.get_collection_name().to_owned(), "books");
    assert_eq!(segment.get_level().to_owned(), SegmentLevel::L1);
    assert_eq!(segment.get_sorted().to_owned(), true);
    assert_eq!(segment.get_storage_version().to_owned(), 2);

    let compact = client
        .compact(
            CompactRequest::builder()
                .collection_name("books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(compact.compaction_id().to_owned(), 1);
    assert_eq!(compact.plan_count().to_owned(), 1);
    let task = client
        .optimize(
            OptimizeRequest::builder()
                .collection_name("books")
                .target_size("1MB")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(task.is_done());
    assert!(!task.is_cancelled());
    assert!(task.current_progress().is_some());
    assert!(!task.progress_history().is_empty());
    let optimize = task.get_result(1_000).await.unwrap();
    assert_eq!(optimize.status_text().to_owned(), "success");
    assert_eq!(optimize.collection_name().to_owned(), "books");
    assert_eq!(optimize.compaction_id().to_owned(), 1);
    assert_eq!(optimize.target_size().to_owned(), "1MB");
    assert!(!optimize.progress_history().is_empty());
    assert!(!task.cancel());
    let compaction = client
        .get_compaction_state(
            GetCompactionStateRequest::builder()
                .compaction_id(1)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(
        compaction.state().to_owned(),
        CompactionStateCode::Completed
    );
    assert_eq!(compaction.executing_plans().to_owned(), 0);
    assert_eq!(compaction.timed_out_plans().to_owned(), 0);
    assert_eq!(compaction.completed_plans().to_owned(), 1);
    assert_eq!(compaction.failed_plans().to_owned(), 0);

    let plans = client
        .get_compaction_plans(
            GetCompactionPlansRequest::builder()
                .compaction_id(1)
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(plans.state().to_owned(), CompactionStateCode::Completed);
    assert_eq!(plans.merges().len().to_owned(), 1);
    assert_eq!(
        plans.merges()[0].get_source_segment_ids().to_owned(),
        [1, 2]
    );
    assert_eq!(plans.merges()[0].get_target_segment_id().to_owned(), 3);

    let analyzer = client
        .run_analyzer(
            RunAnalyzerRequest::builder()
                .analyzer_params("{}")
                .texts(["hello"])
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(analyzer.results().len().to_owned(), 1);
    assert_eq!(analyzer.results()[0].get_tokens().len().to_owned(), 1);
    let token = &analyzer.results()[0].get_tokens()[0];
    assert_eq!(token.get_text().to_owned(), "hello");
    assert_eq!(token.get_start_offset().to_owned(), 0);
    assert_eq!(token.get_end_offset().to_owned(), 5);
    assert_eq!(token.get_position().to_owned(), 0);
    assert_eq!(token.get_position_length().to_owned(), 1);
    assert_eq!(token.get_hash().to_owned(), 123);
    server.assert_request_contains("flush", &["collection_names: [\"books\"]"]);
    server.assert_request_contains("manual_compaction", &["collection_name: \"books\""]);
    server.assert_request_contains(
        "run_analyzer",
        &["placeholder: [[104, 101, 108, 108, 111]]"],
    );
    for rpc in [
        "get_version",
        "check_health",
        "flush",
        "flush_all",
        "get_flush_all_state",
        "get_persistent_segment_info",
        "get_query_segment_info",
        "manual_compaction",
        "get_compaction_state",
        "get_compaction_state_with_plans",
        "run_analyzer",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}
