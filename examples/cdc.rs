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
