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

use milvus::v2::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const REPLICA_NUMBER: &str = "database.replica.number";

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".to_owned());
    let database = tutorial_database_name();

    let config = ConnectConfig::new().uri(uri).token(token);
    let client = ClientV2::new(&config).await?;

    print_databases(&client, "Databases before the tutorial").await?;

    println!("\nCreating database {database:?}");
    client
        .create_database(
            CreateDatabaseRequest::builder()
                .database_name(database.as_str())
                .build()?,
        )
        .await?;

    // Capture the tutorial result so cleanup still runs if a later operation fails.
    let tutorial_result = demonstrate_database_interfaces(&client, &database).await;
    let cleanup_result: Result<()> = async {
        client.use_database("default")?;
        println!("\nDropping tutorial database {database:?}");
        client
            .drop_database(
                DropDatabaseRequest::builder()
                    .database_name(database.as_str())
                    .build()?,
            )
            .await
    }
    .await;

    if let Err(error) = &cleanup_result {
        eprintln!("Failed to clean up {database:?}: {error}");
    }
    tutorial_result?;
    cleanup_result?;

    print_databases(&client, "Databases after cleanup").await?;
    Ok(())
}

async fn demonstrate_database_interfaces(client: &ClientV2, database: &str) -> Result<()> {
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(database)
                .build()?,
        )
        .await?;
    println!(
        "Created database: name={:?}, id={}, timestamp={}",
        description.database_name(),
        description.database_id(),
        description.created_timestamp()
    );

    println!("\nSetting {REPLICA_NUMBER}=1");
    client
        .alter_database_properties(
            AlterDatabasePropertiesRequest::builder()
                .database_name(database)
                .property(REPLICA_NUMBER, "1")
                .build()?,
        )
        .await?;

    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(database)
                .build()?,
        )
        .await?;
    println!(
        "Property value: {}",
        description
            .properties()
            .get(REPLICA_NUMBER)
            .map_or("<not set>", String::as_str)
    );

    println!("\nRemoving {REPLICA_NUMBER}");
    client
        .drop_database_properties(
            DropDatabasePropertiesRequest::builder()
                .database_name(database)
                .property_key(REPLICA_NUMBER)
                .build()?,
        )
        .await?;

    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(database)
                .build()?,
        )
        .await?;
    println!(
        "Property is present after removal: {}",
        description.properties().contains_key(REPLICA_NUMBER)
    );

    client.use_database(database)?;
    println!("\nSelected database: {}", client.current_database());
    println!("Subsequent requests with no database_name will use this selected database.");

    Ok(())
}

async fn print_databases(client: &ClientV2, heading: &str) -> Result<()> {
    let response = client
        .list_databases(ListDatabasesRequest::builder().build()?)
        .await?;
    println!("{heading}: {}", response.database_names().join(", "));
    Ok(())
}

fn tutorial_database_name() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("rust_sdk_tutorial_{timestamp}_{}", std::process::id())
}
