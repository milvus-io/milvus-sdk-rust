use std::borrow::Cow;

use crate::{
    proto::schema::{
        self, field_data::Field, scalar_field::Data as ScalarData,
        vector_field::Data as VectorData, DataType, ScalarField, VectorField,
    },
    schema::FieldSchema,
    value::{Value, ValueVec},
};

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

#[derive(Debug, Clone)]
pub struct FieldColumn {
    pub name: String,
    pub dtype: DataType,
    pub value: ValueVec,
    pub dim: i64,
    pub max_length: i32,
}

impl From<schema::FieldData> for FieldColumn {
    fn from(fd: schema::FieldData) -> Self {
        let (dim, max_length) = fd
            .field
            .as_ref()
            .map(get_dim_max_length)
            .unwrap_or((None, None));

        let value: ValueVec = fd.field.map(Into::into).unwrap_or(ValueVec::None);
        let dtype = DataType::from_i32(fd.r#type).unwrap_or(DataType::None);

        FieldColumn {
            name: fd.field_name,
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

impl FieldColumn {
    pub fn new<V: Into<ValueVec>>(schm: &FieldSchema, v: V) -> FieldColumn {
        FieldColumn {
            name: schm.name.clone(),
            dtype: schm.dtype,
            value: v.into(),
            dim: schm.dim,
            max_length: schm.max_length,
        }
    }

    pub fn get(&self, idx: usize) -> Option<Value<'_>> {
        Some(match &self.value {
            ValueVec::None => Value::None,
            ValueVec::Bool(v) => Value::Bool(*v.get(idx)?),
            ValueVec::Int(v) => match self.dtype {
                DataType::Int8 => Value::Int8(*v.get(idx)? as _),
                DataType::Int16 => Value::Int16(*v.get(idx)? as _),
                DataType::Int32 => Value::Int32(*v.get(idx)?),
                _ => unreachable!(),
            },
            ValueVec::Long(v) => Value::Long(*v.get(idx)?),
            ValueVec::Float(v) => match self.dtype {
                DataType::Float => Value::Float(*v.get(idx)?),
                DataType::FloatVector => {
                    let dim = self.dim as usize;
                    Value::FloatArray(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
                }
                _ => unreachable!(),
            },
            ValueVec::Double(v) => Value::Double(*v.get(idx)?),
            ValueVec::Binary(v) => {
                let dim = (self.dim / 8) as usize;
                Value::Binary(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
            }
            ValueVec::String(v) => Value::String(Cow::Borrowed(v.get(idx)?.as_ref())),
        })
    }

    pub fn push(&mut self, val: Value) {
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
    pub fn len(&self) -> usize {
        self.value.len() / self.dim as usize
    }

    pub fn copy_with_metadata(&self) -> Self {
        Self {
            dim: self.dim,
            dtype: self.dtype,
            max_length: self.max_length,
            name: self.name.clone(),
            value: match &self.value {
                ValueVec::None => ValueVec::None,
                ValueVec::Bool(_) => ValueVec::Bool(Vec::new()),
                ValueVec::Int(_) => ValueVec::Int(Vec::new()),
                ValueVec::Long(_) => ValueVec::Long(Vec::new()),
                ValueVec::Float(_) => ValueVec::Float(Vec::new()),
                ValueVec::Double(_) => ValueVec::Double(Vec::new()),
                ValueVec::String(_) => ValueVec::String(Vec::new()),
                ValueVec::Binary(_) => ValueVec::Binary(Vec::new()),
            },
        }
    }
}

impl From<FieldColumn> for schema::FieldData {
    fn from(this: FieldColumn) -> schema::FieldData {
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

fn get_dim_max_length(field: &Field) -> (Option<i64>, Option<i32>) {
    let dim = match field {
        Field::Scalars(ScalarField { data: Some(_) }) => 1i64,
        Field::Vectors(VectorField { dim, .. }) => *dim,
        _ => 0i64,
    };

    (Some(dim), None) // no idea how to get max_length
}
