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
use prost::alloc::vec::Vec;
use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use thiserror::Error as ThisError;

use crate::proto::{
    common::KeyValuePair,
    schema::{
        self, field_data::Field, scalar_field::Data as ScalarData,
        vector_field::Data as VectorData, DataType, ScalarField, VectorField,
    },
};

pub use crate::proto::schema::FieldData;

pub trait HasDataType {
    fn data_type() -> DataType;
}

macro_rules! impl_has_data_type {
    ( $($t: ty, $o: expr ),+ ) => {$(
        impl HasDataType for $t {
            fn data_type() -> DataType {
                $o
            }
        }
    )*};
}

impl_has_data_type! {
    bool, DataType::Bool,
    i8, DataType::Int8,
    i16, DataType::Int16,
    i32, DataType::Int32,
    i64, DataType::Int64,
    f32, DataType::Float,
    f64, DataType::Double,
    String, DataType::String,
    Cow<'_, str>, DataType::String,
    Vec<f32>, DataType::FloatVector,
    Vec<u8>, DataType::BinaryVector,
    Cow<'_, [f32]>, DataType::FloatVector,
    Cow<'_, [u8]>, DataType::BinaryVector
}

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

impl<'a> Default for CollectionSchemaBuilder<'a> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            description: None,
            inner: Vec::new(),
        }
    }
}

impl<'a> CollectionSchemaBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
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

        let this = std::mem::take(self);

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

pub enum Value<'a> {
    None,
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    FloatArray(Cow<'a, [f32]>),
    Binary(Cow<'a, [u8]>),
    String(Cow<'a, str>),
}

macro_rules! impl_from_for_field_data_column {
    ( $($t: ty, $o: ident ),+ ) => {$(
        impl From<$t> for Value<'static> {
            fn from(v: $t) -> Self {
                Self::$o(v as _)
            }
        }
    )*};
}

impl_from_for_field_data_column! {
    bool, Bool,
    i8,  Int8,
    i16, Int16,
    i32, Int32,
    i64, Long,
    f32, Float,
    f64, Double
}

impl Value<'_> {
    fn data_type(&self) -> DataType {
        match self {
            Value::None => DataType::None,
            Value::Bool(_) => DataType::Bool,
            Value::Int8(_) => DataType::Int8,
            Value::Int16(_) => DataType::Int16,
            Value::Int32(_) => DataType::Int32,
            Value::Long(_) => DataType::Int64,
            Value::Float(_) => DataType::Float,
            Value::Double(_) => DataType::Double,
            Value::String(_) => DataType::String,
            Value::FloatArray(_) => DataType::FloatVector,
            Value::Binary(_) => DataType::BinaryVector,
        }
    }
}

impl From<String> for Value<'static> {
    fn from(v: String) -> Self {
        Self::String(Cow::Owned(v))
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(v: &'a str) -> Self {
        Self::String(Cow::Borrowed(v))
    }
}

impl<'a> From<&'a [u8]> for Value<'a> {
    fn from(v: &'a [u8]) -> Self {
        Self::Binary(Cow::Borrowed(v))
    }
}

impl From<Vec<u8>> for Value<'static> {
    fn from(v: Vec<u8>) -> Self {
        Self::Binary(Cow::Owned(v))
    }
}

impl<'a> From<&'a [f32]> for Value<'a> {
    fn from(v: &'a [f32]) -> Self {
        Self::FloatArray(Cow::Borrowed(v))
    }
}

impl From<Vec<f32>> for Value<'static> {
    fn from(v: Vec<f32>) -> Self {
        Self::FloatArray(Cow::Owned(v))
    }
}

