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

//! Internal schema caching, validation, and request conversion helpers.

use super::cache::{COLLECTION_TS_CACHE, SCHEMA_CACHE};
use super::ClientV2;
use crate::proto::{common, milvus, schema};
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::request;
use crate::v2::types::{
    ConsistencyLevel, DataType, FieldData, FieldPartialUpdateOp, FieldPartialUpdateOpType,
};
use crate::v2::utils::{array_f32_to_bf16, array_f32_to_f16};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tonic::Request;

pub(super) const ALLOW_INSERT_AUTO_ID: &str = "allow_insert_auto_id";

///////////////////////////////////////////////////////////////////////////////
// ResolvedData
///////////////////////////////////////////////////////////////////////////////
pub(super) struct ResolvedData {
    description: Arc<milvus::DescribeCollectionResponse>,
    pub(super) canonical_collection_name: String,
    pub(super) row_count: usize,
    pub(super) schema_timestamp: u64,
}

impl ResolvedData {
    pub(super) fn to_proto_fields(
        &self,
        columns: &[FieldData],
        rows: &[request::dml::EntityRow],
        is_upsert: bool,
        partial_update: bool,
    ) -> Result<Vec<schema::FieldData>> {
        let collection_schema = self.description.schema.as_ref().ok_or_else(|| {
            Error::MalformedResponse("describe collection returned no schema".into())
        })?;
        if columns.is_empty() {
            let columns = rows_to_columns(rows, collection_schema, is_upsert, partial_update)?;
            columns_to_proto(columns, collection_schema)
        } else {
            columns_to_proto_by_ref(columns, collection_schema)
        }
    }
}

impl ClientV2 {
    pub(super) fn effective_database(&self, requested: Option<&str>) -> String {
        requested
            .filter(|database| !database.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| self.current_database())
    }

    pub(super) fn rpc_request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);
        let timeout = *self.rpc_timeout.read();
        if !timeout.is_zero() {
            request.set_timeout(timeout);
        }
        request
    }

    pub(super) fn status(&self, status: common::Status) -> Result<()> {
        status_to_result(&Some(status))
    }

    pub(super) fn update_dml_timestamp(&self, database: &str, collection: &str, timestamp: u64) {
        COLLECTION_TS_CACHE.set(&self.cache_endpoint, database, collection, timestamp);
    }

    pub(super) fn remove_dml_timestamp(&self, database: &str, collection: &str) {
        COLLECTION_TS_CACHE.invalidate(&self.cache_endpoint, database, collection);
    }

    pub(super) fn copy_dml_timestamp(
        &self,
        database: &str,
        source_collection: &str,
        target_collection: &str,
    ) {
        COLLECTION_TS_CACHE.copy(
            &self.cache_endpoint,
            database,
            source_collection,
            target_collection,
        );
    }

    pub(super) async fn deduce_guarantee_timestamp(
        &self,
        database: &str,
        collection: &str,
        explicit: Option<ConsistencyLevel>,
    ) -> Result<u64> {
        let canonical_collection = if matches!(explicit, Some(ConsistencyLevel::Session) | None) {
            let description = self
                .get_collection_description(database, collection)
                .await?;
            canonical_collection_name(&description, collection)
        } else {
            collection.to_owned()
        };
        match explicit {
            Some(level) => Ok(COLLECTION_TS_CACHE.guarantee_timestamp(
                &self.cache_endpoint,
                database,
                &canonical_collection,
                level,
            )),
            None => Ok(COLLECTION_TS_CACHE
                .get(&self.cache_endpoint, database, &canonical_collection)
                .unwrap_or(1)),
        }
    }

    pub(super) async fn get_collection_description(
        &self,
        database: &str,
        collection: &str,
    ) -> Result<Arc<milvus::DescribeCollectionResponse>> {
        self.get_collection_description_with_force(database, collection, false)
            .await
    }

    pub(super) async fn describe_collection_uncached(
        &self,
        database: &str,
        collection: &str,
    ) -> Result<milvus::DescribeCollectionResponse> {
        let response = rpc_with_retry!(
            self,
            describe_collection,
            milvus::DescribeCollectionRequest {
                base: None,
                db_name: database.to_owned(),
                collection_name: collection.to_owned(),
                collection_id: 0,
                time_stamp: 0,
            }
        )?;
        status_to_result(&response.status)?;
        Ok(response)
    }

    async fn get_collection_description_with_force(
        &self,
        database: &str,
        collection: &str,
        force_update: bool,
    ) -> Result<Arc<milvus::DescribeCollectionResponse>> {
        SCHEMA_CACHE
            .get_or_load(
                &self.cache_endpoint,
                database,
                collection,
                force_update,
                &self.schema_load_scope,
                || self.describe_collection_uncached(database, collection),
            )
            .await
    }

    pub(super) async fn resolve_data(
        &self,
        database: &str,
        collection: &str,
        columns: &[FieldData],
        rows: &[request::dml::EntityRow],
        is_upsert: bool,
        partial_update: bool,
        field_ops: &[FieldPartialUpdateOp],
    ) -> Result<ResolvedData> {
        if columns.is_empty() == rows.is_empty() {
            return Err(Error::validation(
                "data".into(),
                "exactly one of columns or rows must be provided".into(),
            ));
        }
        let description = self
            .get_collection_description(database, collection)
            .await?;
        let resolved = validate_data_with_description(
            columns,
            rows,
            &description,
            is_upsert,
            partial_update,
            field_ops,
        );
        if !matches!(&resolved, Err(Error::Validation(_))) {
            return resolved.map(|row_count| ResolvedData {
                canonical_collection_name: canonical_collection_name(&description, collection),
                description: Arc::clone(&description),
                row_count,
                schema_timestamp: description.update_timestamp,
            });
        }

        let description = self
            .get_collection_description_with_force(database, collection, true)
            .await?;
        let row_count = validate_data_with_description(
            columns,
            rows,
            &description,
            is_upsert,
            partial_update,
            field_ops,
        )?;
        Ok(ResolvedData {
            canonical_collection_name: canonical_collection_name(&description, collection),
            description: Arc::clone(&description),
            row_count,
            schema_timestamp: description.update_timestamp,
        })
    }

    pub(super) fn remove_collection_description(&self, database: &str, collection: &str) {
        SCHEMA_CACHE.invalidate(&self.cache_endpoint, database, collection);
    }

    pub(super) fn remove_collection_cache(&self, database: &str, collection: &str) {
        SCHEMA_CACHE.invalidate(&self.cache_endpoint, database, collection);
        COLLECTION_TS_CACHE.invalidate(&self.cache_endpoint, database, collection);
    }

    pub(super) fn rename_collection_cache(
        &self,
        old_database: &str,
        old_collection: &str,
        new_database: &str,
        new_collection: &str,
    ) {
        SCHEMA_CACHE.invalidate(&self.cache_endpoint, old_database, old_collection);
        SCHEMA_CACHE.invalidate(&self.cache_endpoint, new_database, new_collection);
        COLLECTION_TS_CACHE.move_ts(
            &self.cache_endpoint,
            old_database,
            old_collection,
            new_database,
            new_collection,
        );
    }

    pub(super) fn clear_database_cache(&self, database: &str) {
        SCHEMA_CACHE.invalidate_database(&self.cache_endpoint, database);
        COLLECTION_TS_CACHE.invalidate_database(&self.cache_endpoint, database);
    }
}

fn canonical_collection_name(
    description: &milvus::DescribeCollectionResponse,
    requested: &str,
) -> String {
    if description.collection_name.is_empty() {
        requested.to_owned()
    } else {
        description.collection_name.clone()
    }
}

