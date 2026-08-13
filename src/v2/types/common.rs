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

//! Common V2 configuration, data, function, identifier, and metric types.

use crate::proto::{common, schema};
use crate::v2::error::{Error, Result};
use crate::v2::types::dql::{BoostRerank, DecayRerank, ModelRerank, RRFRerank, WeightedRerank};
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

///////////////////////////////////////////////////////////////////////////////
// ConsistencyLevel
///////////////////////////////////////////////////////////////////////////////
/// Consistency guarantee used by query and search operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ConsistencyLevel {
    /// Represents the Strong case.
    Strong,
    /// Represents the Session case.
    Session,
    /// Represents the Bounded case.
    Bounded,
    /// Represents the Eventually case.
    Eventually,
    #[default]
    /// Represents the Customized case.
    Customized,
}

impl ConsistencyLevel {
    pub(crate) fn into_proto(self) -> common::ConsistencyLevel {
        match self {
            Self::Strong => common::ConsistencyLevel::Strong,
            Self::Session => common::ConsistencyLevel::Session,
            Self::Bounded => common::ConsistencyLevel::Bounded,
            Self::Eventually => common::ConsistencyLevel::Eventually,
            Self::Customized => common::ConsistencyLevel::Customized,
        }
    }

    pub(crate) fn from_proto(value: i32) -> Self {
        match common::ConsistencyLevel::try_from(value).ok() {
            Some(common::ConsistencyLevel::Strong) => Self::Strong,
            Some(common::ConsistencyLevel::Session) => Self::Session,
            Some(common::ConsistencyLevel::Bounded) => Self::Bounded,
            Some(common::ConsistencyLevel::Eventually) => Self::Eventually,
            Some(common::ConsistencyLevel::Customized) | None => Self::Customized,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// MetricType
///////////////////////////////////////////////////////////////////////////////
/// Distance or similarity metric used by vector indexes and searches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetricType {
    /// Lets the Milvus server determine the metric type.
    #[default]
    Default,
    /// Represents the L2 case.
    L2,
    /// Represents the Ip case.
    Ip,
    /// Represents the Cosine case.
    Cosine,
    /// Represents the Hamming case.
    Hamming,
    /// Represents the Jaccard case.
    Jaccard,
    /// Represents the MhJaccard case.
    MhJaccard,
    /// Represents the Bm25 case.
    Bm25,
    /// Represents the MaxSimCosine case.
    MaxSimCosine,
    /// Represents the MaxSimIp case.
    MaxSimIp,
    /// Represents the MaxSimL2 case.
    MaxSimL2,
    /// Represents the MaxSimJaccard case.
    MaxSimJaccard,
    /// Represents the MaxSimHamming case.
    MaxSimHamming,
}

impl MetricType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::L2 => "L2",
            Self::Ip => "IP",
            Self::Cosine => "COSINE",
            Self::Hamming => "HAMMING",
            Self::Jaccard => "JACCARD",
            Self::MhJaccard => "MHJACCARD",
            Self::Bm25 => "BM25",
            Self::MaxSimCosine => "MAX_SIM_COSINE",
            Self::MaxSimIp => "MAX_SIM_IP",
            Self::MaxSimL2 => "MAX_SIM_L2",
            Self::MaxSimJaccard => "MAX_SIM_JACCARD",
            Self::MaxSimHamming => "MAX_SIM_HAMMING",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "DEFAULT" | "INVALID" => Self::Default,
            "L2" => Self::L2,
            "IP" => Self::Ip,
            "COSINE" => Self::Cosine,
            "HAMMING" => Self::Hamming,
            "JACCARD" => Self::Jaccard,
            "MHJACCARD" => Self::MhJaccard,
            "BM25" => Self::Bm25,
            "MAX_SIM" | "MAX_SIM_COSINE" => Self::MaxSimCosine,
            "MAX_SIM_IP" => Self::MaxSimIp,
            "MAX_SIM_L2" => Self::MaxSimL2,
            "MAX_SIM_JACCARD" => Self::MaxSimJaccard,
            "MAX_SIM_HAMMING" => Self::MaxSimHamming,
            _ => Self::Default,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DataType
///////////////////////////////////////////////////////////////////////////////
/// Milvus field data type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DataType {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the Bool case.
    Bool,
    /// Represents the Int8 case.
    Int8,
    /// Represents the Int16 case.
    Int16,
    /// Represents the Int32 case.
    Int32,
    /// Represents the Int64 case.
    Int64,
    /// Represents the Float case.
    Float,
    /// Represents the Double case.
    Double,
    /// Represents the VarChar case.
    VarChar,
    /// Represents the Json case.
    Json,
    /// Represents the Geometry case.
    Geometry,
    /// Represents the Timestamptz case.
    Timestamptz,
    /// Represents the Array case.
    Array,
    /// Represents the Struct case.
    Struct,
    /// Represents the FloatVector case.
    FloatVector,
    /// Represents the BinaryVector case.
    BinaryVector,
    /// Represents the Float16Vector case.
    Float16Vector,
    /// Represents the BFloat16Vector case.
    BFloat16Vector,
    /// Represents the SparseFloatVector case.
    SparseFloatVector,
    /// Represents the Int8Vector case.
    Int8Vector,
}

impl DataType {
    pub(crate) fn into_proto(self) -> schema::DataType {
        match self {
            Self::Unknown => schema::DataType::None,
            Self::Bool => schema::DataType::Bool,
            Self::Int8 => schema::DataType::Int8,
            Self::Int16 => schema::DataType::Int16,
            Self::Int32 => schema::DataType::Int32,
            Self::Int64 => schema::DataType::Int64,
            Self::Float => schema::DataType::Float,
            Self::Double => schema::DataType::Double,
            Self::VarChar => schema::DataType::VarChar,
            Self::Json => schema::DataType::Json,
            Self::Geometry => schema::DataType::Geometry,
            Self::Timestamptz => schema::DataType::Timestamptz,
            Self::Array => schema::DataType::Array,
            Self::Struct => schema::DataType::Struct,
            Self::FloatVector => schema::DataType::FloatVector,
            Self::BinaryVector => schema::DataType::BinaryVector,
            Self::Float16Vector => schema::DataType::Float16Vector,
            Self::BFloat16Vector => schema::DataType::BFloat16Vector,
            Self::SparseFloatVector => schema::DataType::SparseFloatVector,
            Self::Int8Vector => schema::DataType::Int8Vector,
        }
    }

    pub(crate) fn try_from_proto(value: schema::DataType) -> Result<Self> {
        Ok(match value {
            schema::DataType::None => Self::Unknown,
            schema::DataType::Bool => Self::Bool,
            schema::DataType::Int8 => Self::Int8,
            schema::DataType::Int16 => Self::Int16,
            schema::DataType::Int32 => Self::Int32,
            schema::DataType::Int64 => Self::Int64,
            schema::DataType::Float => Self::Float,
            schema::DataType::Double => Self::Double,
            schema::DataType::VarChar | schema::DataType::String => Self::VarChar,
            schema::DataType::Json => Self::Json,
            schema::DataType::Geometry => Self::Geometry,
            schema::DataType::Timestamptz => Self::Timestamptz,
            schema::DataType::Array => Self::Array,
            schema::DataType::Struct => Self::Struct,
            schema::DataType::FloatVector => Self::FloatVector,
            schema::DataType::BinaryVector => Self::BinaryVector,
            schema::DataType::Float16Vector => Self::Float16Vector,
            schema::DataType::BFloat16Vector => Self::BFloat16Vector,
            schema::DataType::SparseFloatVector => Self::SparseFloatVector,
            schema::DataType::Int8Vector => Self::Int8Vector,
            _ => {
                return Err(Error::conversion(format!(
                    "unsupported protobuf data type {value:?}"
                )))
            }
        })
    }

    pub(crate) fn is_vector(self) -> bool {
        matches!(
            self,
            Self::FloatVector
                | Self::BinaryVector
                | Self::Float16Vector
                | Self::BFloat16Vector
                | Self::SparseFloatVector
                | Self::Int8Vector
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionType
///////////////////////////////////////////////////////////////////////////////
/// Type of server-side schema function.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum FunctionType {
    #[default]
    /// Represents the Unknown case.
    Unknown,
    /// Represents the Bm25 case.
    Bm25,
    /// Represents the TextEmbedding case.
    TextEmbedding,
    /// Represents the Rerank case.
    Rerank,
}

///////////////////////////////////////////////////////////////////////////////
// Function
///////////////////////////////////////////////////////////////////////////////
/// Server-side function attached to a collection schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Function {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) function_type: FunctionType,
    pub(crate) input_fields: Vec<String>,
    pub(crate) output_fields: Vec<String>,
    pub(crate) params: HashMap<String, String>,
}

impl Function {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            function_type: FunctionType::Unknown,
            input_fields: Vec::new(),
            output_fields: Vec::new(),
            params: HashMap::new(),
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

    /// Sets the function type and returns the updated value.
    pub fn function_type(mut self, value: FunctionType) -> Self {
        self.function_type = value;
        self
    }

    /// Sets the function type and returns this value for further mutation.
    pub fn set_function_type(&mut self, value: FunctionType) -> &mut Self {
        self.function_type = value;
        self
    }

    /// Returns the configured function type.
    pub fn get_function_type(&self) -> FunctionType {
        self.function_type
    }

    /// Sets the input fields and returns the updated value.
    pub fn input_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.input_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the input fields and returns this value for further mutation.
    pub fn set_input_fields(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.input_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured input fields.
    pub fn get_input_fields(&self) -> &[String] {
        &self.input_fields
    }

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.output_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the output fields and returns this value for further mutation.
    pub fn set_output_fields(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.output_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured output fields.
    pub fn get_output_fields(&self) -> &[String] {
        &self.output_fields
    }

    /// Sets the params and returns the updated value.
    pub fn params(mut self, value: HashMap<String, String>) -> Self {
        self.params = value;
        self
    }

    /// Sets the params and returns this value for further mutation.
    pub fn set_params(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.params = value;
        self
    }

    /// Returns the configured params.
    pub fn get_params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Adds one add input field to the existing values.
    pub fn add_input_field(mut self, value: impl Into<String>) -> Self {
        self.input_fields.push(value.into());
        self
    }

    /// Adds one add output field to the existing values.
    pub fn add_output_field(mut self, value: impl Into<String>) -> Self {
        self.output_fields.push(value.into());
        self
    }

    /// Sets the param and returns the updated value.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub(crate) fn into_proto(self) -> schema::FunctionSchema {
        schema::FunctionSchema {
            name: self.name,
            id: 0,
            description: self.description,
            r#type: match self.function_type {
                FunctionType::Unknown => schema::FunctionType::Unknown,
                FunctionType::Bm25 => schema::FunctionType::Bm25,
                FunctionType::TextEmbedding => schema::FunctionType::TextEmbedding,
                FunctionType::Rerank => schema::FunctionType::Rerank,
            } as i32,
            input_field_names: self.input_fields,
            input_field_ids: Vec::new(),
            output_field_names: self.output_fields,
            output_field_ids: Vec::new(),
            params: pairs(self.params),
        }
    }

