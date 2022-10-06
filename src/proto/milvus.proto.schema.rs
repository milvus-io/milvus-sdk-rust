///*
/// @brief Field schema
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldSchema {
    #[prost(int64, tag = "1")]
    pub field_id: i64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub is_primary_key: bool,
    #[prost(string, tag = "4")]
    pub description: ::prost::alloc::string::String,
    #[prost(enumeration = "DataType", tag = "5")]
    pub data_type: i32,
    #[prost(message, repeated, tag = "6")]
    pub type_params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
    #[prost(message, repeated, tag = "7")]
    pub index_params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
    #[prost(bool, tag = "8")]
    pub auto_id: bool,
    /// To keep compatible with older version, the default state is `Created`.
    #[prost(enumeration = "FieldState", tag = "9")]
    pub state: i32,
}
///*
/// @brief Collection schema
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CollectionSchema {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// deprecated later, keep compatible with c++ part now
    #[prost(bool, tag = "3")]
    pub auto_id: bool,
    #[prost(message, repeated, tag = "4")]
    pub fields: ::prost::alloc::vec::Vec<FieldSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BoolArray {
    #[prost(bool, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IntArray {
    #[prost(int32, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LongArray {
    #[prost(int64, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FloatArray {
    #[prost(float, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DoubleArray {
    #[prost(double, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<f64>,
}
/// For special fields such as bigdecimal, array...
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BytesArray {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringArray {
    #[prost(string, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScalarField {
    #[prost(oneof = "scalar_field::Data", tags = "1, 2, 3, 4, 5, 6, 7")]
    pub data: ::core::option::Option<scalar_field::Data>,
}
/// Nested message and enum types in `ScalarField`.
pub mod scalar_field {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "1")]
        BoolData(super::BoolArray),
        #[prost(message, tag = "2")]
        IntData(super::IntArray),
        #[prost(message, tag = "3")]
        LongData(super::LongArray),
        #[prost(message, tag = "4")]
        FloatData(super::FloatArray),
        #[prost(message, tag = "5")]
        DoubleData(super::DoubleArray),
        #[prost(message, tag = "6")]
        StringData(super::StringArray),
        #[prost(message, tag = "7")]
        BytesData(super::BytesArray),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VectorField {
    #[prost(int64, tag = "1")]
    pub dim: i64,
    #[prost(oneof = "vector_field::Data", tags = "2, 3")]
    pub data: ::core::option::Option<vector_field::Data>,
}
/// Nested message and enum types in `VectorField`.
pub mod vector_field {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "2")]
        FloatVector(super::FloatArray),
        #[prost(bytes, tag = "3")]
        BinaryVector(::prost::alloc::vec::Vec<u8>),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldData {
    #[prost(enumeration = "DataType", tag = "1")]
    pub r#type: i32,
    #[prost(string, tag = "2")]
    pub field_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub field_id: i64,
    #[prost(oneof = "field_data::Field", tags = "3, 4")]
    pub field: ::core::option::Option<field_data::Field>,
}
/// Nested message and enum types in `FieldData`.
pub mod field_data {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Field {
        #[prost(message, tag = "3")]
        Scalars(super::ScalarField),
        #[prost(message, tag = "4")]
        Vectors(super::VectorField),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IDs {
    #[prost(oneof = "i_ds::IdField", tags = "1, 2")]
    pub id_field: ::core::option::Option<i_ds::IdField>,
}
/// Nested message and enum types in `IDs`.
pub mod i_ds {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum IdField {
        #[prost(message, tag = "1")]
        IntId(super::LongArray),
        #[prost(message, tag = "2")]
        StrId(super::StringArray),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchResultData {
    #[prost(int64, tag = "1")]
    pub num_queries: i64,
    #[prost(int64, tag = "2")]
    pub top_k: i64,
    #[prost(message, repeated, tag = "3")]
    pub fields_data: ::prost::alloc::vec::Vec<FieldData>,
    #[prost(float, repeated, tag = "4")]
    pub scores: ::prost::alloc::vec::Vec<f32>,
    #[prost(message, optional, tag = "5")]
    pub ids: ::core::option::Option<IDs>,
    #[prost(int64, repeated, tag = "6")]
    pub topks: ::prost::alloc::vec::Vec<i64>,
}
///*
/// @brief Field data type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataType {
    None = 0,
    Bool = 1,
    Int8 = 2,
    Int16 = 3,
    Int32 = 4,
    Int64 = 5,
    Float = 10,
    Double = 11,
    String = 20,
    /// variable-length strings with a specified maximum length
    VarChar = 21,
    BinaryVector = 100,
    FloatVector = 101,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FieldState {
    FieldCreated = 0,
    FieldCreating = 1,
    FieldDropping = 2,
    FieldDropped = 3,
}
