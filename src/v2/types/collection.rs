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

//! Collection schemas, fields, defaults, replicas, and load-state types.

use super::common::{pairs, ConsistencyLevel, DataType, Function};
use crate::proto::{common, milvus, schema};
use crate::v2::error::{Error, Result};
use prost::Message;
use std::collections::{HashMap, HashSet};

const MAX_ARRAY_CAPACITY: u32 = 4_096;

///////////////////////////////////////////////////////////////////////////////
// LoadState
///////////////////////////////////////////////////////////////////////////////
/// Load state of a collection or partition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadState {
    #[default]
    /// Represents the NotExist case.
    NotExist,
    /// Represents the NotLoad case.
    NotLoad,
    /// Represents the Loading case.
    Loading,
    /// Represents the Loaded case.
    Loaded,
    /// Represents the Unknown case.
    Unknown,
}

impl LoadState {
    pub(crate) fn from_proto(value: i32) -> Self {
        match common::LoadState::try_from(value).ok() {
            Some(common::LoadState::NotExist) => Self::NotExist,
            Some(common::LoadState::NotLoad) => Self::NotLoad,
            Some(common::LoadState::Loading) => Self::Loading,
            Some(common::LoadState::Loaded) => Self::Loaded,
            _ => Self::Unknown,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DefaultValue
///////////////////////////////////////////////////////////////////////////////
/// Default value assigned when a nullable or defaulted field is omitted.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DefaultValue {
    /// Represents the Bool case.
    Bool(bool),
    /// Represents the Int32 case.
    Int32(i32),
    /// Represents the Int64 case.
    Int64(i64),
    /// Represents the Float case.
    Float(f32),
    /// Represents the Double case.
    Double(f64),
    /// Represents the String case.
    String(String),
    /// Represents the Bytes case.
    Bytes(Vec<u8>),
    /// Represents the TimestampTz case.
    TimestampTz(i64),
}

impl DefaultValue {
    fn into_proto(self) -> schema::ValueField {
        use schema::value_field::Data;
        let data = match self {
            Self::Bool(v) => Data::BoolData(v),
            Self::Int32(v) => Data::IntData(v),
            Self::Int64(v) => Data::LongData(v),
            Self::Float(v) => Data::FloatData(v),
            Self::Double(v) => Data::DoubleData(v),
            Self::String(v) => Data::StringData(v),
            Self::Bytes(v) => Data::BytesData(v),
            Self::TimestampTz(v) => Data::TimestamptzData(v),
        };
        schema::ValueField {
            data: Some(data),
            ..Default::default()
        }
    }

    fn from_proto(value: schema::ValueField) -> Result<Self> {
        use schema::value_field::Data;
        Ok(
            match value.data.ok_or_else(|| {
                Error::MalformedResponse("collection field default value has no data".into())
            })? {
                Data::BoolData(v) => Self::Bool(v),
                Data::IntData(v) => Self::Int32(v),
                Data::LongData(v) => Self::Int64(v),
                Data::FloatData(v) => Self::Float(v),
                Data::DoubleData(v) => Self::Double(v),
                Data::StringData(v) => Self::String(v),
                Data::BytesData(v) => Self::Bytes(v),
                Data::TimestamptzData(v) => Self::TimestampTz(v),
                Data::DateData(_) | Data::TimeData(_) => {
                    return Err(Error::conversion(
                        "unsupported default value data type Date/Time",
                    ))
                }
            },
        )
    }

    fn matches_data_type(&self, data_type: DataType) -> bool {
        matches!(
            (self, data_type),
            (Self::Bool(_), DataType::Bool)
                | (
                    Self::Int32(_),
                    DataType::Int8 | DataType::Int16 | DataType::Int32
                )
                | (Self::Int64(_), DataType::Int64)
                | (Self::Float(_), DataType::Float)
                | (Self::Double(_), DataType::Double)
                | (
                    Self::String(_),
                    DataType::VarChar | DataType::Geometry | DataType::Timestamptz
                )
                | (Self::TimestampTz(_), DataType::Timestamptz)
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// FieldSchema
///////////////////////////////////////////////////////////////////////////////
/// Schema definition for a collection field.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FieldSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) data_type: DataType,
    pub(crate) element_type: Option<DataType>,
    pub(crate) is_primary_key: bool,
    pub(crate) auto_id: bool,
    pub(crate) is_partition_key: bool,
    pub(crate) is_clustering_key: bool,
    pub(crate) nullable: bool,
    pub(crate) default_value: Option<DefaultValue>,
    pub(crate) type_params: HashMap<String, String>,
    pub(crate) index_params: HashMap<String, String>,
    pub(crate) external_field: String,
}

impl FieldSchema {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            data_type: DataType::Unknown,
            element_type: None,
            is_primary_key: false,
            auto_id: false,
            is_partition_key: false,
            is_clustering_key: false,
            nullable: false,
            default_value: None,
            type_params: HashMap::new(),
            index_params: HashMap::new(),
            external_field: String::new(),
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Returns the configured name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the data type and returns the updated value.
    pub fn data_type(mut self, value: DataType) -> Self {
        self.data_type = value;
        self
    }

    /// Sets the data type and returns this value for further mutation.
    pub fn set_data_type(&mut self, value: DataType) -> &mut Self {
        self.data_type = value;
        self
    }

    /// Returns the configured data type.
    pub fn get_data_type(&self) -> DataType {
        self.data_type
    }

    /// Sets the element type and returns the updated value.
    pub fn element_type(mut self, value: DataType) -> Self {
        self.element_type = Some(value);
        self
    }

    /// Sets the element type and returns this value for further mutation.
    pub fn set_element_type(&mut self, value: DataType) -> &mut Self {
        self.element_type = Some(value);
        self
    }

    /// Returns the configured element type.
    pub fn get_element_type(&self) -> Option<DataType> {
        self.element_type
    }

    /// Sets the primary key and returns the updated value.
    pub fn primary_key(mut self, value: bool) -> Self {
        self.is_primary_key = value;
        self
    }

    /// Sets the primary key and returns this value for further mutation.
    pub fn set_primary_key(&mut self, value: bool) -> &mut Self {
        self.is_primary_key = value;
        self
    }

    /// Returns whether primary key.
    pub fn is_primary_key(&self) -> bool {
        self.is_primary_key
    }

    /// Sets the auto id and returns the updated value.
    pub fn auto_id(mut self, value: bool) -> Self {
        self.auto_id = value;
        self
    }

    /// Sets the auto id and returns this value for further mutation.
    pub fn set_auto_id(&mut self, value: bool) -> &mut Self {
        self.auto_id = value;
        self
    }

    /// Returns whether auto id.
    pub fn is_auto_id(&self) -> bool {
        self.auto_id
    }

    /// Sets the partition key and returns the updated value.
    pub fn partition_key(mut self, enabled: bool) -> Self {
        self.is_partition_key = enabled;
        self
    }

    /// Sets the partition key and returns this value for further mutation.
    pub fn set_partition_key(&mut self, enabled: bool) -> &mut Self {
        self.is_partition_key = enabled;
        self
    }

    /// Returns whether partition key.
    pub fn is_partition_key(&self) -> bool {
        self.is_partition_key
    }

    /// Sets the clustering key and returns the updated value.
    pub fn clustering_key(mut self, enabled: bool) -> Self {
        self.is_clustering_key = enabled;
        self
    }

    /// Sets the clustering key and returns this value for further mutation.
    pub fn set_clustering_key(&mut self, enabled: bool) -> &mut Self {
        self.is_clustering_key = enabled;
        self
    }

    /// Returns whether clustering key.
    pub fn is_clustering_key(&self) -> bool {
        self.is_clustering_key
    }

    /// Sets the nullable and returns the updated value.
    pub fn nullable(mut self, enabled: bool) -> Self {
        self.nullable = enabled;
        self
    }

    /// Sets the nullable and returns this value for further mutation.
    pub fn set_nullable(&mut self, enabled: bool) -> &mut Self {
        self.nullable = enabled;
        self
    }

    /// Returns whether nullable.
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Sets the default value and returns the updated value.
    pub fn default_value(mut self, value: DefaultValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Sets the default value and returns this value for further mutation.
    pub fn set_default_value(&mut self, value: DefaultValue) -> &mut Self {
        self.default_value = Some(value);
        self
    }

    /// Returns the configured default value.
    pub fn get_default_value(&self) -> &Option<DefaultValue> {
        &self.default_value
    }

    /// Sets the type params and returns the updated value.
    pub fn type_params(mut self, value: HashMap<String, String>) -> Self {
        for (key, value) in value {
            self.type_params.entry(key).or_insert(value);
        }
        self
    }

    /// Sets the type params and returns this value for further mutation.
    pub fn set_type_params(&mut self, value: HashMap<String, String>) -> &mut Self {
        for (key, value) in value {
            self.type_params.entry(key).or_insert(value);
        }
        self
    }

    /// Returns the configured type params.
    pub fn get_type_params(&self) -> &HashMap<String, String> {
        &self.type_params
    }

    /// Sets the index params and returns the updated value.
    pub fn index_params(mut self, value: HashMap<String, String>) -> Self {
        self.index_params = value;
        self
    }

    /// Sets the index params and returns this value for further mutation.
    pub fn set_index_params(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.index_params = value;
        self
    }

    /// Returns the configured index params.
    pub fn get_index_params(&self) -> &HashMap<String, String> {
        &self.index_params
    }

    /// Sets the external field mapping name and returns the updated value.
    pub fn external_field(mut self, value: impl Into<String>) -> Self {
        self.external_field = value.into();
        self
    }

    /// Sets the external field mapping name and returns this value for further mutation.
    pub fn set_external_field(&mut self, value: impl Into<String>) -> &mut Self {
        self.external_field = value.into();
        self
    }

    /// Returns the external field mapping name.
    pub fn get_external_field(&self) -> &str {
        &self.external_field
    }

    /// Sets the dimension and returns the updated value.
    pub fn dimension(mut self, dimension: u32) -> Self {
        if dimension > 0 {
            self.type_params.insert("dim".into(), dimension.to_string());
        }
        self
    }

    /// Sets the dimension and returns this value for further mutation.
    pub fn set_dimension(&mut self, dimension: u32) -> &mut Self {
        if dimension > 0 {
            self.type_params.insert("dim".into(), dimension.to_string());
        }
        self
    }

    /// Returns the configured dimension.
    pub fn get_dimension(&self) -> u32 {
        self.type_params
            .get("dim")
            .and_then(|value| value.parse().ok())
            .unwrap_or_default()
    }

    /// Sets the max length and returns the updated value.
    pub fn max_length(mut self, max_length: u32) -> Self {
        self.type_params
            .insert("max_length".into(), max_length.to_string());
        self
    }

    /// Sets the max length and returns this value for further mutation.
    pub fn set_max_length(&mut self, max_length: u32) -> &mut Self {
        self.type_params
            .insert("max_length".into(), max_length.to_string());
        self
    }

    /// Returns the configured max length.
    pub fn get_max_length(&self) -> u32 {
        self.type_params
            .get("max_length")
            .map_or(65_535, |value| value.parse().unwrap_or_default())
    }

    /// Sets the max capacity and returns the updated value.
    pub fn max_capacity(mut self, max_capacity: u32) -> Self {
        self.type_params
            .insert("max_capacity".into(), max_capacity.to_string());
        self
    }

    /// Sets the max capacity and returns this value for further mutation.
    pub fn set_max_capacity(&mut self, max_capacity: u32) -> &mut Self {
        self.type_params
            .insert("max_capacity".into(), max_capacity.to_string());
        self
    }

    /// Returns the configured max capacity.
    pub fn get_max_capacity(&self) -> u32 {
        self.type_params
            .get("max_capacity")
            .and_then(|value| value.parse().ok())
            .unwrap_or_default()
    }

    /// Sets the enable analyzer and returns the updated value.
    pub fn enable_analyzer(mut self, enabled: bool) -> Self {
        self.type_params
            .insert("enable_analyzer".into(), enabled.to_string());
        self
    }

    /// Sets the enable analyzer and returns this value for further mutation.
    pub fn set_enable_analyzer(&mut self, enabled: bool) -> &mut Self {
        self.type_params
            .insert("enable_analyzer".into(), enabled.to_string());
        self
    }

    /// Returns whether analyzer enabled.
    pub fn is_analyzer_enabled(&self) -> bool {
        self.type_params
            .get("enable_analyzer")
            .is_some_and(|value| value == "true")
    }

    /// Sets the enable match and returns the updated value.
    pub fn enable_match(mut self, enabled: bool) -> Self {
        self.type_params
            .insert("enable_match".into(), enabled.to_string());
        self
    }

    /// Sets the enable match and returns this value for further mutation.
    pub fn set_enable_match(&mut self, enabled: bool) -> &mut Self {
        self.type_params
            .insert("enable_match".into(), enabled.to_string());
        self
    }

    /// Returns whether match enabled.
    pub fn is_match_enabled(&self) -> bool {
        self.type_params
            .get("enable_match")
            .is_some_and(|value| value == "true")
    }

    /// Sets the analyzer params and returns the updated value.
    pub fn analyzer_params(mut self, value: serde_json::Value) -> Self {
        self.type_params
            .insert("analyzer_params".into(), value.to_string());
        self
    }

    /// Sets the analyzer params and returns this value for further mutation.
    pub fn set_analyzer_params(&mut self, value: serde_json::Value) -> &mut Self {
        self.type_params
            .insert("analyzer_params".into(), value.to_string());
        self
    }

    /// Returns the configured analyzer params.
    pub fn get_analyzer_params(&self) -> Result<serde_json::Value> {
        self.get_json_type_param("analyzer_params")
    }

    /// Sets the multi analyzer params and returns the updated value.
    pub fn multi_analyzer_params(mut self, value: serde_json::Value) -> Self {
        self.type_params
            .insert("multi_analyzer_params".into(), value.to_string());
        self
    }

    /// Sets the multi analyzer params and returns this value for further mutation.
    pub fn set_multi_analyzer_params(&mut self, value: serde_json::Value) -> &mut Self {
        self.type_params
            .insert("multi_analyzer_params".into(), value.to_string());
        self
    }

    /// Returns the configured multi analyzer params.
    pub fn get_multi_analyzer_params(&self) -> Result<serde_json::Value> {
        self.get_json_type_param("multi_analyzer_params")
    }

    /// Sets the type param and returns the updated value.
    pub fn type_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.type_params.entry(key.into()).or_insert(value.into());
        self
    }

    /// Sets the index param and returns the updated value.
    pub fn index_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.index_params.insert(key.into(), value.into());
        self
    }

    fn get_json_type_param(&self, key: &str) -> Result<serde_json::Value> {
        self.type_params.get(key).map_or_else(
            || Ok(serde_json::Value::Null),
            |value| serde_json::from_str(value).map_err(Into::into),
        )
    }

    pub(crate) fn into_proto(self) -> schema::FieldSchema {
        schema::FieldSchema {
            name: self.name,
            description: self.description,
            data_type: self.data_type.into_proto() as i32,
            element_type: self
                .element_type
                .map(DataType::into_proto)
                .unwrap_or(schema::DataType::None) as i32,
            is_primary_key: self.is_primary_key,
            auto_id: self.auto_id,
            is_partition_key: self.is_partition_key,
            is_clustering_key: self.is_clustering_key,
            nullable: self.nullable,
            default_value: self.default_value.map(DefaultValue::into_proto),
            type_params: pairs(self.type_params),
            index_params: pairs(self.index_params),
            external_field: self.external_field,
            ..Default::default()
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.is_primary_key && !matches!(self.data_type, DataType::Int64 | DataType::VarChar) {
            return Err(Error::validation(
                self.name.clone(),
                "primary key field type must be Int64 or VarChar".into(),
            ));
        }
        if let Some(default) = &self.default_value {
            if self.is_primary_key {
                return Err(Error::validation(
                    self.name.clone(),
                    "primary key fields cannot have a default value".into(),
                ));
            }
            if !default.matches_data_type(self.data_type) {
                return Err(Error::validation(
                    self.name.clone(),
                    format!(
                        "default value is not supported for or does not match {:?}",
                        self.data_type
                    ),
                ));
            }
            match (default, self.data_type) {
                (DefaultValue::Int32(value), DataType::Int8)
                    if *value < i8::MIN as i32 || *value > i8::MAX as i32 =>
                {
                    return Err(Error::validation(
                        self.name.clone(),
                        "default value is outside Int8 range".into(),
                    ));
                }
                (DefaultValue::Int32(value), DataType::Int16)
                    if *value < i16::MIN as i32 || *value > i16::MAX as i32 =>
                {
                    return Err(Error::validation(
                        self.name.clone(),
                        "default value is outside Int16 range".into(),
                    ));
                }
                _ => {}
            }
        }
        if matches!(
            self.data_type,
            DataType::FloatVector
                | DataType::BinaryVector
                | DataType::Float16Vector
                | DataType::BFloat16Vector
                | DataType::Int8Vector
        ) && self.get_dimension() == 0
        {
            return Err(Error::validation(
                self.name.clone(),
                "dense vector fields require a positive dimension".into(),
            ));
        }
        if self.data_type == DataType::Array && self.element_type.is_none() {
            return Err(Error::validation(
                self.name.clone(),
                "array fields require an element type".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn encode(self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::new();
        self.into_proto().encode(&mut bytes)?;
        Ok(bytes)
    }

    pub(crate) fn from_proto(value: schema::FieldSchema) -> Result<Self> {
        let field_name = value.name.clone();
        let proto_data_type = schema::DataType::try_from(value.data_type).map_err(|_| {
            Error::MalformedResponse(format!(
                "collection field {field_name:?} has unknown data type {}",
                value.data_type
            ))
        })?;
        let data_type = DataType::try_from_proto(proto_data_type).map_err(|_| {
            Error::MalformedResponse(format!(
                "collection field {field_name:?} has unsupported data type {proto_data_type:?}"
            ))
        })?;
        if data_type == DataType::Unknown {
            return Err(Error::MalformedResponse(format!(
                "collection field {field_name:?} has no data type"
            )));
        }
        let element_type = match schema::DataType::try_from(value.element_type) {
            Ok(schema::DataType::None) => None,
            Ok(proto_element_type) => {
                let element_type = DataType::try_from_proto(proto_element_type).map_err(|_| {
                    Error::MalformedResponse(format!(
                        "collection field {field_name:?} has unsupported element type {proto_element_type:?}"
                    ))
                })?;
                Some(element_type)
            }
            Err(_) => {
                return Err(Error::MalformedResponse(format!(
                    "collection field {field_name:?} has unknown element type {}",
                    value.element_type
                )));
            }
        };
        let default_value = value
            .default_value
            .map(DefaultValue::from_proto)
            .transpose()?;
        let field = Self {
            name: value.name,
            description: value.description,
            data_type,
            element_type,
            is_primary_key: value.is_primary_key,
            auto_id: value.auto_id,
            is_partition_key: value.is_partition_key,
            is_clustering_key: value.is_clustering_key,
            nullable: value.nullable,
            default_value,
            type_params: value
                .type_params
                .into_iter()
                .map(|v| (v.key, v.value))
                .collect(),
            index_params: value
                .index_params
                .into_iter()
                .map(|v| (v.key, v.value))
                .collect(),
            external_field: value.external_field,
        };
        if field.name.is_empty() {
            return Err(Error::MalformedResponse(
                "collection schema contains a field with an empty name".into(),
            ));
        }
        field.validate().map_err(|error| {
            Error::MalformedResponse(format!(
                "collection field {:?} is invalid: {error}",
                field.name
            ))
        })?;
        Ok(field)
    }
}

///////////////////////////////////////////////////////////////////////////////
// StructFieldSchema
///////////////////////////////////////////////////////////////////////////////
/// Schema definition for a field nested inside a struct field.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct StructFieldSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) max_capacity: u32,
    pub(crate) fields: Vec<FieldSchema>,
    pub(crate) nullable: bool,
}

impl StructFieldSchema {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            max_capacity: 0,
            fields: Vec::new(),
            nullable: false,
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Returns the configured name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the max capacity and returns the updated value.
    pub fn max_capacity(mut self, value: u32) -> Self {
        self.max_capacity = value;
        self
    }

    /// Sets the max capacity and returns this value for further mutation.
    pub fn set_max_capacity(&mut self, value: u32) -> &mut Self {
        self.max_capacity = value;
        self
    }

    /// Returns the configured max capacity.
    pub fn get_max_capacity(&self) -> u32 {
        self.max_capacity
    }

    /// Sets the fields and returns the updated value.
    pub fn fields(mut self, value: Vec<FieldSchema>) -> Self {
        self.fields = value;
        self
    }

    /// Sets the fields and returns this value for further mutation.
    pub fn set_fields(&mut self, value: Vec<FieldSchema>) -> &mut Self {
        self.fields = value;
        self
    }

    /// Returns the configured fields.
    pub fn get_fields(&self) -> &[FieldSchema] {
        &self.fields
    }

    /// Adds one add field to the existing values.
    pub fn add_field(mut self, field: FieldSchema) -> Self {
        self.fields.push(field);
        self
    }

    /// Sets whether the struct field is nullable and returns the updated value.
    pub fn nullable(mut self, value: bool) -> Self {
        self.nullable = value;
        self
    }

    /// Sets whether the struct field is nullable and returns this value for further mutation.
    pub fn set_nullable(&mut self, value: bool) -> &mut Self {
        self.nullable = value;
        self
    }

    /// Returns whether the struct field is nullable.
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::validation(
                "struct_field.name".into(),
                "must be specified".into(),
            ));
        }
        if self.max_capacity == 0 || self.max_capacity > MAX_ARRAY_CAPACITY {
            return Err(Error::validation(
                self.name.clone(),
                format!("struct field max capacity must be within [1, {MAX_ARRAY_CAPACITY}]"),
            ));
        }
        if self.fields.is_empty() {
            return Err(Error::validation(
                self.name.clone(),
                "struct fields require at least one sub-field".into(),
            ));
        }

        let mut names = HashSet::with_capacity(self.fields.len());
        for field in &self.fields {
            if field.name.is_empty() {
                return Err(Error::validation(
                    self.name.clone(),
                    "struct sub-field names must be specified".into(),
                ));
            }
            if !names.insert(field.name.as_str()) {
                return Err(Error::validation(
                    self.name.clone(),
                    format!("duplicate struct sub-field {:?}", field.name),
                ));
            }
            if !matches!(
                field.data_type,
                DataType::Bool
                    | DataType::Int8
                    | DataType::Int16
                    | DataType::Int32
                    | DataType::Int64
                    | DataType::Float
                    | DataType::Double
                    | DataType::VarChar
                    | DataType::FloatVector
                    | DataType::BinaryVector
                    | DataType::Float16Vector
                    | DataType::BFloat16Vector
                    | DataType::Int8Vector
            ) {
                return Err(Error::validation(
                    field.name.clone(),
                    format!(
                        "data type {:?} is not supported for a struct sub-field",
                        field.data_type
                    ),
                ));
            }
            if field.is_primary_key {
                return Err(Error::validation(
                    field.name.clone(),
                    "struct sub-fields cannot be primary keys".into(),
                ));
            }
            if field.auto_id {
                return Err(Error::validation(
                    field.name.clone(),
                    "struct sub-fields cannot use auto ID".into(),
                ));
            }
            if field.is_partition_key {
                return Err(Error::validation(
                    field.name.clone(),
                    "struct sub-fields cannot be partition keys".into(),
                ));
            }
            if field.is_clustering_key {
                return Err(Error::validation(
                    field.name.clone(),
                    "struct sub-fields cannot be clustering keys".into(),
                ));
            }
            if !self.nullable && field.nullable {
                return Err(Error::validation(
                    field.name.clone(),
                    "sub-fields of a non-nullable struct cannot be nullable".into(),
                ));
            }
            if field.default_value.is_some() {
                return Err(Error::validation(
                    field.name.clone(),
                    "struct sub-fields cannot have default values".into(),
                ));
            }
            field.validate()?;
        }
        Ok(())
    }

    pub(crate) fn into_proto(self) -> schema::StructArrayFieldSchema {
        let max_capacity = self.max_capacity.to_string();
        schema::StructArrayFieldSchema {
            field_id: 0,
            name: self.name,
            description: self.description,
            fields: self
                .fields
                .into_iter()
                .map(|field| {
                    let data_type = field.data_type;
                    let mut value = field.into_proto();
                    value.data_type = if data_type.is_vector() {
                        schema::DataType::ArrayOfVector
                    } else {
                        schema::DataType::Array
                    } as i32;
                    value.element_type = data_type.into_proto() as i32;
                    value.type_params.retain(|pair| pair.key != "max_capacity");
                    value.type_params.push(common::KeyValuePair {
                        key: "max_capacity".into(),
                        value: max_capacity.clone(),
                    });
                    value
                })
                .collect(),
            type_params: Vec::new(),
            nullable: self.nullable,
            ..Default::default()
        }
    }

    pub(crate) fn from_proto(value: schema::StructArrayFieldSchema) -> Result<Self> {
        let struct_name = value.name.clone();
        let capacities = value
            .fields
            .iter()
            .map(|field| {
                let capacity = field
                    .type_params
                    .iter()
                    .find(|pair| pair.key == "max_capacity")
                    .ok_or_else(|| {
                        Error::MalformedResponse(format!(
                            "struct field {struct_name:?} sub-field {:?} has no max_capacity",
                            field.name
                        ))
                    })?;
                capacity.value.parse::<u32>().map_err(|_| {
                    Error::MalformedResponse(format!(
                        "struct field {struct_name:?} sub-field {:?} has invalid max_capacity {:?}",
                        field.name, capacity.value
                    ))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let max_capacity = capacities.first().copied().ok_or_else(|| {
            Error::MalformedResponse(format!("struct field {struct_name:?} has no sub-fields"))
        })?;
        if capacities.iter().any(|capacity| *capacity != max_capacity) {
            return Err(Error::MalformedResponse(format!(
                "struct field {struct_name:?} has inconsistent max_capacity values"
            )));
        }
        let field = Self {
            name: value.name,
            description: value.description,
            max_capacity,
            nullable: value.nullable,
            fields: value
                .fields
                .into_iter()
                .map(|mut field| {
                    field.data_type = field.element_type;
                    field.element_type = schema::DataType::None as i32;
                    field.type_params.retain(|pair| pair.key != "max_capacity");
                    FieldSchema::from_proto(field)
                })
                .collect::<Result<Vec<_>>>()?,
        };
        if field.name.is_empty() {
            return Err(Error::MalformedResponse(
                "collection schema contains a struct field with an empty name".into(),
            ));
        }
        if field.max_capacity == 0 {
            return Err(Error::MalformedResponse(format!(
                "struct field {:?} has a non-positive max_capacity",
                field.name
            )));
        }
        let mut names = HashSet::with_capacity(field.fields.len());
        for sub_field in &field.fields {
            if !names.insert(sub_field.name.as_str()) {
                return Err(Error::MalformedResponse(format!(
                    "struct field {:?} contains duplicate sub-field {:?}",
                    field.name, sub_field.name
                )));
            }
            if sub_field.is_primary_key
                || sub_field.auto_id
                || sub_field.is_partition_key
                || sub_field.is_clustering_key
                || sub_field.default_value.is_some()
            {
                return Err(Error::MalformedResponse(format!(
                    "struct field {:?} sub-field {:?} contains forbidden schema attributes",
                    field.name, sub_field.name
                )));
            }
        }
        Ok(field)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CollectionSchema
///////////////////////////////////////////////////////////////////////////////
/// Schema definition for a Milvus collection.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CollectionSchema {
    pub(crate) description: String,
    pub(crate) enable_dynamic_field: bool,
    pub(crate) fields: Vec<FieldSchema>,
    pub(crate) struct_fields: Vec<StructFieldSchema>,
    pub(crate) functions: Vec<Function>,
    pub(crate) properties: HashMap<String, String>,
    pub(crate) external_source: String,
    pub(crate) external_spec: Option<serde_json::Value>,
}

impl CollectionSchema {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            description: String::new(),
            enable_dynamic_field: true,
            fields: Vec::new(),
            struct_fields: Vec::new(),
            functions: Vec::new(),
            properties: HashMap::new(),
            external_source: String::new(),
            external_spec: None,
        }
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the enable dynamic field and returns the updated value.
    pub fn enable_dynamic_field(mut self, enabled: bool) -> Self {
        self.enable_dynamic_field = enabled;
        self
    }

    /// Sets the enable dynamic field and returns this value for further mutation.
    pub fn set_enable_dynamic_field(&mut self, enabled: bool) -> &mut Self {
        self.enable_dynamic_field = enabled;
        self
    }

    /// Returns whether dynamic field enabled.
    pub fn is_dynamic_field_enabled(&self) -> bool {
        self.enable_dynamic_field
    }

    /// Sets the fields and returns the updated value.
    pub fn fields(mut self, value: Vec<FieldSchema>) -> Self {
        self.fields = value;
        self
    }

    /// Sets the fields and returns this value for further mutation.
    pub fn set_fields(&mut self, value: Vec<FieldSchema>) -> &mut Self {
        self.fields = value;
        self
    }

    /// Returns the configured fields.
    pub fn get_fields(&self) -> &[FieldSchema] {
        &self.fields
    }

    /// Sets the struct fields and returns the updated value.
    pub fn struct_fields(mut self, value: Vec<StructFieldSchema>) -> Self {
        self.struct_fields = value;
        self
    }

    /// Sets the struct fields and returns this value for further mutation.
    pub fn set_struct_fields(&mut self, value: Vec<StructFieldSchema>) -> &mut Self {
        self.struct_fields = value;
        self
    }

    /// Returns the configured struct fields.
    pub fn get_struct_fields(&self) -> &[StructFieldSchema] {
        &self.struct_fields
    }

    /// Sets the functions and returns the updated value.
    pub fn functions(mut self, value: Vec<Function>) -> Self {
        self.functions = value;
        self
    }

    /// Sets the functions and returns this value for further mutation.
    pub fn set_functions(&mut self, value: Vec<Function>) -> &mut Self {
        self.functions = value;
        self
    }

    /// Returns the configured functions.
    pub fn get_functions(&self) -> &[Function] {
        &self.functions
    }

    /// Sets the properties and returns the updated value.
    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.properties = value;
        self
    }

    /// Sets the properties and returns this value for further mutation.
    pub fn set_properties(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.properties = value;
        self
    }

    /// Returns the configured properties.
    pub fn get_properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// Adds one add field to the existing values.
    pub fn add_field(mut self, field: FieldSchema) -> Self {
        self.fields.push(field);
        self
    }

    /// Adds one add struct field to the existing values.
    pub fn add_struct_field(mut self, field: StructFieldSchema) -> Self {
        self.struct_fields.push(field);
        self
    }

    /// Adds one add function to the existing values.
    pub fn add_function(mut self, function: impl Into<Function>) -> Self {
        self.functions.push(function.into());
        self
    }

    /// Sets the property and returns the updated value.
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Sets the external collection source path and returns the updated value.
    pub fn external_source(mut self, value: impl Into<String>) -> Self {
        self.external_source = value.into();
        self
    }

    /// Sets the external collection source path and returns this value for further mutation.
    pub fn set_external_source(&mut self, value: impl Into<String>) -> &mut Self {
        self.external_source = value.into();
        self
    }

    /// Returns the external collection source path.
    pub fn get_external_source(&self) -> &str {
        &self.external_source
    }

    /// Sets the external collection spec JSON and returns the updated value.
    pub fn external_spec(mut self, value: serde_json::Value) -> Self {
        self.external_spec = Some(value);
        self
    }

    /// Sets the external collection spec JSON and returns this value for further mutation.
    pub fn set_external_spec(&mut self, value: serde_json::Value) -> &mut Self {
        self.external_spec = Some(value);
        self
    }

    /// Returns the external collection spec JSON.
    pub fn get_external_spec(&self) -> Option<&serde_json::Value> {
        self.external_spec.as_ref()
    }

    pub(crate) fn to_proto(&self) -> schema::CollectionSchema {
        schema::CollectionSchema {
            description: self.description.clone(),
            fields: self
                .fields
                .clone()
                .into_iter()
                .map(FieldSchema::into_proto)
                .collect(),
            struct_array_fields: self
                .struct_fields
                .clone()
                .into_iter()
                .map(StructFieldSchema::into_proto)
                .collect(),
            enable_dynamic_field: self.enable_dynamic_field,
            properties: pairs(self.properties.clone()),
            functions: self
                .functions
                .clone()
                .into_iter()
                .map(Function::into_proto)
                .collect(),
            external_source: self.external_source.clone(),
            external_spec: self
                .external_spec
                .as_ref()
                .map(serde_json::Value::to_string)
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    pub(crate) fn encode_for_collection(
        &self,
        collection_name: &str,
        description: Option<&str>,
    ) -> Result<Vec<u8>> {
        for field in &self.fields {
            field.validate()?;
        }
        for struct_field in &self.struct_fields {
            struct_field.validate()?;
        }
        let mut schema = self.to_proto();
        schema.name = collection_name.to_owned();
        if let Some(description) = description {
            schema.description = description.to_owned();
        }
        let mut bytes = Vec::new();
        schema.encode(&mut bytes)?;
        Ok(bytes)
    }

    pub(crate) fn from_proto(value: schema::CollectionSchema) -> Result<Self> {
        let external_spec = if value.external_spec.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&value.external_spec).map_err(|error| {
                Error::MalformedResponse(format!(
                    "collection schema external_spec is not valid JSON: {error}"
                ))
            })?)
        };
        Ok(Self {
            description: value.description,
            enable_dynamic_field: value.enable_dynamic_field,
            fields: value
                .fields
                .into_iter()
                .map(FieldSchema::from_proto)
                .collect::<Result<Vec<_>>>()?,
            struct_fields: value
                .struct_array_fields
                .into_iter()
                .map(StructFieldSchema::from_proto)
                .collect::<Result<Vec<_>>>()?,
            functions: value
                .functions
                .into_iter()
                .map(Function::from_proto)
                .collect(),
            properties: value
                .properties
                .into_iter()
                .map(|v| (v.key, v.value))
                .collect(),
            external_source: value.external_source,
            external_spec,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// CollectionDesc
///////////////////////////////////////////////////////////////////////////////
/// Detailed collection metadata returned by describe operations.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CollectionDesc {
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) description: String,
    pub(crate) num_partitions: i64,
    pub(crate) field_names: Vec<String>,
    pub(crate) vector_field_names: Vec<String>,
    pub(crate) primary_field_name: String,
    pub(crate) enable_dynamic_field: bool,
    pub(crate) auto_id: bool,
    pub(crate) num_shards: i64,
    pub(crate) schema: CollectionSchema,
    pub(crate) collection_id: i64,
    pub(crate) aliases: Vec<String>,
    pub(crate) created_time: u64,
    pub(crate) created_utc_time: u64,
    pub(crate) update_time: u64,
    pub(crate) consistency_level: ConsistencyLevel,
    pub(crate) properties: HashMap<String, String>,
}

impl CollectionDesc {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            description: String::new(),
            num_partitions: 0,
            field_names: Vec::new(),
            vector_field_names: Vec::new(),
            primary_field_name: String::new(),
            enable_dynamic_field: true,
            auto_id: false,
            num_shards: 1,
            schema: CollectionSchema::new(),
            collection_id: 0,
            aliases: Vec::new(),
            created_time: 0,
            created_utc_time: 0,
            update_time: 0,
            consistency_level: ConsistencyLevel::default(),
            properties: HashMap::new(),
        }
    }

    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.database_name = value.into();
        self
    }

    /// Sets the database name and returns this value for further mutation.
    pub fn set_database_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.database_name = value.into();
        self
    }

    /// Returns the configured database name.
    pub fn get_database_name(&self) -> &str {
        &self.database_name
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.collection_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn set_collection_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.collection_name = value.into();
        self
    }

    /// Returns the configured collection name.
    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the num partitions and returns the updated value.
    pub fn num_partitions(mut self, value: i64) -> Self {
        self.num_partitions = value;
        self
    }

    /// Sets the num partitions and returns this value for further mutation.
    pub fn set_num_partitions(&mut self, value: i64) -> &mut Self {
        self.num_partitions = value;
        self
    }

    /// Returns the configured num partitions.
    pub fn get_num_partitions(&self) -> i64 {
        self.num_partitions
    }

    /// Sets the field names and returns the updated value.
    pub fn field_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the field names and returns this value for further mutation.
    pub fn set_field_names(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured field names.
    pub fn get_field_names(&self) -> &[String] {
        &self.field_names
    }

    /// Performs the vector field names operation.
    pub fn vector_field_names(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.vector_field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the vector field names and returns this value for further mutation.
    pub fn set_vector_field_names(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.vector_field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured vector field names.
    pub fn get_vector_field_names(&self) -> &[String] {
        &self.vector_field_names
    }

    /// Sets the primary field name and returns the updated value.
    pub fn primary_field_name(mut self, value: impl Into<String>) -> Self {
        self.primary_field_name = value.into();
        self
    }

    /// Sets the primary field name and returns this value for further mutation.
    pub fn set_primary_field_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.primary_field_name = value.into();
        self
    }

    /// Returns the configured primary field name.
    pub fn get_primary_field_name(&self) -> &str {
        &self.primary_field_name
    }

    /// Sets the enable dynamic field and returns the updated value.
    pub fn enable_dynamic_field(mut self, value: bool) -> Self {
        self.enable_dynamic_field = value;
        self
    }

    /// Sets the enable dynamic field and returns this value for further mutation.
    pub fn set_enable_dynamic_field(&mut self, value: bool) -> &mut Self {
        self.enable_dynamic_field = value;
        self
    }

    /// Returns whether dynamic field enabled.
    pub fn is_dynamic_field_enabled(&self) -> bool {
        self.enable_dynamic_field
    }

    /// Sets the auto id and returns the updated value.
    pub fn auto_id(mut self, value: bool) -> Self {
        self.auto_id = value;
        self
    }

    /// Sets the auto id and returns this value for further mutation.
    pub fn set_auto_id(&mut self, value: bool) -> &mut Self {
        self.auto_id = value;
        self
    }

    /// Returns the configured auto id.
    pub fn get_auto_id(&self) -> bool {
        self.auto_id
    }

    /// Sets the num shards and returns the updated value.
    pub fn num_shards(mut self, value: i64) -> Self {
        self.num_shards = value;
        self
    }

    /// Sets the num shards and returns this value for further mutation.
    pub fn set_num_shards(&mut self, value: i64) -> &mut Self {
        self.num_shards = value;
        self
    }

    /// Returns the configured num shards.
    pub fn get_num_shards(&self) -> i64 {
        self.num_shards
    }

    /// Sets the schema and returns the updated value.
    pub fn schema(mut self, value: CollectionSchema) -> Self {
        self.schema = value;
        self
    }

    /// Sets the schema and returns this value for further mutation.
    pub fn set_schema(&mut self, value: CollectionSchema) -> &mut Self {
        self.schema = value;
        self
    }

    /// Returns the configured schema.
    pub fn get_schema(&self) -> &CollectionSchema {
        &self.schema
    }

    /// Sets the collection id and returns the updated value.
    pub fn collection_id(mut self, value: i64) -> Self {
        self.collection_id = value;
        self
    }

    /// Sets the collection id and returns this value for further mutation.
    pub fn set_collection_id(&mut self, value: i64) -> &mut Self {
        self.collection_id = value;
        self
    }

    /// Returns the configured collection id.
    pub fn get_collection_id(&self) -> i64 {
        self.collection_id
    }

    /// Sets the aliases and returns the updated value.
    pub fn aliases(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.aliases = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the aliases and returns this value for further mutation.
    pub fn set_aliases(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.aliases = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured aliases.
    pub fn get_aliases(&self) -> &[String] {
        &self.aliases
    }

    /// Sets the created time and returns the updated value.
    pub fn created_time(mut self, value: u64) -> Self {
        self.created_time = value;
        self
    }

    /// Sets the created time and returns this value for further mutation.
    pub fn set_created_time(&mut self, value: u64) -> &mut Self {
        self.created_time = value;
        self
    }

    /// Returns the configured created time.
    pub fn get_created_time(&self) -> u64 {
        self.created_time
    }

    /// Sets the created utc time and returns the updated value.
    pub fn created_utc_time(mut self, value: u64) -> Self {
        self.created_utc_time = value;
        self
    }

    /// Sets the created utc time and returns this value for further mutation.
    pub fn set_created_utc_time(&mut self, value: u64) -> &mut Self {
        self.created_utc_time = value;
        self
    }

    /// Returns the configured created utc time.
    pub fn get_created_utc_time(&self) -> u64 {
        self.created_utc_time
    }

    /// Sets the update time and returns the updated value.
    pub fn update_time(mut self, value: u64) -> Self {
        self.update_time = value;
        self
    }

    /// Sets the update time and returns this value for further mutation.
    pub fn set_update_time(&mut self, value: u64) -> &mut Self {
        self.update_time = value;
        self
    }

    /// Returns the configured update time.
    pub fn get_update_time(&self) -> u64 {
        self.update_time
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.consistency_level = value;
        self
    }

    /// Sets the consistency level and returns this value for further mutation.
    pub fn set_consistency_level(&mut self, value: ConsistencyLevel) -> &mut Self {
        self.consistency_level = value;
        self
    }

    /// Returns the configured consistency level.
    pub fn get_consistency_level(&self) -> ConsistencyLevel {
        self.consistency_level
    }

    /// Sets the properties and returns the updated value.
    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.properties = value;
        self
    }

    /// Sets the properties and returns this value for further mutation.
    pub fn set_properties(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.properties = value;
        self
    }

    /// Returns the configured properties.
    pub fn get_properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// Returns the external collection source path.
    pub fn get_external_source(&self) -> &str {
        self.schema.get_external_source()
    }

    /// Returns the external collection spec JSON.
    pub fn get_external_spec(&self) -> Option<&serde_json::Value> {
        self.schema.get_external_spec()
    }

    /// Adds one add field name to the existing values.
    pub fn add_field_name(mut self, value: impl Into<String>) -> Self {
        self.field_names.push(value.into());
        self
    }

    /// Adds one add vector field name to the existing values.
    pub fn add_vector_field_name(mut self, value: impl Into<String>) -> Self {
        self.vector_field_names.push(value.into());
        self
    }

    /// Adds one add alias to the existing values.
    pub fn add_alias(mut self, value: impl Into<String>) -> Self {
        self.aliases.push(value.into());
        self
    }

    pub(crate) fn from_proto(value: milvus::DescribeCollectionResponse) -> Result<Self> {
        let schema = CollectionSchema::from_proto(value.schema.ok_or_else(|| {
            Error::MalformedResponse("describe collection returned no schema".into())
        })?)?;
        let mut field_names = Vec::new();
        for name in schema
            .get_fields()
            .iter()
            .map(|field| field.get_name())
            .chain(
                schema
                    .get_struct_fields()
                    .iter()
                    .map(|field| field.get_name()),
            )
        {
            if !field_names.iter().any(|existing| existing == name) {
                field_names.push(name.to_owned());
            }
        }
        let mut vector_field_names = schema
            .get_fields()
            .iter()
            .filter(|field| field.get_data_type().is_vector())
            .map(|field| field.get_name().to_owned())
            .collect::<Vec<_>>();
        for struct_field in schema.get_struct_fields() {
            for field in struct_field
                .get_fields()
                .iter()
                .filter(|field| field.get_data_type().is_vector())
            {
                let name = format!("{}[{}]", struct_field.get_name(), field.get_name());
                if !vector_field_names.contains(&name) {
                    vector_field_names.push(name);
                }
            }
        }
        let primary_field_name = schema
            .get_fields()
            .iter()
            .find(|field| field.is_primary_key())
            .map(|field| field.get_name().to_owned())
            .unwrap_or_default();
        let auto_id = schema.get_fields().iter().any(|field| field.is_auto_id());
        let enable_dynamic_field = schema.is_dynamic_field_enabled();
        Ok(Self {
            database_name: value.db_name,
            collection_name: value.collection_name,
            description: schema.get_description().to_owned(),
            num_partitions: value.num_partitions,
            field_names,
            vector_field_names,
            primary_field_name,
            enable_dynamic_field,
            auto_id,
            num_shards: i64::from(value.shards_num),
            schema,
            collection_id: value.collection_id,
            aliases: value.aliases,
            created_time: value.created_timestamp,
            created_utc_time: value.created_utc_timestamp,
            update_time: value.update_timestamp,
            consistency_level: ConsistencyLevel::from_proto(value.consistency_level),
            properties: value
                .properties
                .into_iter()
                .map(|v| (v.key, v.value))
                .collect(),
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// ShardReplica
///////////////////////////////////////////////////////////////////////////////
/// Replica information for one collection shard.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ShardReplica {
    pub(crate) leader_id: i64,
    pub(crate) leader_address: String,
    pub(crate) channel_name: String,
    pub(crate) node_ids: Vec<i64>,
}

impl ShardReplica {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            leader_id: 0,
            leader_address: String::new(),
            channel_name: String::new(),
            node_ids: Vec::new(),
        }
    }

    /// Sets the leader id and returns the updated value.
    pub fn leader_id(mut self, value: i64) -> Self {
        self.leader_id = value;
        self
    }

    /// Sets the leader id and returns this value for further mutation.
    pub fn set_leader_id(&mut self, value: i64) -> &mut Self {
        self.leader_id = value;
        self
    }

    /// Returns the configured leader id.
    pub fn get_leader_id(&self) -> i64 {
        self.leader_id
    }

    /// Sets the leader address and returns the updated value.
    pub fn leader_address(mut self, value: impl Into<String>) -> Self {
        self.leader_address = value.into();
        self
    }

    /// Sets the leader address and returns this value for further mutation.
    pub fn set_leader_address(&mut self, value: impl Into<String>) -> &mut Self {
        self.leader_address = value.into();
        self
    }

    /// Returns the configured leader address.
    pub fn get_leader_address(&self) -> &str {
        &self.leader_address
    }

    /// Sets the channel name and returns the updated value.
    pub fn channel_name(mut self, value: impl Into<String>) -> Self {
        self.channel_name = value.into();
        self
    }

    /// Sets the channel name and returns this value for further mutation.
    pub fn set_channel_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.channel_name = value.into();
        self
    }

    /// Returns the configured channel name.
    pub fn get_channel_name(&self) -> &str {
        &self.channel_name
    }

    /// Sets the node ids and returns the updated value.
    pub fn node_ids(mut self, value: Vec<i64>) -> Self {
        self.node_ids = value;
        self
    }

    /// Sets the node ids and returns this value for further mutation.
    pub fn set_node_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.node_ids = value;
        self
    }

    /// Returns the configured node ids.
    pub fn get_node_ids(&self) -> &[i64] {
        &self.node_ids
    }

    /// Adds one add node id to the existing values.
    pub fn add_node_id(mut self, value: i64) -> Self {
        self.node_ids.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReplicaInfo
///////////////////////////////////////////////////////////////////////////////
/// Replica-group information for a loaded collection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReplicaInfo {
    pub(crate) replica_id: i64,
    pub(crate) collection_id: i64,
    pub(crate) partition_ids: Vec<i64>,
    pub(crate) shards: Vec<ShardReplica>,
    pub(crate) node_ids: Vec<i64>,
    pub(crate) resource_group: String,
    pub(crate) outbound_nodes: HashMap<String, i32>,
}

impl ReplicaInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            replica_id: 0,
            collection_id: 0,
            partition_ids: Vec::new(),
            shards: Vec::new(),
            node_ids: Vec::new(),
            resource_group: String::new(),
            outbound_nodes: HashMap::new(),
        }
    }

    /// Sets the replica id and returns the updated value.
    pub fn replica_id(mut self, value: i64) -> Self {
        self.replica_id = value;
        self
    }

    /// Sets the replica id and returns this value for further mutation.
    pub fn set_replica_id(&mut self, value: i64) -> &mut Self {
        self.replica_id = value;
        self
    }

    /// Returns the configured replica id.
    pub fn get_replica_id(&self) -> i64 {
        self.replica_id
    }

    /// Sets the collection id and returns the updated value.
    pub fn collection_id(mut self, value: i64) -> Self {
        self.collection_id = value;
        self
    }

    /// Sets the collection id and returns this value for further mutation.
    pub fn set_collection_id(&mut self, value: i64) -> &mut Self {
        self.collection_id = value;
        self
    }

    /// Returns the configured collection id.
    pub fn get_collection_id(&self) -> i64 {
        self.collection_id
    }

    /// Sets the partition ids and returns the updated value.
    pub fn partition_ids(mut self, value: Vec<i64>) -> Self {
        self.partition_ids = value;
        self
    }

    /// Sets the partition ids and returns this value for further mutation.
    pub fn set_partition_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.partition_ids = value;
        self
    }

    /// Returns the configured partition ids.
    pub fn get_partition_ids(&self) -> &[i64] {
        &self.partition_ids
    }

    /// Sets the shards and returns the updated value.
    pub fn shards(mut self, value: Vec<ShardReplica>) -> Self {
        self.shards = value;
        self
    }

    /// Sets the shards and returns this value for further mutation.
    pub fn set_shards(&mut self, value: Vec<ShardReplica>) -> &mut Self {
        self.shards = value;
        self
    }

    /// Returns the configured shards.
    pub fn get_shards(&self) -> &[ShardReplica] {
        &self.shards
    }

    /// Sets the node ids and returns the updated value.
    pub fn node_ids(mut self, value: Vec<i64>) -> Self {
        self.node_ids = value;
        self
    }

    /// Sets the node ids and returns this value for further mutation.
    pub fn set_node_ids(&mut self, value: Vec<i64>) -> &mut Self {
        self.node_ids = value;
        self
    }

    /// Returns the configured node ids.
    pub fn get_node_ids(&self) -> &[i64] {
        &self.node_ids
    }

    /// Sets the resource group and returns the updated value.
    pub fn resource_group(mut self, value: impl Into<String>) -> Self {
        self.resource_group = value.into();
        self
    }

    /// Sets the resource group and returns this value for further mutation.
    pub fn set_resource_group(&mut self, value: impl Into<String>) -> &mut Self {
        self.resource_group = value.into();
        self
    }

    /// Returns the configured resource group.
    pub fn get_resource_group(&self) -> &str {
        &self.resource_group
    }

    /// Sets the outbound nodes and returns the updated value.
    pub fn outbound_nodes(mut self, value: HashMap<String, i32>) -> Self {
        self.outbound_nodes = value;
        self
    }

    /// Sets the outbound nodes and returns this value for further mutation.
    pub fn set_outbound_nodes(&mut self, value: HashMap<String, i32>) -> &mut Self {
        self.outbound_nodes = value;
        self
    }

    /// Returns the configured outbound nodes.
    pub fn get_outbound_nodes(&self) -> &HashMap<String, i32> {
        &self.outbound_nodes
    }

    /// Adds one add partition id to the existing values.
    pub fn add_partition_id(mut self, value: i64) -> Self {
        self.partition_ids.push(value);
        self
    }

    /// Adds one add shard to the existing values.
    pub fn add_shard(mut self, value: ShardReplica) -> Self {
        self.shards.push(value);
        self
    }

    /// Adds one add node id to the existing values.
    pub fn add_node_id(mut self, value: i64) -> Self {
        self.node_ids.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// CollectionInfo
///////////////////////////////////////////////////////////////////////////////
/// Summary metadata for a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CollectionInfo {
    pub(crate) name: String,
    pub(crate) id: i64,
    pub(crate) created_timestamp: u64,
    pub(crate) created_utc_timestamp: u64,
    pub(crate) query_service_available: Option<bool>,
    pub(crate) shard_count: Option<i32>,
}

impl CollectionInfo {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            id: 0,
            created_timestamp: 0,
            created_utc_timestamp: 0,
            query_service_available: None,
            shard_count: None,
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    /// Sets the name and returns this value for further mutation.
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    /// Returns the configured name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the id and returns the updated value.
    pub fn id(mut self, value: i64) -> Self {
        self.id = value;
        self
    }

    /// Sets the id and returns this value for further mutation.
    pub fn set_id(&mut self, value: i64) -> &mut Self {
        self.id = value;
        self
    }

    /// Returns the configured id.
    pub fn get_id(&self) -> i64 {
        self.id
    }

    /// Sets the created timestamp and returns the updated value.
    pub fn created_timestamp(mut self, value: u64) -> Self {
        self.created_timestamp = value;
        self
    }

    /// Sets the created timestamp and returns this value for further mutation.
    pub fn set_created_timestamp(&mut self, value: u64) -> &mut Self {
        self.created_timestamp = value;
        self
    }

    /// Returns the configured created timestamp.
    pub fn get_created_timestamp(&self) -> u64 {
        self.created_timestamp
    }

    /// Sets the created utc timestamp and returns the updated value.
    pub fn created_utc_timestamp(mut self, value: u64) -> Self {
        self.created_utc_timestamp = value;
        self
    }

    /// Sets the created utc timestamp and returns this value for further mutation.
    pub fn set_created_utc_timestamp(&mut self, value: u64) -> &mut Self {
        self.created_utc_timestamp = value;
        self
    }

    /// Returns the configured created utc timestamp.
    pub fn get_created_utc_timestamp(&self) -> u64 {
        self.created_utc_timestamp
    }

    /// Sets the query service available and returns the updated value.
    pub fn query_service_available(mut self, value: bool) -> Self {
        self.query_service_available = Some(value);
        self
    }

    /// Sets the query service available and returns this value for further mutation.
    pub fn set_query_service_available(&mut self, value: bool) -> &mut Self {
        self.query_service_available = Some(value);
        self
    }

    /// Returns whether query service is available, or `None` when the server omitted this metadata.
    pub fn get_query_service_available(&self) -> Option<bool> {
        self.query_service_available
    }

    /// Sets the shard count and returns the updated value.
    pub fn shard_count(mut self, value: i32) -> Self {
        self.shard_count = Some(value);
        self
    }

    /// Sets the shard count and returns this value for further mutation.
    pub fn set_shard_count(&mut self, value: i32) -> &mut Self {
        self.shard_count = Some(value);
        self
    }

    /// Returns the shard count, or `None` when an older server omitted this metadata.
    pub fn get_shard_count(&self) -> Option<i32> {
        self.shard_count
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod collection_schema_tests {
    use super::{CollectionSchema, DataType, FieldSchema, StructFieldSchema};
    use crate::proto::{common, schema};
    use crate::v2::error::Error;

    #[test]
    fn dynamic_field_is_enabled_by_default() {
        assert!(CollectionSchema::new().is_dynamic_field_enabled());
        assert!(CollectionSchema::new().is_dynamic_field_enabled());
        assert!(!CollectionSchema::new()
            .enable_dynamic_field(false)
            .is_dynamic_field_enabled());
    }

    #[test]
    fn namespace_is_not_exposed_or_preserved_by_sdk_schema() {
        let sdk = CollectionSchema::from_proto(schema::CollectionSchema {
            enable_namespace: true,
            ..Default::default()
        })
        .expect("valid collection schema");

        assert!(!sdk.to_proto().enable_namespace);
    }

    #[test]
    fn struct_fields_convert_to_proto_and_round_trip() {
        let struct_field = StructFieldSchema::new()
            .name("items")
            .description("nested item data")
            .max_capacity(32)
            .add_field(
                FieldSchema::new()
                    .name("label")
                    .data_type(DataType::VarChar)
                    .max_length(128),
            )
            .add_field(
                FieldSchema::new()
                    .name("embedding")
                    .data_type(DataType::FloatVector)
                    .dimension(4),
            );
        let sdk = CollectionSchema::new().add_struct_field(struct_field);

        let proto = sdk.to_proto();
        assert_eq!(proto.struct_array_fields.len(), 1);

        let proto_struct = &proto.struct_array_fields[0];
        assert_eq!(proto_struct.name, "items");
        assert_eq!(proto_struct.description, "nested item data");
        assert_eq!(proto_struct.fields.len(), 2);

        let scalar = &proto_struct.fields[0];
        assert_eq!(
            schema::DataType::try_from(scalar.data_type).unwrap(),
            schema::DataType::Array
        );
        assert_eq!(
            schema::DataType::try_from(scalar.element_type).unwrap(),
            schema::DataType::VarChar
        );
        assert!(scalar
            .type_params
            .iter()
            .any(|pair| pair.key == "max_capacity" && pair.value == "32"));
        assert!(scalar
            .type_params
            .iter()
            .any(|pair| pair.key == "max_length" && pair.value == "128"));

        let vector = &proto_struct.fields[1];
        assert_eq!(
            schema::DataType::try_from(vector.data_type).unwrap(),
            schema::DataType::ArrayOfVector
        );
        assert_eq!(
            schema::DataType::try_from(vector.element_type).unwrap(),
            schema::DataType::FloatVector
        );
        assert!(vector
            .type_params
            .iter()
            .any(|pair| pair.key == "max_capacity" && pair.value == "32"));
        assert!(vector
            .type_params
            .iter()
            .any(|pair| pair.key == "dim" && pair.value == "4"));

        assert_eq!(CollectionSchema::from_proto(proto).unwrap(), sdk);
    }

    #[test]
    fn array_element_type_converts_to_proto_and_round_trips() {
        let sdk = CollectionSchema::new().add_field(
            FieldSchema::new()
                .name("tags")
                .data_type(DataType::Array)
                .element_type(DataType::VarChar)
                .max_capacity(16),
        );

        let proto = sdk.to_proto();
        assert_eq!(proto.fields.len(), 1);
        assert_eq!(
            schema::DataType::try_from(proto.fields[0].data_type).unwrap(),
            schema::DataType::Array
        );
        assert_eq!(
            schema::DataType::try_from(proto.fields[0].element_type).unwrap(),
            schema::DataType::VarChar
        );
        assert_eq!(CollectionSchema::from_proto(proto).unwrap(), sdk);
    }

    #[test]
    fn collection_schema_rejects_unknown_field_types() {
        let error = CollectionSchema::from_proto(schema::CollectionSchema {
            fields: vec![schema::FieldSchema {
                name: "broken".into(),
                data_type: 999,
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap_err();

        assert!(matches!(
            error,
            Error::MalformedResponse(message)
                if message.contains("broken") && message.contains("unknown data type")
        ));
    }

    #[test]
    fn collection_schema_rejects_invalid_struct_subfields_without_dropping_them() {
        let error = CollectionSchema::from_proto(schema::CollectionSchema {
            struct_array_fields: vec![schema::StructArrayFieldSchema {
                name: "items".into(),
                fields: vec![schema::FieldSchema {
                    name: "broken".into(),
                    data_type: schema::DataType::Array as i32,
                    element_type: 999,
                    type_params: vec![common::KeyValuePair {
                        key: "max_capacity".into(),
                        value: "16".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        })
        .unwrap_err();

        assert!(matches!(
            error,
            Error::MalformedResponse(message)
                if message.contains("broken") && message.contains("unknown data type")
        ));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod nullable_default_tests {
    use super::{CollectionSchema, DataType, DefaultValue, FieldSchema};
    use crate::v2::types::common::FieldData;

    #[test]
    fn nullable_field_data_validates_and_encodes_compact_values() {
        let data = FieldData::nullable(
            FieldData::Int32 {
                name: "optional".into(),
                values: vec![10, 30],
            },
            vec![true, false, true],
        )
        .unwrap();
        assert_eq!(data.len(), 3);
        assert!(data.is_null(1));

        let proto = data.into_proto().unwrap();
        assert_eq!(proto.valid_data, vec![true, false, true]);
        assert!(FieldData::nullable(
            FieldData::Int32 {
                name: "invalid".into(),
                values: vec![1],
            },
            vec![true, true],
        )
        .is_err());
    }

    #[test]
    fn collection_schema_rejects_unrepresentable_or_mismatched_defaults() {
        let vector_default = CollectionSchema::new().add_field(
            FieldSchema::new()
                .name("vector")
                .data_type(DataType::FloatVector)
                .dimension(2)
                .default_value(DefaultValue::Float(1.0)),
        );
        assert!(vector_default
            .encode_for_collection("invalid", None)
            .is_err());

        let mismatched = CollectionSchema::new().add_field(
            FieldSchema::new()
                .name("count")
                .data_type(DataType::Int64)
                .default_value(DefaultValue::String("one".into())),
        );
        assert!(mismatched.encode_for_collection("invalid", None).is_err());

        let unsupported_json = CollectionSchema::new().add_field(
            FieldSchema::new()
                .name("metadata")
                .data_type(DataType::Json)
                .default_value(DefaultValue::String("{}".into())),
        );
        assert!(unsupported_json
            .encode_for_collection("invalid", None)
            .is_err());

        let valid = CollectionSchema::new().add_field(
            FieldSchema::new()
                .name("count")
                .data_type(DataType::Int64)
                .default_value(DefaultValue::Int64(1)),
        );
        assert!(valid.encode_for_collection("valid", None).is_ok());
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn collection_desc_default_values() {
        let value = CollectionDesc::new();
        let expected_database_name: String = String::new();
        let expected_collection_name: String = String::new();
        let expected_description: String = String::new();
        let expected_num_partitions: i64 = 0;
        let expected_field_names: Vec<String> = Default::default();
        let expected_vector_field_names: Vec<String> = Default::default();
        let expected_primary_field_name: String = String::new();
        let expected_enable_dynamic_field: bool = true;
        let expected_auto_id: bool = false;
        let expected_num_shards: i64 = 1;
        let expected_schema = CollectionSchema::new();
        let expected_collection_id: i64 = 0;
        let expected_aliases: Vec<String> = Default::default();
        let expected_created_time: u64 = 0;
        let expected_created_utc_time: u64 = 0;
        let expected_update_time: u64 = 0;
        let expected_consistency_level: ConsistencyLevel = Default::default();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.get_database_name().to_owned(), expected_database_name);
        assert_eq!(
            value.get_collection_name().to_owned(),
            expected_collection_name
        );
        assert_eq!(value.get_description().to_owned(), expected_description);
        assert_eq!(
            value.get_num_partitions().to_owned(),
            expected_num_partitions
        );
        assert_eq!(value.get_field_names().to_owned(), expected_field_names);
        assert_eq!(
            value.get_vector_field_names().to_owned(),
            expected_vector_field_names
        );
        assert_eq!(
            value.get_primary_field_name().to_owned(),
            expected_primary_field_name
        );
        assert_eq!(
            value.is_dynamic_field_enabled(),
            expected_enable_dynamic_field
        );
        assert_eq!(value.get_auto_id().to_owned(), expected_auto_id);
        assert_eq!(value.get_num_shards().to_owned(), expected_num_shards);
        assert_eq!(value.get_schema().to_owned(), expected_schema);
        assert_eq!(value.get_collection_id().to_owned(), expected_collection_id);
        assert_eq!(value.get_aliases().to_owned(), expected_aliases);
        assert_eq!(value.get_created_time().to_owned(), expected_created_time);
        assert_eq!(
            value.get_created_utc_time().to_owned(),
            expected_created_utc_time
        );
        assert_eq!(value.get_update_time().to_owned(), expected_update_time);
        assert_eq!(
            value.get_consistency_level().to_owned(),
            expected_consistency_level
        );
        assert_eq!(value.get_properties().to_owned(), expected_properties);
    }

    #[test]
    fn collection_desc_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let description = "description-value".to_owned();
        let num_partitions = 7;
        let field_names = vec!["field_names-value".to_owned()];
        let vector_field_names = vec!["vector_field_names-value".to_owned()];
        let primary_field_name = "primary_field_name-value".to_owned();
        let enable_dynamic_field = true;
        let auto_id = true;
        let num_shards = 7;
        let schema = CollectionSchema::new().description("schema");
        let collection_id = 7;
        let aliases = vec!["aliases-value".to_owned()];
        let created_time = 7;
        let created_utc_time = 7;
        let update_time = 7;
        let consistency_level = ConsistencyLevel::Strong;
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = CollectionDesc::new()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .description(description.clone())
            .num_partitions(num_partitions.clone())
            .field_names(field_names.clone())
            .vector_field_names(vector_field_names.clone())
            .primary_field_name(primary_field_name.clone())
            .enable_dynamic_field(enable_dynamic_field.clone())
            .auto_id(auto_id.clone())
            .num_shards(num_shards.clone())
            .schema(schema.clone())
            .collection_id(collection_id.clone())
            .aliases(aliases.clone())
            .created_time(created_time.clone())
            .created_utc_time(created_utc_time.clone())
            .update_time(update_time.clone())
            .consistency_level(consistency_level.clone())
            .properties(properties.clone());

        assert_eq!(value.get_database_name().to_owned(), database_name);
        assert_eq!(value.get_collection_name().to_owned(), collection_name);
        assert_eq!(value.get_description().to_owned(), description);
        assert_eq!(value.get_num_partitions().to_owned(), num_partitions);
        assert_eq!(value.get_field_names().to_owned(), field_names);
        assert_eq!(
            value.get_vector_field_names().to_owned(),
            vector_field_names
        );
        assert_eq!(
            value.get_primary_field_name().to_owned(),
            primary_field_name
        );
        assert_eq!(
            value.is_dynamic_field_enabled().to_owned(),
            enable_dynamic_field
        );
        assert_eq!(value.get_auto_id().to_owned(), auto_id);
        assert_eq!(value.get_num_shards().to_owned(), num_shards);
        assert_eq!(value.get_schema().to_owned(), schema);
        assert_eq!(value.get_collection_id().to_owned(), collection_id);
        assert_eq!(value.get_aliases().to_owned(), aliases);
        assert_eq!(value.get_created_time().to_owned(), created_time);
        assert_eq!(value.get_created_utc_time().to_owned(), created_utc_time);
        assert_eq!(value.get_update_time().to_owned(), update_time);
        assert_eq!(value.get_consistency_level().to_owned(), consistency_level);
        assert_eq!(value.get_properties().to_owned(), properties);
    }

    #[test]
    fn shard_replica_default_values() {
        let value = ShardReplica::new();
        let expected_leader_id: i64 = 0;
        let expected_leader_address: String = String::new();
        let expected_channel_name: String = String::new();
        let expected_node_ids: Vec<i64> = Default::default();

        assert_eq!(value.get_leader_id().to_owned(), expected_leader_id);
        assert_eq!(
            value.get_leader_address().to_owned(),
            expected_leader_address
        );
        assert_eq!(value.get_channel_name().to_owned(), expected_channel_name);
        assert_eq!(value.get_node_ids().to_owned(), expected_node_ids);
    }

    #[test]
    fn shard_replica_populated_values() {
        let leader_id = 7;
        let leader_address = "leader_address-value".to_owned();
        let channel_name = "channel_name-value".to_owned();
        let node_ids = vec![7];
        let value = ShardReplica::new()
            .leader_id(leader_id.clone())
            .leader_address(leader_address.clone())
            .channel_name(channel_name.clone())
            .node_ids(node_ids.clone());

        assert_eq!(value.get_leader_id().to_owned(), leader_id);
        assert_eq!(value.get_leader_address().to_owned(), leader_address);
        assert_eq!(value.get_channel_name().to_owned(), channel_name);
        assert_eq!(value.get_node_ids().to_owned(), node_ids);
    }

    #[test]
    fn replica_info_default_values() {
        let value = ReplicaInfo::new();
        let expected_replica_id: i64 = 0;
        let expected_collection_id: i64 = 0;
        let expected_partition_ids: Vec<i64> = Default::default();
        let expected_shards: Vec<ShardReplica> = Default::default();
        let expected_node_ids: Vec<i64> = Default::default();
        let expected_resource_group: String = String::new();
        let expected_outbound_nodes: HashMap<String, i32> = Default::default();

        assert_eq!(value.get_replica_id().to_owned(), expected_replica_id);
        assert_eq!(value.get_collection_id().to_owned(), expected_collection_id);
        assert_eq!(value.get_partition_ids().to_owned(), expected_partition_ids);
        assert_eq!(value.get_shards().to_owned(), expected_shards);
        assert_eq!(value.get_node_ids().to_owned(), expected_node_ids);
        assert_eq!(
            value.get_resource_group().to_owned(),
            expected_resource_group
        );
        assert_eq!(
            value.get_outbound_nodes().to_owned(),
            expected_outbound_nodes
        );
    }

    #[test]
    fn replica_info_populated_values() {
        let replica_id = 7;
        let collection_id = 7;
        let partition_ids = vec![7];
        let shards = vec![ShardReplica::new()];
        let node_ids = vec![7];
        let resource_group = "resource_group-value".to_owned();
        let outbound_nodes = HashMap::from([("key-value".to_owned(), 7)]);
        let value = ReplicaInfo::new()
            .replica_id(replica_id.clone())
            .collection_id(collection_id.clone())
            .partition_ids(partition_ids.clone())
            .shards(shards.clone())
            .node_ids(node_ids.clone())
            .resource_group(resource_group.clone())
            .outbound_nodes(outbound_nodes.clone());

        assert_eq!(value.get_replica_id().to_owned(), replica_id);
        assert_eq!(value.get_collection_id().to_owned(), collection_id);
        assert_eq!(value.get_partition_ids().to_owned(), partition_ids);
        assert_eq!(value.get_shards().to_owned(), shards);
        assert_eq!(value.get_node_ids().to_owned(), node_ids);
        assert_eq!(value.get_resource_group().to_owned(), resource_group);
        assert_eq!(value.get_outbound_nodes().to_owned(), outbound_nodes);
    }

    #[test]
    fn collection_info_default_values() {
        let value = CollectionInfo::new();
        let expected_name: String = String::new();
        let expected_id: i64 = 0;
        let expected_created_timestamp: u64 = 0;
        let expected_created_utc_timestamp: u64 = 0;
        let expected_query_service_available: Option<bool> = None;
        let expected_shard_count: Option<i32> = None;

        assert_eq!(value.get_name().to_owned(), expected_name);
        assert_eq!(value.get_id().to_owned(), expected_id);
        assert_eq!(
            value.get_created_timestamp().to_owned(),
            expected_created_timestamp
        );
        assert_eq!(
            value.get_created_utc_timestamp(),
            expected_created_utc_timestamp
        );
        assert_eq!(
            value.get_query_service_available(),
            expected_query_service_available
        );
        assert_eq!(value.get_shard_count().to_owned(), expected_shard_count);
    }

    #[test]
    fn collection_info_populated_values() {
        let name = "name-value".to_owned();
        let id = 7;
        let created_timestamp = 7;
        let created_utc_timestamp = 7;
        let query_service_available = true;
        let shard_count = 7;
        let value = CollectionInfo::new()
            .name(name.clone())
            .id(id.clone())
            .created_timestamp(created_timestamp.clone())
            .created_utc_timestamp(created_utc_timestamp.clone())
            .query_service_available(query_service_available.clone())
            .shard_count(shard_count.clone());

        assert_eq!(value.get_name().to_owned(), name);
        assert_eq!(value.get_id().to_owned(), id);
        assert_eq!(value.get_created_timestamp().to_owned(), created_timestamp);
        assert_eq!(
            value.get_created_utc_timestamp().to_owned(),
            created_utc_timestamp
        );
        assert_eq!(
            value.get_query_service_available(),
            Some(query_service_available)
        );
        assert_eq!(value.get_shard_count(), Some(shard_count));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod constructor_value_tests {
    use super::*;

    #[test]
    fn schema_constructors_initialize_required_fields() {
        let field_from_new = FieldSchema::new()
            .name("id")
            .data_type(DataType::Int64)
            .primary_key(true)
            .auto_id(true);
        assert_eq!(field_from_new.get_name(), "id");
        assert_eq!(field_from_new.get_data_type(), DataType::Int64);

        let struct_from_new = StructFieldSchema::new()
            .name("history")
            .max_capacity(8)
            .add_field(FieldSchema::new().name("score").data_type(DataType::Float));
        assert_eq!(struct_from_new.get_name(), "history");
        assert_eq!(struct_from_new.get_fields().len(), 1);

        let schema_from_default = CollectionSchema::new()
            .enable_dynamic_field(false)
            .add_field(field_from_new.clone());
        let schema_from_direct = CollectionSchema::new()
            .enable_dynamic_field(false)
            .fields(vec![field_from_new]);
        assert_eq!(schema_from_direct, schema_from_default);
    }

    #[test]
    fn field_schema_constructor_values() {
        let value = FieldSchema::new()
            .name("field")
            .data_type(DataType::Unknown);

        assert_eq!(value.get_name(), "field");
        assert!(value.get_description().is_empty());
        assert_eq!(value.get_data_type().to_owned(), DataType::Unknown);
        assert_eq!(value.get_element_type().to_owned(), None);
        assert!(!value.is_primary_key());
        assert!(!value.is_auto_id());
        assert!(!value.is_partition_key());
        assert!(!value.is_clustering_key());
        assert!(!value.is_nullable());
        assert_eq!(value.get_dimension(), 0);
        assert_eq!(value.get_max_length(), 65_535);
        assert_eq!(value.get_max_capacity(), 0);
        assert!(!value.is_analyzer_enabled());
        assert!(!value.is_match_enabled());
        assert_eq!(
            value.get_analyzer_params().unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(
            value.get_multi_analyzer_params().unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(value.get_default_value().to_owned(), None);
        assert!(value.get_type_params().is_empty());
        assert!(value.get_index_params().is_empty());
    }

    #[test]
    fn primary_key_and_auto_id_are_configured_independently() {
        let value = FieldSchema::new()
            .name("id")
            .data_type(DataType::Int64)
            .auto_id(true)
            .primary_key(false);

        assert!(!value.is_primary_key());
        assert!(value.is_auto_id());
    }

    #[test]
    fn analyzer_and_match_flags_round_trip_as_typed_schema_options() {
        let value = FieldSchema::new()
            .name("text")
            .data_type(DataType::VarChar)
            .enable_analyzer(true)
            .enable_match(true);

        let proto = value.clone().into_proto();
        assert!(proto
            .type_params
            .iter()
            .any(|pair| pair.key == "enable_analyzer" && pair.value == "true"));
        assert!(proto
            .type_params
            .iter()
            .any(|pair| pair.key == "enable_match" && pair.value == "true"));
        assert_eq!(FieldSchema::from_proto(proto).unwrap(), value.clone());

        let mut disabled = value;
        disabled.set_enable_analyzer(false).set_enable_match(false);
        assert!(!disabled.is_analyzer_enabled());
        assert!(!disabled.is_match_enabled());
        assert_eq!(
            disabled
                .get_type_params()
                .get("enable_analyzer")
                .map(String::as_str),
            Some("false")
        );
        assert_eq!(
            disabled
                .get_type_params()
                .get("enable_match")
                .map(String::as_str),
            Some("false")
        );
    }

    #[test]
    fn numeric_type_params_follow_cpp_field_schema_behavior() {
        let mut value = FieldSchema::new().dimension(0);
        assert_eq!(value.get_dimension(), 0);
        assert!(!value.get_type_params().contains_key("dim"));

        value
            .set_dimension(128)
            .set_max_length(1_024)
            .set_max_capacity(32);
        assert_eq!(value.get_dimension(), 128);
        assert_eq!(value.get_max_length(), 1_024);
        assert_eq!(value.get_max_capacity(), 32);

        value.set_dimension(0);
        assert_eq!(value.get_dimension(), 128);
    }

    #[test]
    fn analyzer_params_round_trip_through_typed_methods() {
        let analyzer = serde_json::json!({
            "tokenizer": "standard",
            "filter": ["lowercase"]
        });
        let multi_analyzer = serde_json::json!({
            "analyzers": {"english": {"type": "english"}},
            "by_field": "language"
        });
        let mut value = FieldSchema::new()
            .name("text")
            .data_type(DataType::VarChar)
            .analyzer_params(analyzer.clone());
        assert_eq!(value.get_analyzer_params().unwrap(), analyzer);

        value.set_multi_analyzer_params(multi_analyzer.clone());
        assert_eq!(value.get_multi_analyzer_params().unwrap(), multi_analyzer);

        let decoded = FieldSchema::from_proto(value.clone().into_proto()).unwrap();
        assert_eq!(decoded.get_analyzer_params().unwrap(), analyzer);
        assert_eq!(decoded.get_multi_analyzer_params().unwrap(), multi_analyzer);

        let malformed = FieldSchema::new().type_param("analyzer_params", "not-json");
        assert!(malformed.get_analyzer_params().is_err());
    }

    #[test]
    fn type_params_merge_without_overwriting_existing_values() {
        let mut value = FieldSchema::new()
            .enable_analyzer(true)
            .type_params(HashMap::from([
                ("enable_analyzer".into(), "false".into()),
                ("custom".into(), "first".into()),
            ]))
            .type_param("custom", "second");

        value.set_type_params(HashMap::from([
            ("enable_analyzer".into(), "false".into()),
            ("other".into(), "value".into()),
        ]));

        assert!(value.is_analyzer_enabled());
        assert_eq!(
            value.get_type_params().get("custom").map(String::as_str),
            Some("first")
        );
        assert_eq!(
            value.get_type_params().get("other").map(String::as_str),
            Some("value")
        );
    }

    #[test]
    fn field_schema_populated_values() {
        let default_value = DefaultValue::String("unknown".to_owned());
        let value = FieldSchema::new()
            .name("tags")
            .data_type(DataType::Array)
            .description("description")
            .element_type(DataType::VarChar)
            .primary_key(true)
            .auto_id(true)
            .partition_key(true)
            .clustering_key(true)
            .nullable(true)
            .default_value(default_value.clone())
            .dimension(8)
            .max_length(128)
            .max_capacity(16)
            .enable_analyzer(true)
            .enable_match(true)
            .type_param("custom", "type")
            .index_param("index", "value");

        assert_eq!(value.get_name().to_owned(), "tags");
        assert_eq!(value.get_description().to_owned(), "description");
        assert_eq!(value.get_data_type().to_owned(), DataType::Array);
        assert_eq!(value.get_element_type().to_owned(), Some(DataType::VarChar));
        assert!(value.is_primary_key());
        assert!(value.is_auto_id());
        assert!(value.is_partition_key());
        assert!(value.is_clustering_key());
        assert!(value.is_nullable());
        assert!(value.is_analyzer_enabled());
        assert!(value.is_match_enabled());
        assert_eq!(value.get_default_value(), &Some(default_value));
        assert_eq!(
            value.get_type_params().get("dim").map(String::as_str),
            Some("8")
        );
        assert_eq!(
            value
                .get_type_params()
                .get("max_length")
                .map(String::as_str),
            Some("128")
        );
        assert_eq!(
            value
                .get_type_params()
                .get("max_capacity")
                .map(String::as_str),
            Some("16")
        );
        assert_eq!(
            value.get_type_params().get("custom").map(String::as_str),
            Some("type")
        );
        assert_eq!(
            value.get_index_params().get("index").map(String::as_str),
            Some("value")
        );
    }

    #[test]
    fn field_schema_mutable_setters_update_existing_value() {
        let default_value = DefaultValue::String("unknown".to_owned());
        let mut value = FieldSchema::new()
            .name("field")
            .data_type(DataType::Unknown);
        value
            .set_name("tags")
            .set_description("description")
            .set_data_type(DataType::Array)
            .set_element_type(DataType::VarChar)
            .set_primary_key(true)
            .set_auto_id(true)
            .set_partition_key(true)
            .set_clustering_key(true)
            .set_nullable(true)
            .set_default_value(default_value.clone())
            .set_type_params(HashMap::from([("max_capacity".into(), "16".into())]))
            .set_enable_analyzer(true)
            .set_enable_match(true)
            .set_index_params(HashMap::from([("index".into(), "value".into())]));

        assert_eq!(value.get_name(), "tags");
        assert_eq!(value.get_description(), "description");
        assert_eq!(value.get_data_type(), DataType::Array);
        assert_eq!(value.get_element_type(), Some(DataType::VarChar));
        assert!(value.is_primary_key());
        assert!(value.is_auto_id());
        assert!(value.is_partition_key());
        assert!(value.is_clustering_key());
        assert!(value.is_nullable());
        assert!(value.is_analyzer_enabled());
        assert!(value.is_match_enabled());
        assert_eq!(value.get_default_value(), &Some(default_value));
        assert_eq!(
            value
                .get_type_params()
                .get("max_capacity")
                .map(String::as_str),
            Some("16")
        );
        assert_eq!(
            value.get_index_params().get("index").map(String::as_str),
            Some("value")
        );
    }

    #[test]
    fn struct_field_schema_constructor_values() {
        let value = StructFieldSchema::new().name("items");

        assert_eq!(value.get_name(), "items");
        assert!(value.get_description().is_empty());
        assert_eq!(value.get_max_capacity().to_owned(), 0);
        assert!(value.get_fields().is_empty());
    }

    #[test]
    fn struct_field_schema_populated_values() {
        let field = FieldSchema::new()
            .name("text")
            .data_type(DataType::VarChar)
            .max_length(128);
        let value = StructFieldSchema::new()
            .name("items")
            .description("description")
            .max_capacity(16)
            .nullable(true)
            .add_field(field.clone());

        assert_eq!(value.get_name().to_owned(), "items");
        assert_eq!(value.get_description().to_owned(), "description");
        assert_eq!(value.get_max_capacity().to_owned(), 16);
        assert!(value.is_nullable());
        assert_eq!(value.get_fields().to_owned(), [field]);
        assert_eq!(
            StructFieldSchema::from_proto(value.clone().into_proto()).unwrap(),
            value
        );
    }

    #[test]
    fn struct_field_schema_nullable_parent_allows_nullable_subfields() {
        let nullable_parent = StructFieldSchema::new()
            .name("items")
            .max_capacity(8)
            .nullable(true)
            .add_field(
                FieldSchema::new()
                    .name("note")
                    .data_type(DataType::VarChar)
                    .max_length(64)
                    .nullable(true),
            );
        assert!(nullable_parent.validate().is_ok());

        let non_nullable_parent = StructFieldSchema::new()
            .name("items")
            .max_capacity(8)
            .add_field(
                FieldSchema::new()
                    .name("note")
                    .data_type(DataType::VarChar)
                    .max_length(64)
                    .nullable(true),
            );
        let error = non_nullable_parent.validate().unwrap_err();
        assert!(error.to_string().contains("non-nullable struct"));
    }

    #[test]
    fn collection_schema_default_values() {
        let value = CollectionSchema::new();

        assert!(value.get_description().is_empty());
        assert!(value.is_dynamic_field_enabled());
        assert!(value.get_fields().is_empty());
        assert!(value.get_struct_fields().is_empty());
        assert!(value.get_functions().is_empty());
        assert!(value.get_properties().is_empty());
    }

    #[test]
    fn collection_schema_populated_values() {
        let field = FieldSchema::new()
            .name("id")
            .data_type(DataType::Int64)
            .primary_key(true);
        let struct_field = StructFieldSchema::new()
            .name("items")
            .max_capacity(8)
            .add_field(
                FieldSchema::new()
                    .name("text")
                    .data_type(DataType::VarChar)
                    .max_length(64),
            );
        let function = Function::new()
            .name("bm25")
            .function_type(crate::v2::FunctionType::Bm25)
            .input_fields(["text"])
            .output_fields(["sparse"]);
        let value = CollectionSchema::new()
            .description("description")
            .enable_dynamic_field(false)
            .add_field(field.clone())
            .add_struct_field(struct_field.clone())
            .add_function(function.clone())
            .property("key", "value");

        assert_eq!(value.get_description().to_owned(), "description");
        assert!(!value.is_dynamic_field_enabled());
        assert_eq!(value.get_fields().to_owned(), [field]);
        assert_eq!(value.get_struct_fields().to_owned(), [struct_field]);
        assert_eq!(value.get_functions().to_owned(), [function]);
        assert_eq!(
            value.get_properties().get("key").map(String::as_str),
            Some("value")
        );
        assert_eq!(
            CollectionSchema::from_proto(value.to_proto()).unwrap(),
            value
        );
    }

    #[test]
    fn collection_schema_mutable_setters_update_existing_value() {
        let field = FieldSchema::new()
            .name("id")
            .data_type(DataType::Int64)
            .primary_key(true);
        let struct_field = StructFieldSchema::new().name("items");
        let function = Function::new()
            .name("bm25")
            .function_type(crate::v2::FunctionType::Bm25);
        let mut value = CollectionSchema::new();
        value
            .set_description("description")
            .set_enable_dynamic_field(false)
            .set_fields(vec![field.clone()])
            .set_struct_fields(vec![struct_field.clone()])
            .set_functions(vec![function.clone()])
            .set_properties(HashMap::from([("key".into(), "value".into())]));

        assert_eq!(value.get_description(), "description");
        assert!(!value.is_dynamic_field_enabled());
        assert_eq!(value.get_fields(), [field]);
        assert_eq!(value.get_struct_fields(), [struct_field]);
        assert_eq!(value.get_functions(), [function]);
        assert_eq!(
            value.get_properties().get("key").map(String::as_str),
            Some("value")
        );
    }

    #[test]
    fn collection_schema_external_fields_round_trip() {
        let value = CollectionSchema::new()
            .external_source("s3://bucket/path")
            .external_spec(serde_json::json!({"format": "parquet"}))
            .add_field(
                FieldSchema::new()
                    .name("id")
                    .data_type(DataType::Int64)
                    .primary_key(true)
                    .external_field("id"),
            );

        assert_eq!(value.get_external_source(), "s3://bucket/path");
        assert_eq!(
            value.get_external_spec(),
            Some(&serde_json::json!({"format": "parquet"}))
        );
        assert_eq!(value.get_fields()[0].get_external_field(), "id");

        let proto = value.to_proto();
        assert_eq!(proto.external_source, "s3://bucket/path");
        assert_eq!(proto.external_spec, r#"{"format":"parquet"}"#);
        assert_eq!(proto.fields[0].external_field, "id");
        assert_eq!(CollectionSchema::from_proto(proto).unwrap(), value);
    }

    #[test]
    fn collection_schema_external_spec_defaults_to_none() {
        let value = CollectionSchema::new().external_source("s3://bucket/path");
        assert!(value.get_external_spec().is_none());
        assert_eq!(value.to_proto().external_spec, "");
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod enum_conversion_tests {
    use super::*;

    #[test]
    fn load_state_converts_from_proto() {
        let cases = [
            (common::LoadState::NotExist, LoadState::NotExist),
            (common::LoadState::NotLoad, LoadState::NotLoad),
            (common::LoadState::Loading, LoadState::Loading),
            (common::LoadState::Loaded, LoadState::Loaded),
        ];

        for (proto, sdk) in cases {
            assert_eq!(LoadState::from_proto(proto as i32), sdk);
        }
        assert_eq!(LoadState::from_proto(i32::MAX), LoadState::Unknown);
    }

    #[test]
    fn default_value_round_trips_proto() {
        let values = [
            DefaultValue::Bool(true),
            DefaultValue::Int32(7),
            DefaultValue::Int64(8),
            DefaultValue::Float(1.5),
            DefaultValue::Double(2.5),
            DefaultValue::String("value".to_owned()),
            DefaultValue::Bytes(vec![1, 2]),
            DefaultValue::TimestampTz(100),
        ];

        for value in values {
            assert_eq!(
                DefaultValue::from_proto(value.clone().into_proto()).unwrap(),
                value
            );
        }
        assert!(DefaultValue::from_proto(schema::ValueField::default()).is_err());
    }
}
