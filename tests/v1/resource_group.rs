use milvus::client::Client;
use milvus::error::Result;
use milvus::resource_group::{CreateRgOptions, UpdateRgOptions};
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn resource_group_lifecycle() -> Result<()> {
    let client = Client::new(URL).await?;
    let rg_name = format!("test_rg_{}", gen_random_name());

    let test_result: Result<()> = async {
        let options = CreateRgOptions::new().limits(10).requests(0);
        client
            .create_resource_group(&rg_name, Some(options))
            .await?;

        let groups = client.list_resource_groups().await?;
        assert!(groups.contains(&rg_name));

        let described = client.describe_resource_group(&rg_name).await?;
        assert!(described.is_some());

        let options = UpdateRgOptions::new().limits(0).requests(0);
        let configs = HashMap::from([(rg_name.clone(), options)]);
        client.update_resource_groups(configs).await?;

        Ok(())
    }
    .await;

    let cleanup_result = client.drop_resource_group(&rg_name).await;

    test_result?;
    cleanup_result?;

    let groups = client.list_resource_groups().await?;
    assert!(!groups.contains(&rg_name));

    Ok(())
}

#[tokio::test]
#[ignore = "resource group transfer needs cluster resources that standalone Milvus may not provide"]
async fn resource_group_transfer_operations() -> Result<()> {
    let client = Client::new(URL).await?;
    let source = format!("test_rg_source_{}", gen_random_name());
    let target = format!("test_rg_target_{}", gen_random_name());
    let collection = format!("test_rg_collection_{}", gen_random_name());

    let test_result: Result<()> = async {
        client
            .create_resource_group(&source, Some(CreateRgOptions::new().limits(1).requests(0)))
            .await?;
        client
            .create_resource_group(&target, Some(CreateRgOptions::new().limits(1).requests(0)))
            .await?;

        client.transfer_node(&source, &target, 0).await?;
        client
            .transfer_replica(&source, &target, &collection, 0)
            .await?;

        let configs = HashMap::from([
            (source.clone(), UpdateRgOptions::new().limits(0).requests(0)),
            (target.clone(), UpdateRgOptions::new().limits(0).requests(0)),
        ]);
        client.update_resource_groups(configs).await?;

        Ok(())
    }
    .await;

    let cleanup_source = client.drop_resource_group(&source).await;
    let cleanup_target = client.drop_resource_group(&target).await;

    test_result?;
    cleanup_source?;
    cleanup_target?;

    Ok(())
}
