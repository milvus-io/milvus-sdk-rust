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

struct Test {
    i64_1: i64,
    bl: bool,
}

// #[tokio::test]
// #[ignore]
// async fn load_release_collection() -> Result<()> {
//     const URL: &str = "http://localhost:19530";

//     // create collection
//     let client = Client::new(URL).await?;
//     let test = client.get_collection::<Test>().await?;

//     if !test.exists().await? {
//         test.create(None, None).await?;
//     }

//     println!("collection prepared.");

//     // load with `load_unblocked` and release
//     test.load_unblocked(1).await?;
//     loop {
//         if test.is_load().await? {
//             println!("#");
//             break;
//         }
//         thread::sleep(Duration::from_millis(1000));
//     }

//     test.release().await?;

//     // load with `load_blocked` and release
//     test.load_blocked(1).await?;
//     test.release().await?;

//     // clean data
//     match test.drop().await {
//         Ok(()) => Ok(()),
//         Err(e) => Err(e),
//     }
// }
