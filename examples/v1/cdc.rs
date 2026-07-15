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

use milvus::cdc::{CrossClusterTopology, MilvusCluster, ReplicateConfiguration};
use milvus::client::Client;
use milvus::error::Result;

const CLUSTER_A_URI: &str = "http://192.168.1.1:19530";
const CLUSTER_B_URI: &str = "http://192.168.1.1:19500";
const CLUSTER_A_ID: &str = "cdc-test-upstream";
const CLUSTER_B_ID: &str = "cdc-test-downstream";
const PCHANNEL_NUM: usize = 16;

fn generate_pchannels(cluster_id: &str) -> Vec<String> {
    (0..PCHANNEL_NUM)
        .map(|i| format!("{cluster_id}-rootcoord-dml_{i}"))
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let cluster_a_client = Client::new(CLUSTER_A_URI).await?;
    let cluster_b_client = Client::new(CLUSTER_B_URI).await?;

    let cluster_a = MilvusCluster {
        cluster_id: CLUSTER_A_ID.to_string(),
        uri: CLUSTER_A_URI.to_string(),
        token: String::new(),
        pchannels: generate_pchannels(CLUSTER_A_ID),
    };

    let cluster_b = MilvusCluster {
        cluster_id: CLUSTER_B_ID.to_string(),
        uri: CLUSTER_B_URI.to_string(),
        token: String::new(),
        pchannels: generate_pchannels(CLUSTER_B_ID),
    };

    let topology = CrossClusterTopology {
        source_cluster_id: CLUSTER_A_ID.to_string(),
        target_cluster_id: CLUSTER_B_ID.to_string(),
    };

    let configuration = ReplicateConfiguration {
        clusters: vec![cluster_a, cluster_b],
        cross_cluster_topologies: vec![topology],
    };

    cluster_a_client
        .update_replicate_configuration(configuration.clone(), false)
        .await?;
    cluster_b_client
        .update_replicate_configuration(configuration, false)
        .await?;

    println!("CDC replication configuration updated on both clusters.");
    Ok(())
}
