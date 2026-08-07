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
use milvus::v2::request::database::*;

#[tokio::test]
async fn database_interfaces_reach_rpc_server() {
    let server = MockServer::start().await;
    let client = &server.client;

    client.use_database("tenant").unwrap();
    assert_eq!(client.current_database(), "tenant");
    client
        .create_database(
            CreateDatabaseRequest::builder()
                .database_name("tenant")
                .properties(std::collections::HashMap::from([(
                    "initial".into(),
                    "true".into(),
                )]))
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let databases = client
        .list_databases(
            ListDatabasesRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(databases.database_names().to_owned(), ["default", "tenant"]);

    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name("tenant")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(description.database_name().to_owned(), "tenant");
    assert_eq!(description.database_id().to_owned(), 20);
    assert_eq!(description.created_timestamp().to_owned(), 200);
    assert_eq!(
        description.properties().get("initial").unwrap().to_owned(),
        "true"
    );

    client
        .alter_database_properties(
            AlterDatabasePropertiesRequest::builder()
                .database_name("tenant")
                .property("key", "value")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name("tenant")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(
        description.properties().get("key").unwrap().to_owned(),
        "value"
    );

    client
        .drop_database_properties(
            DropDatabasePropertiesRequest::builder()
                .database_name("tenant")
                .property_key("key")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name("tenant")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert!(!description.properties().contains_key("key"));
    assert_eq!(
        description.properties().get("initial").unwrap().to_owned(),
        "true"
    );

    client
        .drop_database(
            DropDatabaseRequest::builder()
                .database_name("tenant")
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    let databases = client
        .list_databases(
            ListDatabasesRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(databases.database_names().to_owned(), ["default"]);

    server.assert_request_contains("create_database", &["db_name: \"tenant\"", "initial"]);
    server.assert_request_contains("alter_database", &["db_name: \"tenant\""]);
    server.assert_request_contains("drop_database", &["db_name: \"tenant\""]);

    for rpc in [
        "create_database",
        "list_databases",
        "alter_database",
        "describe_database",
        "drop_database",
    ] {
        server.assert_called(rpc);
    }
    server.shutdown().await;
}
