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

use milvus::client::{self, *};
use milvus::error::Result;
use milvus::index::IndexType;
use milvus::options::CreateCollectionOptions;
use milvus::proto::schema;
use milvus::{collection, schema::*};
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn load_release_partitions() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;
    client
        .create_partition(schema.name().to_string(), "partition_A".to_string())
        .await?;

    // Create an index on the vector field before loading partitions
    client
        .create_index(
            schema.name(),
            "feature",
            milvus::index::IndexParams::new(
                "id".to_string(),
                IndexType::IvfFlat,
                milvus::index::MetricType::L2,
                HashMap::new(),
            ),
        )
        .await?;

    client.release_collection(schema.name()).await?;

    client
        .load_partitions(schema.name(), vec!["partition_A"], 0, None)
        .await?;

    let mut status = client.get_load_state(schema.name(), None).await?;

    assert_eq!(status, milvus::proto::common::LoadState::Loaded);

    status = client
        .get_load_state(
            schema.name(),
            Some(milvus::options::GetLoadStateOptions::with_partition_names(
                vec!["partition_A".to_string()],
            )),
        )
        .await?;
    assert_eq!(status, milvus::proto::common::LoadState::Loaded);

    client.release_partitions(schema.name(), vec!["partition_A"]).await?;
    status = client.get_load_state(schema.name(), None).await?;
    assert_eq!(status,milvus::proto::common::LoadState::NotLoad);

    client.drop_collection(schema.name()).await?;
    Ok(())
}
