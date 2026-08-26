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

//! Request types for collection lifecycle and schema operations.

use crate::proto::{common, milvus};
use crate::v2::error::{Error, Result};
use crate::v2::request::validation::{non_empty_strings, positive_i32, required};
use crate::v2::types::{
    CollectionSchema, ConsistencyLevel, DataType, FieldSchema, Function, FunctionType, IndexParam,
    IndexType, MetricType, StructFieldSchema,
};
use std::collections::{HashMap, HashSet};

///////////////////////////////////////////////////////////////////////////////
// CreateCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_collection operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CreateCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) description: Option<String>,
    pub(crate) schema: Option<CollectionSchema>,
    pub(crate) num_partitions: i64,
    pub(crate) num_shards: i32,
    pub(crate) consistency_level: ConsistencyLevel,
    pub(crate) index_params: Vec<IndexParam>,
    pub(crate) properties: HashMap<String, String>,
}

impl CreateCollectionRequest {
    /// Creates a builder for this request.
    pub fn builder() -> CreateCollectionRequestBuilder {
        CreateCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateCollectionRequestBuilder {
        CreateCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the description.
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    /// Returns the schema.
    pub fn schema(&self) -> &Option<CollectionSchema> {
        &self.schema
    }

    /// Returns the num partitions.
    pub fn num_partitions(&self) -> i64 {
        self.num_partitions
    }

    /// Returns the num shards.
    pub fn num_shards(&self) -> i32 {
        self.num_shards
    }

    /// Returns the consistency level.
    pub fn consistency_level(&self) -> ConsistencyLevel {
        self.consistency_level
    }

    /// Returns the index params.
    pub fn index_params(&self) -> &[IndexParam] {
        &self.index_params
    }

    /// Returns the properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn into_proto(self, default_db: &str) -> Result<milvus::CreateCollectionRequest> {
        let encoded_schema = self
            .schema
            .ok_or_else(|| Error::validation("schema".into(), "must be specified".into()))?
            .encode_for_collection(&self.collection_name, self.description.as_deref())?;
        Ok(milvus::CreateCollectionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            schema: encoded_schema,
            shards_num: self.num_shards,
            consistency_level: self.consistency_level.into_proto() as i32,
            properties: self
                .properties
                .into_iter()
                .map(|(key, value)| common::KeyValuePair { key, value })
                .collect(),
            num_partitions: self.num_partitions,
            ..Default::default()
        })
    }
}

impl CreateCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            description: None,
            schema: None,
            num_partitions: 0,
            num_shards: 1,
            consistency_level: ConsistencyLevel::Bounded,
            index_params: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

