// use std::collections::HashMap;

// use crate::{
//     proto::schema::{
//         self, field_data::Field, scalar_field::Data as ScalarData,
//         vector_field::Data as VectorData, DataType, ScalarField, VectorField,
//     },
//     schema::{CollectionSchema, Error},
// };

// pub trait AsFieldDataValue {
//     fn as_field_data_value(&self) -> FieldDataValue<'_>;
// }

// impl AsFieldDataValue for i64 {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::Int64(*self)
//     }
// }

// impl AsFieldDataValue for i32 {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::Int32(*self)
//     }
// }

// impl AsFieldDataValue for i16 {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::Int16(*self)
//     }
// }

// impl AsFieldDataValue for i8 {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::Int8(*self)
//     }
// }

// impl AsFieldDataValue for [u8] {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::BinaryVector(self)
//     }
// }

// impl AsFieldDataValue for Vec<u8> {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::BinaryVector(self)
//     }
// }

// impl AsFieldDataValue for [f32] {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::FloatVector(self)
//     }
// }

// impl AsFieldDataValue for Vec<f32> {
//     fn as_field_data_value(&self) -> FieldDataValue<'_> {
//         FieldDataValue::FloatVector(self)
//     }
// }

// #[derive(Debug, Clone)]
// pub enum FieldDataValue<'a> {
//     None,
//     Bool(bool),
//     Int8(i8),
//     Int16(i16),
//     Int32(i32),
//     Int64(i64),
//     Float(f32),
//     Double(f64),
//     String(&'a str),
//     BinaryVector(&'a [u8]),
//     FloatVector(&'a [f32]),
// }

// impl<'a> FieldDataValue<'a> {
//     pub fn get_dim(&self) -> usize {
//         match self {
//             FieldDataValue::None => 0,
//             FieldDataValue::Bool(_)
//             | FieldDataValue::Int8(_)
//             | FieldDataValue::Int16(_)
//             | FieldDataValue::Int32(_)
//             | FieldDataValue::Int64(_)
//             | FieldDataValue::Float(_)
//             | FieldDataValue::Double(_)
//             | FieldDataValue::String(_) => 1,
//             FieldDataValue::BinaryVector(s) => s.len() * 8,
//             FieldDataValue::FloatVector(s) => s.len(),
//         }
//     }

//     #[inline]
//     pub fn data_type(&self) -> DataType {
//         match self {
//             FieldDataValue::None => DataType::None,
//             FieldDataValue::Bool(_) => DataType::Bool,
//             FieldDataValue::Int8(_) => DataType::Int8,
//             FieldDataValue::Int16(_) => DataType::Int16,
//             FieldDataValue::Int32(_) => DataType::Int32,
//             FieldDataValue::Int64(_) => DataType::Int64,
//             FieldDataValue::Float(_) => DataType::Float,
//             FieldDataValue::Double(_) => DataType::Double,
//             FieldDataValue::String(_) => DataType::String,
//             // FieldDataValue::VarChar(_) => DataType::VarChar,
//             FieldDataValue::BinaryVector(_) => DataType::BinaryVector,
//             FieldDataValue::FloatVector(_) => DataType::FloatVector,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub enum FieldDataColumn {
//     None,
//     Bool(Vec<bool>),
//     Int8(Vec<i32>),
//     Int16(Vec<i32>),
//     Int32(Vec<i32>),
//     Int64(Vec<i64>),
//     Float(Vec<f32>),
//     Double(Vec<f64>),
//     String(Vec<String>),
//     VarChar(Vec<String>),

//     BinaryVector(i64, Vec<u8>),
//     FloatVector(i64, Vec<f32>),
// }

