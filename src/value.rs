use std::borrow::Cow;

use crate::proto::{
    self,
    schema::{
        field_data::Field, scalar_field::Data as ScalarData, vector_field::Data as VectorData,
        DataType,
    },
};

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
    Json(Cow<'a, [u8]>),
    Array(Cow<'a, proto::schema::ScalarField>),
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
    pub fn data_type(&self) -> DataType {
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
            Value::Json(_) => DataType::Json,
            Value::FloatArray(_) => DataType::FloatVector,
            Value::Binary(_) => DataType::BinaryVector,
            Value::Array(_) => DataType::Array,
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
    Json(Vec<Vec<u8>>),
    Array(Vec<proto::schema::ScalarField>),
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

macro_rules! impl_try_from_for_value_vec {
    ( $($o: ident, $t: ty ),+ ) => {$(
        impl TryFrom<ValueVec> for $t {
            type Error = crate::error::Error;
            fn try_from(value: ValueVec) -> Result<Self, Self::Error> {
                match value {
                    ValueVec::$o(v) => Ok(v),
                    _ => Err(crate::error::Error::Conversion),
                }
            }
        }
    )*};
}

impl_try_from_for_value_vec! {
    Bool, Vec<bool>,
    Int, Vec<i32>,
    Long, Vec<i64>,
    String, Vec<String>,
    Binary, Vec<u8>,
    Float, Vec<f32>,
    Double, Vec<f64>
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
            DataType::Json => Self::String(Vec::new()),
            DataType::Array => Self::Array(Vec::new()),
            DataType::BinaryVector => Self::Binary(Vec::new()),
            DataType::FloatVector => Self::Float(Vec::new()),
            DataType::Float16Vector => Self::Binary(Vec::new()),
            DataType::BFloat16Vector => Self::Binary(Vec::new()),
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
            ValueVec::Json(v) => v.len(),
            ValueVec::Array(v) => v.len(),
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
            ValueVec::Json(v) => v.clear(),
            ValueVec::Array(v) => v.clear(),
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
                    ScalarData::JsonData(v) => Self::Json(v.data),
                    ScalarData::ArrayData(v) => Self::Array(v.data),
                    ScalarData::BytesData(_) => unimplemented!(), // Self::Bytes(v.data),
                },
                None => Self::None,
            },

            Field::Vectors(arr) => match arr.data {
                Some(x) => match x {
                    VectorData::FloatVector(v) => Self::Float(v.data),
                    VectorData::BinaryVector(v) => Self::Binary(v),
                    VectorData::Bfloat16Vector(v) => Self::Binary(v),
                    VectorData::Float16Vector(v) => Self::Binary(v),
                },
                None => Self::None,
            },
        }
    }
}

macro_rules! impl_try_from_for_value_column {
    ( $($o: ident,$t: ty ),+ ) => {$(
        impl TryFrom<Value<'_>> for $t {
            type Error = crate::error::Error;
            fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
                match value {
                    Value::$o(v) => Ok(v),
                    _ => Err(crate::error::Error::Conversion),
                }
            }
        }
    )*};
}

impl_try_from_for_value_column! {
    Bool,bool,
    Int8,i8,
    Int16,i16,
    Int32,i32,
    Long,i64,
    Float,f32,
    Double,f64
}

#[cfg(test)]
mod test {
    use crate::{
        error::Error,
        value::{Value, ValueVec},
    };

    #[test]
    fn test_try_from_for_value_column() {
        // bool
        let b = Value::Bool(false);
        let b: Result<bool, Error> = b.try_into();
        assert!(b.is_ok());
        assert!(!b.unwrap());
        //i8
        let int8 = Value::Int8(12);
        let r: Result<i8, Error> = int8.try_into();
        assert!(r.is_ok());
        assert_eq!(12, r.unwrap());
        //i16
        let int16 = Value::Int16(1225);
        let r: Result<i16, Error> = int16.try_into();
        assert!(r.is_ok());
        assert_eq!(1225, r.unwrap());
        //i32
        let int32 = Value::Int32(37360798);
        let r: Result<i32, Error> = int32.try_into();
        assert!(r.is_ok());
        assert_eq!(37360798, r.unwrap());
        //i64
        let long = Value::Long(123);
        let r: Result<i64, Error> = long.try_into();
        assert!(r.is_ok());
        assert_eq!(123, r.unwrap());

        let float = Value::Float(22104f32);
        let r: Result<f32, Error> = float.try_into();
        assert!(r.is_ok());
        assert_eq!(22104f32, r.unwrap());

        let double = Value::Double(22104f64);
        let r: Result<f64, Error> = double.try_into();
        assert!(r.is_ok());
        assert_eq!(22104f64, r.unwrap());
    }

    #[test]
    fn test_try_from_for_value_vec() {
        let b = ValueVec::Bool(vec![false, false]);
        let b: Result<Vec<bool>, Error> = b.try_into();
        assert!(b.is_ok());
        assert_eq!(vec![false, false], b.unwrap());

        let b = ValueVec::Int(vec![1, 2]);
        let b: Result<Vec<i32>, Error> = b.try_into();
        assert!(b.is_ok());
        assert_eq!(vec![1, 2], b.unwrap());

        let v: Vec<i64> = vec![4095291003, 2581116377, 3892395808];
        let b = ValueVec::Long(v.clone());
        let b: Result<Vec<i64>, Error> = b.try_into();
        assert!(b.is_ok());
        assert_eq!(v, b.unwrap());

        let v: Vec<f32> = vec![11., 7., 754., 68., 34.];
        let b = ValueVec::Float(v.clone());
        let b: Result<Vec<f32>, Error> = b.try_into();
        assert!(b.is_ok());
        assert_eq!(v, b.unwrap());

        let v: Vec<f64> = vec![28., 9., 92., 6099786761., 64.];
        let b = ValueVec::Double(v.clone());
        let b_err: Result<Vec<f32>, Error> = b.clone().try_into();
        assert!(b_err.is_err());
        let b: Result<Vec<f64>, Error> = b.try_into();
        assert_eq!(v, b.unwrap());

        let v = vec![28, 5, 70, 62, 57, 103, 68];
        let b = ValueVec::Binary(v.clone());
        let b: Result<Vec<u8>, Error> = b.try_into();
        assert!(b.is_ok());
        assert_eq!(v, b.unwrap());

        let v: Vec<String> = vec!["Janoato", "Samoa", "opde@tuwuv.yt"]
            .into_iter()
            .map(|a| a.into())
            .collect();
        let b = ValueVec::String(v.clone());
        let b: Result<Vec<String>, Error> = b.try_into();
        assert!(b.is_ok());
        assert_eq!(v, b.unwrap());
    }
}
