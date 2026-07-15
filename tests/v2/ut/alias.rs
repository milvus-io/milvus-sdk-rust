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
use milvus::v2::request::alias::*;
use milvus::v2::request::collection::CreateCollectionRequest;
use milvus::v2::request::dml::{InsertRequest, UpsertRequest};
use milvus::v2::request::dql::{GetRequest, Ids, QueryRequest};
use milvus::v2::{
    ClientV2, CollectionSchema, ConnectConfig, ConsistencyLevel, DataType, FieldData, FieldSchema,
};

#[tokio::test]
async fn alias_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client
        .create_alias(
            CreateAliasRequest::builder()
                .collection_name("books")
                .alias("books_alias")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = client
        .describe_alias(
            DescribeAliasRequest::builder()
                .alias("books_alias")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(description.database_name().to_owned(), "default");
    assert_eq!(description.alias().to_owned(), "books_alias");
    assert_eq!(description.collection_name().to_owned(), "books");

    client
        .alter_alias(
            AlterAliasRequest::builder()
                .collection_name("renamed_books")
                .alias("books_alias")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = client
        .describe_alias(
            DescribeAliasRequest::builder()
                .alias("books_alias")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(description.collection_name().to_owned(), "renamed_books");

    let aliases = client
        .list_aliases(
            ListAliasesRequest::builder()
                .collection_name("renamed_books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(aliases.database_name().to_owned(), "default");
    assert_eq!(aliases.collection_name().to_owned(), "renamed_books");
    assert_eq!(aliases.aliases(), ["books_alias"]);

    client
        .drop_alias(
            DropAliasRequest::builder()
                .alias("books_alias")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let aliases = client
        .list_aliases(
            ListAliasesRequest::builder()
                .collection_name("renamed_books")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(aliases.aliases().is_empty());

    for rpc in [
        "create_alias",
        "alter_alias",
        "describe_alias",
        "list_aliases",
        "drop_alias",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}

#[tokio::test]
async fn alias_mutations_evict_cached_alias_schema() {
    let server = MockServer::start().await;
    let client = &server.client;
    let reader = ClientV2::new(&ConnectConfig::new().uri(&server.uri))
        .await
        .expect("connect independent client to the same endpoint");
    let database = "analytics";
    let alias = "books_alias";

    for (collection, primary_key) in [("old_books", "old_id"), ("new_books", "new_id")] {
        client
            .create_collection(
                CreateCollectionRequest::builder()
                    .database_name(database)
                    .collection_name(collection)
                    .schema(
                        CollectionSchema::new().add_field(
                            FieldSchema::new()
                                .name(primary_key)
                                .data_type(DataType::Int64)
                                .primary_key(true),
                        ),
                    )
                    .build()
                    .expect("valid collection request"),
            )
            .await
            .expect("create collection");
    }

    let get = || {
        GetRequest::builder()
            .database_name(database)
            .collection_name(alias)
            .ids(Ids::Int64(vec![1]))
            .build()
            .expect("valid get request")
    };

    reader
        .get(get())
        .await
        .expect_err("missing alias has no primary key");
    assert_eq!(server.service.call_count("describe_collection"), 1);

    client
        .create_alias(
            CreateAliasRequest::builder()
                .database_name(database)
                .collection_name("old_books")
                .alias(alias)
                .build()
                .expect("valid create alias request"),
        )
        .await
        .expect("create alias");
    reader.get(get()).await.expect("get through created alias");
    assert_eq!(server.service.call_count("describe_collection"), 2);
    assert!(server.service.request_text("query").contains("old_id in"));

    client
        .alter_alias(
            AlterAliasRequest::builder()
                .database_name(database)
                .collection_name("new_books")
                .alias(alias)
                .build()
                .expect("valid alter alias request"),
        )
        .await
        .expect("alter alias");
    reader.get(get()).await.expect("get through altered alias");
    assert_eq!(server.service.call_count("describe_collection"), 3);
    assert!(server.service.request_text("query").contains("new_id in"));

    client
        .drop_alias(
            DropAliasRequest::builder()
                .database_name(database)
                .alias(alias)
                .build()
                .expect("valid drop alias request"),
        )
        .await
        .expect("drop alias");
    reader
        .get(get())
        .await
        .expect_err("dropped alias has no primary key");
    assert_eq!(server.service.call_count("describe_collection"), 4);

    server.shutdown().await;
}

#[tokio::test]
async fn alias_mutations_copy_and_clear_session_timestamps() {
    let server = MockServer::start().await;
    let client = &server.client;
    let database = "alias_session_db";
    let alias = "books_alias";

    for collection in ["old_books", "new_books"] {
        client
            .create_collection(
                CreateCollectionRequest::builder()
                    .database_name(database)
                    .collection_name(collection)
                    .schema(
                        CollectionSchema::new().add_field(
                            FieldSchema::new()
                                .name("id")
                                .data_type(DataType::Int64)
                                .primary_key(true),
                        ),
                    )
                    .build()
                    .expect("valid collection request"),
            )
            .await
            .expect("create collection");
    }

    client
        .insert(
            InsertRequest::builder()
                .database_name(database)
                .collection_name("old_books")
                .columns(vec![FieldData::Int64 {
                    name: "id".into(),
                    values: vec![1],
                }])
                .build()
                .expect("valid alias insert request"),
        )
        .await
        .expect("insert into old alias target");
    client
        .create_alias(
            CreateAliasRequest::builder()
                .database_name(database)
                .collection_name("old_books")
                .alias(alias)
                .build()
                .expect("valid create alias request"),
        )
        .await
        .expect("create alias");

    let session_query = || {
        QueryRequest::builder()
            .database_name(database)
            .collection_name(alias)
            .consistency_level(ConsistencyLevel::Session)
            .build()
            .expect("valid session query request")
    };
    client
        .query(session_query())
        .await
        .expect("query old alias target");
    server.assert_request_contains("query", &["guarantee_timestamp: 10"]);

    client
        .upsert(
            UpsertRequest::builder()
                .insert(
                    InsertRequest::builder()
                        .database_name(database)
                        .collection_name("new_books")
                        .columns(vec![FieldData::Int64 {
                            name: "id".into(),
                            values: vec![2],
                        }])
                        .build()
                        .expect("valid target insert payload"),
                )
                .build()
                .expect("valid target upsert request"),
        )
        .await
        .expect("upsert into new alias target");

    client
        .alter_alias(
            AlterAliasRequest::builder()
                .database_name(database)
                .collection_name("new_books")
                .alias(alias)
                .build()
                .expect("valid alter alias request"),
        )
        .await
        .expect("retarget alias");
    client
        .query(session_query())
        .await
        .expect("query new alias target");
    server.assert_request_contains("query", &["guarantee_timestamp: 11"]);

    client
        .drop_alias(
            DropAliasRequest::builder()
                .database_name(database)
                .alias(alias)
                .build()
                .expect("valid drop alias request"),
        )
        .await
        .expect("drop alias");
    client
        .query(session_query())
        .await
        .expect("mock query after dropping alias");
    server.assert_request_contains("query", &["guarantee_timestamp: 1"]);

    server.shutdown().await;
}

#[tokio::test]
async fn alias_and_canonical_names_share_session_timestamps() {
    let server = MockServer::start().await;
    let client = &server.client;
    let alias = "books_session_alias";

    client
        .create_alias(
            CreateAliasRequest::builder()
                .collection_name("books")
                .alias(alias)
                .build()
                .expect("valid create alias request"),
        )
        .await
        .expect("create alias");

    client
        .insert(
            InsertRequest::builder()
                .collection_name(alias)
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
                .expect("valid alias insert request"),
        )
        .await
        .expect("insert through alias");
    client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid canonical query"),
        )
        .await
        .expect("query through canonical name");
    server.assert_request_contains("query", &["guarantee_timestamp: 10"]);

    client
        .upsert(
            UpsertRequest::builder()
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
                                values: vec!["updated book".into()],
                            },
                            FieldData::FloatVector {
                                name: "vector".into(),
                                values: vec![vec![0.2, 0.3]],
                            },
                        ])
                        .build()
                        .expect("valid canonical upsert payload"),
                )
                .build()
                .expect("valid canonical upsert request"),
        )
        .await
        .expect("upsert through canonical name");
    client
        .query(
            QueryRequest::builder()
                .collection_name(alias)
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid alias query"),
        )
        .await
        .expect("query through alias");
    server.assert_request_contains("query", &["guarantee_timestamp: 11"]);

    client
        .delete(
            milvus::v2::request::dml::DeleteRequest::builder()
                .collection_name(alias)
                .ids(Ids::Int64(vec![1]))
                .build()
                .expect("valid alias delete request"),
        )
        .await
        .expect("delete through alias");
    client
        .query(
            QueryRequest::builder()
                .collection_name("books")
                .consistency_level(ConsistencyLevel::Session)
                .build()
                .expect("valid canonical query after delete"),
        )
        .await
        .expect("query canonical name after alias delete");
    server.assert_request_contains("query", &["guarantee_timestamp: 12"]);

    server.shutdown().await;
}
