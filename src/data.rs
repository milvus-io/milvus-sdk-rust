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
            ValueVec::SparseFloat(v) => {
                let content = v.contents.get(idx)?.clone();
                Value::SparseFloat(Cow::Owned(schema::SparseFloatArray {
                    contents: vec![content],
                    dim: v.dim,
                }))
            }
            ValueVec::StructArray(v) => {
                let row = struct_array_get_row(v, idx)?;
                Value::StructArray(Cow::Owned(row))
            }
            ValueVec::VectorArray(v) => {
                let entry = v.data.get(idx)?.clone();
                Value::VectorArray(Cow::Owned(schema::VectorArray {
                    dim: v.dim,
                    data: vec![entry],
                    element_type: v.element_type,
                }))
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
            | (ValueVec::Binary(vec), Value::BFloat16Vector(i)) => {
                vec.extend_from_slice(i.as_ref())
            }
            (ValueVec::Float(vec), Value::FloatArray(i)) => vec.extend_from_slice(i.as_ref()),
            (ValueVec::Geometry(vec), Value::Geometry(i)) => vec.push(i.into_owned()),
            (ValueVec::GeometryWkt(vec), Value::GeometryWkt(i)) => vec.push(i.to_string()),
            // SparseFloat: append the single-row contents entry from get(idx).
            (ValueVec::SparseFloat(dst), Value::SparseFloat(src)) => {
                let src = src.into_owned();
                dst.contents.extend(src.contents);
                if src.dim > dst.dim {
                    dst.dim = src.dim;
                }
            }
            // VectorArray: append the single data entry from get(idx).
            (ValueVec::VectorArray(dst), Value::VectorArray(src)) => {
                dst.data.extend(src.into_owned().data);
            }
            // StructArray: merge single-row sub-fields into dst.
            (ValueVec::StructArray(dst), Value::StructArray(src)) => {
                struct_array_push_row(dst, src.into_owned());
            }
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
            ValueVec::SparseFloat(v) => {
                ValueVec::SparseFloat(crate::proto::schema::SparseFloatArray {
                    contents: Vec::new(),
                    dim: v.dim,
                })
            }
            ValueVec::StructArray(_) => {
                ValueVec::StructArray(crate::proto::schema::StructArrayField { fields: Vec::new() })
            }
            ValueVec::VectorArray(v) => ValueVec::VectorArray(crate::proto::schema::VectorArray {
                dim: v.dim,
                data: Vec::new(),
                element_type: v.element_type,
            }),
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

pub(crate) fn slice_field_columns(
    fields_data: &[FieldColumn],
    offset: usize,
    len: usize,
) -> std::result::Result<Vec<FieldColumn>, String> {
    let mut result_data = fields_data
        .iter()
        .map(FieldColumn::copy_with_metadata)
        .collect::<Vec<FieldColumn>>();

    for j in 0..fields_data.len() {
        for i in offset..offset + len {
            if i >= fields_data[j].len() {
                return Err(format!(
                    "field data bounds exceeded: field={}, index={}, field_len={}",
                    fields_data[j].name,
                    i,
                    fields_data[j].len()
                ));
            }
            result_data[j].push(
                fields_data[j]
                    .get(i)
                    .ok_or_else(|| "out of range while indexing field data".to_owned())?,
            );
        }
    }

    Ok(result_data)
}

/// Extract a single row from a StructArrayField by slicing each sub-field.
fn struct_array_get_row(
    v: &crate::proto::schema::StructArrayField,
    idx: usize,
) -> Option<crate::proto::schema::StructArrayField> {
    let fields = v
        .fields
        .iter()
        .map(|fd| field_data_get_row(fd, idx))
        .collect::<Option<Vec<_>>>()?;
    Some(crate::proto::schema::StructArrayField { fields })
}

/// Extract row `idx` from a single FieldData column.
fn field_data_get_row(
    fd: &crate::proto::schema::FieldData,
    idx: usize,
) -> Option<crate::proto::schema::FieldData> {
    let field = match fd.field.as_ref()? {
        Field::Scalars(s) => {
            let data = scalar_data_get_row(s.data.as_ref()?, idx)?;
            Field::Scalars(ScalarField { data: Some(data) })
        }
        Field::Vectors(v) => Field::Vectors(vector_field_get_row(v, idx)?),
        Field::StructArrays(sa) => Field::StructArrays(struct_array_get_row(sa, idx)?),
    };
    Some(crate::proto::schema::FieldData {
        r#type: fd.r#type,
        field_name: fd.field_name.clone(),
        field_id: fd.field_id,
        is_dynamic: fd.is_dynamic,
        valid_data: if idx < fd.valid_data.len() {
            vec![fd.valid_data[idx]]
        } else {
            vec![]
        },
        field: Some(field),
    })
}

fn scalar_data_get_row(sd: &ScalarData, idx: usize) -> Option<ScalarData> {
    Some(match sd {
        ScalarData::BoolData(v) => ScalarData::BoolData(schema::BoolArray {
            data: vec![*v.data.get(idx)?],
        }),
        ScalarData::IntData(v) => ScalarData::IntData(schema::IntArray {
            data: vec![*v.data.get(idx)?],
        }),
        ScalarData::LongData(v) => ScalarData::LongData(schema::LongArray {
            data: vec![*v.data.get(idx)?],
        }),
        ScalarData::FloatData(v) => ScalarData::FloatData(schema::FloatArray {
            data: vec![*v.data.get(idx)?],
        }),
        ScalarData::DoubleData(v) => ScalarData::DoubleData(schema::DoubleArray {
            data: vec![*v.data.get(idx)?],
        }),
        ScalarData::StringData(v) => ScalarData::StringData(schema::StringArray {
            data: vec![v.data.get(idx)?.clone()],
        }),
        ScalarData::BytesData(v) => ScalarData::BytesData(schema::BytesArray {
            data: vec![v.data.get(idx)?.clone()],
        }),
        ScalarData::ArrayData(v) => ScalarData::ArrayData(schema::ArrayArray {
            data: vec![v.data.get(idx)?.clone()],
            element_type: v.element_type,
        }),
        ScalarData::JsonData(v) => ScalarData::JsonData(schema::JsonArray {
            data: vec![v.data.get(idx)?.clone()],
        }),
        ScalarData::GeometryData(v) => ScalarData::GeometryData(schema::GeometryArray {
            data: vec![v.data.get(idx)?.clone()],
        }),
        ScalarData::TimestamptzData(v) => ScalarData::TimestamptzData(schema::TimestamptzArray {
            data: vec![*v.data.get(idx)?],
        }),
        ScalarData::GeometryWktData(v) => ScalarData::GeometryWktData(schema::GeometryWktArray {
            data: vec![v.data.get(idx)?.clone()],
        }),
    })
}

/// Extract row `idx` from a VectorField (used for struct sub-fields).
fn vector_field_get_row(vf: &VectorField, idx: usize) -> Option<VectorField> {
    let dim = vf.dim;
    let data = match vf.data.as_ref()? {
        VectorData::FloatVector(v) => {
            let d = dim.max(1) as usize;
            let start = idx * d;
            if start + d > v.data.len() {
                return None;
            }
            VectorData::FloatVector(schema::FloatArray {
                data: v.data[start..start + d].to_vec(),
            })
        }
        VectorData::BinaryVector(v) => {
            let bytes_per_row = (dim as usize / 8).max(1);
            let start = idx * bytes_per_row;
            if start + bytes_per_row > v.len() {
                return None;
            }
            VectorData::BinaryVector(v[start..start + bytes_per_row].to_vec())
        }
        VectorData::Float16Vector(v) => {
            let bytes_per_row = (dim.max(1) as usize) * 2;
            let start = idx * bytes_per_row;
            if start + bytes_per_row > v.len() {
                return None;
            }
            VectorData::Float16Vector(v[start..start + bytes_per_row].to_vec())
        }
        VectorData::Bfloat16Vector(v) => {
            let bytes_per_row = (dim.max(1) as usize) * 2;
            let start = idx * bytes_per_row;
            if start + bytes_per_row > v.len() {
                return None;
            }
            VectorData::Bfloat16Vector(v[start..start + bytes_per_row].to_vec())
        }
        VectorData::SparseFloatVector(v) => {
            let content = v.contents.get(idx)?.clone();
            VectorData::SparseFloatVector(schema::SparseFloatArray {
                contents: vec![content],
                dim: v.dim,
            })
        }
        VectorData::Int8Vector(v) => {
            let d = dim.max(1) as usize;
            let start = idx * d;
            if start + d > v.len() {
                return None;
            }
            VectorData::Int8Vector(v[start..start + d].to_vec())
        }
        VectorData::VectorArray(v) => {
            let entry = v.data.get(idx)?.clone();
            VectorData::VectorArray(schema::VectorArray {
                dim: v.dim,
                data: vec![entry],
                element_type: v.element_type,
            })
        }
    };
    Some(VectorField {
        dim,
        data: Some(data),
    })
}

/// Merge a single-row StructArrayField into an accumulator.
fn struct_array_push_row(
    dst: &mut crate::proto::schema::StructArrayField,
    src: crate::proto::schema::StructArrayField,
) {
    if dst.fields.is_empty() {
        dst.fields = src.fields;
        return;
    }
    debug_assert_eq!(
        dst.fields.len(),
        src.fields.len(),
        "struct array field count mismatch in push"
    );
    for (dst_fd, src_fd) in dst.fields.iter_mut().zip(src.fields.into_iter()) {
        field_data_push_row(dst_fd, src_fd);
    }
}

fn field_data_push_row(
    dst: &mut crate::proto::schema::FieldData,
    src: crate::proto::schema::FieldData,
) {
    dst.valid_data.extend(src.valid_data);
    match (&mut dst.field, src.field) {
        (Some(dst_field), Some(src_field)) => match (dst_field, src_field) {
            (Field::Scalars(dst_s), Field::Scalars(src_s)) => {
                if let (Some(dst_d), Some(src_d)) = (&mut dst_s.data, src_s.data) {
                    scalar_data_push_row(dst_d, src_d);
                }
            }
            (Field::Vectors(dst_v), Field::Vectors(src_v)) => {
                vector_field_push_row(dst_v, src_v);
            }
            (Field::StructArrays(dst_sa), Field::StructArrays(src_sa)) => {
                struct_array_push_row(dst_sa, src_sa);
            }
            _ => panic!("field type mismatch in struct array push"),
        },
        (dst_slot @ None, src_field) => {
            *dst_slot = src_field;
        }
        _ => {}
    }
}

fn scalar_data_push_row(dst: &mut ScalarData, src: ScalarData) {
    match (dst, src) {
        (ScalarData::BoolData(d), ScalarData::BoolData(s)) => d.data.extend(s.data),
        (ScalarData::IntData(d), ScalarData::IntData(s)) => d.data.extend(s.data),
        (ScalarData::LongData(d), ScalarData::LongData(s)) => d.data.extend(s.data),
        (ScalarData::FloatData(d), ScalarData::FloatData(s)) => d.data.extend(s.data),
        (ScalarData::DoubleData(d), ScalarData::DoubleData(s)) => d.data.extend(s.data),
        (ScalarData::StringData(d), ScalarData::StringData(s)) => d.data.extend(s.data),
        (ScalarData::BytesData(d), ScalarData::BytesData(s)) => d.data.extend(s.data),
        (ScalarData::ArrayData(d), ScalarData::ArrayData(s)) => d.data.extend(s.data),
        (ScalarData::JsonData(d), ScalarData::JsonData(s)) => d.data.extend(s.data),
        (ScalarData::GeometryData(d), ScalarData::GeometryData(s)) => d.data.extend(s.data),
        (ScalarData::TimestamptzData(d), ScalarData::TimestamptzData(s)) => d.data.extend(s.data),
        (ScalarData::GeometryWktData(d), ScalarData::GeometryWktData(s)) => d.data.extend(s.data),
        _ => panic!("scalar type mismatch in struct array push"),
    }
}

fn vector_field_push_row(dst: &mut VectorField, src: VectorField) {
    if let (Some(dst_d), Some(src_d)) = (&mut dst.data, src.data) {
        match (dst_d, src_d) {
            (VectorData::FloatVector(d), VectorData::FloatVector(s)) => d.data.extend(s.data),
            (VectorData::BinaryVector(d), VectorData::BinaryVector(s)) => d.extend(s),
            (VectorData::Float16Vector(d), VectorData::Float16Vector(s)) => d.extend(s),
            (VectorData::Bfloat16Vector(d), VectorData::Bfloat16Vector(s)) => d.extend(s),
            (VectorData::SparseFloatVector(d), VectorData::SparseFloatVector(s)) => {
                d.contents.extend(s.contents);
                if s.dim > d.dim {
                    d.dim = s.dim;
                }
            }
            (VectorData::Int8Vector(d), VectorData::Int8Vector(s)) => d.extend(s),
            (VectorData::VectorArray(d), VectorData::VectorArray(s)) => d.data.extend(s.data),
            _ => panic!("vector type mismatch in struct array push"),
        }
    }
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

#[cfg(test)]
mod test {
    use super::{slice_field_columns, FieldColumn};
    use crate::{
        proto::schema::{
            self, field_data::Field, scalar_field::Data as ScalarData,
            vector_field::Data as VectorData, DataType, ScalarField, SparseFloatArray, VectorField,
        },
        value::{Value, ValueVec},
    };

    #[test]
    fn sparse_float_get_returns_single_row() {
        let column = FieldColumn {
            name: "sparse".to_string(),
            dtype: DataType::SparseFloatVector,
            value: ValueVec::SparseFloat(SparseFloatArray {
                contents: vec![vec![1, 2], vec![3, 4]],
                dim: 8,
            }),
            dim: 0,
            max_length: 0,
            is_dynamic: false,
        };

        let value = column.get(1).unwrap();
        match value {
            Value::SparseFloat(data) => {
                assert_eq!(data.dim, 8);
                assert_eq!(data.contents, vec![vec![3, 4]]);
            }
            _ => panic!("expected sparse float row"),
        }
    }

    #[test]
    fn sparse_float_push_appends_rows() {
        let mut col = FieldColumn {
            name: "sparse".to_string(),
            dtype: DataType::SparseFloatVector,
            value: ValueVec::SparseFloat(SparseFloatArray {
                contents: vec![],
                dim: 0,
            }),
            dim: 0,
            max_length: 0,
            is_dynamic: false,
        };

        let source = FieldColumn {
            name: "sparse".to_string(),
            dtype: DataType::SparseFloatVector,
            value: ValueVec::SparseFloat(SparseFloatArray {
                contents: vec![vec![1, 2], vec![3, 4], vec![5, 6]],
                dim: 8,
            }),
            dim: 0,
            max_length: 0,
            is_dynamic: false,
        };

        // Push rows 0 and 2 (simulating topk result slicing)
        col.push(source.get(0).unwrap());
        col.push(source.get(2).unwrap());

        match &col.value {
            ValueVec::SparseFloat(v) => {
                assert_eq!(v.contents.len(), 2);
                assert_eq!(v.contents[0], vec![1, 2]);
                assert_eq!(v.contents[1], vec![5, 6]);
                assert_eq!(v.dim, 8);
            }
            _ => panic!("expected sparse float"),
        }
    }

    #[test]
    fn vector_array_get_returns_single_entry() {
        let vf0 = VectorField {
            dim: 4,
            data: Some(VectorData::FloatVector(schema::FloatArray {
                data: vec![1.0, 2.0, 3.0, 4.0],
            })),
        };
        let vf1 = VectorField {
            dim: 4,
            data: Some(VectorData::FloatVector(schema::FloatArray {
                data: vec![5.0, 6.0, 7.0, 8.0],
            })),
        };

        let column = FieldColumn {
            name: "va".to_string(),
            dtype: DataType::FloatVector,
            value: ValueVec::VectorArray(schema::VectorArray {
                dim: 4,
                data: vec![vf0, vf1],
                element_type: DataType::FloatVector as i32,
            }),
            dim: 4,
            max_length: 0,
            is_dynamic: false,
        };

        // get(1) should return only the second entry
        let val = column.get(1).unwrap();
        match val {
            Value::VectorArray(va) => {
                assert_eq!(va.data.len(), 1);
                match va.data[0].data.as_ref().unwrap() {
                    VectorData::FloatVector(f) => {
                        assert_eq!(f.data, vec![5.0, 6.0, 7.0, 8.0]);
                    }
                    _ => panic!("expected float vector"),
                }
            }
            _ => panic!("expected vector array"),
        }

        // Out of bounds
        assert!(column.get(2).is_none());
    }

    #[test]
    fn vector_array_push_appends_entries() {
        let vf0 = VectorField {
            dim: 2,
            data: Some(VectorData::FloatVector(schema::FloatArray {
                data: vec![1.0, 2.0],
            })),
        };
        let vf1 = VectorField {
            dim: 2,
            data: Some(VectorData::FloatVector(schema::FloatArray {
                data: vec![3.0, 4.0],
            })),
        };
        let vf2 = VectorField {
            dim: 2,
            data: Some(VectorData::FloatVector(schema::FloatArray {
                data: vec![5.0, 6.0],
            })),
        };

        let source = FieldColumn {
            name: "va".to_string(),
            dtype: DataType::FloatVector,
            value: ValueVec::VectorArray(schema::VectorArray {
                dim: 2,
                data: vec![vf0, vf1, vf2],
                element_type: DataType::FloatVector as i32,
            }),
            dim: 2,
            max_length: 0,
            is_dynamic: false,
        };

        let mut result = source.copy_with_metadata();

        // Push rows 0 and 2
        result.push(source.get(0).unwrap());
        result.push(source.get(2).unwrap());

        match &result.value {
            ValueVec::VectorArray(va) => {
                assert_eq!(va.data.len(), 2);
                assert_eq!(va.dim, 2);
                assert_eq!(va.element_type, DataType::FloatVector as i32);
                // First entry should be vf0's data
                match va.data[0].data.as_ref().unwrap() {
                    VectorData::FloatVector(f) => assert_eq!(f.data, vec![1.0, 2.0]),
                    _ => panic!("expected float vector"),
                }
                // Second entry should be vf2's data
                match va.data[1].data.as_ref().unwrap() {
                    VectorData::FloatVector(f) => assert_eq!(f.data, vec![5.0, 6.0]),
                    _ => panic!("expected float vector"),
                }
            }
            _ => panic!("expected vector array"),
        }
    }

    #[test]
    fn struct_array_get_returns_single_row() {
        // Build a StructArrayField with two sub-fields: Int [10, 20, 30] and String ["a", "b", "c"]
        let int_field = schema::FieldData {
            r#type: DataType::Int32 as i32,
            field_name: "id".to_string(),
            field_id: 1,
            is_dynamic: false,
            valid_data: vec![],
            field: Some(Field::Scalars(ScalarField {
                data: Some(ScalarData::IntData(schema::IntArray {
                    data: vec![10, 20, 30],
                })),
            })),
        };
        let str_field = schema::FieldData {
            r#type: DataType::String as i32,
            field_name: "name".to_string(),
            field_id: 2,
            is_dynamic: false,
            valid_data: vec![],
            field: Some(Field::Scalars(ScalarField {
                data: Some(ScalarData::StringData(schema::StringArray {
                    data: vec!["a".into(), "b".into(), "c".into()],
                })),
            })),
        };

        let column = FieldColumn {
            name: "sa".to_string(),
            dtype: DataType::Array,
            value: ValueVec::StructArray(schema::StructArrayField {
                fields: vec![int_field, str_field],
            }),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        // get(1) should extract row at index 1: Int=20, String="b"
        let val = column.get(1).unwrap();
        match val {
            Value::StructArray(sa) => {
                assert_eq!(sa.fields.len(), 2);
                match sa.fields[0].field.as_ref().unwrap() {
                    Field::Scalars(s) => match s.data.as_ref().unwrap() {
                        ScalarData::IntData(v) => assert_eq!(v.data, vec![20]),
                        _ => panic!("expected int data"),
                    },
                    _ => panic!("expected scalars"),
                }
                match sa.fields[1].field.as_ref().unwrap() {
                    Field::Scalars(s) => match s.data.as_ref().unwrap() {
                        ScalarData::StringData(v) => assert_eq!(v.data, vec!["b".to_string()]),
                        _ => panic!("expected string data"),
                    },
                    _ => panic!("expected scalars"),
                }
            }
            _ => panic!("expected struct array"),
        }

        // Out of bounds
        assert!(column.get(3).is_none());
    }

    #[test]
    fn struct_array_push_merges_rows() {
        let int_field = schema::FieldData {
            r#type: DataType::Int32 as i32,
            field_name: "id".to_string(),
            field_id: 1,
            is_dynamic: false,
            valid_data: vec![],
            field: Some(Field::Scalars(ScalarField {
                data: Some(ScalarData::IntData(schema::IntArray {
                    data: vec![10, 20, 30],
                })),
            })),
        };
        let str_field = schema::FieldData {
            r#type: DataType::String as i32,
            field_name: "name".to_string(),
            field_id: 2,
            is_dynamic: false,
            valid_data: vec![],
            field: Some(Field::Scalars(ScalarField {
                data: Some(ScalarData::StringData(schema::StringArray {
                    data: vec!["a".into(), "b".into(), "c".into()],
                })),
            })),
        };

        let source = FieldColumn {
            name: "sa".to_string(),
            dtype: DataType::Array,
            value: ValueVec::StructArray(schema::StructArrayField {
                fields: vec![int_field, str_field],
            }),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        let mut result = source.copy_with_metadata();

        // Push rows 0 and 2 (simulating result slicing with offset)
        result.push(source.get(0).unwrap());
        result.push(source.get(2).unwrap());

        match &result.value {
            ValueVec::StructArray(sa) => {
                assert_eq!(sa.fields.len(), 2);
                // Int sub-field should have [10, 30]
                match sa.fields[0].field.as_ref().unwrap() {
                    Field::Scalars(s) => match s.data.as_ref().unwrap() {
                        ScalarData::IntData(v) => assert_eq!(v.data, vec![10, 30]),
                        _ => panic!("expected int data"),
                    },
                    _ => panic!("expected scalars"),
                }
                // String sub-field should have ["a", "c"]
                match sa.fields[1].field.as_ref().unwrap() {
                    Field::Scalars(s) => match s.data.as_ref().unwrap() {
                        ScalarData::StringData(v) => {
                            assert_eq!(v.data, vec!["a".to_string(), "c".to_string()])
                        }
                        _ => panic!("expected string data"),
                    },
                    _ => panic!("expected scalars"),
                }
            }
            _ => panic!("expected struct array"),
        }
    }

    #[test]
    fn struct_array_get_and_push_support_vector_subfields() {
        let vector_field = schema::FieldData {
            r#type: DataType::FloatVector as i32,
            field_name: "embedding".to_string(),
            field_id: 1,
            is_dynamic: false,
            valid_data: vec![true, false, true],
            field: Some(Field::Vectors(VectorField {
                dim: 2,
                data: Some(VectorData::FloatVector(schema::FloatArray {
                    data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                })),
            })),
        };

        let source = FieldColumn {
            name: "sa".to_string(),
            dtype: DataType::ArrayOfStruct,
            value: ValueVec::StructArray(schema::StructArrayField {
                fields: vec![vector_field],
            }),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        let row = source.get(2).unwrap();
        match row {
            Value::StructArray(sa) => {
                assert_eq!(sa.fields.len(), 1);
                assert_eq!(sa.fields[0].valid_data, vec![true]);
                match sa.fields[0].field.as_ref().unwrap() {
                    Field::Vectors(v) => match v.data.as_ref().unwrap() {
                        VectorData::FloatVector(f) => assert_eq!(f.data, vec![5.0, 6.0]),
                        _ => panic!("expected float vector"),
                    },
                    _ => panic!("expected vectors"),
                }
            }
            _ => panic!("expected struct array"),
        }

        let mut result = source.copy_with_metadata();
        result.push(source.get(0).unwrap());
        result.push(source.get(2).unwrap());

        match &result.value {
            ValueVec::StructArray(sa) => {
                assert_eq!(sa.fields.len(), 1);
                assert_eq!(sa.fields[0].valid_data, vec![true, true]);
                match sa.fields[0].field.as_ref().unwrap() {
                    Field::Vectors(v) => match v.data.as_ref().unwrap() {
                        VectorData::FloatVector(f) => assert_eq!(f.data, vec![1.0, 2.0, 5.0, 6.0]),
                        _ => panic!("expected float vector"),
                    },
                    _ => panic!("expected vectors"),
                }
            }
            _ => panic!("expected struct array"),
        }
    }

    #[test]
    fn struct_array_push_preserves_nullable_scalar_valid_data() {
        let nullable_field = schema::FieldData {
            r#type: DataType::String as i32,
            field_name: "name".to_string(),
            field_id: 1,
            is_dynamic: false,
            valid_data: vec![true, false, true],
            field: Some(Field::Scalars(ScalarField {
                data: Some(ScalarData::StringData(schema::StringArray {
                    data: vec!["a".into(), "".into(), "c".into()],
                })),
            })),
        };

        let source = FieldColumn {
            name: "nullable".to_string(),
            dtype: DataType::ArrayOfStruct,
            value: ValueVec::StructArray(schema::StructArrayField {
                fields: vec![nullable_field],
            }),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        let mut result = source.copy_with_metadata();
        result.push(source.get(0).unwrap());
        result.push(source.get(1).unwrap());
        result.push(source.get(2).unwrap());

        match &result.value {
            ValueVec::StructArray(sa) => {
                assert_eq!(sa.fields.len(), 1);
                assert_eq!(sa.fields[0].valid_data, vec![true, false, true]);
                match sa.fields[0].field.as_ref().unwrap() {
                    Field::Scalars(s) => match s.data.as_ref().unwrap() {
                        ScalarData::StringData(v) => {
                            assert_eq!(
                                v.data,
                                vec!["a".to_string(), "".to_string(), "c".to_string()]
                            )
                        }
                        _ => panic!("expected string data"),
                    },
                    _ => panic!("expected scalars"),
                }
            }
            _ => panic!("expected struct array"),
        }
    }

    #[test]
    fn slice_field_columns_slices_sparse_float_rows() {
        let source = FieldColumn {
            name: "sparse".to_string(),
            dtype: DataType::SparseFloatVector,
            value: ValueVec::SparseFloat(SparseFloatArray {
                contents: vec![vec![1, 2], vec![3, 4], vec![5, 6]],
                dim: 8,
            }),
            dim: 0,
            max_length: 0,
            is_dynamic: false,
        };

        let sliced = slice_field_columns(&[source], 1, 2).unwrap();
        match &sliced[0].value {
            ValueVec::SparseFloat(v) => {
                assert_eq!(v.contents, vec![vec![3, 4], vec![5, 6]]);
                assert_eq!(v.dim, 8);
            }
            _ => panic!("expected sparse float"),
        }
    }

    #[test]
    fn slice_field_columns_slices_vector_array_rows() {
        let source = FieldColumn {
            name: "va".to_string(),
            dtype: DataType::ArrayOfVector,
            value: ValueVec::VectorArray(schema::VectorArray {
                dim: 2,
                data: vec![
                    VectorField {
                        dim: 2,
                        data: Some(VectorData::FloatVector(schema::FloatArray {
                            data: vec![1.0, 2.0],
                        })),
                    },
                    VectorField {
                        dim: 2,
                        data: Some(VectorData::FloatVector(schema::FloatArray {
                            data: vec![3.0, 4.0],
                        })),
                    },
                    VectorField {
                        dim: 2,
                        data: Some(VectorData::FloatVector(schema::FloatArray {
                            data: vec![5.0, 6.0],
                        })),
                    },
                ],
                element_type: DataType::FloatVector as i32,
            }),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        let sliced = slice_field_columns(&[source], 0, 2).unwrap();
        match &sliced[0].value {
            ValueVec::VectorArray(v) => {
                assert_eq!(v.data.len(), 2);
                match v.data[0].data.as_ref().unwrap() {
                    VectorData::FloatVector(f) => assert_eq!(f.data, vec![1.0, 2.0]),
                    _ => panic!("expected float vector"),
                }
                match v.data[1].data.as_ref().unwrap() {
                    VectorData::FloatVector(f) => assert_eq!(f.data, vec![3.0, 4.0]),
                    _ => panic!("expected float vector"),
                }
            }
            _ => panic!("expected vector array"),
        }
    }

    #[test]
    fn slice_field_columns_slices_struct_array_rows() {
        let source = FieldColumn {
            name: "sa".to_string(),
            dtype: DataType::ArrayOfStruct,
            value: ValueVec::StructArray(schema::StructArrayField {
                fields: vec![schema::FieldData {
                    r#type: DataType::String as i32,
                    field_name: "name".to_string(),
                    field_id: 1,
                    is_dynamic: false,
                    valid_data: vec![true, true, true],
                    field: Some(Field::Scalars(ScalarField {
                        data: Some(ScalarData::StringData(schema::StringArray {
                            data: vec!["a".into(), "b".into(), "c".into()],
                        })),
                    })),
                }],
            }),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };

        let sliced = slice_field_columns(&[source], 1, 2).unwrap();
        match &sliced[0].value {
            ValueVec::StructArray(v) => match v.fields[0].field.as_ref().unwrap() {
                Field::Scalars(s) => match s.data.as_ref().unwrap() {
                    ScalarData::StringData(arr) => {
                        assert_eq!(arr.data, vec!["b".to_string(), "c".to_string()])
                    }
                    _ => panic!("expected string data"),
                },
                _ => panic!("expected scalars"),
            },
            _ => panic!("expected struct array"),
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
