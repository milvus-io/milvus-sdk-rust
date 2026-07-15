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

use milvus::v2::request::database::{
    AlterDatabasePropertiesRequest, CreateDatabaseRequest, DescribeDatabaseRequest,
    DropDatabasePropertiesRequest, DropDatabaseRequest, ListDatabasesRequest,
};

use super::common;

#[tokio::test]
async fn database_lifecycle_and_selection() {
    let client = common::client().await;
    let database = common::unique_name("database");
    let collection = common::unique_collection_name("database_collection");
    let _cleanup = common::CollectionCleanup::in_database(&database, [&collection]);

    client
        .create_database(
            CreateDatabaseRequest::builder()
                .database_name(&database)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create database");

    let databases = client
        .list_databases(
            ListDatabasesRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list databases");
    assert!(databases.database_names().contains(&database));

    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(&database)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe database");
    assert_eq!(description.database_name().to_owned(), database);

    client
        .alter_database_properties(
            AlterDatabasePropertiesRequest::builder()
                .database_name(&database)
                .property("database.replica.number", "1")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("alter database properties");
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(&database)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe database after altering properties");
    assert_eq!(
        description.properties().get("database.replica.number"),
        Some(&"1".to_owned())
    );

    client
        .drop_database_properties(
            DropDatabasePropertiesRequest::builder()
                .database_name(&database)
                .property_key("database.replica.number")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop database properties");
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(&database)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe database after dropping properties");
    assert!(!description
        .properties()
        .contains_key("database.replica.number"));

    client
        .use_database(&database)
        .expect("use created database");
    assert_eq!(client.current_database(), database);
    common::create_advanced_collection(&client, &collection).await;
    common::drop_collection(&client, &collection)
        .await
        .expect("drop collection from selected database");

    client
        .use_database("default")
        .expect("use default database");
    client
        .drop_database(
            DropDatabaseRequest::builder()
                .database_name(&database)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop database");
    let databases = client
        .list_databases(
            ListDatabasesRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list databases after drop");
    assert!(!databases.database_names().contains(&database));
}
