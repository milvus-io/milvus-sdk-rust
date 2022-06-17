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

use crate::proto::{
  common::ConsistencyLevel,
  milvus::{milvus_service_client::MilvusServiceClient, CreateCollectionRequest},
  schema::CollectionSchema,
};
use anyhow::Result;
use prost::{bytes::BytesMut, Message};
use tonic::{transport::Channel, Request};

const DEFAULT_DST: &'static str = "http://[::1]:19530";

pub struct Client {
  client: MilvusServiceClient<Channel>,
}

impl Client {
  pub async fn new(dst: Option<&str>) -> Result<Self> {
    let dst = dst.unwrap_or(DEFAULT_DST).to_owned();
    let client = MilvusServiceClient::connect(dst).await?;
    Ok(Self { client })
  }

  pub async fn create_collection(
    &mut self,
    schema: CollectionSchema,
    shards_num: i32,
  ) -> Result<()> {
    let mut buf = BytesMut::new();
    schema.encode(&mut buf)?;
    let buf = buf.freeze();

    let request = Request::new(CreateCollectionRequest {
      base: None,
      db_name: String::new(),
      collection_name: schema.name.clone(),
      schema: buf.to_vec(),
      shards_num,
      consistency_level: ConsistencyLevel::Session as i32,
    });

    let response = self.client.create_collection(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::Client;

  #[tokio::test]
  async fn create_collection() -> Result<()> {
    let mut client = Client::new(None).await?;

    Ok(())
  }
}
