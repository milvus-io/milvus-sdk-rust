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
use crate::proto::schema::VectorField;
use crate::proto::schema::{
    self, field_data::Field, scalar_field::Data as ScalarData, DataType, ScalarField,
};
use prost::alloc::vec::Vec;
use std::collections::HashMap;
use std::error::Error as _;
use thiserror::Error as ThisError;

#[derive(Debug, Clone)]
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

impl<'a> From<&'a schema::FieldSchema> for FieldSchema {
    fn from(fld: &'a schema::FieldSchema) -> Self {
        let dim: i32 = fld
            .type_params
            .iter()
            .find(|k| &k.key == "dim")
            .and_then(|x| x.value.parse().ok())
            .unwrap_or(1);

        FieldSchema {
            name: fld.name.clone(),
            description: fld.description.clone(),
            dtype: DataType::from_i32(fld.data_type).unwrap(),
            is_primary: fld.is_primary_key,
            auto_id: fld.auto_id,
            max_length: 0,
            dim,
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
            max_length,
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
            dim,
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
            dim,
            ..Default::default()
        }
    }
    pub fn convert_field(self) -> schema::FieldSchema {
        let params = match self.dtype {
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
            type_params: params,
            index_params: Vec::new(),
            auto_id: self.auto_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionSchema {
    inner: Vec<FieldSchema>,
    auto_id: bool,
}

impl CollectionSchema {
    pub fn convert_collection<S>(self, name: S, description: S) -> schema::CollectionSchema
    where
        S: Into<String>,
    {
        schema::CollectionSchema {
            name: name.into(),
            description: description.into(),
            auto_id: self.auto_id,
            fields: self
                .inner
                .into_iter()
                .map(FieldSchema::convert_field)
                .collect(),
        }
    }
}

impl<'a> From<&'a schema::CollectionSchema> for CollectionSchema {
    fn from(v: &'a schema::CollectionSchema) -> Self {
        CollectionSchema {
            inner: v.fields.iter().map(Into::into).collect(),
            auto_id: v.auto_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionSchemaBuilder {
    inner: Vec<FieldSchema>,
}

impl CollectionSchemaBuilder {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add_field(&mut self, schema: FieldSchema) -> &mut Self {
        self.inner.push(schema);
        self
    }

    pub fn set_primary_key<S>(&mut self, name: S) -> Result<&mut Self>
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
                    return Ok(self);
                } else {
                    return Err(error::Error::from(Error::UnsupportedPrimaryKey(
                        f.dtype.to_owned(),
                    )));
                }
            }
        }
        Err(error::Error::from(Error::NoSuchKey(n)))
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
        Err(error::Error::from(Error::NoPrimaryKey()))
    }

    pub fn build(&self) -> Result<CollectionSchema> {
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
            inner: self.inner.clone(),
            auto_id: auto,
        })
    }
}

pub trait AsFieldDataValue {
    fn as_field_data_value(&self) -> FieldDataValue<'_>;
}

impl AsFieldDataValue for i64 {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::Int64(*self)
    }
}

impl AsFieldDataValue for i32 {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::Int32(*self)
    }
}

impl AsFieldDataValue for i16 {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::Int16(*self)
    }
}

impl AsFieldDataValue for i8 {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::Int8(*self)
    }
}

impl AsFieldDataValue for [u8] {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::BinaryVector(self)
    }
}

impl AsFieldDataValue for Vec<u8> {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::BinaryVector(self)
    }
}

impl AsFieldDataValue for [f32] {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::FloatVector(self)
    }
}

impl AsFieldDataValue for Vec<f32> {
    fn as_field_data_value(&self) -> FieldDataValue<'_> {
        FieldDataValue::FloatVector(self)
    }
}

