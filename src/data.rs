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
        let dtype = DataType::try_from(fd.r#type).unwrap_or(DataType::None);

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
    pub fn new<V: Into<ValueVec>>(schm: &FieldSchema, v: V) -> FieldColumn {
        FieldColumn {
            name: schm.name.clone(),
            dtype: schm.dtype,
            value: v.into(),
            dim: schm.dim,
            max_length: schm.max_length,
            is_dynamic: false,
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
            ValueVec::Long(v) => match self.dtype {
                DataType::Timestamptz => Value::Timestamptz(*v.get(idx)?),
                _ => Value::Long(*v.get(idx)?),
            },
            ValueVec::Float(v) => match self.dtype {
                DataType::Float => Value::Float(*v.get(idx)?),
                DataType::FloatVector => {
                    let dim = self.dim as usize;
                    Value::FloatArray(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
                }
                _ => unreachable!(),
            },
            ValueVec::Double(v) => Value::Double(*v.get(idx)?),
            ValueVec::Binary(v) => match self.dtype {
                DataType::BinaryVector => {
                    let dim = (self.dim / 8) as usize;
                    Value::Binary(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
                }
                DataType::Int8Vector => {
                    let dim = self.dim as usize;
                    Value::Int8Vector(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
                }
                DataType::Float16Vector => {
                    let dim = (self.dim * 2) as usize;
                    Value::Float16Vector(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
                }
                DataType::BFloat16Vector => {
                    let dim = (self.dim * 2) as usize;
                    Value::BFloat16Vector(Cow::Borrowed(&v[idx * dim..idx * dim + dim]))
                }
                _ => unreachable!(),
            },
            ValueVec::String(v) => Value::String(Cow::Borrowed(v.get(idx)?.as_ref())),
            ValueVec::Json(v) => Value::Json(Cow::Borrowed(v.get(idx)?.as_ref())),
            ValueVec::Array(v) => Value::Array(Cow::Borrowed(v.get(idx)?)),
            ValueVec::Geometry(v) => Value::Geometry(Cow::Borrowed(v.get(idx)?.as_ref())),
            ValueVec::GeometryWkt(v) => Value::GeometryWkt(Cow::Borrowed(v.get(idx)?.as_ref())),
            ValueVec::Timestamptz(v) => Value::Timestamptz(*v.get(idx)?),
            // Known proto-level limitation: aggregate field payloads do not expose
            // row-level element access, so callers can only retrieve the whole
            // aggregate after a bounds check against the batch's row count.
            ValueVec::SparseFloat(v) => {
                if idx >= v.contents.len() {
                    return None;
                }
                Value::SparseFloat(Cow::Borrowed(v))
            }
            ValueVec::StructArray(v) => {
                if idx >= struct_array_row_count(v) {
                    return None;
                }
                Value::StructArray(Cow::Borrowed(v))
            }
            ValueVec::VectorArray(v) => {
                if idx >= v.data.len() {
                    return None;
                }
                Value::VectorArray(Cow::Borrowed(v))
            }
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
            (ValueVec::Long(vec), Value::Timestamptz(i)) => vec.push(i),
            (ValueVec::Timestamptz(vec), Value::Timestamptz(i)) => vec.push(i),
            (ValueVec::Float(vec), Value::Float(i)) => vec.push(i),
            (ValueVec::Double(vec), Value::Double(i)) => vec.push(i),
            (ValueVec::String(vec), Value::String(i)) => vec.push(i.to_string()),
            (ValueVec::Binary(vec), Value::Binary(i))
            | (ValueVec::Binary(vec), Value::Int8Vector(i))
            | (ValueVec::Binary(vec), Value::Float16Vector(i))
            | (ValueVec::Binary(vec), Value::BFloat16Vector(i)) => vec.extend_from_slice(i.as_ref()),
            (ValueVec::Float(vec), Value::FloatArray(i)) => vec.extend_from_slice(i.as_ref()),
            (ValueVec::Geometry(vec), Value::Geometry(i)) => vec.push(i.into_owned()),
            (ValueVec::GeometryWkt(vec), Value::GeometryWkt(i)) => vec.push(i.to_string()),
            // Complex aggregate types: these represent the entire field data and are
            // copied as a whole rather than pushed element by element.
            (ValueVec::SparseFloat(dst), Value::SparseFloat(src)) => *dst = src.into_owned(),
            (ValueVec::StructArray(dst), Value::StructArray(src)) => *dst = src.into_owned(),
            (ValueVec::VectorArray(dst), Value::VectorArray(src)) => *dst = src.into_owned(),
            _ => panic!("column type mismatch"),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let dim = self.dim as usize;
        if dim == 0 {
            return self.value.len();
        }
        match self.dtype {
            // Binary vectors: dim bits per vector = dim/8 bytes per vector
            DataType::BinaryVector => self.value.len() / (dim / 8).max(1),
            // Float16/BFloat16 vectors: 2 bytes per dimension
            DataType::Float16Vector | DataType::BFloat16Vector => self.value.len() / (dim * 2),
            // Int8 vectors: 1 byte per dimension
            DataType::Int8Vector => self.value.len() / dim,
            // Float vectors: 1 float (in the vec) per dimension
            // Scalar types: 1 element per row
            _ => self.value.len() / dim,
        }
    }

    pub fn copy_with_metadata(&self) -> Self {
        // Preserve the actual ValueVec variant rather than recreating from dtype,
        // because some types (e.g., Geometry) can have multiple wire representations
        // (WKB vs WKT) that both map to the same DataType.
        let empty_value = match &self.value {
            ValueVec::None => ValueVec::None,
            ValueVec::Bool(_) => ValueVec::Bool(Vec::new()),
            ValueVec::Int(_) => ValueVec::Int(Vec::new()),
            ValueVec::Long(_) => ValueVec::Long(Vec::new()),
            ValueVec::Float(_) => ValueVec::Float(Vec::new()),
            ValueVec::Double(_) => ValueVec::Double(Vec::new()),
            ValueVec::String(_) => ValueVec::String(Vec::new()),
            ValueVec::Json(_) => ValueVec::Json(Vec::new()),
            ValueVec::Binary(_) => ValueVec::Binary(Vec::new()),
            ValueVec::Array(_) => ValueVec::Array(Vec::new()),
            ValueVec::Geometry(_) => ValueVec::Geometry(Vec::new()),
            ValueVec::GeometryWkt(_) => ValueVec::GeometryWkt(Vec::new()),
            ValueVec::Timestamptz(_) => ValueVec::Timestamptz(Vec::new()),
            ValueVec::SparseFloat(_) => ValueVec::SparseFloat(
                crate::proto::schema::SparseFloatArray {
                    contents: Vec::new(),
                    dim: 0,
                },
            ),
            ValueVec::StructArray(_) => ValueVec::StructArray(
                crate::proto::schema::StructArrayField {
                    fields: Vec::new(),
                },
            ),
            ValueVec::VectorArray(_) => ValueVec::VectorArray(
                crate::proto::schema::VectorArray {
                    dim: 0,
                    data: Vec::new(),
                    element_type: 0,
                },
            ),
        };
        Self {
            dim: self.dim,
            dtype: self.dtype,
            max_length: self.max_length,
            name: self.name.clone(),
            value: empty_value,
            is_dynamic: self.is_dynamic,
        }
    }
}

fn struct_array_row_count(v: &crate::proto::schema::StructArrayField) -> usize {
    v.fields
        .first()
        .cloned()
        .map(FieldColumn::from)
        .map(|column| column.len())
        .unwrap_or(0)
}

impl From<FieldColumn> for schema::FieldData {
    fn from(this: FieldColumn) -> schema::FieldData {
        schema::FieldData {
            field_name: this.name.to_string(),
            field_id: 0,
            valid_data: vec![],
            r#type: this.dtype as _,
            field: Some(match this.value {
                ValueVec::None => Field::Scalars(ScalarField { data: None }),
                ValueVec::Bool(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::BoolData(schema::BoolArray { data: v })),
                }),
                ValueVec::Int(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::IntData(schema::IntArray { data: v })),
                }),
                ValueVec::Long(v) => match this.dtype {
                    DataType::Timestamptz => Field::Scalars(ScalarField {
                        data: Some(ScalarData::TimestamptzData(schema::TimestamptzArray {
                            data: v,
                        })),
                    }),
                    _ => Field::Scalars(ScalarField {
                        data: Some(ScalarData::LongData(schema::LongArray { data: v })),
                    }),
                },
                ValueVec::Float(v) => match this.dtype {
                    DataType::Float => Field::Scalars(ScalarField {
                        data: Some(ScalarData::FloatData(schema::FloatArray { data: v })),
                    }),
                    DataType::FloatVector => Field::Vectors(VectorField {
                        data: Some(VectorData::FloatVector(schema::FloatArray { data: v })),
                        dim: this.dim,
                    }),
                    _ => Field::Scalars(ScalarField {
                        data: Some(ScalarData::FloatData(schema::FloatArray { data: v })),
                    }),
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
                ValueVec::Binary(v) => match this.dtype {
                    DataType::Int8Vector => Field::Vectors(VectorField {
                        data: Some(VectorData::Int8Vector(v)),
                        dim: this.dim,
                    }),
                    DataType::Float16Vector => Field::Vectors(VectorField {
                        data: Some(VectorData::Float16Vector(v)),
                        dim: this.dim,
                    }),
                    DataType::BFloat16Vector => Field::Vectors(VectorField {
                        data: Some(VectorData::Bfloat16Vector(v)),
                        dim: this.dim,
                    }),
                    _ => Field::Vectors(VectorField {
                        data: Some(VectorData::BinaryVector(v)),
                        dim: this.dim,
                    }),
                },
                ValueVec::Geometry(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::GeometryData(schema::GeometryArray { data: v })),
                }),
                ValueVec::GeometryWkt(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::GeometryWktData(schema::GeometryWktArray {
                        data: v,
                    })),
                }),
                ValueVec::Timestamptz(v) => Field::Scalars(ScalarField {
                    data: Some(ScalarData::TimestamptzData(schema::TimestamptzArray {
                        data: v,
                    })),
                }),
                ValueVec::SparseFloat(v) => Field::Vectors(VectorField {
                    data: Some(VectorData::SparseFloatVector(v)),
                    dim: this.dim,
                }),
                ValueVec::StructArray(v) => Field::StructArrays(v),
                ValueVec::VectorArray(v) => Field::Vectors(VectorField {
                    data: Some(VectorData::VectorArray(v)),
                    dim: this.dim,
                }),
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
        Field::StructArrays(_) => 1i64,
        _ => 0i64,
    };

    (Some(dim), None) // no idea how to get max_length
}
