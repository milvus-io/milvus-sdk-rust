use crate::proto::{self, schema::DataType};

pub(crate) type Timestamp = u64;

#[derive(Debug, Clone)]
pub struct Field {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub dtype: DataType,
    pub is_primary_key: bool,
}

impl From<proto::schema::FieldSchema> for Field {
    fn from(value: proto::schema::FieldSchema) -> Self {
        Self {
            id: value.field_id,
            name: value.name,
            description: value.description,
            dtype: DataType::from_i32(value.data_type).unwrap_or(DataType::None),
            is_primary_key: value.is_primary_key,
        }
    }
}