#[derive(Debug, Clone)]
pub enum FieldDataValue<'a> {
    None,
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(&'a str),
    BinaryVector(&'a [u8]),
    FloatVector(&'a [f32]),
}

impl<'a> FieldDataValue<'a> {
    pub fn get_dim(&self) -> usize {
        match self {
            FieldDataValue::None => 0,
            FieldDataValue::Bool(_)
            | FieldDataValue::Int8(_)
            | FieldDataValue::Int16(_)
            | FieldDataValue::Int32(_)
            | FieldDataValue::Int64(_)
            | FieldDataValue::Float(_)
            | FieldDataValue::Double(_)
            | FieldDataValue::String(_) => 1,
            FieldDataValue::BinaryVector(s) => s.len() * 8,
            FieldDataValue::FloatVector(s) => s.len(),
        }
    }

    #[inline]
    pub fn data_type(&self) -> DataType {
        match self {
            FieldDataValue::None => DataType::None,
            FieldDataValue::Bool(_) => DataType::Bool,
            FieldDataValue::Int8(_) => DataType::Int8,
            FieldDataValue::Int16(_) => DataType::Int16,
            FieldDataValue::Int32(_) => DataType::Int32,
            FieldDataValue::Int64(_) => DataType::Int64,
            FieldDataValue::Float(_) => DataType::Float,
            FieldDataValue::Double(_) => DataType::Double,
            FieldDataValue::String(_) => DataType::String,
            // FieldDataValue::VarChar(_) => DataType::VarChar,
            FieldDataValue::BinaryVector(_) => DataType::BinaryVector,
            FieldDataValue::FloatVector(_) => DataType::FloatVector,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldDataKind {
    None,
    Bool(Vec<bool>),
    Int8(Vec<i32>),
    Int16(Vec<i32>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    String(Vec<String>),
    VarChar(Vec<String>),

    BinaryVector(i64, Vec<u8>),
    FloatVector(i64, Vec<f32>),
}

impl FieldDataKind {
    #[inline]
    pub fn data_type(&self) -> DataType {
        match self {
            FieldDataKind::None => DataType::None,
            FieldDataKind::Bool(_) => DataType::Bool,
            FieldDataKind::Int8(_) => DataType::Int8,
            FieldDataKind::Int16(_) => DataType::Int16,
            FieldDataKind::Int32(_) => DataType::Int32,
            FieldDataKind::Int64(_) => DataType::Int64,
            FieldDataKind::Float(_) => DataType::Float,
            FieldDataKind::Double(_) => DataType::Double,
            FieldDataKind::String(_) => DataType::String,
            FieldDataKind::VarChar(_) => DataType::VarChar,
            FieldDataKind::BinaryVector(..) => DataType::BinaryVector,
            FieldDataKind::FloatVector(..) => DataType::FloatVector,
        }
    }

    fn push(&mut self, v: FieldDataValue) -> std::result::Result<(), Error> {
        match (v, self) {
            (FieldDataValue::Bool(v), FieldDataKind::Bool(vec)) => vec.push(v),
            (FieldDataValue::Int8(v), FieldDataKind::Int8(vec)) => vec.push(v as _),
            (FieldDataValue::Int16(v), FieldDataKind::Int16(vec)) => vec.push(v as _),
            (FieldDataValue::Int32(v), FieldDataKind::Int32(vec)) => vec.push(v),
            (FieldDataValue::Int64(v), FieldDataKind::Int64(vec)) => vec.push(v),
            (FieldDataValue::Float(v), FieldDataKind::Float(vec)) => vec.push(v),
            (FieldDataValue::Double(v), FieldDataKind::Double(vec)) => vec.push(v),
            (FieldDataValue::String(v), FieldDataKind::String(vec)) => vec.push(v.to_string()),
            (FieldDataValue::BinaryVector(v), FieldDataKind::BinaryVector(_, vec)) => {
                vec.extend_from_slice(v)
            }
            (FieldDataValue::FloatVector(v), FieldDataKind::FloatVector(_, vec)) => {
                vec.extend_from_slice(v)
            }

            (a, b) => return Err(Error::FieldWrongType(a.data_type(), b.data_type())),
        }

        Ok(())
    }

    #[inline]
    fn dim(&self) -> i64 {
        match self {
            FieldDataKind::None => 0,
            FieldDataKind::Bool(_)
            | FieldDataKind::Int8(_)
            | FieldDataKind::Int16(_)
            | FieldDataKind::Int32(_)
            | FieldDataKind::Int64(_)
            | FieldDataKind::Float(_)
            | FieldDataKind::Double(_)
            | FieldDataKind::String(_)
            | FieldDataKind::VarChar(_) => 1,
            FieldDataKind::BinaryVector(d, _) => *d,
            FieldDataKind::FloatVector(d, _) => *d,
        }
    }

    fn convert(self) -> Field {
        match self {
            FieldDataKind::None => Field::Scalars(ScalarField { data: None }),
            FieldDataKind::Bool(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::BoolData(schema::BoolArray { data: v })),
            }),
            FieldDataKind::Int8(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::IntData(schema::IntArray { data: v })),
            }),
            FieldDataKind::Int16(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::IntData(schema::IntArray { data: v })),
            }),
            FieldDataKind::Int32(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::IntData(schema::IntArray { data: v })),
            }),
            FieldDataKind::Int64(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::LongData(schema::LongArray { data: v })),
            }),
            FieldDataKind::Float(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::FloatData(schema::FloatArray { data: v })),
            }),
            FieldDataKind::Double(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::DoubleData(schema::DoubleArray { data: v })),
            }),
            FieldDataKind::String(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::StringData(schema::StringArray { data: v })),
            }),
            FieldDataKind::VarChar(v) => Field::Scalars(ScalarField {
                data: Some(ScalarData::StringData(schema::StringArray { data: v })),
            }),
            FieldDataKind::BinaryVector(dim, v) => Field::Vectors(VectorField {
                data: Some(schema::vector_field::Data::BinaryVector(v)),
                dim,
            }),
            FieldDataKind::FloatVector(dim, v) => Field::Vectors(VectorField {
                data: Some(schema::vector_field::Data::FloatVector(
                    schema::FloatArray { data: v },
                )),
                dim,
            }),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        match self {
            FieldDataKind::None => (),
            FieldDataKind::Bool(v) => v.clear(),
            FieldDataKind::Int8(v) => v.clear(),
            FieldDataKind::Int16(v) => v.clear(),
            FieldDataKind::Int32(v) => v.clear(),
            FieldDataKind::Int64(v) => v.clear(),
            FieldDataKind::Float(v) => v.clear(),
            FieldDataKind::Double(v) => v.clear(),
            FieldDataKind::String(v) => v.clear(),
            FieldDataKind::VarChar(v) => v.clear(),
            FieldDataKind::BinaryVector(_, v) => v.clear(),
            FieldDataKind::FloatVector(_, v) => v.clear(),
        }
    }
}