impl From<CreateSimpleCollectionRequest> for CreateCollectionRequest {
    fn from(value: CreateSimpleCollectionRequest) -> Self {
        let mut primary_field = crate::v2::types::FieldSchema::new()
            .name(&value.primary_field)
            .data_type(value.primary_field_type)
            .primary_key(true)
            .auto_id(value.auto_id);
        if value.primary_field_type == DataType::VarChar {
            primary_field = primary_field.max_length(value.max_length);
        }
        let schema = CollectionSchema::new()
            .enable_dynamic_field(value.enable_dynamic_field)
            .add_field(primary_field)
            .add_field(
                crate::v2::types::FieldSchema::new()
                    .name(&value.vector_field)
                    .data_type(crate::v2::types::DataType::FloatVector)
                    .dimension(value.dimension),
            );
        CreateCollectionRequest {
            database_name: value.database_name,
            collection_name: value.collection_name,
            description: None,
            schema: Some(schema),
            num_partitions: 0,
            num_shards: 1,
            consistency_level: value.consistency_level,
            index_params: vec![IndexParam::new()
                .field_name(value.vector_field)
                .index_type(IndexType::AutoIndex)
                .metric_type(value.metric_type)],
            properties: HashMap::new(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateCollectionRequest.
#[derive(Debug, Clone)]
pub struct CreateCollectionRequestBuilder {
    value: CreateCollectionRequest,
}

impl CreateCollectionRequestBuilder {
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

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = Some(value.into());
        self
    }

    /// Sets the schema and returns the updated value.
    pub fn schema(mut self, value: CollectionSchema) -> Self {
        self.value.schema = Some(value);
        self
    }

    /// Sets the num partitions and returns the updated value.
    pub fn num_partitions(mut self, value: i64) -> Self {
        self.value.num_partitions = value;
        self
    }

    /// Sets the num shards and returns the updated value.
    pub fn num_shards(mut self, value: i32) -> Self {
        self.value.num_shards = value;
        self
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.value.consistency_level = value;
        self
    }

    /// Sets the index params and returns the updated value.
    pub fn index_params(mut self, value: Vec<IndexParam>) -> Self {
        self.value.index_params = value;
        self
    }

    /// Sets the index param and returns the updated value.
    pub fn index_param(mut self, value: IndexParam) -> Self {
        self.value.index_params.push(value);
        self
    }

    /// Sets the properties and returns the updated value.
    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateCollectionRequest> {
        required("collection_name", &self.value.collection_name)?;
        if self.value.num_partitions < 0 {
            return Err(Error::validation(
                "num_partitions".into(),
                "must not be negative".into(),
            ));
        }
        positive_i32("num_shards", self.value.num_shards)?;
        let schema = self
            .value
            .schema
            .as_ref()
            .ok_or_else(|| Error::validation("schema".into(), "must be specified".into()))?;
        let mut field_names =
            HashSet::with_capacity(schema.get_fields().len() + schema.get_struct_fields().len());
        let mut primary_key_count = 0;
        for field in schema.get_fields() {
            required("field.name", field.get_name())?;
            if !field_names.insert(field.get_name()) {
                return Err(Error::validation(
                    "schema".into(),
                    format!("duplicate top-level field name {:?}", field.get_name()),
                ));
            }
            if field.get_data_type() == DataType::Unknown {
                return Err(Error::validation(
                    "field.data_type".into(),
                    format!("must be specified for field {:?}", field.get_name()),
                ));
            }
            field.validate()?;
            if field.is_primary_key() {
                primary_key_count += 1;
            }
        }
        for field in schema.get_struct_fields() {
            field.validate()?;
            if !field_names.insert(field.get_name()) {
                return Err(Error::validation(
                    "schema".into(),
                    format!("duplicate top-level field name {:?}", field.get_name()),
                ));
            }
        }
        if primary_key_count != 1 {
            return Err(Error::validation(
                "schema".into(),
                format!("must contain exactly one primary key field, found {primary_key_count}"),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateSimpleCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_simple_collection operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CreateSimpleCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) dimension: u32,
    pub(crate) primary_field: String,
    pub(crate) primary_field_type: DataType,
    pub(crate) max_length: u32,
    pub(crate) vector_field: String,
    pub(crate) auto_id: bool,
    pub(crate) enable_dynamic_field: bool,
    pub(crate) consistency_level: ConsistencyLevel,
    pub(crate) metric_type: MetricType,
}

impl CreateSimpleCollectionRequest {
    /// Creates a builder for this request.
    pub fn builder() -> CreateSimpleCollectionRequestBuilder {
        CreateSimpleCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateSimpleCollectionRequestBuilder {
        CreateSimpleCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the dimension.
    pub fn dimension(&self) -> u32 {
        self.dimension
    }

    /// Returns the primary field.
    pub fn primary_field(&self) -> &str {
        &self.primary_field
    }

    /// Returns the primary field type.
    pub fn primary_field_type(&self) -> DataType {
        self.primary_field_type
    }

    /// Returns the max length.
    pub fn max_length(&self) -> u32 {
        self.max_length
    }

    /// Returns the vector field.
    pub fn vector_field(&self) -> &str {
        &self.vector_field
    }

    /// Returns whether auto id.
    pub fn is_auto_id(&self) -> bool {
        self.auto_id
    }

    /// Returns whether dynamic field enabled.
    pub fn is_dynamic_field_enabled(&self) -> bool {
        self.enable_dynamic_field
    }

    /// Returns the consistency level.
    pub fn consistency_level(&self) -> ConsistencyLevel {
        self.consistency_level
    }

    /// Returns the metric type.
    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }
}

impl CreateSimpleCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            dimension: 0,
            primary_field: "id".into(),
            primary_field_type: DataType::Int64,
            max_length: 65_535,
            vector_field: "vector".into(),
            auto_id: false,
            enable_dynamic_field: true,
            consistency_level: ConsistencyLevel::Bounded,
            metric_type: MetricType::Cosine,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateSimpleCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateSimpleCollectionRequest.
#[derive(Debug, Clone)]
pub struct CreateSimpleCollectionRequestBuilder {
    value: CreateSimpleCollectionRequest,
}

impl CreateSimpleCollectionRequestBuilder {
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

    /// Sets the dimension and returns the updated value.
    pub fn dimension(mut self, value: u32) -> Self {
        self.value.dimension = value;
        self
    }

    /// Sets the primary field and returns the updated value.
    pub fn primary_field(mut self, value: impl Into<String>) -> Self {
        self.value.primary_field = value.into();
        self
    }

    /// Sets the primary field type and returns the updated value.
    pub fn primary_field_type(mut self, value: DataType) -> Self {
        self.value.primary_field_type = value;
        self
    }

    /// Sets the max length and returns the updated value.
    pub fn max_length(mut self, value: u32) -> Self {
        self.value.max_length = value;
        self
    }

    /// Sets the vector field and returns the updated value.
    pub fn vector_field(mut self, value: impl Into<String>) -> Self {
        self.value.vector_field = value.into();
        self
    }

    /// Sets the auto id and returns the updated value.
    pub fn auto_id(mut self, value: bool) -> Self {
        self.value.auto_id = value;
        self
    }

    /// Sets the enable dynamic field and returns the updated value.
    pub fn enable_dynamic_field(mut self, value: bool) -> Self {
        self.value.enable_dynamic_field = value;
        self
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.value.consistency_level = value;
        self
    }

    /// Sets the metric type and returns the updated value.
    pub fn metric_type(mut self, value: MetricType) -> Self {
        self.value.metric_type = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<CreateSimpleCollectionRequest> {
        required("collection_name", &self.value.collection_name)?;
        if self.value.dimension == 0 {
            return Err(Error::validation(
                "dimension".into(),
                "must be greater than zero".into(),
            ));
        }
        required("primary_field", &self.value.primary_field)?;
        required("vector_field", &self.value.vector_field)?;
        if !matches!(
            self.value.primary_field_type,
            DataType::Int64 | DataType::VarChar
        ) {
            return Err(Error::validation(
                "primary_field_type".into(),
                "must be Int64 or VarChar".into(),
            ));
        }
        if self.value.primary_field_type == DataType::VarChar && self.value.max_length == 0 {
            return Err(Error::validation(
                "max_length".into(),
                "must be greater than zero for a VarChar primary field".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl DropCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropCollectionRequestBuilder {
        DropCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropCollectionRequestBuilder {
        DropCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DropCollectionRequest {
        let mut value = milvus::DropCollectionRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropCollectionRequest.
#[derive(Debug, Clone)]
pub struct DropCollectionRequestBuilder {
    value: DropCollectionRequest,
}

impl DropCollectionRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 has_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HasCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl HasCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> HasCollectionRequestBuilder {
        HasCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> HasCollectionRequestBuilder {
        HasCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::HasCollectionRequest {
        let mut value = milvus::HasCollectionRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for HasCollectionRequest.
#[derive(Debug, Clone)]
pub struct HasCollectionRequestBuilder {
    value: HasCollectionRequest,
}

impl HasCollectionRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<HasCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReleaseCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 release_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReleaseCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ReleaseCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ReleaseCollectionRequestBuilder {
        ReleaseCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ReleaseCollectionRequestBuilder {
        ReleaseCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ReleaseCollectionRequest {
        let mut value = milvus::ReleaseCollectionRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReleaseCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ReleaseCollectionRequest.
#[derive(Debug, Clone)]
pub struct ReleaseCollectionRequestBuilder {
    value: ReleaseCollectionRequest,
}

impl ReleaseCollectionRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ReleaseCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl DescribeCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DescribeCollectionRequestBuilder {
        DescribeCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeCollectionRequestBuilder {
        DescribeCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DescribeCollectionRequest {
        let mut value = milvus::DescribeCollectionRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeCollectionRequest.
#[derive(Debug, Clone)]
pub struct DescribeCollectionRequestBuilder {
    value: DescribeCollectionRequest,
}

impl DescribeCollectionRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// LoadCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 load_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoadCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) sync: bool,
    pub(crate) replica_number: i32,
    /// Overall loading wait timeout in milliseconds. A value less than or
    /// equal to zero waits indefinitely.
    pub(crate) timeout_ms: i64,
    pub(crate) refresh: bool,
    pub(crate) load_fields: Vec<String>,
    pub(crate) skip_load_dynamic_field: bool,
    pub(crate) resource_groups: Vec<String>,
}

impl LoadCollectionRequest {
    /// Creates a builder for this request.
    pub fn builder() -> LoadCollectionRequestBuilder {
        LoadCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> LoadCollectionRequestBuilder {
        LoadCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns whether sync.
    pub fn is_sync(&self) -> bool {
        self.sync
    }

    /// Returns the replica number.
    pub fn replica_number(&self) -> i32 {
        self.replica_number
    }

    /// Returns the timeout ms.
    pub fn timeout_ms(&self) -> i64 {
        self.timeout_ms
    }

    /// Returns whether the request should refresh.
    pub fn should_refresh(&self) -> bool {
        self.refresh
    }

    /// Returns the load fields.
    pub fn load_fields(&self) -> &[String] {
        &self.load_fields
    }

    /// Returns whether the request should skip load dynamic field.
    pub fn should_skip_load_dynamic_field(&self) -> bool {
        self.skip_load_dynamic_field
    }

    /// Returns the resource groups.
    pub fn resource_groups(&self) -> &[String] {
        &self.resource_groups
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::LoadCollectionRequest {
        let mut value = milvus::LoadCollectionRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value.replica_number = self.replica_number;
        value.refresh = self.refresh;
        value.load_fields = self.load_fields;
        value.skip_load_dynamic_field = self.skip_load_dynamic_field;
        value.resource_groups = self.resource_groups;
        value
    }
}

impl LoadCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            sync: true,
            replica_number: 1,
            timeout_ms: 60_000,
            refresh: false,
            load_fields: Vec::new(),
            skip_load_dynamic_field: false,
            resource_groups: Vec::new(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// LoadCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for LoadCollectionRequest.
#[derive(Debug, Clone)]
pub struct LoadCollectionRequestBuilder {
    value: LoadCollectionRequest,
}

impl LoadCollectionRequestBuilder {
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

    /// Sets the sync and returns the updated value.
    pub fn sync(mut self, value: bool) -> Self {
        self.value.sync = value;
        self
    }

    /// Sets the replica number and returns the updated value.
    pub fn replica_number(mut self, value: i32) -> Self {
        self.value.replica_number = value;
        self
    }

    /// Sets the timeout ms and returns the updated value.
    pub fn timeout_ms(mut self, value: i64) -> Self {
        self.value.timeout_ms = value;
        self
    }

    /// Sets the refresh and returns the updated value.
    pub fn refresh(mut self, value: bool) -> Self {
        self.value.refresh = value;
        self
    }

    /// Sets the load fields and returns the updated value.
    pub fn load_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.load_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the skip load dynamic field and returns the updated value.
    pub fn skip_load_dynamic_field(mut self, value: bool) -> Self {
        self.value.skip_load_dynamic_field = value;
        self
    }

    /// Sets the resource groups and returns the updated value.
    pub fn resource_groups(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.resource_groups = values.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<LoadCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        positive_i32("replica_number", self.value.replica_number)?;
        non_empty_strings("load_fields", &self.value.load_fields)?;
        non_empty_strings("resource_groups", &self.value.resource_groups)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RefreshLoadRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 refresh_load operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RefreshLoadRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) sync: bool,
    /// Overall refresh wait timeout in milliseconds. A value less than or
    /// equal to zero waits indefinitely.
    pub(crate) timeout_ms: i64,
}

impl RefreshLoadRequest {
    /// Creates a builder for this request.
    pub fn builder() -> RefreshLoadRequestBuilder {
        RefreshLoadRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RefreshLoadRequestBuilder {
        RefreshLoadRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns whether sync.
    pub fn is_sync(&self) -> bool {
        self.sync
    }

    /// Returns the timeout ms.
    pub fn timeout_ms(&self) -> i64 {
        self.timeout_ms
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::LoadCollectionRequest {
        milvus::LoadCollectionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            refresh: true,
            ..Default::default()
        }
    }
}

impl RefreshLoadRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            sync: true,
            timeout_ms: 60_000,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RefreshLoadRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RefreshLoadRequest.
#[derive(Debug, Clone)]
pub struct RefreshLoadRequestBuilder {
    value: RefreshLoadRequest,
}

impl RefreshLoadRequestBuilder {
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

    /// Sets the sync and returns the updated value.
    pub fn sync(mut self, value: bool) -> Self {
        self.value.sync = value;
        self
    }

    /// Sets the timeout ms and returns the updated value.
    pub fn timeout_ms(mut self, value: i64) -> Self {
        self.value.timeout_ms = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RefreshLoadRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// BatchDescribeCollectionsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 batch_describe_collections operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BatchDescribeCollectionsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_names: Vec<String>,
    pub(crate) collection_ids: Vec<i64>,
}

impl BatchDescribeCollectionsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_names: Default::default(),
            collection_ids: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> BatchDescribeCollectionsRequestBuilder {
        BatchDescribeCollectionsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> BatchDescribeCollectionsRequestBuilder {
        BatchDescribeCollectionsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection names.
    pub fn collection_names(&self) -> &[String] {
        &self.collection_names
    }

    /// Returns the collection ids.
    pub fn collection_ids(&self) -> &[i64] {
        &self.collection_ids
    }

    pub(crate) fn into_proto(self) -> milvus::BatchDescribeCollectionRequest {
        milvus::BatchDescribeCollectionRequest {
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_names,
            collection_id: self.collection_ids,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// BatchDescribeCollectionsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for BatchDescribeCollectionsRequest.
#[derive(Debug, Clone)]
pub struct BatchDescribeCollectionsRequestBuilder {
    value: BatchDescribeCollectionsRequest,
}

impl BatchDescribeCollectionsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection names and returns the updated value.
    pub fn collection_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.collection_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_names.push(value.into());
        self
    }

    /// Sets the collection ids and returns the updated value.
    pub fn collection_ids(mut self, value: Vec<i64>) -> Self {
        self.value.collection_ids = value;
        self
    }

    /// Sets the collection id and returns the updated value.
    pub fn collection_id(mut self, value: i64) -> Self {
        self.value.collection_ids.push(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<BatchDescribeCollectionsRequest> {
        non_empty_strings("collection_names", &self.value.collection_names)?;
        if self.value.collection_ids.iter().any(|value| *value <= 0) {
            return Err(Error::validation(
                "collection_ids".into(),
                "must contain only positive values".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCollectionStatsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_collection_stats operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetCollectionStatsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl GetCollectionStatsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetCollectionStatsRequestBuilder {
        GetCollectionStatsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetCollectionStatsRequestBuilder {
        GetCollectionStatsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetCollectionStatisticsRequest {
        let mut value = milvus::GetCollectionStatisticsRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCollectionStatsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetCollectionStatsRequest.
#[derive(Debug, Clone)]
pub struct GetCollectionStatsRequestBuilder {
    value: GetCollectionStatsRequest,
}

impl GetCollectionStatsRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetCollectionStatsRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListCollectionsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_collections operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListCollectionsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) only_loaded: bool,
}

impl ListCollectionsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            only_loaded: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListCollectionsRequestBuilder {
        ListCollectionsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListCollectionsRequestBuilder {
        ListCollectionsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns whether only loaded.
    pub fn is_only_loaded(&self) -> bool {
        self.only_loaded
    }

    pub(crate) fn into_proto(self) -> milvus::ShowCollectionsRequest {
        let mut v = milvus::ShowCollectionsRequest::default();
        v.db_name = self.database_name.unwrap_or_default();
        v.r#type = if self.only_loaded {
            milvus::ShowType::InMemory as i32
        } else {
            milvus::ShowType::All as i32
        };
        v
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListCollectionsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListCollectionsRequest.
#[derive(Debug, Clone)]
pub struct ListCollectionsRequestBuilder {
    value: ListCollectionsRequest,
}

impl ListCollectionsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the only loaded and returns the updated value.
    pub fn only_loaded(mut self, value: bool) -> Self {
        self.value.only_loaded = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<ListCollectionsRequest> {
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetLoadStateRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_load_state operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetLoadStateRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
}

impl GetLoadStateRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_names: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetLoadStateRequestBuilder {
        GetLoadStateRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetLoadStateRequestBuilder {
        GetLoadStateRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the partition names.
    pub fn partition_names(&self) -> &[String] {
        &self.partition_names
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetLoadStateRequest {
        milvus::GetLoadStateRequest {
            base: None,
            collection_name: self.collection_name,
            partition_names: self.partition_names,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetLoadStateRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetLoadStateRequest.
#[derive(Debug, Clone)]
pub struct GetLoadStateRequestBuilder {
    value: GetLoadStateRequest,
}

impl GetLoadStateRequestBuilder {
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

    /// Sets the partition names and returns the updated value.
    pub fn partition_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.partition_names.clear();
        for value in values.into_iter().map(Into::into) {
            if !self.value.partition_names.contains(&value) {
                self.value.partition_names.push(value);
            }
        }
        self
    }

    /// Sets the partition name and returns the updated value.
    pub fn partition_name(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !self.value.partition_names.contains(&value) {
            self.value.partition_names.push(value);
        }
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<GetLoadStateRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        non_empty_strings("partition_names", &self.value.partition_names)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterCollectionPropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_collection_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AlterCollectionPropertiesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) properties: HashMap<String, String>,
}

impl AlterCollectionPropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            properties: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AlterCollectionPropertiesRequestBuilder {
        AlterCollectionPropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterCollectionPropertiesRequestBuilder {
        AlterCollectionPropertiesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::AlterCollectionRequest {
        milvus::AlterCollectionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            collection_id: 0,
            properties: kv(self.properties),
            delete_keys: Vec::new(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterCollectionPropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterCollectionPropertiesRequest.
#[derive(Debug, Clone)]
pub struct AlterCollectionPropertiesRequestBuilder {
    value: AlterCollectionPropertiesRequest,
}

impl AlterCollectionPropertiesRequestBuilder {
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

    /// Sets the properties and returns the updated value.
    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    /// Sets the property and returns the updated value.
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.value.properties.insert(key.into(), value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AlterCollectionPropertiesRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        if self.value.properties.is_empty() {
            return Err(Error::validation(
                "properties".into(),
                "must contain at least one property".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionPropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_collection_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropCollectionPropertiesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) property_keys: HashSet<String>,
}

impl DropCollectionPropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            property_keys: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropCollectionPropertiesRequestBuilder {
        DropCollectionPropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropCollectionPropertiesRequestBuilder {
        DropCollectionPropertiesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the property keys.
    pub fn property_keys(&self) -> &HashSet<String> {
        &self.property_keys
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::AlterCollectionRequest {
        milvus::AlterCollectionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            collection_id: 0,
            properties: Vec::new(),
            delete_keys: self.property_keys.into_iter().collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionPropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropCollectionPropertiesRequest.
#[derive(Debug, Clone)]
pub struct DropCollectionPropertiesRequestBuilder {
    value: DropCollectionPropertiesRequest,
}

impl DropCollectionPropertiesRequestBuilder {
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

    /// Sets the property keys and returns the updated value.
    pub fn property_keys(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.property_keys = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the property key and returns the updated value.
    pub fn property_key(mut self, value: impl Into<String>) -> Self {
        self.value.property_keys.insert(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropCollectionPropertiesRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        if self.value.property_keys.is_empty() {
            return Err(Error::validation(
                "property_keys".into(),
                "must contain at least one property key".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterCollectionFieldPropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_collection_field_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AlterCollectionFieldPropertiesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: String,
    pub(crate) properties: HashMap<String, String>,
}

impl AlterCollectionFieldPropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            properties: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AlterCollectionFieldPropertiesRequestBuilder {
        AlterCollectionFieldPropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterCollectionFieldPropertiesRequestBuilder {
        AlterCollectionFieldPropertiesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::AlterCollectionFieldRequest {
        milvus::AlterCollectionFieldRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            field_name: self.field_name,
            properties: kv(self.properties),
            delete_keys: Vec::new(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterCollectionFieldPropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterCollectionFieldPropertiesRequest.
#[derive(Debug, Clone)]
pub struct AlterCollectionFieldPropertiesRequestBuilder {
    value: AlterCollectionFieldPropertiesRequest,
}

impl AlterCollectionFieldPropertiesRequestBuilder {
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

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = value.into();
        self
    }

    /// Sets the properties and returns the updated value.
    pub fn properties(mut self, value: HashMap<String, String>) -> Self {
        self.value.properties = value;
        self
    }

    /// Sets the property and returns the updated value.
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.value.properties.insert(key.into(), value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AlterCollectionFieldPropertiesRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("field_name", &self.value.field_name)?;
        if self.value.properties.is_empty() {
            return Err(Error::validation(
                "properties".into(),
                "must contain at least one property".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionFieldPropertiesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_collection_field_properties operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DropCollectionFieldPropertiesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: String,
    pub(crate) property_keys: HashSet<String>,
}

impl DropCollectionFieldPropertiesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            property_keys: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropCollectionFieldPropertiesRequestBuilder {
        DropCollectionFieldPropertiesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropCollectionFieldPropertiesRequestBuilder {
        DropCollectionFieldPropertiesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the property keys.
    pub fn property_keys(&self) -> &HashSet<String> {
        &self.property_keys
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::AlterCollectionFieldRequest {
        milvus::AlterCollectionFieldRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            field_name: self.field_name,
            properties: Vec::new(),
            delete_keys: self.property_keys.into_iter().collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionFieldPropertiesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropCollectionFieldPropertiesRequest.
#[derive(Debug, Clone)]
pub struct DropCollectionFieldPropertiesRequestBuilder {
    value: DropCollectionFieldPropertiesRequest,
}

impl DropCollectionFieldPropertiesRequestBuilder {
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

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = value.into();
        self
    }

    /// Sets the property keys and returns the updated value.
    pub fn property_keys(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.property_keys = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the property key and returns the updated value.
    pub fn property_key(mut self, value: impl Into<String>) -> Self {
        self.value.property_keys.insert(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropCollectionFieldPropertiesRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("field_name", &self.value.field_name)?;
        if self.value.property_keys.is_empty() {
            return Err(Error::validation(
                "property_keys".into(),
                "must contain at least one property key".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddCollectionFieldRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 add_collection_field operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AddCollectionFieldRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field: Option<FieldSchema>,
}

impl AddCollectionFieldRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AddCollectionFieldRequestBuilder {
        AddCollectionFieldRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AddCollectionFieldRequestBuilder {
        AddCollectionFieldRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field.
    pub fn field(&self) -> Option<&FieldSchema> {
        self.field.as_ref()
    }

    pub(crate) fn into_proto(self) -> Result<milvus::AddCollectionFieldRequest> {
        Ok(milvus::AddCollectionFieldRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            schema: self
                .field
                .ok_or_else(|| Error::validation("field".into(), "must be specified".into()))?
                .encode()?,
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddCollectionFieldRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AddCollectionFieldRequest.
#[derive(Debug, Clone)]
pub struct AddCollectionFieldRequestBuilder {
    value: AddCollectionFieldRequest,
}

impl AddCollectionFieldRequestBuilder {
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

    /// Sets the field and returns the updated value.
    pub fn field(mut self, value: FieldSchema) -> Self {
        self.value.field = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AddCollectionFieldRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        let field = self
            .value
            .field
            .as_ref()
            .ok_or_else(|| Error::validation("field".into(), "must be specified".into()))?;
        required("field.name", field.get_name())?;
        if field.get_data_type() == DataType::Unknown {
            return Err(Error::validation(
                "field.data_type".into(),
                "must be specified".into(),
            ));
        }
        field.validate()?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddCollectionFunctionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 add_collection_function operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[deprecated(
    note = "Milvus 3.0 and later do not support adding a function separately; use AddFunctionFieldRequest instead"
)]
pub struct AddCollectionFunctionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) function: Option<Function>,
}

#[allow(deprecated)]
impl AddCollectionFunctionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            function: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AddCollectionFunctionRequestBuilder {
        AddCollectionFunctionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AddCollectionFunctionRequestBuilder {
        AddCollectionFunctionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the function.
    pub fn function(&self) -> &Option<Function> {
        &self.function
    }

    pub(crate) fn into_proto(self) -> milvus::AddCollectionFunctionRequest {
        milvus::AddCollectionFunctionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            function_schema: self.function.map(Function::into_proto),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddCollectionFunctionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AddCollectionFunctionRequest.
#[derive(Debug, Clone)]
#[deprecated(
    note = "Milvus 3.0 and later do not support adding a function separately; use AddFunctionFieldRequestBuilder instead"
)]
pub struct AddCollectionFunctionRequestBuilder {
    #[allow(deprecated)]
    value: AddCollectionFunctionRequest,
}

#[allow(deprecated)]
impl AddCollectionFunctionRequestBuilder {
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

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: impl Into<Function>) -> Self {
        self.value.function = Some(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AddCollectionFunctionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        let function = self
            .value
            .function
            .as_ref()
            .ok_or_else(|| Error::validation("function".into(), "must be specified".into()))?;
        required("function.name", function.get_name())?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterCollectionFunctionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 alter_collection_function operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AlterCollectionFunctionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) function: Option<Function>,
}

impl AlterCollectionFunctionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            function: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AlterCollectionFunctionRequestBuilder {
        AlterCollectionFunctionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AlterCollectionFunctionRequestBuilder {
        AlterCollectionFunctionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the function.
    pub fn function(&self) -> &Option<Function> {
        &self.function
    }

    pub(crate) fn into_proto(self) -> milvus::AlterCollectionFunctionRequest {
        let function = self
            .function
            .expect("validated alter-function request contains a function");
        let function_name = function.get_name().to_owned();
        milvus::AlterCollectionFunctionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            function_name,
            function_schema: Some(function.into_proto()),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AlterCollectionFunctionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AlterCollectionFunctionRequest.
#[derive(Debug, Clone)]
pub struct AlterCollectionFunctionRequestBuilder {
    value: AlterCollectionFunctionRequest,
}

impl AlterCollectionFunctionRequestBuilder {
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

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: impl Into<Function>) -> Self {
        self.value.function = Some(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AlterCollectionFunctionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        let function = self
            .value
            .function
            .as_ref()
            .ok_or_else(|| Error::validation("function".into(), "must be specified".into()))?;
        required("function.name", function.get_name())?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionFunctionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_collection_function operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[deprecated(
    note = "Milvus 3.0 and later do not support dropping a function separately; use DropFunctionFieldRequest instead"
)]
pub struct DropCollectionFunctionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) function_name: String,
}

#[allow(deprecated)]
impl DropCollectionFunctionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            function_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropCollectionFunctionRequestBuilder {
        DropCollectionFunctionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropCollectionFunctionRequestBuilder {
        DropCollectionFunctionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the function name.
    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub(crate) fn into_proto(self) -> milvus::DropCollectionFunctionRequest {
        milvus::DropCollectionFunctionRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            function_name: self.function_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionFunctionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropCollectionFunctionRequest.
#[derive(Debug, Clone)]
#[deprecated(
    note = "Milvus 3.0 and later do not support dropping a function separately; use DropFunctionFieldRequestBuilder instead"
)]
pub struct DropCollectionFunctionRequestBuilder {
    #[allow(deprecated)]
    value: DropCollectionFunctionRequest,
}

#[allow(deprecated)]
impl DropCollectionFunctionRequestBuilder {
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

    /// Sets the function name and returns the updated value.
    pub fn function_name(mut self, value: impl Into<String>) -> Self {
        self.value.function_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropCollectionFunctionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("function_name", &self.value.function_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// TruncateCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 truncate_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TruncateCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl TruncateCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> TruncateCollectionRequestBuilder {
        TruncateCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> TruncateCollectionRequestBuilder {
        TruncateCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::TruncateCollectionRequest {
        let mut value = milvus::TruncateCollectionRequest::default();
        value.db_name = self.database_name.unwrap_or_else(|| default_db.to_owned());
        value.collection_name = self.collection_name;
        value
    }
}

///////////////////////////////////////////////////////////////////////////////
// TruncateCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for TruncateCollectionRequest.
#[derive(Debug, Clone)]
pub struct TruncateCollectionRequestBuilder {
    value: TruncateCollectionRequest,
}

impl TruncateCollectionRequestBuilder {
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

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<TruncateCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeReplicasRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 describe_replicas operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeReplicasRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) with_shard_nodes: bool,
}

impl DescribeReplicasRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            with_shard_nodes: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DescribeReplicasRequestBuilder {
        DescribeReplicasRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DescribeReplicasRequestBuilder {
        DescribeReplicasRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns whether the request should include shard nodes.
    pub fn should_include_shard_nodes(&self) -> bool {
        self.with_shard_nodes
    }

    pub(crate) fn into_proto(self) -> milvus::GetReplicasRequest {
        milvus::GetReplicasRequest {
            base: None,
            collection_id: 0,
            with_shard_nodes: self.with_shard_nodes,
            collection_name: self.collection_name,
            db_name: self.database_name.unwrap_or_default(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeReplicasRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeReplicasRequest.
#[derive(Debug, Clone)]
pub struct DescribeReplicasRequestBuilder {
    value: DescribeReplicasRequest,
}

impl DescribeReplicasRequestBuilder {
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

    /// Returns this value configured with with shard nodes.
    pub fn with_shard_nodes(mut self, value: bool) -> Self {
        self.value.with_shard_nodes = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DescribeReplicasRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// RenameCollectionRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 rename_collection operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RenameCollectionRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) new_collection_name: String,
    pub(crate) new_database_name: Option<String>,
}

impl RenameCollectionRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            new_collection_name: Default::default(),
            new_database_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> RenameCollectionRequestBuilder {
        RenameCollectionRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> RenameCollectionRequestBuilder {
        RenameCollectionRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the new collection name.
    pub fn new_collection_name(&self) -> &str {
        &self.new_collection_name
    }

    /// Returns the new database name.
    pub fn new_database_name(&self) -> &Option<String> {
        &self.new_database_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::RenameCollectionRequest {
        let db_name = self
            .database_name
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| default_db.to_owned());
        let new_db_name = self
            .new_database_name
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| db_name.clone());
        milvus::RenameCollectionRequest {
            base: None,
            db_name,
            old_name: self.collection_name,
            new_name: self.new_collection_name,
            new_db_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RenameCollectionRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for RenameCollectionRequest.
#[derive(Debug, Clone)]
pub struct RenameCollectionRequestBuilder {
    value: RenameCollectionRequest,
}

impl RenameCollectionRequestBuilder {
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

    /// Sets the new collection name and returns the updated value.
    pub fn new_collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.new_collection_name = value.into();
        self
    }

    /// Sets the new database name and returns the updated value.
    pub fn new_database_name(mut self, value: impl Into<String>) -> Self {
        self.value.new_database_name = Some(value.into());
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<RenameCollectionRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("new_collection_name", &self.value.new_collection_name)?;
        Ok(self.value)
    }
}

fn validate_collection_name(_database_name: Option<&str>, collection_name: &str) -> Result<()> {
    required("collection_name", collection_name)
}

fn kv(values: HashMap<String, String>) -> Vec<common::KeyValuePair> {
    values
        .into_iter()
        .map(|(key, value)| common::KeyValuePair {
            key,
            value,
            ..Default::default()
        })
        .collect()
}

///////////////////////////////////////////////////////////////////////////////
// AddCollectionStructFieldRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 add_collection_struct_field operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AddCollectionStructFieldRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) struct_field: Option<StructFieldSchema>,
}

impl AddCollectionStructFieldRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            struct_field: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AddCollectionStructFieldRequestBuilder {
        AddCollectionStructFieldRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AddCollectionStructFieldRequestBuilder {
        AddCollectionStructFieldRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the struct field schema.
    pub fn struct_field(&self) -> Option<&StructFieldSchema> {
        self.struct_field.as_ref()
    }

    pub(crate) fn into_proto(self) -> Result<milvus::AddCollectionStructFieldRequest> {
        Ok(milvus::AddCollectionStructFieldRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            struct_array_field_schema: Some(
                self.struct_field
                    .ok_or_else(|| {
                        Error::validation("struct_field".into(), "must be specified".into())
                    })?
                    .into_proto(),
            ),
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddCollectionStructFieldRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AddCollectionStructFieldRequest.
#[derive(Debug, Clone)]
pub struct AddCollectionStructFieldRequestBuilder {
    value: AddCollectionStructFieldRequest,
}

impl AddCollectionStructFieldRequestBuilder {
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

    /// Sets the struct field schema and returns the updated value.
    pub fn struct_field(mut self, value: StructFieldSchema) -> Self {
        self.value.struct_field = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AddCollectionStructFieldRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        let struct_field =
            self.value.struct_field.as_ref().ok_or_else(|| {
                Error::validation("struct_field".into(), "must be specified".into())
            })?;
        if struct_field.get_name().is_empty() {
            return Err(Error::validation(
                "struct_field.name".into(),
                "must be specified".into(),
            ));
        }
        struct_field.validate()?;
        // The server only accepts a nullable struct field when adding it to an existing
        // collection; reject a non-nullable schema up front.
        if !struct_field.is_nullable() {
            return Err(Error::validation(
                "struct_field.nullable".into(),
                "must be true when adding a struct field to an existing collection".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddFunctionFieldRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 add_function_field operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AddFunctionFieldRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field: Option<FieldSchema>,
    pub(crate) function: Option<Function>,
    pub(crate) index: Option<IndexParam>,
}

impl AddFunctionFieldRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field: Default::default(),
            function: Default::default(),
            index: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> AddFunctionFieldRequestBuilder {
        AddFunctionFieldRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> AddFunctionFieldRequestBuilder {
        AddFunctionFieldRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the output field schema.
    pub fn field(&self) -> Option<&FieldSchema> {
        self.field.as_ref()
    }

    /// Returns the function definition.
    pub fn function(&self) -> Option<&Function> {
        self.function.as_ref()
    }

    /// Returns the bound index parameter.
    pub fn index(&self) -> Option<&IndexParam> {
        self.index.as_ref()
    }

    /// Returns the index definition bound to the output field, as `index_name` + `extra_params`.
    fn bound_index(&self) -> Result<(String, Vec<common::KeyValuePair>)> {
        let index = self
            .index
            .as_ref()
            .ok_or_else(|| Error::validation("index".into(), "must be specified".into()))?;
        let output_name = self
            .field
            .as_ref()
            .map(|field| field.get_name())
            .unwrap_or_default();
        // The bound index must target the output field and use an explicit index type, matching
        // pymilvus's add_function_field validation.
        let index_field = index.get_field_name();
        if !index_field.is_empty() && index_field != output_name {
            return Err(Error::validation(
                "index.field_name".into(),
                "must match the function output field name".into(),
            ));
        }
        if index.get_index_type() == IndexType::Invalid {
            return Err(Error::validation(
                "index.index_type".into(),
                "an explicit index type is required".into(),
            ));
        }
        let mut extra_params = index.get_extra_params().clone();
        extra_params.remove("index_type");
        extra_params.remove("metric_type");
        let mut pairs = kv(extra_params);
        pairs.push(common::KeyValuePair {
            key: "index_type".into(),
            value: index.get_index_type().as_str().into(),
            ..Default::default()
        });
        if let Some(metric_type) = index
            .get_metric_type()
            .filter(|value| *value != MetricType::Default)
        {
            pairs.push(common::KeyValuePair {
                key: "metric_type".into(),
                value: metric_type.as_str().into(),
                ..Default::default()
            });
        }
        Ok((index.get_index_name().to_owned(), pairs))
    }

    pub(crate) fn into_proto(self) -> Result<milvus::AlterCollectionSchemaRequest> {
        use crate::proto::milvus::alter_collection_schema_request::{self as req};

        let (index_name, extra_params) = self.bound_index()?;
        let field_schema = self
            .field
            .ok_or_else(|| Error::validation("field".into(), "must be specified".into()))?;
        let function = self
            .function
            .ok_or_else(|| Error::validation("function".into(), "must be specified".into()))?;

        let add_request = req::AddRequest {
            field_infos: vec![req::FieldInfo {
                field_schema: Some(field_schema.into_proto()),
                index_name,
                extra_params,
            }],
            func_schema: vec![function.into_proto()],
            do_physical_backfill: false,
        };
        Ok(milvus::AlterCollectionSchemaRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            action: Some(req::Action {
                op: Some(req::action::Op::AddRequest(add_request)),
            }),
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// AddFunctionFieldRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for AddFunctionFieldRequest.
#[derive(Debug, Clone)]
pub struct AddFunctionFieldRequestBuilder {
    value: AddFunctionFieldRequest,
}

impl AddFunctionFieldRequestBuilder {
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

    /// Sets the output field schema and returns the updated value.
    pub fn field(mut self, value: FieldSchema) -> Self {
        self.value.field = Some(value);
        self
    }

    /// Sets the function definition and returns the updated value.
    pub fn function(mut self, value: Function) -> Self {
        self.value.function = Some(value);
        self
    }

    /// Sets the bound index parameter and returns the updated value.
    pub fn index(mut self, value: IndexParam) -> Self {
        self.value.index = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<AddFunctionFieldRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        let field = self
            .value
            .field
            .as_ref()
            .ok_or_else(|| Error::validation("field".into(), "must be specified".into()))?;
        let function = self
            .value
            .function
            .as_ref()
            .ok_or_else(|| Error::validation("function".into(), "must be specified".into()))?;
        required("field.name", field.get_name())?;
        if field.get_data_type() == DataType::Unknown {
            return Err(Error::validation(
                "field.data_type".into(),
                "must be specified".into(),
            ));
        }
        field.validate()?;
        required("function.name", function.get_name())?;

        // Only BM25 and MinHash function fields can be added to an existing collection; other
        // function types are defined at collection creation, as in pymilvus.
        let expected = match function.get_function_type() {
            FunctionType::Bm25 => Some(DataType::SparseFloatVector),
            FunctionType::MinHash => Some(DataType::BinaryVector),
            _ => {
                return Err(Error::validation(
                    "function.function_type".into(),
                    format!(
                        "{:?} functions cannot be added to an existing collection",
                        function.get_function_type()
                    ),
                ));
            }
        };
        if let Some(expected) = expected {
            if field.get_data_type() != expected {
                return Err(Error::validation(
                    "field.data_type".into(),
                    format!(
                        "must be {expected:?} for a {:?} function",
                        function.get_function_type()
                    ),
                ));
            }
        }

        // Validate the bound index up front so the request fails before RPC.
        self.value.bound_index()?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropFunctionFieldRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_function_field operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DropFunctionFieldRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) function_name: String,
}

impl DropFunctionFieldRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            function_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropFunctionFieldRequestBuilder {
        DropFunctionFieldRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropFunctionFieldRequestBuilder {
        DropFunctionFieldRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the function name.
    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub(crate) fn into_proto(self) -> Result<milvus::AlterCollectionSchemaRequest> {
        use crate::proto::milvus::alter_collection_schema_request::{self as req};

        let drop_request = req::DropRequest {
            drop_function_output_fields: true,
            identifier: Some(req::drop_request::Identifier::FunctionName(
                self.function_name,
            )),
        };
        Ok(milvus::AlterCollectionSchemaRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            action: Some(req::Action {
                op: Some(req::action::Op::DropRequest(drop_request)),
            }),
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropFunctionFieldRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropFunctionFieldRequest.
#[derive(Debug, Clone)]
pub struct DropFunctionFieldRequestBuilder {
    value: DropFunctionFieldRequest,
}

impl DropFunctionFieldRequestBuilder {
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

    /// Sets the function name and returns the updated value.
    pub fn function_name(mut self, value: impl Into<String>) -> Self {
        self.value.function_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropFunctionFieldRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        required("function_name", &self.value.function_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionFieldRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_collection_field operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DropCollectionFieldRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) field_name: Option<String>,
    pub(crate) field_id: Option<i64>,
}

impl DropCollectionFieldRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            field_name: Default::default(),
            field_id: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropCollectionFieldRequestBuilder {
        DropCollectionFieldRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropCollectionFieldRequestBuilder {
        DropCollectionFieldRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the field name, if specified.
    pub fn field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }

    /// Returns the field id, if specified.
    pub fn field_id(&self) -> Option<i64> {
        self.field_id
    }

    pub(crate) fn into_proto(self) -> Result<milvus::AlterCollectionSchemaRequest> {
        use crate::proto::milvus::alter_collection_schema_request::{self as req};

        let identifier = if let Some(name) = self.field_name.filter(|name| !name.is_empty()) {
            req::drop_request::Identifier::FieldName(name)
        } else if let Some(id) = self.field_id {
            req::drop_request::Identifier::FieldId(id)
        } else {
            return Err(Error::validation(
                "field_name/field_id".into(),
                "exactly one of field_name or field_id must be specified".into(),
            ));
        };
        let drop_request = req::DropRequest {
            drop_function_output_fields: false,
            identifier: Some(identifier),
        };
        Ok(milvus::AlterCollectionSchemaRequest {
            base: None,
            db_name: self.database_name.unwrap_or_default(),
            collection_name: self.collection_name,
            collection_id: 0,
            action: Some(req::Action {
                op: Some(req::action::Op::DropRequest(drop_request)),
            }),
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropCollectionFieldRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropCollectionFieldRequest.
#[derive(Debug, Clone)]
pub struct DropCollectionFieldRequestBuilder {
    value: DropCollectionFieldRequest,
}

impl DropCollectionFieldRequestBuilder {
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

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.value.field_name = Some(value.into());
        self
    }

    /// Sets the field id and returns the updated value.
    pub fn field_id(mut self, value: i64) -> Self {
        self.value.field_id = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<DropCollectionFieldRequest> {
        validate_collection_name(
            self.value.database_name.as_deref(),
            &self.value.collection_name,
        )?;
        let has_name = self
            .value
            .field_name
            .as_deref()
            .is_some_and(|name| !name.is_empty());
        let has_id = self.value.field_id.is_some_and(|id| id > 0);
        if has_name == has_id {
            return Err(Error::validation(
                "field_name/field_id".into(),
                "exactly one of field_name or field_id must be specified".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod create_collection_tests {
    use super::{CreateCollectionRequest, CreateSimpleCollectionRequest};
    use crate::proto::schema;
    use crate::v2::types::{
        CollectionSchema, DataType, DefaultValue, FieldSchema, IndexType, MetricType,
        StructFieldSchema,
    };
    use prost::Message;

    fn primary_field(name: &str) -> FieldSchema {
        FieldSchema::new()
            .name(name)
            .data_type(DataType::Int64)
            .primary_key(true)
    }

    fn schema_with_primary() -> CollectionSchema {
        CollectionSchema::new().add_field(primary_field("id"))
    }

    #[test]
    fn default_request_keeps_cpp_null_schema_semantics() {
        let request = CreateCollectionRequest::empty();
        assert!(request.schema.is_none());
        assert!(request.into_proto("default").is_err());
    }

    #[test]
    fn dense_vector_fields_require_a_positive_dimension() {
        for data_type in [
            DataType::FloatVector,
            DataType::BinaryVector,
            DataType::Float16Vector,
            DataType::BFloat16Vector,
            DataType::Int8Vector,
        ] {
            let error = CreateCollectionRequest::builder()
                .collection_name("books")
                .schema(
                    schema_with_primary().add_field(
                        crate::v2::types::FieldSchema::new()
                            .name("vector")
                            .data_type(data_type),
                    ),
                )
                .build()
                .expect_err("dense vector field without dimension must fail");
            assert!(error.to_string().contains("positive dimension"));
        }

        CreateCollectionRequest::builder()
            .collection_name("books")
            .schema(
                schema_with_primary().add_field(
                    crate::v2::types::FieldSchema::new()
                        .name("sparse")
                        .data_type(DataType::SparseFloatVector),
                ),
            )
            .build()
            .expect("sparse vector fields do not require a dimension");
    }

    #[test]
    fn struct_fields_enforce_server_schema_invariants() {
        let build = |struct_field| {
            CreateCollectionRequest::builder()
                .collection_name("books")
                .schema(schema_with_primary().add_struct_field(struct_field))
                .build()
        };

        assert!(build(StructFieldSchema::new()).is_err());
        assert!(build(
            StructFieldSchema::new()
                .name("events")
                .add_field(FieldSchema::new().name("kind").data_type(DataType::Int64))
        )
        .is_err());
        assert!(build(StructFieldSchema::new().name("events").max_capacity(8)).is_err());
        assert!(build(
            StructFieldSchema::new()
                .name("events")
                .max_capacity(8)
                .add_field(FieldSchema::new().name("kind").data_type(DataType::Int64))
                .add_field(FieldSchema::new().name("kind").data_type(DataType::Int32))
        )
        .is_err());

        for invalid_field in [
            FieldSchema::new()
                .name("nested")
                .data_type(DataType::Array)
                .element_type(DataType::Int64),
            FieldSchema::new()
                .name("primary")
                .data_type(DataType::Int64)
                .primary_key(true),
            FieldSchema::new()
                .name("defaulted")
                .data_type(DataType::Int64)
                .default_value(DefaultValue::Int64(1)),
        ] {
            assert!(build(
                StructFieldSchema::new()
                    .name("events")
                    .max_capacity(8)
                    .add_field(invalid_field)
            )
            .is_err());
        }

        build(
            StructFieldSchema::new()
                .name("events")
                .max_capacity(8)
                .add_field(FieldSchema::new().name("kind").data_type(DataType::Int64))
                .add_field(
                    FieldSchema::new()
                        .name("embedding")
                        .data_type(DataType::FloatVector)
                        .dimension(4),
                ),
        )
        .expect("valid struct field schema");
    }

    #[test]
    fn schema_requires_unique_top_level_field_names() {
        let build = |schema| {
            CreateCollectionRequest::builder()
                .collection_name("books")
                .schema(schema)
                .build()
        };

        let duplicate_scalar = build(
            schema_with_primary()
                .add_field(FieldSchema::new().name("id").data_type(DataType::VarChar)),
        )
        .expect_err("duplicate scalar field names must fail");
        assert!(duplicate_scalar
            .to_string()
            .contains("duplicate top-level field name"));

        let scalar_struct_collision = build(
            CollectionSchema::new()
                .add_field(primary_field("events"))
                .add_struct_field(
                    StructFieldSchema::new()
                        .name("events")
                        .max_capacity(8)
                        .add_field(FieldSchema::new().name("kind").data_type(DataType::Int64)),
                ),
        )
        .expect_err("scalar and struct field names must share one namespace");
        assert!(scalar_struct_collision
            .to_string()
            .contains("duplicate top-level field name"));
    }

    #[test]
    fn schema_requires_exactly_one_primary_key() {
        let no_primary_key = CreateCollectionRequest::builder()
            .collection_name("books")
            .schema(
                CollectionSchema::new().add_field(
                    FieldSchema::new()
                        .name("title")
                        .data_type(DataType::VarChar),
                ),
            )
            .build()
            .expect_err("schema without a primary key must fail");
        assert!(no_primary_key
            .to_string()
            .contains("exactly one primary key field, found 0"));

        let multiple_primary_keys = CreateCollectionRequest::builder()
            .collection_name("books")
            .schema(
                CollectionSchema::new()
                    .add_field(primary_field("id"))
                    .add_field(primary_field("alternate_id")),
            )
            .build()
            .expect_err("schema with multiple primary keys must fail");
        assert!(multiple_primary_keys
            .to_string()
            .contains("exactly one primary key field, found 2"));
    }

    #[test]
    fn request_name_is_used_for_the_encoded_schema() {
        let request = CreateCollectionRequest::builder()
            .collection_name("books")
            .description("request description")
            .schema(schema_with_primary().description("schema description"))
            .build()
            .expect("valid request");

        let proto = request.into_proto("default").unwrap();
        let encoded_schema = schema::CollectionSchema::decode(proto.schema.as_slice()).unwrap();
        assert_eq!(proto.collection_name, "books");
        assert_eq!(encoded_schema.name, "books");
        assert_eq!(encoded_schema.description, "request description");
    }

    #[test]
    fn simple_request_converts_to_create_collection_request() {
        let request: CreateCollectionRequest = CreateSimpleCollectionRequest::builder()
            .database_name("catalog")
            .collection_name("books")
            .dimension(128)
            .primary_field("book_id")
            .primary_field_type(DataType::VarChar)
            .max_length(1_024)
            .vector_field("embedding")
            .auto_id(true)
            .enable_dynamic_field(false)
            .metric_type(MetricType::L2)
            .build()
            .expect("valid request")
            .into();

        assert_eq!(request.database_name.as_deref(), Some("catalog"));
        assert_eq!(request.collection_name, "books");
        assert_eq!(request.num_shards, 1);
        let schema = request.schema.as_ref().unwrap();
        assert!(!schema.enable_dynamic_field);
        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[0].name, "book_id");
        assert_eq!(schema.fields[0].data_type, DataType::VarChar);
        assert!(schema.fields[0].is_primary_key);
        assert!(schema.fields[0].auto_id);
        assert_eq!(
            schema.fields[0]
                .type_params
                .get("max_length")
                .map(String::as_str),
            Some("1024")
        );
        assert_eq!(schema.fields[1].name, "embedding");
        assert_eq!(
            schema.fields[1].type_params.get("dim").map(String::as_str),
            Some("128")
        );
        assert_eq!(request.index_params.len(), 1);
        assert_eq!(
            request.index_params[0].get_field_name().to_owned(),
            "embedding"
        );
        assert_eq!(
            request.index_params[0].get_index_type(),
            IndexType::AutoIndex
        );
        assert_eq!(
            request.index_params[0].get_metric_type(),
            Some(MetricType::L2)
        );
    }

    #[test]
    fn simple_request_defaults_to_cosine_autoindex() {
        let simple = CreateSimpleCollectionRequest::builder()
            .collection_name("books")
            .dimension(128)
            .build()
            .expect("valid request");
        assert_eq!(simple.metric_type().to_owned(), MetricType::Cosine);
        assert_eq!(simple.primary_field_type().to_owned(), DataType::Int64);
        assert_eq!(simple.max_length().to_owned(), 65_535);
        assert!(simple.is_dynamic_field_enabled());

        let request: CreateCollectionRequest = simple.into();
        let schema = request.schema.as_ref().unwrap();
        assert!(schema.enable_dynamic_field);
        assert_eq!(schema.fields[0].data_type, DataType::Int64);
        assert_eq!(request.index_params.len(), 1);
        assert_eq!(
            request.index_params[0].get_index_type(),
            IndexType::AutoIndex
        );
        assert_eq!(
            request.index_params[0].get_metric_type(),
            Some(MetricType::Cosine)
        );
    }

    #[test]
    fn simple_request_rejects_unsupported_primary_field_type_before_rpc() {
        let result = CreateSimpleCollectionRequest::builder()
            .collection_name("invalid")
            .dimension(128)
            .primary_field_type(DataType::Int32)
            .build();

        assert!(result.is_err());
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod loading_request_tests {
    use super::{GetLoadStateRequest, LoadCollectionRequest, RefreshLoadRequest};

    #[test]
    fn load_collection_defaults_and_sdk_only_fields() {
        let request = LoadCollectionRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        assert!(request.is_sync());
        assert_eq!(request.timeout_ms().to_owned(), 60_000);
        let proto = request.into_proto("default");
        assert_eq!(proto.db_name, "default");
        assert_eq!(proto.collection_name, "books");
    }

    #[test]
    fn refresh_load_defaults_and_conversion() {
        let request = RefreshLoadRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        assert!(request.sync);
        assert_eq!(request.timeout_ms, 60_000);
        let proto = request.into_proto("default");
        assert_eq!(proto.db_name, "default");
        assert_eq!(proto.collection_name, "books");
        assert!(proto.refresh);
    }

    #[test]
    fn get_load_state_uses_default_database_and_combines_partitions() {
        let request = GetLoadStateRequest::builder()
            .collection_name("books")
            .partition_names(["history", "science", "history"])
            .partition_name("fiction")
            .partition_name("history")
            .build()
            .expect("valid request");

        assert_eq!(
            request.partition_names(),
            &["history", "science", "fiction"]
        );
        let proto = request.into_proto("catalog");
        assert_eq!(proto.db_name, "catalog");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.partition_names, ["history", "science", "fiction"]);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod batch_describe_request_tests {
    use super::BatchDescribeCollectionsRequest;

    #[test]
    fn singular_and_plural_collection_selectors_are_combined() {
        let request = BatchDescribeCollectionsRequest::builder()
            .database_name("catalog")
            .collection_names(["books", "authors"])
            .collection_name("publishers")
            .collection_ids(vec![10, 20])
            .collection_id(30)
            .build()
            .expect("valid request");

        assert_eq!(
            request.database_name().as_deref().to_owned(),
            Some("catalog")
        );
        assert_eq!(
            request.collection_names(),
            &["books", "authors", "publishers"]
        );
        assert_eq!(request.collection_ids().to_owned(), [10, 20, 30]);

        let proto = request.into_proto();
        assert_eq!(proto.db_name, "catalog");
        assert_eq!(proto.collection_name, ["books", "authors", "publishers"]);
        assert_eq!(proto.collection_id, [10, 20, 30]);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod collection_property_request_tests {
    use super::{
        AlterCollectionFieldPropertiesRequest, AlterCollectionPropertiesRequest,
        DropCollectionFieldPropertiesRequest, DropCollectionPropertiesRequest,
    };

    #[test]
    fn alter_and_drop_collection_properties_encode_separate_operations() {
        let alter = AlterCollectionPropertiesRequest::builder()
            .collection_name("books")
            .property("collection.ttl.seconds", "60")
            .build()
            .expect("valid request")
            .into_proto("catalog");
        assert_eq!(alter.db_name, "catalog");
        assert_eq!(alter.properties.len(), 1);
        assert!(alter.delete_keys.is_empty());

        let drop = DropCollectionPropertiesRequest::builder()
            .collection_name("books")
            .property_keys(["test.one", "test.two"])
            .property_key("test.one")
            .build()
            .expect("valid request")
            .into_proto("catalog");
        assert!(drop.properties.is_empty());
        assert_eq!(drop.delete_keys.len(), 2);
        assert!(drop.delete_keys.contains(&"test.one".to_owned()));
        assert!(drop.delete_keys.contains(&"test.two".to_owned()));
    }

    #[test]
    fn alter_and_drop_field_properties_encode_separate_operations() {
        let alter = AlterCollectionFieldPropertiesRequest::builder()
            .collection_name("books")
            .field_name("title")
            .property("max_length", "256")
            .build()
            .expect("valid request")
            .into_proto("catalog");
        assert_eq!(alter.db_name, "catalog");
        assert_eq!(alter.field_name, "title");
        assert_eq!(alter.properties.len(), 1);
        assert!(alter.delete_keys.is_empty());

        let drop = DropCollectionFieldPropertiesRequest::builder()
            .collection_name("books")
            .field_name("title")
            .property_key("max_length")
            .build()
            .expect("valid request")
            .into_proto("catalog");
        assert!(drop.properties.is_empty());
        assert_eq!(drop.delete_keys, ["max_length"]);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn create_collection_request_default_values() {
        let value = CreateCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_description: Option<String> = None;
        let expected_schema: Option<CollectionSchema> = None;
        let expected_num_partitions: i64 = 0;
        let expected_num_shards: i32 = 1;
        let expected_consistency_level: ConsistencyLevel = ConsistencyLevel::Bounded;
        let expected_index_params: Vec<IndexParam> = Default::default();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.description().to_owned(), expected_description);
        assert_eq!(value.schema().to_owned(), expected_schema);
        assert_eq!(value.num_partitions().to_owned(), expected_num_partitions);
        assert_eq!(value.num_shards().to_owned(), expected_num_shards);
        assert_eq!(
            value.consistency_level().to_owned(),
            expected_consistency_level
        );
        assert_eq!(value.index_params().to_owned(), expected_index_params);
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn create_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let description = "description-value".to_owned();
        let schema = CollectionSchema::new().description("schema").add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        );
        let num_partitions = 7;
        let num_shards = 7;
        let consistency_level = ConsistencyLevel::Strong;
        let index_params = vec![IndexParam::new()
            .field_name("field")
            .index_type(IndexType::Invalid)
            .metric_type(MetricType::Default)
            .index_name("index")];
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = CreateCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .description(description.clone())
            .schema(schema.clone())
            .num_partitions(num_partitions.clone())
            .num_shards(num_shards.clone())
            .consistency_level(consistency_level.clone())
            .index_params(index_params.clone())
            .properties(properties.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.description().to_owned(), Some(description));
        assert_eq!(value.schema().to_owned(), Some(schema));
        assert_eq!(value.num_partitions().to_owned(), num_partitions);
        assert_eq!(value.num_shards().to_owned(), num_shards);
        assert_eq!(value.consistency_level().to_owned(), consistency_level);
        assert_eq!(value.index_params().to_owned(), index_params);
        assert_eq!(value.properties().to_owned(), properties);
    }

    #[test]
    fn create_simple_collection_request_default_values() {
        let value = CreateSimpleCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_dimension: u32 = 0;
        let expected_primary_field: String = "id".to_owned();
        let expected_primary_field_type: DataType = DataType::Int64;
        let expected_max_length: u32 = 65_535;
        let expected_vector_field: String = "vector".to_owned();
        let expected_auto_id: bool = false;
        let expected_enable_dynamic_field: bool = true;
        let expected_consistency_level: ConsistencyLevel = ConsistencyLevel::Bounded;
        let expected_metric_type: MetricType = MetricType::Cosine;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.dimension().to_owned(), expected_dimension);
        assert_eq!(value.primary_field().to_owned(), expected_primary_field);
        assert_eq!(
            value.primary_field_type().to_owned(),
            expected_primary_field_type
        );
        assert_eq!(value.max_length().to_owned(), expected_max_length);
        assert_eq!(value.vector_field().to_owned(), expected_vector_field);
        assert_eq!(value.is_auto_id().to_owned(), expected_auto_id);
        assert_eq!(
            value.is_dynamic_field_enabled(),
            expected_enable_dynamic_field
        );
        assert_eq!(
            value.consistency_level().to_owned(),
            expected_consistency_level
        );
        assert_eq!(value.metric_type().to_owned(), expected_metric_type);
    }

    #[test]
    fn create_simple_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let dimension = 7;
        let primary_field = "primary_field-value".to_owned();
        let primary_field_type = DataType::VarChar;
        let max_length = 7;
        let vector_field = "vector_field-value".to_owned();
        let auto_id = true;
        let enable_dynamic_field = true;
        let consistency_level = ConsistencyLevel::Strong;
        let metric_type = MetricType::Cosine;
        let value = CreateSimpleCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .dimension(dimension.clone())
            .primary_field(primary_field.clone())
            .primary_field_type(primary_field_type.clone())
            .max_length(max_length.clone())
            .vector_field(vector_field.clone())
            .auto_id(auto_id.clone())
            .enable_dynamic_field(enable_dynamic_field.clone())
            .consistency_level(consistency_level.clone())
            .metric_type(metric_type.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.dimension().to_owned(), dimension);
        assert_eq!(value.primary_field().to_owned(), primary_field);
        assert_eq!(value.primary_field_type().to_owned(), primary_field_type);
        assert_eq!(value.max_length().to_owned(), max_length);
        assert_eq!(value.vector_field().to_owned(), vector_field);
        assert_eq!(value.is_auto_id().to_owned(), auto_id);
        assert_eq!(
            value.is_dynamic_field_enabled().to_owned(),
            enable_dynamic_field
        );
        assert_eq!(value.consistency_level().to_owned(), consistency_level);
        assert_eq!(value.metric_type().to_owned(), metric_type);
    }

    #[test]
    fn drop_collection_request_default_values() {
        let value = DropCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn drop_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = DropCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn has_collection_request_default_values() {
        let value = HasCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn has_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = HasCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn release_collection_request_default_values() {
        let value = ReleaseCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn release_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = ReleaseCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn describe_collection_request_default_values() {
        let value = DescribeCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn describe_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = DescribeCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn load_collection_request_default_values() {
        let value = LoadCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_sync: bool = true;
        let expected_replica_number: i32 = 1;
        let expected_refresh: bool = false;
        let expected_load_fields: Vec<String> = Default::default();
        let expected_skip_load_dynamic_field: bool = false;
        let expected_resource_groups: Vec<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.is_sync().to_owned(), expected_sync);
        assert_eq!(value.replica_number().to_owned(), expected_replica_number);
        assert_eq!(value.should_refresh().to_owned(), expected_refresh);
        assert_eq!(value.load_fields().to_owned(), expected_load_fields);
        assert_eq!(
            value.should_skip_load_dynamic_field(),
            expected_skip_load_dynamic_field
        );
        assert_eq!(value.resource_groups().to_owned(), expected_resource_groups);
    }

    #[test]
    fn load_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let sync = true;
        let replica_number = 7;
        let refresh = true;
        let load_fields = vec!["load_fields-value".to_owned()];
        let skip_load_dynamic_field = true;
        let resource_groups = vec!["resource_groups-value".to_owned()];
        let value = LoadCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .sync(sync.clone())
            .replica_number(replica_number.clone())
            .refresh(refresh.clone())
            .load_fields(load_fields.clone())
            .skip_load_dynamic_field(skip_load_dynamic_field.clone())
            .resource_groups(resource_groups.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.is_sync().to_owned(), sync);
        assert_eq!(value.replica_number().to_owned(), replica_number);
        assert_eq!(value.should_refresh().to_owned(), refresh);
        assert_eq!(value.load_fields().to_owned(), load_fields);
        assert_eq!(
            value.should_skip_load_dynamic_field(),
            skip_load_dynamic_field
        );
        assert_eq!(value.resource_groups().to_owned(), resource_groups);
    }

    #[test]
    fn refresh_load_request_default_values() {
        let value = RefreshLoadRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_sync: bool = true;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.is_sync().to_owned(), expected_sync);
    }

    #[test]
    fn refresh_load_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let sync = true;
        let value = RefreshLoadRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .sync(sync.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.is_sync().to_owned(), sync);
    }

    #[test]
    fn batch_describe_collections_request_default_values() {
        let value = BatchDescribeCollectionsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_names: Vec<String> = Default::default();
        let expected_collection_ids: Vec<i64> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(
            value.collection_names().to_owned(),
            expected_collection_names
        );
        assert_eq!(value.collection_ids().to_owned(), expected_collection_ids);
    }

    #[test]
    fn batch_describe_collections_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_names = vec!["collection_names-value".to_owned()];
        let collection_ids = vec![7];
        let value = BatchDescribeCollectionsRequest::builder()
            .database_name(database_name.clone())
            .collection_names(collection_names.clone())
            .collection_ids(collection_ids.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_names().to_owned(), collection_names);
        assert_eq!(value.collection_ids().to_owned(), collection_ids);
    }

    #[test]
    fn get_collection_stats_request_default_values() {
        let value = GetCollectionStatsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn get_collection_stats_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = GetCollectionStatsRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn list_collections_request_default_values() {
        let value = ListCollectionsRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_only_loaded: bool = false;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.is_only_loaded().to_owned(), expected_only_loaded);
    }

    #[test]
    fn list_collections_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let only_loaded = true;
        let value = ListCollectionsRequest::builder()
            .database_name(database_name.clone())
            .only_loaded(only_loaded.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.is_only_loaded().to_owned(), only_loaded);
    }

    #[test]
    fn get_load_state_request_default_values() {
        let value = GetLoadStateRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_names: Vec<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
    }

    #[test]
    fn get_load_state_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_names = vec!["partition_names-value".to_owned()];
        let value = GetLoadStateRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_names(partition_names.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_names().to_owned(), partition_names);
    }

    #[test]
    fn alter_collection_properties_request_default_values() {
        let value = AlterCollectionPropertiesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn alter_collection_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = AlterCollectionPropertiesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .properties(properties.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.properties().to_owned(), properties);
    }

    #[test]
    fn drop_collection_properties_request_default_values() {
        let value = DropCollectionPropertiesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_property_keys: HashSet<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.property_keys().to_owned(), expected_property_keys);
    }

    #[test]
    fn drop_collection_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let property_keys = HashSet::from(["property_keys-value".to_owned()]);
        let value = DropCollectionPropertiesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .property_keys(property_keys.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.property_keys().to_owned(), property_keys);
    }

    #[test]
    fn alter_collection_field_properties_request_default_values() {
        let value = AlterCollectionFieldPropertiesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field_name: String = String::new();
        let expected_properties: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field_name().to_owned(), expected_field_name);
        assert_eq!(value.properties().to_owned(), expected_properties);
    }

    #[test]
    fn alter_collection_field_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field_name = "field_name-value".to_owned();
        let properties = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = AlterCollectionFieldPropertiesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field_name(field_name.clone())
            .properties(properties.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field_name().to_owned(), field_name);
        assert_eq!(value.properties().to_owned(), properties);
    }

    #[test]
    fn drop_collection_field_properties_request_default_values() {
        let value = DropCollectionFieldPropertiesRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field_name: String = String::new();
        let expected_property_keys: HashSet<String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field_name().to_owned(), expected_field_name);
        assert_eq!(value.property_keys().to_owned(), expected_property_keys);
    }

    #[test]
    fn drop_collection_field_properties_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field_name = "field_name-value".to_owned();
        let property_keys = HashSet::from(["property_keys-value".to_owned()]);
        let value = DropCollectionFieldPropertiesRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field_name(field_name.clone())
            .property_keys(property_keys.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field_name().to_owned(), field_name);
        assert_eq!(value.property_keys().to_owned(), property_keys);
    }

    #[test]
    fn add_collection_field_request_default_values() {
        let value = AddCollectionFieldRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_field: Option<&FieldSchema> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.field(), expected_field);
    }

    #[test]
    fn add_collection_field_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let field = FieldSchema::new().name("field").data_type(DataType::Int64);
        let value = AddCollectionFieldRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .field(field.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.field(), Some(&field));
    }

    #[test]
    #[allow(deprecated)]
    fn add_collection_function_request_default_values() {
        let value = AddCollectionFunctionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_function: Option<Function> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.function().to_owned(), expected_function);
    }

    #[test]
    #[allow(deprecated)]
    fn add_collection_function_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let function = Function::new()
            .name("function")
            .function_type(crate::v2::FunctionType::Bm25);
        let value = AddCollectionFunctionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .function(function.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.function().to_owned(), Some(function));
    }

    #[test]
    fn alter_collection_function_request_default_values() {
        let value = AlterCollectionFunctionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_function: Option<Function> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.function().to_owned(), expected_function);
    }

    #[test]
    fn alter_collection_function_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let function = Function::new()
            .name("function")
            .function_type(crate::v2::FunctionType::Bm25);
        let value = AlterCollectionFunctionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .function(function.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.function().to_owned(), Some(function));
        assert_eq!(value.into_proto().function_name, "function");
    }

    #[test]
    #[allow(deprecated)]
    fn drop_collection_function_request_default_values() {
        let value = DropCollectionFunctionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_function_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.function_name().to_owned(), expected_function_name);
    }

    #[test]
    #[allow(deprecated)]
    fn drop_collection_function_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let function_name = "function_name-value".to_owned();
        let value = DropCollectionFunctionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .function_name(function_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.function_name().to_owned(), function_name);
    }

    #[test]
    fn truncate_collection_request_default_values() {
        let value = TruncateCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
    }

    #[test]
    fn truncate_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let value = TruncateCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
    }

    #[test]
    fn describe_replicas_request_default_values() {
        let value = DescribeReplicasRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_with_shard_nodes: bool = false;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(
            value.should_include_shard_nodes().to_owned(),
            expected_with_shard_nodes
        );
    }

    #[test]
    fn describe_replicas_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let with_shard_nodes = true;
        let value = DescribeReplicasRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .with_shard_nodes(with_shard_nodes.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(
            value.should_include_shard_nodes().to_owned(),
            with_shard_nodes
        );
    }

    #[test]
    fn rename_collection_request_default_values() {
        let value = RenameCollectionRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_new_collection_name: String = String::new();
        let expected_new_database_name: Option<String> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.new_collection_name(), &expected_new_collection_name);
        assert_eq!(
            value.new_database_name().to_owned(),
            expected_new_database_name
        );
    }

    #[test]
    fn rename_collection_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let new_collection_name = "new_collection_name-value".to_owned();
        let new_database_name = "new_database_name-value".to_owned();
        let value = RenameCollectionRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .new_collection_name(new_collection_name.clone())
            .new_database_name(new_database_name.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.new_collection_name().to_owned(), new_collection_name);
        assert_eq!(
            value.new_database_name().to_owned(),
            Some(new_database_name)
        );
    }

    #[test]
    fn rename_collection_request_defaults_new_database_to_source_database() {
        let value = RenameCollectionRequest::builder()
            .database_name("catalog")
            .collection_name("books")
            .new_collection_name("renamed_books")
            .build()
            .expect("valid request");

        let proto = value.into_proto("default");

        assert_eq!(proto.db_name, "catalog");
        assert_eq!(proto.new_db_name, "catalog");
    }
}

#[cfg(test)]
mod function_struct_field_request_tests {
    use super::*;
    use crate::proto::milvus::alter_collection_schema_request as req;
    use crate::v2::types::{DataType, FieldSchema, Function, IndexParam, IndexType, MetricType};

    #[test]
    fn add_collection_struct_field_requires_nullable() {
        let struct_field = StructFieldSchema::new()
            .name("obj")
            .max_capacity(10)
            .add_field(FieldSchema::new().name("a").data_type(DataType::Int64));

        let error = AddCollectionStructFieldRequest::builder()
            .collection_name("books")
            .struct_field(struct_field)
            .build()
            .expect_err("non-nullable struct field must be rejected");
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn add_collection_struct_field_builds_proto() {
        let struct_field = StructFieldSchema::new()
            .name("obj")
            .max_capacity(10)
            .nullable(true)
            .add_field(FieldSchema::new().name("a").data_type(DataType::Int64));

        let value = AddCollectionStructFieldRequest::builder()
            .collection_name("books")
            .struct_field(struct_field)
            .build()
            .expect("valid request");

        let proto = value.into_proto().expect("proto");
        assert_eq!(proto.collection_name, "books");
        let schema = proto.struct_array_field_schema.expect("struct schema");
        assert_eq!(schema.name, "obj");
        assert!(schema.nullable);
    }

    #[test]
    fn add_function_field_rejects_mismatched_output_type() {
        let field = FieldSchema::new()
            .name("sparse")
            .data_type(DataType::FloatVector);
        let function = Function::new()
            .name("f")
            .function_type(FunctionType::Bm25)
            .input_fields(vec!["text".to_owned()]);
        let index = IndexParam::new()
            .field_name("sparse")
            .index_type(IndexType::SparseInvertedIndex)
            .metric_type(MetricType::Bm25);

        let error = AddFunctionFieldRequest::builder()
            .collection_name("books")
            .field(field)
            .function(function)
            .index(index)
            .build()
            .expect_err("mismatched output type must be rejected");
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn add_function_field_rejects_unsupported_function_type() {
        let function = Function::new()
            .name("f")
            .function_type(FunctionType::TextEmbedding)
            .input_fields(vec!["text".to_owned()]);
        let index = IndexParam::new()
            .field_name("dense")
            .index_type(IndexType::AutoIndex)
            .metric_type(MetricType::L2);

        let error = AddFunctionFieldRequest::builder()
            .collection_name("books")
            .field(
                FieldSchema::new()
                    .name("dense")
                    .data_type(DataType::FloatVector)
                    .dimension(4),
            )
            .function(function)
            .index(index)
            .build()
            .expect_err("unsupported function type must be rejected");
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn add_function_field_rejects_invalid_field() {
        let field = FieldSchema::new()
            .name("sparse")
            .data_type(DataType::BinaryVector)
            .primary_key(true);
        let function = Function::new()
            .name("f")
            .function_type(FunctionType::MinHash)
            .input_fields(vec!["text".to_owned()]);
        let index = IndexParam::new().field_name("sparse");

        let error = AddFunctionFieldRequest::builder()
            .collection_name("books")
            .field(field)
            .function(function)
            .index(index)
            .build()
            .expect_err("invalid field must be rejected in build()");
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn add_function_field_requires_explicit_index_type() {
        let field = FieldSchema::new()
            .name("sparse")
            .data_type(DataType::SparseFloatVector);
        let function = Function::new()
            .name("f")
            .function_type(FunctionType::Bm25)
            .input_fields(vec!["text".to_owned()]);
        let index = IndexParam::new().field_name("sparse");

        let error = AddFunctionFieldRequest::builder()
            .collection_name("books")
            .field(field)
            .function(function)
            .index(index)
            .build()
            .expect_err("missing index type must be rejected");
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn add_function_field_builds_proto() {
        let field = FieldSchema::new()
            .name("sparse")
            .data_type(DataType::SparseFloatVector);
        let function = Function::new()
            .name("f")
            .function_type(FunctionType::Bm25)
            .input_fields(vec!["text".to_owned()])
            .output_fields(vec!["sparse".to_owned()]);
        let index = IndexParam::new()
            .field_name("sparse")
            .index_name("sparse_idx")
            .index_type(IndexType::SparseInvertedIndex)
            .metric_type(MetricType::Bm25);

        let value = AddFunctionFieldRequest::builder()
            .collection_name("books")
            .field(field)
            .function(function)
            .index(index)
            .build()
            .expect("valid request");

        let proto = value.into_proto().expect("proto");
        assert_eq!(proto.collection_name, "books");
        let action = proto.action.expect("action");
        match action.op.expect("op") {
            req::action::Op::AddRequest(add) => {
                assert_eq!(add.field_infos.len(), 1);
                assert_eq!(add.field_infos[0].index_name, "sparse_idx");
                assert!(add.field_infos[0]
                    .extra_params
                    .iter()
                    .any(|pair| pair.key == "index_type" && pair.value == "SPARSE_INVERTED_INDEX"));
                assert_eq!(add.func_schema.len(), 1);
                assert_eq!(add.func_schema[0].name, "f");
            }
            req::action::Op::DropRequest(_) => panic!("expected add action"),
        }
    }

    #[test]
    fn drop_function_field_builds_proto() {
        let value = DropFunctionFieldRequest::builder()
            .collection_name("books")
            .function_name("f")
            .build()
            .expect("valid request");

        let proto = value.into_proto().expect("proto");
        let action = proto.action.expect("action");
        match action.op.expect("op") {
            req::action::Op::DropRequest(drop) => {
                assert!(drop.drop_function_output_fields);
                assert!(matches!(
                    drop.identifier,
                    Some(req::drop_request::Identifier::FunctionName(ref name)) if name == "f"
                ));
            }
            req::action::Op::AddRequest(_) => panic!("expected drop action"),
        }
    }

    #[test]
    fn drop_collection_field_requires_exactly_one_identifier() {
        let error = DropCollectionFieldRequest::builder()
            .collection_name("books")
            .build()
            .expect_err("no identifier must be rejected");
        assert!(matches!(error, Error::Validation(_)));

        let error = DropCollectionFieldRequest::builder()
            .collection_name("books")
            .field_name("a")
            .field_id(1)
            .build()
            .expect_err("both identifiers must be rejected");
        assert!(matches!(error, Error::Validation(_)));
    }

    #[test]
    fn drop_collection_field_builds_proto() {
        let by_name = DropCollectionFieldRequest::builder()
            .collection_name("books")
            .field_name("a")
            .build()
            .expect("valid request");
        match by_name
            .into_proto()
            .expect("proto")
            .action
            .expect("action")
            .op
            .expect("op")
        {
            req::action::Op::DropRequest(drop) => {
                assert!(matches!(
                    drop.identifier,
                    Some(req::drop_request::Identifier::FieldName(ref name)) if name == "a"
                ));
            }
            req::action::Op::AddRequest(_) => panic!("expected drop action"),
        }

        let by_id = DropCollectionFieldRequest::builder()
            .collection_name("books")
            .field_id(7)
            .build()
            .expect("valid request");
        match by_id
            .into_proto()
            .expect("proto")
            .action
            .expect("action")
            .op
            .expect("op")
        {
            req::action::Op::DropRequest(drop) => {
                assert!(matches!(
                    drop.identifier,
                    Some(req::drop_request::Identifier::FieldId(7))
                ));
            }
            req::action::Op::AddRequest(_) => panic!("expected drop action"),
        }
    }

    #[test]
    fn drop_collection_field_empty_name_falls_back_to_field_id() {
        // build() treats an empty field_name as absent; into_proto() must agree so the
        // wire identifier matches the validated field_id.
        let value = DropCollectionFieldRequest::builder()
            .collection_name("books")
            .field_name("")
            .field_id(7)
            .build()
            .expect("empty name counts as absent");
        match value
            .into_proto()
            .expect("proto")
            .action
            .expect("action")
            .op
            .expect("op")
        {
            req::action::Op::DropRequest(drop) => {
                assert!(matches!(
                    drop.identifier,
                    Some(req::drop_request::Identifier::FieldId(7))
                ));
            }
            req::action::Op::AddRequest(_) => panic!("expected drop action"),
        }
    }
}
