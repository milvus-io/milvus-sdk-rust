use crate::proto::schema::{
  field_data, scalar_field, vector_field, BoolArray, DataType, FieldData, FloatArray, ScalarField,
  StringArray, VectorField,
};
use prost::{bytes::BytesMut, Message};

pub fn fields_data(data: Vec<impl FieldDatable>) -> Vec<FieldData> {
  data.into_iter().map(FieldDatable::to_field_data).collect()
}

pub trait FieldDatable {
  fn to_field_data(self) -> FieldData;
}

impl FieldDatable for (String, Vec<f32>, i64) {
  fn to_field_data(self) -> FieldData {
    let (field_name, data, dim) = self;
    float_vector(field_name, data, dim)
  }
}
impl FieldDatable for (String, Vec<f32>) {
  fn to_field_data(self) -> FieldData {
    let (field_name, data) = self;
    float_data(field_name, data)
  }
}
impl FieldDatable for (String, Vec<String>) {
  fn to_field_data(self) -> FieldData {
    let (field_name, data) = self;
    string_data(field_name, data)
  }
}

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

pub fn float_vector(field_name: impl Into<String>, data: Vec<f32>, dim: i64) -> FieldData {
  FieldData {
    r#type: DataType::FloatVector as i32,
    field_name: field_name.into(),
    field_id: 0,
    field: Some(field_data::Field::Vectors(VectorField {
      dim,
      data: Some(vector_field::Data::FloatVector(FloatArray { data })),
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
