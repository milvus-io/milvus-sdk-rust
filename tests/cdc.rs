use milvus::cdc::ReplicateConfiguration;
use milvus::client::Client;
use milvus::error::Result;

mod common;
use common::*;

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
    let (checkpoint, salvage_checkpoint) =
        client.get_replicate_info("test-cluster", "test-pchannel").await?;
    let _ = checkpoint;
    let _ = salvage_checkpoint;
    Ok(())
}
