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

use milvus::cdc::{GetReplicateInfoResponse, ReplicateConfiguration};
use milvus::client::Client;
use milvus::error::Result;

use super::common::*;

#[tokio::test]
#[ignore = "replication configuration APIs require a replication-enabled Milvus deployment"]
async fn get_replicate_configuration() -> Result<()> {
    let client = Client::new(URL).await?;
    let _configuration: ReplicateConfiguration = client.get_replicate_configuration().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "replication configuration APIs require a replication-enabled Milvus deployment"]
async fn update_replicate_configuration() -> Result<()> {
    let client = Client::new(URL).await?;
    let configuration = client.get_replicate_configuration().await?;
    client
        .update_replicate_configuration(configuration, false)
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "replication metadata APIs require a replication-enabled Milvus deployment"]
async fn get_replicate_info() -> Result<()> {
    let client = Client::new(URL).await?;
    let _response: GetReplicateInfoResponse = client
        .get_replicate_info("test-cluster", "test-pchannel")
        .await?;
    Ok(())
}
