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
            dtype: DataType::try_from(value.data_type).unwrap_or(DataType::None),
            is_primary_key: value.is_primary_key,
        }
    }
}