#[derive(Debug, Clone)]
pub enum ValueVec {
    None,
    Bool(Vec<bool>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Binary(Vec<u8>),
    String(Vec<String>),
}

macro_rules! impl_from_for_value_vec {
    ( $($t: ty, $o: ident ),+ ) => {$(
        impl From<$t> for ValueVec {
            fn from(v: $t) -> Self {
                Self::$o(v)
            }
        }
    )*};
}

impl_from_for_value_vec! {
    Vec<bool>, Bool,
    Vec<i32>, Int,
    Vec<i64>, Long,
    Vec<String>, String,
    Vec<u8>, Binary,
    Vec<f32>, Float,
    Vec<f64>, Double
}

impl From<Vec<i8>> for ValueVec {
    fn from(v: Vec<i8>) -> Self {
        Self::Int(v.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<i16>> for ValueVec {
    fn from(v: Vec<i16>) -> Self {
        Self::Int(v.into_iter().map(Into::into).collect())
    }
}

impl ValueVec {
    pub fn new(dtype: DataType) -> Self {
        match dtype {
            DataType::None => Self::None,
            DataType::Bool => Self::Bool(Vec::new()),
            DataType::Int8 => Self::Int(Vec::new()),
            DataType::Int16 => Self::Int(Vec::new()),
            DataType::Int32 => Self::Int(Vec::new()),
            DataType::Int64 => Self::Long(Vec::new()),
            DataType::Float => Self::Float(Vec::new()),
            DataType::Double => Self::Double(Vec::new()),
            DataType::String => Self::String(Vec::new()),
            DataType::VarChar => Self::String(Vec::new()),
            DataType::BinaryVector => Self::Binary(Vec::new()),
            DataType::FloatVector => Self::Float(Vec::new()),
        }
    }

    pub fn check_dtype(&self, dtype: DataType) -> bool {
        match (self, dtype) {
            (ValueVec::Binary(..), DataType::BinaryVector)
            | (ValueVec::Float(..), DataType::FloatVector)
            | (ValueVec::Float(..), DataType::Float)
            | (ValueVec::Int(..), DataType::Int8)
            | (ValueVec::Int(..), DataType::Int16)
            | (ValueVec::Int(..), DataType::Int32)
            | (ValueVec::Long(..), DataType::Int64)
            | (ValueVec::Bool(..), DataType::Bool)
            | (ValueVec::String(..), DataType::String)
            | (ValueVec::String(..), DataType::VarChar)
            | (ValueVec::None, _)
            | (ValueVec::Double(..), DataType::Double) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        match self {
            ValueVec::None => 0,
            ValueVec::Bool(v) => v.len(),
            ValueVec::Int(v) => v.len(),
            ValueVec::Long(v) => v.len(),
            ValueVec::Float(v) => v.len(),
            ValueVec::Double(v) => v.len(),
            ValueVec::Binary(v) => v.len(),
            ValueVec::String(v) => v.len(),
        }
    }

    pub fn clear(&mut self) {
        match self {
            ValueVec::None => (),
            ValueVec::Bool(v) => v.clear(),
            ValueVec::Int(v) => v.clear(),
            ValueVec::Long(v) => v.clear(),
            ValueVec::Float(v) => v.clear(),
            ValueVec::Double(v) => v.clear(),
            ValueVec::Binary(v) => v.clear(),
            ValueVec::String(v) => v.clear(),
        }
    }
}

impl From<Field> for ValueVec {
    fn from(f: Field) -> Self {
        match f {
            Field::Scalars(s) => match s.data {
                Some(x) => match x {
                    ScalarData::BoolData(v) => Self::Bool(v.data),
                    ScalarData::IntData(v) => Self::Int(v.data),
                    ScalarData::LongData(v) => Self::Long(v.data),
                    ScalarData::FloatData(v) => Self::Float(v.data),
                    ScalarData::DoubleData(v) => Self::Double(v.data),
                    ScalarData::StringData(v) => Self::String(v.data),
                    ScalarData::BytesData(v) => unimplemented!(), // Self::Bytes(v.data),
                },
                None => Self::None,
            },

            Field::Vectors(arr) => match arr.data {
                Some(x) => match x {
                    VectorData::FloatVector(v) => Self::Float(v.data),
                    VectorData::BinaryVector(v) => Self::Binary(v),
                },
                None => Self::None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldColumn<'a> {
    name: Cow<'a, str>,
    dtype: DataType,
    value: ValueVec,
    dim: i64,
    max_length: i32,
}

impl<'a> From<schema::FieldData> for FieldColumn<'a> {
    fn from(fd: schema::FieldData) -> Self {
        let (dim, max_length) = fd
            .field
            .as_ref()
            .map(get_dim_max_length)
            .unwrap_or((None, None));

        let value: ValueVec = fd.field.map(Into::into).unwrap_or(ValueVec::None);
        let dtype = DataType::from_i32(fd.r#type).unwrap_or(DataType::None);

        FieldColumn {
            name: Cow::Owned(fd.field_name),
            dtype,
            dim: dim.unwrap_or_else(|| match dtype {
                DataType::None => 0,
                DataType::Bool
                | DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::Float
                | DataType::Double
                | DataType::String
                | DataType::VarChar => 1,
                DataType::BinaryVector => 256,
                DataType::FloatVector => 128,
            }),
            max_length: max_length.unwrap_or(0),
            value,
        }
    }
}

impl<'a> FieldColumn<'a> {
    pub fn index(&self, idx: usize) -> Option<Value<'_>> {
        unimplemented!()
    }

    fn push(&mut self, val: Value) {
        match (&mut self.value, val) {
            (ValueVec::None, Value::None) => (),
            (ValueVec::Bool(vec), Value::Bool(i)) => vec.push(i),
            (ValueVec::Int(vec), Value::Int8(i)) => vec.push(i as _),
            (ValueVec::Int(vec), Value::Int16(i)) => vec.push(i as _),
            (ValueVec::Int(vec), Value::Int32(i)) => vec.push(i),
            (ValueVec::Long(vec), Value::Long(i)) => vec.push(i),
            (ValueVec::Float(vec), Value::Float(i)) => vec.push(i),
            (ValueVec::Double(vec), Value::Double(i)) => vec.push(i),
            (ValueVec::String(vec), Value::String(i)) => vec.push(i.to_string()),
            (ValueVec::Binary(vec), Value::Binary(i)) => vec.extend_from_slice(i.as_ref()),
            (ValueVec::Float(vec), Value::FloatArray(i)) => vec.extend_from_slice(i.as_ref()),
            _ => panic!("column type mismatch"),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.value.len()
    }
}

fn get_dim_max_length(field: &Field) -> (Option<i64>, Option<i32>) {
    let dim = match field {
        Field::Scalars(ScalarField { data: Some(_) }) => 1i64,
        Field::Vectors(VectorField { dim, .. }) => *dim,
        _ => 0i64,
    };

    (Some(dim), None) // no idea how to get max_length
}

impl<'a> From<FieldColumn<'a>> for schema::FieldData {
    fn from(this: FieldColumn<'a>) -> schema::FieldData {
        schema::FieldData {
            field_name: this.name.to_string(),
            field_id: 0,
            r#type: this.dtype as _,
            field: Some(match this.value {
                ValueVec::None => Field::Scalars(ScalarField { data: None }),
                ValueVec::Bool(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::BoolData(schema::BoolArray { data: v })),
                }),
                ValueVec::Int(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::IntData(schema::IntArray { data: v })),
                }),
                ValueVec::Long(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::LongData(schema::LongArray { data: v })),
                }),
                ValueVec::Float(v) => match this.dtype {
                    DataType::Float => Field::Scalars(ScalarField {
                        data: Some(ScalarData::FloatData(schema::FloatArray { data: v })),
                    }),
                    DataType::FloatVector => Field::Vectors(VectorField {
                        data: Some(VectorData::FloatVector(schema::FloatArray { data: v })),
                        dim: this.dim,
                    }),
                    _ => unimplemented!(),
                },
                ValueVec::Double(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::DoubleData(schema::DoubleArray { data: v })),
                }),
                ValueVec::String(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::StringData(schema::StringArray { data: v })),
                }),
                ValueVec::Binary(v) => Field::Vectors(VectorField {
                    data: Some(VectorData::BinaryVector(v)),
                    dim: this.dim,
                }),
            }),
        }
    }
}