// impl FieldDataColumn {
//     #[inline]
//     pub fn data_type(&self) -> DataType {
//         match self {
//             FieldDataColumn::None => DataType::None,
//             FieldDataColumn::Bool(_) => DataType::Bool,
//             FieldDataColumn::Int8(_) => DataType::Int8,
//             FieldDataColumn::Int16(_) => DataType::Int16,
//             FieldDataColumn::Int32(_) => DataType::Int32,
//             FieldDataColumn::Int64(_) => DataType::Int64,
//             FieldDataColumn::Float(_) => DataType::Float,
//             FieldDataColumn::Double(_) => DataType::Double,
//             FieldDataColumn::String(_) => DataType::String,
//             FieldDataColumn::VarChar(_) => DataType::VarChar,
//             FieldDataColumn::BinaryVector(..) => DataType::BinaryVector,
//             FieldDataColumn::FloatVector(..) => DataType::FloatVector,
//         }
//     }

//     fn push(&mut self, v: FieldDataValue) -> std::result::Result<(), Error> {
//         match (v, self) {
//             (FieldDataValue::Bool(v), FieldDataColumn::Bool(vec)) => vec.push(v),
//             (FieldDataValue::Int8(v), FieldDataColumn::Int8(vec)) => vec.push(v as _),
//             (FieldDataValue::Int16(v), FieldDataColumn::Int16(vec)) => vec.push(v as _),
//             (FieldDataValue::Int32(v), FieldDataColumn::Int32(vec)) => vec.push(v),
//             (FieldDataValue::Int64(v), FieldDataColumn::Int64(vec)) => vec.push(v),
//             (FieldDataValue::Float(v), FieldDataColumn::Float(vec)) => vec.push(v),
//             (FieldDataValue::Double(v), FieldDataColumn::Double(vec)) => vec.push(v),
//             (FieldDataValue::String(v), FieldDataColumn::String(vec)) => vec.push(v.to_string()),
//             (FieldDataValue::BinaryVector(v), FieldDataColumn::BinaryVector(_, vec)) => {
//                 vec.extend_from_slice(v)
//             }
//             (FieldDataValue::FloatVector(v), FieldDataColumn::FloatVector(_, vec)) => {
//                 vec.extend_from_slice(v)
//             }

//             (a, b) => {
//                 return Err(Error::FieldWrongType(
//                     String::new(),
//                     a.data_type(),
//                     b.data_type(),
//                 ))
//             }
//         }

//         Ok(())
//     }

//     #[inline]
//     fn dim(&self) -> i64 {
//         match self {
//             FieldDataColumn::None => 0,
//             FieldDataColumn::Bool(_)
//             | FieldDataColumn::Int8(_)
//             | FieldDataColumn::Int16(_)
//             | FieldDataColumn::Int32(_)
//             | FieldDataColumn::Int64(_)
//             | FieldDataColumn::Float(_)
//             | FieldDataColumn::Double(_)
//             | FieldDataColumn::String(_)
//             | FieldDataColumn::VarChar(_) => 1,
//             FieldDataColumn::BinaryVector(d, _) => *d,
//             FieldDataColumn::FloatVector(d, _) => *d,
//         }
//     }

//     #[inline]
//     pub fn clear(&mut self) {
//         match self {
//             FieldDataColumn::None => (),
//             FieldDataColumn::Bool(v) => v.clear(),
//             FieldDataColumn::Int8(v) => v.clear(),
//             FieldDataColumn::Int16(v) => v.clear(),
//             FieldDataColumn::Int32(v) => v.clear(),
//             FieldDataColumn::Int64(v) => v.clear(),
//             FieldDataColumn::Float(v) => v.clear(),
//             FieldDataColumn::Double(v) => v.clear(),
//             FieldDataColumn::String(v) => v.clear(),
//             FieldDataColumn::VarChar(v) => v.clear(),
//             FieldDataColumn::BinaryVector(_, v) => v.clear(),
//             FieldDataColumn::FloatVector(_, v) => v.clear(),
//         }
//     }
// }

// impl From<Vec<i8>> for FieldDataColumn {
//     fn from(v: Vec<i8>) -> Self {
//         Self::Int8(v.into_iter().map(|x| x as _).collect())
//     }
// }

// impl From<Vec<i16>> for FieldDataColumn {
//     fn from(v: Vec<i16>) -> Self {
//         Self::Int16(v.into_iter().map(|x| x as _).collect())
//     }
// }

