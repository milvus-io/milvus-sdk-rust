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
use milvus::v2::prelude::*;

fn insert_request() -> InsertRequest {
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
        .unwrap()
}

fn zero_row_column_request() -> InsertRequest {
    InsertRequest::builder()
        .collection_name("books")
        .columns(vec![FieldData::Int64 {
            name: "id".into(),
            values: Vec::new(),
        }])
        .build()
        .expect("a non-empty column vector passes request-shape validation")
}

#[tokio::test]
async fn column_based_insert_and_upsert_reject_zero_rows_before_rpc() {
    let server = MockServer::start().await;

    let insert_error = server
        .client
        .insert(zero_row_column_request())
        .await
        .expect_err("zero-row insert columns must fail locally");
    assert!(
        insert_error
            .to_string()
            .contains("columns must contain at least one row"),
        "unexpected insert error: {insert_error}"
    );

    let upsert_error = server
        .client
        .upsert(
            UpsertRequest::builder()
                .insert(zero_row_column_request())
                .build()
                .expect("valid upsert request shape"),
        )
        .await
        .expect_err("zero-row upsert columns must fail locally");
    assert!(
        upsert_error
            .to_string()
            .contains("columns must contain at least one row"),
        "unexpected upsert error: {upsert_error}"
    );

    assert_eq!(server.service.call_count("insert"), 0);
    assert_eq!(server.service.call_count("upsert"), 0);
    server.shutdown().await;
}

#[tokio::test]
async fn local_validation_errors_force_refresh_the_shared_schema() {
    let server = MockServer::start().await;
    let invalid = || {
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
                    values: vec![vec![0.1, 0.2, 0.3]],
                },
            ])
            .build()
            .expect("request shape is valid before schema validation")
    };

    for _ in 0..2 {
        let error = server
            .client
            .insert(invalid())
            .await
            .expect_err("wrong vector dimension must fail locally");
        assert!(
            error.to_string().contains("vector dimension 3"),
            "unexpected validation error: {error}"
        );
    }

    assert_eq!(server.service.call_count("describe_collection"), 3);
    assert_eq!(server.service.call_count("insert"), 0);
    server.shutdown().await;
}

#[tokio::test]
async fn stale_schema_is_force_refreshed_after_local_validation_mismatch() {
    let server = MockServer::start().await;
    server
        .client
        .insert(insert_request())
        .await
        .expect("prime the schema cache");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    server
        .service
        .rename_collection_field("default", "books", "text", "title");
    server
        .client
        .insert(
            InsertRequest::builder()
                .collection_name("books")
                .columns(vec![
                    FieldData::Int64 {
                        name: "id".into(),
                        values: vec![2],
                    },
                    FieldData::VarChar {
                        name: "title".into(),
                        values: vec!["refreshed book".into()],
                    },
                    FieldData::FloatVector {
                        name: "vector".into(),
                        values: vec![vec![0.3, 0.4]],
                    },
                ])
                .build()
                .expect("valid request shape"),
        )
        .await
        .expect("insert succeeds after forced schema refresh");

    assert_eq!(server.service.call_count("describe_collection"), 2);
    assert_eq!(server.service.call_count("insert"), 2);
    server.shutdown().await;
}

#[tokio::test]
async fn dml_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    let insert = client.insert(insert_request()).await.unwrap();
    assert_eq!(insert.ids().to_owned(), Ids::Int64(vec![1]));
    assert_eq!(insert.succeeded_indices().to_owned(), [0]);
    assert!(insert.failed_indices().is_empty());
    assert_eq!(insert.is_acknowledged().to_owned(), true);
    assert_eq!(insert.insert_count().to_owned(), 1);
    assert_eq!(insert.delete_count().to_owned(), 0);
    assert_eq!(insert.upsert_count().to_owned(), 0);
    assert_eq!(insert.timestamp().to_owned(), 10);

    let upsert = client
        .upsert(
            UpsertRequest::builder()
                .insert(insert_request())
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(upsert.ids().to_owned(), Ids::Int64(vec![1]));
    assert_eq!(upsert.succeeded_indices().to_owned(), [0]);
    assert!(upsert.failed_indices().is_empty());
    assert_eq!(upsert.is_acknowledged().to_owned(), true);
    assert_eq!(upsert.insert_count().to_owned(), 0);
    assert_eq!(upsert.delete_count().to_owned(), 0);
    assert_eq!(upsert.upsert_count().to_owned(), 1);
    assert_eq!(upsert.timestamp().to_owned(), 11);

    let delete = client
        .delete(
            DeleteRequest::builder()
                .collection_name("books")
                .ids(Ids::Int64(vec![1]))
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.ids().to_owned(), Ids::Int64(vec![1]));
    assert_eq!(delete.succeeded_indices().to_owned(), [0]);
    assert!(delete.failed_indices().is_empty());
    assert_eq!(delete.is_acknowledged().to_owned(), true);
    assert_eq!(delete.insert_count().to_owned(), 0);
    assert_eq!(delete.delete_count().to_owned(), 1);
    assert_eq!(delete.upsert_count().to_owned(), 0);
    assert_eq!(delete.timestamp().to_owned(), 12);

    server.assert_request_contains("insert", &["collection_name: \"books\"", "num_rows: 1"]);
    server.assert_request_contains("upsert", &["collection_name: \"books\"", "num_rows: 1"]);
    server.assert_request_contains("delete", &["collection_name: \"books\""]);

    for rpc in ["describe_collection", "insert", "upsert", "delete"] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn column_based_partial_upsert_allows_omitted_fields() {
    let server = MockServer::start().await;
    let client = &server.client;
    let update = InsertRequest::builder()
        .collection_name("books")
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::VarChar {
                name: "text".into(),
                values: vec!["updated book".into()],
            },
        ])
        .build()
        .expect("build column-based partial upsert");

    let response = client
        .upsert(
            UpsertRequest::builder()
                .insert(update)
                .partial_update(true)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("partially upsert columns");

    assert_eq!(response.upsert_count().to_owned(), 1);
    server.assert_request_contains(
        "upsert",
        &[
            "collection_name: \"books\"",
            "num_rows: 1",
            "partial_update: true",
        ],
    );
    server.shutdown().await;
}

#[tokio::test]
async fn field_operation_is_encoded_and_implicitly_enables_partial_update() {
    let server = MockServer::start().await;
    let client = &server.client;
    let update = InsertRequest::builder()
        .collection_name("books")
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::array_varchar("tags", vec![vec!["new".into()]]),
        ])
        .build()
        .expect("build column-based field operation");

    let response = client
        .upsert(
            UpsertRequest::builder()
                .insert(update)
                .add_field_op(
                    FieldPartialUpdateOp::new()
                        .field_name("tags")
                        .op_type(FieldPartialUpdateOpType::ArrayAppend),
                )
                .build()
                .expect("valid request"),
        )
        .await
        .expect("upsert with field operation");

    assert_eq!(response.upsert_count(), 1);
    server.assert_request_contains(
        "upsert",
        &[
            "partial_update: true",
            "field_ops: [FieldPartialUpdateOp",
            "field_name: \"tags\"",
            "op: ArrayAppend",
        ],
    );
    server.shutdown().await;
}
