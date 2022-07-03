use crate::proto::schema::{
    field_data, scalar_field, vector_field, BoolArray, DataType, FieldData, FloatArray, IntArray,
    LongArray, ScalarField, StringArray, VectorField,
};
use prost::{bytes::BytesMut, Message};

impl FieldData {
    pub fn binary_vector(field_name: impl Into<String>, data: Vec<bool>, dim: i64) -> FieldData {
        let bool_array = BoolArray { data };
        let mut buf = BytesMut::new();
        bool_array
            .encode(&mut buf)
            .expect("Could not encode binary vec");
        let buf = buf.freeze();

        FieldData {
            r#type: DataType::BinaryVector as i32,
            field_name: field_name.into(),
            field_id: 0,
            field: Some(field_data::Field::Vectors(VectorField {
                dim,
                data: Some(vector_field::Data::BinaryVector(buf.to_vec())),
            })),
        }
    }

    pub fn float_vector(
        field_name: impl Into<String>,
        data: impl Into<Vec<f32>>,
        dim: i64,
    ) -> FieldData {
        FieldData {
            r#type: DataType::FloatVector as i32,
            field_name: field_name.into(),
            field_id: 0,
            field: Some(field_data::Field::Vectors(VectorField {
                dim,
                data: Some(vector_field::Data::FloatVector(FloatArray {
                    data: data.into(),
                })),
            })),
        }
    }

    pub fn int_data(field_name: impl Into<String>, data: Vec<i32>) -> FieldData {
        FieldData {
            r#type: DataType::Int32 as i32,
            field_name: field_name.into(),
            field_id: 0,
            field: Some(field_data::Field::Scalars(ScalarField {
                data: Some(scalar_field::Data::IntData(IntArray { data })),
            })),
        }
    }

    pub fn long_data(field_name: impl Into<String>, data: Vec<i64>) -> FieldData {
        FieldData {
            r#type: DataType::Int64 as i32,
            field_name: field_name.into(),
            field_id: 0,
            field: Some(field_data::Field::Scalars(ScalarField {
                data: Some(scalar_field::Data::LongData(LongArray { data })),
            })),
        }
    }

    fn float_data(field_name: impl Into<String>, data: Vec<f32>) -> FieldData {
        FieldData {
            r#type: DataType::Float as i32,
            field_name: field_name.into(),
            field_id: 0,
            field: Some(field_data::Field::Scalars(ScalarField {
                data: Some(scalar_field::Data::FloatData(FloatArray { data })),
            })),
        }
    }

    fn string_data(field_name: impl Into<String>, data: Vec<String>) -> FieldData {
        FieldData {
            r#type: DataType::String as i32,
            field_name: field_name.into(),
            field_id: 0,
            field: Some(field_data::Field::Scalars(ScalarField {
                data: Some(scalar_field::Data::StringData(StringArray { data })),
            })),
        }
    }
}
