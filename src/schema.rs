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
use crate::proto::schema::FieldState;
use crate::{error, index::FunctionType};
use prost::alloc::vec::Vec;
use prost::encoding::bool;
use thiserror::Error as ThisError;

use crate::proto::{
    common::KeyValuePair,
    schema::{self, DataType},
};

pub use crate::proto::schema::FieldData;

pub trait Schema {
    // fn name(&self) -> &str;
    // fn description(&self) -> &str;
    // fn fields(&self) -> &Vec<FieldSchema>;

    // fn schema(&self) -> CollectionSchema {
    //     CollectionSchema {
    //         name: self.name(),
    //         description: self.description(),
    //         fields: self.fields().to_owned(),
    //     }
    // }

    // type ColumnIntoIter<'a>: Iterator<Item = (&'a FieldSchema, Value<'a>)>;
    // type ColumnIter<'a>: Iterator<Item = (&'static FieldSchema<'static>, Value<'a>)>;

    // fn iter(&self) -> Self::ColumnIntoIter; // Self::ColumnIter<'_>
    // fn into_iter(self) -> Self::ColumnIntoIter;

    // fn validate(&self) -> std::result::Result<(), Error> {
    //     for (schm, val) in self.iter() {
    //         let dtype = val.data_type();

    //         if dtype != schm.dtype
    //             && !(dtype == DataType::String && schm.dtype == DataType::VarChar)
    //         {
    //             return Err(Error::FieldWrongType(
    //                 schm.name.to_string(),
    //                 schm.dtype,
    //                 val.data_type(),
    //             ));
    //         }

    //         match schm.dtype {
    //             DataType::VarChar => match &val {
    //                 Value::String(d) if d.len() > schm.max_length as _ => {
    //                     return Err(Error::DimensionMismatch(
    //                         schm.name.to_string(),
    //                         schm.max_length as _,
    //                         d.len() as _,
    //                     ));
    //                 }
    //                 _ => unreachable!(),
    //             },
    //             DataType::BinaryVector => match &val {
    //                 Value::Binary(d) => {
    //                     return Err(Error::DimensionMismatch(
    //                         schm.name.to_string(),
    //                         schm.dim as _,
    //                         d.len() as _,
    //                     ));
    //                 }
    //                 _ => unreachable!(),
    //             },
    //             DataType::FloatVector => match &val {
    //                 Value::FloatArray(d) => {
    //                     return Err(Error::DimensionMismatch(
    //                         schm.name.to_string(),
    //                         schm.dim as _,
    //                         d.len() as _,
    //                     ));
    //                 }
    //                 _ => unreachable!(),
    //             },
    //             _ => (),
    //         }
    //     }

    //     Ok(())
    // }
}

pub trait FromDataFields: Sized {
    fn from_data_fields(fileds: Vec<FieldData>) -> Option<Self>;
}

// pub trait Column<'a>: IntoFieldData + FromDataFields {
//     type Entity: Schema;
//     type IterRows: Iterator<Item = Self::Entity> + 'a;
//     type IterColumns: Iterator<Item = FieldColumn<'static>> + 'a;

//     fn index(&self, idx: usize) -> Option<Self::Entity>;
//     fn with_capacity(cap: usize) -> Self;
//     fn add(&mut self, entity: Self::Entity);
//     fn len(&self) -> usize;
//     fn iter_columns(&'a self) -> Self::IterColumns;

//     fn iter_rows(&self) -> Box<dyn Iterator<Item = Self::Entity> + '_> {
//         Box::new((0..self.len()).filter_map(|idx| self.index(idx)))
//     }

//     fn is_empty(&self) -> bool {
//         self.len() == 0
//     }

//     fn columns() -> &'static [FieldSchema<'static>] {
//         Self::Entity::SCHEMA
//     }
// }

//     Bool = 1,
//     Int8 = 2,
//     Int16 = 3,
//     Int32 = 4,
//     Int64 = 5,
//     Float = 10,
//     Double = 11,
//     String = 20,
//     /// variable-length strings with a specified maximum length
//     VarChar = 21,
//     BinaryVector = 100,
//     FloatVector = 101,

pub trait IntoFieldData {
    fn into_data_fields(self) -> Vec<FieldData>;
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub description: String,
    pub dtype: DataType,
    pub is_primary: bool,
    pub auto_id: bool,
    pub chunk_size: usize,
    pub dim: i64,                      // only for BinaryVector and FloatVector
    pub max_length: i32,               // only for VarChar
    pub enable_analyzer: Option<bool>, // for BM25 tokenizer
    pub enable_match: Option<bool>,    // for BM25 match
}

impl FieldSchema {
    pub const fn const_default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            dtype: DataType::None,
            is_primary: false,
            auto_id: false,
            chunk_size: 0,
            dim: 0,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }
}

impl Default for FieldSchema {
    fn default() -> Self {
        Self::const_default()
    }
}

