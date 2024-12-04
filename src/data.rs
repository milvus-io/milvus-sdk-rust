use crate::error::Result;
use crate::{
    proto::schema::{
        self, field_data::Field, scalar_field::Data as ScalarData,
        vector_field::Data as VectorData, DataType, ScalarField, VectorField,
    },
    schema::FieldSchema,
    value::{TryIntoValueVecWithSchema, Value, ValueVec},
};
use half::prelude::*;
use std::borrow::Cow;

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
    Vec<f16>, DataType::Float16Vector,
    Vec<bf16>, DataType::BFloat16Vector,
    Vec<u8>, DataType::BinaryVector,
    Cow<'_, [f32]>, DataType::FloatVector,
    Cow<'_, [f16]>, DataType::Float16Vector,
    Cow<'_, [bf16]>, DataType::BFloat16Vector,
    Cow<'_, [u8]>, DataType::BinaryVector
}

/// FieldColumn represents a column of data.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldColumn {
    pub name: String,
    pub dtype: DataType,
    pub value: ValueVec,
    pub dim: i64,
    pub max_length: i32,
    pub is_dynamic: bool,
}

impl From<schema::FieldData> for FieldColumn {
    fn from(fd: schema::FieldData) -> Self {
        let (dim, max_length) = fd
            .field
            .as_ref()
            .map(get_dim_max_length)
            .unwrap_or((Some(1), None));

        let value: ValueVec = fd.field.map(Into::into).unwrap_or(ValueVec::None);
        let dtype = DataType::from_i32(fd.r#type).unwrap_or(DataType::None);

        FieldColumn {
            name: fd.field_name,
            dtype,
            dim: dim.unwrap(),
            max_length: max_length.unwrap_or(0),
            value,
            is_dynamic: fd.is_dynamic,
        }
    }
}

impl FieldColumn {
    /// Create a new FieldColumn from a FieldSchema and a value vector.
    /// Returns an error if the value vector does not match the schema.
    ///
    /// # Example
    /// ```
    /// use milvus::data::FieldColumn;
    /// use milvus::schema::FieldSchema;
    /// use milvus::proto::schema::DataType;
    ///
    /// let schema = FieldSchema::new_int32("int32_schema", "");
    /// let column = FieldColumn::new(&schema, vec![1, 2, 3]).unwrap();
    /// assert_eq!(column.dtype, DataType::Int32);
    /// assert_eq!(column.len(), 3);
    ///
    /// let schema = FieldSchema::new_float16_vector("float16_vector_schema", "", 8);
    /// let column = FieldColumn::new(&schema, vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7,
    /// 0.8]).unwrap();
    /// assert_eq!(column.dtype, DataType::Float16Vector);
    /// ```
    pub fn new<V: TryIntoValueVecWithSchema>(schm: &FieldSchema, v: V) -> Result<FieldColumn> {
        let value: ValueVec = v.try_into_value_vec(schm)?;
        Ok(FieldColumn {
            name: schm.name.clone(),
            dtype: schm.dtype,
            value,
            dim: schm.dim,
            max_length: schm.max_length,
            is_dynamic: false,
        })
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
            ValueVec::Float16(v) => {
                let dim = self.dim as usize;
                Value::Float16Array(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
            }
            ValueVec::BFloat16(v) => {
                let dim = self.dim as usize;
                Value::BFloat16Array(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
            }
            ValueVec::String(v) => Value::String(Cow::Borrowed(v.get(idx)?.as_ref())),
            ValueVec::Json(v) => Value::Json(Cow::Borrowed(v.get(idx)?.as_ref())),
            ValueVec::Array(v) => Value::Array(Cow::Borrowed(v.get(idx)?)),
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
            (ValueVec::Float16(vec), Value::Float16Array(i)) => vec.extend_from_slice(i.as_ref()),
            (ValueVec::Float16(vec), Value::FloatArray(i)) => {
                vec.extend(Vec::<f16>::from_f32_slice(i.as_ref()))
            }
            (ValueVec::BFloat16(vec), Value::BFloat16Array(i)) => vec.extend_from_slice(i.as_ref()),
            (ValueVec::BFloat16(vec), Value::FloatArray(i)) => {
                vec.extend(Vec::<bf16>::from_f32_slice(i.as_ref()))
            }
            _ => panic!("column type mismatch"),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.value.len() / self.dim as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
                ValueVec::Json(_) => ValueVec::Json(Vec::new()),
                ValueVec::Binary(_) => ValueVec::Binary(Vec::new()),
                ValueVec::Float16(_) => ValueVec::Float16(Vec::new()),
                ValueVec::BFloat16(_) => ValueVec::BFloat16(Vec::new()),
                ValueVec::Array(_) => ValueVec::Array(Vec::new()),
            },
            is_dynamic: self.is_dynamic,
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
                    // both scalar and vector fields accept 1-d float array
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
                ValueVec::Json(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::JsonData(schema::JsonArray { data: v })),
                }),
                ValueVec::Array(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::ArrayData(schema::ArrayArray {
                        data: v,
                        element_type: this.dtype as _,
                    })),
                }),
                ValueVec::Binary(v) => Field::Vectors(VectorField {
                    data: Some(VectorData::BinaryVector(v)),
                    dim: this.dim,
                }),
                // milvus-proto assumes that float16 and bfloat16 are stored as little-endian bytes
                ValueVec::BFloat16(v) => {
                    let v: Vec<u8> = v.into_iter().flat_map(|x| x.to_le_bytes()).collect();
                    Field::Vectors(VectorField {
                        data: Some(VectorData::Bfloat16Vector(v)),
                        dim: this.dim,
                    })
                }
                ValueVec::Float16(v) => {
                    let v: Vec<u8> = v.into_iter().flat_map(|x| x.to_le_bytes()).collect();
                    Field::Vectors(VectorField {
                        data: Some(VectorData::Float16Vector(v)),
                        dim: this.dim,
                    })
                }
            }),
            is_dynamic: false,
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
    Vec<u8>[Field::Vectors(VectorField {data: Some(VectorData::BinaryVector(data)), ..}) => Some(data)],
    // milvus-proto assumes that float16 and bfloat16 are stored as little-endian bytes
    Vec<f16>[Field::Vectors(VectorField {data: Some(VectorData::Float16Vector(data)), ..}) => Some(data.chunks(2).map(|x|f16::from_le_bytes([x[0], x[1]])).collect())],
    Vec<bf16>[Field::Vectors(VectorField {data: Some(VectorData::Bfloat16Vector(data)), ..}) => Some(data.chunks(2).map(|x|bf16::from_le_bytes([x[0], x[1]])).collect())]
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