    pub(crate) fn from_proto(value: schema::FunctionSchema) -> Self {
        Self {
            name: value.name,
            description: value.description,
            function_type: match schema::FunctionType::try_from(value.r#type)
                .unwrap_or(schema::FunctionType::Unknown)
            {
                schema::FunctionType::Bm25 => FunctionType::Bm25,
                schema::FunctionType::TextEmbedding => FunctionType::TextEmbedding,
                schema::FunctionType::Rerank => FunctionType::Rerank,
                _ => FunctionType::Unknown,
            },
            input_fields: value.input_field_names,
            output_fields: value.output_field_names,
            params: value.params.into_iter().map(|v| (v.key, v.value)).collect(),
        }
    }
}

impl From<RRFRerank> for Function {
    fn from(value: RRFRerank) -> Self {
        value.function
    }
}

impl From<WeightedRerank> for Function {
    fn from(value: WeightedRerank) -> Self {
        value.function
    }
}

impl From<BoostRerank> for Function {
    fn from(value: BoostRerank) -> Self {
        value.function
    }
}

impl From<DecayRerank> for Function {
    fn from(value: DecayRerank) -> Self {
        value.function
    }
}

impl From<ModelRerank> for Function {
    fn from(value: ModelRerank) -> Self {
        value.function
    }
}

///////////////////////////////////////////////////////////////////////////////
// Ids
///////////////////////////////////////////////////////////////////////////////
/// Primary-key values represented as either integers or strings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Ids {
    /// Represents the Int64 case.
    Int64(Vec<i64>),
    /// Represents the VarChar case.
    VarChar(Vec<String>),
}

impl Ids {
    /// Returns the len.
    pub fn len(&self) -> usize {
        match self {
            Self::Int64(values) => values.len(),
            Self::VarChar(values) => values.len(),
        }
    }

    /// Returns whether empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn value_at(&self, index: usize) -> Result<serde_json::Value> {
        match self {
            Self::Int64(values) => values.get(index).copied().map(serde_json::Value::from),
            Self::VarChar(values) => values.get(index).cloned().map(serde_json::Value::from),
        }
        .ok_or_else(|| {
            Error::validation(
                "index".into(),
                format!("row index {index} is out of bounds for {} rows", self.len()),
            )
        })
    }

    pub(crate) fn into_json(self) -> serde_json::Value {
        match self {
            Self::Int64(values) => {
                serde_json::Value::Array(values.into_iter().map(serde_json::Value::from).collect())
            }
            Self::VarChar(values) => {
                serde_json::Value::Array(values.into_iter().map(serde_json::Value::from).collect())
            }
        }
    }

    pub(crate) fn append(&mut self, other: Self) -> Result<()> {
        match (self, other) {
            (Self::Int64(values), Self::Int64(other)) => values.extend(other),
            (Self::VarChar(values), Self::VarChar(other)) => values.extend(other),
            _ => {
                return Err(Error::MalformedResponse(
                    "cannot combine search results with different primary-key types".into(),
                ))
            }
        }
        Ok(())
    }

    pub(crate) fn is_compatible_with(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Int64(_), Self::Int64(_)) | (Self::VarChar(_), Self::VarChar(_))
        )
    }
}

impl Default for Ids {
    fn default() -> Self {
        Self::Int64(Vec::new())
    }
}

impl Ids {
    pub(crate) fn from_proto(ids: Option<schema::IDs>) -> Self {
        match ids.and_then(|ids| ids.id_field) {
            Some(schema::i_ds::IdField::IntId(values)) => Self::Int64(values.data),
            Some(schema::i_ds::IdField::StrId(values)) => Self::VarChar(values.data),
            None => Self::default(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FieldData
///////////////////////////////////////////////////////////////////////////////
/// Column-oriented field values accepted by DML operations and returned by reads.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum FieldData {
    /// Represents the Bool case.
    Bool {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<bool>,
    },
    /// Represents the Int8 case.
    Int8 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<i8>,
    },
    /// Represents the Int16 case.
    Int16 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<i16>,
    },
    /// Represents the Int32 case.
    Int32 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<i32>,
    },
    /// Represents the Int64 case.
    Int64 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<i64>,
    },
    /// Represents the Float case.
    Float {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<f32>,
    },
    /// Represents the Double case.
    Double {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<f64>,
    },
    /// Represents the VarChar case.
    VarChar {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<String>,
    },
    /// Represents the Json case.
    Json {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<serde_json::Value>,
    },
    /// Represents the Geometry case.
    Geometry {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<String>,
    },
    /// Represents the Timestamptz case.
    Timestamptz {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<String>,
    },
    /// Represents the ArrayBool case.
    ArrayBool {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<bool>>,
    },
    /// Represents the ArrayInt8 case.
    ArrayInt8 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<i8>>,
    },
    /// Represents the ArrayInt16 case.
    ArrayInt16 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<i16>>,
    },
    /// Represents the ArrayInt32 case.
    ArrayInt32 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<i32>>,
    },
    /// Represents the ArrayInt64 case.
    ArrayInt64 {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<i64>>,
    },
    /// Represents the ArrayFloat case.
    ArrayFloat {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<f32>>,
    },
    /// Represents the ArrayDouble case.
    ArrayDouble {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<f64>>,
    },
    /// Represents the ArrayVarChar case.
    ArrayVarChar {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<String>>,
    },
    /// Represents the Struct case.
    Struct {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<StructValue>>,
    },
    /// Represents the FloatVector case.
    FloatVector {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<f32>>,
    },
    /// Represents the BinaryVector case.
    BinaryVector {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<u8>>,
    },
    /// Represents the Float16Vector case.
    Float16Vector {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<u16>>,
    },
    /// Represents the BFloat16Vector case.
    BFloat16Vector {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<u16>>,
    },
    /// Represents the SparseFloatVector case.
    SparseFloatVector {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<SparseVector>,
    },
    /// Represents the Int8Vector case.
    Int8Vector {
        /// Field name shared with the collection schema.
        name: String,
        /// Values for this field, in row order.
        values: Vec<Vec<i8>>,
    },
    /// Represents the Nullable case.
    Nullable {
        /// Wrapped field data.
        data: Box<FieldData>,
        /// Validity bitmap corresponding to `data`.
        valid_data: Vec<bool>,
    },
}

impl FieldData {
    /// Performs the boolean operation.
    pub fn boolean(name: impl Into<String>, values: Vec<bool>) -> Self {
        Self::Bool {
            name: name.into(),
            values,
        }
    }

    /// Performs the int8 operation.
    pub fn int8(name: impl Into<String>, values: Vec<i8>) -> Self {
        Self::Int8 {
            name: name.into(),
            values,
        }
    }

    /// Performs the int16 operation.
    pub fn int16(name: impl Into<String>, values: Vec<i16>) -> Self {
        Self::Int16 {
            name: name.into(),
            values,
        }
    }

    /// Performs the int32 operation.
    pub fn int32(name: impl Into<String>, values: Vec<i32>) -> Self {
        Self::Int32 {
            name: name.into(),
            values,
        }
    }

    /// Performs the int64 operation.
    pub fn int64(name: impl Into<String>, values: Vec<i64>) -> Self {
        Self::Int64 {
            name: name.into(),
            values,
        }
    }

    /// Performs the float operation.
    pub fn float(name: impl Into<String>, values: Vec<f32>) -> Self {
        Self::Float {
            name: name.into(),
            values,
        }
    }

    /// Performs the double operation.
    pub fn double(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self::Double {
            name: name.into(),
            values,
        }
    }

    /// Performs the varchar operation.
    pub fn varchar(name: impl Into<String>, values: Vec<String>) -> Self {
        Self::VarChar {
            name: name.into(),
            values,
        }
    }

    /// Performs the json operation.
    pub fn json(name: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        Self::Json {
            name: name.into(),
            values,
        }
    }

    /// Performs the geometry operation.
    pub fn geometry(name: impl Into<String>, values: Vec<String>) -> Self {
        Self::Geometry {
            name: name.into(),
            values,
        }
    }

    /// Performs the timestamptz operation.
    pub fn timestamptz(name: impl Into<String>, values: Vec<String>) -> Self {
        Self::Timestamptz {
            name: name.into(),
            values,
        }
    }

    /// Performs the array bool operation.
    pub fn array_bool(name: impl Into<String>, values: Vec<Vec<bool>>) -> Self {
        Self::ArrayBool {
            name: name.into(),
            values,
        }
    }

    /// Performs the array int8 operation.
    pub fn array_int8(name: impl Into<String>, values: Vec<Vec<i8>>) -> Self {
        Self::ArrayInt8 {
            name: name.into(),
            values,
        }
    }

    /// Performs the array int16 operation.
    pub fn array_int16(name: impl Into<String>, values: Vec<Vec<i16>>) -> Self {
        Self::ArrayInt16 {
            name: name.into(),
            values,
        }
    }

    /// Performs the array int32 operation.
    pub fn array_int32(name: impl Into<String>, values: Vec<Vec<i32>>) -> Self {
        Self::ArrayInt32 {
            name: name.into(),
            values,
        }
    }

    /// Performs the array int64 operation.
    pub fn array_int64(name: impl Into<String>, values: Vec<Vec<i64>>) -> Self {
        Self::ArrayInt64 {
            name: name.into(),
            values,
        }
    }

    /// Performs the array float operation.
    pub fn array_float(name: impl Into<String>, values: Vec<Vec<f32>>) -> Self {
        Self::ArrayFloat {
            name: name.into(),
            values,
        }
    }

    /// Performs the array double operation.
    pub fn array_double(name: impl Into<String>, values: Vec<Vec<f64>>) -> Self {
        Self::ArrayDouble {
            name: name.into(),
            values,
        }
    }

    /// Performs the array varchar operation.
    pub fn array_varchar(name: impl Into<String>, values: Vec<Vec<String>>) -> Self {
        Self::ArrayVarChar {
            name: name.into(),
            values,
        }
    }

    /// Performs the struct field operation.
    pub fn struct_field(name: impl Into<String>, values: Vec<Vec<StructValue>>) -> Self {
        Self::Struct {
            name: name.into(),
            values,
        }
    }

    /// Performs the float vector operation.
    pub fn float_vector(name: impl Into<String>, values: Vec<Vec<f32>>) -> Self {
        Self::FloatVector {
            name: name.into(),
            values,
        }
    }

    /// Performs the binary vector operation.
    pub fn binary_vector(name: impl Into<String>, values: Vec<Vec<u8>>) -> Self {
        Self::BinaryVector {
            name: name.into(),
            values,
        }
    }

    /// Performs the float16 vector operation.
    pub fn float16_vector(name: impl Into<String>, values: Vec<Vec<u16>>) -> Self {
        Self::Float16Vector {
            name: name.into(),
            values,
        }
    }

    /// Performs the bfloat16 vector operation.
    pub fn bfloat16_vector(name: impl Into<String>, values: Vec<Vec<u16>>) -> Self {
        Self::BFloat16Vector {
            name: name.into(),
            values,
        }
    }

    /// Performs the sparse float vector operation.
    pub fn sparse_float_vector(name: impl Into<String>, values: Vec<SparseVector>) -> Self {
        Self::SparseFloatVector {
            name: name.into(),
            values,
        }
    }

    /// Performs the int8 vector operation.
    pub fn int8_vector(name: impl Into<String>, values: Vec<Vec<i8>>) -> Self {
        Self::Int8Vector {
            name: name.into(),
            values,
        }
    }

    /// Returns this value configured with with validity.
    pub fn with_validity(self, valid_data: Vec<bool>) -> Result<Self> {
        Self::nullable(self, valid_data)
    }