fn validate_data_with_description(
    columns: &[FieldData],
    rows: &[request::dml::EntityRow],
    description: &milvus::DescribeCollectionResponse,
    is_upsert: bool,
    partial_update: bool,
    field_ops: &[FieldPartialUpdateOp],
) -> Result<usize> {
    let collection_schema = description
        .schema
        .as_ref()
        .ok_or_else(|| Error::MalformedResponse("describe collection returned no schema".into()))?;
    let row_count = if columns.is_empty() {
        let columns = rows_to_columns(rows, collection_schema, is_upsert, partial_update)?;
        let row_count = if partial_update {
            rows.len()
        } else {
            request::dml::validate_columns(&columns)?
        };
        validate_columns_against_schema(&columns, collection_schema, is_upsert, partial_update)?;
        validate_field_partial_update_ops(field_ops, collection_schema, &columns)?;
        row_count
    } else {
        let row_count = request::dml::validate_columns(columns)?;
        validate_columns_against_schema(columns, collection_schema, is_upsert, partial_update)?;
        validate_field_partial_update_ops(field_ops, collection_schema, columns)?;
        row_count
    };
    Ok(row_count)
}

fn validate_field_partial_update_ops(
    operations: &[FieldPartialUpdateOp],
    collection: &schema::CollectionSchema,
    fields: &[FieldData],
) -> Result<()> {
    for operation in operations {
        let field_name = operation.get_field_name();
        if !fields.iter().any(|field| field.name() == field_name) {
            return Err(Error::validation(
                field_name.to_owned(),
                "field operation requires the field to be present in the upsert payload".into(),
            ));
        }

        let field = collection
            .fields
            .iter()
            .find(|field| field.name == field_name);
        let struct_field_exists = collection
            .struct_array_fields
            .iter()
            .any(|field| field.name == field_name);
        if field.is_none() && !struct_field_exists {
            return Err(Error::validation(
                field_name.to_owned(),
                "field operation targets a field that is not present in the collection schema"
                    .into(),
            ));
        }

        if operation.get_op_type() != FieldPartialUpdateOpType::Replace
            && !field.is_some_and(|field| field.data_type == schema::DataType::Array as i32)
        {
            return Err(Error::validation(
                field_name.to_owned(),
                "ARRAY_APPEND and ARRAY_REMOVE operations require an array field".into(),
            ));
        }
    }
    Ok(())
}

fn rows_to_columns(
    rows: &[serde_json::Map<String, Value>],
    collection: &schema::CollectionSchema,
    is_upsert: bool,
    partial_update: bool,
) -> Result<Vec<FieldData>> {
    let mut columns = Vec::new();
    for field in &collection.fields {
        if field.is_dynamic || field.is_function_output {
            continue;
        }
        let field_name = field.name.clone();
        let data_type = schema::DataType::try_from(field.data_type).map_err(|_| {
            Error::conversion(format!(
                "field {field_name:?} has unknown protobuf data type {}",
                field.data_type
            ))
        })?;
        let present_count = rows
            .iter()
            .filter(|row| row.contains_key(&field_name))
            .count();
        let any_present = present_count > 0;
        if partial_update && !field.is_primary_key && !any_present {
            continue;
        }
        if partial_update && !field.is_primary_key && present_count != rows.len() {
            return Err(Error::validation(
                field_name,
                "a partial-update field must be present in every input row or omitted from every input row"
                    .into(),
            ));
        }
        if !any_present
            && (!is_required_input(field, is_upsert)
                || field.nullable
                || field.default_value.is_some())
        {
            continue;
        }

        let default = field_default_json(field)?;
        let mut valid_data = Vec::with_capacity(rows.len());
        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            match row.get(&field_name).filter(|value| !value.is_null()) {
                Some(value) => {
                    valid_data.push(true);
                    values.push(value);
                }
                None => {
                    if let Some(default) = &default {
                        valid_data.push(true);
                        values.push(default);
                    } else if field.nullable {
                        valid_data.push(false);
                    } else {
                        return Err(Error::validation(
                            field_name.clone(),
                            "field is missing or null but is neither nullable nor defaulted".into(),
                        ));
                    }
                }
            }
        }
        let column = json_values_to_field_data(
            &field_name,
            DataType::try_from_proto(data_type)?,
            field,
            values,
        )?;
        let column = if valid_data.iter().all(|valid| *valid) {
            column
        } else {
            FieldData::nullable(column, valid_data)?
        };
        columns.push(column);
    }

    for field in &collection.struct_array_fields {
        let field_name = field.name.clone();
        let present = rows
            .iter()
            .filter(|row| row.contains_key(&field_name))
            .count();
        if partial_update && present == 0 {
            continue;
        }
        if present != rows.len() {
            return Err(Error::validation(
                field_name,
                "struct field must be present in every row".into(),
            ));
        }
        let values = rows
            .iter()
            .map(|row| {
                row.get(&field_name)
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        Error::validation(
                            field_name.clone(),
                            "struct field value must be an array of objects".into(),
                        )
                    })?
                    .iter()
                    .map(|value| {
                        value.as_object().cloned().ok_or_else(|| {
                            Error::validation(
                                field_name.clone(),
                                "every struct element must be an object".into(),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        columns.push(FieldData::Struct {
            name: field_name,
            values,
        });
    }

    let known_fields = collection
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .chain(
            collection
                .struct_array_fields
                .iter()
                .map(|field| field.name.as_str()),
        )
        .collect::<HashSet<_>>();
    let has_extra_fields = rows
        .iter()
        .any(|row| row.keys().any(|name| !known_fields.contains(name.as_str())));
    if has_extra_fields && !collection.enable_dynamic_field {
        return Err(Error::validation(
            "rows".into(),
            "row contains fields that are not present in the collection schema".into(),
        ));
    }
    if collection.enable_dynamic_field && has_extra_fields {
        let values = rows
            .iter()
            .map(|row| {
                Value::Object(
                    row.iter()
                        .filter(|(name, _)| !known_fields.contains(name.as_str()))
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect(),
                )
            })
            .collect();
        columns.push(FieldData::Json {
            name: "$meta".into(),
            values,
        });
    }
    Ok(columns)
}

fn validate_columns_against_schema(
    columns: &[FieldData],
    collection: &schema::CollectionSchema,
    is_upsert: bool,
    partial_update: bool,
) -> Result<()> {
    let fields = collection
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<HashMap<_, _>>();
    let struct_fields = collection
        .struct_array_fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<HashMap<_, _>>();
    let mut provided = HashSet::new();

    for column in columns {
        if !provided.insert(column.name().to_owned()) {
            return Err(Error::validation(
                column.name().to_owned(),
                "field is provided more than once".into(),
            ));
        }
        if struct_fields.contains_key(column.name()) {
            if !matches!(column.inner(), FieldData::Struct { .. }) {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "field data type does not match struct schema".into(),
                ));
            }
            if column.valid_data().is_some() {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "struct array fields cannot be nullable".into(),
                ));
            }
            continue;
        }
        if column.name() == "$meta" {
            if !collection.enable_dynamic_field || column.data_type() != DataType::Json {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "dynamic field requires enabled JSON $meta data".into(),
                ));
            }
            if column
                .as_json()
                .is_some_and(|values| values.iter().any(|value| !value.is_object()))
            {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "dynamic field values must be JSON objects".into(),
                ));
            }
            continue;
        }
        let field = fields.get(column.name()).copied().ok_or_else(|| {
            Error::validation(
                column.name().to_owned(),
                "field is not present in collection schema".into(),
            )
        })?;

        if field.is_primary_key && field.auto_id && !is_upsert && !allows_insert_auto_id(collection)
        {
            return Err(Error::validation(
                column.name().to_owned(),
                "field must not be supplied for this operation".into(),
            ));
        }
        if field.is_function_output {
            return Err(Error::validation(
                column.name().to_owned(),
                "function output fields must not be supplied".into(),
            ));
        }
        if let Some(valid_data) = column.valid_data() {
            if matches!(column.inner(), FieldData::Nullable { .. }) {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "nested nullable field data is not supported".into(),
                ));
            }
            let valid_count = valid_data.iter().filter(|valid| **valid).count();
            if column.inner().len() != valid_count {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "nullable values and validity bitmap have inconsistent lengths".into(),
                ));
            }
            if valid_data.iter().any(|valid| !valid)
                && !field.nullable
                && field.default_value.is_none()
            {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "null values require a nullable field or a schema default".into(),
                ));
            }
        }
        validate_column_type(column, field)?;
        if let Some(element_type) = column.array_element_type() {
            let schema_element = schema::DataType::try_from(field.element_type)
                .ok()
                .and_then(|value| DataType::try_from_proto(value).ok());
            if schema_element != Some(element_type) {
                return Err(Error::validation(
                    column.name().to_owned(),
                    "array element type does not match collection schema".into(),
                ));
            }
        }
        column.validate_value_constraints(field)?;
    }

    for field in &collection.fields {
        if is_required_input(field, is_upsert)
            && !(partial_update && !field.is_primary_key)
            && !field.nullable
            && field.default_value.is_none()
            && !field.is_function_output
            && !provided.contains(&field.name)
        {
            return Err(Error::validation(
                field.name.clone(),
                "required field is missing".into(),
            ));
        }
    }
    for field in &collection.struct_array_fields {
        if !partial_update && !provided.contains(&field.name) {
            return Err(Error::validation(
                field.name.clone(),
                "required struct field is missing".into(),
            ));
        }
    }
    Ok(())
}

