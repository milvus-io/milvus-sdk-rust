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

use milvus::v2::request::collection::{
    CreateSimpleCollectionRequest, GetLoadStateRequest, ReleaseCollectionRequest,
};
use milvus::v2::request::partition::{
    CreatePartitionRequest, DropPartitionRequest, HasPartitionRequest, ListPartitionsRequest,
    LoadPartitionsRequest, ReleasePartitionsRequest,
};
use milvus::v2::LoadState;

use super::common;

#[tokio::test]
async fn partition_lifecycle() {
    let client = common::client().await;
    let collection = common::unique_collection_name("partition");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    let partition = common::unique_name("partition");
    client
        .create_collection(
            CreateSimpleCollectionRequest::builder()
                .collection_name(&collection)
                .dimension(4)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create partition collection");
    client
        .release_collection(
            ReleaseCollectionRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("release partition collection");

    client
        .create_partition(
            CreatePartitionRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create partition");

    let has_partition = client
        .has_partition(
            HasPartitionRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("check partition");
    assert!(has_partition.exists());

    let partitions = client
        .list_partitions(
            ListPartitionsRequest::builder()
                .collection_name(&collection)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("list partitions");
    assert!(partitions.partition_names().contains(&partition));
    assert!(partitions
        .partitions()
        .iter()
        .any(|info| info.get_name() == &partition));

    client
        .load_partitions(
            LoadPartitionsRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .sync(true)
                .timeout_ms(60_000)
                .load_fields(["id", "vector"])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load partition");
    let state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get loaded partition state");
    assert_eq!(state.state().to_owned(), LoadState::Loaded);
    assert_eq!(state.progress().to_owned(), 100);

    client
        .release_partitions(
            ReleasePartitionsRequest::builder()
                .collection_name(&collection)
                .partition_names([&partition])
                .build()
                .expect("valid request"),
        )
        .await
        .expect("release partition");
    let state = client
        .get_load_state(
            GetLoadStateRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("get released partition state");
    assert_eq!(state.state().to_owned(), LoadState::NotLoad);
    assert_eq!(state.progress().to_owned(), 0);

    client
        .drop_partition(
            DropPartitionRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("drop partition");
    let has_partition = client
        .has_partition(
            HasPartitionRequest::builder()
                .collection_name(&collection)
                .partition_name(&partition)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("check dropped partition");
    assert!(!has_partition.exists());

    common::drop_collection(&client, &collection)
        .await
        .expect("drop partition collection");
}