    /// Returns the as bool.
    pub fn as_bool(&self) -> Option<&[bool]> {
        match self.inner() {
            Self::Bool { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as int8.
    pub fn as_int8(&self) -> Option<&[i8]> {
        match self.inner() {
            Self::Int8 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as int16.
    pub fn as_int16(&self) -> Option<&[i16]> {
        match self.inner() {
            Self::Int16 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as int32.
    pub fn as_int32(&self) -> Option<&[i32]> {
        match self.inner() {
            Self::Int32 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as int64.
    pub fn as_int64(&self) -> Option<&[i64]> {
        match self.inner() {
            Self::Int64 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as float.
    pub fn as_float(&self) -> Option<&[f32]> {
        match self.inner() {
            Self::Float { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as double.
    pub fn as_double(&self) -> Option<&[f64]> {
        match self.inner() {
            Self::Double { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as varchar.
    pub fn as_varchar(&self) -> Option<&[String]> {
        match self.inner() {
            Self::VarChar { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as json.
    pub fn as_json(&self) -> Option<&[serde_json::Value]> {
        match self.inner() {
            Self::Json { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as geometry.
    pub fn as_geometry(&self) -> Option<&[String]> {
        match self.inner() {
            Self::Geometry { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as timestamptz.
    pub fn as_timestamptz(&self) -> Option<&[String]> {
        match self.inner() {
            Self::Timestamptz { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array bool.
    pub fn as_array_bool(&self) -> Option<&[Vec<bool>]> {
        match self.inner() {
            Self::ArrayBool { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array int8.
    pub fn as_array_int8(&self) -> Option<&[Vec<i8>]> {
        match self.inner() {
            Self::ArrayInt8 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array int16.
    pub fn as_array_int16(&self) -> Option<&[Vec<i16>]> {
        match self.inner() {
            Self::ArrayInt16 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array int32.
    pub fn as_array_int32(&self) -> Option<&[Vec<i32>]> {
        match self.inner() {
            Self::ArrayInt32 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array int64.
    pub fn as_array_int64(&self) -> Option<&[Vec<i64>]> {
        match self.inner() {
            Self::ArrayInt64 { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array float.
    pub fn as_array_float(&self) -> Option<&[Vec<f32>]> {
        match self.inner() {
            Self::ArrayFloat { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array double.
    pub fn as_array_double(&self) -> Option<&[Vec<f64>]> {
        match self.inner() {
            Self::ArrayDouble { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as array varchar.
    pub fn as_array_varchar(&self) -> Option<&[Vec<String>]> {
        match self.inner() {
            Self::ArrayVarChar { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as struct.
    pub fn as_struct(&self) -> Option<&[Vec<StructValue>]> {
        match self.inner() {
            Self::Struct { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as float vectors.
    pub fn as_float_vectors(&self) -> Option<&[Vec<f32>]> {
        match self.inner() {
            Self::FloatVector { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as binary vectors.
    pub fn as_binary_vectors(&self) -> Option<&[Vec<u8>]> {
        match self.inner() {
            Self::BinaryVector { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as float16 vectors.
    pub fn as_float16_vectors(&self) -> Option<&[Vec<u16>]> {
        match self.inner() {
            Self::Float16Vector { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as bfloat16 vectors.
    pub fn as_bfloat16_vectors(&self) -> Option<&[Vec<u16>]> {
        match self.inner() {
            Self::BFloat16Vector { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as sparse float vectors.
    pub fn as_sparse_float_vectors(&self) -> Option<&[SparseVector]> {
        match self.inner() {
            Self::SparseFloatVector { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the as int8 vectors.
    pub fn as_int8_vectors(&self) -> Option<&[Vec<i8>]> {
        match self.inner() {
            Self::Int8Vector { values, .. } => Some(values),
            _ => None,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        match self {
            Self::Bool { name, .. }
            | Self::Int8 { name, .. }
            | Self::Int16 { name, .. }
            | Self::Int32 { name, .. }
            | Self::Int64 { name, .. }
            | Self::Float { name, .. }
            | Self::Double { name, .. }
            | Self::VarChar { name, .. }
            | Self::Json { name, .. }
            | Self::Geometry { name, .. }
            | Self::Timestamptz { name, .. }
            | Self::ArrayBool { name, .. }
            | Self::ArrayInt8 { name, .. }
            | Self::ArrayInt16 { name, .. }
            | Self::ArrayInt32 { name, .. }
            | Self::ArrayInt64 { name, .. }
            | Self::ArrayFloat { name, .. }
            | Self::ArrayDouble { name, .. }
            | Self::ArrayVarChar { name, .. }
            | Self::Struct { name, .. }
            | Self::FloatVector { name, .. }
            | Self::BinaryVector { name, .. }
            | Self::Float16Vector { name, .. }
            | Self::BFloat16Vector { name, .. }
            | Self::SparseFloatVector { name, .. }
            | Self::Int8Vector { name, .. } => name,
            Self::Nullable { data, .. } => data.name(),
        }
    }

    /// Returns the len.
    pub fn len(&self) -> usize {
        match self {
            Self::Bool { values, .. } => values.len(),
            Self::Int8 { values, .. } => values.len(),
            Self::Int16 { values, .. } => values.len(),
            Self::Int32 { values, .. } => values.len(),
            Self::Int64 { values, .. } => values.len(),
            Self::Float { values, .. } => values.len(),
            Self::Double { values, .. } => values.len(),
            Self::VarChar { values, .. } => values.len(),
            Self::Json { values, .. } => values.len(),
            Self::Geometry { values, .. } => values.len(),
            Self::Timestamptz { values, .. } => values.len(),
            Self::ArrayBool { values, .. } => values.len(),
            Self::ArrayInt8 { values, .. } => values.len(),
            Self::ArrayInt16 { values, .. } => values.len(),
            Self::ArrayInt32 { values, .. } => values.len(),
            Self::ArrayInt64 { values, .. } => values.len(),
            Self::ArrayFloat { values, .. } => values.len(),
            Self::ArrayDouble { values, .. } => values.len(),
            Self::ArrayVarChar { values, .. } => values.len(),
            Self::Struct { values, .. } => values.len(),
            Self::FloatVector { values, .. } => values.len(),
            Self::BinaryVector { values, .. } => values.len(),
            Self::Float16Vector { values, .. } => values.len(),
            Self::BFloat16Vector { values, .. } => values.len(),
            Self::SparseFloatVector { values, .. } => values.len(),
            Self::Int8Vector { values, .. } => values.len(),
            Self::Nullable { valid_data, .. } => valid_data.len(),
        }
    }

    /// Returns whether empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the valid data.
    pub fn valid_data(&self) -> Option<&[bool]> {
        match self {
            Self::Nullable { valid_data, .. } => Some(valid_data),
            _ => None,
        }
    }

    /// Returns whether null.
    pub fn is_null(&self, index: usize) -> bool {
        self.valid_data()
            .and_then(|values| values.get(index))
            .is_some_and(|valid| !valid)
    }

    /// Performs the nullable operation.
    pub fn nullable(data: FieldData, valid_data: Vec<bool>) -> Result<Self> {
        let valid_count = valid_data.iter().filter(|valid| **valid).count();
        if data.len() != valid_count {
            return Err(Error::validation(
                data.name().to_owned(),
                format!(
                    "nullable data contains {} values but validity bitmap contains {valid_count} valid rows",
                    data.len()
                ),
            ));
        }
        Ok(Self::Nullable {
            data: Box::new(data),
            valid_data,
        })
    }

    /// Returns the inner.
    pub fn inner(&self) -> &FieldData {
        match self {
            Self::Nullable { data, .. } => data,
            data => data,
        }
    }

    pub(crate) fn append(&mut self, other: FieldData) -> Result<()> {
        let compatible = match (self, other) {
            (
                Self::Bool { name, values },
                Self::Bool {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Int8 { name, values },
                Self::Int8 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Int16 { name, values },
                Self::Int16 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Int32 { name, values },
                Self::Int32 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Int64 { name, values },
                Self::Int64 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Float { name, values },
                Self::Float {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Double { name, values },
                Self::Double {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::VarChar { name, values },
                Self::VarChar {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Json { name, values },
                Self::Json {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Geometry { name, values },
                Self::Geometry {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Timestamptz { name, values },
                Self::Timestamptz {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayBool { name, values },
                Self::ArrayBool {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayInt8 { name, values },
                Self::ArrayInt8 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayInt16 { name, values },
                Self::ArrayInt16 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayInt32 { name, values },
                Self::ArrayInt32 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayInt64 { name, values },
                Self::ArrayInt64 {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayFloat { name, values },
                Self::ArrayFloat {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayDouble { name, values },
                Self::ArrayDouble {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::ArrayVarChar { name, values },
                Self::ArrayVarChar {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Struct { name, values },
                Self::Struct {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::FloatVector { name, values },
                Self::FloatVector {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::BinaryVector { name, values },
                Self::BinaryVector {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Float16Vector { name, values },
                Self::Float16Vector {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::BFloat16Vector { name, values },
                Self::BFloat16Vector {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::SparseFloatVector { name, values },
                Self::SparseFloatVector {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Int8Vector { name, values },
                Self::Int8Vector {
                    name: other_name,
                    values: other,
                },
            ) if *name == other_name => {
                values.extend(other);
                true
            }
            (
                Self::Nullable { data, valid_data },
                Self::Nullable {
                    data: other_data,
                    valid_data: other_valid_data,
                },
            ) => {
                data.append(*other_data)?;
                valid_data.extend(other_valid_data);
                true
            }
            _ => false,
        };
        if compatible {
            Ok(())
        } else {
            Err(Error::MalformedResponse(
                "cannot combine incompatible search output fields".into(),
            ))
        }
    }

    pub(crate) fn is_compatible_with(&self, other: &FieldData) -> bool {
        match (self, other) {
            (
                Self::Nullable { data, .. },
                Self::Nullable {
                    data: other_data, ..
                },
            ) => data.is_compatible_with(other_data),
            (Self::Nullable { .. }, _) | (_, Self::Nullable { .. }) => false,
            _ => {
                self.name() == other.name()
                    && self.data_type() == other.data_type()
                    && self.array_element_type() == other.array_element_type()
                    && self.dimension() == other.dimension()
            }
        }
    }

    pub(crate) fn value_at(&self, index: usize) -> Result<serde_json::Value> {
        use serde_json::{json, Value};

        fn value<T: serde::Serialize>(values: &[T], index: usize) -> Result<Value> {
            values
                .get(index)
                .ok_or_else(|| {
                    Error::validation(
                        "index".into(),
                        format!(
                            "row index {index} is out of bounds for {} rows",
                            values.len()
                        ),
                    )
                })
                .and_then(|value| serde_json::to_value(value).map_err(Into::into))
        }

        match self {
            Self::Bool { values, .. } => value(values, index),
            Self::Int8 { values, .. } => value(values, index),
            Self::Int16 { values, .. } => value(values, index),
            Self::Int32 { values, .. } => value(values, index),
            Self::Int64 { values, .. } => value(values, index),
            Self::Float { values, .. } => value(values, index),
            Self::Double { values, .. } => value(values, index),
            Self::VarChar { values, .. } => value(values, index),
            Self::Json { values, .. } => value(values, index),
            Self::Geometry { values, .. } => value(values, index),
            Self::Timestamptz { values, .. } => value(values, index),
            Self::ArrayBool { values, .. } => value(values, index),
            Self::ArrayInt8 { values, .. } => value(values, index),
            Self::ArrayInt16 { values, .. } => value(values, index),
            Self::ArrayInt32 { values, .. } => value(values, index),
            Self::ArrayInt64 { values, .. } => value(values, index),
            Self::ArrayFloat { values, .. } => value(values, index),
            Self::ArrayDouble { values, .. } => value(values, index),
            Self::ArrayVarChar { values, .. } => value(values, index),
            Self::Struct { values, .. } => value(values, index),
            Self::FloatVector { values, .. } => value(values, index),
            Self::BinaryVector { values, .. } => value(values, index),
            Self::Float16Vector { values, .. } => value(values, index),
            Self::BFloat16Vector { values, .. } => value(values, index),
            Self::SparseFloatVector { values, .. } => Ok(Value::Object(
                values
                    .get(index)
                    .ok_or_else(|| {
                        Error::validation(
                            "index".into(),
                            format!(
                                "row index {index} is out of bounds for {} rows",
                                values.len()
                            ),
                        )
                    })?
                    .iter()
                    .map(|(key, value)| (key.to_string(), json!(value)))
                    .collect(),
            )),
            Self::Int8Vector { values, .. } => value(values, index),
            Self::Nullable { data, valid_data } => {
                let valid = *valid_data.get(index).ok_or_else(|| {
                    Error::validation(
                        "index".into(),
                        format!(
                            "row index {index} is out of bounds for {} rows",
                            valid_data.len()
                        ),
                    )
                })?;
                if !valid {
                    return Ok(Value::Null);
                }
                let compact_index = valid_data[..index].iter().filter(|valid| **valid).count();
                data.value_at(compact_index)
            }
        }
    }

    pub(crate) fn data_type(&self) -> DataType {
        match self {
            Self::Bool { .. } => DataType::Bool,
            Self::Int8 { .. } => DataType::Int8,
            Self::Int16 { .. } => DataType::Int16,
            Self::Int32 { .. } => DataType::Int32,
            Self::Int64 { .. } => DataType::Int64,
            Self::Float { .. } => DataType::Float,
            Self::Double { .. } => DataType::Double,
            Self::VarChar { .. } => DataType::VarChar,
            Self::Json { .. } => DataType::Json,
            Self::Geometry { .. } => DataType::Geometry,
            Self::Timestamptz { .. } => DataType::Timestamptz,
            Self::ArrayBool { .. }
            | Self::ArrayInt8 { .. }
            | Self::ArrayInt16 { .. }
            | Self::ArrayInt32 { .. }
            | Self::ArrayInt64 { .. }
            | Self::ArrayFloat { .. }
            | Self::ArrayDouble { .. }
            | Self::ArrayVarChar { .. } => DataType::Array,
            Self::Struct { .. } => DataType::Struct,
            Self::FloatVector { .. } => DataType::FloatVector,
            Self::BinaryVector { .. } => DataType::BinaryVector,
            Self::Float16Vector { .. } => DataType::Float16Vector,
            Self::BFloat16Vector { .. } => DataType::BFloat16Vector,
            Self::SparseFloatVector { .. } => DataType::SparseFloatVector,
            Self::Int8Vector { .. } => DataType::Int8Vector,
            Self::Nullable { data, .. } => data.data_type(),
        }
    }

    pub(crate) fn dimension(&self) -> Option<usize> {
        match self {
            Self::FloatVector { values, .. } => values.first().map(Vec::len),
            Self::Float16Vector { values, .. } => values.first().map(Vec::len),
            Self::BFloat16Vector { values, .. } => values.first().map(Vec::len),
            Self::BinaryVector { values, .. } => {
                values.first().and_then(|value| value.len().checked_mul(8))
            }
            Self::Int8Vector { values, .. } => values.first().map(Vec::len),
            Self::Nullable { data, .. } => data.dimension(),
            _ => None,
        }
    }

    pub(crate) fn array_element_type(&self) -> Option<DataType> {
        match self {
            Self::ArrayBool { .. } => Some(DataType::Bool),
            Self::ArrayInt8 { .. } => Some(DataType::Int8),
            Self::ArrayInt16 { .. } => Some(DataType::Int16),
            Self::ArrayInt32 { .. } => Some(DataType::Int32),
            Self::ArrayInt64 { .. } => Some(DataType::Int64),
            Self::ArrayFloat { .. } => Some(DataType::Float),
            Self::ArrayDouble { .. } => Some(DataType::Double),
            Self::ArrayVarChar { .. } => Some(DataType::VarChar),
            Self::Nullable { data, .. } => data.array_element_type(),
            _ => None,
        }
    }

    pub(crate) fn validate_value_constraints(&self, field: &schema::FieldSchema) -> Result<()> {
        match self {
            Self::Nullable { data, .. } => data.validate_value_constraints(field),
            Self::VarChar { name, values }
            | Self::Geometry { name, values }
            | Self::Timestamptz { name, values } => {
                let max_length = field
                    .type_params
                    .iter()
                    .find(|pair| pair.key == "max_length")
                    .and_then(|pair| pair.value.parse::<usize>().ok());
                if let Some(max_length) = max_length {
                    if values.iter().any(|value| value.len() > max_length) {
                        return Err(Error::validation(
                            name.clone(),
                            format!("string exceeds max_length {max_length}"),
                        ));
                    }
                }
                Ok(())
            }
            Self::ArrayBool { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayInt8 { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayInt16 { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayInt32 { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayInt64 { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayFloat { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayDouble { name, values } => validate_array_capacity(name, values, field),
            Self::ArrayVarChar { name, values } => validate_array_capacity(name, values, field),
            _ => Ok(()),
        }
    }

    pub(crate) fn into_proto(self) -> Result<schema::FieldData> {
        use schema::{field_data, scalar_field, vector_field};
        if let Self::Nullable { data, valid_data } = self {
            let valid_count = valid_data.iter().filter(|valid| **valid).count();
            if data.len() != valid_count {
                return Err(Error::validation(
                    data.name().to_owned(),
                    "nullable data and validity bitmap have inconsistent lengths".into(),
                ));
            }
            let mut proto = data.into_proto()?;
            proto.valid_data = valid_data;
            return Ok(proto);
        }
        let (name, data_type, field) = match self {
            Self::Bool { name, values } => (
                name,
                schema::DataType::Bool,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::BoolData(schema::BoolArray {
                        data: values,
                    })),
                }),
            ),
            Self::Int8 { name, values } => (
                name,
                schema::DataType::Int8,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: values.into_iter().map(i32::from).collect(),
                    })),
                }),
            ),
            Self::Int16 { name, values } => (
                name,
                schema::DataType::Int16,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: values.into_iter().map(i32::from).collect(),
                    })),
                }),
            ),
            Self::Int32 { name, values } => (
                name,
                schema::DataType::Int32,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: values,
                    })),
                }),
            ),
            Self::Int64 { name, values } => (
                name,
                schema::DataType::Int64,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::LongData(schema::LongArray {
                        data: values,
                    })),
                }),
            ),
            Self::Float { name, values } => (
                name,
                schema::DataType::Float,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::FloatData(schema::FloatArray {
                        data: values,
                    })),
                }),
            ),
            Self::Double { name, values } => (
                name,
                schema::DataType::Double,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::DoubleData(schema::DoubleArray {
                        data: values,
                    })),
                }),
            ),
            Self::VarChar { name, values } => (
                name,
                schema::DataType::VarChar,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::StringData(schema::StringArray {
                        data: values,
                    })),
                }),
            ),
            Self::Json { name, values } => (
                name,
                schema::DataType::Json,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::JsonData(schema::JsonArray {
                        data: values
                            .into_iter()
                            .map(|value| serde_json::to_vec(&value))
                            .collect::<std::result::Result<Vec<_>, _>>()?,
                    })),
                }),
            ),
            Self::Geometry { name, values } => (
                name,
                schema::DataType::Geometry,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::GeometryWktData(
                        schema::GeometryWktArray { data: values },
                    )),
                }),
            ),
            Self::Timestamptz { name, values } => (
                name,
                schema::DataType::Timestamptz,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::StringData(schema::StringArray {
                        data: values,
                    })),
                }),
            ),
            Self::ArrayBool { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Bool, values, |data| {
                    scalar_field::Data::BoolData(schema::BoolArray { data })
                }),
            ),
            Self::ArrayInt8 { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Int8, values, |data| {
                    scalar_field::Data::IntData(schema::IntArray {
                        data: data.into_iter().map(i32::from).collect(),
                    })
                }),
            ),
            Self::ArrayInt16 { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Int16, values, |data| {
                    scalar_field::Data::IntData(schema::IntArray {
                        data: data.into_iter().map(i32::from).collect(),
                    })
                }),
            ),
            Self::ArrayInt32 { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Int32, values, |data| {
                    scalar_field::Data::IntData(schema::IntArray { data })
                }),
            ),
            Self::ArrayInt64 { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Int64, values, |data| {
                    scalar_field::Data::LongData(schema::LongArray { data })
                }),
            ),
            Self::ArrayFloat { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Float, values, |data| {
                    scalar_field::Data::FloatData(schema::FloatArray { data })
                }),
            ),
            Self::ArrayDouble { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::Double, values, |data| {
                    scalar_field::Data::DoubleData(schema::DoubleArray { data })
                }),
            ),
            Self::ArrayVarChar { name, values } => (
                name,
                schema::DataType::Array,
                array_proto_field(DataType::VarChar, values, |data| {
                    scalar_field::Data::StringData(schema::StringArray { data })
                }),
            ),
            Self::Struct { name, .. } => {
                return Err(Error::validation(
                    name,
                    "struct field data requires its collection schema".into(),
                ))
            }
            Self::FloatVector { name, values } => {
                let dimension = vector_dimension(&name, &values)?;
                let data = values.into_iter().flatten().collect();
                (
                    name,
                    schema::DataType::FloatVector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::FloatVector(schema::FloatArray { data })),
                    }),
                )
            }
            Self::BinaryVector { name, values } => {
                let width = vector_dimension(&name, &values)?;
                let dimension = width.checked_mul(8).ok_or_else(|| {
                    Error::validation(name.clone(), "binary vector dimension overflows".into())
                })?;
                (
                    name,
                    schema::DataType::BinaryVector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::BinaryVector(
                            values.into_iter().flatten().collect(),
                        )),
                    }),
                )
            }
            Self::Float16Vector { name, values } => {
                let dimension = vector_dimension(&name, &values)?;
                let data = encode_u16_vectors(values);
                (
                    name,
                    schema::DataType::Float16Vector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::Float16Vector(data)),
                    }),
                )
            }
            Self::BFloat16Vector { name, values } => {
                let dimension = vector_dimension(&name, &values)?;
                let data = encode_u16_vectors(values);
                (
                    name,
                    schema::DataType::BFloat16Vector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::Bfloat16Vector(data)),
                    }),
                )
            }
            Self::SparseFloatVector { name, values } => {
                let dimension = values
                    .iter()
                    .flat_map(|value| value.keys().copied())
                    .max()
                    .map_or(0, |value| i64::from(value) + 1);
                let contents = values
                    .into_iter()
                    .map(|values| encode_sparse_vector(&name, values))
                    .collect::<Result<_>>()?;
                (
                    name,
                    schema::DataType::SparseFloatVector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension,
                        data: Some(vector_field::Data::SparseFloatVector(
                            schema::SparseFloatArray {
                                contents,
                                dim: dimension,
                            },
                        )),
                    }),
                )
            }
            Self::Int8Vector { name, values } => {
                let dimension = vector_dimension(&name, &values)?;
                (
                    name,
                    schema::DataType::Int8Vector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::Int8Vector(
                            values
                                .into_iter()
                                .flatten()
                                .map(|value| value as u8)
                                .collect(),
                        )),
                    }),
                )
            }
            Self::Nullable { .. } => unreachable!(),
        };

        Ok(schema::FieldData {
            r#type: data_type as i32,
            field_name: name,
            field_id: 0,
            is_dynamic: false,
            valid_data: Vec::new(),
            field: Some(field),
        })
    }

    /// Encodes a borrowed column directly into its final protobuf buffers.
    ///
    /// DML keeps the SDK payload available for a possible schema refresh, so
    /// this path must not clone the whole `FieldData` before flattening vectors.
    pub(crate) fn to_proto(&self) -> Result<schema::FieldData> {
        use schema::{field_data, scalar_field, vector_field};
        if let Self::Nullable { data, valid_data } = self {
            let valid_count = valid_data.iter().filter(|valid| **valid).count();
            if data.len() != valid_count {
                return Err(Error::validation(
                    data.name().to_owned(),
                    "nullable data and validity bitmap have inconsistent lengths".into(),
                ));
            }
            let mut proto = data.to_proto()?;
            proto.valid_data = valid_data.clone();
            return Ok(proto);
        }
        let (name, data_type, field) = match self {
            Self::Bool { name, values } => (
                name.clone(),
                schema::DataType::Bool,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::BoolData(schema::BoolArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::Int8 { name, values } => (
                name.clone(),
                schema::DataType::Int8,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: values.iter().copied().map(i32::from).collect(),
                    })),
                }),
            ),
            Self::Int16 { name, values } => (
                name.clone(),
                schema::DataType::Int16,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: values.iter().copied().map(i32::from).collect(),
                    })),
                }),
            ),
            Self::Int32 { name, values } => (
                name.clone(),
                schema::DataType::Int32,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::Int64 { name, values } => (
                name.clone(),
                schema::DataType::Int64,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::LongData(schema::LongArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::Float { name, values } => (
                name.clone(),
                schema::DataType::Float,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::FloatData(schema::FloatArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::Double { name, values } => (
                name.clone(),
                schema::DataType::Double,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::DoubleData(schema::DoubleArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::VarChar { name, values } => (
                name.clone(),
                schema::DataType::VarChar,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::StringData(schema::StringArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::Json { name, values } => (
                name.clone(),
                schema::DataType::Json,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::JsonData(schema::JsonArray {
                        data: values
                            .iter()
                            .map(serde_json::to_vec)
                            .collect::<std::result::Result<Vec<_>, _>>()?,
                    })),
                }),
            ),
            Self::Geometry { name, values } => (
                name.clone(),
                schema::DataType::Geometry,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::GeometryWktData(
                        schema::GeometryWktArray {
                            data: values.clone(),
                        },
                    )),
                }),
            ),
            Self::Timestamptz { name, values } => (
                name.clone(),
                schema::DataType::Timestamptz,
                field_data::Field::Scalars(schema::ScalarField {
                    data: Some(scalar_field::Data::StringData(schema::StringArray {
                        data: values.clone(),
                    })),
                }),
            ),
            Self::ArrayBool { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Bool, values, |data| {
                    scalar_field::Data::BoolData(schema::BoolArray { data })
                }),
            ),
            Self::ArrayInt8 { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Int8, values, |data| {
                    scalar_field::Data::IntData(schema::IntArray {
                        data: data.into_iter().map(i32::from).collect(),
                    })
                }),
            ),
            Self::ArrayInt16 { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Int16, values, |data| {
                    scalar_field::Data::IntData(schema::IntArray {
                        data: data.into_iter().map(i32::from).collect(),
                    })
                }),
            ),
            Self::ArrayInt32 { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Int32, values, |data| {
                    scalar_field::Data::IntData(schema::IntArray { data })
                }),
            ),
            Self::ArrayInt64 { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Int64, values, |data| {
                    scalar_field::Data::LongData(schema::LongArray { data })
                }),
            ),
            Self::ArrayFloat { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Float, values, |data| {
                    scalar_field::Data::FloatData(schema::FloatArray { data })
                }),
            ),
            Self::ArrayDouble { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::Double, values, |data| {
                    scalar_field::Data::DoubleData(schema::DoubleArray { data })
                }),
            ),
            Self::ArrayVarChar { name, values } => (
                name.clone(),
                schema::DataType::Array,
                array_proto_field_by_ref(DataType::VarChar, values, |data| {
                    scalar_field::Data::StringData(schema::StringArray { data })
                }),
            ),
            Self::Struct { name, .. } => {
                return Err(Error::validation(
                    name.clone(),
                    "struct field data requires its collection schema".into(),
                ))
            }
            Self::FloatVector { name, values } => {
                let dimension = vector_dimension(name, values)?;
                let data = values.iter().flatten().copied().collect();
                (
                    name.clone(),
                    schema::DataType::FloatVector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::FloatVector(schema::FloatArray { data })),
                    }),
                )
            }
            Self::BinaryVector { name, values } => {
                let width = vector_dimension(name, values)?;
                let dimension = width.checked_mul(8).ok_or_else(|| {
                    Error::validation(name.clone(), "binary vector dimension overflows".into())
                })?;
                (
                    name.clone(),
                    schema::DataType::BinaryVector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::BinaryVector(
                            values.iter().flatten().copied().collect(),
                        )),
                    }),
                )
            }
            Self::Float16Vector { name, values } => {
                let dimension = vector_dimension(name, values)?;
                let data = encode_u16_vectors_by_ref(values);
                (
                    name.clone(),
                    schema::DataType::Float16Vector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::Float16Vector(data)),
                    }),
                )
            }
            Self::BFloat16Vector { name, values } => {
                let dimension = vector_dimension(name, values)?;
                let data = encode_u16_vectors_by_ref(values);
                (
                    name.clone(),
                    schema::DataType::BFloat16Vector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::Bfloat16Vector(data)),
                    }),
                )
            }
            Self::SparseFloatVector { name, values } => {
                let dimension = values
                    .iter()
                    .flat_map(|value| value.keys().copied())
                    .max()
                    .map_or(0, |value| i64::from(value) + 1);
                let contents = values
                    .iter()
                    .map(|values| encode_sparse_vector_by_ref(name, values))
                    .collect::<Result<_>>()?;
                (
                    name.clone(),
                    schema::DataType::SparseFloatVector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension,
                        data: Some(vector_field::Data::SparseFloatVector(
                            schema::SparseFloatArray {
                                contents,
                                dim: dimension,
                            },
                        )),
                    }),
                )
            }
            Self::Int8Vector { name, values } => {
                let dimension = vector_dimension(name, values)?;
                (
                    name.clone(),
                    schema::DataType::Int8Vector,
                    field_data::Field::Vectors(schema::VectorField {
                        dim: dimension as i64,
                        data: Some(vector_field::Data::Int8Vector(
                            values
                                .iter()
                                .flatten()
                                .copied()
                                .map(|value| value as u8)
                                .collect(),
                        )),
                    }),
                )
            }
            Self::Nullable { .. } => unreachable!(),
        };

        Ok(schema::FieldData {
            r#type: data_type as i32,
            field_name: name,
            field_id: 0,
            is_dynamic: false,
            valid_data: Vec::new(),
            field: Some(field),
        })
    }

    pub(crate) fn into_proto_with_schema(
        self,
        field: &schema::FieldSchema,
    ) -> Result<schema::FieldData> {
        let mut proto = self.into_proto()?;
        proto.r#type = field.data_type;
        proto.field_id = field.field_id;
        proto.is_dynamic = field.is_dynamic;
        if let Some(schema::field_data::Field::Vectors(vectors)) = proto.field.as_mut() {
            if let Some(dimension) = field
                .type_params
                .iter()
                .find(|pair| pair.key == "dim")
                .and_then(|pair| pair.value.parse::<i64>().ok())
            {
                vectors.dim = dimension;
            }
        }
        Ok(proto)
    }

    pub(crate) fn to_proto_with_schema(
        &self,
        field: &schema::FieldSchema,
    ) -> Result<schema::FieldData> {
        let mut proto = self.to_proto()?;
        proto.r#type = field.data_type;
        proto.field_id = field.field_id;
        proto.is_dynamic = field.is_dynamic;
        if let Some(schema::field_data::Field::Vectors(vectors)) = proto.field.as_mut() {
            if let Some(dimension) = field
                .type_params
                .iter()
                .find(|pair| pair.key == "dim")
                .and_then(|pair| pair.value.parse::<i64>().ok())
            {
                vectors.dim = dimension;
            }
        }
        Ok(proto)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RetryConfig
///////////////////////////////////////////////////////////////////////////////
/// Retry policy applied to V2 RPC calls.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RetryConfig {
    /// Maximum number of attempts, including the initial RPC call.
    pub(crate) max_attempts: u32,
    /// Maximum wall-clock time spent retrying one RPC. Zero disables this limit.
    pub(crate) max_retry_timeout: Duration,
    /// Delay before the second attempt.
    pub(crate) initial_backoff: Duration,
    /// Upper bound for the delay between attempts.
    pub(crate) max_backoff: Duration,
    /// Multiplier applied to the delay after each failed attempt.
    pub(crate) backoff_multiplier: f64,
    /// Whether server-side rate-limit responses may be retried.
    pub(crate) retry_on_rate_limit: bool,
}

impl RetryConfig {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            max_attempts: 75,
            max_retry_timeout: Duration::ZERO,
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_secs(3),
            backoff_multiplier: 3.0,
            retry_on_rate_limit: true,
        }
    }

    /// Sets the max attempts and returns the updated value.
    pub fn max_attempts(mut self, value: u32) -> Self {
        self.max_attempts = value;
        self
    }

    /// Sets the max attempts and returns this value for further mutation.
    pub fn set_max_attempts(&mut self, value: u32) -> &mut Self {
        self.max_attempts = value;
        self
    }

    /// Returns the configured max attempts.
    pub fn get_max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Sets the max retry timeout and returns the updated value.
    pub fn max_retry_timeout(mut self, value: Duration) -> Self {
        self.max_retry_timeout = value;
        self
    }

    /// Sets the max retry timeout and returns this value for further mutation.
    pub fn set_max_retry_timeout(&mut self, value: Duration) -> &mut Self {
        self.max_retry_timeout = value;
        self
    }

    /// Returns the configured max retry timeout.
    pub fn get_max_retry_timeout(&self) -> Duration {
        self.max_retry_timeout
    }

    /// Sets the initial backoff and returns the updated value.
    pub fn initial_backoff(mut self, value: Duration) -> Self {
        self.initial_backoff = value;
        self
    }

    /// Sets the initial backoff and returns this value for further mutation.
    pub fn set_initial_backoff(&mut self, value: Duration) -> &mut Self {
        self.initial_backoff = value;
        self
    }

    /// Returns the configured initial backoff.
    pub fn get_initial_backoff(&self) -> Duration {
        self.initial_backoff
    }

    /// Sets the max backoff and returns the updated value.
    pub fn max_backoff(mut self, value: Duration) -> Self {
        self.max_backoff = value;
        self
    }

    /// Sets the max backoff and returns this value for further mutation.
    pub fn set_max_backoff(&mut self, value: Duration) -> &mut Self {
        self.max_backoff = value;
        self
    }

    /// Returns the configured max backoff.
    pub fn get_max_backoff(&self) -> Duration {
        self.max_backoff
    }

    /// Sets the backoff multiplier and returns the updated value.
    pub fn backoff_multiplier(mut self, value: f64) -> Self {
        self.backoff_multiplier = value;
        self
    }

    /// Sets the backoff multiplier and returns this value for further mutation.
    pub fn set_backoff_multiplier(&mut self, value: f64) -> &mut Self {
        self.backoff_multiplier = value;
        self
    }

    /// Returns the configured backoff multiplier.
    pub fn get_backoff_multiplier(&self) -> f64 {
        self.backoff_multiplier
    }

    /// Sets the retry on rate limit and returns the updated value.
    pub fn retry_on_rate_limit(mut self, value: bool) -> Self {
        self.retry_on_rate_limit = value;
        self
    }

    /// Sets the retry on rate limit and returns this value for further mutation.
    pub fn set_retry_on_rate_limit(&mut self, value: bool) -> &mut Self {
        self.retry_on_rate_limit = value;
        self
    }

    /// Returns the configured retry on rate limit.
    pub fn get_retry_on_rate_limit(&self) -> bool {
        self.retry_on_rate_limit
    }
}

///////////////////////////////////////////////////////////////////////////////
// ConnectConfig
///////////////////////////////////////////////////////////////////////////////
/// Connection settings used to create a ClientV2.
#[derive(Clone)]
#[non_exhaustive]
pub struct ConnectConfig {
    pub(crate) uri: String,
    pub(crate) token: Option<String>,
    pub(crate) tls_server_name: Option<String>,
    pub(crate) ca_certificate: Option<String>,
    pub(crate) client_certificate: Option<String>,
    pub(crate) client_key: Option<String>,
    pub(crate) connect_timeout: Duration,
    pub(crate) rpc_timeout: Duration,
    pub(crate) keepalive_time: Duration,
    pub(crate) keepalive_timeout: Duration,
    pub(crate) keepalive_while_idle: bool,
    pub(crate) database: String,
    pub(crate) retry: RetryConfig,
}

impl std::fmt::Debug for ConnectConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = self.token.as_ref().map(|_| "[REDACTED]");
        let client_key = self.client_key.as_ref().map(|_| "[REDACTED]");
        formatter
            .debug_struct("ConnectConfig")
            .field("uri", &self.uri)
            .field("token", &token)
            .field("tls_server_name", &self.tls_server_name)
            .field("ca_certificate", &self.ca_certificate)
            .field("client_certificate", &self.client_certificate)
            .field("client_key", &client_key)
            .field("connect_timeout", &self.connect_timeout)
            .field("rpc_timeout", &self.rpc_timeout)
            .field("keepalive_time", &self.keepalive_time)
            .field("keepalive_timeout", &self.keepalive_timeout)
            .field("keepalive_while_idle", &self.keepalive_while_idle)
            .field("database", &self.database)
            .field("retry", &self.retry)
            .finish()
    }
}

impl ConnectConfig {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            uri: "http://localhost:19530".to_owned(),
            token: None,
            tls_server_name: None,
            ca_certificate: None,
            client_certificate: None,
            client_key: None,
            connect_timeout: Duration::from_secs(10),
            rpc_timeout: Duration::ZERO,
            keepalive_time: Duration::from_secs(10),
            keepalive_timeout: Duration::from_secs(5),
            keepalive_while_idle: true,
            database: String::new(),
            retry: RetryConfig::new(),
        }
    }

    /// Sets the uri and returns the updated value.
    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = uri.into();
        self
    }

    /// Sets the uri and returns this value for further mutation.
    pub fn set_uri(&mut self, uri: impl Into<String>) -> &mut Self {
        self.uri = uri.into();
        self
    }

    /// Returns the configured uri.
    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    /// Sets the token and returns the updated value.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        use base64::Engine;
        self.token = Some(base64::engine::general_purpose::STANDARD.encode(token.into()));
        self
    }

    /// Sets the token and returns this value for further mutation.
    pub fn set_token(&mut self, token: impl Into<String>) -> &mut Self {
        use base64::Engine;
        self.token = Some(base64::engine::general_purpose::STANDARD.encode(token.into()));
        self
    }

    /// Returns the configured token.
    pub fn get_token(&self) -> &Option<String> {
        &self.token
    }

    /// Overrides the DNS name used to verify the Milvus server's TLS certificate.
    pub fn tls_server_name(mut self, value: impl Into<String>) -> Self {
        self.tls_server_name = optional_config_string(value);
        self
    }

    /// Updates the DNS name used to verify the Milvus server's TLS certificate.
    pub fn set_tls_server_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.tls_server_name = optional_config_string(value);
        self
    }

    /// Returns the configured TLS server-name override.
    pub fn get_tls_server_name(&self) -> &Option<String> {
        &self.tls_server_name
    }

    /// Sets the path to a PEM-encoded custom CA certificate file.
    pub fn ca_certificate(mut self, value: impl Into<String>) -> Self {
        self.ca_certificate = optional_config_string(value);
        self
    }

    /// Updates the path to a PEM-encoded custom CA certificate file.
    pub fn set_ca_certificate(&mut self, value: impl Into<String>) -> &mut Self {
        self.ca_certificate = optional_config_string(value);
        self
    }

    /// Returns the custom CA certificate file path.
    pub fn get_ca_certificate(&self) -> &Option<String> {
        &self.ca_certificate
    }

    /// Sets the path to the PEM-encoded client certificate file used for mutual TLS.
    pub fn client_certificate(mut self, value: impl Into<String>) -> Self {
        self.client_certificate = optional_config_string(value);
        self
    }

    /// Updates the path to the PEM-encoded client certificate file used for mutual TLS.
    pub fn set_client_certificate(&mut self, value: impl Into<String>) -> &mut Self {
        self.client_certificate = optional_config_string(value);
        self
    }

    /// Returns the client certificate file path.
    pub fn get_client_certificate(&self) -> &Option<String> {
        &self.client_certificate
    }

    /// Sets the path to the PEM-encoded client private-key file used for mutual TLS.
    pub fn client_key(mut self, value: impl Into<String>) -> Self {
        self.client_key = optional_config_string(value);
        self
    }

    /// Updates the path to the PEM-encoded client private-key file used for mutual TLS.
    pub fn set_client_key(&mut self, value: impl Into<String>) -> &mut Self {
        self.client_key = optional_config_string(value);
        self
    }

    /// Returns the client private-key file path.
    pub fn get_client_key(&self) -> &Option<String> {
        &self.client_key
    }

    /// Sets the maximum time ClientV2::new waits for Milvus to become ready.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Updates the maximum time ClientV2::new waits for Milvus to become ready.
    pub fn set_connect_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.connect_timeout = timeout;
        self
    }

    /// Returns the maximum time ClientV2::new waits for Milvus to become ready.
    pub fn get_connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// Sets the rpc timeout and returns the updated value.
    pub fn rpc_timeout(mut self, timeout: Duration) -> Self {
        self.rpc_timeout = timeout;
        self
    }

    /// Sets the rpc timeout and returns this value for further mutation.
    pub fn set_rpc_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.rpc_timeout = timeout;
        self
    }

    /// Returns the configured rpc timeout.
    pub fn get_rpc_timeout(&self) -> Duration {
        self.rpc_timeout
    }

    /// Sets the keepalive time and returns the updated value.
    pub fn keepalive_time(mut self, value: Duration) -> Self {
        self.keepalive_time = value;
        self
    }

    /// Sets the keepalive time and returns this value for further mutation.
    pub fn set_keepalive_time(&mut self, value: Duration) -> &mut Self {
        self.keepalive_time = value;
        self
    }

    /// Returns the configured keepalive time.
    pub fn get_keepalive_time(&self) -> Duration {
        self.keepalive_time
    }

    /// Sets the keepalive timeout and returns the updated value.
    pub fn keepalive_timeout(mut self, value: Duration) -> Self {
        self.keepalive_timeout = value;
        self
    }

    /// Sets the keepalive timeout and returns this value for further mutation.
    pub fn set_keepalive_timeout(&mut self, value: Duration) -> &mut Self {
        self.keepalive_timeout = value;
        self
    }

    /// Returns the configured keepalive timeout.
    pub fn get_keepalive_timeout(&self) -> Duration {
        self.keepalive_timeout
    }

    /// Sets the keepalive while idle and returns the updated value.
    pub fn keepalive_while_idle(mut self, value: bool) -> Self {
        self.keepalive_while_idle = value;
        self
    }

    /// Sets the keepalive while idle and returns this value for further mutation.
    pub fn set_keepalive_while_idle(&mut self, value: bool) -> &mut Self {
        self.keepalive_while_idle = value;
        self
    }

    /// Returns the configured keepalive while idle.
    pub fn get_keepalive_while_idle(&self) -> bool {
        self.keepalive_while_idle
    }

    /// Sets the database and returns the updated value.
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = database.into();
        self
    }

    /// Sets the database and returns this value for further mutation.
    pub fn set_database(&mut self, database: impl Into<String>) -> &mut Self {
        self.database = database.into();
        self
    }

    /// Returns the configured database.
    pub fn get_database(&self) -> &str {
        &self.database
    }

    /// Sets the retry and returns the updated value.
    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    /// Sets the retry and returns this value for further mutation.
    pub fn set_retry(&mut self, retry: RetryConfig) -> &mut Self {
        self.retry = retry;
        self
    }

    /// Returns the configured retry.
    pub fn get_retry(&self) -> &RetryConfig {
        &self.retry
    }

    /// Performs the username password operation.
    pub fn username_password(self, username: &str, password: &str) -> Self {
        self.token(format!("{username}:{password}"))
    }
}

