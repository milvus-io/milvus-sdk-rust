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
    // ClientV2::new connects to `uri` and authenticates database administration with `token`.
    println!("Calling ClientV2::new: connect to Milvus");
    let client = ClientV2::new(&config).await?;
    println!("ClientV2::new completed");

    print_databases(&client, "Databases before the tutorial").await?;

    println!("\nCreating database {database:?}");
    // create_database creates a logical database. `database_name` is the unique name used by later
    // requests and by `use_database`.
    println!("Calling create_database: create {database:?}");
    client
        .create_database(
            CreateDatabaseRequest::builder()
                .database_name(database.as_str())
                .build()?,
        )
        .await?;
    println!("create_database completed");

    // Capture the tutorial result so cleanup still runs if a later operation fails.
    let tutorial_result = demonstrate_database_interfaces(&client, &database).await;
    let cleanup_result: Result<()> = async {
        // use_database changes the database selected by this client. Returning to `default` ensures
        // the database being deleted is not still selected.
        println!("Calling use_database: select default");
        client.use_database("default")?;
        println!("use_database completed");
        println!("\nDropping tutorial database {database:?}");
        // drop_database permanently removes the named database; it must contain no collections.
        println!("Calling drop_database: remove {database:?}");
        client
            .drop_database(
                DropDatabaseRequest::builder()
                    .database_name(database.as_str())
                    .build()?,
            )
            .await?;
        println!("drop_database completed");
        Ok(())
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
    // describe_database returns metadata and properties for `database_name`.
    println!("Calling describe_database: inspect {database:?}");
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(database)
                .build()?,
        )
        .await?;
    println!("describe_database completed");
    println!(
        "Created database: name={:?}, id={}, timestamp={}",
        description.database_name(),
        description.database_id(),
        description.created_timestamp()
    );

    println!("\nSetting {REPLICA_NUMBER}=1");
    // alter_database_properties adds or replaces database property key/value pairs. This setting
    // configures the default replica count for collections in the database.
    println!("Calling alter_database_properties: set {REPLICA_NUMBER}=1");
    client
        .alter_database_properties(
            AlterDatabasePropertiesRequest::builder()
                .database_name(database)
                .property(REPLICA_NUMBER, "1")
                .build()?,
        )
        .await?;
    println!("alter_database_properties completed");

    // describe_database is called again to read back the updated property map.
    println!("Calling describe_database: read the updated property");
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(database)
                .build()?,
        )
        .await?;
    println!("describe_database completed");
    println!(
        "Property value: {}",
        description
            .properties()
            .get(REPLICA_NUMBER)
            .map_or("<not set>", String::as_str)
    );

    println!("\nRemoving {REPLICA_NUMBER}");
    // drop_database_properties removes the specified key while preserving other properties.
    println!("Calling drop_database_properties: remove {REPLICA_NUMBER}");
    client
        .drop_database_properties(
            DropDatabasePropertiesRequest::builder()
                .database_name(database)
                .property_key(REPLICA_NUMBER)
                .build()?,
        )
        .await?;
    println!("drop_database_properties completed");

    // describe_database verifies that the property is no longer present.
    println!("Calling describe_database: verify property removal");
    let description = client
        .describe_database(
            DescribeDatabaseRequest::builder()
                .database_name(database)
                .build()?,
        )
        .await?;
    println!("describe_database completed");
    println!(
        "Property is present after removal: {}",
        description.properties().contains_key(REPLICA_NUMBER)
    );

    // use_database selects this database for requests that omit an explicit database name.
    println!("Calling use_database: select {database:?}");
    client.use_database(database)?;
    println!("use_database completed");
    println!("\nSelected database: {}", client.current_database());
    println!("Subsequent requests with no database_name will use this selected database.");

    Ok(())
}

async fn print_databases(client: &ClientV2, heading: &str) -> Result<()> {
    // list_databases returns every database name visible to the authenticated user.
    println!("Calling list_databases: list visible databases");
    let response = client
        .list_databases(ListDatabasesRequest::builder().build()?)
        .await?;
    println!("list_databases completed");
    println!("{heading}: {}", response.database_names().join(", "));
    Ok(())
}

fn tutorial_database_name() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("RUST_V2_DATABASE_{timestamp}_{}", std::process::id())
}
