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

//! Response types returned by query and search operations.
//!
//! Query and search responses may contain multiple result groups when a request has multiple query
//! vectors. Iterate each group and then its rows before reading IDs, scores, or output fields. The
//! borrowed row iterators avoid materializing every result as an owned JSON map.

use crate::proto::{milvus, schema};
use crate::v2::error::{Error, Result};
use crate::v2::types::Ids;
use crate::v2::types::{group_aggregation_buckets, DataType, FieldData, SparseVector};
pub use crate::v2::types::{HighlightResult, QueryResults, SearchResults, SingleResult};
use std::collections::{HashMap, HashSet};

fn field_data(value: schema::FieldData) -> Result<FieldData> {
    let field_name = value.field_name.clone();
    decode_field_data(value).ok_or_else(|| {
        Error::MalformedResponse(format!("failed to decode response field {field_name:?}"))
    })
}

fn decode_field_data(value: schema::FieldData) -> Option<FieldData> {
    use schema::{field_data::Field, scalar_field, vector_field};
    let data_type = schema::DataType::try_from(value.r#type).ok();
    // Prefer the field-specific validity channel introduced by newer servers;
    // fall back to the legacy top-level field only for legacy writers.
    let valid_data = match value.field {
        Some(Field::Scalars(ref scalars)) if !scalars.valid_data.is_empty() => {
            scalars.valid_data.clone()
        }
        Some(Field::Vectors(ref vectors)) if !vectors.valid_data.is_empty() => {
            vectors.valid_data.clone()
        }
        _ => value.valid_data.clone(),
    };
    let valid_count = valid_data.iter().filter(|valid| **valid).count();
    let name = value.field_name;
    let data = match value.field? {
        Field::Scalars(scalars) => match scalars.data? {
            scalar_field::Data::BoolData(v)
                if matches!(data_type, Some(schema::DataType::Bool)) =>
            {
                Some(FieldData::Bool {
                    name,
                    values: v.data,
                })
            }
            scalar_field::Data::IntData(v) => match data_type {
                Some(schema::DataType::Int8) => Some(FieldData::Int8 {
                    name,
                    values: v
                        .data
                        .into_iter()
                        .map(i8::try_from)
                        .collect::<std::result::Result<_, _>>()
                        .ok()?,
                }),
                Some(schema::DataType::Int16) => Some(FieldData::Int16 {
                    name,
                    values: v
                        .data
                        .into_iter()
                        .map(i16::try_from)
                        .collect::<std::result::Result<_, _>>()
                        .ok()?,
                }),
                Some(schema::DataType::Int32) => Some(FieldData::Int32 {
                    name,
                    values: v.data,
                }),
                _ => None,
            },
            scalar_field::Data::LongData(v)
                if matches!(data_type, Some(schema::DataType::Int64)) =>
            {
                Some(FieldData::Int64 {
                    name,
                    values: v.data,
                })
            }
            scalar_field::Data::FloatData(v)
                if matches!(data_type, Some(schema::DataType::Float)) =>
            {
                Some(FieldData::Float {
                    name,
                    values: v.data,
                })
            }
            scalar_field::Data::DoubleData(v)
                if matches!(data_type, Some(schema::DataType::Double)) =>
            {
                Some(FieldData::Double {
                    name,
                    values: v.data,
                })
            }
            scalar_field::Data::StringData(v) => match data_type {
                Some(schema::DataType::Timestamptz) => Some(FieldData::Timestamptz {
                    name,
                    values: v.data,
                }),
                Some(schema::DataType::String | schema::DataType::VarChar) => {
                    Some(FieldData::VarChar {
                        name,
                        values: v.data,
                    })
                }
                _ => None,
            },
            scalar_field::Data::JsonData(v)
                if matches!(data_type, Some(schema::DataType::Json)) =>
            {
                let values = if v.data.len() == valid_data.len() {
                    v.data
                        .into_iter()
                        .zip(&valid_data)
                        .filter_map(|(bytes, valid)| (*valid).then_some(bytes))
                        .collect()
                } else {
                    v.data
                };
                Some(FieldData::Json {
                    name,
                    values: values
                        .into_iter()
                        .map(|bytes| serde_json::from_slice(&bytes).ok())
                        .collect::<Option<Vec<_>>>()?,
                })
            }
            scalar_field::Data::GeometryWktData(v)
                if matches!(data_type, Some(schema::DataType::Geometry)) =>
            {
                Some(FieldData::Geometry {
                    name,
                    values: v.data,
                })
            }
            scalar_field::Data::TimestamptzData(v)
                if matches!(data_type, Some(schema::DataType::Timestamptz)) =>
            {
                Some(FieldData::Timestamptz {
                    name,
                    values: v.data.into_iter().map(|value| value.to_string()).collect(),
                })
            }
            scalar_field::Data::ArrayData(v)
                if matches!(data_type, Some(schema::DataType::Array)) =>
            {
                let element_type = schema::DataType::try_from(v.element_type)
                    .ok()
                    .and_then(|value| DataType::try_from_proto(value).ok())?;
                let rows = if v.data.len() == valid_data.len() {
                    v.data
                        .into_iter()
                        .zip(&valid_data)
                        .filter_map(|(value, valid)| valid.then_some(value))
                        .collect()
                } else {
                    v.data
                };
                array_field_data(name, element_type, rows)
            }
            _ => None,
        },
        Field::Vectors(vectors) => {
            let dimension = usize::try_from(vectors.dim).ok()?;
            match vectors.data {
                Some(vector_field::Data::FloatVector(v))
                    if matches!(data_type, Some(schema::DataType::FloatVector))
                        && dimension > 0
                        && v.data.len() % dimension == 0 =>
                {
                    Some(FieldData::FloatVector {
                        name,
                        values: v.data.chunks(dimension).map(<[f32]>::to_vec).collect(),
                    })
                }
                Some(vector_field::Data::BinaryVector(v))
                    if matches!(data_type, Some(schema::DataType::BinaryVector))
                        && dimension > 0
                        && dimension % 8 == 0
                        && v.len() % (dimension / 8) == 0 =>
                {
                    Some(FieldData::BinaryVector {
                        name,
                        values: v.chunks(dimension / 8).map(<[u8]>::to_vec).collect(),
                    })
                }
                Some(vector_field::Data::Float16Vector(v))
                    if matches!(data_type, Some(schema::DataType::Float16Vector))
                        && dimension > 0 =>
                {
                    Some(FieldData::Float16Vector {
                        name,
                        values: decode_u16_vectors(&v, dimension)?,
                    })
                }
                Some(vector_field::Data::Bfloat16Vector(v))
                    if matches!(data_type, Some(schema::DataType::BFloat16Vector))
                        && dimension > 0 =>
                {
                    Some(FieldData::BFloat16Vector {
                        name,
                        values: decode_u16_vectors(&v, dimension)?,
                    })
                }
                Some(vector_field::Data::Int8Vector(v))
                    if matches!(data_type, Some(schema::DataType::Int8Vector))
                        && dimension > 0
                        && v.len() % dimension == 0 =>
                {
                    Some(FieldData::Int8Vector {
                        name,
                        values: v
                            .chunks(dimension)
                            .map(|row| row.iter().map(|byte| *byte as i8).collect())
                            .collect(),
                    })
                }
                Some(vector_field::Data::SparseFloatVector(v))
                    if matches!(data_type, Some(schema::DataType::SparseFloatVector)) =>
                {
                    Some(FieldData::SparseFloatVector {
                        name,
                        values: v
                            .contents
                            .into_iter()
                            .map(decode_sparse_vector)
                            .collect::<Option<Vec<_>>>()?,
                    })
                }
                None if valid_count == 0 => empty_vector_field_data(name, data_type, dimension),
                _ => None,
            }
        }
        Field::StructArrays(value)
            if matches!(data_type, Some(schema::DataType::ArrayOfStruct)) =>
        {
            struct_field_data(name, value)
        }
        Field::StructArrays(_) => None,
    }?;
    if valid_data.is_empty() {
        Some(data)
    } else {
        let data = if data.len() == valid_data.len() && data.len() != valid_count {
            compact_field_data(data, &valid_data)?
        } else {
            data
        };
        FieldData::nullable(data, valid_data).ok()
    }
}

fn empty_vector_field_data(
    name: String,
    data_type: Option<schema::DataType>,
    dimension: usize,
) -> Option<FieldData> {
    Some(match data_type? {
        schema::DataType::FloatVector if dimension > 0 => FieldData::FloatVector {
            name,
            values: Vec::new(),
        },
        schema::DataType::BinaryVector if dimension > 0 && dimension % 8 == 0 => {
            FieldData::BinaryVector {
                name,
                values: Vec::new(),
            }
        }
        schema::DataType::Float16Vector if dimension > 0 => FieldData::Float16Vector {
            name,
            values: Vec::new(),
        },
        schema::DataType::BFloat16Vector if dimension > 0 => FieldData::BFloat16Vector {
            name,
            values: Vec::new(),
        },
        schema::DataType::SparseFloatVector => FieldData::SparseFloatVector {
            name,
            values: Vec::new(),
        },
        schema::DataType::Int8Vector if dimension > 0 => FieldData::Int8Vector {
            name,
            values: Vec::new(),
        },
        _ => return None,
    })
}

fn array_field_data(
    name: String,
    element_type: DataType,
    rows: Vec<schema::ScalarField>,
) -> Option<FieldData> {
    use schema::scalar_field;

    fn decode_rows<T>(
        rows: Vec<schema::ScalarField>,
        decode: impl Fn(schema::ScalarField) -> Option<Vec<T>>,
    ) -> Option<Vec<Vec<T>>> {
        rows.into_iter().map(decode).collect()
    }

    Some(match element_type {
        DataType::Bool => FieldData::ArrayBool {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::BoolData(values) => Some(values.data),
                _ => None,
            })?,
        },
        DataType::Int8 => FieldData::ArrayInt8 {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::IntData(values) => values
                    .data
                    .into_iter()
                    .map(i8::try_from)
                    .collect::<std::result::Result<_, _>>()
                    .ok(),
                _ => None,
            })?,
        },
        DataType::Int16 => FieldData::ArrayInt16 {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::IntData(values) => values
                    .data
                    .into_iter()
                    .map(i16::try_from)
                    .collect::<std::result::Result<_, _>>()
                    .ok(),
                _ => None,
            })?,
        },
        DataType::Int32 => FieldData::ArrayInt32 {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::IntData(values) => Some(values.data),
                _ => None,
            })?,
        },
        DataType::Int64 => FieldData::ArrayInt64 {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::LongData(values) => Some(values.data),
                _ => None,
            })?,
        },
        DataType::Float => FieldData::ArrayFloat {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::FloatData(values) => Some(values.data),
                _ => None,
            })?,
        },
        DataType::Double => FieldData::ArrayDouble {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::DoubleData(values) => Some(values.data),
                _ => None,
            })?,
        },
        DataType::VarChar => FieldData::ArrayVarChar {
            name,
            values: decode_rows(rows, |row| match row.data? {
                scalar_field::Data::StringData(values) => Some(values.data),
                _ => None,
            })?,
        },
        _ => return None,
    })
}

fn decode_u16_vectors(bytes: &[u8], dimension: usize) -> Option<Vec<Vec<u16>>> {
    let row_bytes = dimension.checked_mul(2)?;
    if row_bytes == 0 || bytes.len() % row_bytes != 0 {
        return None;
    }
    Some(
        bytes
            .chunks(row_bytes)
            .map(|row| {
                row.chunks_exact(2)
                    .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                    .collect()
            })
            .collect(),
    )
}