fn optional_config_string(value: impl Into<String>) -> Option<String> {
    let value = value.into();
    (!value.is_empty()).then_some(value)
}

pub(super) fn pairs(values: HashMap<String, String>) -> Vec<common::KeyValuePair> {
    values
        .into_iter()
        .map(|(key, value)| common::KeyValuePair { key, value })
        .collect()
}

/// JSON object used as the value of a Milvus struct field.
pub type StructValue = serde_json::Map<String, serde_json::Value>;

/// JSON object representing one row for row-oriented DML input.
pub type EntityRow = serde_json::Map<String, serde_json::Value>;

/// Sparse vector represented by dimension indexes and values.
pub type SparseVector = BTreeMap<u32, f32>;

fn validate_array_capacity<T>(
    name: &str,
    values: &[Vec<T>],
    field: &schema::FieldSchema,
) -> Result<()> {
    let max_capacity = field
        .type_params
        .iter()
        .find(|pair| pair.key == "max_capacity")
        .and_then(|pair| pair.value.parse::<usize>().ok());
    if max_capacity.is_some_and(|limit| values.iter().any(|value| value.len() > limit)) {
        return Err(Error::validation(
            name.to_owned(),
            format!("array exceeds max_capacity {}", max_capacity.unwrap()),
        ));
    }
    Ok(())
}

