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

use milvus::v2::request::index::{
    AlterIndexPropertiesRequest, CreateIndexRequest, DescribeIndexRequest,
    DropIndexPropertiesRequest, DropIndexRequest, ListIndexesRequest,
};
use milvus::v2::{IndexParam, IndexStateCode, IndexType, MetricType};

use super::common;

#[tokio::test]
async fn index_lifecycle() {
    let client = common::client().await;
    let collection = common::unique_collection_name("index");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    let index_name = common::unique_name("vector_index");
    common::create_advanced_collection(&client, &collection).await;

    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(&collection)
                .index_param(
                    IndexParam::new()
                        .field_name(common::VECTOR_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::L2)
                        .index_name(&index_name),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create index");

    let description = client
        .describe_index(
            DescribeIndexRequest::builder()
                .collection_name(&collection)
                .field_name(common::VECTOR_FIELD)
                .index_name(&index_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe index");
    let index = description
        .indexes()
        .iter()
        .find(|index| index.get_index_name() == &index_name)
        .expect("created index description");
    assert_eq!(index.get_field_name().to_owned(), common::VECTOR_FIELD);
    assert_eq!(index.get_metric_type().to_owned(), MetricType::L2);
    assert_eq!(index.get_state().to_owned(), IndexStateCode::Finished);

    client
        .alter_index_properties(
            AlterIndexPropertiesRequest::builder()
                .collection_name(&collection)
                .index_name(&index_name)
                .property("mmap.enabled", "true")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("alter index properties");
    let description = client
        .describe_index(
            DescribeIndexRequest::builder()
                .collection_name(&collection)
                .field_name(common::VECTOR_FIELD)
                .index_name(&index_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe index after altering properties");
    let index = description
        .indexes()
        .iter()
        .find(|index| index.get_index_name() == &index_name)
        .expect("altered index description");
    assert_eq!(
        index.get_extra_params().get("mmap.enabled"),
        Some(&"true".to_owned())
    );

    client
        .drop_index_properties(
            DropIndexPropertiesRequest::builder()
                .collection_name(&collection)
                .index_name(&index_name)
                .property_key("mmap.enabled")
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop index properties");
    let description = client
        .describe_index(
            DescribeIndexRequest::builder()
                .collection_name(&collection)
                .field_name(common::VECTOR_FIELD)
                .index_name(&index_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("describe index after dropping properties");
    let index = description
        .indexes()
        .iter()
        .find(|index| index.get_index_name() == &index_name)
        .expect("index description after dropping properties");
    assert!(!index.get_extra_params().contains_key("mmap.enabled"));

    let indexes = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list indexes");
    assert!(indexes.index_names().contains(&index_name));

    client
        .drop_index(
            DropIndexRequest::builder()
                .collection_name(&collection)
                .field_name(common::VECTOR_FIELD)
                .index_name(&index_name)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop index");
    common::drop_collection(&client, &collection)
        .await
        .expect("drop index collection");
}