fn allows_insert_auto_id(collection: &schema::CollectionSchema) -> bool {
    collection
        .properties
        .iter()
        .find(|property| property.key == ALLOW_INSERT_AUTO_ID)
        .is_some_and(|property| {
            property.value == "1"
                || property.value.eq_ignore_ascii_case("t")
                || property.value.eq_ignore_ascii_case("true")
        })
}

fn is_required_input(field: &schema::FieldSchema, is_upsert: bool) -> bool {
    if field.is_dynamic || field.name == "$meta" || field.is_function_output {
        return false;
    }
    if field.is_primary_key && field.auto_id {
        return is_upsert;
    }
    true
}

fn validate_column_type(column: &FieldData, field: &schema::FieldSchema) -> Result<()> {
    let schema_type = schema::DataType::try_from(field.data_type).map_err(|_| {
        Error::conversion(format!(
            "field {:?} has unknown protobuf data type {}",
            field.name, field.data_type
        ))
    })?;
    let compatible = match (column.data_type(), schema_type) {
        (DataType::VarChar, schema::DataType::String | schema::DataType::VarChar) => true,
        (input, expected) => input.into_proto() == expected,
    };
    if !compatible {
        return Err(Error::validation(
            column.name().to_owned(),
            format!("field data type does not match schema type {schema_type:?}"),
        ));
    }
    if let Some(input_dimension) = column.dimension() {
        let schema_dimension = field
            .type_params
            .iter()
            .find(|pair| pair.key == "dim")
            .and_then(|pair| pair.value.parse::<usize>().ok())
            .ok_or_else(|| {
                Error::validation(field.name.clone(), "vector schema has no valid dim".into())
            })?;
        if input_dimension != schema_dimension {
            return Err(Error::validation(
                field.name.clone(),
                format!("vector dimension {input_dimension} does not match schema dimension {schema_dimension}"),
            ));
        }
    }
    Ok(())
}

fn columns_to_proto(
    columns: Vec<FieldData>,
    collection: &schema::CollectionSchema,
) -> Result<Vec<schema::FieldData>> {
    columns
        .into_iter()
        .map(|column| column_to_proto(column, collection))
        .collect()
}

fn columns_to_proto_by_ref(
    columns: &[FieldData],
    collection: &schema::CollectionSchema,
) -> Result<Vec<schema::FieldData>> {
    columns
        .iter()
        .map(|column| column_to_proto_by_ref(column, collection))
        .collect()
}

fn column_to_proto_by_ref(
    column: &FieldData,
    collection: &schema::CollectionSchema,
) -> Result<schema::FieldData> {
    if let Some(field) = collection
        .struct_array_fields
        .iter()
        .find(|field| field.name == column.name())
    {
        return struct_column_to_proto(column, field);
    }
    if column.name() == "$meta" {
        if !collection.enable_dynamic_field || column.data_type() != DataType::Json {
            return Err(Error::validation(
                column.name().to_owned(),
                "dynamic field requires enabled JSON $meta data".into(),
            ));
        }
        let mut proto = column.to_proto()?;
        proto.r#type = schema::DataType::Json as i32;
        proto.field_id = 0;
        proto.is_dynamic = true;
        return Ok(proto);
    }
    let field = collection
        .fields
        .iter()
        .find(|field| field.name == column.name())
        .ok_or_else(|| {
            Error::validation(
                column.name().to_owned(),
                "field is not present in collection schema".into(),
            )
        })?;
    column.to_proto_with_schema(field)
}

fn column_to_proto(
    column: FieldData,
    collection: &schema::CollectionSchema,
) -> Result<schema::FieldData> {
    if let Some(field) = collection
        .struct_array_fields
        .iter()
        .find(|field| field.name == column.name())
    {
        return struct_column_to_proto(&column, field);
    }
    if column.name() == "$meta" {
        if !collection.enable_dynamic_field || column.data_type() != DataType::Json {
            return Err(Error::validation(
                column.name().to_owned(),
                "dynamic field requires enabled JSON $meta data".into(),
            ));
        }
        let mut proto = column.into_proto()?;
        proto.r#type = schema::DataType::Json as i32;
        proto.field_id = 0;
        proto.is_dynamic = true;
        return Ok(proto);
    }
    let field = collection
        .fields
        .iter()
        .find(|field| field.name == column.name())
        .ok_or_else(|| {
            Error::validation(
                column.name().to_owned(),
                "field is not present in collection schema".into(),
            )
        })?;
    column.into_proto_with_schema(field)
}