fn array_proto_field<T>(
    element_type: DataType,
    values: Vec<Vec<T>>,
    encode: impl Fn(Vec<T>) -> schema::scalar_field::Data,
) -> schema::field_data::Field {
    schema::field_data::Field::Scalars(schema::ScalarField {
        data: Some(schema::scalar_field::Data::ArrayData(schema::ArrayArray {
            data: values
                .into_iter()
                .map(|values| schema::ScalarField {
                    data: Some(encode(values)),
                })
                .collect(),
            element_type: element_type.into_proto() as i32,
        })),
    })
}

fn array_proto_field_by_ref<T: Clone>(
    element_type: DataType,
    values: &[Vec<T>],
    encode: impl Fn(Vec<T>) -> schema::scalar_field::Data,
) -> schema::field_data::Field {
    schema::field_data::Field::Scalars(schema::ScalarField {
        data: Some(schema::scalar_field::Data::ArrayData(schema::ArrayArray {
            data: values
                .iter()
                .map(|values| schema::ScalarField {
                    data: Some(encode(values.clone())),
                })
                .collect(),
            element_type: element_type.into_proto() as i32,
        })),
    })
}

pub(crate) fn validate_sparse_vector(name: &str, values: &SparseVector) -> Result<()> {
    for (&index, &value) in values {
        if index == u32::MAX {
            return Err(Error::validation(
                name.to_owned(),
                "sparse vector indices must be less than u32::MAX".into(),
            ));
        }
        if !value.is_finite() {
            return Err(Error::validation(
                name.to_owned(),
                "sparse vector values must be finite".into(),
            ));
        }
        if value < 0.0 {
            return Err(Error::validation(
                name.to_owned(),
                "sparse vector values must not be negative".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn encode_sparse_vector(name: &str, values: SparseVector) -> Result<Vec<u8>> {
    validate_sparse_vector(name, &values)?;
    let mut bytes = Vec::with_capacity(values.len() * 8);
    for (index, value) in values {
        bytes.extend(index.to_le_bytes());
        bytes.extend(value.to_le_bytes());
    }
    Ok(bytes)
}

fn encode_sparse_vector_by_ref(name: &str, values: &SparseVector) -> Result<Vec<u8>> {
    validate_sparse_vector(name, values)?;
    let mut bytes = Vec::with_capacity(values.len() * 8);
    for (&index, &value) in values {
        bytes.extend(index.to_le_bytes());
        bytes.extend(value.to_le_bytes());
    }
    Ok(bytes)
}

fn vector_dimension<T>(name: &str, values: &[Vec<T>]) -> Result<usize> {
    let Some(dimension) = values.first().map(Vec::len) else {
        return Ok(0);
    };
    if dimension == 0 {
        return Err(Error::validation(
            name.to_owned(),
            "vectors must not be empty".into(),
        ));
    }
    if values.iter().any(|value| value.len() != dimension) {
        return Err(Error::validation(
            name.to_owned(),
            format!("every vector must contain {dimension} elements/bytes"),
        ));
    }
    Ok(dimension)
}

fn encode_u16_vectors(values: Vec<Vec<u16>>) -> Vec<u8> {
    values
        .into_iter()
        .flatten()
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn encode_u16_vectors_by_ref(values: &[Vec<u16>]) -> Vec<u8> {
    values
        .iter()
        .flatten()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod data_type_tests {
    use super::DataType;
    use crate::proto::schema;

    #[test]
    fn non_primitive_types_convert_to_and_from_proto() {
        let cases = [
            (DataType::Geometry, schema::DataType::Geometry),
            (DataType::Timestamptz, schema::DataType::Timestamptz),
            (DataType::Struct, schema::DataType::Struct),
        ];

        for (sdk, proto) in cases {
            assert_eq!(sdk.into_proto(), proto);
            assert_eq!(DataType::try_from_proto(proto).unwrap(), sdk);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod field_data_tests {
    use super::{DataType, Error, FieldData, SparseVector};
    use crate::proto::schema::{self, field_data, scalar_field, vector_field};

    #[test]
    fn typed_constructors_and_accessors_cover_scalar_array_and_vector_values() {
        let ids = FieldData::int64("id", vec![1, 2]);
        assert_eq!(ids.name(), "id");
        assert_eq!(ids.as_int64(), Some([1, 2].as_slice()));
        assert!(ids.as_varchar().is_none());

        let tags = FieldData::array_varchar("tags", vec![vec!["rust".into(), "milvus".into()]]);
        assert_eq!(
            tags.as_array_varchar(),
            Some([vec!["rust".to_owned(), "milvus".to_owned()]].as_slice())
        );

        let vectors = FieldData::float_vector("embedding", vec![vec![0.1, 0.2], vec![0.3, 0.4]]);
        assert_eq!(
            vectors.as_float_vectors(),
            Some([vec![0.1, 0.2], vec![0.3, 0.4]].as_slice())
        );
    }

    #[test]
    fn borrowed_encoding_matches_consuming_encoding() {
        let fields = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::VarChar {
                name: "text".into(),
                values: vec!["a".into(), "b".into()],
            },
            FieldData::ArrayFloat {
                name: "weights".into(),
                values: vec![vec![0.1, 0.2], vec![0.3]],
            },
            FieldData::FloatVector {
                name: "dense".into(),
                values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
            },
            FieldData::Float16Vector {
                name: "half".into(),
                values: vec![vec![0x3c00, 0xbc00]],
            },
            FieldData::SparseFloatVector {
                name: "sparse".into(),
                values: vec![SparseVector::from([(1, 0.5), (3, 1.5)])],
            },
            FieldData::nullable(
                FieldData::Int32 {
                    name: "nullable".into(),
                    values: vec![10, 30],
                },
                vec![true, false, true],
            )
            .unwrap(),
        ];

        for field in fields {
            assert_eq!(
                field.to_proto().unwrap(),
                field.clone().into_proto().unwrap()
            );
        }
    }

    #[test]
    fn sparse_vector_columns_reject_negative_values_and_maximum_index() {
        for values in [
            SparseVector::from([(1, -0.5)]),
            SparseVector::from([(u32::MAX, 0.5)]),
        ] {
            assert!(FieldData::sparse_float_vector("sparse", vec![values])
                .into_proto()
                .is_err());
        }
    }

    #[test]
    fn with_validity_wraps_compact_values_and_checks_the_bitmap() {
        let nullable = FieldData::int64("age", vec![18, 30])
            .with_validity(vec![true, false, true])
            .unwrap();

        assert_eq!(nullable.len(), 3);
        assert_eq!(nullable.as_int64(), Some([18, 30].as_slice()));
        assert_eq!(nullable.valid_data(), Some([true, false, true].as_slice()));
        assert!(nullable.is_null(1));

        let error = FieldData::int64("age", vec![18, 30])
            .with_validity(vec![true, false])
            .unwrap_err();
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn geometry_and_timestamptz_use_their_sdk_and_wire_types() {
        let geometry = FieldData::Geometry {
            name: "location".into(),
            values: vec!["POINT (1 2)".into()],
        }
        .into_proto()
        .unwrap();
        assert_eq!(geometry.r#type, schema::DataType::Geometry as i32);
        assert!(matches!(
            geometry.field,
            Some(field_data::Field::Scalars(schema::ScalarField {
                data: Some(scalar_field::Data::GeometryWktData(_))
            }))
        ));

        let timestamptz = FieldData::Timestamptz {
            name: "observed_at".into(),
            values: vec!["2026-07-15T12:30:00+08:00".into()],
        }
        .into_proto()
        .unwrap();
        assert_eq!(timestamptz.r#type, schema::DataType::Timestamptz as i32);
        assert!(matches!(
            timestamptz.field,
            Some(field_data::Field::Scalars(schema::ScalarField {
                data: Some(scalar_field::Data::StringData(_))
            }))
        ));
    }

    #[test]
    fn narrow_integer_columns_keep_sdk_types_on_the_shared_int32_wire_array() {
        for (field, expected_type, expected_values) in [
            (
                FieldData::Int8 {
                    name: "int8_value".into(),
                    values: vec![-128, 127],
                },
                schema::DataType::Int8,
                vec![-128, 127],
            ),
            (
                FieldData::Int16 {
                    name: "int16_value".into(),
                    values: vec![-32768, 32767],
                },
                schema::DataType::Int16,
                vec![-32768, 32767],
            ),
        ] {
            let encoded = field.into_proto().unwrap();
            assert_eq!(encoded.r#type, expected_type as i32);
            let Some(field_data::Field::Scalars(schema::ScalarField {
                data: Some(scalar_field::Data::IntData(values)),
            })) = encoded.field
            else {
                panic!("expected integer scalar data")
            };
            assert_eq!(values.data, expected_values);
        }
    }

    #[test]
    fn typed_array_columns_encode_their_element_types() {
        let cases = vec![
            (
                FieldData::ArrayBool {
                    name: "values".into(),
                    values: vec![vec![true, false]],
                },
                DataType::Bool,
            ),
            (
                FieldData::ArrayInt8 {
                    name: "values".into(),
                    values: vec![vec![-128, 127]],
                },
                DataType::Int8,
            ),
            (
                FieldData::ArrayInt16 {
                    name: "values".into(),
                    values: vec![vec![-32768, 32767]],
                },
                DataType::Int16,
            ),
            (
                FieldData::ArrayInt32 {
                    name: "values".into(),
                    values: vec![vec![-1, 1]],
                },
                DataType::Int32,
            ),
            (
                FieldData::ArrayInt64 {
                    name: "values".into(),
                    values: vec![vec![-1, 1]],
                },
                DataType::Int64,
            ),
            (
                FieldData::ArrayFloat {
                    name: "values".into(),
                    values: vec![vec![0.5, 1.5]],
                },
                DataType::Float,
            ),
            (
                FieldData::ArrayDouble {
                    name: "values".into(),
                    values: vec![vec![0.5, 1.5]],
                },
                DataType::Double,
            ),
            (
                FieldData::ArrayVarChar {
                    name: "values".into(),
                    values: vec![vec!["a".into(), "b".into()]],
                },
                DataType::VarChar,
            ),
        ];

        for (field, element_type) in cases {
            assert_eq!(field.array_element_type(), Some(element_type));
            let encoded = field.into_proto().unwrap();
            assert_eq!(encoded.r#type, schema::DataType::Array as i32);
            let Some(field_data::Field::Scalars(schema::ScalarField {
                data: Some(scalar_field::Data::ArrayData(values)),
            })) = encoded.field
            else {
                panic!("expected array scalar data")
            };
            assert_eq!(values.element_type, element_type.into_proto() as i32);
            assert_eq!(values.data.len(), 1);
        }
    }

    #[test]
    fn half_precision_columns_encode_u16_values_as_little_endian_bytes() {
        let encoded = FieldData::Float16Vector {
            name: "embedding".into(),
            values: vec![vec![0x3c00, 0xbc00]],
        }
        .into_proto()
        .unwrap();

        let Some(field_data::Field::Vectors(schema::VectorField {
            dim,
            data: Some(vector_field::Data::Float16Vector(bytes)),
            ..
        })) = encoded.field
        else {
            panic!("expected float16 vector data")
        };
        assert_eq!(dim, 2);
        assert_eq!(bytes, vec![0x00, 0x3c, 0x00, 0xbc]);
    }

    #[test]
    fn vector_columns_derive_dimension_from_values() {
        let encoded = FieldData::BinaryVector {
            name: "embedding".into(),
            values: vec![vec![0xaa, 0x55], vec![0x00, 0xff]],
        }
        .into_proto()
        .unwrap();

        let Some(field_data::Field::Vectors(vectors)) = encoded.field else {
            panic!("expected binary vector data")
        };
        assert_eq!(vectors.dim, 16);
    }

    #[test]
    fn vector_columns_reject_inconsistent_value_dimensions() {
        let error = FieldData::FloatVector {
            name: "embedding".into(),
            values: vec![vec![0.1, 0.2], vec![0.3]],
        }
        .into_proto()
        .unwrap_err();

        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn all_null_vector_columns_use_the_schema_dimension() {
        let data = FieldData::nullable(
            FieldData::FloatVector {
                name: "embedding".into(),
                values: Vec::new(),
            },
            vec![false, false],
        )
        .unwrap();
        let field = schema::FieldSchema {
            data_type: schema::DataType::FloatVector as i32,
            type_params: vec![crate::proto::common::KeyValuePair {
                key: "dim".into(),
                value: "4".into(),
            }],
            ..Default::default()
        };

        let encoded = data.into_proto_with_schema(&field).unwrap();
        let Some(field_data::Field::Vectors(vectors)) = encoded.field else {
            panic!("expected float vector data")
        };
        assert_eq!(vectors.dim, 4);
        assert_eq!(encoded.valid_data, vec![false, false]);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod constructor_value_tests {
    use super::*;

    #[test]
    fn function_constructor_values() {
        let value = Function::new()
            .name("function")
            .function_type(FunctionType::Unknown);

        assert_eq!(value.get_name(), "function");
        assert!(value.get_description().is_empty());
        assert_eq!(value.get_function_type().to_owned(), FunctionType::Unknown);
        assert!(value.get_input_fields().is_empty());
        assert!(value.get_output_fields().is_empty());
        assert!(value.get_params().is_empty());
    }

    #[test]
    fn function_populated_values() {
        let value = Function::new()
            .name("bm25")
            .function_type(FunctionType::Bm25)
            .description("description")
            .input_fields(["text"])
            .output_fields(["sparse"])
            .param("key", "value");

        assert_eq!(value.get_name().to_owned(), "bm25");
        assert_eq!(value.get_description().to_owned(), "description");
        assert_eq!(value.get_function_type().to_owned(), FunctionType::Bm25);
        assert_eq!(value.get_input_fields().to_owned(), ["text"]);
        assert_eq!(value.get_output_fields().to_owned(), ["sparse"]);
        assert_eq!(
            value.get_params().get("key").map(String::as_str),
            Some("value")
        );
        assert_eq!(Function::from_proto(value.clone().into_proto()), value);

        let built = Function::new()
            .name("bm25")
            .function_type(FunctionType::Bm25)
            .description("description")
            .input_fields(["text"])
            .output_fields(["sparse"])
            .params(HashMap::from([("key".into(), "value".into())]));
        assert_eq!(built, value);
    }

    #[test]
    fn function_mutable_setters_update_existing_value() {
        let mut value = Function::new()
            .name("function")
            .function_type(FunctionType::Unknown);
        value
            .set_name("bm25")
            .set_description("description")
            .set_function_type(FunctionType::Bm25)
            .set_input_fields(["text"])
            .set_output_fields(["sparse"])
            .set_params(HashMap::from([("key".into(), "value".into())]));

        assert_eq!(value.get_name(), "bm25");
        assert_eq!(value.get_description(), "description");
        assert_eq!(value.get_function_type(), FunctionType::Bm25);
        assert_eq!(value.get_input_fields(), ["text"]);
        assert_eq!(value.get_output_fields(), ["sparse"]);
        assert_eq!(
            value.get_params().get("key").map(String::as_str),
            Some("value")
        );
    }

    #[test]
    fn retry_config_default_values() {
        let value = RetryConfig::new();

        assert_eq!(value.get_max_attempts().to_owned(), 75);
        assert_eq!(value.get_max_retry_timeout().to_owned(), Duration::ZERO);
        assert_eq!(
            value.get_initial_backoff().to_owned(),
            Duration::from_millis(10)
        );
        assert_eq!(value.get_max_backoff().to_owned(), Duration::from_secs(3));
        assert_eq!(value.get_backoff_multiplier().to_owned(), 3.0);
        assert!(value.get_retry_on_rate_limit());
    }

    #[test]
    fn retry_config_populated_values() {
        let value = RetryConfig::new()
            .max_attempts(7)
            .max_retry_timeout(Duration::from_secs(8))
            .initial_backoff(Duration::from_millis(20))
            .max_backoff(Duration::from_secs(2))
            .backoff_multiplier(2.0)
            .retry_on_rate_limit(false);

        assert_eq!(value.get_max_attempts().to_owned(), 7);
        assert_eq!(
            value.get_max_retry_timeout().to_owned(),
            Duration::from_secs(8)
        );
        assert_eq!(
            value.get_initial_backoff().to_owned(),
            Duration::from_millis(20)
        );
        assert_eq!(value.get_max_backoff().to_owned(), Duration::from_secs(2));
        assert_eq!(value.get_backoff_multiplier().to_owned(), 2.0);
        assert!(!value.get_retry_on_rate_limit());
    }

    #[test]
    fn connect_config_constructor_values() {
        let value = ConnectConfig::new().uri("http://localhost:19530");

        assert_eq!(value.get_uri().to_owned(), "http://localhost:19530");
        assert_eq!(value.get_token().to_owned(), None);
        assert_eq!(value.get_tls_server_name().to_owned(), None);
        assert_eq!(value.get_ca_certificate().to_owned(), None);
        assert_eq!(value.get_client_certificate().to_owned(), None);
        assert_eq!(value.get_client_key().to_owned(), None);
        assert_eq!(
            value.get_connect_timeout().to_owned(),
            Duration::from_secs(10)
        );
        assert_eq!(value.get_rpc_timeout().to_owned(), Duration::ZERO);
        assert_eq!(
            value.get_keepalive_time().to_owned(),
            Duration::from_secs(10)
        );
        assert_eq!(
            value.get_keepalive_timeout().to_owned(),
            Duration::from_secs(5)
        );
        assert!(value.get_keepalive_while_idle());
        assert!(value.get_database().is_empty());
        assert_eq!(value.get_retry().get_max_attempts().to_owned(), 75);
        assert_eq!(
            value.get_retry().get_initial_backoff(),
            Duration::from_millis(10)
        );
    }

    #[test]
    fn connect_config_populated_values() {
        let retry = RetryConfig::new().max_attempts(7);
        let value = ConnectConfig::new()
            .uri("http://milvus:19530")
            .token("token")
            .tls_server_name("milvus.example.com")
            .ca_certificate("/certs/ca.pem")
            .client_certificate("/certs/client.pem")
            .client_key("/certs/client-key.pem")
            .database("database")
            .connect_timeout(Duration::from_secs(1))
            .rpc_timeout(Duration::from_secs(2))
            .keepalive_time(Duration::from_secs(3))
            .keepalive_timeout(Duration::from_secs(4))
            .keepalive_while_idle(false)
            .retry(retry.clone());

        assert_eq!(value.get_uri().to_owned(), "http://milvus:19530");
        assert_eq!(value.get_token().as_deref(), Some("dG9rZW4="));
        assert_eq!(
            value.get_tls_server_name().as_deref(),
            Some("milvus.example.com")
        );
        assert_eq!(value.get_ca_certificate().as_deref(), Some("/certs/ca.pem"));
        assert_eq!(
            value.get_client_certificate().as_deref(),
            Some("/certs/client.pem")
        );
        assert_eq!(
            value.get_client_key().as_deref(),
            Some("/certs/client-key.pem")
        );
        assert_eq!(value.get_database().to_owned(), "database");
        assert_eq!(
            value.get_connect_timeout().to_owned(),
            Duration::from_secs(1)
        );
        assert_eq!(value.get_rpc_timeout().to_owned(), Duration::from_secs(2));
        assert_eq!(
            value.get_keepalive_time().to_owned(),
            Duration::from_secs(3)
        );
        assert_eq!(
            value.get_keepalive_timeout().to_owned(),
            Duration::from_secs(4)
        );
        assert!(!value.get_keepalive_while_idle());
        assert_eq!(
            value.get_retry().get_max_attempts(),
            retry.get_max_attempts()
        );

        let credentials = ConnectConfig::new()
            .uri("http://milvus:19530")
            .username_password("user", "password");
        assert_eq!(
            credentials.get_token().as_deref(),
            Some("dXNlcjpwYXNzd29yZA==")
        );
    }

    #[test]
    fn connect_config_debug_output_redacts_token() {
        let value = ConnectConfig::new()
            .uri("http://milvus:19530")
            .token("root:Milvus")
            .client_key("/secret/client-key.pem")
            .database("default");

        let debug = format!("{value:?}");
        assert!(debug.contains("http://milvus:19530"));
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("root:Milvus"));
        assert!(!debug.contains("cm9vdDpNaWx2dXM="));
        assert!(!debug.contains("/secret/client-key.pem"));
    }

    #[test]
    fn connect_config_mutable_setters_update_existing_value() {
        let retry = RetryConfig::new().max_attempts(7);
        let mut value = ConnectConfig::new().uri("http://localhost:19530");
        value
            .set_uri("http://milvus:19530")
            .set_token("root:Milvus")
            .set_tls_server_name("milvus.example.com")
            .set_ca_certificate("/certs/ca.pem")
            .set_client_certificate("/certs/client.pem")
            .set_client_key("/certs/client-key.pem")
            .set_database("database")
            .set_connect_timeout(Duration::from_secs(1))
            .set_rpc_timeout(Duration::from_secs(2))
            .set_keepalive_time(Duration::from_secs(3))
            .set_keepalive_timeout(Duration::from_secs(4))
            .set_keepalive_while_idle(false)
            .set_retry(retry.clone());

        assert_eq!(value.get_uri(), "http://milvus:19530");
        assert_eq!(value.get_token().as_deref(), Some("cm9vdDpNaWx2dXM="));
        assert_eq!(
            value.get_tls_server_name().as_deref(),
            Some("milvus.example.com")
        );
        assert_eq!(value.get_ca_certificate().as_deref(), Some("/certs/ca.pem"));
        assert_eq!(
            value.get_client_certificate().as_deref(),
            Some("/certs/client.pem")
        );
        assert_eq!(
            value.get_client_key().as_deref(),
            Some("/certs/client-key.pem")
        );
        assert_eq!(value.get_database(), "database");
        assert_eq!(value.get_connect_timeout(), Duration::from_secs(1));
        assert_eq!(value.get_rpc_timeout(), Duration::from_secs(2));
        assert_eq!(value.get_keepalive_time(), Duration::from_secs(3));
        assert_eq!(value.get_keepalive_timeout(), Duration::from_secs(4));
        assert!(!value.get_keepalive_while_idle());
        assert_eq!(
            value.get_retry().get_max_attempts(),
            retry.get_max_attempts()
        );

        value
            .set_tls_server_name("")
            .set_ca_certificate("")
            .set_client_certificate("")
            .set_client_key("");
        assert_eq!(value.get_tls_server_name(), &None);
        assert_eq!(value.get_ca_certificate(), &None);
        assert_eq!(value.get_client_certificate(), &None);
        assert_eq!(value.get_client_key(), &None);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod enum_conversion_tests {
    use super::*;

    #[test]
    fn consistency_level_round_trips_proto() {
        let values = [
            ConsistencyLevel::Strong,
            ConsistencyLevel::Session,
            ConsistencyLevel::Bounded,
            ConsistencyLevel::Eventually,
            ConsistencyLevel::Customized,
        ];

        for value in values {
            assert_eq!(
                ConsistencyLevel::from_proto(value.into_proto() as i32),
                value
            );
        }
        assert_eq!(
            ConsistencyLevel::from_proto(i32::MAX),
            ConsistencyLevel::Customized
        );
    }

    #[test]
    fn metric_type_round_trips_wire_name() {
        let values = [
            MetricType::Default,
            MetricType::L2,
            MetricType::Ip,
            MetricType::Cosine,
            MetricType::Hamming,
            MetricType::Jaccard,
            MetricType::MhJaccard,
            MetricType::Bm25,
            MetricType::MaxSimCosine,
            MetricType::MaxSimIp,
            MetricType::MaxSimL2,
            MetricType::MaxSimJaccard,
            MetricType::MaxSimHamming,
        ];

        for value in values {
            assert_eq!(MetricType::from_str(value.as_str()), value);
        }
        assert_eq!(MetricType::from_str("INVALID"), MetricType::Default);
        assert_eq!(MetricType::from_str("unknown"), MetricType::Default);
    }

    #[test]
    fn data_type_round_trips_proto() {
        let values = [
            DataType::Unknown,
            DataType::Bool,
            DataType::Int8,
            DataType::Int16,
            DataType::Int32,
            DataType::Int64,
            DataType::Float,
            DataType::Double,
            DataType::VarChar,
            DataType::Json,
            DataType::Geometry,
            DataType::Timestamptz,
            DataType::Array,
            DataType::Struct,
            DataType::FloatVector,
            DataType::BinaryVector,
            DataType::Float16Vector,
            DataType::BFloat16Vector,
            DataType::SparseFloatVector,
            DataType::Int8Vector,
        ];

        for value in values {
            assert_eq!(DataType::try_from_proto(value.into_proto()).unwrap(), value);
        }
    }

    #[test]
    fn function_type_round_trips_through_function_proto() {
        let values = [
            FunctionType::Unknown,
            FunctionType::Bm25,
            FunctionType::TextEmbedding,
            FunctionType::Rerank,
        ];

        for value in values {
            let function = Function::new().name("function").function_type(value);
            assert_eq!(
                Function::from_proto(function.into_proto()).get_function_type(),
                value
            );
        }
    }

    #[test]
    fn ids_convert_from_proto_and_into_json() {
        let integer = Ids::Int64(vec![1, 2]);
        assert_eq!(integer.clone().into_json(), serde_json::json!([1, 2]));
        assert_eq!(
            Ids::from_proto(Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                    data: vec![1, 2],
                    ..Default::default()
                })),
            })),
            integer
        );

        let strings = Ids::VarChar(vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(strings.clone().into_json(), serde_json::json!(["a", "b"]));
        assert_eq!(
            Ids::from_proto(Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::StrId(schema::StringArray {
                    data: vec!["a".to_owned(), "b".to_owned()],
                    ..Default::default()
                })),
            })),
            strings
        );
    }

    #[test]
    fn field_data_converts_to_proto() {
        let value = FieldData::VarChar {
            name: "text".to_owned(),
            values: vec!["value".to_owned()],
        };
        let proto = value.into_proto().expect("field data conversion");

        assert_eq!(proto.field_name, "text");
        assert_eq!(proto.r#type, schema::DataType::VarChar as i32);
    }
}
