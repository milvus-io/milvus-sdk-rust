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

//! Reusable value types for V2 data-manipulation operations.

use crate::proto::schema;

///////////////////////////////////////////////////////////////////////////////
// FieldPartialUpdateOpType
///////////////////////////////////////////////////////////////////////////////
/// Operation applied to a field during a partial upsert.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldPartialUpdateOpType {
    /// Overwrite the existing field value.
    #[default]
    Replace,
    /// Append the supplied elements to an existing array field.
    ArrayAppend,
    /// Remove every occurrence of the supplied elements from an existing array field.
    ArrayRemove,
}

impl FieldPartialUpdateOpType {
    pub(crate) fn into_proto(self) -> schema::field_partial_update_op::OpType {
        match self {
            Self::Replace => schema::field_partial_update_op::OpType::Replace,
            Self::ArrayAppend => schema::field_partial_update_op::OpType::ArrayAppend,
            Self::ArrayRemove => schema::field_partial_update_op::OpType::ArrayRemove,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// FieldPartialUpdateOp
///////////////////////////////////////////////////////////////////////////////
/// Describes how a field value is applied during an upsert.
///
/// `ArrayAppend` and `ArrayRemove` apply only to array fields and implicitly
/// enable partial-update semantics for the request. These array operations
/// require Milvus 2.6.17 or later.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldPartialUpdateOp {
    pub(crate) field_name: String,
    pub(crate) op_type: FieldPartialUpdateOpType,
}

impl FieldPartialUpdateOp {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            field_name: String::new(),
            op_type: FieldPartialUpdateOpType::Replace,
        }
    }

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.field_name = value.into();
        self
    }

    /// Sets the field name and returns this value for further mutation.
    pub fn set_field_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.field_name = value.into();
        self
    }

    /// Returns the configured field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// Sets the op type and returns the updated value.
    pub fn op_type(mut self, value: FieldPartialUpdateOpType) -> Self {
        self.op_type = value;
        self
    }

    /// Sets the op type and returns this value for further mutation.
    pub fn set_op_type(&mut self, value: FieldPartialUpdateOpType) -> &mut Self {
        self.op_type = value;
        self
    }

    /// Returns the configured op type.
    pub fn get_op_type(&self) -> FieldPartialUpdateOpType {
        self.op_type
    }

    pub(crate) fn into_proto(self) -> schema::FieldPartialUpdateOp {
        schema::FieldPartialUpdateOp {
            field_name: self.field_name,
            op: self.op_type.into_proto() as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{FieldPartialUpdateOp, FieldPartialUpdateOpType};
    use crate::proto::schema;

    #[test]
    fn field_partial_update_op_defaults_and_setters() {
        let mut operation = FieldPartialUpdateOp::new();
        assert!(operation.get_field_name().is_empty());
        assert_eq!(operation.get_op_type(), FieldPartialUpdateOpType::Replace);

        operation
            .set_field_name("tags")
            .set_op_type(FieldPartialUpdateOpType::ArrayAppend);
        assert_eq!(operation.get_field_name(), "tags");
        assert_eq!(
            operation.get_op_type(),
            FieldPartialUpdateOpType::ArrayAppend
        );

        let proto = operation.into_proto();
        assert_eq!(proto.field_name, "tags");
        assert_eq!(
            proto.op,
            schema::field_partial_update_op::OpType::ArrayAppend as i32
        );
    }

    #[test]
    fn field_partial_update_op_supports_fluent_construction() {
        let operation = FieldPartialUpdateOp::new()
            .field_name("tags")
            .op_type(FieldPartialUpdateOpType::ArrayRemove);

        assert_eq!(operation.get_field_name(), "tags");
        assert_eq!(
            operation.get_op_type(),
            FieldPartialUpdateOpType::ArrayRemove
        );
    }
}
