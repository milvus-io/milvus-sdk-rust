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

use milvus::v2::request::alias::{
    AlterAliasRequest, CreateAliasRequest, DescribeAliasRequest, DropAliasRequest,
    ListAliasesRequest,
};

use super::common;

#[tokio::test]
async fn alias_lifecycle() {
    let client = common::client().await;
    let first_collection = common::unique_collection_name("alias_first");
    let second_collection = common::unique_collection_name("alias_second");
    let _cleanup = common::CollectionCleanup::new([&first_collection, &second_collection]);
    let alias = common::unique_name("alias");
    common::create_advanced_collection(&client, &first_collection).await;
    common::create_advanced_collection(&client, &second_collection).await;

    client
        .create_alias(
            CreateAliasRequest::builder()
                .collection_name(&first_collection)
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create alias");

    let description = client
        .describe_alias(
            DescribeAliasRequest::builder()
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe alias");
    assert_eq!(description.alias().to_owned(), alias);
    assert_eq!(description.collection_name().to_owned(), first_collection);

    let aliases = client
        .list_aliases(
            ListAliasesRequest::builder()
                .collection_name(&first_collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list aliases");
    assert!(aliases.aliases().contains(&alias));

    client
        .alter_alias(
            AlterAliasRequest::builder()
                .collection_name(&second_collection)
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("alter alias");
    let description = client
        .describe_alias(
            DescribeAliasRequest::builder()
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe altered alias");
    assert_eq!(description.collection_name().to_owned(), second_collection);

    client
        .drop_alias(
            DropAliasRequest::builder()
                .alias(&alias)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop alias");
    common::drop_collection(&client, &first_collection)
        .await
        .expect("drop first alias collection");
    common::drop_collection(&client, &second_collection)
        .await
        .expect("drop second alias collection");
}