fn struct_column_to_proto(
    column: &FieldData,
    struct_schema: &schema::StructArrayFieldSchema,
) -> Result<schema::FieldData> {
    use schema::{field_data, vector_field};

    let FieldData::Struct { name, values } = column else {
        return Err(Error::validation(
            struct_schema.name.clone(),
            "field data type does not match struct schema".into(),
        ));
    };
    let max_capacity = struct_schema
        .type_params
        .iter()
        .chain(
            struct_schema
                .fields
                .iter()
                .flat_map(|field| field.type_params.iter()),
        )
        .find(|pair| pair.key == "max_capacity")
        .and_then(|pair| pair.value.parse::<usize>().ok());
    if let Some(max_capacity) = max_capacity {
        if values.iter().any(|row| row.len() > max_capacity) {
            return Err(Error::validation(
                name.clone(),
                format!("struct array exceeds max_capacity {max_capacity}"),
            ));
        }
    }

    let mut proto_fields = Vec::new();
    for sub_schema in &struct_schema.fields {
        if sub_schema.is_function_output {
            continue;
        }
        let element_type = schema::DataType::try_from(sub_schema.element_type)
            .or_else(|_| schema::DataType::try_from(sub_schema.data_type))
            .map_err(|_| {
                Error::conversion(format!(
                    "struct subfield {:?} has unknown protobuf element type {} and data type {}",
                    sub_schema.name, sub_schema.element_type, sub_schema.data_type
                ))
            })?;
        let sdk_type = DataType::try_from_proto(element_type)?;
        let mut element_schema = sub_schema.clone();
        element_schema.data_type = element_type as i32;
        let mut scalar_rows = Vec::new();
        let mut vector_rows = Vec::new();
        let vector_dimension = if sdk_type.is_vector() {
            Some(
                sub_schema
                    .type_params
                    .iter()
                    .find(|pair| pair.key == "dim")
                    .and_then(|pair| pair.value.parse::<i64>().ok())
                    .filter(|dimension| *dimension > 0)
                    .ok_or_else(|| {
                        Error::validation(
                            sub_schema.name.clone(),
                            "vector schema has no valid dim".into(),
                        )
                    })?,
            )
        } else {
            None
        };
        let mut valid_data = Vec::new();
        let default = field_default_json(&element_schema)?;

        for row in values {
            let mut nested_values = Vec::with_capacity(row.len());
            let mut row_valid_data = Vec::with_capacity(row.len());
            for value in row {
                match value
                    .get(&sub_schema.name)
                    .filter(|nested| !nested.is_null())
                {
                    Some(nested) => {
                        row_valid_data.push(true);
                        nested_values.push(nested);
                    }
                    None => {
                        if let Some(default) = &default {
                            row_valid_data.push(true);
                            nested_values.push(default);
                        } else if sub_schema.nullable {
                            row_valid_data.push(false);
                        } else {
                            return Err(Error::validation(
                                format!("{}[{}]", struct_schema.name, sub_schema.name),
                                "struct element is missing or null but the subfield is neither nullable nor defaulted"
                                    .into(),
                            ));
                        }
                    }
                }
            }
            let nested = json_values_to_field_data(
                &sub_schema.name,
                sdk_type,
                &element_schema,
                nested_values,
            )?;
            let nested = if row_valid_data.iter().all(|valid| *valid) {
                nested
            } else {
                FieldData::nullable(nested, row_valid_data.clone())?
            };
            if sdk_type.is_vector() {
                validate_column_type(&nested, &element_schema)?;
            }
            nested.validate_value_constraints(&element_schema)?;
            let nested = nested.into_proto_with_schema(&element_schema)?;
            valid_data.extend(row_valid_data);
            match nested.field {
                Some(field_data::Field::Scalars(scalars)) => scalar_rows.push(scalars),
                Some(field_data::Field::Vectors(vectors)) => vector_rows.push(vectors),
                _ => {
                    return Err(Error::validation(
                        sub_schema.name.clone(),
                        "nested struct values must be scalar or vector data".into(),
                    ))
                }
            }
        }

        let field = if !vector_rows.is_empty() || sdk_type.is_vector() {
            let dimension = vector_dimension.unwrap_or_default();
            field_data::Field::Vectors(schema::VectorField {
                dim: dimension,
                data: Some(vector_field::Data::VectorArray(schema::VectorArray {
                    dim: dimension,
                    data: vector_rows,
                    element_type: element_type as i32,
                })),
            })
        } else {
            field_data::Field::Scalars(schema::ScalarField {
                data: Some(schema::scalar_field::Data::ArrayData(schema::ArrayArray {
                    data: scalar_rows,
                    element_type: element_type as i32,
                })),
            })
        };
        proto_fields.push(schema::FieldData {
            r#type: if sdk_type.is_vector() {
                schema::DataType::ArrayOfVector
            } else {
                schema::DataType::Array
            } as i32,
            field_name: sub_schema.name.clone(),
            field_id: sub_schema.field_id,
            is_dynamic: false,
            valid_data: if valid_data.iter().all(|valid| *valid) {
                Vec::new()
            } else {
                valid_data
            },
            field: Some(field),
        });
    }

    Ok(schema::FieldData {
        r#type: schema::DataType::ArrayOfStruct as i32,
        field_name: name.clone(),
        field_id: struct_schema.field_id,
        is_dynamic: false,
        valid_data: Vec::new(),
        field: Some(field_data::Field::StructArrays(schema::StructArrayField {
            fields: proto_fields,
        })),
    })
}

fn json_values_to_field_data(
    name: &str,
    data_type: DataType,
    field: &schema::FieldSchema,
    values: Vec<&Value>,
) -> Result<FieldData> {
    let invalid = || {
        Error::validation(
            name.to_owned(),
            format!("JSON value does not match {data_type:?}"),
        )
    };
    Ok(match data_type {
        DataType::Bool => FieldData::Bool {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| value.as_bool().ok_or_else(invalid))
                .collect::<Result<_>>()?,
        },
        DataType::Int8 => FieldData::Int8 {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| {
                    value
                        .as_i64()
                        .and_then(|value| i8::try_from(value).ok())
                        .ok_or_else(invalid)
                })
                .collect::<Result<_>>()?,
        },
        DataType::Int16 => FieldData::Int16 {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| {
                    value
                        .as_i64()
                        .and_then(|value| i16::try_from(value).ok())
                        .ok_or_else(invalid)
                })
                .collect::<Result<_>>()?,
        },
        DataType::Int32 => FieldData::Int32 {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| {
                    value
                        .as_i64()
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or_else(invalid)
                })
                .collect::<Result<_>>()?,
        },
        DataType::Int64 => FieldData::Int64 {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| value.as_i64().ok_or_else(invalid))
                .collect::<Result<_>>()?,
        },
        DataType::Float => FieldData::Float {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| json_f32(value).ok_or_else(invalid))
                .collect::<Result<_>>()?,
        },
        DataType::Double => FieldData::Double {
            name: name.into(),
            values: values
                .into_iter()
                .map(|value| value.as_f64().ok_or_else(invalid))
                .collect::<Result<_>>()?,
        },
        DataType::VarChar => FieldData::VarChar {
            name: name.into(),
            values: json_strings(values, invalid)?,
        },
        DataType::Geometry => FieldData::Geometry {
            name: name.into(),
            values: json_strings(values, invalid)?,
        },
        DataType::Timestamptz => FieldData::Timestamptz {
            name: name.into(),
            values: json_strings(values, invalid)?,
        },
        DataType::Json => FieldData::Json {
            name: name.into(),
            values: values.into_iter().cloned().collect(),
        },
        DataType::Array => {
            let element_type = schema::DataType::try_from(field.element_type)
                .map_err(|_| Error::validation(name.into(), "array has no element type".into()))?;
            match DataType::try_from_proto(element_type)? {
                DataType::Bool => FieldData::ArrayBool {
                    name: name.into(),
                    values: json_array_rows(values, Value::as_bool).map_err(|_| invalid())?,
                },
                DataType::Int8 => FieldData::ArrayInt8 {
                    name: name.into(),
                    values: json_array_rows(values, |value| {
                        value.as_i64().and_then(|value| i8::try_from(value).ok())
                    })
                    .map_err(|_| invalid())?,
                },
                DataType::Int16 => FieldData::ArrayInt16 {
                    name: name.into(),
                    values: json_array_rows(values, |value| {
                        value.as_i64().and_then(|value| i16::try_from(value).ok())
                    })
                    .map_err(|_| invalid())?,
                },
                DataType::Int32 => FieldData::ArrayInt32 {
                    name: name.into(),
                    values: json_array_rows(values, |value| {
                        value.as_i64().and_then(|value| i32::try_from(value).ok())
                    })
                    .map_err(|_| invalid())?,
                },
                DataType::Int64 => FieldData::ArrayInt64 {
                    name: name.into(),
                    values: json_array_rows(values, Value::as_i64).map_err(|_| invalid())?,
                },
                DataType::Float => FieldData::ArrayFloat {
                    name: name.into(),
                    values: json_array_rows(values, json_f32).map_err(|_| invalid())?,
                },
                DataType::Double => FieldData::ArrayDouble {
                    name: name.into(),
                    values: json_array_rows(values, Value::as_f64).map_err(|_| invalid())?,
                },
                DataType::VarChar => FieldData::ArrayVarChar {
                    name: name.into(),
                    values: json_array_rows(values, |value| value.as_str().map(str::to_owned))
                        .map_err(|_| invalid())?,
                },
                unsupported => {
                    return Err(Error::validation(
                        name.into(),
                        format!("unsupported array element type {unsupported:?}"),
                    ))
                }
            }
        }
        DataType::FloatVector => FieldData::FloatVector {
            name: name.into(),
            values: values
                .into_iter()
                .map(json_f32_vector)
                .collect::<Result<_>>()?,
        },
        DataType::BinaryVector => FieldData::BinaryVector {
            name: name.into(),
            values: values
                .into_iter()
                .map(json_u8_vector)
                .collect::<Result<_>>()?,
        },
        DataType::Float16Vector => FieldData::Float16Vector {
            name: name.into(),
            values: values
                .into_iter()
                .map(json_f16_vector)
                .collect::<Result<_>>()?,
        },
        DataType::BFloat16Vector => FieldData::BFloat16Vector {
            name: name.into(),
            values: values
                .into_iter()
                .map(json_bf16_vector)
                .collect::<Result<_>>()?,
        },
        DataType::SparseFloatVector => FieldData::SparseFloatVector {
            name: name.into(),
            values: values
                .into_iter()
                .map(json_sparse_vector)
                .collect::<Result<_>>()?,
        },
        DataType::Int8Vector => FieldData::Int8Vector {
            name: name.into(),
            values: values
                .into_iter()
                .map(json_i8_vector)
                .collect::<Result<_>>()?,
        },
        unsupported => {
            return Err(Error::validation(
                name.to_owned(),
                format!("struct subfield type {unsupported:?} is not implemented"),
            ))
        }
    })
}