#[cfg(test)]
mod test {
    use crate::data::*;
    #[test]
    fn test_data_type() {
        assert_eq!(bool::data_type(), DataType::Bool);
        assert_eq!(i8::data_type(), DataType::Int8);
        assert_eq!(i16::data_type(), DataType::Int16);
        assert_eq!(i32::data_type(), DataType::Int32);
        assert_eq!(i64::data_type(), DataType::Int64);
        assert_eq!(f32::data_type(), DataType::Float);
        assert_eq!(f64::data_type(), DataType::Double);
        assert_eq!(String::data_type(), DataType::String);
        assert_eq!(std::borrow::Cow::<str>::data_type(), DataType::String);
        assert_eq!(Vec::<f32>::data_type(), DataType::FloatVector);
        assert_eq!(Vec::<f16>::data_type(), DataType::Float16Vector);
        assert_eq!(Vec::<bf16>::data_type(), DataType::BFloat16Vector);
        assert_eq!(Vec::<u8>::data_type(), DataType::BinaryVector);
        assert_eq!(
            std::borrow::Cow::<[f32]>::data_type(),
            DataType::FloatVector
        );
        assert_eq!(
            std::borrow::Cow::<[f16]>::data_type(),
            DataType::Float16Vector
        );
        assert_eq!(
            std::borrow::Cow::<[bf16]>::data_type(),
            DataType::BFloat16Vector
        );
        assert_eq!(
            std::borrow::Cow::<[u8]>::data_type(),
            DataType::BinaryVector
        );
    }

    #[test]
    fn test_field_column() {
        let field = FieldColumn {
            name: "test".to_string(),
            dtype: DataType::Int32,
            value: ValueVec::Int(vec![1, 2, 3]),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        let field_data: schema::FieldData = field.clone().into();
        let field_column: FieldColumn = field_data.into();
        assert_eq!(field, field_column);
    }

    #[test]
    fn test_field_column_from_schema() {
        let field_schema = FieldSchema::new_int32("int32_schema", "");
        let field_column = FieldColumn::new(&field_schema, vec![1, 2, 3]).unwrap();
        assert_eq!(field_column.dtype, DataType::Int32);
        assert_eq!(field_column.dim, 1);
        assert_eq!(field_column.len(), 3);

        let field_schema = FieldSchema::new_float("float_schema", "");
        let field_column_res = FieldColumn::new(&field_schema, Vec::<i64>::new());
        assert!(field_column_res.is_err());
        let field_column = FieldColumn::new(&field_schema, Vec::<f32>::new()).unwrap();
        assert_eq!(field_column.dtype, DataType::Float);
        assert_eq!(field_column.dim, 1);
        assert_eq!(field_column.len(), 0);

        let test_cases: [(fn(&str, &str, i64) -> FieldSchema, DataType); 3] = [
            (FieldSchema::new_bfloat16_vector, DataType::BFloat16Vector),
            (FieldSchema::new_float16_vector, DataType::Float16Vector),
            (FieldSchema::new_float_vector, DataType::FloatVector),
        ];
        for (new_fn, dtype) in test_cases {
            let field_schema = new_fn("feat", "", 8);

            let field_column_res = FieldColumn::new(&field_schema, vec![0.1, 0.2, 0.3]);
            assert!(field_column_res.is_err());

            let field_column_res = FieldColumn::new(&field_schema, Vec::<f32>::new());
            assert!(field_column_res.is_ok());
            let field_column_res = FieldColumn::new(&field_schema, Vec::<f64>::new());
            assert!(field_column_res.is_ok());
            let field_column_res =
                FieldColumn::new(&field_schema, vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
            assert!(field_column_res.is_ok());
            let field_column_res = FieldColumn::new(
                &field_schema,
                vec![
                    0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6,
                ],
            );
            let mut field_column = field_column_res.unwrap();
            assert_eq!(field_column.dtype, dtype);
            assert_eq!(field_column.dim, 8);
            assert_eq!(field_column.len(), 2);
            let value = field_column.get(0).unwrap();
            assert_eq!(value.data_type(), dtype);
            field_column.push(vec![1.7, 1.8, 1.9, 2.0, 2.1, 2.2, 2.3, 2.4].into());
            assert_eq!(field_column.len(), 3);
            let value = field_column.get(2).unwrap();
            assert_eq!(value.data_type(), dtype);
        }
    }
}
