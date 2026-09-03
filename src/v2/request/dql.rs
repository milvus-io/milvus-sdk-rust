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

//! Request types for query, get, search, hybrid search, and iterators.
//!
//! Choose [`QueryRequest`] for scalar/filter reads and [`SearchRequest`] for nearest-neighbor
//! vector search. Both builders require a collection name; search additionally requires query
//! vectors and an approximate-nearest-neighbor field. Set `output_fields` to select returned
//! fields, and use `limit` to control the number of rows per query vector.

use crate::proto::{common, milvus, schema};
use crate::v2::error::{Error, Result};
use crate::v2::request::dml::json_template;
use crate::v2::request::validation::{
    non_empty_strings, non_negative_i64, positive_i64, positive_usize, required, required_slice,
};
pub use crate::v2::types::Ids;
use crate::v2::types::{
    encode_sparse_vector, validate_sparse_vector, ConsistencyLevel, Function, FunctionChain,
    FunctionScore, MetricType, QueryCursor, SearchAggregation,
};
pub use crate::v2::types::{
    EmbeddingList, HighlightQuery, HighlightType, Highlighter, LexicalHighlighter, SearchVectors,
    SemanticHighlighter,
};
use prost::Message;
use serde_json::Value;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// QueryRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 query operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct QueryRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
    pub(crate) ids: Ids,
    pub(crate) filter: String,
    pub(crate) filter_templates: HashMap<String, Value>,
    pub(crate) output_fields: Vec<String>,
    pub(crate) limit: Option<i64>,
    pub(crate) offset: Option<i64>,
    pub(crate) ignore_growing: bool,
    pub(crate) timezone: String,
    pub(crate) consistency_level: Option<ConsistencyLevel>,
    pub(crate) extra_params: HashMap<String, String>,
}

