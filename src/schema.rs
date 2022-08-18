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

use crate::error::Result;
use crate::value::Value;
use crate::{data::FieldColumn, error};
use prost::alloc::vec::Vec;
use std::borrow::Cow;
use thiserror::Error as ThisError;

use crate::proto::{
    common::KeyValuePair,
    schema::{self, DataType},
};

pub use crate::proto::schema::FieldData;

pub trait Entity {
    const NAME: &'static str;
    const DESCRIPTION: Option<&'static str> = None;
    const SCHEMA: &'static [FieldSchema<'static>];

    fn schema() -> CollectionSchema<'static> {
        CollectionSchema {
            name: Cow::Borrowed(Self::NAME),
            description: Self::DESCRIPTION.map(Cow::Borrowed),
            fields: Cow::Borrowed(Self::SCHEMA),
        }
    }

    type ColumnIntoIter: Iterator<Item = (&'static FieldSchema<'static>, Value<'static>)>;
    // type ColumnIter<'a>: Iterator<Item = (&'static FieldSchema<'static>, Value<'a>)>;

    fn iter(&self) -> Self::ColumnIntoIter; // Self::ColumnIter<'_>
    fn into_iter(self) -> Self::ColumnIntoIter;

    fn validate(&self) -> std::result::Result<(), Error> {
        for (schm, val) in self.iter() {
            let dtype = val.data_type();

            if dtype != schm.dtype
                && !(dtype == DataType::String && schm.dtype == DataType::VarChar)
            {
                return Err(Error::FieldWrongType(
                    schm.name.to_string(),
                    schm.dtype,
                    val.data_type(),
                ));
            }

            match schm.dtype {
                DataType::VarChar => match &val {
                    Value::String(d) if d.len() > schm.max_length as _ => {
                        return Err(Error::DimensionMismatch(
                            schm.name.to_string(),
                            schm.max_length as _,
                            d.len() as _,
                        ));
                    }
                    _ => unreachable!(),
                },
                DataType::BinaryVector => match &val {
                    Value::Binary(d) => {
                        return Err(Error::DimensionMismatch(
                            schm.name.to_string(),
                            schm.dim as _,
                            d.len() as _,
                        ));
                    }
                    _ => unreachable!(),
                },
                DataType::FloatVector => match &val {
                    Value::FloatArray(d) => {
                        return Err(Error::DimensionMismatch(
                            schm.name.to_string(),
                            schm.dim as _,
                            d.len() as _,
                        ));
                    }
                    _ => unreachable!(),
                },
                _ => (),
            }
        }

        Ok(())
    }
}

pub trait IntoDataFields {
    fn into_data_fields(self) -> Vec<FieldData>;
}

pub trait FromDataFields: Sized {
    fn from_data_fields(fileds: Vec<FieldData>) -> Option<Self>;
}

pub trait Collection<'a>: IntoDataFields + FromDataFields {
    type Entity: Entity;
    type IterRows: Iterator<Item = Self::Entity> + 'a;
    type IterColumns: Iterator<Item = FieldColumn<'static>> + 'a;

    fn index(&self, idx: usize) -> Option<Self::Entity>;
    fn with_capacity(cap: usize) -> Self;
    fn add(&mut self, entity: Self::Entity);
    fn len(&self) -> usize;
    fn iter_columns(&'a self) -> Self::IterColumns;

    fn iter_rows(&self) -> Box<dyn Iterator<Item = Self::Entity> + '_> {
        Box::new((0..self.len()).filter_map(|idx| self.index(idx)))
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn columns() -> &'static [FieldSchema<'static>] {
        Self::Entity::SCHEMA
    }
}

#[derive(Debug, Clone)]
pub struct FieldSchema<'a> {
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub dtype: DataType,
    pub is_primary: bool,
    pub auto_id: bool,
    pub chunk_size: usize,
    pub dim: i64,        // only for BinaryVector and FloatVector
    pub max_length: i32, // only for VarChar
}

impl FieldSchema<'static> {
    pub const fn const_default() -> Self {
        Self {
            name: Cow::Borrowed("field"),
            description: None,
            dtype: DataType::None,
            is_primary: false,
            auto_id: false,
            chunk_size: 0,
            dim: 0,
            max_length: 0,
        }
    }
}

impl Default for FieldSchema<'static> {
    fn default() -> Self {
        Self::const_default()
    }
}

impl<'a> From<&'a schema::FieldSchema> for FieldSchema<'a> {
    fn from(fld: &'a schema::FieldSchema) -> Self {
        let dim: i64 = fld
            .type_params
            .iter()
            .find(|k| &k.key == "dim")
            .and_then(|x| x.value.parse().ok())
            .unwrap_or(1);

        let dtype = DataType::from_i32(fld.data_type).unwrap();

        FieldSchema {
            name: Cow::Borrowed(fld.name.as_str()),
            description: if fld.description.as_str() != "" {
                Some(Cow::Borrowed(fld.description.as_str()))
            } else {
                None
            },
            dtype,
            is_primary: fld.is_primary_key,
            auto_id: fld.auto_id,
            max_length: 0,
            chunk_size: (dim
                * match dtype {
                    DataType::BinaryVector => dim / 8,
                    _ => dim,
                }) as _,
            dim,
        }
    }
}

