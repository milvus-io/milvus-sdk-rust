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

//! Requires replication-enabled Milvus deployments at ports 19530 and 29530.

use milvus::v2::error::Result;
use milvus::v2::prelude::*;

fn channels(cluster: &str) -> Vec<String> {
    (0..16)
        .map(|index| format!("{cluster}-rootcoord-dml_{index}"))
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    const URI_A: &str = "http://localhost:19530";
    const URI_B: &str = "http://localhost:29530";
    const CLUSTER_A: &str = "cdc-a";
    const CLUSTER_B: &str = "cdc-b";
    let client_a = ClientV2::new(&ConnectConfig::new().uri(URI_A)).await?;
    let client_b = ClientV2::new(&ConnectConfig::new().uri(URI_B)).await?;
    let version_request = GetServerVersionRequest::builder().build()?;
    println!(
        "Cluster A connected: {}",
        client_a
            .server_version(version_request.clone())
            .await?
            .version()
    );
    println!(
        "Cluster B connected: {}",
        client_b.server_version(version_request).await?.version()
    );

    let configuration = ReplicateConfiguration::new()
        .add_cluster(
            ReplicateCluster::new()
                .cluster_id(CLUSTER_A)
                .uri(URI_A)
                .physical_channels(channels(CLUSTER_A)),
        )
        .add_cluster(
            ReplicateCluster::new()
                .cluster_id(CLUSTER_B)
                .uri(URI_B)
                .physical_channels(channels(CLUSTER_B)),
        )
        .add_topology(
            CrossClusterTopology::new()
                .source_cluster_id(CLUSTER_A)
                .target_cluster_id(CLUSTER_B),
        );
    let update = UpdateReplicateConfigurationRequest::builder()
        .configuration(configuration)
        .build()?;
    client_a
        .update_replicate_configuration(update.clone())
        .await?;
    println!("Replicate configuration updated for cluster A");
    client_b.update_replicate_configuration(update).await?;
    println!("Replicate configuration updated for cluster B");

    let response = client_a
        .get_replicate_configuration(GetReplicateConfigurationRequest::builder().build()?)
        .await?;
    println!("Replicate configuration:");
    for cluster in response.configuration().get_clusters() {
        println!(
            "  clusterId={}, uri={}, pchannels={:?}",
            cluster.get_cluster_id(),
            cluster.get_uri(),
            cluster.get_physical_channels()
        );
    }
    for topology in response.configuration().get_topology() {
        println!(
            "  topology: sourceClusterId={}, targetClusterId={}",
            topology.get_source_cluster_id(),
            topology.get_target_cluster_id()
        );
    }

    let info = client_b
        .get_replicate_info(
            GetReplicateInfoRequest::builder()
                .source_cluster_id(CLUSTER_A)
                .target_physical_channel(&channels(CLUSTER_B)[0])
                .build()?,
        )
        .await?;
    let checkpoint = info.checkpoint();
    println!(
        "  checkpoint: clusterId={}, pchannel={}, messageId={}, walName={:?}, timeTick={}",
        checkpoint.get_cluster_id(),
        checkpoint.get_physical_channel(),
        checkpoint.get_message_id().get_id(),
        checkpoint.get_message_id().get_wal_name(),
        checkpoint.get_time_tick()
    );
    if let Some(salvage) = info.salvage_checkpoint() {
        println!(
            "  salvageCheckpoint: clusterId={}, pchannel={}, messageId={}, walName={:?}, timeTick={}",
            salvage.get_cluster_id(),
            salvage.get_physical_channel(),
            salvage.get_message_id().get_id(),
            salvage.get_message_id().get_wal_name(),
            salvage.get_time_tick()
        );
        println!("Dump messages:");
        client_a
            .dump_messages(
                DumpMessagesRequest::builder()
                    .physical_channel(salvage.get_physical_channel())
                    .start_message_id(salvage.get_message_id().clone())
                    .start_time_tick(salvage.get_time_tick())
                    .build()?,
                |message| {
                    println!("\tmessage id: {}", message.get_message_id().get_id());
                    println!("\tpayload size: {}", message.get_payload().len());
                    Ok(())
                },
            )
            .await?;
    } else {
        println!("  salvageCheckpoint: unavailable");
    }
    Ok(())
}