impl From<Field> for FieldDataKind {
    fn from(f: Field) -> Self {
        match f {
            Field::Scalars(s) => match s.data {
                Some(x) => match x {
                    ScalarData::BoolData(v) => Self::Bool(v.data),
                    ScalarData::IntData(v) => Self::Int32(v.data),
                    ScalarData::LongData(v) => Self::Int64(v.data),
                    ScalarData::FloatData(v) => Self::Float(v.data),
                    ScalarData::DoubleData(v) => Self::Double(v.data),
                    ScalarData::StringData(v) => Self::String(v.data),
                    ScalarData::BytesData(v) => unimplemented!(), // Self::Bytes(v.data),
                },
                None => Self::None,
            },

            Field::Vectors(arr) => match arr.data {
                Some(x) => match x {
                    schema::vector_field::Data::FloatVector(v) => {
                        Self::FloatVector(arr.dim, v.data)
                    }
                    schema::vector_field::Data::BinaryVector(v) => Self::BinaryVector(arr.dim, v),
                },
                None => Self::None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldData {
    name: String,
    dim: usize,
    field: FieldDataKind,
}

impl FieldData {
    fn new(name: String, dim: usize, field: FieldDataKind) -> Self {
        Self { name, dim, field }
    }

    pub fn convert(self) -> schema::FieldData {
        schema::FieldData {
            field_name: self.name,
            field_id: 0,
            r#type: self.field.data_type() as _,
            field: Some(self.field.convert()),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.field.clear();
    }
}

impl From<schema::FieldData> for FieldData {
    fn from(fd: schema::FieldData) -> Self {
        let field: FieldDataKind = fd.field.map(Into::into).unwrap_or(FieldDataKind::None);

        FieldData {
            name: fd.field_name,
            dim: field.dim() as _,
            field,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionFieldsData {
    num_rows: usize,
    fields: HashMap<String, FieldData>,
}

impl CollectionFieldsData {
    pub fn add<'a, I: IntoIterator<Item = (&'a str, &'a dyn AsFieldDataValue)>>(
        &mut self,
        row: I,
    ) -> std::result::Result<(), Error> {
        for (k, v) in row {
            let v: FieldDataValue = v.as_field_data_value();
            let x = self
                .fields
                .get_mut(k)
                .ok_or_else(|| Error::FieldDoesNotExists(k.to_string()))?;

            let actual_dim = v.get_dim();

            if actual_dim != x.dim {
                return Err(Error::DimensionMismatch(
                    k.to_string(),
                    actual_dim as _,
                    x.dim as _,
                ));
            }

            x.field.push(v)?;
        }

        self.num_rows += 1;

        Ok(())
    }

    pub fn new(schema: &CollectionSchema) -> Self {
        let mut fields = HashMap::new();

        for fld in schema.inner.iter() {
            let (dim, knd) = match fld.dtype {
                DataType::None => (0, FieldDataKind::None),
                DataType::Bool => (1, FieldDataKind::Bool(Vec::new())),
                DataType::Int8 => (1, FieldDataKind::Int8(Vec::new())),
                DataType::Int16 => (1, FieldDataKind::Int16(Vec::new())),
                DataType::Int32 => (1, FieldDataKind::Int32(Vec::new())),
                DataType::Int64 => (1, FieldDataKind::Int64(Vec::new())),
                DataType::Float => (1, FieldDataKind::Float(Vec::new())),
                DataType::Double => (1, FieldDataKind::Double(Vec::new())),
                DataType::String => (1, FieldDataKind::String(Vec::new())),
                DataType::VarChar => (1, FieldDataKind::VarChar(Vec::new())),
                DataType::BinaryVector => (
                    fld.dim,
                    FieldDataKind::BinaryVector(fld.dim as _, Vec::new()),
                ),
                DataType::FloatVector => (
                    fld.dim,
                    FieldDataKind::FloatVector(fld.dim as _, Vec::new()),
                ),
            };

            fields.insert(
                fld.name.clone(),
                FieldData::new(fld.name.clone(), dim as _, knd),
            );
        }

        Self {
            fields,
            num_rows: 0,
        }
    }

    #[inline]
    pub fn convert(self) -> Vec<schema::FieldData> {
        self.fields.into_values().map(|x| x.convert()).collect()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.num_rows
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_rows == 0
    }

    pub fn clear(&mut self) {
        self.num_rows = 0;
        self.fields.values_mut().for_each(|x| x.clear())
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

    #[error("dimension mismatch for {0:?}, got {1:?}, expected {2:?}")]
    DimensionMismatch(String, i32, i32),

    #[error("wrong data type, got {0:?}, expected {1:?}")]
    FieldWrongType(DataType, DataType),

    #[error("field does not exists in schema: {0:?}")]
    FieldDoesNotExists(String),

    #[error("can not find such key {0:?}")]
    NoSuchKey(String),
}