fn field_default_json(field: &schema::FieldSchema) -> Result<Option<Value>> {
    use schema::value_field::Data;
    let Some(default) = &field.default_value else {
        return Ok(None);
    };
    let data_type = schema::DataType::try_from(field.data_type).map_err(|_| {
        Error::conversion(format!(
            "field {:?} has unknown protobuf data type {}",
            field.name, field.data_type
        ))
    })?;
    let invalid = || {
        Error::validation(
            field.name.clone(),
            format!("default value does not match field type {data_type:?}"),
        )
    };
    let value = match (&default.data, data_type) {
        (Some(Data::BoolData(value)), schema::DataType::Bool) => Value::Bool(*value),
        (
            Some(Data::IntData(value)),
            schema::DataType::Int8 | schema::DataType::Int16 | schema::DataType::Int32,
        ) => serde_json::json!(*value),
        (Some(Data::LongData(value)), schema::DataType::Int64) => serde_json::json!(*value),
        (Some(Data::FloatData(value)), schema::DataType::Float) => serde_json::json!(*value),
        (Some(Data::DoubleData(value)), schema::DataType::Double) => serde_json::json!(*value),
        (
            Some(Data::StringData(value)),
            schema::DataType::VarChar
            | schema::DataType::String
            | schema::DataType::Geometry
            | schema::DataType::Timestamptz,
        ) => Value::String(value.clone()),
        (Some(Data::TimestamptzData(value)), schema::DataType::Timestamptz) => {
            Value::String(value.to_string())
        }
        (Some(Data::StringData(value)), schema::DataType::Json) => {
            serde_json::from_str(value).map_err(|_| invalid())?
        }
        (Some(Data::BytesData(value)), schema::DataType::Json) => {
            serde_json::from_slice(value).map_err(|_| invalid())?
        }
        _ => return Err(invalid()),
    };
    Ok(Some(value))
}

fn json_strings(values: Vec<&Value>, invalid: impl Fn() -> Error) -> Result<Vec<String>> {
    values
        .into_iter()
        .map(|value| value.as_str().map(str::to_owned).ok_or_else(|| invalid()))
        .collect()
}

fn json_array_rows<T>(
    values: Vec<&Value>,
    parse: impl Fn(&Value) -> Option<T>,
) -> Result<Vec<Vec<T>>> {
    values
        .into_iter()
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| Error::conversion("expected a JSON array row"))?
                .iter()
                .map(|value| {
                    parse(value)
                        .ok_or_else(|| Error::conversion("JSON array element has an invalid type"))
                })
                .collect()
        })
        .collect()
}

fn json_f32(value: &Value) -> Option<f32> {
    let value = value.as_f64()?;
    if !value.is_finite() || !(f32::MIN as f64..=f32::MAX as f64).contains(&value) {
        return None;
    }
    let narrowed = value as f32;
    narrowed.is_finite().then_some(narrowed)
}

fn json_f32_vector(value: &Value) -> Result<Vec<f32>> {
    value
        .as_array()
        .ok_or_else(|| Error::conversion("float vector must be a JSON array"))?
        .iter()
        .map(|value| {
            json_f32(value).ok_or_else(|| {
                Error::conversion(
                    "float vector element must be a finite number within the f32 range",
                )
            })
        })
        .collect()
}

fn json_u8_vector(value: &Value) -> Result<Vec<u8>> {
    value
        .as_array()
        .ok_or_else(|| Error::conversion("binary vector must be a JSON byte array"))?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| {
                    Error::conversion("binary vector element must be an integer from 0 through 255")
                })
        })
        .collect()
}

fn json_f16_vector(value: &Value) -> Result<Vec<u16>> {
    let values = json_half_vector(value)?;
    if values
        .iter()
        .any(|value| !(-65504.0..=65504.0).contains(value))
    {
        return Err(Error::validation(
            "float16 vector".into(),
            "value is outside Float16 range [-65504, 65504]".into(),
        ));
    }
    Ok(array_f32_to_f16(&values))
}

fn json_bf16_vector(value: &Value) -> Result<Vec<u16>> {
    Ok(array_f32_to_bf16(&json_half_vector(value)?))
}

fn json_half_vector(value: &Value) -> Result<Vec<f32>> {
    value
        .as_array()
        .ok_or_else(|| Error::conversion("half-precision vector must be a JSON array"))?
        .iter()
        .map(|value| {
            if !value.is_f64() {
                return Err(Error::conversion(
                    "half-precision vector element must be a JSON float",
                ));
            }
            json_f32(value).ok_or_else(|| {
                Error::conversion(
                    "half-precision vector element must be finite and within the f32 range",
                )
            })
        })
        .collect()
}

fn json_i8_vector(value: &Value) -> Result<Vec<i8>> {
    value
        .as_array()
        .ok_or_else(|| Error::conversion("int8 vector must be a JSON array"))?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .and_then(|value| i8::try_from(value).ok())
                .ok_or_else(|| {
                    Error::conversion(
                        "int8 vector element must be an integer from -128 through 127",
                    )
                })
        })
        .collect()
}