fn compact_field_data(data: FieldData, valid_data: &[bool]) -> Option<FieldData> {
    fn compact<T>(values: Vec<T>, valid_data: &[bool]) -> Option<Vec<T>> {
        (values.len() == valid_data.len()).then(|| {
            values
                .into_iter()
                .zip(valid_data)
                .filter_map(|(value, valid)| valid.then_some(value))
                .collect()
        })
    }

    Some(match data {
        FieldData::Bool { name, values } => FieldData::Bool {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Int8 { name, values } => FieldData::Int8 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Int16 { name, values } => FieldData::Int16 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Int32 { name, values } => FieldData::Int32 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Int64 { name, values } => FieldData::Int64 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Float { name, values } => FieldData::Float {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Double { name, values } => FieldData::Double {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::VarChar { name, values } => FieldData::VarChar {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Json { name, values } => FieldData::Json {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Geometry { name, values } => FieldData::Geometry {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Timestamptz { name, values } => FieldData::Timestamptz {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayBool { name, values } => FieldData::ArrayBool {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayInt8 { name, values } => FieldData::ArrayInt8 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayInt16 { name, values } => FieldData::ArrayInt16 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayInt32 { name, values } => FieldData::ArrayInt32 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayInt64 { name, values } => FieldData::ArrayInt64 {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayFloat { name, values } => FieldData::ArrayFloat {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayDouble { name, values } => FieldData::ArrayDouble {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::ArrayVarChar { name, values } => FieldData::ArrayVarChar {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Struct { name, values } => FieldData::Struct {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::FloatVector { name, values } => FieldData::FloatVector {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::BinaryVector { name, values } => FieldData::BinaryVector {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Float16Vector { name, values } => FieldData::Float16Vector {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::BFloat16Vector { name, values } => FieldData::BFloat16Vector {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::SparseFloatVector { name, values } => FieldData::SparseFloatVector {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Int8Vector { name, values } => FieldData::Int8Vector {
            name,
            values: compact(values, valid_data)?,
        },
        FieldData::Nullable { .. } => return None,
    })
}

fn decode_sparse_vector(bytes: Vec<u8>) -> Option<SparseVector> {
    if bytes.len() % 8 != 0 {
        return None;
    }
    bytes
        .chunks_exact(8)
        .map(|item| {
            let index = u32::from_le_bytes(item[..4].try_into().ok()?);
            let value = f32::from_le_bytes(item[4..].try_into().ok()?);
            value.is_finite().then_some((index, value))
        })
        .collect()
}

fn struct_field_data(name: String, value: schema::StructArrayField) -> Option<FieldData> {
    use schema::{field_data as proto_field_data, scalar_field, vector_field};

    let mut parent_valid_data: Option<Vec<bool>> = None;
    for field in &value.fields {
        if !field.valid_data.is_empty() {
            parent_valid_data = Some(match parent_valid_data.take() {
                Some(valid_data) => {
                    if valid_data.len() != field.valid_data.len() {
                        return None;
                    }
                    valid_data
                        .iter()
                        .zip(&field.valid_data)
                        .map(|(parent, sub)| *parent && *sub)
                        .collect()
                }
                None => field.valid_data.clone(),
            });
        }
    }
    let parent_valid_at = |index: usize| {
        parent_valid_data
            .as_ref()
            .and_then(|valid_data| valid_data.get(index).copied())
    };
    let mut rows: Vec<Vec<crate::v2::types::StructValue>> = Vec::new();
    for field in value.fields {
        let field_name = field.field_name.clone();
        let grouped_values = match field.field? {
            proto_field_data::Field::Scalars(scalars) => {
                let scalar_field::Data::ArrayData(array) = scalars.data? else {
                    return None;
                };
                let field_name = field_name.clone();
                let element_type = array.element_type;
                let field_id = field.field_id;
                decode_struct_subfield_rows(
                    array.data,
                    parent_valid_data.as_deref(),
                    &parent_valid_at,
                    move |scalars| {
                        field_data_to_json_values(decode_field_data(schema::FieldData {
                            r#type: element_type,
                            field_name: field_name.clone(),
                            field_id,
                            is_dynamic: false,
                            valid_data: Vec::new(),
                            field: Some(proto_field_data::Field::Scalars(scalars)),
                            ..Default::default()
                        })?)
                    },
                )?
            }
            proto_field_data::Field::Vectors(vectors) => {
                let vector_field::Data::VectorArray(array) = vectors.data? else {
                    return None;
                };
                let field_name = field_name.clone();
                let element_type = array.element_type;
                let field_id = field.field_id;
                decode_struct_subfield_rows(
                    array.data,
                    parent_valid_data.as_deref(),
                    &parent_valid_at,
                    move |vectors| {
                        field_data_to_json_values(decode_field_data(schema::FieldData {
                            r#type: element_type,
                            field_name: field_name.clone(),
                            field_id,
                            is_dynamic: false,
                            valid_data: Vec::new(),
                            field: Some(proto_field_data::Field::Vectors(vectors)),
                            ..Default::default()
                        })?)
                    },
                )?
            }
            proto_field_data::Field::StructArrays(_) => return None,
        };

        if rows.is_empty() {
            rows = grouped_values
                .iter()
                .map(|values| {
                    (0..values.len())
                        .map(|_| crate::v2::types::StructValue::new())
                        .collect()
                })
                .collect();
        }
        if rows.len() != grouped_values.len()
            || rows
                .iter()
                .zip(&grouped_values)
                .any(|(row, values)| row.len() != values.len())
        {
            return None;
        }
        for (row, values) in rows.iter_mut().zip(grouped_values) {
            for (item, value) in row.iter_mut().zip(values) {
                item.insert(field_name.clone(), value);
            }
        }
    }
    let Some(valid_data) = parent_valid_data else {
        return Some(FieldData::Struct { name, values: rows });
    };
    let values = rows
        .into_iter()
        .zip(&valid_data)
        .filter_map(|(row, valid)| (*valid).then_some(row))
        .collect();
    let data = FieldData::Struct { name, values };
    FieldData::nullable(data, valid_data).ok()
}

/// Aligns a decoded struct sub-field column to the logical struct rows.
///
/// Servers can return a sub-field either with a full-length payload (one entry
/// per logical row, null rows carrying an empty placeholder) or a compacted
/// payload (only the non-null rows). When a validity mask is present the
/// compacted payload is re-expanded so every logical row maps back to its
/// original position, mirroring the Java SDK's `alignColumnData`.
fn align_struct_subfield(
    grouped: Vec<Vec<serde_json::Value>>,
    valid_data: Option<&[bool]>,
) -> Option<Vec<Vec<serde_json::Value>>> {
    let Some(valid_data) = valid_data else {
        return Some(grouped);
    };
    if grouped.len() == valid_data.len() {
        // Full-length payload: null rows are empty placeholders already.
        return Some(grouped);
    }
    let valid_count = valid_data.iter().filter(|valid| **valid).count();
    if grouped.len() != valid_count {
        return None;
    }
    // Compacted payload: re-expand to the full logical row count.
    let mut aligned = Vec::with_capacity(valid_data.len());
    let mut grouped = grouped.into_iter();
    for valid in valid_data {
        if *valid {
            aligned.push(grouped.next()?);
        } else {
            aligned.push(Vec::new());
        }
    }
    Some(aligned)
}

/// Decodes one struct sub-field's payload rows and aligns them back to the
/// logical struct rows. Scalar (`ArrayData`) and vector (`VectorArray`)
/// sub-fields share the same full-length vs compacted handling; only the
/// per-row decoding differs, which is supplied through `decode_row`.
fn decode_struct_subfield_rows<T>(
    rows: Vec<T>,
    valid_data: Option<&[bool]>,
    parent_valid_at: &dyn Fn(usize) -> Option<bool>,
    decode_row: impl Fn(T) -> Option<Vec<serde_json::Value>>,
) -> Option<Vec<Vec<serde_json::Value>>> {
    let full_length = valid_data.is_none_or(|valid_data| rows.len() == valid_data.len());
    let grouped = if full_length {
        rows.into_iter()
            .enumerate()
            .map(|(index, row)| {
                if parent_valid_at(index) == Some(false) {
                    return Some(Vec::new());
                }
                decode_row(row)
            })
            .collect::<Option<Vec<_>>>()?
    } else {
        rows.into_iter()
            .map(decode_row)
            .collect::<Option<Vec<_>>>()?
    };
    align_struct_subfield(grouped, valid_data)
}

fn field_data_to_json_values(value: FieldData) -> Option<Vec<serde_json::Value>> {
    use serde_json::{json, Value};

    Some(match value {
        FieldData::Bool { values, .. } => values.into_iter().map(Value::Bool).collect(),
        FieldData::Int8 { values, .. } => values.into_iter().map(|value| json!(value)).collect(),
        FieldData::Int16 { values, .. } => values.into_iter().map(|value| json!(value)).collect(),
        FieldData::Int32 { values, .. } => values.into_iter().map(|value| json!(value)).collect(),
        FieldData::Int64 { values, .. } => values.into_iter().map(|value| json!(value)).collect(),
        FieldData::Float { values, .. } => values.into_iter().map(|value| json!(value)).collect(),
        FieldData::Double { values, .. } => values.into_iter().map(|value| json!(value)).collect(),
        FieldData::VarChar { values, .. }
        | FieldData::Geometry { values, .. }
        | FieldData::Timestamptz { values, .. } => values.into_iter().map(Value::String).collect(),
        FieldData::Json { values, .. } => values,
        FieldData::ArrayBool { values, .. } => array_values_to_json(values),
        FieldData::ArrayInt8 { values, .. } => array_values_to_json(values),
        FieldData::ArrayInt16 { values, .. } => array_values_to_json(values),
        FieldData::ArrayInt32 { values, .. } => array_values_to_json(values),
        FieldData::ArrayInt64 { values, .. } => array_values_to_json(values),
        FieldData::ArrayFloat { values, .. } => array_values_to_json(values),
        FieldData::ArrayDouble { values, .. } => array_values_to_json(values),
        FieldData::ArrayVarChar { values, .. } => array_values_to_json(values),
        FieldData::FloatVector { values, .. } => values
            .into_iter()
            .map(|value| Value::Array(value.into_iter().map(|item| json!(item)).collect()))
            .collect(),
        FieldData::BinaryVector { values, .. } => values
            .into_iter()
            .map(|value| Value::Array(value.into_iter().map(|item| json!(item)).collect()))
            .collect(),
        FieldData::Float16Vector { values, .. } | FieldData::BFloat16Vector { values, .. } => {
            values
                .into_iter()
                .map(|value| Value::Array(value.into_iter().map(|item| json!(item)).collect()))
                .collect()
        }
        FieldData::Int8Vector { values, .. } => values
            .into_iter()
            .map(|value| Value::Array(value.into_iter().map(|item| json!(item)).collect()))
            .collect(),
        FieldData::SparseFloatVector { values, .. } => values
            .into_iter()
            .map(|value| {
                Value::Object(
                    value
                        .into_iter()
                        .map(|(index, value)| (index.to_string(), json!(value)))
                        .collect(),
                )
            })
            .collect(),
        FieldData::Nullable { data, valid_data } => {
            let mut values = field_data_to_json_values(*data)?.into_iter();
            valid_data
                .into_iter()
                .map(|valid| {
                    if valid {
                        values.next().unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    }
                })
                .collect()
        }
        FieldData::Struct { .. } => return None,
    })
}

fn array_values_to_json<T: serde::Serialize>(values: Vec<Vec<T>>) -> Vec<serde_json::Value> {
    values
        .into_iter()
        .map(|values| serde_json::to_value(values).expect("primitive arrays serialize to JSON"))
        .collect()
}

///////////////////////////////////////////////////////////////////////////////
// QueryResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 query operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct QueryResponse {
    pub(crate) results: QueryResults,
    pub(crate) session_timestamp: u64,
}

impl QueryResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            results: QueryResults::new(),
            session_timestamp: 0,
        }
    }
}

impl QueryResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> QueryResponseBuilder {
        QueryResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the results.
    pub fn results(&self) -> &QueryResults {
        &self.results
    }

    /// Returns the session timestamp.
    pub fn session_timestamp(&self) -> u64 {
        self.session_timestamp
    }

    pub(crate) fn from_proto(value: milvus::QueryResults) -> Result<Self> {
        let output_fields = value
            .fields_data
            .into_iter()
            .map(field_data)
            .collect::<Result<Vec<_>>>()?;
        let row_count = output_fields.first().map_or(0, FieldData::len);
        let element_indices = value
            .element_indices
            .into_iter()
            .map(|indices| {
                indices
                    .indices
                    .map(|indices| indices.data)
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        if !element_indices.is_empty() && element_indices.len() != row_count {
            return Err(Error::MalformedResponse(format!(
                "query response element_indices length {} does not match row count {row_count}",
                element_indices.len()
            )));
        }
        Ok(Self {
            results: QueryResults {
                output_fields,
                output_field_names: value.output_fields,
                element_indices,
            },
            session_timestamp: value.session_ts,
        })
    }

    pub(crate) fn split_at(self, count: usize) -> Result<(Self, Option<Self>)> {
        let row_count = usize::try_from(self.results.get_row_count()).map_err(|_| {
            Error::MalformedResponse("query response row count does not fit usize".into())
        })?;
        if count >= row_count {
            return Ok((self, None));
        }

        let sizes = [count, row_count - count];
        let mut first_fields = Vec::with_capacity(self.results.output_fields.len());
        let mut remaining_fields = Vec::with_capacity(self.results.output_fields.len());
        for field in self.results.output_fields {
            let mut fields = split_field_data(field, &sizes).ok_or_else(|| {
                Error::MalformedResponse(
                    "failed to split query response into iterator batches".into(),
                )
            })?;
            remaining_fields.push(fields.pop().expect("split produces the remaining field"));
            first_fields.push(fields.pop().expect("split produces the first field"));
        }

        let mut element_indices = self.results.element_indices;
        let remaining_indices = if element_indices.is_empty() {
            Vec::new()
        } else {
            element_indices.split_off(count)
        };
        let output_field_names = self.results.output_field_names;
        let session_timestamp = self.session_timestamp;
        Ok((
            Self {
                results: QueryResults {
                    output_fields: first_fields,
                    output_field_names: output_field_names.clone(),
                    element_indices,
                },
                session_timestamp,
            },
            Some(Self {
                results: QueryResults {
                    output_fields: remaining_fields,
                    output_field_names,
                    element_indices: remaining_indices,
                },
                session_timestamp,
            }),
        ))
    }
}

///////////////////////////////////////////////////////////////////////////////
// QueryResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for QueryResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct QueryResponseBuilder {
    value: QueryResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl QueryResponseBuilder {
    /// Sets the results and returns the updated value.
    pub fn results(mut self, value: QueryResults) -> Self {
        self.value.results = value;
        self
    }

    /// Sets the session timestamp and returns the updated value.
    pub fn session_timestamp(mut self, value: u64) -> Self {
        self.value.session_timestamp = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> QueryResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 search operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SearchResponse {
    pub(crate) results: SearchResults,
    pub(crate) session_timestamp: u64,
    pub(crate) cost: i64,
    pub(crate) scanned_remote_bytes: i64,
    pub(crate) scanned_total_bytes: i64,
    pub(crate) cache_hit_ratio: f32,
}

///////////////////////////////////////////////////////////////////////////////
// SearchResponse
///////////////////////////////////////////////////////////////////////////////
impl SearchResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> SearchResponseBuilder {
        SearchResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the results.
    pub fn results(&self) -> &SearchResults {
        &self.results
    }

    /// Returns the session timestamp.
    pub fn session_timestamp(&self) -> u64 {
        self.session_timestamp
    }

    /// Returns the cost.
    pub fn cost(&self) -> i64 {
        self.cost
    }

    /// Returns the scanned remote bytes.
    pub fn scanned_remote_bytes(&self) -> i64 {
        self.scanned_remote_bytes
    }

    /// Returns the scanned total bytes.
    pub fn scanned_total_bytes(&self) -> i64 {
        self.scanned_total_bytes
    }

    /// Returns the cache hit ratio.
    pub fn cache_hit_ratio(&self) -> f32 {
        self.cache_hit_ratio
    }

    pub(crate) fn from_proto(value: milvus::SearchResults) -> Result<Self> {
        Self::from_proto_with_row_limit(value, None)
    }

    pub(crate) fn from_proto_with_row_limit(
        value: milvus::SearchResults,
        row_limit: Option<usize>,
    ) -> Result<Self> {
        let extra_info = value
            .status
            .as_ref()
            .map(|status| &status.extra_info)
            .cloned()
            .unwrap_or_default();
        let result = value
            .results
            .ok_or_else(|| Error::MalformedResponse("no result for search".into()))?;
        let field_names = result
            .fields_data
            .iter()
            .map(|field| field.field_name.as_str())
            .collect::<HashSet<_>>();
        let mut score_field_name = "score".to_owned();
        while field_names.contains(score_field_name.as_str()) {
            score_field_name.insert(0, '_');
        }
        let query_count = usize::try_from(result.num_queries).map_err(|_| {
            Error::MalformedResponse(format!(
                "search response contains invalid query count {}",
                result.num_queries
            ))
        })?;
        if result.topks.len() < query_count {
            return Err(Error::MalformedResponse(format!(
                "search response contains {} top-k values for {query_count} queries",
                result.topks.len()
            )));
        }
        let mut row_counts = result
            .topks
            .iter()
            .take(query_count)
            .map(|top_k| {
                usize::try_from(*top_k).map_err(|_| {
                    Error::MalformedResponse(format!(
                        "search response contains invalid top-k value {top_k}"
                    ))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if let (Some(limit), [row_count]) = (row_limit, row_counts.as_mut_slice()) {
            *row_count = (*row_count).min(limit);
        }
        let total_rows = row_counts.iter().try_fold(0_usize, |total, count| {
            total.checked_add(*count).ok_or_else(|| {
                Error::MalformedResponse("search response row count overflowed usize".into())
            })
        })?;
        let mut highlight_results = vec![HashMap::new(); total_rows];
        for highlight in result.highlight_results {
            for (index, data) in highlight.datas.into_iter().take(total_rows).enumerate() {
                highlight_results[index].insert(
                    highlight.field_name.clone(),
                    HighlightResult {
                        field_name: highlight.field_name.clone(),
                        fragments: data.fragments,
                        scores: data.scores,
                    },
                );
            }
        }
        let field_count = result.fields_data.len();
        let mut fields_by_query = (0..query_count)
            .map(|_| Vec::with_capacity(field_count))
            .collect::<Vec<_>>();
        if total_rows > 0 {
            for mut field in result.fields_data {
                if query_count == 1 {
                    field = truncate_proto_field_data(field, total_rows)?;
                }
                let fields =
                    split_field_data(field_data(field)?, &row_counts).ok_or_else(|| {
                        Error::MalformedResponse(
                            "search response output fields do not match its top-k values".into(),
                        )
                    })?;
                for (query_fields, field) in fields_by_query.iter_mut().zip(fields) {
                    query_fields.push(field);
                }
            }
        }
        let ids = Ids::from_proto(result.ids)?;
        let element_indices = result.element_indices.map(|indices| indices.data);
        let mut single_results = Vec::with_capacity(query_count);
        let mut offset = 0_usize;
        for (row_count, output_fields) in row_counts.into_iter().zip(fields_by_query) {
            let scores = slice_values(&result.scores, offset, row_count).ok_or_else(|| {
                Error::MalformedResponse(
                    "search response scores do not match its top-k values".into(),
                )
            })?;
            let ids = slice_ids(&ids, offset, row_count).ok_or_else(|| {
                Error::MalformedResponse("search response IDs do not match its top-k values".into())
            })?;
            let element_indices = element_indices
                .as_ref()
                .map(|indices| {
                    slice_values(indices, offset, row_count).ok_or_else(|| {
                        Error::MalformedResponse(
                            "search response element indices do not match its top-k values".into(),
                        )
                    })
                })
                .transpose()?;
            let highlights =
                slice_values(&highlight_results, offset, row_count).ok_or_else(|| {
                    Error::MalformedResponse(
                        "search response highlights do not match its top-k values".into(),
                    )
                })?;
            single_results.push(SingleResult {
                ids,
                scores,
                element_indices,
                output_fields,
                output_field_names: result.output_fields.clone(),
                primary_field_name: result.primary_field_name.clone(),
                score_field_name: score_field_name.clone(),
                highlight_results: highlights,
            });
            offset = offset.checked_add(row_count).ok_or_else(|| {
                Error::MalformedResponse("search response row count overflowed usize".into())
            })?;
        }
        Ok(Self {
            session_timestamp: value.session_ts,
            cost: parse_extra(&extra_info, "report_value", -1_i64),
            scanned_remote_bytes: parse_extra(&extra_info, "scanned_remote_bytes", -1_i64),
            scanned_total_bytes: parse_extra(&extra_info, "scanned_total_bytes", -1_i64),
            cache_hit_ratio: parse_extra(&extra_info, "cache_hit_ratio", -1.0_f32),
            results: SearchResults {
                results: single_results,
                recalls: result.recalls,
                agg_buckets: group_aggregation_buckets(
                    result.agg_buckets,
                    result.agg_topks,
                    result.num_queries,
                )?,
            },
        })
    }

    pub(crate) fn row_count(&self) -> Result<usize> {
        match self.results.results.as_slice() {
            [result] => Ok(result.len()),
            results => Err(Error::MalformedResponse(format!(
                "search iterator response must contain exactly one result, got {}",
                results.len()
            ))),
        }
    }

    pub(crate) fn split_at(mut self, count: usize) -> Result<(Self, Option<Self>)> {
        let row_count = self.row_count()?;
        if count >= row_count {
            return Ok((self, None));
        }
        let result = self
            .results
            .results
            .pop()
            .expect("validated one search iterator result");
        let (first, remaining) = split_single_result(result, count)?;
        let mut remaining_response = Self {
            results: SearchResults {
                results: vec![remaining],
                recalls: self.results.recalls.clone(),
                agg_buckets: self.results.agg_buckets.clone(),
            },
            session_timestamp: self.session_timestamp,
            cost: self.cost,
            scanned_remote_bytes: self.scanned_remote_bytes,
            scanned_total_bytes: self.scanned_total_bytes,
            cache_hit_ratio: self.cache_hit_ratio,
        };
        self.results.results.push(first);
        std::mem::swap(
            &mut remaining_response.results.recalls,
            &mut self.results.recalls,
        );
        Ok((self, Some(remaining_response)))
    }

    pub(crate) fn append(&mut self, mut other: Self) -> Result<()> {
        let [left] = self.results.results.as_mut_slice() else {
            return Err(Error::MalformedResponse(
                "search iterator cache must contain exactly one result".into(),
            ));
        };
        let [right] = other.results.results.as_mut_slice() else {
            return Err(Error::MalformedResponse(
                "search iterator page must contain exactly one result".into(),
            ));
        };
        if left.output_field_names != right.output_field_names
            || left.primary_field_name != right.primary_field_name
            || left.score_field_name != right.score_field_name
            || left.output_fields.len() != right.output_fields.len()
            || !left.ids.is_compatible_with(&right.ids)
            || left
                .output_fields
                .iter()
                .zip(&right.output_fields)
                .any(|(left, right)| !left.is_compatible_with(right))
        {
            return Err(Error::MalformedResponse(
                "search iterator pages contain incompatible result schemas".into(),
            ));
        }
        left.ids.append(std::mem::take(&mut right.ids))?;
        left.scores.append(&mut right.scores);
        match (&mut left.element_indices, right.element_indices.take()) {
            (Some(left), Some(mut right)) => left.append(&mut right),
            (None, None) => {}
            _ => {
                return Err(Error::MalformedResponse(
                    "search iterator pages disagree about element indices".into(),
                ));
            }
        }
        for (left, right) in left
            .output_fields
            .iter_mut()
            .zip(std::mem::take(&mut right.output_fields))
        {
            left.append(right)?;
        }
        left.highlight_results.append(&mut right.highlight_results);
        Ok(())
    }
}

fn split_single_result(value: SingleResult, count: usize) -> Result<(SingleResult, SingleResult)> {
    let row_count = value.len();
    let sizes = [count, row_count - count];
    let mut ids = split_ids(value.ids, &sizes).ok_or_else(|| {
        Error::MalformedResponse("failed to split search iterator primary keys".into())
    })?;
    let mut scores = split_values(value.scores, &sizes)
        .ok_or_else(|| Error::MalformedResponse("failed to split search iterator scores".into()))?;
    let mut element_indices = value
        .element_indices
        .map(|indices| {
            split_values(indices, &sizes).ok_or_else(|| {
                Error::MalformedResponse("failed to split search iterator element indices".into())
            })
        })
        .transpose()?;
    let mut highlights = split_values(value.highlight_results, &sizes).ok_or_else(|| {
        Error::MalformedResponse("failed to split search iterator highlights".into())
    })?;
    let mut first_fields = Vec::with_capacity(value.output_fields.len());
    let mut remaining_fields = Vec::with_capacity(value.output_fields.len());
    for field in value.output_fields {
        let mut fields = split_field_data(field, &sizes).ok_or_else(|| {
            Error::MalformedResponse("failed to split search iterator output fields".into())
        })?;
        remaining_fields.push(fields.pop().expect("split produces remaining field"));
        first_fields.push(fields.pop().expect("split produces first field"));
    }

    let remaining = SingleResult {
        ids: ids.pop().expect("split produces remaining IDs"),
        scores: scores.pop().expect("split produces remaining scores"),
        element_indices: element_indices.as_mut().map(|indices| {
            indices
                .pop()
                .expect("split produces remaining element indices")
        }),
        output_fields: remaining_fields,
        output_field_names: value.output_field_names.clone(),
        primary_field_name: value.primary_field_name.clone(),
        score_field_name: value.score_field_name.clone(),
        highlight_results: highlights
            .pop()
            .expect("split produces remaining highlights"),
    };
    let first = SingleResult {
        ids: ids.pop().expect("split produces first IDs"),
        scores: scores.pop().expect("split produces first scores"),
        element_indices: element_indices
            .as_mut()
            .map(|indices| indices.pop().expect("split produces first element indices")),
        output_fields: first_fields,
        output_field_names: value.output_field_names,
        primary_field_name: value.primary_field_name,
        score_field_name: value.score_field_name,
        highlight_results: highlights.pop().expect("split produces first highlights"),
    };
    Ok((first, remaining))
}

fn split_ids(value: Ids, sizes: &[usize]) -> Option<Vec<Ids>> {
    match value {
        Ids::Int64(values) => {
            split_values(values, sizes).map(|values| values.into_iter().map(Ids::Int64).collect())
        }
        Ids::VarChar(values) => {
            split_values(values, sizes).map(|values| values.into_iter().map(Ids::VarChar).collect())
        }
    }
}

fn truncate_proto_field_data(
    mut value: schema::FieldData,
    logical_count: usize,
) -> Result<schema::FieldData> {
    use schema::field_data::Field;
    let field_name = value.field_name.clone();
    let encoded_count = proto_field_row_count(&value).ok_or_else(|| {
        Error::MalformedResponse(format!(
            "failed to determine response field row count for {field_name:?}"
        ))
    })?;
    // Prefer the field-specific validity channel (newer servers write here), falling back to the
    // legacy top-level channel, exactly as `decode_field_data` does.
    let valid_data: Vec<bool> = match value.field {
        Some(Field::Scalars(ref scalars)) if !scalars.valid_data.is_empty() => {
            scalars.valid_data.clone()
        }
        Some(Field::Vectors(ref vectors)) if !vectors.valid_data.is_empty() => {
            vectors.valid_data.clone()
        }
        _ => value.valid_data.clone(),
    };
    let payload_count = if valid_data.is_empty() {
        logical_count
    } else {
        if valid_data.len() < logical_count {
            return Err(Error::MalformedResponse(format!(
                "response field {field_name:?} validity bitmap is shorter than its result range"
            )));
        }
        let total_valid = valid_data.iter().filter(|valid| **valid).count();
        let selected_valid = valid_data
            .iter()
            .take(logical_count)
            .filter(|valid| **valid)
            .count();
        if encoded_count == valid_data.len() {
            logical_count
        } else if encoded_count == total_valid {
            selected_valid
        } else if encoded_count == logical_count {
            logical_count
        } else if encoded_count == selected_valid {
            selected_valid
        } else {
            return Err(Error::MalformedResponse(format!(
                "response field {field_name:?} data does not match its validity bitmap"
            )));
        }
    };
    if encoded_count < payload_count {
        return Err(Error::MalformedResponse(format!(
            "response field {field_name:?} is shorter than its result range"
        )));
    }

    // Truncate both the field-specific and the legacy validity channels so the preferred bitmap
    // still matches the truncated payload; leaving the field-specific channel untruncated would
    // make `decode_field_data` see mismatched lengths and fail the whole decode.
    if let Some(Field::Scalars(scalars)) = value.field.as_mut() {
        if !scalars.valid_data.is_empty() {
            scalars.valid_data.truncate(logical_count);
        }
    } else if let Some(Field::Vectors(vectors)) = value.field.as_mut() {
        if !vectors.valid_data.is_empty() {
            vectors.valid_data.truncate(logical_count);
        }
    }
    value.valid_data.truncate(logical_count);
    truncate_proto_payload(&mut value, payload_count).ok_or_else(|| {
        Error::MalformedResponse(format!(
            "failed to slice response field {field_name:?} to its result range"
        ))
    })?;
    Ok(value)
}

fn proto_field_row_count(value: &schema::FieldData) -> Option<usize> {
    use schema::{field_data::Field, scalar_field::Data as Scalar, vector_field::Data as Vector};

    match value.field.as_ref()? {
        Field::Scalars(scalars) => Some(match scalars.data.as_ref()? {
            Scalar::BoolData(values) => values.data.len(),
            Scalar::IntData(values) => values.data.len(),
            Scalar::LongData(values) => values.data.len(),
            Scalar::FloatData(values) => values.data.len(),
            Scalar::DoubleData(values) => values.data.len(),
            Scalar::StringData(values) => values.data.len(),
            Scalar::BytesData(values) => values.data.len(),
            Scalar::ArrayData(values) => values.data.len(),
            Scalar::JsonData(values) => values.data.len(),
            Scalar::GeometryData(values) => values.data.len(),
            Scalar::TimestamptzData(values) => values.data.len(),
            Scalar::GeometryWktData(values) => values.data.len(),
            Scalar::MolData(values) => values.data.len(),
            Scalar::MolSmilesData(values) => values.data.len(),
            Scalar::DateData(values) => values.data.len(),
            Scalar::TimeData(values) => values.data.len(),
        }),
        Field::Vectors(vectors) => {
            let dimension = usize::try_from(vectors.dim).ok()?;
            let Some(data) = vectors.data.as_ref() else {
                return value.valid_data.iter().all(|valid| !*valid).then_some(0);
            };
            match data {
                Vector::FloatVector(values) if dimension > 0 => {
                    (values.data.len() % dimension == 0).then_some(values.data.len() / dimension)
                }
                Vector::BinaryVector(values) if dimension > 0 && dimension % 8 == 0 => {
                    let row_bytes = dimension / 8;
                    (values.len() % row_bytes == 0).then_some(values.len() / row_bytes)
                }
                Vector::Float16Vector(values) | Vector::Bfloat16Vector(values) if dimension > 0 => {
                    let row_bytes = dimension.checked_mul(2)?;
                    (values.len() % row_bytes == 0).then_some(values.len() / row_bytes)
                }
                Vector::SparseFloatVector(values) => Some(values.contents.len()),
                Vector::Int8Vector(values) if dimension > 0 => {
                    (values.len() % dimension == 0).then_some(values.len() / dimension)
                }
                Vector::VectorArray(values) => Some(values.data.len()),
                _ => None,
            }
        }
        Field::StructArrays(values) => values.fields.first().and_then(proto_field_row_count),
    }
}

fn truncate_proto_payload(value: &mut schema::FieldData, count: usize) -> Option<()> {
    use schema::{field_data::Field, scalar_field::Data as Scalar, vector_field::Data as Vector};

    match value.field.as_mut()? {
        Field::Scalars(scalars) => match scalars.data.as_mut()? {
            Scalar::BoolData(values) => values.data.truncate(count),
            Scalar::IntData(values) => values.data.truncate(count),
            Scalar::LongData(values) => values.data.truncate(count),
            Scalar::FloatData(values) => values.data.truncate(count),
            Scalar::DoubleData(values) => values.data.truncate(count),
            Scalar::StringData(values) => values.data.truncate(count),
            Scalar::BytesData(values) => values.data.truncate(count),
            Scalar::ArrayData(values) => values.data.truncate(count),
            Scalar::JsonData(values) => values.data.truncate(count),
            Scalar::GeometryData(values) => values.data.truncate(count),
            Scalar::TimestamptzData(values) => values.data.truncate(count),
            Scalar::GeometryWktData(values) => values.data.truncate(count),
            Scalar::MolData(values) => values.data.truncate(count),
            Scalar::MolSmilesData(values) => values.data.truncate(count),
            Scalar::DateData(values) => values.data.truncate(count),
            Scalar::TimeData(values) => values.data.truncate(count),
        },
        Field::Vectors(vectors) => {
            let dimension = usize::try_from(vectors.dim).ok()?;
            let Some(data) = vectors.data.as_mut() else {
                return (count == 0).then_some(());
            };
            match data {
                Vector::FloatVector(values) => values.data.truncate(count.checked_mul(dimension)?),
                Vector::BinaryVector(values) => {
                    values.truncate(count.checked_mul(dimension.checked_div(8)?)?)
                }
                Vector::Float16Vector(values) | Vector::Bfloat16Vector(values) => {
                    values.truncate(count.checked_mul(dimension)?.checked_mul(2)?)
                }
                Vector::SparseFloatVector(values) => values.contents.truncate(count),
                Vector::Int8Vector(values) => values.truncate(count.checked_mul(dimension)?),
                Vector::VectorArray(values) => values.data.truncate(count),
            }
        }
        Field::StructArrays(values) => {
            for field in &mut values.fields {
                let sliced = truncate_proto_field_data(std::mem::take(field), count).ok()?;
                *field = sliced;
            }
        }
    }
    Some(())
}

fn slice_values<T: Clone>(values: &[T], offset: usize, size: usize) -> Option<Vec<T>> {
    let end = offset.checked_add(size)?;
    Some(values.get(offset..end)?.to_vec())
}

fn slice_ids(ids: &Ids, offset: usize, size: usize) -> Option<Ids> {
    match ids {
        Ids::Int64(values) => slice_values(values, offset, size).map(Ids::Int64),
        Ids::VarChar(values) => slice_values(values, offset, size).map(Ids::VarChar),
    }
}

fn split_values<T>(values: Vec<T>, sizes: &[usize]) -> Option<Vec<Vec<T>>> {
    let required = sizes
        .iter()
        .try_fold(0_usize, |total, size| total.checked_add(*size))?;
    if values.len() != required {
        return None;
    }
    let mut values = values.into_iter();
    Some(
        sizes
            .iter()
            .map(|size| values.by_ref().take(*size).collect())
            .collect(),
    )
}

fn split_named_values<T>(
    name: String,
    values: Vec<T>,
    sizes: &[usize],
    make: impl Fn(String, Vec<T>) -> FieldData,
) -> Option<Vec<FieldData>> {
    Some(
        split_values(values, sizes)?
            .into_iter()
            .map(|values| make(name.clone(), values))
            .collect(),
    )
}

fn split_field_data(data: FieldData, sizes: &[usize]) -> Option<Vec<FieldData>> {
    match data {
        FieldData::Bool { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Bool {
                name,
                values,
            })
        }
        FieldData::Int8 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Int8 {
                name,
                values,
            })
        }
        FieldData::Int16 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Int16 {
                name,
                values,
            })
        }
        FieldData::Int32 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Int32 {
                name,
                values,
            })
        }
        FieldData::Int64 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Int64 {
                name,
                values,
            })
        }
        FieldData::Float { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Float {
                name,
                values,
            })
        }
        FieldData::Double { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Double {
                name,
                values,
            })
        }
        FieldData::VarChar { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::VarChar {
                name,
                values,
            })
        }
        FieldData::Json { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Json {
                name,
                values,
            })
        }
        FieldData::Geometry { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Geometry {
                name,
                values,
            })
        }
        FieldData::Timestamptz { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Timestamptz {
                name,
                values,
            })
        }
        FieldData::ArrayBool { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayBool {
                name,
                values,
            })
        }
        FieldData::ArrayInt8 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayInt8 {
                name,
                values,
            })
        }
        FieldData::ArrayInt16 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayInt16 {
                name,
                values,
            })
        }
        FieldData::ArrayInt32 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayInt32 {
                name,
                values,
            })
        }
        FieldData::ArrayInt64 { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayInt64 {
                name,
                values,
            })
        }
        FieldData::ArrayFloat { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayFloat {
                name,
                values,
            })
        }
        FieldData::ArrayDouble { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::ArrayDouble {
                name,
                values,
            })
        }
        FieldData::ArrayVarChar { name, values } => {
            split_named_values(name, values, sizes, |name, values| {
                FieldData::ArrayVarChar { name, values }
            })
        }
        FieldData::Struct { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Struct {
                name,
                values,
            })
        }
        FieldData::FloatVector { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::FloatVector {
                name,
                values,
            })
        }
        FieldData::BinaryVector { name, values } => {
            split_named_values(name, values, sizes, |name, values| {
                FieldData::BinaryVector { name, values }
            })
        }
        FieldData::Float16Vector { name, values } => {
            split_named_values(name, values, sizes, |name, values| {
                FieldData::Float16Vector { name, values }
            })
        }
        FieldData::BFloat16Vector { name, values } => {
            split_named_values(name, values, sizes, |name, values| {
                FieldData::BFloat16Vector { name, values }
            })
        }
        FieldData::SparseFloatVector { name, values } => {
            split_named_values(name, values, sizes, |name, values| {
                FieldData::SparseFloatVector { name, values }
            })
        }
        FieldData::Int8Vector { name, values } => {
            split_named_values(name, values, sizes, |name, values| FieldData::Int8Vector {
                name,
                values,
            })
        }
        FieldData::Nullable { data, valid_data } => {
            let valid_data = split_values(valid_data, sizes)?;
            let inner_sizes = valid_data
                .iter()
                .map(|values| values.iter().filter(|valid| **valid).count())
                .collect::<Vec<_>>();
            let data = split_field_data(*data, &inner_sizes)?;
            data.into_iter()
                .zip(valid_data)
                .map(|(data, valid_data)| FieldData::nullable(data, valid_data).ok())
                .collect()
        }
    }
}

