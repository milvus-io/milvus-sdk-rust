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
use milvus::schema::{self, FieldSchema};
use milvus::value::Value;
use std::thread;
use std::time::Duration;

// #[tokio::test]
// async fn main() -> Result<()> {
//     const URL: &str = "http://localhost:19530";

//     let client = Client::new(URL).await?;

//     let schema =
//         CollectionSchemaBuilder::new("hello_milvus", "a guide example for milvus rust SDK")
//             .add_field(FieldSchema::new_primary_int64("id", "", true))
//             .add_field(FieldSchema::new_float_vector("embed", "", 256))
//             .build()?;
//     let collection = client
//         .create_collection(schema.clone(), 2, common::ConsistencyLevel::Eventually)
//         .await?;

//     if let Err(err) = hello_milvus(&collection).await {
//         println!("failed to run hello milvus: {:?}", err);
//     }
//     collection.drop().await?;

//     Ok(())
// }

// async fn hello_milvus(collection: &Collection) -> Result<()> {
//     let mut embed_data = Vec::<f32>::new();
//     for _ in 1..=256 * 1000 {
//         let mut rng = rand::thread_rng();
//         let embed = rng.gen();
//         embed_data.push(embed);
//     }
//     let embed_column =
//         FieldColumn::new(collection.schema().get_field("embed").unwrap(), embed_data);

//     collection.insert(vec![embed_column], None).await?;
//     collection.flush().await?;
//     collection.load_blocked(1).await?;

//     let result = collection.query::<_, [&str; 0]>("id > 0", []).await?;

//     println!(
//         "result num: {}",
//         result.first().map(|c| c.len()).unwrap_or(0),
//     );

//     Ok(())
// }