fn json_sparse_vector(value: &Value) -> Result<crate::v2::types::SparseVector> {
    let object = value
        .as_object()
        .ok_or_else(|| Error::conversion("sparse vector must be a JSON object"))?;
    if let (Some(indices), Some(values)) = (object.get("indices"), object.get("values")) {
        let indices = indices
            .as_array()
            .ok_or_else(|| Error::conversion("sparse vector indices must be a JSON array"))?;
        let values = values
            .as_array()
            .ok_or_else(|| Error::conversion("sparse vector values must be a JSON array"))?;
        if indices.len() != values.len() {
            return Err(Error::conversion(
                "sparse vector indices and values must have equal lengths",
            ));
        }
        let pairs = indices
            .iter()
            .zip(values)
            .map(|(index, value)| {
                let index = index
                    .as_u64()
                    .and_then(|index| u32::try_from(index).ok())
                    .ok_or_else(|| {
                        Error::conversion("sparse vector index must be an unsigned 32-bit integer")
                    })?;
                let value = json_f32(value).ok_or_else(|| {
                    Error::conversion(
                        "sparse vector value must be a finite number within the f32 range",
                    )
                })?;
                Ok((index, value))
            })
            .collect::<Result<crate::v2::types::SparseVector>>()?;
        if pairs.len() != indices.len() {
            return Err(Error::conversion("sparse vector contains invalid entries"));
        }
        return Ok(pairs);
    }
    object
        .iter()
        .map(|(index, value)| {
            let index = index.parse::<u32>().map_err(|_| {
                Error::conversion("sparse vector object key must be an unsigned 32-bit integer")
            })?;
            let value = json_f32(value).ok_or_else(|| {
                Error::conversion(
                    "sparse vector value must be a finite number within the f32 range",
                )
            })?;
            Ok((index, value))
        })
        .collect()
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{
        columns_to_proto, json_bf16_vector, json_f16_vector, json_f32_vector, json_sparse_vector,
        json_values_to_field_data, rows_to_columns, validate_column_type,
        validate_columns_against_schema, validate_field_partial_update_ops, ALLOW_INSERT_AUTO_ID,
    };
    use crate::proto::{common, schema};
    use crate::v2::types::{FieldData, FieldPartialUpdateOp, FieldPartialUpdateOpType};
    use serde_json::json;

    fn field(id: i64, name: &str, data_type: schema::DataType) -> schema::FieldSchema {
        schema::FieldSchema {
            field_id: id,
            name: name.into(),
            data_type: data_type as i32,
            ..Default::default()
        }
    }

    fn collection_schema() -> schema::CollectionSchema {
        let mut id = field(100, "id", schema::DataType::Int64);
        id.is_primary_key = true;
        let mut vector = field(101, "vector", schema::DataType::FloatVector);
        vector.type_params.push(common::KeyValuePair {
            key: "dim".into(),
            value: "2".into(),
        });
        let mut optional = field(102, "note", schema::DataType::VarChar);
        optional.nullable = true;
        schema::CollectionSchema {
            fields: vec![id, vector, optional],
            ..Default::default()
        }
    }

    #[test]
    fn narrow_integer_columns_and_rows_preserve_schema_types() {
        let int8_schema = field(101, "int8_value", schema::DataType::Int8);
        let int16_schema = field(102, "int16_value", schema::DataType::Int16);

        validate_column_type(
            &FieldData::Int8 {
                name: "int8_value".into(),
                values: vec![-128, 127],
            },
            &int8_schema,
        )
        .unwrap();
        validate_column_type(
            &FieldData::Int16 {
                name: "int16_value".into(),
                values: vec![-32768, 32767],
            },
            &int16_schema,
        )
        .unwrap();
        assert!(validate_column_type(
            &FieldData::Int32 {
                name: "int8_value".into(),
                values: vec![1],
            },
            &int8_schema,
        )
        .is_err());

        let int8_values = [json!(-128), json!(127)];
        let int8 = json_values_to_field_data(
            "int8_value",
            crate::v2::types::DataType::Int8,
            &int8_schema,
            int8_values.iter().collect(),
        )
        .unwrap();
        assert!(matches!(int8, FieldData::Int8 { values, .. } if values == vec![-128, 127]));

        let out_of_range = [json!(128)];
        assert!(json_values_to_field_data(
            "int8_value",
            crate::v2::types::DataType::Int8,
            &int8_schema,
            out_of_range.iter().collect(),
        )
        .is_err());
    }

    #[test]
    fn row_half_vectors_accept_floats_and_convert_to_u16() {
        assert_eq!(
            json_f16_vector(&json!([1.0, -1.0])).unwrap(),
            vec![0x3c00, 0xbc00]
        );
        assert_eq!(
            json_bf16_vector(&json!([1.0, -1.0])).unwrap(),
            vec![0x3f80, 0xbf80]
        );
        assert!(json_f16_vector(&json!([65505.0])).is_err());
        assert!(json_f16_vector(&json!([1])).is_err());
        assert!(json_bf16_vector(&json!([1])).is_err());
    }

    #[test]
    fn row_float_values_reject_f32_overflow() {
        let float_schema = field(101, "value", schema::DataType::Float);
        for value in [json!(1e300), json!(-1e300)] {
            assert!(json_values_to_field_data(
                "value",
                crate::v2::types::DataType::Float,
                &float_schema,
                vec![&value],
            )
            .is_err());
        }

        let mut array_schema = field(102, "values", schema::DataType::Array);
        array_schema.element_type = schema::DataType::Float as i32;
        let array = json!([0.5, 1e300]);
        assert!(json_values_to_field_data(
            "values",
            crate::v2::types::DataType::Array,
            &array_schema,
            vec![&array],
        )
        .is_err());

        assert!(json_f32_vector(&json!([0.5, 1e300])).is_err());
        assert!(json_bf16_vector(&json!([1e300])).is_err());
        assert!(json_sparse_vector(&json!({"1": 1e300})).is_err());
        assert!(json_sparse_vector(&json!({
            "indices": [1],
            "values": [1e300]
        }))
        .is_err());
    }

    #[test]
    fn row_float_values_accept_f32_boundaries() {
        let float_schema = field(101, "value", schema::DataType::Float);
        let values = [json!(f32::MIN as f64), json!(f32::MAX as f64)];
        let data = json_values_to_field_data(
            "value",
            crate::v2::types::DataType::Float,
            &float_schema,
            values.iter().collect(),
        )
        .unwrap();
        assert!(matches!(
            data,
            FieldData::Float { values, .. }
                if values == vec![f32::MIN, f32::MAX]
        ));
        assert_eq!(json_f32_vector(&json!([0.5, -0.5])).unwrap(), [0.5, -0.5]);
    }

    fn extended_collection_schema() -> schema::CollectionSchema {
        let mut id = field(100, "id", schema::DataType::Int64);
        id.is_primary_key = true;
        let location = field(101, "location", schema::DataType::Geometry);
        let observed_at = field(102, "observed_at", schema::DataType::Timestamptz);

        let mut label = field(201, "label", schema::DataType::Array);
        label.element_type = schema::DataType::VarChar as i32;
        label.type_params.push(common::KeyValuePair {
            key: "max_capacity".into(),
            value: "4".into(),
        });
        let mut nested_location = field(202, "location", schema::DataType::Array);
        nested_location.element_type = schema::DataType::Geometry as i32;
        nested_location.type_params.push(common::KeyValuePair {
            key: "max_capacity".into(),
            value: "4".into(),
        });
        let mut nested_time = field(203, "observed_at", schema::DataType::Array);
        nested_time.element_type = schema::DataType::Timestamptz as i32;
        nested_time.type_params.push(common::KeyValuePair {
            key: "max_capacity".into(),
            value: "4".into(),
        });
        let mut embedding = field(204, "embedding", schema::DataType::ArrayOfVector);
        embedding.element_type = schema::DataType::FloatVector as i32;
        embedding.type_params.extend([
            common::KeyValuePair {
                key: "max_capacity".into(),
                value: "4".into(),
            },
            common::KeyValuePair {
                key: "dim".into(),
                value: "2".into(),
            },
        ]);
        schema::CollectionSchema {
            fields: vec![id, location, observed_at],
            struct_array_fields: vec![schema::StructArrayFieldSchema {
                field_id: 200,
                name: "events".into(),
                description: String::new(),
                fields: vec![label, nested_location, nested_time, embedding],
                type_params: Vec::new(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn column_validation_uses_schema_types_dimensions_and_field_ids() {
        let schema = collection_schema();
        let columns = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
            },
        ];
        validate_columns_against_schema(&columns, &schema, false, false).unwrap();
        let proto = columns_to_proto(columns, &schema).unwrap();
        assert_eq!(proto[0].field_id, 100);
        assert_eq!(proto[0].r#type, schema::DataType::Int64 as i32);
        assert_eq!(proto[1].field_id, 101);
        assert_eq!(proto[1].r#type, schema::DataType::FloatVector as i32);
    }

    #[test]
    fn column_validation_rejects_schema_mismatches() {
        let schema = collection_schema();
        let wrong_dimension = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2, 0.3]],
            },
        ];
        assert!(validate_columns_against_schema(&wrong_dimension, &schema, false, false).is_err());

        let unknown = vec![FieldData::Int64 {
            name: "unknown".into(),
            values: vec![1],
        }];
        assert!(validate_columns_against_schema(&unknown, &schema, false, false).is_err());
    }

    #[test]
    fn enabled_dynamic_schema_accepts_and_encodes_meta_without_a_pseudo_field() {
        let mut schema = collection_schema();
        schema.enable_dynamic_field = true;
        let columns = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
            },
            FieldData::Json {
                name: "$meta".into(),
                values: vec![json!({"a": 1}), json!({"a": 2, "b": "value"})],
            },
        ];

        validate_columns_against_schema(&columns, &schema, false, false).unwrap();
        let proto = columns_to_proto(columns, &schema).unwrap();
        let dynamic = proto.last().unwrap();
        assert_eq!(dynamic.field_name, "$meta");
        assert_eq!(dynamic.r#type, schema::DataType::Json as i32);
        assert_eq!(dynamic.field_id, 0);
        assert!(dynamic.is_dynamic);
    }

    #[test]
    fn dynamic_meta_requires_enabled_schema_and_json_objects() {
        let mut schema = collection_schema();
        let dynamic = FieldData::Json {
            name: "$meta".into(),
            values: vec![json!({"a": 1})],
        };
        assert!(
            validate_columns_against_schema(&[dynamic.clone()], &schema, false, false).is_err()
        );

        schema.enable_dynamic_field = true;
        let non_object = FieldData::Json {
            name: "$meta".into(),
            values: vec![json!("not an object")],
        };
        assert!(validate_columns_against_schema(&[non_object], &schema, false, false).is_err());
    }

    #[test]
    fn auto_id_insert_accepts_explicit_primary_keys_only_when_enabled() {
        let mut schema = collection_schema();
        schema.fields[0].auto_id = true;
        let columns = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
            },
        ];

        assert!(validate_columns_against_schema(&columns, &schema, false, false).is_err());
        schema.properties.push(common::KeyValuePair {
            key: ALLOW_INSERT_AUTO_ID.into(),
            value: "False".into(),
        });
        assert!(validate_columns_against_schema(&columns, &schema, false, false).is_err());

        for enabled in ["true", "True", "TRUE", "t", "T", "1"] {
            schema.properties[0].value = enabled.into();
            validate_columns_against_schema(&columns, &schema, false, false).unwrap();
        }
        validate_columns_against_schema(&columns, &schema, true, false).unwrap();

        let rows = vec![
            json!({"id": 1, "vector": [0.1, 0.2]})
                .as_object()
                .unwrap()
                .clone(),
            json!({"id": 2, "vector": [0.3, 0.4]})
                .as_object()
                .unwrap()
                .clone(),
        ];
        let row_columns = rows_to_columns(&rows, &schema, false, false).unwrap();
        validate_columns_against_schema(&row_columns, &schema, false, false).unwrap();
    }

    #[test]
    fn geometry_timestamptz_and_struct_columns_validate_and_encode() {
        let schema = extended_collection_schema();
        let columns = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            FieldData::Geometry {
                name: "location".into(),
                values: vec!["POINT (1 2)".into()],
            },
            FieldData::Timestamptz {
                name: "observed_at".into(),
                values: vec!["2026-07-15T12:30:00+08:00".into()],
            },
            FieldData::Struct {
                name: "events".into(),
                values: vec![vec![[
                    ("label".into(), json!("start")),
                    ("location".into(), json!("POINT (3 4)")),
                    ("observed_at".into(), json!("2026-07-15T12:31:00+08:00")),
                    ("embedding".into(), json!([0.1, 0.2])),
                ]
                .into_iter()
                .collect()]],
            },
        ];

        validate_columns_against_schema(&columns, &schema, false, false).unwrap();
        let proto = columns_to_proto(columns, &schema).unwrap();
        assert_eq!(proto[1].r#type, schema::DataType::Geometry as i32);
        assert_eq!(proto[2].r#type, schema::DataType::Timestamptz as i32);
        assert_eq!(proto[3].r#type, schema::DataType::ArrayOfStruct as i32);
        let Some(schema::field_data::Field::StructArrays(structs)) = &proto[3].field else {
            panic!("expected struct array data");
        };
        assert_eq!(structs.fields.len(), 4);
        assert_eq!(structs.fields[0].r#type, schema::DataType::Array as i32);
        assert_eq!(
            structs.fields[3].r#type,
            schema::DataType::ArrayOfVector as i32
        );
    }

    #[test]
    fn struct_vector_subfields_validate_schema_dimensions() {
        for (element_type, schema_dimension, value) in [
            (schema::DataType::FloatVector, "2", json!([0.1, 0.2, 0.3])),
            (schema::DataType::BinaryVector, "16", json!([170])),
            (schema::DataType::Float16Vector, "2", json!([0.1, 0.2, 0.3])),
            (
                schema::DataType::BFloat16Vector,
                "2",
                json!([0.1, 0.2, 0.3]),
            ),
            (schema::DataType::Int8Vector, "2", json!([1, 2, 3])),
        ] {
            let mut collection = extended_collection_schema();
            let embedding = &mut collection.struct_array_fields[0].fields[3];
            embedding.element_type = element_type as i32;
            embedding
                .type_params
                .iter_mut()
                .find(|pair| pair.key == "dim")
                .expect("embedding dimension")
                .value = schema_dimension.into();
            let column = FieldData::Struct {
                name: "events".into(),
                values: vec![vec![[
                    ("label".into(), json!("start")),
                    ("location".into(), json!("POINT (3 4)")),
                    ("observed_at".into(), json!("2026-07-15T12:31:00+08:00")),
                    ("embedding".into(), value),
                ]
                .into_iter()
                .collect()]],
            };

            let error = columns_to_proto(vec![column], &collection)
                .expect_err("mismatched nested vector dimension must fail");
            assert!(
                error.to_string().contains("vector dimension"),
                "unexpected {element_type:?} validation error: {error}"
            );
        }

        let collection = extended_collection_schema();
        let row = json!({
            "id": 1,
            "location": "POINT (1 2)",
            "observed_at": "2026-07-15T12:30:00+08:00",
            "events": [{
                "label": "start",
                "location": "POINT (3 4)",
                "observed_at": "2026-07-15T12:31:00+08:00",
                "embedding": [0.1, 0.2, 0.3]
            }]
        })
        .as_object()
        .expect("row object")
        .clone();
        let columns = rows_to_columns(&[row], &collection, false, false).unwrap();
        let error = columns_to_proto(columns, &collection)
            .expect_err("row-oriented nested vector dimension mismatch must fail");
        assert!(error.to_string().contains("vector dimension"));
    }

    #[test]
    fn row_input_resolves_geometry_timestamptz_and_struct_values() {
        let schema = extended_collection_schema();
        let row = json!({
            "id": 1,
            "location": "POINT (1 2)",
            "observed_at": "2026-07-15T12:30:00+08:00",
            "events": [{
                "label": "start",
                "location": "POINT (3 4)",
                "observed_at": "2026-07-15T12:31:00+08:00",
                "embedding": [0.1, 0.2]
            }]
        })
        .as_object()
        .unwrap()
        .clone();
        let columns = rows_to_columns(&[row], &schema, false, false).unwrap();
        assert!(matches!(columns[1], FieldData::Geometry { .. }));
        assert!(matches!(columns[2], FieldData::Timestamptz { .. }));
        assert!(matches!(columns[3], FieldData::Struct { .. }));
        columns_to_proto(columns, &schema).unwrap();
    }

    #[test]
    fn struct_subfields_apply_nullable_and_default_values() {
        let mut schema = extended_collection_schema();
        let struct_schema = &mut schema.struct_array_fields[0];
        struct_schema.fields[0].default_value = Some(schema::ValueField {
            data: Some(schema::value_field::Data::StringData("default".into())),
        });
        struct_schema.fields[1].nullable = true;
        let values = vec![vec![[
            ("observed_at".into(), json!("2026-07-15T12:31:00+08:00")),
            ("embedding".into(), json!([0.1, 0.2])),
        ]
        .into_iter()
        .collect()]];
        let proto = columns_to_proto(
            vec![
                FieldData::Int64 {
                    name: "id".into(),
                    values: vec![1],
                },
                FieldData::Geometry {
                    name: "location".into(),
                    values: vec!["POINT (1 1)".into()],
                },
                FieldData::Timestamptz {
                    name: "observed_at".into(),
                    values: vec!["2026-07-15T12:30:00+08:00".into()],
                },
                FieldData::Struct {
                    name: "events".into(),
                    values,
                },
            ],
            &schema,
        )
        .unwrap();
        let Some(schema::field_data::Field::StructArrays(structs)) = &proto[3].field else {
            panic!("expected struct array data");
        };
        assert!(structs.fields[0].valid_data.is_empty());
        assert_eq!(structs.fields[1].valid_data, vec![false]);
        let Some(schema::field_data::Field::Scalars(label)) = &structs.fields[0].field else {
            panic!("expected scalar struct subfield");
        };
        let Some(schema::scalar_field::Data::ArrayData(labels)) = &label.data else {
            panic!("expected array data for struct subfield");
        };
        let Some(schema::scalar_field::Data::StringData(labels)) = &labels.data[0].data else {
            panic!("expected string struct subfield");
        };
        assert_eq!(labels.data, vec!["default"]);
    }

    #[test]
    fn sparse_row_input_accepts_map_and_indices_values_formats() {
        assert_eq!(
            json_sparse_vector(&json!({"1": 0.5, "8": 0.25})).unwrap(),
            [(1, 0.5), (8, 0.25)].into_iter().collect()
        );
        assert_eq!(
            json_sparse_vector(&json!({
                "indices": [1, 8],
                "values": [0.5, 0.25]
            }))
            .unwrap(),
            [(1, 0.5), (8, 0.25)].into_iter().collect()
        );
        assert!(json_sparse_vector(&json!({
            "indices": [1, 1],
            "values": [0.5, 0.25]
        }))
        .is_err());
    }

    #[test]
    fn row_input_allows_nullable_fields_to_be_omitted() {
        let schema = collection_schema();
        let row = json!({
            "id": 1,
            "vector": [0.1, 0.2]
        })
        .as_object()
        .unwrap()
        .clone();

        let columns = rows_to_columns(&[row], &schema, false, false).unwrap();

        assert_eq!(columns.len(), 2);
        assert!(!columns.iter().any(|column| column.name() == "note"));
    }

    #[test]
    fn partial_upsert_rows_allow_omitted_normal_fields() {
        let schema = collection_schema();
        let row = json!({"id": 1}).as_object().unwrap().clone();

        let columns = rows_to_columns(&[row], &schema, true, true).unwrap();

        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].name(), "id");
        validate_columns_against_schema(&columns, &schema, true, true).unwrap();
    }

    #[test]
    fn partial_upsert_rows_require_primary_but_allow_omitted_struct_fields() {
        let schema = collection_schema();
        let row = json!({"vector": [0.1, 0.2]}).as_object().unwrap().clone();
        assert!(rows_to_columns(&[row], &schema, true, true).is_err());

        let schema = extended_collection_schema();
        let row = json!({
            "id": 1,
            "location": "POINT (1 2)"
        })
        .as_object()
        .unwrap()
        .clone();
        let columns = rows_to_columns(&[row], &schema, true, true).unwrap();
        assert!(!columns.iter().any(|column| column.name() == "events"));
        validate_columns_against_schema(&columns, &schema, true, true).unwrap();
    }

    #[test]
    fn row_input_compacts_nulls_and_applies_defaults() {
        let mut schema = collection_schema();
        schema.fields[2].default_value = Some(schema::ValueField {
            data: Some(schema::value_field::Data::StringData("default note".into())),
        });
        let mut tags = field(103, "tags", schema::DataType::Array);
        tags.element_type = schema::DataType::Int64 as i32;
        tags.nullable = true;
        schema.fields.push(tags);

        let rows = vec![
            json!({
                "id": 1,
                "vector": [0.1, 0.2],
                "note": null,
                "tags": [1, 2]
            })
            .as_object()
            .unwrap()
            .clone(),
            json!({
                "id": 2,
                "vector": [0.3, 0.4],
                "note": "explicit",
                "tags": null
            })
            .as_object()
            .unwrap()
            .clone(),
        ];

        let columns = rows_to_columns(&rows, &schema, false, false).unwrap();
        let note = columns
            .iter()
            .find(|column| column.name() == "note")
            .unwrap();
        assert!(note.valid_data().is_none());
        assert!(matches!(
            note,
            FieldData::VarChar { values, .. }
                if values == &vec!["default note".to_owned(), "explicit".to_owned()]
        ));
        let tags = columns
            .iter()
            .find(|column| column.name() == "tags")
            .unwrap();
        assert_eq!(tags.valid_data(), Some([true, false].as_slice()));
        assert!(matches!(
            tags.inner(),
            FieldData::ArrayInt64 { values, .. } if values.len() == 1
        ));

        let proto = columns_to_proto(columns, &schema).unwrap();
        let tags = proto
            .iter()
            .find(|field| field.field_name == "tags")
            .unwrap();
        assert_eq!(tags.valid_data, vec![true, false]);
    }

    #[test]
    fn nullable_column_requires_nullable_or_defaulted_schema() {
        let schema = collection_schema();
        let id = FieldData::nullable(
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1],
            },
            vec![true, false],
        )
        .unwrap();
        assert!(validate_columns_against_schema(&[id], &schema, false, false).is_err());

        let note = FieldData::nullable(
            FieldData::VarChar {
                name: "note".into(),
                values: vec!["present".into()],
            },
            vec![true, false],
        )
        .unwrap();
        let columns = vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::FloatVector {
                name: "vector".into(),
                values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
            },
            note,
        ];
        validate_columns_against_schema(&columns, &schema, false, false).unwrap();
    }

    #[test]
    fn field_partial_update_operations_require_submitted_array_fields() {
        let mut collection = collection_schema();
        let mut tags = field(103, "tags", schema::DataType::Array);
        tags.element_type = schema::DataType::Int64 as i32;
        collection.fields.push(tags);
        let payload = vec![FieldData::ArrayInt64 {
            name: "tags".into(),
            values: vec![vec![1]],
        }];

        let append = FieldPartialUpdateOp::new()
            .field_name("tags")
            .op_type(FieldPartialUpdateOpType::ArrayAppend);
        validate_field_partial_update_ops(&[append], &collection, &payload).unwrap();

        let non_array = FieldPartialUpdateOp::new()
            .field_name("note")
            .op_type(FieldPartialUpdateOpType::ArrayRemove);
        let note_payload = vec![FieldData::VarChar {
            name: "note".into(),
            values: vec!["value".into()],
        }];
        assert!(
            validate_field_partial_update_ops(&[non_array], &collection, &note_payload).is_err()
        );

        let omitted = FieldPartialUpdateOp::new()
            .field_name("tags")
            .op_type(FieldPartialUpdateOpType::ArrayAppend);
        assert!(validate_field_partial_update_ops(&[omitted], &collection, &[]).is_err());
    }
}
