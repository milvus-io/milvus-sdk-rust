use std::collections::HashMap;

use milvus::client::Client;
use milvus::error::Result;
use milvus::resource_group::{CreateRgOptions, UpdateRgOptions};

const RG_NAME: &str = "test_rg";

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;

    //create resource group
    println!("================Create Resource Group==================");
    let res = client.list_resource_groups().await?;
    //drop rg if exists
    if res.iter().find(|&s| s.eq(RG_NAME)).is_some() {
        let options = UpdateRgOptions::new().limits(0).requests(0);
        let configs: HashMap<String, CreateRgOptions> =
            HashMap::from([(RG_NAME.to_string(), options)]);
        client.update_resource_groups(configs).await?;
        client.drop_resource_group(RG_NAME).await?;
    }
    println!("Before creating any resource groups: {:?}", res);
    let options = CreateRgOptions::new().limits(10).requests(2);
    client.create_resource_group(RG_NAME, Some(options)).await?;
    let res = client.list_resource_groups().await?;
    println!("After creating a resource group: {:?}", res);

    //describe resource group
    println!("================Describe Resource Group================");
    if let Some(res) = client.describe_resource_group(RG_NAME).await? {
        println!("Resource {RG_NAME}:\n {:#?}", res);
    } else {
        println!("Resource {RG_NAME} doesn't exist!");
    }

    // update resoure group
    println!("================Update Resource Group==================");
    let options = UpdateRgOptions::new().limits(0).requests(0);
    let configs: HashMap<String, CreateRgOptions> = HashMap::from([(RG_NAME.to_string(), options)]);
    client.update_resource_groups(configs).await?;
    if let Some(res) = client.describe_resource_group(RG_NAME).await? {
        println!("Resource {RG_NAME}:\n {:#?}", res);
    } else {
        println!("Resource {RG_NAME} doesn't exist!");
    }

    //drop resource group
    println!("===============Drop Resource Group====================");
    client.drop_resource_group(RG_NAME).await?;

    let res = client.list_resource_groups().await?;
    println!("After drop resource groups:{:?}", res);
    Ok(())
}
