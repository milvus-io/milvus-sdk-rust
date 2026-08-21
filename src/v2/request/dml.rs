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

//! Request types for insert, upsert, and delete operations.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::required;
use crate::v2::types::{FieldData, Ids};
use serde_json::Value;
use std::collections::HashMap;

pub use crate::v2::types::{EntityRow, FieldPartialUpdateOp, FieldPartialUpdateOpType};

///////////////////////////////////////////////////////////////////////////////
// InsertRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 insert operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct InsertRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_name: String,
    pub(crate) columns: Vec<FieldData>,
    pub(crate) rows: Vec<EntityRow>,
}

impl InsertRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_name: Default::default(),
            columns: Default::default(),
            rows: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> InsertRequestBuilder {
        InsertRequestBuilder {
            value: Self::empty(),
            rows: Vec::new(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(mut self) -> InsertRequestBuilder {
        let rows = std::mem::take(&mut self.rows)
            .into_iter()
            .map(Value::Object)
            .collect();
        InsertRequestBuilder { value: self, rows }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition name.
    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    /// Returns the columns.
    pub fn columns(&self) -> &[FieldData] {
        &self.columns
    }

    /// Returns the rows.
    pub fn rows(&self) -> &[EntityRow] {
        &self.rows
    }

    pub(crate) fn to_proto_with_fields(
        &self,
        fields_data: Vec<crate::proto::schema::FieldData>,
        num_rows: usize,
        schema_timestamp: u64,
        default_db: &str,
    ) -> Result<milvus::InsertRequest> {
        Ok(milvus::InsertRequest {
            base: None,
            db_name: self
                .database_name
                .clone()
                .unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name.clone(),
            partition_name: self.partition_name.clone(),
            fields_data,
            hash_keys: Vec::new(),
            num_rows: num_rows as u32,
            schema_timestamp,
            namespace: None,
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// InsertRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for InsertRequest.
#[derive(Debug, Clone)]
pub struct InsertRequestBuilder {
    value: InsertRequest,
    rows: Vec<Value>,
}

impl InsertRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Sets the columns and returns the updated value.
    pub fn columns(mut self, value: Vec<FieldData>) -> Self {
        self.value.columns = value;
        self
    }

    /// Replaces the current rows with JSON values validated by [`Self::build`].
    pub fn rows<I, R>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<Value>,
    {
        self.rows = values.into_iter().map(Into::into).collect();
        self
    }

    /// Appends one JSON value validated as a row by [`Self::build`].
    pub fn row(mut self, value: impl Into<Value>) -> Self {
        self.rows.push(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<InsertRequest> {
        required("collection_name", &self.value.collection_name)?;
        match (self.value.columns.is_empty(), self.rows.is_empty()) {
            (false, false) => Err(Error::validation(
                "data".into(),
                "columns and rows cannot both be provided".into(),
            )),
            (true, true) => Err(Error::validation(
                "data".into(),
                "either non-empty columns or rows must be provided".into(),
            )),
            _ => {
                let rows = self
                    .rows
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| {
                        let Value::Object(row) = value else {
                            return Err(Error::validation(
                                format!("rows[{index}]"),
                                "must be a JSON object".into(),
                            ));
                        };
                        Ok(row)
                    })
                    .collect::<Result<_>>()?;
                let mut value = self.value;
                value.rows = rows;
                Ok(value)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpsertRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 upsert operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct UpsertRequest {
    pub(crate) insert: InsertRequest,
    pub(crate) partial_update: bool,
    pub(crate) field_ops: Vec<FieldPartialUpdateOp>,
}

impl UpsertRequest {
    fn empty() -> Self {
        Self {
            insert: InsertRequest::empty(),
            partial_update: Default::default(),
            field_ops: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> UpsertRequestBuilder {
        UpsertRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UpsertRequestBuilder {
        UpsertRequestBuilder { value: self }
    }

    /// Returns the insert.
    pub fn insert(&self) -> &InsertRequest {
        &self.insert
    }

    /// Returns whether partial update.
    pub fn is_partial_update(&self) -> bool {
        self.partial_update
            || self
                .field_ops
                .iter()
                .any(|operation| operation.op_type != FieldPartialUpdateOpType::Replace)
    }

    /// Returns the field ops.
    pub fn field_ops(&self) -> &[FieldPartialUpdateOp] {
        &self.field_ops
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpsertRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UpsertRequest.
#[derive(Debug, Clone)]
pub struct UpsertRequestBuilder {
    value: UpsertRequest,
}

impl UpsertRequestBuilder {
    /// Sets the insert and returns the updated value.
    pub fn insert(mut self, value: InsertRequest) -> Self {
        self.value.insert = value;
        self
    }

    /// Sets the partial update and returns the updated value.
    pub fn partial_update(mut self, value: bool) -> Self {
        self.value.partial_update = value;
        self
    }

    /// Sets the field ops and returns the updated value.
    pub fn field_ops(mut self, value: Vec<FieldPartialUpdateOp>) -> Self {
        self.value.field_ops = value;
        self
    }

    /// Adds one add field op to the existing values.
    pub fn add_field_op(mut self, value: FieldPartialUpdateOp) -> Self {
        self.value.field_ops.push(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<UpsertRequest> {
        required("collection_name", &self.value.insert.collection_name)?;
        for operation in &self.value.field_ops {
            required("field_ops.field_name", operation.get_field_name())?;
        }
        match (
            self.value.insert.columns.is_empty(),
            self.value.insert.rows.is_empty(),
        ) {
            (false, false) => Err(Error::validation(
                "data".into(),
                "columns and rows cannot both be provided".into(),
            )),
            (true, true) => Err(Error::validation(
                "data".into(),
                "either non-empty columns or rows must be provided".into(),
            )),
            _ => Ok(self.value),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DeleteRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 delete operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DeleteRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_name: String,
    pub(crate) filter: String,
    pub(crate) filter_templates: HashMap<String, Value>,
    pub(crate) ids: Ids,
}

impl DeleteRequest {
    /// Creates a builder for this request.
    pub fn builder() -> DeleteRequestBuilder {
        DeleteRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DeleteRequestBuilder {
        DeleteRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition name.
    pub fn partition_name(&self) -> &str {
        &self.partition_name
    }

    /// Returns the filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns the filter templates.
    pub fn filter_templates(&self) -> &HashMap<String, Value> {
        &self.filter_templates
    }

    /// Returns the ids.
    pub fn ids(&self) -> &Ids {
        &self.ids
    }

    pub(crate) fn has_ids(&self) -> bool {
        !self.ids.is_empty()
    }

    pub(crate) fn into_proto(
        self,
        default_db: &str,
        primary_field_name: Option<&str>,
    ) -> Result<milvus::DeleteRequest> {
        let (expr, expr_template_values) = if !self.filter.is_empty() {
            (
                self.filter,
                self.filter_templates
                    .into_iter()
                    .map(|(key, value)| Ok((key, json_template(value)?)))
                    .collect::<Result<_>>()?,
            )
        } else {
            let primary_field_name = primary_field_name.ok_or_else(|| {
                Error::MalformedResponse("collection schema has no primary key".into())
            })?;
            (
                format!("{primary_field_name} in {{ids}}"),
                [("ids".to_owned(), json_template(self.ids.into_json())?)]
                    .into_iter()
                    .collect(),
            )
        };
        Ok(milvus::DeleteRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_name: self.partition_name,
            expr,
            hash_keys: Vec::new(),
            consistency_level: crate::proto::common::ConsistencyLevel::Strong as i32,
            expr_template_values,
            ..Default::default()
        })
    }
}

impl DeleteRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            partition_name: String::new(),
            filter: String::new(),
            filter_templates: HashMap::new(),
            ids: Ids::default(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DeleteRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DeleteRequest.
#[derive(Debug, Clone)]
pub struct DeleteRequestBuilder {
    value: DeleteRequest,
}

impl DeleteRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        self.value.partition_name = value.into();
        self
    }

    /// Sets the filter and returns the updated value.
    pub fn filter(mut self, value: impl Into<String>) -> Self {
        self.value.filter = value.into();
        self
    }

    /// Sets the filter templates and returns the updated value.
    pub fn filter_templates(mut self, value: HashMap<String, Value>) -> Self {
        self.value.filter_templates = value;
        self
    }

    /// Sets the ids and returns the updated value.
    pub fn ids(mut self, value: Ids) -> Self {
        self.value.ids = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DeleteRequest> {
        required("collection_name", &self.value.collection_name)?;
        match (self.value.filter.is_empty(), self.value.ids.is_empty()) {
            (true, true) => Err(Error::validation(
                "condition".into(),
                "deletion condition must be specified by primary keys or filter".into(),
            )),
            (false, false) => Err(Error::validation(
                "condition".into(),
                "only one deletion condition can be specified".into(),
            )),
            _ => Ok(self.value),
        }
    }
}

pub(crate) fn validate_columns(columns: &[FieldData]) -> Result<usize> {
    let count = columns.first().map(FieldData::len).unwrap_or(0);
    if count == 0 {
        return Err(Error::validation(
            "columns".into(),
            "columns must contain at least one row".into(),
        ));
    }
    if columns.iter().any(|column| column.len() != count) {
        return Err(Error::validation(
            "columns".into(),
            "all columns must have the same row count".into(),
        ));
    }
    Ok(count)
}

pub(crate) fn json_template(value: Value) -> Result<crate::proto::schema::TemplateValue> {
    use crate::proto::schema::{
        template_value, BoolArray, DoubleArray, LongArray, StringArray, TemplateArrayValue,
        TemplateValue,
    };
    let value = match value {
        Value::Bool(value) => template_value::Val::BoolVal(value),
        Value::Number(value) if value.is_i64() => {
            template_value::Val::Int64Val(value.as_i64().unwrap())
        }
        Value::Number(value) if value.is_u64() => {
            return Err(Error::validation(
                "filter_template".into(),
                format!("unsigned integer {value} exceeds i64::MAX"),
            ));
        }
        Value::Number(value) => {
            template_value::Val::FloatVal(value.as_f64().ok_or_else(|| {
                Error::conversion("filter template number cannot be represented as f64")
            })?)
        }
        Value::String(value) => template_value::Val::StringVal(value),
        Value::Array(values) if values.iter().all(Value::is_boolean) => {
            template_value::Val::ArrayVal(TemplateArrayValue {
                data: Some(crate::proto::schema::template_array_value::Data::BoolData(
                    BoolArray {
                        data: values.into_iter().map(|v| v.as_bool().unwrap()).collect(),
                    },
                )),
            })
        }
        Value::Array(values) if values.iter().all(|v| v.is_i64()) => {
            template_value::Val::ArrayVal(TemplateArrayValue {
                data: Some(crate::proto::schema::template_array_value::Data::LongData(
                    LongArray {
                        data: values.into_iter().map(|v| v.as_i64().unwrap()).collect(),
                    },
                )),
            })
        }
        Value::Array(values)
            if values.iter().any(|value| {
                matches!(value, Value::Number(number) if number.is_u64() && !number.is_i64())
            }) =>
        {
            return Err(Error::validation(
                "filter_template".into(),
                "unsigned integer in array exceeds i64::MAX".into(),
            ));
        }
        Value::Array(values) if values.iter().all(Value::is_number) => {
            template_value::Val::ArrayVal(TemplateArrayValue {
                data: Some(
                    crate::proto::schema::template_array_value::Data::DoubleData(DoubleArray {
                        data: values.into_iter().map(|v| v.as_f64().unwrap()).collect(),
                    }),
                ),
            })
        }
        Value::Array(values) if values.iter().all(Value::is_string) => {
            template_value::Val::ArrayVal(TemplateArrayValue {
                data: Some(
                    crate::proto::schema::template_array_value::Data::StringData(StringArray {
                        data: values
                            .into_iter()
                            .map(|v| v.as_str().unwrap().to_owned())
                            .collect(),
                    }),
                ),
            })
        }
        other => {
            return Err(Error::validation(
                "filter_template".into(),
                format!("unsupported JSON template value: {other}"),
            ))
        }
    };
    Ok(TemplateValue { val: Some(value) })
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod helper_method_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shared_dml_helpers_are_exercised() {
        assert_eq!(
            validate_columns(&[FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            }])
            .unwrap(),
            2
        );
        assert!(json_template(json!([1, 2])).is_ok());
    }

    #[test]
    fn validate_columns_rejects_zero_rows() {
        assert!(matches!(
            validate_columns(&[FieldData::Int64 {
                name: "id".into(),
                values: Vec::new(),
            }]),
            Err(Error::Validation(error))
                if error.parameter() == "columns"
                    && error.reason() == "columns must contain at least one row"
        ));
    }

    #[test]
    fn json_template_rejects_unsigned_integers_above_i64_max() {
        use crate::proto::schema::template_value;

        let boundary = json_template(json!(i64::MAX as u64)).unwrap();
        assert!(matches!(
            boundary.val,
            Some(template_value::Val::Int64Val(i64::MAX))
        ));

        let out_of_range = i64::MAX as u64 + 1;
        assert!(matches!(
            json_template(json!(out_of_range)),
            Err(Error::Validation(error)) if error.parameter() == "filter_template"
        ));
        assert!(matches!(
            json_template(json!([1, out_of_range, 2.5])),
            Err(Error::Validation(error)) if error.parameter() == "filter_template"
        ));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod insert_request_tests {
    use super::{EntityRow, InsertRequest};
    use crate::v2::error::Error;
    use crate::v2::types::FieldData;
    use serde_json::json;

    #[test]
    fn build_rejects_columns_and_rows_together() {
        let result = InsertRequest::builder()
            .collection_name("books")
            .rows(vec![EntityRow::new()])
            .columns(vec![FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            }])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn build_accepts_exactly_one_input_representation() {
        let request = InsertRequest::builder()
            .collection_name("books")
            .columns(vec![FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            }])
            .build()
            .unwrap();
        assert_eq!(request.columns.len(), 1);
        assert!(request.rows.is_empty());

        let request = InsertRequest::builder()
            .collection_name("books")
            .rows(vec![EntityRow::new()])
            .build()
            .unwrap();
        assert!(request.columns.is_empty());
        assert_eq!(request.rows.len(), 1);

        let request = InsertRequest::builder()
            .collection_name("books")
            .columns(Vec::new())
            .rows(vec![EntityRow::new()])
            .build()
            .unwrap();
        assert!(request.columns.is_empty());
        assert_eq!(request.rows.len(), 1);
    }

    #[test]
    fn build_rejects_missing_or_empty_input() {
        assert!(InsertRequest::builder()
            .collection_name("books")
            .build()
            .is_err());
        assert!(InsertRequest::builder()
            .collection_name("books")
            .columns(Vec::new())
            .build()
            .is_err());
        assert!(InsertRequest::builder()
            .collection_name("books")
            .rows(Vec::<EntityRow>::new())
            .build()
            .is_err());
    }

    #[test]
    fn row_appends_values_and_build_validates_json_objects() {
        let request = InsertRequest::builder()
            .collection_name("books")
            .row(json!({"id": 1, "title": "Rust"}))
            .row(json!({"id": 2, "title": "Milvus"}))
            .build()
            .unwrap();

        assert_eq!(request.rows().len(), 2);
        assert_eq!(request.rows()[0]["id"], json!(1));
        assert_eq!(request.rows()[1]["title"], json!("Milvus"));

        for value in [json!(null), json!(1), json!([1, 2])] {
            assert!(matches!(
                InsertRequest::builder()
                    .collection_name("books")
                    .row(value)
                    .build(),
                Err(Error::Validation(error))
                    if error.parameter() == "rows[0]"
                        && error.reason() == "must be a JSON object"
            ));
        }
    }

    #[test]
    fn rows_accepts_entity_rows_and_json_values_and_validates_in_build() {
        let entity_row = EntityRow::from_iter([("id".into(), json!(0))]);
        let request = InsertRequest::builder()
            .collection_name("books")
            .rows([entity_row])
            .build()
            .unwrap();
        assert_eq!(request.rows()[0]["id"], json!(0));

        let request = InsertRequest::builder()
            .collection_name("books")
            .rows([
                json!({"id": 1, "title": "Rust"}),
                json!({"id": 2, "title": "Milvus"}),
            ])
            .build()
            .unwrap();

        assert_eq!(request.rows().len(), 2);
        assert_eq!(request.rows()[0]["title"], json!("Rust"));
        assert_eq!(request.rows()[1]["id"], json!(2));

        assert!(matches!(
            InsertRequest::builder()
                .collection_name("books")
                .rows([json!({"id": 1}), json!(2)])
                .build(),
            Err(Error::Validation(error))
                if error.parameter() == "rows[1]"
                    && error.reason() == "must be a JSON object"
        ));
    }

    #[test]
    fn into_builder_preserves_validated_rows() {
        let request = InsertRequest::builder()
            .collection_name("books")
            .row(json!({"id": 1, "title": "Rust"}))
            .build()
            .unwrap()
            .into_builder()
            .partition_name("featured")
            .build()
            .unwrap();

        assert_eq!(request.partition_name(), "featured");
        assert_eq!(request.rows()[0]["id"], json!(1));
        assert_eq!(request.rows()[0]["title"], json!("Rust"));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod delete_request_tests {
    use super::DeleteRequest;
    use crate::proto::schema::{template_array_value, template_value};
    use crate::v2::types::Ids;

    #[test]
    fn build_requires_exactly_one_deletion_condition() {
        assert!(DeleteRequest::builder()
            .collection_name("books")
            .build()
            .is_err());
        assert!(DeleteRequest::builder()
            .collection_name("books")
            .filter("id > 0")
            .ids(Ids::Int64(vec![1]))
            .build()
            .is_err());
        assert!(DeleteRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(Vec::new()))
            .build()
            .is_err());
        assert!(DeleteRequest::builder()
            .collection_name("books")
            .filter("id > 0")
            .build()
            .is_ok());
    }

    #[test]
    fn ids_encode_as_primary_key_filter_template() {
        let int_request = DeleteRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(vec![1, 2]))
            .build()
            .unwrap()
            .into_proto("default", Some("id"))
            .unwrap();
        assert_eq!(int_request.expr, "id in {ids}");
        let Some(template_value::Val::ArrayVal(values)) =
            &int_request.expr_template_values["ids"].val
        else {
            panic!("expected ID array template");
        };
        assert!(matches!(
            &values.data,
            Some(template_array_value::Data::LongData(values)) if values.data == vec![1, 2]
        ));

        let string_request = DeleteRequest::builder()
            .collection_name("books")
            .ids(Ids::VarChar(vec!["a".into(), "b".into()]))
            .build()
            .unwrap()
            .into_proto("default", Some("key"))
            .unwrap();
        assert_eq!(string_request.expr, "key in {ids}");
        let Some(template_value::Val::ArrayVal(values)) =
            &string_request.expr_template_values["ids"].val
        else {
            panic!("expected ID array template");
        };
        assert!(matches!(
            &values.data,
            Some(template_array_value::Data::StringData(values))
                if values.data == vec!["a".to_owned(), "b".to_owned()]
        ));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn insert_request_default_values() {
        let value = InsertRequest::empty();

        assert!(value.database_name().is_none());
        assert!(value.collection_name().is_empty());
        assert!(value.partition_name().is_empty());
        assert!(value.columns().is_empty());
        assert!(value.rows().is_empty());
        assert!(InsertRequest::builder().build().is_err());
    }

    #[test]
    fn insert_request_populated_values() {
        let columns = vec![FieldData::Int64 {
            name: "id".to_owned(),
            values: vec![1, 2],
        }];
        let value = InsertRequest::builder()
            .database_name("database")
            .collection_name("collection")
            .partition_name("partition")
            .columns(columns.clone())
            .build()
            .expect("valid column input");

        assert_eq!(
            value.database_name().as_deref().to_owned(),
            Some("database")
        );
        assert_eq!(value.collection_name().to_owned(), "collection");
        assert_eq!(value.partition_name().to_owned(), "partition");
        assert_eq!(value.columns().to_owned(), columns);
        assert!(value.rows().is_empty());
    }

    #[test]
    fn delete_request_default_values() {
        let value = DeleteRequest::empty();

        assert!(value.database_name().is_none());
        assert!(value.collection_name().is_empty());
        assert!(value.partition_name().is_empty());
        assert!(value.filter().is_empty());
        assert!(value.filter_templates().is_empty());
        assert!(value.ids().is_empty());
        assert!(DeleteRequest::builder().build().is_err());
    }

    #[test]
    fn delete_request_populated_values() {
        let filter_templates = HashMap::from([("minimum".to_owned(), serde_json::json!(10))]);
        let value = DeleteRequest::builder()
            .database_name("database")
            .collection_name("collection")
            .partition_name("partition")
            .filter("id > {minimum}")
            .filter_templates(filter_templates.clone())
            .build()
            .expect("valid filter input");

        assert_eq!(
            value.database_name().as_deref().to_owned(),
            Some("database")
        );
        assert_eq!(value.collection_name().to_owned(), "collection");
        assert_eq!(value.partition_name().to_owned(), "partition");
        assert_eq!(value.filter().to_owned(), "id > {minimum}");
        assert_eq!(value.filter_templates().to_owned(), filter_templates);
        assert!(value.ids().is_empty());
    }

    #[test]
    fn upsert_request_default_values() {
        let value = UpsertRequest::empty();
        let expected_insert = InsertRequest::empty();
        let expected_partial_update: bool = false;
        let expected_field_ops: Vec<FieldPartialUpdateOp> = Vec::new();

        assert_eq!(value.insert().to_owned(), expected_insert);
        assert_eq!(
            value.is_partial_update().to_owned(),
            expected_partial_update
        );
        assert_eq!(value.field_ops(), expected_field_ops);
    }

    #[test]
    fn upsert_request_populated_values() {
        let insert = InsertRequest::builder()
            .collection_name("books")
            .rows(vec![EntityRow::new()])
            .build()
            .expect("valid insert request");
        let partial_update = true;
        let field_ops = vec![FieldPartialUpdateOp::new()
            .field_name("tags")
            .op_type(FieldPartialUpdateOpType::ArrayAppend)];
        let value = UpsertRequest::builder()
            .insert(insert.clone())
            .partial_update(partial_update.clone())
            .field_ops(field_ops.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.insert().to_owned(), insert);
        assert_eq!(value.is_partial_update().to_owned(), partial_update);
        assert_eq!(value.field_ops(), field_ops);
    }

    #[test]
    fn non_replace_field_operation_implicitly_enables_partial_update() {
        let insert = InsertRequest::builder()
            .collection_name("books")
            .rows(vec![EntityRow::from_iter([
                ("id".to_owned(), serde_json::json!(1)),
                ("tags".to_owned(), serde_json::json!(["new"])),
            ])])
            .build()
            .expect("valid insert request");
        let value = UpsertRequest::builder()
            .insert(insert)
            .add_field_op(
                FieldPartialUpdateOp::new()
                    .field_name("tags")
                    .op_type(FieldPartialUpdateOpType::ArrayAppend),
            )
            .build()
            .expect("valid upsert request");

        assert!(value.is_partial_update());
    }

    #[test]
    fn field_operation_requires_a_field_name() {
        let insert = InsertRequest::builder()
            .collection_name("books")
            .rows(vec![EntityRow::new()])
            .build()
            .expect("valid insert request");

        assert!(UpsertRequest::builder()
            .insert(insert)
            .add_field_op(FieldPartialUpdateOp::new())
            .build()
            .is_err());
    }
}