impl<'a> FieldSchema<'a> {
    pub const fn new_bool(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Bool,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_int8(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Int8,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_int16(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Int16,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_int32(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Int32,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_int64(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Int64,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_primary_int64(
        name: &'a str,
        description: Option<&'a str>,
        auto_id: bool,
    ) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Int64,
            is_primary: true,
            auto_id,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_primary_varchar(
        name: &'a str,
        description: Option<&'a str>,
        auto_id: bool,
        max_length: i32,
    ) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::VarChar,
            is_primary: true,
            auto_id,
            max_length,
            chunk_size: 1,
            dim: 1,
        }
    }

    pub const fn new_float(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Float,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_double(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::Double,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_string(name: &'a str, description: Option<&'a str>) -> Self {
        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::String,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
        }
    }

    pub const fn new_varchar(name: &'a str, description: Option<&'a str>, max_length: i32) -> Self {
        if max_length <= 0 {
            panic!("max_length should be positive");
        }

        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::String,
            max_length,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
        }
    }

    pub const fn new_binary_vector(name: &'a str, description: Option<&'a str>, dim: i64) -> Self {
        if dim <= 0 {
            panic!("dim should be positive");
        }

        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::BinaryVector,
            chunk_size: dim as usize / 8,
            dim,
            is_primary: false,
            auto_id: false,
            max_length: 0,
        }
    }

    pub const fn new_float_vector(name: &'a str, description: Option<&'a str>, dim: i64) -> Self {
        if dim <= 0 {
            panic!("dim should be positive");
        }

        Self {
            name: Cow::Borrowed(name),
            description: match description {
                Some(d) => Some(Cow::Borrowed(d)),
                None => None,
            },
            dtype: DataType::FloatVector,
            chunk_size: dim as usize,
            dim,
            is_primary: false,
            auto_id: false,
            max_length: 0,
        }
    }
}

impl<'a> From<FieldSchema<'a>> for schema::FieldSchema {
    fn from(fld: FieldSchema<'a>) -> schema::FieldSchema {
        let params = match fld.dtype {
            DataType::BinaryVector | DataType::FloatVector => vec![KeyValuePair {
                key: "dim".to_string(),
                value: fld.dim.to_string(),
            }],
            DataType::VarChar => vec![KeyValuePair {
                key: "max_length".to_string(),
                value: fld.max_length.to_string(),
            }],
            _ => Vec::new(),
        };

        schema::FieldSchema {
            field_id: 0,
            name: fld.name.into(),
            is_primary_key: fld.is_primary,
            description: fld.description.unwrap_or(Cow::Borrowed("")).into(),
            data_type: fld.dtype as i32,
            type_params: params,
            index_params: Vec::new(),
            auto_id: fld.auto_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionSchema<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) description: Option<Cow<'a, str>>,
    pub(crate) fields: Cow<'a, [FieldSchema<'a>]>,
}

impl<'a> CollectionSchema<'a> {
    #[inline]
    pub fn auto_id(&self) -> bool {
        self.fields.as_ref().into_iter().any(|x| x.auto_id)
    }

    pub fn primary_column(&self) -> Option<&FieldSchema<'a>> {
        self.fields.iter().find(|s| s.is_primary)
    }

    pub fn validate(&self) -> Result<()> {
        self.primary_column().ok_or_else(|| Error::NoPrimaryKey)?;
        // TODO addidtional schema checks need to be added here
        Ok(())
    }
}

impl<'a> From<CollectionSchema<'a>> for schema::CollectionSchema {
    fn from(col: CollectionSchema<'a>) -> Self {
        schema::CollectionSchema {
            name: col.name.to_string(),
            auto_id: col.auto_id(),
            description: col.description.unwrap_or(Cow::Borrowed("")).to_string(),
            fields: col
                .fields
                .into_owned()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl<'a> From<&'a schema::CollectionSchema> for CollectionSchema<'a> {
    fn from(v: &'a schema::CollectionSchema) -> Self {
        CollectionSchema {
            fields: v.fields.iter().map(Into::into).collect(),
            name: Cow::Borrowed(v.name.as_str()),
            description: if v.description.as_str() != "" {
                Some(Cow::Borrowed(v.description.as_str()))
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionSchemaBuilder<'a> {
    name: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    inner: Vec<FieldSchema<'a>>,
}

impl<'a> CollectionSchemaBuilder<'a> {
    pub fn new(name: Cow<'a, str>, description: Option<Cow<'a, str>>) -> Self {
        Self {
            name,
            description,
            inner: Vec::new(),
        }
    }

    pub fn add_field(&mut self, schema: FieldSchema<'a>) -> &mut Self {
        self.inner.push(schema);
        self
    }

    pub fn set_primary_key<S>(&mut self, name: S) -> Result<&mut Self>
    where
        S: AsRef<str>,
    {
        let n = name.as_ref();
        for f in self.inner.iter_mut() {
            if f.is_primary {
                return Err(error::Error::from(Error::DuplicatePrimaryKey(
                    n.to_string(),
                    f.name.to_string(),
                )));
            }
        }

        for f in self.inner.iter_mut() {
            if n == f.name.as_ref() {
                if f.dtype == DataType::Int64 || f.dtype == DataType::VarChar {
                    f.is_primary = true;
                    return Ok(self);
                } else {
                    return Err(error::Error::from(Error::UnsupportedPrimaryKey(
                        f.dtype.to_owned(),
                    )));
                }
            }
        }

        Err(error::Error::from(Error::NoSuchKey(n.to_string())))
    }

    pub fn enable_auto_id(&mut self) -> Result<&mut Self> {
        for f in self.inner.iter_mut() {
            if f.is_primary {
                if f.dtype == DataType::Int64 {
                    f.auto_id = true;
                    return Ok(self);
                } else {
                    return Err(error::Error::from(Error::UnsupportedAutoId(
                        f.dtype.to_owned(),
                    )));
                }
            }
        }

        Err(error::Error::from(Error::NoPrimaryKey))
    }

    pub fn build(&mut self) -> Result<CollectionSchema> {
        let mut has_primary = false;

        for f in self.inner.iter() {
            if f.is_primary {
                has_primary = true;
                break;
            }
        }

        if !has_primary {
            return Err(error::Error::from(Error::NoPrimaryKey));
        }

        let this = std::mem::replace(self, CollectionSchemaBuilder::new("".into(), None));

        Ok(CollectionSchema {
            fields: this.inner.into(),
            name: this.name,
            description: this.description,
        })
    }
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("try to set primary key {0:?}, but {1:?} is also key")]
    DuplicatePrimaryKey(String, String),

    #[error("can not find any primary key")]
    NoPrimaryKey,

    #[error("primary key must be int64 or varchar, unsupported type {0:?}")]
    UnsupportedPrimaryKey(DataType),

    #[error("auto id must be int64, unsupported type {0:?}")]
    UnsupportedAutoId(DataType),

    #[error("dimension mismatch for {0:?}, expected dim {1:?}, got {2:?}")]
    DimensionMismatch(String, i32, i32),

    #[error("wrong field data type, field {0} expected to be{1:?}, but got {2:?}")]
    FieldWrongType(String, DataType, DataType),

    #[error("field does not exists in schema: {0:?}")]
    FieldDoesNotExists(String),

    #[error("can not find such key {0:?}")]
    NoSuchKey(String),
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    use super::FieldSchema;

    struct Test {
        pub id: i64,
        pub hash: Vec<u8>,
        pub listing_id: i32,
        pub provider: i8,
    }

    impl super::Entity for Test {
        const NAME: &'static str = "test";
        const SCHEMA: &'static [FieldSchema<'static>] = &[
            FieldSchema::new_primary_int64("id", None, false),
            FieldSchema::new_binary_vector("hash", None, 1024),
            FieldSchema::new_int32("listing_id", None),
            FieldSchema::new_int8("provider", None),
        ];

        //
        // Non-static one is wating for GATs (https://github.com/rust-lang/rust/pull/96709)
        //
        // type ColumnIter<'a> = std::array::IntoIter<
        //     (&'static FieldSchema<'static>, Value<'a>),
        //     { Self::SCHEMA.len() },
        // >;

        type ColumnIntoIter = std::array::IntoIter<
            (&'static FieldSchema<'static>, Value<'static>),
            { Self::SCHEMA.len() },
        >;

        fn iter(&self) -> Self::ColumnIntoIter {
            [
                (&Self::SCHEMA[0], self.id.into()),
                (&Self::SCHEMA[1], self.hash.clone().into()),
                (&Self::SCHEMA[2], self.listing_id.into()),
                (&Self::SCHEMA[3], self.provider.into()),
            ]
            .into_iter()
        }

        fn into_iter(self) -> Self::ColumnIntoIter {
            [
                (&Self::SCHEMA[0], self.id.into()),
                (&Self::SCHEMA[1], self.hash.into()),
                (&Self::SCHEMA[2], self.listing_id.into()),
                (&Self::SCHEMA[3], self.provider.into()),
            ]
            .into_iter()
        }
    }

    #[test]
    fn test_const_schema() {}
}
