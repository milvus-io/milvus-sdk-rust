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

use crate::error;
use crate::error::Result;
use crate::proto::common::KeyValuePair;
use crate::proto::schema;
use crate::proto::schema::DataType;
use prost::alloc::vec::Vec;
use std::error::Error as _;
use thiserror::Error as ThisError;

pub struct FieldSchema {
    name: String,
    description: String,
    dtype: DataType,
    is_primary: bool,
    auto_id: bool,
    dim: i32,        // only for BinaryVector and FloatVector
    max_length: i32, // only for VarChar
}

impl Default for FieldSchema {
    fn default() -> Self {
        Self {
            name: "Field".to_string(),
            description: "".to_string(),
            dtype: DataType::None,
            is_primary: false,
            auto_id: false,
            dim: 0,
            max_length: 0,
        }
    }
}

impl FieldSchema {
    pub fn new_bool<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Bool,
            ..Default::default()
        }
    }

    pub fn new_int8<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Int8,
            ..Default::default()
        }
    }

    pub fn new_int16<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Int16,
            ..Default::default()
        }
    }

    pub fn new_int32<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Int32,
            ..Default::default()
        }
    }

    pub fn new_int64<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Int64,
            ..Default::default()
        }
    }

    pub fn new_float<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Float,
            ..Default::default()
        }
    }

    pub fn new_double<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::Double,
            ..Default::default()
        }
    }

    pub fn new_string<S>(name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::String,
            ..Default::default()
        }
    }

    pub fn new_varchar<S>(name: S, description: S, max_length: i32) -> Self
    where
        S: Into<String>,
    {
        assert!(max_length > 0, "max_length should be positive");
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::String,
            max_length: max_length,
            ..Default::default()
        }
    }

    pub fn new_binary_vector<S>(name: S, description: S, dim: i32) -> Self
    where
        S: Into<String>,
    {
        assert!(dim > 0, "dim should be positive");
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::BinaryVector,
            dim: dim,
            ..Default::default()
        }
    }

    pub fn new_float_vector<S>(name: S, description: S, dim: i32) -> Self
    where
        S: Into<String>,
    {
        assert!(dim > 0, "dim should be positive");
        Self {
            name: name.into(),
            description: description.into(),
            dtype: DataType::FloatVector,
            dim: dim,
            ..Default::default()
        }
    }
    pub fn convert_field(self) -> schema::FieldSchema {
        let tp = match self.dtype {
            DataType::BinaryVector | DataType::FloatVector => vec![KeyValuePair {
                key: "dim".to_string(),
                value: self.dim.to_string(),
            }],
            DataType::VarChar => vec![KeyValuePair {
                key: "max_length".to_string(),
                value: self.max_length.to_string(),
            }],
            _ => Vec::new(),
        };
        schema::FieldSchema {
            field_id: 0,
            name: self.name,
            is_primary_key: self.is_primary,
            description: self.description,
            data_type: self.dtype as i32,
            type_params: tp,
            index_params: Vec::new(),
            auto_id: self.auto_id,
        }
    }
}

pub struct CollectionSchema {
    inner: Vec<FieldSchema>,
    auto_id: bool,
}

impl CollectionSchema {
    pub fn unpack(self) -> (Vec<FieldSchema>, bool) {
        (self.inner, self.auto_id)
    }
}

pub struct CollectionSchemaBuilder {
    inner: Vec<FieldSchema>,
}

impl CollectionSchemaBuilder {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add(&mut self, schema: FieldSchema) {
        self.inner.push(schema);
    }

    pub fn set_primary_key<S>(&mut self, name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let n = name.into();
        for f in self.inner.iter_mut() {
            if f.is_primary {
                return Err(error::Error::from(Error::DuplicatePrimaryKey(
                    n,
                    f.name.to_owned(),
                )));
            }
        }
        for f in self.inner.iter_mut() {
            if n == f.name {
                if f.dtype == DataType::Int64 || f.dtype == DataType::VarChar {
                    f.is_primary = true;
                    return Ok(());
                } else {
                    return Err(error::Error::from(Error::UnsupportedPrimaryKey(
                        f.dtype.to_owned(),
                    )));
                }
            }
        }
        Err(error::Error::from(Error::NoSuchKey(n)))
    }

    pub fn enable_auto_id(&mut self) -> Result<()> {
        for f in self.inner.iter_mut() {
            if f.is_primary {
                if f.dtype == DataType::Int64 {
                    f.auto_id = true;
                    return Ok(());
                } else {
                    return Err(error::Error::from(Error::UnsupportedAutoId(
                        f.dtype.to_owned(),
                    )));
                }
            }
        }
        Err(error::Error::from(Error::NoPrimaryKey()))
    }

    pub fn build(self) -> Result<CollectionSchema> {
        let mut has_primary = false;
        let mut auto = false;
        for f in self.inner.iter() {
            if f.is_primary {
                has_primary = true;
                if f.auto_id {
                    auto = true;
                }
                break;
            }
        }
        if !has_primary {
            return Err(error::Error::from(Error::NoPrimaryKey()));
        }
        Ok(CollectionSchema {
            inner: self.inner,
            auto_id: auto,
        })
    }
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("try to set primary key {0:?}, but {1:?} is also key")]
    DuplicatePrimaryKey(String, String),

    #[error("can not find any primary key")]
    NoPrimaryKey(),

    #[error("primary key must be int64 or varchar, unsupported type {0:?}")]
    UnsupportedPrimaryKey(DataType),

    #[error("auto id must be int64, unsupported type {0:?}")]
    UnsupportedAutoId(DataType),

    #[error("can not find such key {0:?}")]
    NoSuchKey(String),
}