pub struct CollectionDataBatch<E> {
    num_rows: usize,
    columns: HashMap<Cow<'static, str>, FieldColumn<'static>>,
    _m: PhantomData<E>,
}

impl<E: Entity> From<Vec<FieldData>> for CollectionDataBatch<E> {
    fn from(data: Vec<FieldData>) -> Self {
        let columns: HashMap<Cow<'static, str>, FieldColumn> = data
            .into_iter()
            .map(|mut fld| (std::mem::take(&mut fld.field_name).into(), fld.into()))
            .collect();

        let schema = E::schema();
        let primary = schema.primary_column().unwrap();
        let num_rows = columns.get(&primary.name).unwrap().len();

        Self {
            num_rows,
            columns,
            _m: Default::default(),
        }
    }
}

impl<E> From<CollectionDataBatch<E>> for Vec<FieldData> {
    fn from(batch: CollectionDataBatch<E>) -> Vec<FieldData> {
        batch.columns.into_values().map(Into::into).collect()
    }
}

impl<E: Entity> Default for CollectionDataBatch<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> CollectionDataBatch<E> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_rows == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.num_rows
    }
}

impl<E: Entity> CollectionDataBatch<E> {
    pub fn new() -> Self {
        let columns: HashMap<_, _> = E::SCHEMA
            .iter()
            .map(|x| {
                (
                    x.name.clone(),
                    FieldColumn {
                        name: x.name.clone(),
                        dtype: x.dtype,
                        value: ValueVec::new(x.dtype),
                        dim: x.dim,
                        max_length: x.max_length,
                    },
                )
            })
            .collect();

        Self {
            num_rows: 0,
            columns,
            _m: Default::default(),
        }
    }