// macro_rules! impl_from_for_field_data_column {
//     ( $($t: ty, $o: ident ),+ ) => {$(
//         impl From<Vec<$t>> for FieldDataColumn {
//             fn from(v: Vec<$t>) -> Self {
//                 Self::$o(v)
//             }
//         }
//     )*};
// }

// impl_from_for_field_data_column! {
//     bool, Bool,
//     i32, Int32,
//     i64, Int64,
//     f32, Float,
//     f64, Double,
//     String, String
// }

// impl From<FieldDataColumn> for Field {
//     fn from(this: FieldDataColumn) -> Self {
//         match this {
//             FieldDataColumn::None => Field::Scalars(ScalarField { data: None }),
//             FieldDataColumn::Bool(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::BoolData(schema::BoolArray { data: v })),
//             }),
//             FieldDataColumn::Int8(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::IntData(schema::IntArray { data: v })),
//             }),
//             FieldDataColumn::Int16(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::IntData(schema::IntArray { data: v })),
//             }),
//             FieldDataColumn::Int32(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::IntData(schema::IntArray { data: v })),
//             }),
//             FieldDataColumn::Int64(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::LongData(schema::LongArray { data: v })),
//             }),
//             FieldDataColumn::Float(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::FloatData(schema::FloatArray { data: v })),
//             }),
//             FieldDataColumn::Double(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::DoubleData(schema::DoubleArray { data: v })),
//             }),
//             FieldDataColumn::String(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::StringData(schema::StringArray { data: v })),
//             }),
//             FieldDataColumn::VarChar(v) => Field::Scalars(ScalarField {
//                 data: Some(ScalarData::StringData(schema::StringArray { data: v })),
//             }),
//             FieldDataColumn::BinaryVector(dim, v) => Field::Vectors(VectorField {
//                 data: Some(VectorData::BinaryVector(v)),
//                 dim,
//             }),
//             FieldDataColumn::FloatVector(dim, v) => Field::Vectors(VectorField {
//                 data: Some(VectorData::FloatVector(schema::FloatArray { data: v })),
//                 dim,
//             }),
//         }
//     }
// }

// impl From<Field> for FieldDataColumn {
//     fn from(f: Field) -> Self {
//         match f {
//             Field::Scalars(s) => match s.data {
//                 Some(x) => match x {
//                     ScalarData::BoolData(v) => Self::Bool(v.data),
//                     ScalarData::IntData(v) => Self::Int32(v.data),
//                     ScalarData::LongData(v) => Self::Int64(v.data),
//                     ScalarData::FloatData(v) => Self::Float(v.data),
//                     ScalarData::DoubleData(v) => Self::Double(v.data),
//                     ScalarData::StringData(v) => Self::String(v.data),
//                     ScalarData::BytesData(v) => unimplemented!(), // Self::Bytes(v.data),
//                 },
//                 None => Self::None,
//             },

//             Field::Vectors(arr) => match arr.data {
//                 Some(x) => match x {
//                     VectorData::FloatVector(v) => Self::FloatVector(arr.dim, v.data),
//                     VectorData::BinaryVector(v) => Self::BinaryVector(arr.dim, v),
//                 },
//                 None => Self::None,
//             },
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct FieldData {
//     name: String,
//     dim: usize,
//     field: FieldDataColumn,
// }

// impl FieldData {
//     fn new(name: String, dim: usize, field: FieldDataColumn) -> Self {
//         Self { name, dim, field }
//     }

//     #[inline]
//     pub fn clear(&mut self) {
//         self.field.clear();
//     }
// }

// impl From<FieldData> for schema::FieldData {
//     fn from(this: FieldData) -> schema::FieldData {
//         schema::FieldData {
//             field_name: this.name.to_string(),
//             field_id: 0,
//             r#type: this.field.data_type() as _,
//             field: Some(this.field.into()),
//         }
//     }
// }

// impl From<schema::FieldData> for FieldData {
//     fn from(fd: schema::FieldData) -> Self {
//         let field: FieldDataColumn = fd.field.map(Into::into).unwrap_or(FieldDataColumn::None);

//         FieldData {
//             name: fd.field_name.into(),
//             dim: field.dim() as _,
//             field,
//         }
//     }
// }