fn parse_extra<T>(values: &HashMap<String, String>, key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    values
        .get(key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

///////////////////////////////////////////////////////////////////////////////
// SearchResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for SearchResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct SearchResponseBuilder {
    value: SearchResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl SearchResponseBuilder {
    /// Sets the results and returns the updated value.
    pub fn results(mut self, value: SearchResults) -> Self {
        self.value.results = value;
        self
    }

    /// Sets the session timestamp and returns the updated value.
    pub fn session_timestamp(mut self, value: u64) -> Self {
        self.value.session_timestamp = value;
        self
    }

    /// Sets the cost and returns the updated value.
    pub fn cost(mut self, value: i64) -> Self {
        self.value.cost = value;
        self
    }

    /// Sets the scanned remote bytes and returns the updated value.
    pub fn scanned_remote_bytes(mut self, value: i64) -> Self {
        self.value.scanned_remote_bytes = value;
        self
    }

    /// Sets the scanned total bytes and returns the updated value.
    pub fn scanned_total_bytes(mut self, value: i64) -> Self {
        self.value.scanned_total_bytes = value;
        self
    }

    /// Sets the cache hit ratio and returns the updated value.
    pub fn cache_hit_ratio(mut self, value: f32) -> Self {
        self.value.cache_hit_ratio = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> SearchResponse {
        self.value
    }
}

impl SearchResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            results: SearchResults::new(),
            session_timestamp: 0,
            cost: -1,
            scanned_remote_bytes: -1,
            scanned_total_bytes: -1,
            cache_hit_ratio: -1.0,
        }
    }
}

