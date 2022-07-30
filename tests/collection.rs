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

use milvus::client::*;
use milvus::error::Result;
use milvus::schema::*;
use std::thread;
use std::time::Duration;

#[test]
fn build_collection() -> Result<()> {
    let mut builder = CollectionSchemaBuilder::new();
    builder
        .add_field(FieldSchema::new_int64("i64_1", ""))
        .add_field(FieldSchema::new_bool("bl", ""))
        .set_primary_key("i64_1")?
        .enable_auto_id()?
        .build()?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn load_release_collection() -> Result<()> {
    const URL: &str = "http://localhost:19530";
    const NAME: &str = "tttest";

    // create collection
    let client = Client::new(URL).await?;
    let collection = match client.get_collection(NAME).await? {
        Some(c) => c,
        None => {
            let schema = CollectionSchemaBuilder::new()
                .add_field(FieldSchema::new_int64("i64_1", ""))
                .add_field(FieldSchema::new_bool("bl", ""))
                .set_primary_key("i64_1")?
                .enable_auto_id()?
                .build()?;
            client
                .create_collection(NAME, "tt", schema, 1, ConsistencyLevel::Session)
                .await?
        }
    };

    println!("collection prepared.");

    // load with `load_unblocked` and release
    collection.load_unblocked(1).await?;
    loop {
        if collection.is_load().await? {
            println!("#");
            break;
        }
        thread::sleep(Duration::from_millis(1000));
    }
    collection.release().await?;

    // load with `load_blocked` and release
    collection.load_blocked(1).await?;
    collection.release().await?;

    // clean data
    match client.drop_collection(NAME).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}