impl From<schema::FieldSchema> for FieldSchema {
    fn from(fld: schema::FieldSchema) -> Self {
        let dim: i64 = fld
            .type_params
            .iter()
            .find(|k| &k.key == "dim")
            .and_then(|x| x.value.parse().ok())
            .unwrap_or(1);

        let dtype = DataType::from_i32(fld.data_type).unwrap();

        FieldSchema {
            name: fld.name,
            description: fld.description,
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
            enable_analyzer: None,
            enable_match: None,
        }
    }
}

impl FieldSchema {
    pub fn new_bool(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Bool,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_int8(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Int8,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_int16(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Int16,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_int32(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Int32,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_int64(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Int64,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_primary_int64(name: &str, description: &str, auto_id: bool) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Int64,
            is_primary: true,
            auto_id,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_primary_varchar(
        name: &str,
        description: &str,
        auto_id: bool,
        max_length: i32,
    ) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::VarChar,
            is_primary: true,
            auto_id,
            max_length,
            chunk_size: 1,
            dim: 1,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_float(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Float,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_double(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::Double,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_string(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::String,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_varchar(name: &str, description: &str, max_length: i32) -> Self {
        if max_length <= 0 {
            panic!("max_length should be positive");
        }

        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::VarChar,
            max_length,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_binary_vector(name: &str, description: &str, dim: i64) -> Self {
        if dim <= 0 {
            panic!("dim should be positive");
        }

        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::BinaryVector,
            chunk_size: dim as usize / 8,
            dim,
            is_primary: false,
            auto_id: false,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_float_vector(name: &str, description: &str, dim: i64) -> Self {
        if dim <= 0 {
            panic!("dim should be positive");
        }

        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::FloatVector,
            chunk_size: dim as usize,
            dim,
            is_primary: false,
            auto_id: false,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn new_sparse_float_vector(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            dtype: DataType::SparseFloatVector,
            chunk_size: 0,
            dim: 0,
            is_primary: false,
            auto_id: false,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }
}

impl From<FieldSchema> for schema::FieldSchema {
    fn from(fld: FieldSchema) -> schema::FieldSchema {
        let mut params = match fld.dtype {
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

        if let Some(enable_analyzer) = fld.enable_analyzer {
            params.push(KeyValuePair {
                key: "enable_analyzer".to_string(),
                value: enable_analyzer.to_string(),
            });
        }

        if let Some(enable_match) = fld.enable_match {
            params.push(KeyValuePair {
                key: "enable_match".to_string(),
                value: enable_match.to_string(),
            });
        }

        schema::FieldSchema {
            field_id: 0,
            name: fld.name.into(),
            is_primary_key: fld.is_primary,
            description: fld.description,
            data_type: fld.dtype as i32,
            type_params: params,
            index_params: Vec::new(),
            auto_id: fld.auto_id,
            state: FieldState::FieldCreated as _,
            element_type: 0,
            default_value: None,
            is_dynamic: false,
            is_partition_key: false,
            is_clustering_key: false,
            nullable: false,
            is_function_output: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldSchemaBuilder {
    name: String,
    description: String,
    dtype: DataType,
    chunk_size: usize,
    dim: i64,
    is_primary: bool,
    auto_id: bool,
    max_length: i32,
    enable_analyzer: Option<bool>,
    enable_match: Option<bool>,
}

impl FieldSchemaBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            dtype: DataType::None,
            is_primary: false,
            auto_id: false,
            chunk_size: 1,
            dim: 1,
            max_length: 0,
            enable_analyzer: None,
            enable_match: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_owned();
        self
    }

    pub fn with_dtype(mut self, dtype: DataType) -> Self {
        self.dtype = dtype;
        self
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    pub fn with_dim(mut self, dim: i64) -> Self {
        self.dim = dim;
        self
    }

    pub fn with_primary(mut self, is_primary: bool) -> Self {
        self.is_primary = is_primary;
        self
    }

    pub fn with_auto_id(mut self, auto_id: bool) -> Self {
        self.auto_id = auto_id;
        self
    }

    pub fn with_max_length(mut self, max_length: i32) -> Self {
        self.max_length = max_length;
        self
    }

    pub fn enable_analyzer(mut self, enable_analyzer: bool) -> Self {
        self.enable_analyzer = Some(enable_analyzer);
        self
    }

    pub fn with_enable_match(mut self, enable_match: bool) -> Self {
        self.enable_match = Some(enable_match);
        self
    }

    pub fn build(self) -> FieldSchema {
        FieldSchema {
            name: self.name,
            description: self.description,
            dtype: self.dtype,
            is_primary: self.is_primary,
            auto_id: self.auto_id,
            chunk_size: self.chunk_size,
            dim: self.dim,
            max_length: self.max_length,
            enable_analyzer: self.enable_analyzer,
            enable_match: self.enable_match,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionSchema {
    pub name: String,
    pub typ: FunctionType,
    pub input_field_names: Vec<String>,
    pub output_field_names: Vec<String>,
}

impl From<FunctionSchema> for schema::FunctionSchema {
    fn from(value: FunctionSchema) -> Self {
        Self {
            name: value.name,
            id: 0,
            description: "".into(),
            r#type: Into::<schema::FunctionType>::into(value.typ) as i32,
            input_field_names: value.input_field_names,
            input_field_ids: Vec::new(),
            output_field_names: value.output_field_names,
            output_field_ids: Vec::new(),
            params: Vec::new(),
        }
    }
}

impl From<schema::FunctionSchema> for FunctionSchema {
    fn from(value: schema::FunctionSchema) -> Self {
        Self {
            name: value.name,
            typ: schema::FunctionType::from_i32(value.r#type).unwrap().into(),
            input_field_names: value.input_field_names,
            output_field_names: value.output_field_names,
        }
    }
}

pub struct FunctionSchemaBuilder {
    pub name: String,
    pub typ: FunctionType,
    pub input_field_names: Vec<String>,
    pub output_field_names: Vec<String>,
}

impl FunctionSchemaBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            typ: FunctionType::Unknown,
            input_field_names: Vec::new(),
            output_field_names: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_typ(mut self, typ: FunctionType) -> Self {
        self.typ = typ;
        self
    }

    pub fn with_input_field_names(mut self, input_field_names: Vec<String>) -> Self {
        self.input_field_names = input_field_names;
        self
    }

    pub fn with_output_field_names(mut self, output_field_names: Vec<String>) -> Self {
        self.output_field_names = output_field_names;
        self
    }

    pub fn build(self) -> FunctionSchema {
        FunctionSchema {
            name: self.name,
            typ: self.typ,
            input_field_names: self.input_field_names,
            output_field_names: self.output_field_names,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) fields: Vec<FieldSchema>,
    pub(crate) enable_dynamic_field: bool,
    pub(crate) functions: Vec<FunctionSchema>,
}

impl CollectionSchema {
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn auto_id(&self) -> bool {
        self.fields.iter().any(|x| x.auto_id)
    }

    pub fn primary_column(&self) -> Option<&FieldSchema> {
        self.fields.iter().find(|s| s.is_primary)
    }

    pub fn validate(&self) -> Result<()> {
        self.primary_column().ok_or_else(|| Error::NoPrimaryKey)?;
        // TODO addidtional schema checks need to be added here
        Ok(())
    }

    pub fn get_field<S>(&self, name: S) -> Option<&FieldSchema>
    where
        S: AsRef<str>,
    {
        let name = name.as_ref();
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn is_valid_vector_field(&self, field_name: &str) -> Result<()> {
        for f in &self.fields {
            if f.name == field_name {
                if f.dtype == DataType::BinaryVector || f.dtype == DataType::FloatVector {
                    return Ok(());
                } else {
                    return Err(error::Error::from(Error::NotVectorField(
                        field_name.to_owned(),
                    )));
                }
            }
        }
        return Err(error::Error::from(Error::NoSuchKey(field_name.to_owned())));
    }
}

impl From<CollectionSchema> for schema::CollectionSchema {
    fn from(col: CollectionSchema) -> Self {
        schema::CollectionSchema {
            name: col.name.to_string(),
            auto_id: col.auto_id(),
            description: col.description,
            fields: col.fields.into_iter().map(Into::into).collect(),
            enable_dynamic_field: col.enable_dynamic_field,
            properties: Vec::new(),
            db_name: "".to_string(),
            functions: col.functions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<schema::CollectionSchema> for CollectionSchema {
    fn from(v: schema::CollectionSchema) -> Self {
        CollectionSchema {
            fields: v.fields.into_iter().map(Into::into).collect(),
            name: v.name,
            description: v.description,
            enable_dynamic_field: v.enable_dynamic_field,
            functions: v.functions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionSchemaBuilder {
    name: String,
    description: String,
    inner: Vec<FieldSchema>,
    enable_dynamic_field: bool,
    functions: Vec<FunctionSchema>,
}

impl CollectionSchemaBuilder {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            inner: Vec::new(),
            enable_dynamic_field: false,
            functions: Vec::new(),
        }
    }

    pub fn add_field(&mut self, schema: FieldSchema) -> &mut Self {
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

    pub fn enable_dynamic_field(&mut self) -> &mut Self {
        self.enable_dynamic_field = true;
        self
    }

    pub fn add_function(&mut self, schema: FunctionSchema) -> &mut Self {
        self.functions.push(schema);
        self
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

        let this = std::mem::replace(self, CollectionSchemaBuilder::new("".into(), ""));

        Ok(CollectionSchema {
            fields: this.inner.into(),
            name: this.name,
            description: this.description,
            enable_dynamic_field: this.enable_dynamic_field,
            functions: this.functions.into(),
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

    #[error("field {0:?} must be a vector field")]
    NotVectorField(String),
}