/// Response returned by the ClientV2 get operation.
pub type GetResponse = QueryResponse;
/// Response returned by the ClientV2 hybrid_search operation.
pub type HybridSearchResponse = SearchResponse;

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{field_data, split_field_data, QueryResponse, SearchResponse};
    use crate::proto::{common, milvus, schema};
    use crate::v2::types::{AggregationBucketValue, DataType, FieldData};

    #[test]
    fn decode_field_data_prefers_field_specific_validity() {
        // Server sends the new field-specific validity that contradicts the legacy
        // top-level field; the field-specific channel must win.
        let proto = schema::FieldData {
            r#type: schema::DataType::Int64 as i32,
            field_name: "id".into(),
            valid_data: vec![false, false, false],
            field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![true, false, true],
                data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                    data: vec![1, 2, 3],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let decoded = super::decode_field_data(proto).expect("decodes");
        let FieldData::Nullable { data, valid_data } = decoded else {
            panic!("expected nullable field data")
        };
        assert_eq!(valid_data, vec![true, false, true]);
        assert_eq!(data.as_int64(), Some([1, 3].as_slice()));
    }

    #[test]
    fn decode_field_data_falls_back_to_legacy_validity() {
        // Legacy writer: only the top-level field carries validity.
        let proto = schema::FieldData {
            r#type: schema::DataType::Int64 as i32,
            field_name: "id".into(),
            valid_data: vec![true, false, true],
            field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                    data: vec![1, 2, 3],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let decoded = super::decode_field_data(proto).expect("decodes");
        let FieldData::Nullable { data, valid_data } = decoded else {
            panic!("expected nullable field data")
        };
        assert_eq!(valid_data, vec![true, false, true]);
        assert_eq!(data.as_int64(), Some([1, 3].as_slice()));
    }

    fn all_null_vector_field(data_type: schema::DataType, dimension: i64) -> schema::FieldData {
        schema::FieldData {
            r#type: data_type as i32,
            field_name: "embedding".into(),
            valid_data: vec![false, false],
            field: Some(schema::field_data::Field::Vectors(schema::VectorField {
                valid_data: Vec::new(),
                dim: dimension,
                data: None,
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    #[test]
    fn all_null_vector_pages_decode_empty_typed_payloads() {
        for (proto_type, sdk_type, dimension) in [
            (schema::DataType::FloatVector, DataType::FloatVector, 2),
            (schema::DataType::BinaryVector, DataType::BinaryVector, 8),
            (schema::DataType::Float16Vector, DataType::Float16Vector, 2),
            (
                schema::DataType::BFloat16Vector,
                DataType::BFloat16Vector,
                2,
            ),
            (
                schema::DataType::SparseFloatVector,
                DataType::SparseFloatVector,
                0,
            ),
            (schema::DataType::Int8Vector, DataType::Int8Vector, 2),
        ] {
            let decoded = field_data(all_null_vector_field(proto_type, dimension)).unwrap();
            assert_eq!(decoded.data_type(), sdk_type);
            assert_eq!(decoded.valid_data(), Some([false, false].as_slice()));
            assert_eq!(decoded.len(), 2);
            assert!(decoded.inner().is_empty());
        }

        let mut malformed = all_null_vector_field(schema::DataType::FloatVector, 2);
        malformed.valid_data = vec![true, false];
        assert!(field_data(malformed).is_err());
    }

    #[test]
    fn query_and_search_decode_all_null_vector_pages() {
        let field = all_null_vector_field(schema::DataType::FloatVector, 2);
        let query = QueryResponse::from_proto(milvus::QueryResults {
            fields_data: vec![field.clone()],
            output_fields: vec!["embedding".into()],
            ..Default::default()
        })
        .unwrap();
        let query_field = query.results().get_output_field("embedding").unwrap();
        assert_eq!(query_field.valid_data(), Some([false, false].as_slice()));
        assert!(query_field.inner().is_empty());

        let search = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 1,
                top_k: 2,
                topks: vec![2],
                scores: vec![0.9, 0.8],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![1, 2],
                    })),
                    ..Default::default()
                }),
                fields_data: vec![field],
                output_fields: vec!["embedding".into()],
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
        let search_field = search.results().get_results()[0]
            .get_output_field("embedding")
            .unwrap();
        assert_eq!(search_field.valid_data(), Some([false, false].as_slice()));
        assert!(search_field.inner().is_empty());
    }

    #[test]
    fn nullable_array_and_sparse_fields_decode_without_proto_exposure() {
        use schema::{field_data::Field, scalar_field, vector_field};

        let nullable = field_data(schema::FieldData {
            r#type: schema::DataType::Int64 as i32,
            field_name: "optional".into(),
            valid_data: vec![true, false, true],
            field: Some(Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::LongData(schema::LongArray {
                    data: vec![10, 30],
                })),
                ..Default::default()
            })),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(nullable.valid_data(), Some([true, false, true].as_slice()));
        assert!(nullable.is_null(1));
        assert!(
            matches!(nullable.inner(), FieldData::Int64 { values, .. } if values == &vec![10, 30])
        );

        let array = field_data(schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "tags".into(),
            field: Some(Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::VarChar as i32,
                    data: vec![schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(scalar_field::Data::StringData(schema::StringArray {
                            data: vec!["a".into(), "b".into()],
                        })),
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        })
        .unwrap();
        assert!(matches!(array, FieldData::ArrayVarChar { values, .. } if values.len() == 1));

        let mut sparse_bytes = Vec::new();
        sparse_bytes.extend(2_u32.to_le_bytes());
        sparse_bytes.extend(0.5_f32.to_le_bytes());
        let sparse = field_data(schema::FieldData {
            r#type: schema::DataType::SparseFloatVector as i32,
            field_name: "sparse".into(),
            field: Some(Field::Vectors(schema::VectorField {
                valid_data: Vec::new(),
                dim: 3,
                data: Some(vector_field::Data::SparseFloatVector(
                    schema::SparseFloatArray {
                        contents: vec![sparse_bytes],
                        dim: 3,
                    },
                )),
                ..Default::default()
            })),
            ..Default::default()
        })
        .unwrap();
        assert!(
            matches!(sparse, FieldData::SparseFloatVector { values, .. } if values[0].get(&2) == Some(&0.5))
        );
    }

    #[test]
    fn narrow_integer_wire_data_decodes_to_its_sdk_type() {
        use schema::{field_data::Field, scalar_field};

        for (data_type, expected) in [
            (
                schema::DataType::Int8,
                FieldData::Int8 {
                    name: "value".into(),
                    values: vec![-128, 127],
                },
            ),
            (
                schema::DataType::Int16,
                FieldData::Int16 {
                    name: "value".into(),
                    values: vec![-32768, 32767],
                },
            ),
        ] {
            let decoded = field_data(schema::FieldData {
                r#type: data_type as i32,
                field_name: "value".into(),
                field: Some(Field::Scalars(schema::ScalarField {
                    valid_data: Vec::new(),
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: match data_type {
                            schema::DataType::Int8 => vec![-128, 127],
                            schema::DataType::Int16 => vec![-32768, 32767],
                            _ => unreachable!(),
                        },
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            })
            .unwrap();
            assert_eq!(decoded, expected);
        }
    }

    #[test]
    fn shared_scalar_arrays_reject_mismatched_and_unknown_type_tags() {
        use schema::{field_data::Field, scalar_field, vector_field};

        fn int_field(data_type: i32) -> schema::FieldData {
            schema::FieldData {
                r#type: data_type,
                field_name: "integer".into(),
                field: Some(Field::Scalars(schema::ScalarField {
                    valid_data: Vec::new(),
                    data: Some(scalar_field::Data::IntData(schema::IntArray {
                        data: vec![1],
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            }
        }

        fn string_field(data_type: i32) -> schema::FieldData {
            schema::FieldData {
                r#type: data_type,
                field_name: "text".into(),
                field: Some(Field::Scalars(schema::ScalarField {
                    valid_data: Vec::new(),
                    data: Some(scalar_field::Data::StringData(schema::StringArray {
                        data: vec!["value".into()],
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            }
        }

        for value in [
            int_field(schema::DataType::Bool as i32),
            int_field(i32::MAX),
            string_field(schema::DataType::Bool as i32),
            string_field(i32::MAX),
            schema::FieldData {
                r#type: schema::DataType::Int64 as i32,
                field_name: "boolean".into(),
                field: Some(Field::Scalars(schema::ScalarField {
                    valid_data: Vec::new(),
                    data: Some(scalar_field::Data::BoolData(schema::BoolArray {
                        data: vec![true],
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            },
            schema::FieldData {
                r#type: schema::DataType::Bool as i32,
                field_name: "integer".into(),
                field: Some(Field::Scalars(schema::ScalarField {
                    valid_data: Vec::new(),
                    data: Some(scalar_field::Data::LongData(schema::LongArray {
                        data: vec![1],
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            },
            schema::FieldData {
                r#type: schema::DataType::Bool as i32,
                field_name: "embedding".into(),
                field: Some(Field::Vectors(schema::VectorField {
                    valid_data: Vec::new(),
                    dim: 2,
                    data: Some(vector_field::Data::FloatVector(schema::FloatArray {
                        data: vec![0.1, 0.2],
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            },
        ] {
            assert!(field_data(value).is_err());
        }
    }

    #[test]
    fn typed_arrays_round_trip_without_json_type_erasure() {
        let cases = vec![
            FieldData::ArrayBool {
                name: "values".into(),
                values: vec![vec![true, false]],
            },
            FieldData::ArrayInt8 {
                name: "values".into(),
                values: vec![vec![-128, 127]],
            },
            FieldData::ArrayInt16 {
                name: "values".into(),
                values: vec![vec![-32768, 32767]],
            },
            FieldData::ArrayInt32 {
                name: "values".into(),
                values: vec![vec![-1, 1]],
            },
            FieldData::ArrayInt64 {
                name: "values".into(),
                values: vec![vec![-1, 1]],
            },
            FieldData::ArrayFloat {
                name: "values".into(),
                values: vec![vec![0.5, 1.5]],
            },
            FieldData::ArrayDouble {
                name: "values".into(),
                values: vec![vec![0.5, 1.5]],
            },
            FieldData::ArrayVarChar {
                name: "values".into(),
                values: vec![vec!["a".into(), "b".into()]],
            },
        ];

        for original in cases {
            let decoded = field_data(original.clone().into_proto().unwrap()).unwrap();
            assert_eq!(decoded, original);
        }
    }

    #[test]
    fn half_precision_wire_bytes_decode_to_u16_values() {
        use schema::{field_data::Field, vector_field};

        let decoded = field_data(schema::FieldData {
            r#type: schema::DataType::Float16Vector as i32,
            field_name: "embedding".into(),
            field: Some(Field::Vectors(schema::VectorField {
                valid_data: Vec::new(),
                dim: 2,
                data: Some(vector_field::Data::Float16Vector(vec![
                    0x00, 0x3c, 0x00, 0xbc,
                ])),
                ..Default::default()
            })),
            ..Default::default()
        })
        .unwrap();

        assert!(matches!(
            decoded,
            FieldData::Float16Vector { values, .. }
                if values == vec![vec![0x3c00, 0xbc00]]
        ));
    }

    fn advanced_fields() -> Vec<schema::FieldData> {
        use schema::{field_data, scalar_field, vector_field};

        let geometry = schema::FieldData {
            r#type: schema::DataType::Geometry as i32,
            field_name: "location".into(),
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::GeometryWktData(
                    schema::GeometryWktArray {
                        data: vec!["POINT (1 2)".into()],
                    },
                )),
                ..Default::default()
            })),
            ..Default::default()
        };
        let timestamptz = schema::FieldData {
            r#type: schema::DataType::Timestamptz as i32,
            field_name: "observed_at".into(),
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::StringData(schema::StringArray {
                    data: vec!["2026-07-15T12:30:00+08:00".into()],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let label = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "label".into(),
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::VarChar as i32,
                    data: vec![schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(scalar_field::Data::StringData(schema::StringArray {
                            data: vec!["start".into(), "finish".into()],
                        })),
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let nested_geometry = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "location".into(),
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::Geometry as i32,
                    data: vec![schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(scalar_field::Data::GeometryWktData(
                            schema::GeometryWktArray {
                                data: vec!["POINT (3 4)".into(), "POINT (5 6)".into()],
                            },
                        )),
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let nested_time = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "observed_at".into(),
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::Timestamptz as i32,
                    data: vec![schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(scalar_field::Data::StringData(schema::StringArray {
                            data: vec![
                                "2026-07-15T12:31:00+08:00".into(),
                                "2026-07-15T12:32:00+08:00".into(),
                            ],
                        })),
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let nested_embedding = schema::FieldData {
            r#type: schema::DataType::ArrayOfVector as i32,
            field_name: "embedding".into(),
            field: Some(field_data::Field::Vectors(schema::VectorField {
                valid_data: Vec::new(),
                dim: 2,
                data: Some(vector_field::Data::VectorArray(schema::VectorArray {
                    dim: 2,
                    element_type: schema::DataType::FloatVector as i32,
                    data: vec![schema::VectorField {
                        valid_data: Vec::new(),
                        dim: 2,
                        data: Some(vector_field::Data::FloatVector(schema::FloatArray {
                            data: vec![0.1, 0.2, 0.3, 0.4],
                        })),
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let events = schema::FieldData {
            r#type: schema::DataType::ArrayOfStruct as i32,
            field_name: "events".into(),
            field: Some(field_data::Field::StructArrays(schema::StructArrayField {
                fields: vec![label, nested_geometry, nested_time, nested_embedding],
                ..Default::default()
            })),
            ..Default::default()
        };
        vec![geometry, timestamptz, events]
    }

    fn nullable_struct_fields() -> Vec<schema::FieldData> {
        use schema::{field_data, scalar_field};

        let id = schema::FieldData {
            r#type: schema::DataType::Int64 as i32,
            field_name: "id".into(),
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                data: Some(scalar_field::Data::LongData(schema::LongArray {
                    data: vec![1, 2],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let rating = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "rating".into(),
            valid_data: vec![false, true],
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![false, true],
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::Int32 as i32,
                    data: vec![
                        schema::ScalarField::default(),
                        schema::ScalarField {
                            data: Some(scalar_field::Data::IntData(schema::IntArray {
                                data: vec![5, 4],
                            })),
                            ..Default::default()
                        },
                    ],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let tag = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "tag".into(),
            valid_data: vec![false, true],
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![false, true],
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::VarChar as i32,
                    data: vec![
                        schema::ScalarField::default(),
                        schema::ScalarField {
                            data: Some(scalar_field::Data::StringData(schema::StringArray {
                                data: vec!["favorite".into(), "classic".into()],
                            })),
                            ..Default::default()
                        },
                    ],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let metadata = schema::FieldData {
            r#type: schema::DataType::ArrayOfStruct as i32,
            field_name: "metadata".into(),
            field: Some(field_data::Field::StructArrays(schema::StructArrayField {
                fields: vec![rating, tag],
                ..Default::default()
            })),
            ..Default::default()
        };
        vec![id, metadata]
    }

    fn assert_advanced_fields(fields: &[FieldData]) {
        assert!(matches!(fields[0], FieldData::Geometry { .. }));
        assert!(matches!(fields[1], FieldData::Timestamptz { .. }));
        let FieldData::Struct { values, .. } = &fields[2] else {
            panic!("expected struct field data");
        };
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].len(), 2);
        assert_eq!(values[0][0]["label"], "start");
        assert_eq!(values[0][1]["location"], "POINT (5 6)");
        assert_eq!(values[0][1]["observed_at"], "2026-07-15T12:32:00+08:00");
        let first_embedding = values[0][0]["embedding"].as_array().unwrap();
        let second_embedding = values[0][1]["embedding"].as_array().unwrap();
        assert_eq!(first_embedding.len(), 2);
        assert_eq!(second_embedding.len(), 2);
        assert!((first_embedding[0].as_f64().unwrap() - 0.1).abs() < 1e-6);
        assert!((first_embedding[1].as_f64().unwrap() - 0.2).abs() < 1e-6);
        assert!((second_embedding[0].as_f64().unwrap() - 0.3).abs() < 1e-6);
        assert!((second_embedding[1].as_f64().unwrap() - 0.4).abs() < 1e-6);
    }

    #[test]
    fn query_and_search_decode_geometry_timestamptz_and_struct_fields() {
        let fields = advanced_fields();
        let query = QueryResponse::from_proto(milvus::QueryResults {
            collection_name: "ignored_collection".into(),
            fields_data: fields.clone(),
            output_fields: vec!["geometry".into(), "observed_at".into(), "events".into()],
            primary_field_name: "ignored_primary_field".into(),
            ..Default::default()
        })
        .unwrap();
        assert_advanced_fields(query.results().get_output_fields());
        assert_eq!(
            query.results().get_output_field_names(),
            ["geometry", "observed_at", "events"]
        );
        assert_eq!(query.results().get_row_count().to_owned(), 1);

        let search = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 1,
                top_k: 1,
                topks: vec![1],
                scores: vec![0.9],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![1],
                    })),
                    ..Default::default()
                }),
                fields_data: fields,
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
        assert_advanced_fields(search.results().get_results()[0].get_output_fields());
    }

    #[test]
    fn query_decodes_nullable_struct_fields() {
        let query = QueryResponse::from_proto(milvus::QueryResults {
            collection_name: "ignored_collection".into(),
            fields_data: nullable_struct_fields(),
            output_fields: vec!["id".into(), "metadata".into()],
            primary_field_name: "id".into(),
            ..Default::default()
        })
        .unwrap();

        let rows: Vec<_> = query.results().rows().unwrap().collect();
        assert_eq!(rows.len(), 2);
        assert!(matches!(
            rows[0].get("metadata").unwrap(),
            crate::v2::types::ResultValue::Null
        ));
        let crate::v2::types::ResultValue::Struct(values) = rows[1].get("metadata").unwrap() else {
            panic!("expected non-null struct values");
        };
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].get("rating"), Some(&serde_json::json!(5)));
        assert_eq!(values[0].get("tag"), Some(&serde_json::json!("favorite")));
        assert_eq!(values[1].get("rating"), Some(&serde_json::json!(4)));
        assert_eq!(values[1].get("tag"), Some(&serde_json::json!("classic")));
    }

    #[test]
    fn query_decodes_struct_with_differing_sub_field_validity() {
        // A nullable sub-field can be null in one row while the parent and the
        // other sub-fields are valid, producing differing per-sub-field
        // valid_data. Any invalid sub-field row decodes as a null struct row,
        // matching pymilvus, instead of failing the whole query. The decode
        // must not depend on the order of the sub-fields in the response.
        use schema::{field_data, scalar_field};
        let rating = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "rating".into(),
            valid_data: vec![false, true],
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![false, true],
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::Int32 as i32,
                    data: vec![
                        schema::ScalarField::default(),
                        schema::ScalarField {
                            data: Some(scalar_field::Data::IntData(schema::IntArray {
                                data: vec![5, 4],
                            })),
                            ..Default::default()
                        },
                    ],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let tag = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "tag".into(),
            valid_data: vec![true, true],
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![true, true],
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::VarChar as i32,
                    data: vec![
                        schema::ScalarField {
                            data: Some(scalar_field::Data::StringData(schema::StringArray {
                                data: vec!["obsolete".into()],
                            })),
                            ..Default::default()
                        },
                        schema::ScalarField {
                            data: Some(scalar_field::Data::StringData(schema::StringArray {
                                data: vec!["favorite".into(), "classic".into()],
                            })),
                            ..Default::default()
                        },
                    ],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        for fields in [
            vec![rating.clone(), tag.clone()],
            vec![tag.clone(), rating.clone()],
        ] {
            let metadata = schema::FieldData {
                r#type: schema::DataType::ArrayOfStruct as i32,
                field_name: "metadata".into(),
                field: Some(field_data::Field::StructArrays(schema::StructArrayField {
                    fields,
                    ..Default::default()
                })),
                ..Default::default()
            };
            let query = QueryResponse::from_proto(milvus::QueryResults {
                collection_name: "ignored_collection".into(),
                fields_data: vec![
                    schema::FieldData {
                        r#type: schema::DataType::Int64 as i32,
                        field_name: "id".into(),
                        field: Some(field_data::Field::Scalars(schema::ScalarField {
                            data: Some(scalar_field::Data::LongData(schema::LongArray {
                                data: vec![1, 2],
                            })),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    metadata,
                ],
                output_fields: vec!["id".into(), "metadata".into()],
                primary_field_name: "id".into(),
                ..Default::default()
            })
            .unwrap();

            let rows: Vec<_> = query.results().rows().unwrap().collect();
            assert_eq!(rows.len(), 2);
            assert!(matches!(
                rows[0].get("metadata").unwrap(),
                crate::v2::types::ResultValue::Null
            ));
            let crate::v2::types::ResultValue::Struct(values) = rows[1].get("metadata").unwrap()
            else {
                panic!("expected non-null struct values");
            };
            assert_eq!(values.len(), 2);
            assert_eq!(values[0].get("rating"), Some(&serde_json::json!(5)));
            assert_eq!(values[0].get("tag"), Some(&serde_json::json!("favorite")));
            assert_eq!(values[1].get("rating"), Some(&serde_json::json!(4)));
            assert_eq!(values[1].get("tag"), Some(&serde_json::json!("classic")));
        }
    }

    #[test]
    fn query_decodes_compacted_nullable_struct_payload() {
        // A nullable struct row can arrive with a compacted payload where only the
        // non-null rows carry data and a shared valid_data mask marks the null rows.
        // The decoder must re-expand the compacted sub-field columns so the null
        // struct rows map back to their original positions (Java SDK alignColumnData).
        use schema::{field_data, scalar_field};
        let rating = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "rating".into(),
            valid_data: vec![false, true, false],
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![false, true, false],
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::Int32 as i32,
                    data: vec![schema::ScalarField {
                        data: Some(scalar_field::Data::IntData(schema::IntArray {
                            data: vec![5, 4],
                        })),
                        ..Default::default()
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let tag = schema::FieldData {
            r#type: schema::DataType::Array as i32,
            field_name: "tag".into(),
            valid_data: vec![false, true, false],
            field: Some(field_data::Field::Scalars(schema::ScalarField {
                valid_data: vec![false, true, false],
                data: Some(scalar_field::Data::ArrayData(schema::ArrayArray {
                    element_type: schema::DataType::VarChar as i32,
                    data: vec![schema::ScalarField {
                        data: Some(scalar_field::Data::StringData(schema::StringArray {
                            data: vec!["favorite".into(), "classic".into()],
                        })),
                        ..Default::default()
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let metadata = schema::FieldData {
            r#type: schema::DataType::ArrayOfStruct as i32,
            field_name: "metadata".into(),
            field: Some(field_data::Field::StructArrays(schema::StructArrayField {
                fields: vec![rating, tag],
                ..Default::default()
            })),
            ..Default::default()
        };
        let query = QueryResponse::from_proto(milvus::QueryResults {
            collection_name: "ignored_collection".into(),
            fields_data: vec![
                schema::FieldData {
                    r#type: schema::DataType::Int64 as i32,
                    field_name: "id".into(),
                    field: Some(field_data::Field::Scalars(schema::ScalarField {
                        data: Some(scalar_field::Data::LongData(schema::LongArray {
                            data: vec![1, 2, 3],
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                metadata,
            ],
            output_fields: vec!["id".into(), "metadata".into()],
            primary_field_name: "id".into(),
            ..Default::default()
        })
        .unwrap();

        let rows: Vec<_> = query.results().rows().unwrap().collect();
        assert_eq!(rows.len(), 3);
        assert!(matches!(
            rows[0].get("metadata").unwrap(),
            crate::v2::types::ResultValue::Null
        ));
        let crate::v2::types::ResultValue::Struct(values) = rows[1].get("metadata").unwrap() else {
            panic!("expected non-null struct values");
        };
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].get("rating"), Some(&serde_json::json!(5)));
        assert_eq!(values[0].get("tag"), Some(&serde_json::json!("favorite")));
        assert_eq!(values[1].get("rating"), Some(&serde_json::json!(4)));
        assert_eq!(values[1].get("tag"), Some(&serde_json::json!("classic")));
        assert!(matches!(
            rows[2].get("metadata").unwrap(),
            crate::v2::types::ResultValue::Null
        ));
    }

    #[test]
    fn query_decodes_compacted_nullable_struct_vector_subfield() {
        // align_struct_subfield is shared by the scalar (ArrayData) and vector
        // (VectorArray) sub-field decode paths; verify the compacted re-expansion
        // also works when the struct carries a vector sub-field.
        use schema::{field_data, vector_field};
        let embedding = schema::FieldData {
            r#type: schema::DataType::ArrayOfVector as i32,
            field_name: "embedding".into(),
            valid_data: vec![false, true, false],
            field: Some(field_data::Field::Vectors(schema::VectorField {
                valid_data: vec![false, true, false],
                dim: 2,
                data: Some(vector_field::Data::VectorArray(schema::VectorArray {
                    dim: 2,
                    element_type: schema::DataType::FloatVector as i32,
                    data: vec![schema::VectorField {
                        valid_data: Vec::new(),
                        dim: 2,
                        data: Some(vector_field::Data::FloatVector(schema::FloatArray {
                            data: vec![1.0, 2.0, 3.0, 4.0],
                        })),
                    }],
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        let metadata = schema::FieldData {
            r#type: schema::DataType::ArrayOfStruct as i32,
            field_name: "metadata".into(),
            field: Some(field_data::Field::StructArrays(schema::StructArrayField {
                fields: vec![embedding],
                ..Default::default()
            })),
            ..Default::default()
        };
        let query = QueryResponse::from_proto(milvus::QueryResults {
            collection_name: "ignored_collection".into(),
            fields_data: vec![
                schema::FieldData {
                    r#type: schema::DataType::Int64 as i32,
                    field_name: "id".into(),
                    field: Some(field_data::Field::Scalars(schema::ScalarField {
                        data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                            data: vec![1, 2, 3],
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                metadata,
            ],
            output_fields: vec!["id".into(), "metadata".into()],
            primary_field_name: "id".into(),
            ..Default::default()
        })
        .unwrap();

        let rows: Vec<_> = query.results().rows().unwrap().collect();
        assert_eq!(rows.len(), 3);
        assert!(matches!(
            rows[0].get("metadata").unwrap(),
            crate::v2::types::ResultValue::Null
        ));
        let crate::v2::types::ResultValue::Struct(values) = rows[1].get("metadata").unwrap() else {
            panic!("expected non-null struct values");
        };
        assert_eq!(values.len(), 2);
        assert_eq!(
            values[0].get("embedding"),
            Some(&serde_json::json!([1.0, 2.0]))
        );
        assert_eq!(
            values[1].get("embedding"),
            Some(&serde_json::json!([3.0, 4.0]))
        );
        assert!(matches!(
            rows[2].get("metadata").unwrap(),
            crate::v2::types::ResultValue::Null
        ));
    }

    #[test]
    fn search_rejects_missing_result_payload() {
        let error = SearchResponse::from_proto(milvus::SearchResults {
            status: Some(common::Status::default()),
            results: None,
            ..Default::default()
        })
        .expect_err("a successful search response must contain result data");

        assert_eq!(
            error.to_string(),
            "malformed server response: no result for search"
        );
    }

    #[test]
    fn search_decodes_statistics_score_name_and_highlights() {
        let search = SearchResponse::from_proto(milvus::SearchResults {
            status: Some(common::Status {
                extra_info: [
                    ("report_value".into(), "12".into()),
                    ("scanned_remote_bytes".into(), "34".into()),
                    ("scanned_total_bytes".into(), "56".into()),
                    ("cache_hit_ratio".into(), "0.75".into()),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            }),
            results: Some(schema::SearchResultData {
                num_queries: 1,
                top_k: 1,
                topks: vec![1],
                scores: vec![0.9],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![1],
                    })),
                    ..Default::default()
                }),
                fields_data: vec![schema::FieldData {
                    r#type: schema::DataType::Int64 as i32,
                    field_name: "score".into(),
                    field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                            data: vec![42],
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                }],
                highlight_results: vec![common::HighlightResult {
                    field_name: "text".into(),
                    datas: vec![common::HighlightData {
                        fragments: vec!["<em>Milvus</em>".into()],
                        scores: vec![0.8],
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(search.cost().to_owned(), 12);
        assert_eq!(search.scanned_remote_bytes().to_owned(), 34);
        assert_eq!(search.scanned_total_bytes().to_owned(), 56);
        assert_eq!(search.cache_hit_ratio().to_owned(), 0.75);
        let result = &search.results().get_results()[0];
        assert_eq!(result.get_score_field_name().to_owned(), "_score");
        let highlight = &result.get_highlight_results()[0]["text"];
        assert_eq!(highlight.get_field_name().to_owned(), "text");
        assert_eq!(highlight.get_fragments().to_owned(), ["<em>Milvus</em>"]);
        assert_eq!(highlight.get_scores().to_owned(), [0.8]);
    }

    #[test]
    fn query_propagates_malformed_json_field_error() {
        let response = QueryResponse::from_proto(milvus::QueryResults {
            fields_data: vec![schema::FieldData {
                r#type: schema::DataType::Json as i32,
                field_name: "metadata".into(),
                field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                    valid_data: Vec::new(),
                    data: Some(schema::scalar_field::Data::JsonData(schema::JsonArray {
                        data: vec![b"not-json".to_vec()],
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        });

        assert!(response.is_err());
    }

    #[test]
    fn nullable_json_ignores_invalid_row_placeholders() {
        let decoded = field_data(schema::FieldData {
            r#type: schema::DataType::Json as i32,
            field_name: "metadata".into(),
            valid_data: vec![true, false],
            field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                valid_data: Vec::new(),
                data: Some(schema::scalar_field::Data::JsonData(schema::JsonArray {
                    data: vec![br#"{"present":true}"#.to_vec(), Vec::new()],
                })),
                ..Default::default()
            })),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(decoded.valid_data(), Some([true, false].as_slice()));
        assert!(decoded.is_null(1));
        assert!(matches!(
            decoded.inner(),
            FieldData::Json { values, .. } if values == &vec![serde_json::json!({"present": true})]
        ));
    }

    #[test]
    fn search_propagates_out_of_range_narrow_integer_error() {
        let response = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 1,
                top_k: 1,
                topks: vec![1],
                scores: vec![0.9],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![1],
                    })),
                    ..Default::default()
                }),
                fields_data: vec![schema::FieldData {
                    r#type: schema::DataType::Int8 as i32,
                    field_name: "narrow".into(),
                    field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(schema::scalar_field::Data::IntData(schema::IntArray {
                            data: vec![128],
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        });

        assert!(response.is_err());
    }

    #[test]
    fn search_field_splitting_moves_vector_rows_without_cloning_buffers() {
        let rows = vec![vec![0.1, 0.2], vec![0.3, 0.4], vec![0.5, 0.6]];
        let row_pointers = rows.iter().map(|row| row.as_ptr()).collect::<Vec<_>>();
        let fields = split_field_data(
            FieldData::FloatVector {
                name: "embedding".into(),
                values: rows,
            },
            &[2, 1],
        )
        .unwrap();

        let pointers = fields
            .iter()
            .flat_map(|field| match field {
                FieldData::FloatVector { values, .. } => {
                    values.iter().map(|row| row.as_ptr()).collect::<Vec<_>>()
                }
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(pointers, row_pointers);
    }

    #[test]
    fn search_splits_flattened_data_into_one_result_per_query_vector() {
        use schema::{field_data::Field, scalar_field};

        let search = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 2,
                top_k: 2,
                topks: vec![2, 1],
                scores: vec![0.99, 0.98, 0.75],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![10, 11, 20],
                    })),
                    ..Default::default()
                }),
                element_indices: Some(schema::LongArray {
                    data: vec![2, 4, 1],
                    ..Default::default()
                }),
                fields_data: vec![
                    schema::FieldData {
                        r#type: schema::DataType::Int64 as i32,
                        field_name: "value".into(),
                        field: Some(Field::Scalars(schema::ScalarField {
                            valid_data: Vec::new(),
                            data: Some(scalar_field::Data::LongData(schema::LongArray {
                                data: vec![100, 101, 200],
                            })),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    schema::FieldData {
                        r#type: schema::DataType::VarChar as i32,
                        field_name: "nullable_text".into(),
                        valid_data: vec![true, false, true],
                        field: Some(Field::Scalars(schema::ScalarField {
                            valid_data: Vec::new(),
                            data: Some(scalar_field::Data::StringData(schema::StringArray {
                                data: vec!["first".into(), "third".into()],
                            })),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                output_fields: vec!["value".into(), "nullable_text".into()],
                primary_field_name: "id".into(),
                recalls: vec![0.9, 0.8],
                highlight_results: vec![common::HighlightResult {
                    field_name: "text".into(),
                    datas: ["q1-first", "q1-second", "q2-first"]
                        .into_iter()
                        .map(|fragment| common::HighlightData {
                            fragments: vec![fragment.into()],
                            scores: vec![1.0],
                        })
                        .collect(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();

        let results = search.results();
        assert_eq!(results.get_recalls().to_owned(), [0.9, 0.8]);
        assert_eq!(results.len(), 2);

        let first = &results.get_results()[0];
        assert_eq!(
            first.get_ids().to_owned(),
            crate::v2::types::Ids::Int64(vec![10, 11])
        );
        assert_eq!(first.get_scores().to_owned(), [0.99, 0.98]);
        assert_eq!(first.get_element_indices(), Some([2, 4].as_slice()));
        assert_eq!(
            first
                .rows()
                .expect("valid element-level rows")
                .map(|row| row.element_offset())
                .collect::<Vec<_>>(),
            [Some(2), Some(4)]
        );
        assert_eq!(first.len(), 2);
        assert!(matches!(
            first.get_output_field("value"),
            Some(FieldData::Int64 { values, .. }) if values == &vec![100, 101]
        ));
        assert!(matches!(
            first.get_output_field("nullable_text"),
            Some(FieldData::Nullable { valid_data, data })
                if valid_data == &vec![true, false]
                    && matches!(data.as_ref(), FieldData::VarChar { values, .. } if values == &vec!["first".to_owned()])
        ));
        assert_eq!(
            first.get_highlight_results()[1]["text"].get_fragments(),
            ["q1-second"]
        );

        let second = &results.get_results()[1];
        assert_eq!(
            second.get_ids().to_owned(),
            crate::v2::types::Ids::Int64(vec![20])
        );
        assert_eq!(second.get_scores().to_owned(), [0.75]);
        assert_eq!(second.get_element_indices(), Some([1].as_slice()));
        assert!(matches!(
            second.get_output_field("value"),
            Some(FieldData::Int64 { values, .. }) if values == &vec![200]
        ));
        assert!(matches!(
            second.get_output_field("nullable_text"),
            Some(FieldData::Nullable { valid_data, data })
                if valid_data == &vec![true]
                    && matches!(data.as_ref(), FieldData::VarChar { values, .. } if values == &vec!["third".to_owned()])
        ));
        assert_eq!(
            second.get_highlight_results()[0]["text"].get_fragments(),
            ["q2-first"]
        );
    }

    #[test]
    fn search_rejects_oversized_multi_query_field_payload() {
        use schema::{field_data::Field, scalar_field};

        let response = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 2,
                top_k: 1,
                topks: vec![1, 1],
                scores: vec![0.9, 0.8],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![10, 20],
                    })),
                    ..Default::default()
                }),
                fields_data: vec![schema::FieldData {
                    r#type: schema::DataType::Int64 as i32,
                    field_name: "value".into(),
                    field: Some(Field::Scalars(schema::ScalarField {
                        valid_data: Vec::new(),
                        data: Some(scalar_field::Data::LongData(schema::LongArray {
                            data: vec![100, 200, 300],
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                }],
                output_fields: vec!["value".into()],
                primary_field_name: "id".into(),
                ..Default::default()
            }),
            ..Default::default()
        });

        let error = response.expect_err("oversized output field payload must fail decoding");
        assert!(error.to_string().contains("top-k values"));
    }

    #[test]
    fn search_row_limit_truncates_an_oversized_single_query_result() {
        use schema::{field_data::Field, scalar_field};

        let search = SearchResponse::from_proto_with_row_limit(
            milvus::SearchResults {
                results: Some(schema::SearchResultData {
                    num_queries: 1,
                    top_k: 3,
                    topks: vec![3],
                    scores: vec![0.9, 0.8, 0.7],
                    ids: Some(schema::IDs {
                        id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                            data: vec![10, 11, 12],
                        })),
                        ..Default::default()
                    }),
                    element_indices: Some(schema::LongArray {
                        data: vec![4, 5, 6],
                        ..Default::default()
                    }),
                    fields_data: vec![
                        schema::FieldData {
                            r#type: schema::DataType::Int16 as i32,
                            field_name: "age".into(),
                            valid_data: vec![true, true, true],
                            field: Some(Field::Scalars(schema::ScalarField {
                                valid_data: Vec::new(),
                                data: Some(scalar_field::Data::IntData(schema::IntArray {
                                    data: vec![20, 21],
                                })),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        schema::FieldData {
                            r#type: schema::DataType::VarChar as i32,
                            field_name: "name".into(),
                            valid_data: vec![true, true, true],
                            field: Some(Field::Scalars(schema::ScalarField {
                                valid_data: Vec::new(),
                                data: Some(scalar_field::Data::StringData(schema::StringArray {
                                    data: vec!["first".into(), "second".into()],
                                })),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    ],
                    primary_field_name: "id".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Some(2),
        )
        .expect("truncate search response");

        let result = &search.results().get_results()[0];
        assert_eq!(
            result.get_ids().to_owned(),
            crate::v2::Ids::Int64(vec![10, 11])
        );
        assert_eq!(result.get_scores().to_owned(), [0.9, 0.8]);
        assert_eq!(result.get_element_indices(), Some([4, 5].as_slice()));
        assert!(matches!(
            result.get_output_field("age"),
            Some(FieldData::Nullable { valid_data, data })
                if valid_data == &vec![true, true]
                    && matches!(data.as_ref(), FieldData::Int16 { values, .. } if values == &vec![20, 21])
        ));
        assert!(matches!(
            result.get_output_field("name"),
            Some(FieldData::Nullable { valid_data, data })
                if valid_data == &vec![true, true]
                    && matches!(data.as_ref(), FieldData::VarChar { values, .. } if values == &vec!["first".to_owned(), "second".to_owned()])
        ));
    }

    #[test]
    fn search_truncation_also_truncates_field_specific_validity() {
        use schema::{field_data::Field, scalar_field};

        // The server sends field-level validity (newer proto channel) with surplus rows; the
        // single-query row-limit truncation must slice the field-specific validity alongside the
        // payload so decode does not fail on a length mismatch.
        let search = SearchResponse::from_proto_with_row_limit(
            milvus::SearchResults {
                results: Some(schema::SearchResultData {
                    num_queries: 1,
                    top_k: 3,
                    topks: vec![3],
                    scores: vec![0.9, 0.8, 0.7],
                    ids: Some(schema::IDs {
                        id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                            data: vec![10, 11, 12],
                        })),
                        ..Default::default()
                    }),
                    element_indices: Some(schema::LongArray {
                        data: vec![4, 5, 6],
                        ..Default::default()
                    }),
                    fields_data: vec![schema::FieldData {
                        r#type: schema::DataType::Int16 as i32,
                        field_name: "age".into(),
                        valid_data: Vec::new(),
                        field: Some(Field::Scalars(schema::ScalarField {
                            valid_data: vec![true, true, true, true],
                            data: Some(scalar_field::Data::IntData(schema::IntArray {
                                data: vec![20, 21, 22, 23],
                            })),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }],
                    primary_field_name: "id".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Some(2),
        )
        .expect("truncate search response with field-specific validity");

        let result = &search.results().get_results()[0];
        assert!(matches!(
            result.get_output_field("age"),
            Some(FieldData::Nullable { valid_data, data })
                if valid_data == &vec![true, true]
                    && matches!(data.as_ref(), FieldData::Int16 { values, .. } if values == &vec![20, 21])
        ));
    }

    #[test]
    fn search_rejects_short_element_indices() {
        let response = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 1,
                top_k: 2,
                topks: vec![2],
                scores: vec![0.9, 0.8],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![10, 10],
                    })),
                    ..Default::default()
                }),
                element_indices: Some(schema::LongArray {
                    data: vec![3],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        });

        let error = response.expect_err("short element indices must fail decoding");
        assert!(error.to_string().contains("element indices"));
    }

    #[test]
    fn search_iterator_split_and_append_preserve_element_indices() {
        let response = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 1,
                top_k: 3,
                topks: vec![3],
                scores: vec![0.9, 0.8, 0.7],
                ids: Some(schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: vec![10, 10, 11],
                    })),
                    ..Default::default()
                }),
                element_indices: Some(schema::LongArray {
                    data: vec![2, 4, 1],
                    ..Default::default()
                }),
                primary_field_name: "id".into(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("decode element-level search response");

        let (mut first, remaining) = response.split_at(1).expect("split iterator page");
        let remaining = remaining.expect("split has remaining rows");
        assert_eq!(
            first.results().get_results()[0].get_element_indices(),
            Some([2].as_slice())
        );
        assert_eq!(
            remaining.results().get_results()[0].get_element_indices(),
            Some([4, 1].as_slice())
        );

        first.append(remaining).expect("append iterator page");
        assert_eq!(
            first.results().get_results()[0].get_element_indices(),
            Some([2, 4, 1].as_slice())
        );
    }

    #[test]
    fn search_zero_row_page_ignores_empty_field_descriptors() {
        let search = SearchResponse::from_proto_with_row_limit(
            milvus::SearchResults {
                results: Some(schema::SearchResultData {
                    num_queries: 1,
                    topks: vec![0],
                    ids: Some(schema::IDs::default()),
                    fields_data: vec![schema::FieldData {
                        r#type: schema::DataType::Int16 as i32,
                        field_name: "age".into(),
                        ..Default::default()
                    }],
                    primary_field_name: "id".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Some(16),
        )
        .expect("decode empty search-iterator page");

        assert_eq!(search.results().len().to_owned(), 1);
        assert!(search.results().get_results()[0].is_empty());
    }

    #[test]
    fn search_response_groups_aggregation_buckets_per_query() {
        // nq=2 with zero result rows; the first query owns one top-level bucket (agg_topks=[1, 0]).
        let search = SearchResponse::from_proto(milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 2,
                top_k: 0,
                topks: vec![0, 0],
                scores: vec![],
                ids: Some(schema::IDs::default()),
                primary_field_name: "id".into(),
                agg_buckets: vec![schema::AggBucket {
                    key: vec![schema::BucketKeyEntry {
                        field_id: 1,
                        field_name: "category".into(),
                        value: Some(schema::bucket_key_entry::Value::StringVal("tech".into())),
                    }],
                    count: 3,
                    metrics: std::collections::HashMap::new(),
                    hits: vec![],
                    sub_groups: vec![],
                }],
                agg_topks: vec![1, 0],
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("decode aggregation search response");

        assert_eq!(search.results().get_agg_buckets().len(), 2);
        assert_eq!(search.results().get_agg_buckets()[0].len(), 1);
        let bucket = &search.results().get_agg_buckets()[0][0];
        assert_eq!(bucket.get_count(), 3);
        assert_eq!(bucket.get_key()[0].get_field_name(), "category");
        assert_eq!(
            bucket.get_key()[0].get_value(),
            &AggregationBucketValue::String("tech".to_owned())
        );
        assert!(search.results().get_agg_buckets()[1].is_empty());
        assert!(search.results().get_results()[0].is_empty());
        assert!(search.results().get_results()[1].is_empty());
    }

    #[test]
    fn query_response_rejects_element_indices_length_mismatch() {
        let error = QueryResponse::from_proto(milvus::QueryResults {
            fields_data: vec![schema::FieldData {
                r#type: schema::DataType::Int64 as i32,
                field_name: "id".into(),
                field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                    data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                        data: vec![1, 2, 3],
                        ..Default::default()
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            output_fields: vec!["id".into()],
            primary_field_name: "id".into(),
            element_indices: vec![milvus::ElementIndices {
                indices: Some(schema::LongArray {
                    data: vec![0],
                    ..Default::default()
                }),
            }],
            ..Default::default()
        })
        .expect_err("element_indices length mismatch must be rejected");
        assert!(error.to_string().contains("element_indices"));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn query_response_default_values() {
        let value = QueryResponse::builder().build();
        let expected_results = QueryResults::new();
        let expected_session_timestamp: u64 = 0;

        assert_eq!(value.results().to_owned(), expected_results);
        assert_eq!(
            value.session_timestamp().to_owned(),
            expected_session_timestamp
        );
    }

    #[test]
    fn query_response_populated_values() {
        let results = QueryResults::new();
        let session_timestamp = 7;
        let value = QueryResponse::builder()
            .results(results.clone())
            .session_timestamp(session_timestamp.clone())
            .build();

        assert_eq!(value.results().to_owned(), results);
        assert_eq!(value.session_timestamp().to_owned(), session_timestamp);
    }

    #[test]
    fn search_response_default_values() {
        let value = SearchResponse::builder().build();
        let expected_results = SearchResults::new();
        let expected_session_timestamp: u64 = 0;
        let expected_cost: i64 = -1;
        let expected_scanned_remote_bytes: i64 = -1;
        let expected_scanned_total_bytes: i64 = -1;
        let expected_cache_hit_ratio: f32 = -1.0;

        assert_eq!(value.results().to_owned(), expected_results);
        assert_eq!(
            value.session_timestamp().to_owned(),
            expected_session_timestamp
        );
        assert_eq!(value.cost().to_owned(), expected_cost);
        assert_eq!(value.scanned_remote_bytes(), expected_scanned_remote_bytes);
        assert_eq!(value.scanned_total_bytes(), expected_scanned_total_bytes);
        assert_eq!(value.cache_hit_ratio().to_owned(), expected_cache_hit_ratio);
    }

    #[test]
    fn search_response_populated_values() {
        let results = SearchResults::new();
        let session_timestamp = 7;
        let cost = 7;
        let scanned_remote_bytes = 7;
        let scanned_total_bytes = 7;
        let cache_hit_ratio = 1.5;
        let value = SearchResponse::builder()
            .results(results.clone())
            .session_timestamp(session_timestamp.clone())
            .cost(cost.clone())
            .scanned_remote_bytes(scanned_remote_bytes.clone())
            .scanned_total_bytes(scanned_total_bytes.clone())
            .cache_hit_ratio(cache_hit_ratio.clone())
            .build();

        assert_eq!(value.results().to_owned(), results);
        assert_eq!(value.session_timestamp().to_owned(), session_timestamp);
        assert_eq!(value.cost().to_owned(), cost);
        assert_eq!(
            value.scanned_remote_bytes().to_owned(),
            scanned_remote_bytes
        );
        assert_eq!(value.scanned_total_bytes().to_owned(), scanned_total_bytes);
        assert_eq!(value.cache_hit_ratio().to_owned(), cache_hit_ratio);
    }
}
