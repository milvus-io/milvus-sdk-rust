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
  common::{ConsistencyLevel, ErrorCode, KeyValuePair},
  milvus::{
    milvus_service_client::MilvusServiceClient, CreateCollectionRequest, DropCollectionRequest,
    HasCollectionRequest,
  },
  schema::{CollectionSchema, DataType, FieldSchema},
};
use anyhow::{bail, Result};
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

  pub async fn create_collection(&mut self, schema: CollectionDef, shards_num: i32) -> Result<()> {
    let schema = CollectionSchema::from(schema);

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

    println!("CREATE COLLECTION RESPONSE={:?}", response);

    Ok(())
  }

  pub async fn drop_collection(&mut self, name: &str) -> Result<()> {
    let request = Request::new(DropCollectionRequest {
      base: None,
      db_name: String::new(),
      collection_name: name.to_owned(),
    });

    let response = self.client.drop_collection(request).await?;

    println!("DROP COLLECTION RESPONSE={:?}", response);

    Ok(())
  }

  pub async fn has_collection(&mut self, name: &str) -> Result<bool> {
    let request = Request::new(HasCollectionRequest {
      base: None,
      db_name: String::new(),
      collection_name: name.to_owned(),
      time_stamp: 0,
    });

    let response = self.client.has_collection(request).await?.into_inner();

    if let Some(status) = response.status {
      if status.error_code != ErrorCode::Success as i32 {
        bail!(status.reason);
      }
    }

    Ok(response.value)
  }
}

struct FieldDef {
  name: String,
  field_type: FieldType,
  data_type: i32,
  pub description: Option<String>,
}

enum FieldType {
  // PrimaryKey(auto_id)
  PrimaryKey(bool),
  Bool,
  Int64,
  Float,
  Double,
  // BinaryVector(dim)
  BinaryVector(i16),
  // FloatVector(dim)
  FloatVector(i16),
}

impl FieldDef {
  pub fn primary_key_field(name: impl AsRef<str>, auto_id: bool) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::PrimaryKey(auto_id),
      data_type: DataType::Int64 as i32,
      description: None,
    }
  }

  pub fn bool_field(name: impl AsRef<str>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::Bool,
      data_type: DataType::Bool as i32,
      description: None,
    }
  }

  pub fn int_64_field(name: impl AsRef<str>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::Int64,
      data_type: DataType::Int64 as i32,
      description: None,
    }
  }

  pub fn float_field(name: impl AsRef<str>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::Float,
      data_type: DataType::Float as i32,
      description: None,
    }
  }

  pub fn double_field(name: impl AsRef<str>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::Double,
      data_type: DataType::Double as i32,
      description: None,
    }
  }

  pub fn binary_vector_field(name: impl AsRef<str>, dim: i16) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::BinaryVector(dim),
      data_type: DataType::BinaryVector as i32,
      description: None,
    }
  }

  pub fn float_vector_field(name: impl AsRef<str>, dim: i16) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      field_type: FieldType::FloatVector(dim),
      data_type: DataType::FloatVector as i32,
      description: None,
    }
  }
}

impl From<FieldDef> for FieldSchema {
  fn from(fd: FieldDef) -> Self {
    let type_params = match fd.field_type {
      FieldType::BinaryVector(dim) => vec![KeyValuePair {
        key: "dim".to_string(),
        value: dim.to_string(),
      }],
      FieldType::FloatVector(dim) => vec![KeyValuePair {
        key: "dim".to_string(),
        value: dim.to_string(),
      }],
      _ => Vec::new(),
    };

    let auto_id = match fd.field_type {
      FieldType::PrimaryKey(auto_id) => auto_id,
      _ => false,
    };

    Self {
      field_id: 0,
      name: fd.name,
      is_primary_key: matches!(fd.field_type, FieldType::PrimaryKey(_)),
      description: fd.description.unwrap_or(String::new()),
      data_type: fd.data_type,
      type_params,
      index_params: Vec::new(),
      auto_id,
    }
  }
}

pub struct CollectionDef {
  name: String,
  description: String,
  auto_id: bool,
  fields: Vec<FieldDef>,
}

impl From<CollectionDef> for CollectionSchema {
  fn from(cs: CollectionDef) -> Self {
    Self {
      name: cs.name,
      description: cs.description,
      auto_id: cs.auto_id,
      fields: cs.fields.into_iter().map(FieldSchema::from).collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::Client;
  use super::CollectionDef;
  use super::FieldDef;

  #[tokio::test]
  async fn create_collection() -> Result<()> {
    let mut client = Client::new(None).await?;

    if client.has_collection("new_schema").await? {
      client.drop_collection("new_schema").await?;
    }

    let schema = CollectionDef {
      name: "new_schema".to_owned(),
      description: "description".to_owned(),
      auto_id: false,
      fields: vec![FieldDef::primary_key_field("book_id", false)],
    };

    client.create_collection(schema, 2).await?;

    Ok(())
  }
}
