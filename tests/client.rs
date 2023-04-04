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
use milvus::options::CreateCollectionOptions;
use milvus::schema::*;

#[tokio::test]
async fn create_client() -> Result<()> {
    const URL: &str = "http://localhost:19530";
    match Client::new(URL).await {
        Ok(_) => return Result::<()>::Ok(()),
        Err(e) => panic!("Error is {}.", e),
    }
}

#[tokio::test]
async fn create_client_wrong_url() -> Result<()> {
    const URL: &str = "http://localhost:9999";
    match Client::new(URL).await {
        Ok(_) => panic!("Should fail due to wrong url."),
        Err(_) => return Result::<()>::Ok(()),
    }
}

#[tokio::test]
async fn create_client_wrong_fmt() -> Result<()> {
    const URL: &str = "9999";
    match Client::new(URL).await {
        Ok(_) => panic!("Should fail due to wrong format url."),
        Err(_) => return Result::<()>::Ok(()),
    }
}

#[tokio::test]
async fn has_collection() -> Result<()> {
    const URL: &str = "http://localhost:19530";
    const NAME: &str = "qwerty";
    let client = Client::new(URL).await?;
    match client.has_collection(NAME).await {
        Ok(has) => {
            if has {
                panic!("Expect no such collection.");
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

#[tokio::test]
async fn create_has_drop_collection() -> Result<()> {
    const URL: &str = "http://localhost:19530";
    const NAME: &str = "create_has_drop_collection";

    let client = Client::new(URL).await?;
    // let client = ClientBuilder::new(URL).username("username").password("password").build().await?;

    let mut schema = CollectionSchemaBuilder::new(NAME, "hello world");
    let schema = schema
        .add_field(FieldSchema::new_int64("i64_field", ""))
        .add_field(FieldSchema::new_bool("bool_field", ""))
        .set_primary_key("i64_field")?
        .enable_auto_id()?
        .build()?;

    if client.has_collection(NAME).await? {
        client.drop_collection(NAME).await?;
    }

    let collection = client
        .create_collection(
            schema,
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Session,
            )),
        )
        .await?;

    assert!(collection.exist().await?);

    client.drop_collection(NAME).await?;
    assert!(collection.exist().await? == false);

    Ok(())
}