    #[inline]
    pub fn add(&mut self, entity: E) {
        for (schm, val) in entity.iter() {
            self.columns.get_mut(schm.name.as_ref()).unwrap().push(val);
        }
    }

    #[inline]
    pub fn column(&self, name: &str) -> Option<&FieldColumn> {
        self.columns.get(name)
    }
}

pub struct CollectionDataBatchBuilder<E> {
    inner: CollectionDataBatch<E>,
}

impl<E: Entity> CollectionDataBatchBuilder<E> {
    pub fn new() -> Self {
        Self {
            inner: CollectionDataBatch::new(),
        }
    }

    pub fn set_column_data<D: Into<ValueVec>>(&mut self, col: &str, data: D) {
        let data: ValueVec = data.into();
        let col = self.inner.columns.get_mut(col).unwrap();

        assert!(data.check_dtype(col.dtype));

        col.value = data;
    }

    pub fn build(&mut self) -> Result<CollectionDataBatch<E>> {
        Ok(std::mem::take(&mut self.inner))
    }
}

pub trait FromField: Sized {
    fn from_field(field: Field) -> Option<Self>;
}

macro_rules! impl_from_field {
    ( $( $t: ty [$($e:tt)*] ),+ ) => {$(

        impl FromField for $t {
            fn from_field(v: Field) -> Option<Self> {
                match v {
                    $($e)*,
                    _ => None
                }
            }
        }
    )*};
}

impl_from_field! {
    Vec<bool>[Field::Scalars(ScalarField {data: Some(ScalarData::BoolData(schema::BoolArray { data }))}) => Some(data)],
    Vec<i8>[Field::Scalars(ScalarField {data: Some(ScalarData::IntData(schema::IntArray { data }))}) => Some(data.into_iter().map(|x|x as _).collect())],
    Vec<i16>[Field::Scalars(ScalarField {data: Some(ScalarData::IntData(schema::IntArray { data }))}) => Some(data.into_iter().map(|x|x as _).collect())],
    Vec<i32>[Field::Scalars(ScalarField {data: Some(ScalarData::IntData(schema::IntArray { data }))}) => Some(data)],
    Vec<i64>[Field::Scalars(ScalarField {data: Some(ScalarData::LongData(schema::LongArray { data }))}) => Some(data)],
    Vec<String>[Field::Scalars(ScalarField {data: Some(ScalarData::StringData(schema::StringArray { data }))}) => Some(data)],
    Vec<f64>[Field::Scalars(ScalarField {data: Some(ScalarData::DoubleData(schema::DoubleArray { data }))}) => Some(data)],
    Vec<u8>[Field::Vectors(VectorField {data: Some(VectorData::BinaryVector(data)), ..}) => Some(data)]
}

impl FromField for Vec<f32> {
    fn from_field(field: Field) -> Option<Self> {
        match field {
            Field::Scalars(ScalarField {
                data: Some(ScalarData::FloatData(schema::FloatArray { data })),
            }) => Some(data),

            Field::Vectors(VectorField {
                data: Some(VectorData::FloatVector(schema::FloatArray { data })),
                ..
            }) => Some(data),

            _ => None,
        }
    }
}

pub fn make_field_data<V: Into<ValueVec>>(schm: &FieldSchema, v: V) -> FieldData {
    let field_column = FieldColumn {
        name: schm.name.clone(),
        dtype: schm.dtype,
        value: v.into(),
        dim: schm.dim,
        max_length: schm.max_length,
    };

    field_column.into()
}

#[cfg(test)]
mod tests {

    use super::{FieldSchema, Value};

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