impl QueryRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_names: Default::default(),
            ids: Default::default(),
            filter: Default::default(),
            filter_templates: Default::default(),
            output_fields: Default::default(),
            limit: Default::default(),
            offset: Default::default(),
            ignore_growing: Default::default(),
            timezone: Default::default(),
            consistency_level: Default::default(),
            extra_params: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> QueryRequestBuilder {
        QueryRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> QueryRequestBuilder {
        QueryRequestBuilder { value: self }
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

    /// Returns the primary keys selected by this query.
    pub fn ids(&self) -> &Ids {
        &self.ids
    }

    /// Returns the filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns the filter templates.
    pub fn filter_templates(&self) -> &HashMap<String, Value> {
        &self.filter_templates
    }

    /// Returns the output fields.
    pub fn output_fields(&self) -> &[String] {
        &self.output_fields
    }

    /// Returns the limit.
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// Returns the offset.
    pub fn offset(&self) -> Option<i64> {
        self.offset
    }

    /// Returns whether the request should ignore growing.
    pub fn should_ignore_growing(&self) -> bool {
        self.ignore_growing
    }

    /// Returns the timezone.
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    /// Returns the consistency level.
    pub fn consistency_level(&self) -> Option<ConsistencyLevel> {
        self.consistency_level
    }

    /// Returns the extra params.
    pub fn extra_params(&self) -> &HashMap<String, String> {
        &self.extra_params
    }

    pub(crate) fn into_proto(
        self,
        default_db: &str,
        primary_field: Option<&str>,
        guarantee_timestamp: u64,
    ) -> Result<milvus::QueryRequest> {
        let mut params = self.extra_params;
        if let Some(limit) = self.limit {
            params.insert("limit".into(), limit.to_string());
        }
        if let Some(offset) = self.offset {
            params.insert("offset".into(), offset.to_string());
        }
        if self.ignore_growing {
            params.insert("ignore_growing".into(), "true".into());
        }
        if !self.timezone.is_empty() {
            params.insert("timezone".into(), self.timezone);
        }
        let (filter, filter_templates) = if self.ids.is_empty() {
            (self.filter, self.filter_templates)
        } else {
            let primary_field = primary_field.ok_or_else(|| {
                Error::validation(
                    "ids".into(),
                    "the collection primary-key field is required to encode query IDs".into(),
                )
            })?;
            let template_name = "__milvus_v2_query_ids";
            (
                format!("{primary_field} in {{{template_name}}}"),
                HashMap::from([(template_name.to_owned(), self.ids.into_json())]),
            )
        };
        Ok(milvus::QueryRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            expr: filter,
            output_fields: self.output_fields,
            partition_names: self.partition_names,
            travel_timestamp: 0,
            guarantee_timestamp,
            query_params: params
                .into_iter()
                .map(|(key, value)| common::KeyValuePair {
                    key,
                    value,
                    ..Default::default()
                })
                .collect(),
            not_return_all_meta: false,
            consistency_level: self
                .consistency_level
                .map(|level| level.into_proto() as i32)
                .unwrap_or_default(),
            use_default_consistency: self.consistency_level.is_none(),
            expr_template_values: filter_templates
                .into_iter()
                .map(|(key, value)| Ok((key, json_template(value)?)))
                .collect::<Result<_>>()?,
            namespace: None,
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// QueryRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for QueryRequest.
#[derive(Debug, Clone)]
pub struct QueryRequestBuilder {
    value: QueryRequest,
}

impl QueryRequestBuilder {
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
        self.value.partition_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Queries entities with the supplied Int64 or VarChar primary keys.
    pub fn ids(mut self, value: Ids) -> Self {
        self.value.ids = value;
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

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.output_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the limit and returns the updated value.
    pub fn limit(mut self, value: i64) -> Self {
        self.value.limit = Some(value);
        self
    }

    /// Sets the offset and returns the updated value.
    pub fn offset(mut self, value: i64) -> Self {
        self.value.offset = Some(value);
        self
    }

    /// Sets the ignore growing and returns the updated value.
    pub fn ignore_growing(mut self, value: bool) -> Self {
        self.value.ignore_growing = value;
        self
    }

    /// Sets the timezone and returns the updated value.
    pub fn timezone(mut self, value: impl Into<String>) -> Self {
        self.value.timezone = value.into();
        self
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.value.consistency_level = Some(value);
        self
    }

    /// Sets the extra params and returns the updated value.
    pub fn extra_params(mut self, value: HashMap<String, String>) -> Self {
        self.value.extra_params = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<QueryRequest> {
        validate_query_request(&self.value)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GetRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
    pub(crate) ids: Ids,
    pub(crate) output_fields: Vec<String>,
    pub(crate) consistency_level: Option<ConsistencyLevel>,
}

impl GetRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            partition_names: Default::default(),
            ids: Default::default(),
            output_fields: Default::default(),
            consistency_level: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetRequestBuilder {
        GetRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetRequestBuilder {
        GetRequestBuilder { value: self }
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

    /// Returns the ids.
    pub fn ids(&self) -> &Ids {
        &self.ids
    }

    /// Returns the output fields.
    pub fn output_fields(&self) -> &[String] {
        &self.output_fields
    }

    /// Returns the consistency level.
    pub fn consistency_level(&self) -> Option<ConsistencyLevel> {
        self.consistency_level
    }

    pub(crate) fn into_proto(
        self,
        default_db: &str,
        primary_field: &str,
        guarantee_timestamp: u64,
    ) -> Result<milvus::QueryRequest> {
        let ids = match self.ids {
            Ids::Int64(values) => Value::Array(values.into_iter().map(Value::from).collect()),
            Ids::VarChar(values) => Value::Array(values.into_iter().map(Value::from).collect()),
        };
        let template_name = "pks_to_get";
        Ok(milvus::QueryRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            expr: format!("{primary_field} in {{{template_name}}}"),
            output_fields: self.output_fields,
            partition_names: self.partition_names,
            travel_timestamp: 0,
            guarantee_timestamp,
            query_params: Vec::new(),
            not_return_all_meta: false,
            consistency_level: self
                .consistency_level
                .map(|level| level.into_proto() as i32)
                .unwrap_or_default(),
            use_default_consistency: self.consistency_level.is_none(),
            expr_template_values: HashMap::from([(template_name.to_owned(), json_template(ids)?)]),
            namespace: None,
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetRequest.
#[derive(Debug, Clone)]
pub struct GetRequestBuilder {
    value: GetRequest,
}

impl GetRequestBuilder {
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
        self.value.partition_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the ids and returns the updated value.
    pub fn ids(mut self, value: Ids) -> Self {
        self.value.ids = value;
        self
    }

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.output_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.value.consistency_level = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// Empty ids are allowed: the client short-circuits a get with no ids into an
    /// empty result without issuing the RPC, matching pymilvus.
    pub fn build(self) -> Result<GetRequest> {
        required("collection_name", &self.value.collection_name)?;
        non_empty_strings("partition_names", &self.value.partition_names)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 search operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SearchRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) ids: Ids,
    pub(crate) vector_field: String,
    pub(crate) vectors: SearchVectors,
    pub(crate) partition_names: Vec<String>,
    pub(crate) filter: String,
    pub(crate) filter_templates: HashMap<String, Value>,
    pub(crate) output_fields: Vec<String>,
    pub(crate) limit: i64,
    pub(crate) offset: i64,
    pub(crate) round_decimal: i64,
    pub(crate) ignore_growing: bool,
    pub(crate) group_by_field: String,
    pub(crate) group_size: i64,
    pub(crate) strict_group_size: bool,
    pub(crate) radius: Option<f64>,
    pub(crate) range_filter: Option<f64>,
    pub(crate) metric_type: Option<MetricType>,
    pub(crate) extra_params: HashMap<String, String>,
    pub(crate) rerank: Option<FunctionScore>,
    pub(crate) timezone: String,
    pub(crate) highlighter: Option<Highlighter>,
    pub(crate) consistency_level: Option<ConsistencyLevel>,
    pub(crate) function_chains: Vec<FunctionChain>,
    pub(crate) search_aggregation: Option<SearchAggregation>,
}

impl SearchRequest {
    /// Creates a builder for this request.
    pub fn builder() -> SearchRequestBuilder {
        SearchRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> SearchRequestBuilder {
        SearchRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the primary keys whose vectors are used as search targets.
    pub fn ids(&self) -> &Ids {
        &self.ids
    }

    /// Returns the vector field.
    pub fn vector_field(&self) -> &str {
        &self.vector_field
    }

    /// Returns the vectors.
    pub fn vectors(&self) -> &SearchVectors {
        &self.vectors
    }

    /// Returns the partition names.
    pub fn partition_names(&self) -> &[String] {
        &self.partition_names
    }

    /// Returns the filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns the filter templates.
    pub fn filter_templates(&self) -> &HashMap<String, Value> {
        &self.filter_templates
    }

    /// Returns the output fields.
    pub fn output_fields(&self) -> &[String] {
        &self.output_fields
    }

    /// Returns the limit.
    pub fn limit(&self) -> i64 {
        self.limit
    }

    /// Returns the offset.
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// Returns the round decimal.
    pub fn round_decimal(&self) -> i64 {
        self.round_decimal
    }

    /// Returns whether the request should ignore growing.
    pub fn should_ignore_growing(&self) -> bool {
        self.ignore_growing
    }

    /// Returns the group by field.
    pub fn group_by_field(&self) -> &str {
        &self.group_by_field
    }

    /// Returns the group size.
    pub fn group_size(&self) -> i64 {
        self.group_size
    }

    /// Returns whether strict group size.
    pub fn is_strict_group_size(&self) -> bool {
        self.strict_group_size
    }

    /// Returns the radius.
    pub fn radius(&self) -> Option<f64> {
        self.radius
    }

    /// Returns the range filter.
    pub fn range_filter(&self) -> Option<f64> {
        self.range_filter
    }

    /// Returns the metric type.
    pub fn metric_type(&self) -> Option<MetricType> {
        self.metric_type
    }

    /// Returns the extra params.
    pub fn extra_params(&self) -> &HashMap<String, String> {
        &self.extra_params
    }

    /// Returns the rerank.
    pub fn rerank(&self) -> &Option<FunctionScore> {
        &self.rerank
    }

    /// Returns the timezone.
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    /// Returns the highlighter.
    pub fn highlighter(&self) -> &Option<Highlighter> {
        &self.highlighter
    }

    /// Returns the consistency level.
    pub fn consistency_level(&self) -> Option<ConsistencyLevel> {
        self.consistency_level
    }

    /// Returns the function chains.
    pub fn function_chains(&self) -> &[FunctionChain] {
        &self.function_chains
    }

    /// Returns the search aggregation.
    pub fn search_aggregation(&self) -> &Option<SearchAggregation> {
        &self.search_aggregation
    }

    #[allow(deprecated)]
    pub(crate) fn into_proto(
        self,
        default_db: &str,
        guarantee_timestamp: u64,
    ) -> Result<milvus::SearchRequest> {
        if self.limit <= 0 {
            return Err(Error::validation(
                "limit".into(),
                "must be greater than zero".into(),
            ));
        }
        if self.group_size <= 0 {
            return Err(Error::validation(
                "group_size".into(),
                "must be greater than zero".into(),
            ));
        }
        let search_by_ids = !self.ids.is_empty();
        let (search_input, nq) = if search_by_ids {
            let nq = self.ids.len() as i64;
            let ids = match self.ids {
                Ids::Int64(values) => schema::IDs {
                    id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                        data: values,
                    })),
                    ..Default::default()
                },
                Ids::VarChar(values) => schema::IDs {
                    id_field: Some(schema::i_ds::IdField::StrId(schema::StringArray {
                        data: values,
                    })),
                    ..Default::default()
                },
            };
            (milvus::search_request::SearchInput::Ids(ids), nq)
        } else {
            let (placeholder_type, values, nq) = match self.vectors {
                SearchVectors::Float(vectors) => {
                    validate_float_search_vectors("vectors", &vectors)?;
                    (
                        common::PlaceholderType::FloatVector,
                        vectors
                            .into_iter()
                            .map(|v| v.into_iter().flat_map(f32::to_le_bytes).collect())
                            .collect::<Vec<_>>(),
                        None,
                    )
                }
                SearchVectors::Binary(vectors) => {
                    validate_dense_search_vectors("vectors", &vectors)?;
                    (common::PlaceholderType::BinaryVector, vectors, None)
                }
                SearchVectors::Float16(vectors) => {
                    validate_dense_search_vectors("vectors", &vectors)?;
                    (
                        common::PlaceholderType::Float16Vector,
                        encode_u16_search_vectors(vectors),
                        None,
                    )
                }
                SearchVectors::BFloat16(vectors) => {
                    validate_dense_search_vectors("vectors", &vectors)?;
                    (
                        common::PlaceholderType::BFloat16Vector,
                        encode_u16_search_vectors(vectors),
                        None,
                    )
                }
                SearchVectors::SparseFloat(vectors) => (
                    common::PlaceholderType::SparseFloatVector,
                    vectors
                        .into_iter()
                        .map(|values| encode_sparse_vector("vectors", values))
                        .collect::<Result<Vec<_>>>()?,
                    None,
                ),
                SearchVectors::Int8(vectors) => {
                    validate_dense_search_vectors("vectors", &vectors)?;
                    (
                        common::PlaceholderType::Int8Vector,
                        vectors
                            .into_iter()
                            .map(|v| v.into_iter().map(|x| x as u8).collect())
                            .collect::<Vec<_>>(),
                        None,
                    )
                }
                SearchVectors::EmbeddedText(values) => (
                    common::PlaceholderType::VarChar,
                    values.into_iter().map(String::into_bytes).collect(),
                    None,
                ),
                SearchVectors::EmbeddingLists(lists) => {
                    let nq = lists.len() as i64;
                    let mut values = Vec::with_capacity(lists.len());
                    for list in lists {
                        validate_float_search_vectors("embedding_lists", &list.vectors)?;
                        values.push(
                            list.vectors
                                .into_iter()
                                .flatten()
                                .flat_map(f32::to_le_bytes)
                                .collect(),
                        );
                    }
                    (
                        common::PlaceholderType::EmbListFloatVector,
                        values,
                        Some(nq),
                    )
                }
            };
            if values.is_empty() {
                return Err(Error::validation(
                    "vectors".into(),
                    "at least one query vector is required".into(),
                ));
            }
            let nq = nq.unwrap_or(values.len() as i64);
            let group = common::PlaceholderGroup {
                placeholders: vec![common::PlaceholderValue {
                    tag: "$0".into(),
                    r#type: placeholder_type as i32,
                    values,
                    element_level: false,
                }],
                ..Default::default()
            };
            let mut bytes = Vec::new();
            group.encode(&mut bytes)?;
            (
                milvus::search_request::SearchInput::PlaceholderGroup(bytes),
                nq,
            )
        };
        let mut nested_params = self
            .extra_params
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect::<serde_json::Map<_, _>>();
        let mut search_params = self
            .extra_params
            .into_iter()
            .map(|(key, value)| common::KeyValuePair {
                key,
                value,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        if !self.vector_field.is_empty() {
            set_search_param(&mut search_params, "anns_field", self.vector_field);
        }
        set_search_param(&mut search_params, "topk", self.limit.to_string());
        if self.offset != 0 {
            set_search_param(&mut search_params, "offset", self.offset.to_string());
            nested_params.insert("offset".into(), Value::String(self.offset.to_string()));
        }
        if self.round_decimal != -1 {
            set_search_param(
                &mut search_params,
                "round_decimal",
                self.round_decimal.to_string(),
            );
            nested_params.insert(
                "round_decimal".into(),
                Value::String(self.round_decimal.to_string()),
            );
        }
        if self.ignore_growing {
            set_search_param(&mut search_params, "ignore_growing", "true");
            nested_params.insert("ignore_growing".into(), Value::String("true".into()));
        }
        if !self.group_by_field.is_empty() {
            set_search_param(
                &mut search_params,
                "group_by_field",
                self.group_by_field.clone(),
            );
            nested_params.insert("group_by_field".into(), Value::String(self.group_by_field));
        }
        if self.group_size != 1 {
            set_search_param(
                &mut search_params,
                "group_size",
                self.group_size.to_string(),
            );
            nested_params.insert(
                "group_size".into(),
                Value::String(self.group_size.to_string()),
            );
        }
        if self.strict_group_size {
            set_search_param(&mut search_params, "strict_group_size", "true");
            nested_params.insert("strict_group_size".into(), Value::String("true".into()));
        }
        if let Some(radius) = self.radius {
            set_search_param(&mut search_params, "radius", radius.to_string());
            nested_params.insert("radius".into(), serde_json::json!(radius));
        }
        if let Some(range_filter) = self.range_filter {
            set_search_param(&mut search_params, "range_filter", range_filter.to_string());
            nested_params.insert("range_filter".into(), serde_json::json!(range_filter));
        }
        if !self.timezone.is_empty() {
            set_search_param(&mut search_params, "timezone", self.timezone.clone());
            nested_params.insert("timezone".into(), Value::String(self.timezone));
        }
        if let Some(metric) = self.metric_type {
            if metric != MetricType::Default {
                set_search_param(&mut search_params, "metric_type", metric.as_str());
            }
        }
        set_search_param(
            &mut search_params,
            "params",
            Value::Object(nested_params).to_string(),
        );
        Ok(milvus::SearchRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_names: self.partition_names,
            dsl: self.filter,
            dsl_type: common::DslType::BoolExprV1 as i32,
            output_fields: self.output_fields,
            search_params,
            travel_timestamp: 0,
            guarantee_timestamp,
            nq,
            not_return_all_meta: false,
            consistency_level: self
                .consistency_level
                .map(|level| level.into_proto() as i32)
                .unwrap_or_default(),
            use_default_consistency: self.consistency_level.is_none(),
            search_by_primary_keys: false,
            sub_reqs: Vec::new(),
            expr_template_values: self
                .filter_templates
                .into_iter()
                .map(|(key, value)| Ok((key, json_template(value)?)))
                .collect::<Result<_>>()?,
            function_score: self.rerank.map(FunctionScore::into_proto),
            namespace: None,
            highlighter: self.highlighter.map(Highlighter::into_proto),
            search_input: Some(search_input),
            search_aggregation: self
                .search_aggregation
                .map(|aggregation| aggregation.into_proto())
                .transpose()?,
            function_chains: self
                .function_chains
                .into_iter()
                .map(FunctionChain::into_proto)
                .collect(),
            ..Default::default()
        })
    }
}

impl SearchRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            ids: Ids::default(),
            vector_field: String::new(),
            vectors: SearchVectors::default(),
            partition_names: Vec::new(),
            filter: String::new(),
            filter_templates: HashMap::new(),
            output_fields: Vec::new(),
            limit: 10,
            offset: 0,
            round_decimal: -1,
            ignore_growing: false,
            group_by_field: String::new(),
            group_size: 1,
            strict_group_size: false,
            radius: None,
            range_filter: None,
            metric_type: None,
            extra_params: HashMap::new(),
            rerank: None,
            timezone: String::new(),
            highlighter: None,
            consistency_level: None,
            function_chains: Vec::new(),
            search_aggregation: None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for SearchRequest.
#[derive(Debug, Clone)]
pub struct SearchRequestBuilder {
    value: SearchRequest,
}

impl SearchRequestBuilder {
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

    /// Uses the vectors stored under these primary keys as the search targets.
    ///
    /// IDs and explicit target vectors cannot be specified at the same time.
    pub fn ids(mut self, value: Ids) -> Self {
        self.value.ids = value;
        self
    }

    /// Sets the vector field and returns the updated value.
    pub fn vector_field(mut self, value: impl Into<String>) -> Self {
        self.value.vector_field = value.into();
        self
    }

    /// Sets the vectors and returns the updated value.
    pub fn vectors(mut self, value: SearchVectors) -> Self {
        self.value.vectors = value;
        self
    }

    /// Sets the partition names and returns the updated value.
    pub fn partition_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.partition_names = values.into_iter().map(Into::into).collect();
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

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.output_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the limit and returns the updated value.
    pub fn limit(mut self, value: i64) -> Self {
        self.value.limit = value;
        self
    }

    /// Sets the offset and returns the updated value.
    pub fn offset(mut self, value: i64) -> Self {
        self.value.offset = value;
        self
    }

    /// Sets the round decimal and returns the updated value.
    pub fn round_decimal(mut self, value: i64) -> Self {
        self.value.round_decimal = value;
        self
    }

    /// Sets the ignore growing and returns the updated value.
    pub fn ignore_growing(mut self, value: bool) -> Self {
        self.value.ignore_growing = value;
        self
    }

    /// Sets the group by field and returns the updated value.
    pub fn group_by_field(mut self, value: impl Into<String>) -> Self {
        self.value.group_by_field = value.into();
        self
    }

    /// Sets the group size and returns the updated value.
    pub fn group_size(mut self, value: i64) -> Self {
        self.value.group_size = value;
        self
    }

    /// Sets the strict group size and returns the updated value.
    pub fn strict_group_size(mut self, value: bool) -> Self {
        self.value.strict_group_size = value;
        self
    }

    /// Sets the radius and returns the updated value.
    pub fn radius(mut self, value: f64) -> Self {
        self.value.radius = Some(value);
        self
    }

    /// Sets the range filter and returns the updated value.
    pub fn range_filter(mut self, value: f64) -> Self {
        self.value.range_filter = Some(value);
        self
    }

    /// Sets the metric type and returns the updated value.
    pub fn metric_type(mut self, value: MetricType) -> Self {
        self.value.metric_type = Some(value);
        self
    }

    /// Sets the extra params and returns the updated value.
    pub fn extra_params(mut self, value: HashMap<String, String>) -> Self {
        self.value.extra_params = value;
        self
    }

    /// Sets the rerank and returns the updated value.
    pub fn rerank(mut self, value: FunctionScore) -> Self {
        self.value.rerank = Some(value);
        self
    }

    /// Sets the timezone and returns the updated value.
    pub fn timezone(mut self, value: impl Into<String>) -> Self {
        self.value.timezone = value.into();
        self
    }

    /// Sets the highlighter and returns the updated value.
    pub fn highlighter(mut self, value: impl Into<Highlighter>) -> Self {
        self.value.highlighter = Some(value.into());
        self
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.value.consistency_level = Some(value);
        self
    }

    /// Sets the function chains and returns the updated value.
    ///
    /// Mutually exclusive with [`Self::rerank`]; Milvus rejects a search carrying both.
    pub fn function_chains(mut self, values: impl IntoIterator<Item = FunctionChain>) -> Self {
        self.value.function_chains = values.into_iter().collect();
        self
    }

    /// Adds a function chain and returns the updated value.
    pub fn add_function_chain(mut self, value: FunctionChain) -> Self {
        self.value.function_chains.push(value);
        self
    }

    /// Sets the hierarchical bucket aggregation and returns the updated value.
    ///
    /// Mutually exclusive with [`Self::group_by_field`] and [`Self::highlighter`]; when set,
    /// `limit` is ignored, `SearchAggregation.size` controls the top-level bucket count, and
    /// `offset` must remain zero.
    pub fn search_aggregation(mut self, value: SearchAggregation) -> Self {
        self.value.search_aggregation = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<SearchRequest> {
        validate_search_request(&self.value)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// SubSearchRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 sub_search operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SubSearchRequest {
    pub(crate) vector_field: String,
    pub(crate) vectors: SearchVectors,
    pub(crate) filter: String,
    pub(crate) filter_templates: HashMap<String, Value>,
    pub(crate) limit: i64,
    pub(crate) metric_type: Option<MetricType>,
    pub(crate) extra_params: HashMap<String, String>,
    pub(crate) radius: Option<f64>,
    pub(crate) range_filter: Option<f64>,
    pub(crate) timezone: String,
}

impl SubSearchRequest {
    /// Creates a builder for this request.
    pub fn builder() -> SubSearchRequestBuilder {
        SubSearchRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> SubSearchRequestBuilder {
        SubSearchRequestBuilder { value: self }
    }

    /// Returns the vector field.
    pub fn vector_field(&self) -> &str {
        &self.vector_field
    }

    /// Returns the vectors.
    pub fn vectors(&self) -> &SearchVectors {
        &self.vectors
    }

    /// Returns the filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns the filter templates.
    pub fn filter_templates(&self) -> &HashMap<String, Value> {
        &self.filter_templates
    }

    /// Returns the limit.
    pub fn limit(&self) -> i64 {
        self.limit
    }

    /// Returns the metric type.
    pub fn metric_type(&self) -> Option<MetricType> {
        self.metric_type
    }

    /// Returns the extra params.
    pub fn extra_params(&self) -> &HashMap<String, String> {
        &self.extra_params
    }

    /// Returns the radius.
    pub fn radius(&self) -> Option<f64> {
        self.radius
    }

    /// Returns the range filter.
    pub fn range_filter(&self) -> Option<f64> {
        self.range_filter
    }

    /// Returns the timezone.
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    fn into_proto(
        self,
        default_db: &str,
        guarantee_timestamp: u64,
    ) -> Result<milvus::SearchRequest> {
        SearchRequest {
            vector_field: self.vector_field,
            vectors: self.vectors,
            filter: self.filter,
            filter_templates: self.filter_templates,
            limit: self.limit,
            metric_type: self.metric_type,
            extra_params: self.extra_params,
            radius: self.radius,
            range_filter: self.range_filter,
            timezone: self.timezone,
            ..SearchRequest::empty()
        }
        .into_proto(default_db, guarantee_timestamp)
    }
}

impl SubSearchRequest {
    fn empty() -> Self {
        Self {
            vector_field: String::new(),
            vectors: SearchVectors::default(),
            filter: String::new(),
            filter_templates: HashMap::new(),
            limit: 10,
            metric_type: None,
            extra_params: HashMap::new(),
            radius: None,
            range_filter: None,
            timezone: String::new(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SubSearchRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for SubSearchRequest.
#[derive(Debug, Clone)]
pub struct SubSearchRequestBuilder {
    value: SubSearchRequest,
}

impl SubSearchRequestBuilder {
    /// Sets the vector field and returns the updated value.
    pub fn vector_field(mut self, value: impl Into<String>) -> Self {
        self.value.vector_field = value.into();
        self
    }

    /// Sets the vectors and returns the updated value.
    pub fn vectors(mut self, value: SearchVectors) -> Self {
        self.value.vectors = value;
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

    /// Sets the limit and returns the updated value.
    pub fn limit(mut self, value: i64) -> Self {
        self.value.limit = value;
        self
    }

    /// Sets the metric type and returns the updated value.
    pub fn metric_type(mut self, value: MetricType) -> Self {
        self.value.metric_type = Some(value);
        self
    }

    /// Sets the extra params and returns the updated value.
    pub fn extra_params(mut self, value: HashMap<String, String>) -> Self {
        self.value.extra_params = value;
        self
    }

    /// Sets the radius and returns the updated value.
    pub fn radius(mut self, value: f64) -> Self {
        self.value.radius = Some(value);
        self
    }

    /// Sets the range filter and returns the updated value.
    pub fn range_filter(mut self, value: f64) -> Self {
        self.value.range_filter = Some(value);
        self
    }

    /// Sets the timezone and returns the updated value.
    pub fn timezone(mut self, value: impl Into<String>) -> Self {
        self.value.timezone = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<SubSearchRequest> {
        validate_sub_search_request(&self.value)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// HybridSearchRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 hybrid_search operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct HybridSearchRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) partition_names: Vec<String>,
    pub(crate) sub_requests: Vec<SubSearchRequest>,
    pub(crate) rerank: Option<Function>,
    pub(crate) limit: i64,
    pub(crate) offset: i64,
    pub(crate) round_decimal: i64,
    pub(crate) ignore_growing: bool,
    pub(crate) extra_params: HashMap<String, String>,
    pub(crate) group_by_field: String,
    pub(crate) group_size: i64,
    pub(crate) strict_group_size: bool,
    pub(crate) output_fields: Vec<String>,
    pub(crate) consistency_level: Option<ConsistencyLevel>,
}

impl HybridSearchRequest {
    fn empty() -> Self {
        Self {
            database_name: None,
            collection_name: String::new(),
            partition_names: Vec::new(),
            sub_requests: Vec::new(),
            rerank: None,
            limit: 10,
            offset: 0,
            round_decimal: -1,
            ignore_growing: false,
            extra_params: HashMap::new(),
            group_by_field: String::new(),
            group_size: 1,
            strict_group_size: false,
            output_fields: Vec::new(),
            consistency_level: None,
        }
    }
}

impl HybridSearchRequest {
    /// Creates a builder for this request.
    pub fn builder() -> HybridSearchRequestBuilder {
        HybridSearchRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> HybridSearchRequestBuilder {
        HybridSearchRequestBuilder { value: self }
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

    /// Returns the sub requests.
    pub fn sub_requests(&self) -> &[SubSearchRequest] {
        &self.sub_requests
    }

    /// Returns the rerank.
    pub fn rerank(&self) -> &Option<Function> {
        &self.rerank
    }

    /// Returns the limit.
    pub fn limit(&self) -> i64 {
        self.limit
    }

    /// Returns the offset.
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// Returns the round decimal.
    pub fn round_decimal(&self) -> i64 {
        self.round_decimal
    }

    /// Returns whether the request should ignore growing.
    pub fn should_ignore_growing(&self) -> bool {
        self.ignore_growing
    }

    /// Returns the extra params.
    pub fn extra_params(&self) -> &HashMap<String, String> {
        &self.extra_params
    }

    /// Returns the group by field.
    pub fn group_by_field(&self) -> &str {
        &self.group_by_field
    }

    /// Returns the group size.
    pub fn group_size(&self) -> i64 {
        self.group_size
    }

    /// Returns whether strict group size.
    pub fn is_strict_group_size(&self) -> bool {
        self.strict_group_size
    }

    /// Returns the output fields.
    pub fn output_fields(&self) -> &[String] {
        &self.output_fields
    }

    /// Returns the consistency level.
    pub fn consistency_level(&self) -> Option<ConsistencyLevel> {
        self.consistency_level
    }

    pub(crate) fn into_proto(
        self,
        default_db: &str,
        guarantee_timestamp: u64,
    ) -> Result<milvus::HybridSearchRequest> {
        if self.sub_requests.is_empty() {
            return Err(Error::validation(
                "sub_requests".into(),
                "at least one sub-search request is required".into(),
            ));
        }
        if self.limit <= 0 {
            return Err(Error::validation(
                "limit".into(),
                "must be greater than zero".into(),
            ));
        }
        if self.group_size <= 0 {
            return Err(Error::validation(
                "group_size".into(),
                "must be greater than zero".into(),
            ));
        }
        let mut requests = Vec::with_capacity(self.sub_requests.len());
        for search in self.sub_requests {
            requests.push(search.into_proto(default_db, guarantee_timestamp)?);
        }
        let mut rank_params = self.extra_params;
        rank_params.insert("limit".into(), self.limit.to_string());
        if self.offset != 0 {
            rank_params.insert("offset".into(), self.offset.to_string());
        }
        if self.round_decimal != -1 {
            rank_params.insert("round_decimal".into(), self.round_decimal.to_string());
        }
        if self.ignore_growing {
            rank_params.insert("ignore_growing".into(), "true".into());
        }
        if !self.group_by_field.is_empty() {
            rank_params.insert("group_by_field".into(), self.group_by_field);
        }
        if self.group_size != 1 {
            rank_params.insert("group_size".into(), self.group_size.to_string());
        }
        if self.strict_group_size {
            rank_params.insert("strict_group_size".into(), "true".into());
        }
        if let Some(rerank) = self.rerank {
            rank_params.extend(rerank.params);
        }
        Ok(milvus::HybridSearchRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            partition_names: self.partition_names,
            requests,
            rank_params: rank_params
                .into_iter()
                .map(|(key, value)| common::KeyValuePair {
                    key,
                    value,
                    ..Default::default()
                })
                .collect(),
            travel_timestamp: 0,
            guarantee_timestamp,
            not_return_all_meta: false,
            output_fields: self.output_fields,
            consistency_level: self
                .consistency_level
                .map(|value| value.into_proto() as i32)
                .unwrap_or_default(),
            use_default_consistency: self.consistency_level.is_none(),
            function_score: None,
            namespace: None,
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// HybridSearchRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for HybridSearchRequest.
#[derive(Debug, Clone)]
pub struct HybridSearchRequestBuilder {
    value: HybridSearchRequest,
}

impl HybridSearchRequestBuilder {
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
        self.value.partition_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the sub requests and returns the updated value.
    pub fn sub_requests(mut self, value: Vec<SubSearchRequest>) -> Self {
        self.value.sub_requests = value;
        self
    }

    /// Sets the rerank and returns the updated value.
    pub fn rerank(mut self, value: impl Into<Function>) -> Self {
        self.value.rerank = Some(value.into());
        self
    }

    /// Sets the limit and returns the updated value.
    pub fn limit(mut self, value: i64) -> Self {
        self.value.limit = value;
        self
    }

    /// Sets the offset and returns the updated value.
    pub fn offset(mut self, value: i64) -> Self {
        self.value.offset = value;
        self
    }

    /// Sets the round decimal and returns the updated value.
    pub fn round_decimal(mut self, value: i64) -> Self {
        self.value.round_decimal = value;
        self
    }

    /// Sets the ignore growing and returns the updated value.
    pub fn ignore_growing(mut self, value: bool) -> Self {
        self.value.ignore_growing = value;
        self
    }

    /// Sets the extra params and returns the updated value.
    pub fn extra_params(mut self, value: HashMap<String, String>) -> Self {
        self.value.extra_params = value;
        self
    }

    /// Sets the group by field and returns the updated value.
    pub fn group_by_field(mut self, value: impl Into<String>) -> Self {
        self.value.group_by_field = value.into();
        self
    }

    /// Sets the group size and returns the updated value.
    pub fn group_size(mut self, value: i64) -> Self {
        self.value.group_size = value;
        self
    }

    /// Sets the strict group size and returns the updated value.
    pub fn strict_group_size(mut self, value: bool) -> Self {
        self.value.strict_group_size = value;
        self
    }

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.output_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the consistency level and returns the updated value.
    pub fn consistency_level(mut self, value: ConsistencyLevel) -> Self {
        self.value.consistency_level = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<HybridSearchRequest> {
        required("collection_name", &self.value.collection_name)?;
        non_empty_strings("partition_names", &self.value.partition_names)?;
        required_slice("sub_requests", &self.value.sub_requests)?;
        positive_i64("limit", self.value.limit)?;
        non_negative_i64("offset", self.value.offset)?;
        positive_i64("group_size", self.value.group_size)?;
        validate_search_extra_params(&self.value.extra_params)?;
        if !(-1..=6).contains(&self.value.round_decimal) {
            return Err(Error::validation(
                "round_decimal".into(),
                "must be within -1..=6".into(),
            ));
        }
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// QueryIteratorRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 query_iterator operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct QueryIteratorRequest {
    pub(crate) query: QueryRequest,
    pub(crate) batch_size: usize,
    pub(crate) reduce_stop_for_best: bool,
    pub(crate) cursor: Option<QueryCursor>,
}

impl QueryIteratorRequest {
    /// Creates a builder for this request.
    pub fn builder() -> QueryIteratorRequestBuilder {
        QueryIteratorRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> QueryIteratorRequestBuilder {
        QueryIteratorRequestBuilder { value: self }
    }

    /// Returns the query.
    pub fn query(&self) -> &QueryRequest {
        &self.query
    }

    /// Returns the limit.
    pub fn limit(&self) -> Option<i64> {
        self.query.limit
    }

    /// Returns the batch size.
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// Returns whether the request should reduce stop for best.
    pub fn should_reduce_stop_for_best(&self) -> bool {
        self.reduce_stop_for_best
    }

    /// Returns the resumable primary-key cursor the iterator starts from, if configured.
    pub fn cursor(&self) -> &Option<QueryCursor> {
        &self.cursor
    }
}

impl QueryIteratorRequest {
    fn empty() -> Self {
        Self {
            query: QueryRequest::empty(),
            batch_size: 1_000,
            reduce_stop_for_best: true,
            cursor: None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// QueryIteratorRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for QueryIteratorRequest.
#[derive(Debug, Clone)]
pub struct QueryIteratorRequestBuilder {
    value: QueryIteratorRequest,
}

impl QueryIteratorRequestBuilder {
    /// Sets the query and returns the updated value.
    pub fn query(mut self, value: QueryRequest) -> Self {
        self.value.query = value;
        self
    }

    /// Sets the limit and returns the updated value.
    pub fn limit(mut self, value: i64) -> Self {
        self.value.query.limit = Some(value);
        self
    }

    /// Sets the batch size and returns the updated value.
    pub fn batch_size(mut self, value: usize) -> Self {
        self.value.batch_size = value;
        self
    }

    /// Sets the reduce stop for best and returns the updated value.
    pub fn reduce_stop_for_best(mut self, value: bool) -> Self {
        self.value.reduce_stop_for_best = value;
        self
    }

    /// Resumes pagination from a previously captured primary-key cursor and returns the updated
    /// value.
    ///
    /// The cursor type must match the collection primary key (`Int64` for Int64 keys,
    /// `VarChar` for VarChar keys). A plain cursor resumes strictly after the cursor value
    /// (`pk > value`); an `element_filter` cursor resumes at the cursor value from the recorded
    /// element offset (`pk >= value`). A request `offset` is ignored on resume.
    ///
    /// The cursor must carry a real primary-key position: a freshly constructed
    /// [`QueryCursor::new`](crate::v2::types::QueryCursor::new) (which defaults to `Int64(0)`)
    /// would silently skip every row with `pk <= 0`, so resume only with a cursor captured from
    /// [`QueryIterator::cursor`](crate::v2::QueryIterator::cursor).
    pub fn cursor(mut self, value: QueryCursor) -> Self {
        self.value.cursor = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<QueryIteratorRequest> {
        validate_query_iterator_query(&self.value.query)?;
        positive_usize("batch_size", self.value.batch_size)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchIteratorRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 search_iterator operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SearchIteratorRequest {
    pub(crate) search: SearchRequest,
    pub(crate) batch_size: usize,
    pub(crate) limit: Option<usize>,
}

impl SearchIteratorRequest {
    /// Creates a builder for this request.
    pub fn builder() -> SearchIteratorRequestBuilder {
        SearchIteratorRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> SearchIteratorRequestBuilder {
        SearchIteratorRequestBuilder { value: self }
    }

    /// Returns the search.
    pub fn search(&self) -> &SearchRequest {
        &self.search
    }

    /// Returns the batch size.
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// Returns the limit.
    pub fn limit(&self) -> Option<usize> {
        self.limit
    }
}

impl SearchIteratorRequest {
    fn empty() -> Self {
        Self {
            search: SearchRequest::empty(),
            batch_size: 1_000,
            limit: None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchIteratorRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for SearchIteratorRequest.
#[derive(Debug, Clone)]
pub struct SearchIteratorRequestBuilder {
    value: SearchIteratorRequest,
}

impl SearchIteratorRequestBuilder {
    /// Sets the search and returns the updated value.
    pub fn search(mut self, value: SearchRequest) -> Self {
        self.value.search = value;
        self
    }

    /// Sets the batch size and returns the updated value.
    pub fn batch_size(mut self, value: usize) -> Self {
        self.value.batch_size = value;
        self
    }

    /// Sets the limit and returns the updated value.
    pub fn limit(mut self, value: usize) -> Self {
        self.value.limit = Some(value);
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> Result<SearchIteratorRequest> {
        validate_search_request(&self.value.search)?;
        if !self.value.search.ids.is_empty() {
            return Err(Error::validation(
                "ids".into(),
                "search iterator does not support IDs as search targets".into(),
            ));
        }
        positive_usize("batch_size", self.value.batch_size)?;
        Ok(self.value)
    }
}

fn validate_query_request(value: &QueryRequest) -> Result<()> {
    validate_query_request_limit(value, false)
}

fn validate_query_iterator_query(value: &QueryRequest) -> Result<()> {
    if !value.ids.is_empty() {
        return Err(Error::validation(
            "ids".into(),
            "query iterator does not support IDs".into(),
        ));
    }
    validate_query_request_limit(value, true)
}

fn validate_query_request_limit(value: &QueryRequest, allow_zero_limit: bool) -> Result<()> {
    required("collection_name", &value.collection_name)?;
    non_empty_strings("partition_names", &value.partition_names)?;
    if !value.ids.is_empty() && !value.filter.is_empty() {
        return Err(Error::validation(
            "ids".into(),
            "IDs and filter cannot be specified at the same time".into(),
        ));
    }
    if let Some(limit) = value.limit {
        if limit < -1 || (!allow_zero_limit && limit == 0) {
            let requirement = if allow_zero_limit {
                "must be -1 or non-negative"
            } else {
                "must be -1 or greater than zero"
            };
            return Err(Error::validation("limit".into(), requirement.into()));
        }
    }
    if let Some(offset) = value.offset {
        non_negative_i64("offset", offset)?;
    }
    Ok(())
}

fn validate_search_request(value: &SearchRequest) -> Result<()> {
    required("collection_name", &value.collection_name)?;
    non_empty_strings("partition_names", &value.partition_names)?;
    positive_i64("limit", value.limit)?;
    non_negative_i64("offset", value.offset)?;
    positive_i64("group_size", value.group_size)?;
    if !(-1..=6).contains(&value.round_decimal) {
        return Err(Error::validation(
            "round_decimal".into(),
            "must be within -1..=6".into(),
        ));
    }
    validate_search_extra_params(&value.extra_params)?;
    validate_finite_range_parameter("radius", value.radius)?;
    validate_finite_range_parameter("range_filter", value.range_filter)?;
    if !value.function_chains.is_empty() && value.rerank.is_some() {
        return Err(Error::validation(
            "function_chains".into(),
            "cannot be used together with rerank".into(),
        ));
    }
    for chain in &value.function_chains {
        chain.validate()?;
    }
    if let Some(aggregation) = &value.search_aggregation {
        aggregation.validate()?;
        if !value.group_by_field.is_empty() {
            return Err(Error::validation(
                "search_aggregation".into(),
                "cannot be used together with group_by_field".into(),
            ));
        }
        if value.offset > 0 {
            return Err(Error::validation(
                "offset".into(),
                "cannot be used together with search_aggregation".into(),
            ));
        }
        if value.highlighter.is_some() {
            return Err(Error::validation(
                "highlighter".into(),
                "cannot be used together with search_aggregation".into(),
            ));
        }
    }
    if !value.ids.is_empty() {
        if search_vectors_are_empty(&value.vectors) {
            return Ok(());
        }
        return Err(Error::validation(
            "ids".into(),
            "IDs and target vectors cannot be specified at the same time".into(),
        ));
    }
    validate_search_vectors(&value.vectors)
}

fn search_vectors_are_empty(vectors: &SearchVectors) -> bool {
    match vectors {
        SearchVectors::Float(values) => values.is_empty(),
        SearchVectors::Binary(values) => values.is_empty(),
        SearchVectors::Float16(values) => values.is_empty(),
        SearchVectors::BFloat16(values) => values.is_empty(),
        SearchVectors::SparseFloat(values) => values.is_empty(),
        SearchVectors::Int8(values) => values.is_empty(),
        SearchVectors::EmbeddedText(values) => values.is_empty(),
        SearchVectors::EmbeddingLists(values) => values.is_empty(),
    }
}

fn validate_sub_search_request(value: &SubSearchRequest) -> Result<()> {
    positive_i64("limit", value.limit)?;
    validate_search_extra_params(&value.extra_params)?;
    validate_finite_range_parameter("radius", value.radius)?;
    validate_finite_range_parameter("range_filter", value.range_filter)?;
    validate_search_vectors(&value.vectors)
}

fn validate_search_extra_params(extra_params: &HashMap<String, String>) -> Result<()> {
    const RESERVED: [&str; 5] = [
        "params",
        "topk",
        "anns_field",
        "metric_type",
        "round_decimal",
    ];
    if let Some(key) = RESERVED
        .into_iter()
        .find(|key| extra_params.contains_key(*key))
    {
        return Err(Error::validation(
            "extra_params".into(),
            format!("must not contain reserved key {key:?}"),
        ));
    }
    Ok(())
}

fn validate_search_vectors(vectors: &SearchVectors) -> Result<()> {
    match vectors {
        SearchVectors::Float(values) => validate_float_search_vectors("vectors", values),
        SearchVectors::Binary(values) => validate_dense_search_vectors("vectors", values),
        SearchVectors::Float16(values) => validate_dense_search_vectors("vectors", values),
        SearchVectors::BFloat16(values) => validate_dense_search_vectors("vectors", values),
        SearchVectors::Int8(values) => validate_dense_search_vectors("vectors", values),
        SearchVectors::SparseFloat(values) => {
            required_slice("vectors", values)?;
            for vector in values {
                if vector.is_empty() {
                    return Err(Error::validation(
                        "vectors".into(),
                        "sparse vectors must not be empty".into(),
                    ));
                }
                validate_sparse_vector("vectors", vector)?;
            }
            Ok(())
        }
        SearchVectors::EmbeddedText(values) => {
            required_slice("vectors", values)?;
            non_empty_strings("vectors", values)
        }
        SearchVectors::EmbeddingLists(lists) => {
            required_slice("embedding_lists", lists)?;
            for list in lists {
                validate_float_search_vectors("embedding_lists", &list.vectors)?;
            }
            Ok(())
        }
    }
}

fn validate_finite_range_parameter(name: &str, value: Option<f64>) -> Result<()> {
    if value.is_some_and(|value| !value.is_finite()) {
        return Err(Error::validation(name.into(), "must be finite".into()));
    }
    Ok(())
}

fn validate_dense_search_vectors<T>(name: &str, vectors: &[Vec<T>]) -> Result<()> {
    let Some(dimension) = vectors.first().map(Vec::len) else {
        return Err(Error::validation(
            name.into(),
            "at least one query vector is required".into(),
        ));
    };
    if dimension == 0 || vectors.iter().any(|vector| vector.len() != dimension) {
        return Err(Error::validation(
            name.into(),
            "all query vectors must have the same non-zero dimension".into(),
        ));
    }
    Ok(())
}

fn validate_float_search_vectors(name: &str, vectors: &[Vec<f32>]) -> Result<()> {
    validate_dense_search_vectors(name, vectors)?;
    if vectors.iter().flatten().any(|value| !value.is_finite()) {
        return Err(Error::validation(
            name.into(),
            "query vectors must contain only finite values".into(),
        ));
    }
    Ok(())
}

fn encode_u16_search_vectors(vectors: Vec<Vec<u16>>) -> Vec<Vec<u8>> {
    vectors
        .into_iter()
        .map(|vector| vector.into_iter().flat_map(u16::to_le_bytes).collect())
        .collect()
}

fn set_search_param(params: &mut Vec<common::KeyValuePair>, key: &str, value: impl Into<String>) {
    let value = value.into();
    if let Some(param) = params.iter_mut().find(|param| param.key == key) {
        param.value = value;
    } else {
        params.push(common::KeyValuePair {
            key: key.into(),
            value,
            ..Default::default()
        });
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod search_request_tests {
    use super::{
        EmbeddingList, HighlightQuery, Highlighter, HybridSearchRequest, LexicalHighlighter,
        SearchRequest, SearchVectors, SubSearchRequest,
    };
    use crate::proto::{common, milvus};
    use crate::v2::types::{
        col, fn_, BoostRerank, DecayRerank, FunctionChain, FunctionChainStage, FunctionScore,
        HighlightType, Ids, MetricOp, MetricSpec, MetricType, ModelRerank, OrderSpec,
        SearchAggregation, SortDirection, SparseVector, WeightedRerank,
    };
    use prost::Message;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn search_parameters_are_encoded_as_rpc_key_value_pairs() {
        let request = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .limit(3)
            .offset(2)
            .round_decimal(4)
            .ignore_growing(true)
            .group_by_field("category")
            .group_size(2)
            .strict_group_size(true)
            .radius(2.5)
            .range_filter(0.5)
            .metric_type(MetricType::L2)
            .extra_params(HashMap::from([("ef".into(), "64".into())]))
            .rerank(
                FunctionScore::new()
                    .add_function(
                        BoostRerank::new()
                            .name("boost")
                            .weight(2.0)
                            .random_score_field("id")
                            .random_score_seed(42),
                    )
                    .add_function(
                        DecayRerank::new()
                            .name("freshness")
                            .decay_function("gauss")
                            .origin(100)
                            .scale(20),
                    )
                    .add_function(
                        ModelRerank::new()
                            .name("cross_encoder")
                            .provider("tei")
                            .queries(["milvus"]),
                    )
                    .params(HashMap::from([("boost_mode".into(), json!("multiply"))])),
            )
            .timezone("Asia/Shanghai")
            .highlighter(
                LexicalHighlighter::new()
                    .highlight_queries(vec![HighlightQuery::new()
                        .query_type("phrase")
                        .field("text")
                        .text("milvus")])
                    .fragment_size(64),
            )
            .build()
            .expect("valid request")
            .into_proto("default", 0)
            .expect("convert search request");

        let value = |key: &str| {
            request
                .search_params
                .iter()
                .find(|param| param.key == key)
                .map(|param| param.value.as_str())
        };
        assert_eq!(value("anns_field"), Some("embedding"));
        assert_eq!(value("topk"), Some("3"));
        assert_eq!(value("offset"), Some("2"));
        assert_eq!(value("round_decimal"), Some("4"));
        assert_eq!(value("ignore_growing"), Some("true"));
        assert_eq!(value("group_by_field"), Some("category"));
        assert_eq!(value("group_size"), Some("2"));
        assert_eq!(value("strict_group_size"), Some("true"));
        assert_eq!(value("radius"), Some("2.5"));
        assert_eq!(value("range_filter"), Some("0.5"));
        assert_eq!(value("timezone"), Some("Asia/Shanghai"));
        assert_eq!(value("metric_type"), Some("L2"));
        assert_eq!(value("ef"), Some("64"));
        let params: serde_json::Value = serde_json::from_str(value("params").unwrap()).unwrap();
        assert_eq!(params["ef"], "64");
        assert_eq!(params["radius"], 2.5);
        assert_eq!(params["range_filter"], 0.5);
        let function_score = request.function_score.unwrap();
        assert_eq!(
            function_score
                .params
                .iter()
                .find(|param| param.key == "boost_mode")
                .map(|param| param.value.as_str()),
            Some("multiply")
        );
        let functions = function_score.functions;
        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0].name, "boost");
        assert_eq!(functions[1].name, "freshness");
        assert_eq!(functions[2].name, "cross_encoder");
        let random_score: serde_json::Value = serde_json::from_str(
            functions[0]
                .params
                .iter()
                .find(|param| param.key == "random_score")
                .map(|param| param.value.as_str())
                .expect("boost random_score parameter"),
        )
        .expect("valid random_score JSON");
        assert_eq!(random_score["seed"], 42);
        assert_eq!(
            functions[2]
                .params
                .iter()
                .find(|param| param.key == "reranker")
                .map(|param| param.value.as_str()),
            Some("model")
        );
        let highlighter = request.highlighter.unwrap();
        assert_eq!(highlighter.r#type, common::HighlightType::Lexical as i32);
        assert_eq!(
            highlighter
                .params
                .iter()
                .find(|param| param.key == "fragment_size")
                .map(|param| param.value.as_str()),
            Some("64")
        );
    }

    #[test]
    fn default_metric_is_omitted_from_search_params() {
        let request = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .limit(3)
            .metric_type(MetricType::Default)
            .build()
            .expect("valid request")
            .into_proto("default", 0)
            .expect("convert search request");

        assert!(request
            .search_params
            .iter()
            .all(|param| param.key != "metric_type"));
    }

    #[test]
    fn search_encodes_sparse_text_and_embedding_list_inputs() {
        let sparse: SparseVector = [(1, 0.25), (9, 0.75)].into_iter().collect();
        let cases = [
            (
                SearchVectors::SparseFloat(vec![sparse]),
                common::PlaceholderType::SparseFloatVector,
                1,
            ),
            (
                SearchVectors::EmbeddedText(vec!["milvus search".into()]),
                common::PlaceholderType::VarChar,
                1,
            ),
            (
                SearchVectors::EmbeddingLists(vec![
                    EmbeddingList::new().vectors(vec![vec![0.1, 0.2], vec![0.3, 0.4]])
                ]),
                common::PlaceholderType::EmbListFloatVector,
                1,
            ),
        ];

        for (vectors, expected_type, expected_nq) in cases {
            let request = SearchRequest::builder()
                .collection_name("books")
                .vectors(vectors)
                .build()
                .expect("valid request")
                .into_proto("default", 0)
                .unwrap();
            let milvus::search_request::SearchInput::PlaceholderGroup(bytes) =
                request.search_input.unwrap()
            else {
                panic!("expected placeholder input")
            };
            let group = common::PlaceholderGroup::decode(bytes.as_slice()).unwrap();
            assert_eq!(group.placeholders[0].r#type, expected_type as i32);
            assert_eq!(request.nq, expected_nq);
        }
    }

    #[test]
    fn search_encodes_integer_and_varchar_ids_as_search_inputs() {
        for (ids, expected) in [
            (Ids::Int64(vec![10, 20]), Ids::Int64(vec![10, 20])),
            (
                Ids::VarChar(vec!["ten".into(), "twenty".into()]),
                Ids::VarChar(vec!["ten".into(), "twenty".into()]),
            ),
        ] {
            let request = SearchRequest::builder()
                .collection_name("books")
                .ids(ids)
                .build()
                .expect("valid ID search")
                .into_proto("default", 0)
                .expect("encode ID search");

            let milvus::search_request::SearchInput::Ids(ids) =
                request.search_input.expect("search input")
            else {
                panic!("expected ID search input")
            };
            assert_eq!(Ids::from_proto(Some(ids)).unwrap(), expected);
            assert_eq!(request.nq, 2);
        }
    }

    #[test]
    fn search_rejects_ids_with_explicit_vectors() {
        let result = SearchRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(vec![1]))
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .build();

        assert!(matches!(
            result,
            Err(crate::v2::error::Error::Validation(error)) if error.parameter() == "ids"
        ));
    }

    #[test]
    fn search_rejects_out_of_range_round_decimal() {
        // PyMilvus accepts round_decimal in -2 < rd < 7 (i.e. -1..=6); reject both
        // bounds client-side instead of waiting for the server.
        for round_decimal in [-2, 7, 100] {
            let result = SearchRequest::builder()
                .collection_name("books")
                .vector_field("embedding")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .round_decimal(round_decimal)
                .build();
            assert!(
                matches!(
                    result,
                    Err(crate::v2::error::Error::Validation(error))
                        if error.parameter() == "round_decimal"
                ),
                "round_decimal {round_decimal} must be rejected"
            );
        }
        for round_decimal in [-1, 0, 6] {
            SearchRequest::builder()
                .collection_name("books")
                .vector_field("embedding")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .round_decimal(round_decimal)
                .build()
                .unwrap_or_else(|error| {
                    panic!("round_decimal {round_decimal} must be accepted: {error}")
                });
        }
    }

    #[test]
    fn hybrid_search_rejects_out_of_range_round_decimal() {
        for round_decimal in [-2, 7, 100] {
            let result = HybridSearchRequest::builder()
                .collection_name("books")
                .sub_requests(vec![SubSearchRequest::empty()])
                .round_decimal(round_decimal)
                .build();
            assert!(
                matches!(
                    result,
                    Err(crate::v2::error::Error::Validation(error))
                        if error.parameter() == "round_decimal"
                ),
                "round_decimal {round_decimal} must be rejected"
            );
        }
        for round_decimal in [-1, 0, 6] {
            HybridSearchRequest::builder()
                .collection_name("books")
                .sub_requests(vec![SubSearchRequest::empty()])
                .round_decimal(round_decimal)
                .build()
                .unwrap_or_else(|error| {
                    panic!("round_decimal {round_decimal} must be accepted: {error}")
                });
        }
    }

    #[test]
    fn search_rejects_function_chains_combined_with_rerank() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .map(
                "$score",
                fn_::num_combine(vec![col("$score"), col("popularity")], "sum", None),
            );
        let result = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .function_chains([chain])
            .rerank(FunctionScore::new().add_function(BoostRerank::new().name("boost").weight(2.0)))
            .build();

        assert!(matches!(
            result,
            Err(crate::v2::error::Error::Validation(error))
                if error.parameter() == "function_chains"
        ));
    }

    #[test]
    fn search_rejects_aggregation_combined_with_group_by_field() {
        let aggregation = SearchAggregation::new()
            .fields(["category"])
            .size(10)
            .add_metric(
                "total",
                MetricSpec::new().op(MetricOp::Sum).field_name("price"),
            )
            .add_order(
                OrderSpec::new()
                    .key("_count")
                    .direction(SortDirection::Desc),
            );
        let result = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .group_by_field("category")
            .search_aggregation(aggregation)
            .build();

        assert!(matches!(
            result,
            Err(crate::v2::error::Error::Validation(error))
                if error.parameter() == "search_aggregation"
        ));
    }

    #[test]
    fn search_rejects_aggregation_combined_with_nonzero_offset() {
        let aggregation = SearchAggregation::new()
            .fields(["category"])
            .size(10)
            .add_metric(
                "total",
                MetricSpec::new().op(MetricOp::Sum).field_name("price"),
            )
            .add_order(
                OrderSpec::new()
                    .key("_count")
                    .direction(SortDirection::Desc),
            );
        let result = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .offset(1)
            .search_aggregation(aggregation)
            .build();

        assert!(matches!(
            result,
            Err(crate::v2::error::Error::Validation(error))
                if error.parameter() == "offset"
        ));
    }

    #[test]
    fn search_rejects_aggregation_combined_with_highlighter() {
        let aggregation = SearchAggregation::new()
            .fields(["category"])
            .size(10)
            .add_metric(
                "total",
                MetricSpec::new().op(MetricOp::Sum).field_name("price"),
            )
            .add_order(
                OrderSpec::new()
                    .key("_count")
                    .direction(SortDirection::Desc),
            );
        let result = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .highlighter(Highlighter::new().highlight_type(HighlightType::Lexical))
            .search_aggregation(aggregation)
            .build();

        assert!(matches!(
            result,
            Err(crate::v2::error::Error::Validation(error))
                if error.parameter() == "highlighter"
        ));
    }

    #[test]
    fn search_encodes_function_chains_and_aggregation_on_the_happy_path() {
        let chain = FunctionChain::new()
            .stage(FunctionChainStage::L2Rerank)
            .sort("$score", true, None)
            .limit(10, 0);
        let aggregation = SearchAggregation::new()
            .fields(["category"])
            .size(10)
            .add_metric(
                "total",
                MetricSpec::new().op(MetricOp::Sum).field_name("price"),
            )
            .add_order(
                OrderSpec::new()
                    .key("_count")
                    .direction(SortDirection::Desc),
            );
        let request = SearchRequest::builder()
            .collection_name("books")
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .function_chains([chain])
            .search_aggregation(aggregation)
            .build()
            .expect("valid request with function chains and aggregation")
            .into_proto("default", 0)
            .expect("encode request");

        assert_eq!(request.function_chains.len(), 1);
        assert_eq!(request.function_chains[0].ops.len(), 2);
        assert_eq!(request.function_chains[0].ops[0].op, "sort");
        assert!(request.search_aggregation.is_some());
    }

    #[test]
    fn search_encodes_struct_vector_field_with_embedding_list() {
        let request = SearchRequest::builder()
            .collection_name("books")
            .vector_field("events[embedding]")
            .vectors(SearchVectors::EmbeddingLists(vec![
                EmbeddingList::new().vectors(vec![vec![0.1, 0.2], vec![0.3, 0.4]])
            ]))
            .metric_type(MetricType::MaxSimCosine)
            .limit(3)
            .build()
            .expect("valid request")
            .into_proto("default", 0)
            .unwrap();

        assert_eq!(
            request
                .search_params
                .iter()
                .find(|param| param.key == "anns_field")
                .map(|param| param.value.as_str()),
            Some("events[embedding]")
        );
        assert_eq!(
            request
                .search_params
                .iter()
                .find(|param| param.key == "metric_type")
                .map(|param| param.value.as_str()),
            Some("MAX_SIM_COSINE")
        );
        assert_eq!(request.nq, 1);
        let milvus::search_request::SearchInput::PlaceholderGroup(bytes) =
            request.search_input.unwrap()
        else {
            panic!("expected placeholder input")
        };
        let group = common::PlaceholderGroup::decode(bytes.as_slice()).unwrap();
        assert_eq!(
            group.placeholders[0].r#type,
            common::PlaceholderType::EmbListFloatVector as i32
        );
    }

    #[test]
    fn search_encodes_half_precision_u16_values_as_little_endian_bytes() {
        for (vectors, expected_type, expected_bytes) in [
            (
                SearchVectors::Float16(vec![vec![0x3c00, 0xbc00]]),
                common::PlaceholderType::Float16Vector,
                vec![0x00, 0x3c, 0x00, 0xbc],
            ),
            (
                SearchVectors::BFloat16(vec![vec![0x3f80, 0xbf80]]),
                common::PlaceholderType::BFloat16Vector,
                vec![0x80, 0x3f, 0x80, 0xbf],
            ),
        ] {
            let request = SearchRequest::builder()
                .collection_name("books")
                .vectors(vectors)
                .build()
                .expect("valid request")
                .into_proto("default", 0)
                .unwrap();
            let milvus::search_request::SearchInput::PlaceholderGroup(bytes) =
                request.search_input.unwrap()
            else {
                panic!("expected placeholder input")
            };
            let group = common::PlaceholderGroup::decode(bytes.as_slice()).unwrap();
            assert_eq!(group.placeholders[0].r#type, expected_type as i32);
            assert_eq!(group.placeholders[0].values[0], expected_bytes);
        }
    }

    #[test]
    fn search_builders_reject_non_finite_float_inputs() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::Float(vec![vec![value]]))
                .build()
                .is_err());
            assert!(SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::EmbeddingLists(vec![
                    EmbeddingList::new().vectors(vec![vec![value]])
                ]))
                .build()
                .is_err());
            assert!(SubSearchRequest::builder()
                .vectors(SearchVectors::Float(vec![vec![value]]))
                .build()
                .is_err());
        }

        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::Float(vec![vec![0.1]]))
                .radius(value)
                .build()
                .is_err());
            assert!(SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::Float(vec![vec![0.1]]))
                .range_filter(value)
                .build()
                .is_err());
            assert!(SubSearchRequest::builder()
                .vectors(SearchVectors::Float(vec![vec![0.1]]))
                .radius(value)
                .build()
                .is_err());
            assert!(SubSearchRequest::builder()
                .vectors(SearchVectors::Float(vec![vec![0.1]]))
                .range_filter(value)
                .build()
                .is_err());
        }
    }

    #[test]
    fn search_builders_reject_invalid_sparse_rows() {
        for sparse in [
            SparseVector::from([(1, -0.5)]),
            SparseVector::from([(u32::MAX, 0.5)]),
        ] {
            assert!(SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::SparseFloat(vec![sparse.clone()]))
                .build()
                .is_err());
            assert!(SubSearchRequest::builder()
                .vectors(SearchVectors::SparseFloat(vec![sparse]))
                .build()
                .is_err());
        }
    }

    #[test]
    fn search_builders_reject_reserved_extra_params() {
        for key in [
            "params",
            "topk",
            "anns_field",
            "metric_type",
            "round_decimal",
        ] {
            let extra_params = HashMap::from([(key.to_owned(), "value".to_owned())]);
            assert!(SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .extra_params(extra_params.clone())
                .build()
                .is_err());
            assert!(SubSearchRequest::builder()
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .extra_params(extra_params.clone())
                .build()
                .is_err());

            let sub_search = SubSearchRequest::builder()
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .build()
                .expect("valid sub-search request");
            assert!(HybridSearchRequest::builder()
                .collection_name("books")
                .sub_requests(vec![sub_search])
                .extra_params(extra_params)
                .build()
                .is_err());
        }
    }

    #[test]
    fn hybrid_search_encodes_typed_rank_controls() {
        let sub_request = SubSearchRequest::builder()
            .vector_field("embedding")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .metric_type(MetricType::Cosine)
            .timezone("UTC")
            .build()
            .expect("valid request");
        let request = HybridSearchRequest::builder()
            .collection_name("books")
            .partition_names(["hot"])
            .sub_requests(vec![sub_request])
            .rerank(WeightedRerank::new().weights(vec![1.0]))
            .limit(5)
            .offset(1)
            .round_decimal(3)
            .ignore_growing(true)
            .group_by_field("category")
            .group_size(2)
            .strict_group_size(true)
            .build()
            .expect("valid request")
            .into_proto("default", 42)
            .unwrap();
        let rank_params = request
            .rank_params
            .into_iter()
            .map(|pair| (pair.key, pair.value))
            .collect::<HashMap<_, _>>();
        assert_eq!(request.partition_names, vec!["hot"]);
        assert_eq!(rank_params["limit"], "5");
        assert_eq!(rank_params["offset"], "1");
        assert_eq!(rank_params["round_decimal"], "3");
        assert_eq!(rank_params["ignore_growing"], "true");
        assert_eq!(rank_params["group_by_field"], "category");
        assert_eq!(rank_params["group_size"], "2");
        assert_eq!(rank_params["strict_group_size"], "true");
        assert_eq!(rank_params["strategy"], "weighted");
        assert_eq!(request.guarantee_timestamp, 42);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod query_request_tests {
    use super::{
        GetRequest, HybridSearchRequest, Ids, QueryCursor, QueryIteratorRequest, QueryRequest,
        SearchRequest, SearchVectors, SubSearchRequest,
    };
    use crate::proto::schema::{template_array_value, template_value};
    use std::collections::HashMap;

    #[test]
    fn query_parameters_include_typed_timezone() {
        let request = QueryRequest::builder()
            .collection_name("events")
            .limit(10)
            .offset(2)
            .ignore_growing(true)
            .timezone("Asia/Shanghai")
            .extra_params(HashMap::from([("timezone".to_owned(), "UTC".to_owned())]))
            .build()
            .expect("valid request");
        assert_eq!(request.timezone().to_owned(), "Asia/Shanghai");

        let request = request.into_proto("default", None, 42).unwrap();
        let value = |key: &str| {
            request
                .query_params
                .iter()
                .find(|param| param.key == key)
                .map(|param| param.value.as_str())
        };
        assert_eq!(value("limit"), Some("10"));
        assert_eq!(value("offset"), Some("2"));
        assert_eq!(value("ignore_growing"), Some("true"));
        assert_eq!(value("timezone"), Some("Asia/Shanghai"));
        assert_eq!(request.guarantee_timestamp, 42);
    }

    #[test]
    fn query_ids_encode_as_primary_key_filter_templates() {
        let int_request = QueryRequest::builder()
            .database_name("analytics")
            .collection_name("books")
            .partition_names(["hot"])
            .ids(Ids::Int64(vec![1, 2]))
            .output_fields(["title"])
            .limit(10)
            .offset(2)
            .ignore_growing(true)
            .timezone("Asia/Shanghai")
            .build()
            .expect("valid Int64 ID query")
            .into_proto("default", Some("id"), 42)
            .expect("encode Int64 ID query");
        assert_eq!(int_request.db_name, "analytics");
        assert_eq!(int_request.partition_names, ["hot"]);
        assert_eq!(int_request.output_fields, ["title"]);
        assert_eq!(int_request.expr, "id in {__milvus_v2_query_ids}");
        assert_eq!(int_request.guarantee_timestamp, 42);
        let params = int_request
            .query_params
            .iter()
            .map(|pair| (pair.key.as_str(), pair.value.as_str()))
            .collect::<HashMap<_, _>>();
        assert_eq!(params["limit"], "10");
        assert_eq!(params["offset"], "2");
        assert_eq!(params["ignore_growing"], "true");
        assert_eq!(params["timezone"], "Asia/Shanghai");
        let Some(template_value::Val::ArrayVal(values)) = int_request
            .expr_template_values
            .get("__milvus_v2_query_ids")
            .and_then(|value| value.val.as_ref())
        else {
            panic!("expected Int64 ID array template");
        };
        assert!(matches!(
            &values.data,
            Some(template_array_value::Data::LongData(values)) if values.data == vec![1, 2]
        ));

        let string_request = QueryRequest::builder()
            .collection_name("articles")
            .ids(Ids::VarChar(vec!["a".into(), "b".into()]))
            .build()
            .expect("valid VarChar ID query")
            .into_proto("default", Some("key"), 0)
            .expect("encode VarChar ID query");
        assert_eq!(string_request.expr, "key in {__milvus_v2_query_ids}");
        let Some(template_value::Val::ArrayVal(values)) = string_request
            .expr_template_values
            .get("__milvus_v2_query_ids")
            .and_then(|value| value.val.as_ref())
        else {
            panic!("expected VarChar ID array template");
        };
        assert!(matches!(
            &values.data,
            Some(template_array_value::Data::StringData(values))
                if values.data == vec!["a".to_owned(), "b".to_owned()]
        ));
    }

    #[test]
    fn query_ids_are_mutually_exclusive_with_filter_and_not_supported_by_iterator() {
        assert!(QueryRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(vec![1]))
            .filter("id > 0")
            .build()
            .is_err());

        let query = QueryRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(vec![1]))
            .build()
            .expect("valid ID query");
        assert!(QueryIteratorRequest::builder()
            .query(query)
            .build()
            .is_err());
    }

    #[test]
    fn legacy_transport_fields_are_always_internal_defaults() {
        let query = QueryRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request")
            .into_proto("default", None, 1)
            .unwrap();
        let get = GetRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(vec![1]))
            .build()
            .expect("valid request")
            .into_proto("default", "id", 1)
            .unwrap();
        let search = SearchRequest::builder()
            .collection_name("books")
            .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
            .build()
            .expect("valid request")
            .into_proto("default", 1)
            .unwrap();
        let hybrid = HybridSearchRequest::builder()
            .collection_name("books")
            .sub_requests(vec![SubSearchRequest::builder()
                .vectors(SearchVectors::Float(vec![vec![0.1, 0.2]]))
                .build()
                .expect("valid request")])
            .build()
            .expect("valid request")
            .into_proto("default", 1)
            .unwrap();

        assert_eq!(query.travel_timestamp, 0);
        assert_eq!(get.travel_timestamp, 0);
        assert_eq!(search.travel_timestamp, 0);
        assert_eq!(hybrid.travel_timestamp, 0);
        assert_eq!(query.namespace, None);
        assert_eq!(get.namespace, None);
        assert_eq!(search.namespace, None);
        assert_eq!(hybrid.namespace, None);
    }

    #[test]
    fn query_iterator_request_carries_a_resumable_cursor() {
        let request = QueryIteratorRequest::builder()
            .query(
                QueryRequest::builder()
                    .collection_name("books")
                    .build()
                    .expect("valid query"),
            )
            .batch_size(100)
            .cursor(QueryCursor::int64(1_000, 42))
            .build()
            .expect("valid request");
        assert_eq!(request.cursor(), &Some(QueryCursor::int64(1_000, 42)));
        assert_eq!(request.batch_size(), 100);

        let without = QueryIteratorRequest::builder()
            .query(
                QueryRequest::builder()
                    .collection_name("books")
                    .build()
                    .expect("valid query"),
            )
            .build()
            .expect("valid request");
        assert_eq!(without.cursor(), &None);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn query_request_default_values() {
        let value = QueryRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_names: Vec<String> = Default::default();
        let expected_ids: Ids = Default::default();
        let expected_filter: String = String::new();
        let expected_filter_templates: HashMap<String, Value> = Default::default();
        let expected_output_fields: Vec<String> = Default::default();
        let expected_limit: Option<i64> = None;
        let expected_offset: Option<i64> = None;
        let expected_ignore_growing: bool = false;
        let expected_timezone: String = String::new();
        let expected_consistency_level: Option<ConsistencyLevel> = None;
        let expected_extra_params: HashMap<String, String> = Default::default();

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
        assert_eq!(value.ids().to_owned(), expected_ids);
        assert_eq!(value.filter().to_owned(), expected_filter);
        assert_eq!(
            value.filter_templates().to_owned(),
            expected_filter_templates
        );
        assert_eq!(value.output_fields().to_owned(), expected_output_fields);
        assert_eq!(value.limit().to_owned(), expected_limit);
        assert_eq!(value.offset().to_owned(), expected_offset);
        assert_eq!(
            value.should_ignore_growing().to_owned(),
            expected_ignore_growing
        );
        assert_eq!(value.timezone().to_owned(), expected_timezone);
        assert_eq!(
            value.consistency_level().to_owned(),
            expected_consistency_level
        );
        assert_eq!(value.extra_params().to_owned(), expected_extra_params);
    }

    #[test]
    fn query_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_names = vec!["partition_names-value".to_owned()];
        let filter = "filter-value".to_owned();
        let filter_templates =
            HashMap::from([("key-value".to_owned(), serde_json::json!({"key": "value"}))]);
        let output_fields = vec!["output_fields-value".to_owned()];
        let limit = 7;
        let offset = 7;
        let ignore_growing = true;
        let timezone = "timezone-value".to_owned();
        let consistency_level = ConsistencyLevel::Strong;
        let extra_params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = QueryRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_names(partition_names.clone())
            .filter(filter.clone())
            .filter_templates(filter_templates.clone())
            .output_fields(output_fields.clone())
            .limit(limit.clone())
            .offset(offset.clone())
            .ignore_growing(ignore_growing.clone())
            .timezone(timezone.clone())
            .consistency_level(consistency_level.clone())
            .extra_params(extra_params.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_names().to_owned(), partition_names);
        assert_eq!(value.filter().to_owned(), filter);
        assert_eq!(value.filter_templates().to_owned(), filter_templates);
        assert_eq!(value.output_fields().to_owned(), output_fields);
        assert_eq!(value.limit().to_owned(), Some(limit));
        assert_eq!(value.offset().to_owned(), Some(offset));
        assert_eq!(value.should_ignore_growing().to_owned(), ignore_growing);
        assert_eq!(value.timezone().to_owned(), timezone);
        assert_eq!(
            value.consistency_level().to_owned(),
            Some(consistency_level)
        );
        assert_eq!(value.extra_params().to_owned(), extra_params);
    }

    #[test]
    fn query_request_ids_value() {
        let ids = Ids::VarChar(vec!["first".to_owned(), "second".to_owned()]);
        let value = QueryRequest::builder()
            .collection_name("books")
            .ids(ids.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.ids(), &ids);
    }

    #[test]
    fn get_request_default_values() {
        let value = GetRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_names: Vec<String> = Default::default();
        let expected_ids: Ids = Default::default();
        let expected_output_fields: Vec<String> = Default::default();
        let expected_consistency_level: Option<ConsistencyLevel> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
        assert_eq!(value.ids().to_owned(), expected_ids);
        assert_eq!(value.output_fields().to_owned(), expected_output_fields);
        assert_eq!(
            value.consistency_level().to_owned(),
            expected_consistency_level
        );
    }

    #[test]
    fn get_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_names = vec!["partition_names-value".to_owned()];
        let ids = Ids::VarChar(vec!["id".to_owned()]);
        let output_fields = vec!["output_fields-value".to_owned()];
        let consistency_level = ConsistencyLevel::Strong;
        let value = GetRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_names(partition_names.clone())
            .ids(ids.clone())
            .output_fields(output_fields.clone())
            .consistency_level(consistency_level.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_names().to_owned(), partition_names);
        assert_eq!(value.ids().to_owned(), ids);
        assert_eq!(value.output_fields().to_owned(), output_fields);
        assert_eq!(
            value.consistency_level().to_owned(),
            Some(consistency_level)
        );
    }

    #[test]
    fn search_request_default_values() {
        let value = SearchRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_ids: Ids = Default::default();
        let expected_vector_field: String = String::new();
        let expected_vectors: SearchVectors = Default::default();
        let expected_partition_names: Vec<String> = Default::default();
        let expected_filter: String = String::new();
        let expected_filter_templates: HashMap<String, Value> = Default::default();
        let expected_output_fields: Vec<String> = Default::default();
        let expected_limit: i64 = 10;
        let expected_offset: i64 = 0;
        let expected_round_decimal: i64 = -1;
        let expected_ignore_growing: bool = false;
        let expected_group_by_field: String = String::new();
        let expected_group_size: i64 = 1;
        let expected_strict_group_size: bool = false;
        let expected_radius: Option<f64> = None;
        let expected_range_filter: Option<f64> = None;
        let expected_metric_type: Option<MetricType> = None;
        let expected_extra_params: HashMap<String, String> = Default::default();
        let expected_rerank: Option<FunctionScore> = None;
        let expected_timezone: String = String::new();
        let expected_highlighter: Option<Highlighter> = None;
        let expected_consistency_level: Option<ConsistencyLevel> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.ids().to_owned(), expected_ids);
        assert_eq!(value.vector_field().to_owned(), expected_vector_field);
        assert_eq!(value.vectors().to_owned(), expected_vectors);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
        assert_eq!(value.filter().to_owned(), expected_filter);
        assert_eq!(
            value.filter_templates().to_owned(),
            expected_filter_templates
        );
        assert_eq!(value.output_fields().to_owned(), expected_output_fields);
        assert_eq!(value.limit().to_owned(), expected_limit);
        assert_eq!(value.offset().to_owned(), expected_offset);
        assert_eq!(value.round_decimal().to_owned(), expected_round_decimal);
        assert_eq!(
            value.should_ignore_growing().to_owned(),
            expected_ignore_growing
        );
        assert_eq!(value.group_by_field().to_owned(), expected_group_by_field);
        assert_eq!(value.group_size().to_owned(), expected_group_size);
        assert_eq!(
            value.is_strict_group_size().to_owned(),
            expected_strict_group_size
        );
        assert_eq!(value.radius().to_owned(), expected_radius);
        assert_eq!(value.range_filter().to_owned(), expected_range_filter);
        assert_eq!(value.metric_type().to_owned(), expected_metric_type);
        assert_eq!(value.extra_params().to_owned(), expected_extra_params);
        assert_eq!(value.rerank().to_owned(), expected_rerank);
        assert_eq!(value.timezone().to_owned(), expected_timezone);
        assert_eq!(value.highlighter().to_owned(), expected_highlighter);
        assert_eq!(
            value.consistency_level().to_owned(),
            expected_consistency_level
        );
    }

    #[test]
    fn search_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let ids: Ids = Default::default();
        let vector_field = "vector_field-value".to_owned();
        let vectors = SearchVectors::Float(vec![vec![1.0, 2.0]]);
        let partition_names = vec!["partition_names-value".to_owned()];
        let filter = "filter-value".to_owned();
        let filter_templates =
            HashMap::from([("key-value".to_owned(), serde_json::json!({"key": "value"}))]);
        let output_fields = vec!["output_fields-value".to_owned()];
        let limit = 7;
        let offset = 7;
        let round_decimal = 6;
        let ignore_growing = true;
        let group_by_field = "group_by_field-value".to_owned();
        let group_size = 7;
        let strict_group_size = true;
        let radius = 1.5;
        let range_filter = 1.5;
        let metric_type = MetricType::Cosine;
        let extra_params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let rerank = FunctionScore::new().add_function(
            Function::new()
                .name("function")
                .function_type(crate::v2::FunctionType::Bm25),
        );
        let timezone = "timezone-value".to_owned();
        let highlighter = Highlighter::new().highlight_type(HighlightType::Lexical);
        let consistency_level = ConsistencyLevel::Strong;
        let value = SearchRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .ids(ids.clone())
            .vector_field(vector_field.clone())
            .vectors(vectors.clone())
            .partition_names(partition_names.clone())
            .filter(filter.clone())
            .filter_templates(filter_templates.clone())
            .output_fields(output_fields.clone())
            .limit(limit.clone())
            .offset(offset.clone())
            .round_decimal(round_decimal.clone())
            .ignore_growing(ignore_growing.clone())
            .group_by_field(group_by_field.clone())
            .group_size(group_size.clone())
            .strict_group_size(strict_group_size.clone())
            .radius(radius.clone())
            .range_filter(range_filter.clone())
            .metric_type(metric_type.clone())
            .extra_params(extra_params.clone())
            .rerank(rerank.clone())
            .timezone(timezone.clone())
            .highlighter(highlighter.clone())
            .consistency_level(consistency_level.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.ids().to_owned(), ids);
        assert_eq!(value.vector_field().to_owned(), vector_field);
        assert_eq!(value.vectors().to_owned(), vectors);
        assert_eq!(value.partition_names().to_owned(), partition_names);
        assert_eq!(value.filter().to_owned(), filter);
        assert_eq!(value.filter_templates().to_owned(), filter_templates);
        assert_eq!(value.output_fields().to_owned(), output_fields);
        assert_eq!(value.limit().to_owned(), limit);
        assert_eq!(value.offset().to_owned(), offset);
        assert_eq!(value.round_decimal().to_owned(), round_decimal);
        assert_eq!(value.should_ignore_growing().to_owned(), ignore_growing);
        assert_eq!(value.group_by_field().to_owned(), group_by_field);
        assert_eq!(value.group_size().to_owned(), group_size);
        assert_eq!(value.is_strict_group_size().to_owned(), strict_group_size);
        assert_eq!(value.radius().to_owned(), Some(radius));
        assert_eq!(value.range_filter().to_owned(), Some(range_filter));
        assert_eq!(value.metric_type().to_owned(), Some(metric_type));
        assert_eq!(value.extra_params().to_owned(), extra_params);
        assert_eq!(value.rerank().to_owned(), Some(rerank));
        assert_eq!(value.timezone().to_owned(), timezone);
        assert_eq!(value.highlighter().to_owned(), Some(highlighter));
        assert_eq!(
            value.consistency_level().to_owned(),
            Some(consistency_level)
        );
    }

    #[test]
    fn sub_search_request_default_values() {
        let value = SubSearchRequest::empty();
        let expected_vector_field: String = String::new();
        let expected_vectors: SearchVectors = Default::default();
        let expected_filter: String = String::new();
        let expected_filter_templates: HashMap<String, Value> = Default::default();
        let expected_limit: i64 = 10;
        let expected_metric_type: Option<MetricType> = None;
        let expected_extra_params: HashMap<String, String> = Default::default();
        let expected_radius: Option<f64> = None;
        let expected_range_filter: Option<f64> = None;
        let expected_timezone: String = String::new();

        assert_eq!(value.vector_field().to_owned(), expected_vector_field);
        assert_eq!(value.vectors().to_owned(), expected_vectors);
        assert_eq!(value.filter().to_owned(), expected_filter);
        assert_eq!(
            value.filter_templates().to_owned(),
            expected_filter_templates
        );
        assert_eq!(value.limit().to_owned(), expected_limit);
        assert_eq!(value.metric_type().to_owned(), expected_metric_type);
        assert_eq!(value.extra_params().to_owned(), expected_extra_params);
        assert_eq!(value.radius().to_owned(), expected_radius);
        assert_eq!(value.range_filter().to_owned(), expected_range_filter);
        assert_eq!(value.timezone().to_owned(), expected_timezone);
    }

    #[test]
    fn sub_search_request_populated_values() {
        let vector_field = "vector_field-value".to_owned();
        let vectors = SearchVectors::Float(vec![vec![1.0, 2.0]]);
        let filter = "filter-value".to_owned();
        let filter_templates =
            HashMap::from([("key-value".to_owned(), serde_json::json!({"key": "value"}))]);
        let limit = 7;
        let metric_type = MetricType::Cosine;
        let extra_params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let radius = 1.5;
        let range_filter = 1.5;
        let timezone = "timezone-value".to_owned();
        let value = SubSearchRequest::builder()
            .vector_field(vector_field.clone())
            .vectors(vectors.clone())
            .filter(filter.clone())
            .filter_templates(filter_templates.clone())
            .limit(limit.clone())
            .metric_type(metric_type.clone())
            .extra_params(extra_params.clone())
            .radius(radius.clone())
            .range_filter(range_filter.clone())
            .timezone(timezone.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.vector_field().to_owned(), vector_field);
        assert_eq!(value.vectors().to_owned(), vectors);
        assert_eq!(value.filter().to_owned(), filter);
        assert_eq!(value.filter_templates().to_owned(), filter_templates);
        assert_eq!(value.limit().to_owned(), limit);
        assert_eq!(value.metric_type().to_owned(), Some(metric_type));
        assert_eq!(value.extra_params().to_owned(), extra_params);
        assert_eq!(value.radius().to_owned(), Some(radius));
        assert_eq!(value.range_filter().to_owned(), Some(range_filter));
        assert_eq!(value.timezone().to_owned(), timezone);
    }

    #[test]
    fn hybrid_search_request_default_values() {
        let value = HybridSearchRequest::empty();
        let expected_database_name: Option<String> = None;
        let expected_collection_name: String = String::new();
        let expected_partition_names: Vec<String> = Default::default();
        let expected_sub_requests: Vec<SubSearchRequest> = Default::default();
        let expected_rerank: Option<Function> = None;
        let expected_limit: i64 = 10;
        let expected_offset: i64 = 0;
        let expected_round_decimal: i64 = -1;
        let expected_ignore_growing: bool = false;
        let expected_extra_params: HashMap<String, String> = Default::default();
        let expected_group_by_field: String = String::new();
        let expected_group_size: i64 = 1;
        let expected_strict_group_size: bool = false;
        let expected_output_fields: Vec<String> = Default::default();
        let expected_consistency_level: Option<ConsistencyLevel> = None;

        assert_eq!(value.database_name().to_owned(), expected_database_name);
        assert_eq!(value.collection_name().to_owned(), expected_collection_name);
        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
        assert_eq!(value.sub_requests().to_owned(), expected_sub_requests);
        assert_eq!(value.rerank().to_owned(), expected_rerank);
        assert_eq!(value.limit().to_owned(), expected_limit);
        assert_eq!(value.offset().to_owned(), expected_offset);
        assert_eq!(value.round_decimal().to_owned(), expected_round_decimal);
        assert_eq!(
            value.should_ignore_growing().to_owned(),
            expected_ignore_growing
        );
        assert_eq!(value.extra_params().to_owned(), expected_extra_params);
        assert_eq!(value.group_by_field().to_owned(), expected_group_by_field);
        assert_eq!(value.group_size().to_owned(), expected_group_size);
        assert_eq!(
            value.is_strict_group_size().to_owned(),
            expected_strict_group_size
        );
        assert_eq!(value.output_fields().to_owned(), expected_output_fields);
        assert_eq!(
            value.consistency_level().to_owned(),
            expected_consistency_level
        );
    }

    #[test]
    fn hybrid_search_request_populated_values() {
        let database_name = "database_name-value".to_owned();
        let collection_name = "collection_name-value".to_owned();
        let partition_names = vec!["partition_names-value".to_owned()];
        let sub_requests = vec![SubSearchRequest::empty()];
        let rerank = Function::new()
            .name("function")
            .function_type(crate::v2::FunctionType::Bm25);
        let limit = 7;
        let offset = 7;
        let round_decimal = 6;
        let ignore_growing = true;
        let extra_params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let group_by_field = "group_by_field-value".to_owned();
        let group_size = 7;
        let strict_group_size = true;
        let output_fields = vec!["output_fields-value".to_owned()];
        let consistency_level = ConsistencyLevel::Strong;
        let value = HybridSearchRequest::builder()
            .database_name(database_name.clone())
            .collection_name(collection_name.clone())
            .partition_names(partition_names.clone())
            .sub_requests(sub_requests.clone())
            .rerank(rerank.clone())
            .limit(limit.clone())
            .offset(offset.clone())
            .round_decimal(round_decimal.clone())
            .ignore_growing(ignore_growing.clone())
            .extra_params(extra_params.clone())
            .group_by_field(group_by_field.clone())
            .group_size(group_size.clone())
            .strict_group_size(strict_group_size.clone())
            .output_fields(output_fields.clone())
            .consistency_level(consistency_level.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.database_name().to_owned(), Some(database_name));
        assert_eq!(value.collection_name().to_owned(), collection_name);
        assert_eq!(value.partition_names().to_owned(), partition_names);
        assert_eq!(value.sub_requests().to_owned(), sub_requests);
        assert_eq!(value.rerank().to_owned(), Some(rerank));
        assert_eq!(value.limit().to_owned(), limit);
        assert_eq!(value.offset().to_owned(), offset);
        assert_eq!(value.round_decimal().to_owned(), round_decimal);
        assert_eq!(value.should_ignore_growing().to_owned(), ignore_growing);
        assert_eq!(value.extra_params().to_owned(), extra_params);
        assert_eq!(value.group_by_field().to_owned(), group_by_field);
        assert_eq!(value.group_size().to_owned(), group_size);
        assert_eq!(value.is_strict_group_size().to_owned(), strict_group_size);
        assert_eq!(value.output_fields().to_owned(), output_fields);
        assert_eq!(
            value.consistency_level().to_owned(),
            Some(consistency_level)
        );
    }

    #[test]
    fn query_iterator_request_default_values() {
        let value = QueryIteratorRequest::empty();
        let expected_query = QueryRequest::empty();
        let expected_batch_size: usize = 1_000;
        let expected_reduce_stop_for_best = true;

        assert_eq!(value.query().to_owned(), expected_query);
        assert_eq!(value.batch_size().to_owned(), expected_batch_size);
        assert_eq!(
            value.should_reduce_stop_for_best(),
            expected_reduce_stop_for_best
        );
    }

    #[test]
    fn query_iterator_request_populated_values() {
        let query = QueryRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid query request");
        let batch_size = 7;
        let reduce_stop_for_best = false;
        let value = QueryIteratorRequest::builder()
            .query(query.clone())
            .batch_size(batch_size.clone())
            .reduce_stop_for_best(reduce_stop_for_best)
            .build()
            .expect("valid request");

        assert_eq!(value.query().to_owned(), query);
        assert_eq!(value.batch_size().to_owned(), batch_size);
        assert_eq!(value.should_reduce_stop_for_best(), reduce_stop_for_best);
    }

    #[test]
    fn query_iterator_request_accepts_zero_limit() {
        let query = QueryRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid query request");
        let value = QueryIteratorRequest::builder()
            .query(query)
            .limit(0)
            .build()
            .expect("zero limit is valid for a query iterator");

        assert_eq!(value.limit(), Some(0));
    }

    #[test]
    fn search_iterator_request_default_values() {
        let value = SearchIteratorRequest::empty();
        let expected_search = SearchRequest::empty();
        let expected_batch_size: usize = 1_000;
        let expected_limit: Option<usize> = None;

        assert_eq!(value.search().to_owned(), expected_search);
        assert_eq!(value.batch_size().to_owned(), expected_batch_size);
        assert_eq!(value.limit().to_owned(), expected_limit);
    }

    #[test]
    fn search_iterator_request_populated_values() {
        let search = SearchRequest::builder()
            .collection_name("books")
            .vectors(SearchVectors::Float(vec![vec![0.0]]))
            .build()
            .expect("valid search request");
        let batch_size = 7;
        let limit = 7;
        let value = SearchIteratorRequest::builder()
            .search(search.clone())
            .batch_size(batch_size.clone())
            .limit(limit.clone())
            .build()
            .expect("valid request");

        assert_eq!(value.search().to_owned(), search);
        assert_eq!(value.batch_size().to_owned(), batch_size);
        assert_eq!(value.limit().to_owned(), Some(limit));
    }

    #[test]
    fn search_iterator_request_accepts_zero_limit() {
        let search = SearchRequest::builder()
            .collection_name("books")
            .vectors(SearchVectors::Float(vec![vec![0.0]]))
            .build()
            .expect("valid search request");
        let value = SearchIteratorRequest::builder()
            .search(search)
            .limit(0)
            .build()
            .expect("zero limit is valid for a search iterator");

        assert_eq!(value.limit(), Some(0));
    }

    #[test]
    fn search_iterator_request_rejects_id_search() {
        let search = SearchRequest::builder()
            .collection_name("books")
            .ids(Ids::Int64(vec![1]))
            .build()
            .expect("valid ID search request");

        assert!(SearchIteratorRequest::builder()
            .search(search)
            .build()
            .is_err());
    }

    #[test]
    fn search_iterator_request_applies_search_round_decimal_bound() {
        // The iterator delegates to validate_search_request for the wrapped search,
        // so only a search whose round_decimal is within -1..=6 can be wrapped.
        for round_decimal in [-2, 7, 100] {
            let search = SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::Float(vec![vec![0.0]]))
                .round_decimal(round_decimal)
                .build();
            assert!(
                search.is_err(),
                "round_decimal {round_decimal} must be rejected at the search level"
            );
        }
        for round_decimal in [-1, 0, 6] {
            let search = SearchRequest::builder()
                .collection_name("books")
                .vectors(SearchVectors::Float(vec![vec![0.0]]))
                .round_decimal(round_decimal)
                .build()
                .unwrap_or_else(|error| {
                    panic!("round_decimal {round_decimal} must be accepted: {error}")
                });
            SearchIteratorRequest::builder()
                .search(search)
                .build()
                .unwrap_or_else(|error| {
                    panic!("round_decimal {round_decimal} must be accepted: {error}")
                });
        }
    }
}
