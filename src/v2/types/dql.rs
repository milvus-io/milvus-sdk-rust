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

//! Query, search, reranking, highlighting, and result types.

use super::common::{EntityRow, FieldData, Function, FunctionType, Ids, SparseVector, StructValue};
use crate::proto::{common, schema};
use crate::v2::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;

fn rerank_function(name: impl Into<String>, reranker: &str) -> Function {
    Function::new()
        .name(name)
        .function_type(FunctionType::Rerank)
        .param("reranker", reranker)
}

///////////////////////////////////////////////////////////////////////////////
// RRFRerank
///////////////////////////////////////////////////////////////////////////////
/// Reciprocal-rank-fusion reranking configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RRFRerank {
    pub(crate) function: Function,
}

impl RRFRerank {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            function: Function::new()
                .function_type(FunctionType::Rerank)
                .param("strategy", "rrf")
                .param("params", serde_json::json!({ "k": 60 }).to_string()),
        }
    }

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: Function) -> Self {
        self.function = value;
        self
    }

    /// Sets the function and returns this value for further mutation.
    pub fn set_function(&mut self, value: Function) -> &mut Self {
        self.function = value;
        self
    }

    /// Returns the configured function.
    pub fn get_function(&self) -> &Function {
        &self.function
    }

    /// Sets the k and returns the updated value.
    pub fn k(mut self, value: i64) -> Self {
        self.function.params.insert(
            "params".into(),
            serde_json::json!({ "k": value }).to_string(),
        );
        self
    }
}

impl std::ops::Deref for RRFRerank {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

///////////////////////////////////////////////////////////////////////////////
// WeightedRerank
///////////////////////////////////////////////////////////////////////////////
/// Weighted-score reranking configuration.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct WeightedRerank {
    pub(crate) function: Function,
}

impl WeightedRerank {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            function: Function::new()
                .function_type(FunctionType::Rerank)
                .param("strategy", "weighted")
                .param("params", serde_json::json!({ "weights": [] }).to_string()),
        }
    }

    /// Sets the weights and returns the updated value.
    pub fn weights(mut self, value: Vec<f32>) -> Self {
        self.function.params.insert(
            "params".into(),
            serde_json::json!({ "weights": value }).to_string(),
        );
        self
    }

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: Function) -> Self {
        self.function = value;
        self
    }

    /// Sets the function and returns this value for further mutation.
    pub fn set_function(&mut self, value: Function) -> &mut Self {
        self.function = value;
        self
    }

    /// Returns the configured function.
    pub fn get_function(&self) -> &Function {
        &self.function
    }
}

impl std::ops::Deref for WeightedRerank {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

///////////////////////////////////////////////////////////////////////////////
// BoostRerank
///////////////////////////////////////////////////////////////////////////////
/// Reranker that boosts matches satisfying a filter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BoostRerank {
    pub(crate) function: Function,
}

impl BoostRerank {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            function: rerank_function("", "boost"),
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.function.name = value.into();
        self
    }

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: Function) -> Self {
        self.function = value;
        self
    }

    /// Sets the function and returns this value for further mutation.
    pub fn set_function(&mut self, value: Function) -> &mut Self {
        self.function = value;
        self
    }

    /// Returns the configured function.
    pub fn get_function(&self) -> &Function {
        &self.function
    }

    /// Sets the filter and returns the updated value.
    pub fn filter(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !value.is_empty() {
            self.function.params.insert("filter".into(), value);
        }
        self
    }

    /// Sets the weight and returns the updated value.
    pub fn weight(mut self, value: f32) -> Self {
        if value > 0.0 {
            self.function
                .params
                .insert("weight".into(), value.to_string());
        }
        self
    }

    fn random_score(&self) -> serde_json::Map<String, serde_json::Value> {
        self.function
            .params
            .get("random_score")
            .and_then(|value| serde_json::from_str(value).ok())
            .unwrap_or_default()
    }

    fn set_random_score(&mut self, value: serde_json::Map<String, serde_json::Value>) {
        self.function.params.insert(
            "random_score".into(),
            serde_json::Value::Object(value).to_string(),
        );
    }

    /// Sets the random score field and returns the updated value.
    pub fn random_score_field(mut self, value: impl Into<String>) -> Self {
        let mut random_score = self.random_score();
        random_score.insert("field".into(), serde_json::Value::String(value.into()));
        self.set_random_score(random_score);
        self
    }

    /// Sets the random score seed and returns the updated value.
    pub fn random_score_seed(mut self, value: i64) -> Self {
        let mut random_score = self.random_score();
        random_score.insert("seed".into(), serde_json::Value::Number(value.into()));
        self.set_random_score(random_score);
        self
    }
}

impl std::ops::Deref for BoostRerank {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

///////////////////////////////////////////////////////////////////////////////
// DecayRerank
///////////////////////////////////////////////////////////////////////////////
/// Reranker that applies distance-based score decay.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DecayRerank {
    pub(crate) function: Function,
}

impl DecayRerank {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            function: rerank_function("", "decay"),
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.function.name = value.into();
        self
    }

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: Function) -> Self {
        self.function = value;
        self
    }

    /// Sets the function and returns this value for further mutation.
    pub fn set_function(&mut self, value: Function) -> &mut Self {
        self.function = value;
        self
    }

    /// Returns the configured function.
    pub fn get_function(&self) -> &Function {
        &self.function
    }

    /// Sets the decay function and returns the updated value.
    pub fn decay_function(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !value.is_empty() {
            self.function.params.insert("function".into(), value);
        }
        self
    }

    /// Sets the origin and returns the updated value.
    pub fn origin(mut self, value: impl ToString) -> Self {
        self.function
            .params
            .insert("origin".into(), value.to_string());
        self
    }

    /// Sets the offset and returns the updated value.
    pub fn offset(mut self, value: impl ToString) -> Self {
        self.function
            .params
            .insert("offset".into(), value.to_string());
        self
    }

    /// Sets the scale and returns the updated value.
    pub fn scale(mut self, value: impl ToString) -> Self {
        self.function
            .params
            .insert("scale".into(), value.to_string());
        self
    }

    /// Sets the decay and returns the updated value.
    pub fn decay(mut self, value: f32) -> Self {
        self.function
            .params
            .insert("decay".into(), value.to_string());
        self
    }
}

impl std::ops::Deref for DecayRerank {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

///////////////////////////////////////////////////////////////////////////////
// ModelRerank
///////////////////////////////////////////////////////////////////////////////
/// Reranker backed by a configured model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ModelRerank {
    pub(crate) function: Function,
}

impl ModelRerank {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            function: rerank_function("", "model"),
        }
    }

    /// Sets the name and returns the updated value.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.function.name = value.into();
        self
    }

    /// Sets the function and returns the updated value.
    pub fn function(mut self, value: Function) -> Self {
        self.function = value;
        self
    }

    /// Sets the function and returns this value for further mutation.
    pub fn set_function(&mut self, value: Function) -> &mut Self {
        self.function = value;
        self
    }

    /// Returns the configured function.
    pub fn get_function(&self) -> &Function {
        &self.function
    }

    /// Sets the provider and returns the updated value.
    pub fn provider(mut self, value: impl Into<String>) -> Self {
        self.function.params.insert("provider".into(), value.into());
        self
    }

    /// Sets the queries and returns the updated value.
    pub fn queries(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let values = values.into_iter().map(Into::into).collect::<Vec<String>>();
        self.function
            .params
            .insert("queries".into(), serde_json::json!(values).to_string());
        self
    }

    /// Sets the endpoint and returns the updated value.
    pub fn endpoint(mut self, value: impl Into<String>) -> Self {
        self.function.params.insert("endpoint".into(), value.into());
        self
    }

    /// Sets the max client batch size and returns the updated value.
    pub fn max_client_batch_size(mut self, value: i64) -> Self {
        self.function
            .params
            .insert("max_client_batch_size".into(), value.to_string());
        self
    }
}

impl std::ops::Deref for ModelRerank {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

///////////////////////////////////////////////////////////////////////////////
// FunctionScore
///////////////////////////////////////////////////////////////////////////////
/// Reranking functions and combination settings used by search.
///
/// This type is used both as a V2 request input and as a response domain object.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FunctionScore {
    functions: Vec<Function>,
    params: HashMap<String, serde_json::Value>,
}

impl FunctionScore {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Sets the functions and returns the updated value.
    pub fn functions(mut self, value: Vec<Function>) -> Self {
        self.functions = value;
        self
    }

    /// Sets the functions and returns this value for further mutation.
    pub fn set_functions(&mut self, value: Vec<Function>) -> &mut Self {
        self.functions = value;
        self
    }

    /// Returns the configured functions.
    pub fn get_functions(&self) -> &[Function] {
        &self.functions
    }

    /// Sets the params and returns the updated value.
    pub fn params(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.params = value;
        self
    }

    /// Sets the params and returns this value for further mutation.
    pub fn set_params(&mut self, value: HashMap<String, serde_json::Value>) -> &mut Self {
        self.params = value;
        self
    }

    /// Returns the configured params.
    pub fn get_params(&self) -> &HashMap<String, serde_json::Value> {
        &self.params
    }

    /// Adds one add function to the existing values.
    pub fn add_function(mut self, value: impl Into<Function>) -> Self {
        self.functions.push(value.into());
        self
    }

    pub(crate) fn into_proto(self) -> schema::FunctionScore {
        schema::FunctionScore {
            functions: self
                .functions
                .into_iter()
                .map(Function::into_proto)
                .collect(),
            params: self
                .params
                .into_iter()
                .map(|(key, value)| common::KeyValuePair {
                    key,
                    value: match value {
                        serde_json::Value::String(value) => value,
                        value => value.to_string(),
                    },
                })
                .collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchVectors
///////////////////////////////////////////////////////////////////////////////
/// Vector or text inputs accepted by a search request.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SearchVectors {
    /// Represents the Float case.
    Float(Vec<Vec<f32>>),
    /// Represents the Binary case.
    Binary(Vec<Vec<u8>>),
    /// Represents the Float16 case.
    Float16(Vec<Vec<u16>>),
    /// Represents the BFloat16 case.
    BFloat16(Vec<Vec<u16>>),
    /// Represents the SparseFloat case.
    SparseFloat(Vec<SparseVector>),
    /// Represents the Int8 case.
    Int8(Vec<Vec<i8>>),
    /// Represents the EmbeddedText case.
    EmbeddedText(Vec<String>),
    /// Represents the EmbeddingLists case.
    EmbeddingLists(Vec<EmbeddingList>),
}

impl Default for SearchVectors {
    fn default() -> Self {
        Self::Float(Vec::new())
    }
}

///////////////////////////////////////////////////////////////////////////////
// EmbeddingList
///////////////////////////////////////////////////////////////////////////////
/// A list of embeddings used by struct-vector search.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct EmbeddingList {
    pub(crate) vectors: Vec<Vec<f32>>,
}

impl EmbeddingList {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
        }
    }

    /// Sets the vectors and returns the updated value.
    pub fn vectors(mut self, value: Vec<Vec<f32>>) -> Self {
        self.vectors = value;
        self
    }

    /// Sets the vectors and returns this value for further mutation.
    pub fn set_vectors(&mut self, value: Vec<Vec<f32>>) -> &mut Self {
        self.vectors = value;
        self
    }

    /// Returns the configured vectors.
    pub fn get_vectors(&self) -> &[Vec<f32>] {
        &self.vectors
    }

    /// Adds one add vector to the existing values.
    pub fn add_vector(mut self, value: Vec<f32>) -> Self {
        self.vectors.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// HighlightType
///////////////////////////////////////////////////////////////////////////////
/// Highlighting strategy applied to text search results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum HighlightType {
    #[default]
    /// Represents the Lexical case.
    Lexical,
    /// Represents the Semantic case.
    Semantic,
}

///////////////////////////////////////////////////////////////////////////////
// Highlighter
///////////////////////////////////////////////////////////////////////////////
/// Configuration for text highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Highlighter {
    pub(crate) highlight_type: HighlightType,
    pub(crate) params: HashMap<String, String>,
}

impl Highlighter {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            highlight_type: HighlightType::Lexical,
            params: HashMap::new(),
        }
    }

    /// Sets the highlight type and returns the updated value.
    pub fn highlight_type(mut self, value: HighlightType) -> Self {
        self.highlight_type = value;
        self
    }

    /// Sets the highlight type and returns this value for further mutation.
    pub fn set_highlight_type(&mut self, value: HighlightType) -> &mut Self {
        self.highlight_type = value;
        self
    }

    /// Returns the configured highlight type.
    pub fn get_highlight_type(&self) -> HighlightType {
        self.highlight_type
    }

    /// Sets the params and returns the updated value.
    pub fn params(mut self, value: HashMap<String, String>) -> Self {
        self.params = value;
        self
    }

    /// Sets the params and returns this value for further mutation.
    pub fn set_params(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.params = value;
        self
    }

    /// Returns the configured params.
    pub fn get_params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub(crate) fn into_proto(self) -> common::Highlighter {
        common::Highlighter {
            r#type: match self.highlight_type {
                HighlightType::Lexical => common::HighlightType::Lexical,
                HighlightType::Semantic => common::HighlightType::Semantic,
            } as i32,
            params: self
                .params
                .into_iter()
                .map(|(key, value)| common::KeyValuePair { key, value })
                .collect(),
            ..Default::default()
        }
    }
}

impl From<LexicalHighlighter> for Highlighter {
    fn from(value: LexicalHighlighter) -> Self {
        let mut params = HashMap::new();
        if !value.highlight_queries.is_empty() {
            params.insert(
                "highlight_query".into(),
                Value::Array(
                    value
                        .highlight_queries
                        .into_iter()
                        .map(|query| {
                            serde_json::json!({
                                "type": query.query_type,
                                "field": query.field,
                                "text": query.text,
                            })
                        })
                        .collect(),
                )
                .to_string(),
            );
        }
        if value.highlight_search_text {
            params.insert("highlight_search_text".into(), "true".into());
        }
        if !value.pre_tags.is_empty() {
            params.insert(
                "pre_tags".into(),
                serde_json::json!(value.pre_tags).to_string(),
            );
        }
        if !value.post_tags.is_empty() {
            params.insert(
                "post_tags".into(),
                serde_json::json!(value.post_tags).to_string(),
            );
        }
        if let Some(value) = value.fragment_offset {
            params.insert("fragment_offset".into(), value.to_string());
        }
        if let Some(value) = value.fragment_size {
            params.insert("fragment_size".into(), value.to_string());
        }
        if let Some(value) = value.num_of_fragments {
            params.insert("num_of_fragments".into(), value.to_string());
        }
        Self {
            highlight_type: HighlightType::Lexical,
            params,
        }
    }
}

impl From<SemanticHighlighter> for Highlighter {
    fn from(value: SemanticHighlighter) -> Self {
        let mut params = HashMap::new();
        if !value.queries.is_empty() {
            params.insert(
                "queries".into(),
                serde_json::json!(value.queries).to_string(),
            );
        }
        if !value.input_fields.is_empty() {
            params.insert(
                "input_fields".into(),
                serde_json::json!(value.input_fields).to_string(),
            );
        }
        if !value.pre_tags.is_empty() {
            params.insert(
                "pre_tags".into(),
                serde_json::json!(value.pre_tags).to_string(),
            );
        }
        if !value.post_tags.is_empty() {
            params.insert(
                "post_tags".into(),
                serde_json::json!(value.post_tags).to_string(),
            );
        }
        if let Some(value) = value.threshold {
            params.insert("threshold".into(), value.to_string());
        }
        if value.highlight_only {
            params.insert("highlight_only".into(), "true".into());
        }
        if !value.model_deployment_id.is_empty() {
            params.insert("model_deployment_id".into(), value.model_deployment_id);
        }
        if let Some(value) = value.max_client_batch_size {
            params.insert("max_client_batch_size".into(), value.to_string());
        }
        Self {
            highlight_type: HighlightType::Semantic,
            params,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// HighlightQuery
///////////////////////////////////////////////////////////////////////////////
/// A query fragment used to calculate text highlights.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HighlightQuery {
    pub(crate) query_type: String,
    pub(crate) field: String,
    pub(crate) text: String,
}

impl HighlightQuery {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            query_type: String::new(),
            field: String::new(),
            text: String::new(),
        }
    }

    /// Sets the query type and returns the updated value.
    pub fn query_type(mut self, value: impl Into<String>) -> Self {
        self.query_type = value.into();
        self
    }

    /// Sets the query type and returns this value for further mutation.
    pub fn set_query_type(&mut self, value: impl Into<String>) -> &mut Self {
        self.query_type = value.into();
        self
    }

    /// Returns the configured query type.
    pub fn get_query_type(&self) -> &str {
        &self.query_type
    }

    /// Sets the field and returns the updated value.
    pub fn field(mut self, value: impl Into<String>) -> Self {
        self.field = value.into();
        self
    }

    /// Sets the field and returns this value for further mutation.
    pub fn set_field(&mut self, value: impl Into<String>) -> &mut Self {
        self.field = value.into();
        self
    }

    /// Returns the configured field.
    pub fn get_field(&self) -> &str {
        &self.field
    }

    /// Sets the text and returns the updated value.
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    /// Sets the text and returns this value for further mutation.
    pub fn set_text(&mut self, value: impl Into<String>) -> &mut Self {
        self.text = value.into();
        self
    }

    /// Returns the configured text.
    pub fn get_text(&self) -> &str {
        &self.text
    }
}

///////////////////////////////////////////////////////////////////////////////
// LexicalHighlighter
///////////////////////////////////////////////////////////////////////////////
/// Configuration for lexical-match highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LexicalHighlighter {
    pub(crate) highlight_queries: Vec<HighlightQuery>,
    pub(crate) highlight_search_text: bool,
    pub(crate) pre_tags: Vec<String>,
    pub(crate) post_tags: Vec<String>,
    pub(crate) fragment_offset: Option<i64>,
    pub(crate) fragment_size: Option<i64>,
    pub(crate) num_of_fragments: Option<i64>,
}

impl LexicalHighlighter {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            highlight_queries: Vec::new(),
            highlight_search_text: false,
            pre_tags: Vec::new(),
            post_tags: Vec::new(),
            fragment_offset: None,
            fragment_size: None,
            num_of_fragments: None,
        }
    }

    /// Sets the highlight queries and returns the updated value.
    pub fn highlight_queries(mut self, value: Vec<HighlightQuery>) -> Self {
        self.highlight_queries = value;
        self
    }

    /// Sets the highlight queries and returns this value for further mutation.
    pub fn set_highlight_queries(&mut self, value: Vec<HighlightQuery>) -> &mut Self {
        self.highlight_queries = value;
        self
    }

    /// Returns the configured highlight queries.
    pub fn get_highlight_queries(&self) -> &[HighlightQuery] {
        &self.highlight_queries
    }

    /// Sets the highlight search text and returns the updated value.
    pub fn highlight_search_text(mut self, value: bool) -> Self {
        self.highlight_search_text = value;
        self
    }

    /// Sets the highlight search text and returns this value for further mutation.
    pub fn set_highlight_search_text(&mut self, value: bool) -> &mut Self {
        self.highlight_search_text = value;
        self
    }

    /// Returns the configured highlight search text.
    pub fn get_highlight_search_text(&self) -> bool {
        self.highlight_search_text
    }

    /// Sets the pre tags and returns the updated value.
    pub fn pre_tags(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pre_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the pre tags and returns this value for further mutation.
    pub fn set_pre_tags(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.pre_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured pre tags.
    pub fn get_pre_tags(&self) -> &[String] {
        &self.pre_tags
    }

    /// Sets the post tags and returns the updated value.
    pub fn post_tags(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.post_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the post tags and returns this value for further mutation.
    pub fn set_post_tags(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.post_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured post tags.
    pub fn get_post_tags(&self) -> &[String] {
        &self.post_tags
    }

    /// Sets the fragment offset and returns the updated value.
    pub fn fragment_offset(mut self, value: i64) -> Self {
        self.fragment_offset = Some(value);
        self
    }

    /// Sets the fragment offset and returns this value for further mutation.
    pub fn set_fragment_offset(&mut self, value: i64) -> &mut Self {
        self.fragment_offset = Some(value);
        self
    }

    /// Returns the configured fragment offset.
    pub fn get_fragment_offset(&self) -> Option<i64> {
        self.fragment_offset
    }

    /// Sets the fragment size and returns the updated value.
    pub fn fragment_size(mut self, value: i64) -> Self {
        self.fragment_size = Some(value);
        self
    }

    /// Sets the fragment size and returns this value for further mutation.
    pub fn set_fragment_size(&mut self, value: i64) -> &mut Self {
        self.fragment_size = Some(value);
        self
    }

    /// Returns the configured fragment size.
    pub fn get_fragment_size(&self) -> Option<i64> {
        self.fragment_size
    }

    /// Sets the num of fragments and returns the updated value.
    pub fn num_of_fragments(mut self, value: i64) -> Self {
        self.num_of_fragments = Some(value);
        self
    }

    /// Sets the num of fragments and returns this value for further mutation.
    pub fn set_num_of_fragments(&mut self, value: i64) -> &mut Self {
        self.num_of_fragments = Some(value);
        self
    }

    /// Returns the configured num of fragments.
    pub fn get_num_of_fragments(&self) -> Option<i64> {
        self.num_of_fragments
    }

    /// Adds one add highlight query to the existing values.
    pub fn add_highlight_query(mut self, value: HighlightQuery) -> Self {
        self.highlight_queries.push(value);
        self
    }

    /// Adds one add pre tag to the existing values.
    pub fn add_pre_tag(mut self, value: impl Into<String>) -> Self {
        self.pre_tags.push(value.into());
        self
    }

    /// Adds one add post tag to the existing values.
    pub fn add_post_tag(mut self, value: impl Into<String>) -> Self {
        self.post_tags.push(value.into());
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// SemanticHighlighter
///////////////////////////////////////////////////////////////////////////////
/// Configuration for semantic highlighting.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SemanticHighlighter {
    pub(crate) queries: Vec<String>,
    pub(crate) input_fields: Vec<String>,
    pub(crate) pre_tags: Vec<String>,
    pub(crate) post_tags: Vec<String>,
    pub(crate) threshold: Option<f32>,
    pub(crate) highlight_only: bool,
    pub(crate) model_deployment_id: String,
    pub(crate) max_client_batch_size: Option<i64>,
}

impl SemanticHighlighter {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
            input_fields: Vec::new(),
            pre_tags: Vec::new(),
            post_tags: Vec::new(),
            threshold: None,
            highlight_only: false,
            model_deployment_id: String::new(),
            max_client_batch_size: None,
        }
    }

    /// Sets the queries and returns the updated value.
    pub fn queries(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.queries = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the queries and returns this value for further mutation.
    pub fn set_queries(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.queries = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured queries.
    pub fn get_queries(&self) -> &[String] {
        &self.queries
    }

    /// Sets the input fields and returns the updated value.
    pub fn input_fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.input_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the input fields and returns this value for further mutation.
    pub fn set_input_fields(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.input_fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured input fields.
    pub fn get_input_fields(&self) -> &[String] {
        &self.input_fields
    }

    /// Sets the pre tags and returns the updated value.
    pub fn pre_tags(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pre_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the pre tags and returns this value for further mutation.
    pub fn set_pre_tags(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.pre_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured pre tags.
    pub fn get_pre_tags(&self) -> &[String] {
        &self.pre_tags
    }

    /// Sets the post tags and returns the updated value.
    pub fn post_tags(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.post_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the post tags and returns this value for further mutation.
    pub fn set_post_tags(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.post_tags = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured post tags.
    pub fn get_post_tags(&self) -> &[String] {
        &self.post_tags
    }

    /// Sets the threshold and returns the updated value.
    pub fn threshold(mut self, value: f32) -> Self {
        self.threshold = Some(value);
        self
    }

    /// Sets the threshold and returns this value for further mutation.
    pub fn set_threshold(&mut self, value: f32) -> &mut Self {
        self.threshold = Some(value);
        self
    }

    /// Returns the configured threshold.
    pub fn get_threshold(&self) -> Option<f32> {
        self.threshold
    }

    /// Sets the highlight only and returns the updated value.
    pub fn highlight_only(mut self, value: bool) -> Self {
        self.highlight_only = value;
        self
    }

    /// Sets the highlight only and returns this value for further mutation.
    pub fn set_highlight_only(&mut self, value: bool) -> &mut Self {
        self.highlight_only = value;
        self
    }

    /// Returns the configured highlight only.
    pub fn get_highlight_only(&self) -> bool {
        self.highlight_only
    }

    /// Sets the model deployment id and returns the updated value.
    pub fn model_deployment_id(mut self, value: impl Into<String>) -> Self {
        self.model_deployment_id = value.into();
        self
    }

    /// Sets the model deployment id and returns this value for further mutation.
    pub fn set_model_deployment_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.model_deployment_id = value.into();
        self
    }

    /// Returns the configured model deployment id.
    pub fn get_model_deployment_id(&self) -> &str {
        &self.model_deployment_id
    }

    /// Sets the max client batch size and returns the updated value.
    pub fn max_client_batch_size(mut self, value: i64) -> Self {
        self.max_client_batch_size = Some(value);
        self
    }

    /// Sets the max client batch size and returns this value for further mutation.
    pub fn set_max_client_batch_size(&mut self, value: i64) -> &mut Self {
        self.max_client_batch_size = Some(value);
        self
    }

    /// Returns the configured max client batch size.
    pub fn get_max_client_batch_size(&self) -> Option<i64> {
        self.max_client_batch_size
    }

    /// Adds one add query to the existing values.
    pub fn add_query(mut self, value: impl Into<String>) -> Self {
        self.queries.push(value.into());
        self
    }

    /// Adds one add input field to the existing values.
    pub fn add_input_field(mut self, value: impl Into<String>) -> Self {
        self.input_fields.push(value.into());
        self
    }

    /// Adds one add pre tag to the existing values.
    pub fn add_pre_tag(mut self, value: impl Into<String>) -> Self {
        self.pre_tags.push(value.into());
        self
    }

    /// Adds one add post tag to the existing values.
    pub fn add_post_tag(mut self, value: impl Into<String>) -> Self {
        self.post_tags.push(value.into());
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// QueryResults
///////////////////////////////////////////////////////////////////////////////
/// Column-oriented results returned by a query.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct QueryResults {
    pub(crate) output_fields: Vec<FieldData>,
    pub(crate) output_field_names: Vec<String>,
    pub(crate) element_indices: Vec<Vec<i64>>,
}

impl QueryResults {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            output_fields: Vec::new(),
            output_field_names: Vec::new(),
            element_indices: Vec::new(),
        }
    }

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, value: Vec<FieldData>) -> Self {
        self.output_fields = value;
        self
    }

    /// Sets the output fields and returns this value for further mutation.
    pub fn set_output_fields(&mut self, value: Vec<FieldData>) -> &mut Self {
        self.output_fields = value;
        self
    }

    /// Returns the configured output fields.
    pub fn get_output_fields(&self) -> &[FieldData] {
        &self.output_fields
    }

    /// Performs the output field names operation.
    pub fn output_field_names(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.output_field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the output field names and returns this value for further mutation.
    pub fn set_output_field_names(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.output_field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured output field names.
    pub fn get_output_field_names(&self) -> &[String] {
        &self.output_field_names
    }

    /// Adds one add output field to the existing values.
    pub fn add_output_field(mut self, value: FieldData) -> Self {
        self.output_fields.push(value);
        self
    }

    /// Returns the configured output field.
    pub fn get_output_field(&self, name: &str) -> Option<&FieldData> {
        self.output_fields.iter().find(|field| field.name() == name)
    }

    /// Returns the configured row count.
    pub fn get_row_count(&self) -> u64 {
        if self.output_fields.len() == 1 && self.output_fields[0].name() == "count(*)" {
            if let FieldData::Int64 { values, .. } = self.output_fields[0].inner() {
                return values
                    .first()
                    .and_then(|value| u64::try_from(*value).ok())
                    .unwrap_or_default();
            }
        }
        self.output_fields
            .first()
            .map_or(0, |field| field.len() as u64)
    }

    /// Adds one add output field name to the existing values.
    pub fn add_output_field_name(mut self, value: impl Into<String>) -> Self {
        self.output_field_names.push(value.into());
        self
    }

    /// Sets the per-row element indices and returns the updated value.
    ///
    /// Each inner list holds the element offsets within an entity when an `element_filter`
    /// iterator is used; ordinary queries leave this empty.
    pub fn element_indices(mut self, value: Vec<Vec<i64>>) -> Self {
        self.element_indices = value;
        self
    }

    /// Sets the per-row element indices and returns this value for further mutation.
    pub fn set_element_indices(&mut self, value: Vec<Vec<i64>>) -> &mut Self {
        self.element_indices = value;
        self
    }

    /// Returns the per-row element indices, one list per entity.
    pub fn get_element_indices(&self) -> &[Vec<i64>] {
        &self.element_indices
    }

    /// Appends one entity's element indices and returns the updated value.
    pub fn add_element_indices(mut self, value: Vec<i64>) -> Self {
        self.element_indices.push(value);
        self
    }

    /// Returns whether empty.
    pub fn is_empty(&self) -> bool {
        self.get_row_count() == 0
    }

    /// Returns a borrowing iterator over query rows.
    ///
    /// The iterator reads values directly from the column-oriented result data
    /// without materializing JSON maps for every row.
    pub fn rows(&self) -> Result<ResultRowIter<'_>> {
        let row_count = output_row_count(&self.output_fields)?;
        Ok(ResultRowIter {
            output_fields: &self.output_fields,
            output_field_names: &self.output_field_names,
            ids: None,
            primary_field_name: None,
            scores: None,
            score_field_name: None,
            element_indices: None,
            next: 0,
            row_count,
        })
    }

    /// Materializes all query rows as owned JSON objects.
    ///
    /// Use this when rows must be owned, mutated, serialized, or passed to a
    /// JSON-oriented API. Prefer [`Self::rows`] for borrowing typed access
    /// without allocating a map and JSON values for every row.
    pub fn get_output_rows(&self) -> Result<Vec<EntityRow>> {
        let row_count = output_row_count(&self.output_fields)?;
        (0..row_count)
            .map(|index| output_row(&self.output_fields, &self.output_field_names, index))
            .collect()
    }

    /// Materializes one query row as an owned JSON object.
    ///
    /// Use this when an owned or serializable row is required. Prefer
    /// [`Self::rows`] when processing result values through borrowing typed
    /// accessors.
    pub fn get_output_row(&self, index: usize) -> Result<EntityRow> {
        let row_count = output_row_count(&self.output_fields)?;
        if index >= row_count {
            return Err(row_index_error(index, row_count));
        }
        output_row(&self.output_fields, &self.output_field_names, index)
    }
}

///////////////////////////////////////////////////////////////////////////////
// HighlightResult
///////////////////////////////////////////////////////////////////////////////
/// Highlighted fragments returned for one output field.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct HighlightResult {
    pub(crate) field_name: String,
    pub(crate) fragments: Vec<String>,
    pub(crate) scores: Vec<f32>,
}

impl HighlightResult {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            field_name: String::new(),
            fragments: Vec::new(),
            scores: Vec::new(),
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

    /// Sets the fragments and returns the updated value.
    pub fn fragments(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.fragments = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the fragments and returns this value for further mutation.
    pub fn set_fragments(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.fragments = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured fragments.
    pub fn get_fragments(&self) -> &[String] {
        &self.fragments
    }

    /// Sets the scores and returns the updated value.
    pub fn scores(mut self, value: Vec<f32>) -> Self {
        self.scores = value;
        self
    }

    /// Sets the scores and returns this value for further mutation.
    pub fn set_scores(&mut self, value: Vec<f32>) -> &mut Self {
        self.scores = value;
        self
    }

    /// Returns the configured scores.
    pub fn get_scores(&self) -> &[f32] {
        &self.scores
    }

    /// Adds one add fragment to the existing values.
    pub fn add_fragment(mut self, value: impl Into<String>) -> Self {
        self.fragments.push(value.into());
        self
    }

    /// Adds one add score to the existing values.
    pub fn add_score(mut self, value: f32) -> Self {
        self.scores.push(value);
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// SingleResult
///////////////////////////////////////////////////////////////////////////////
/// The complete list of matched entities and scores for one query vector.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SingleResult {
    pub(crate) ids: Ids,
    pub(crate) scores: Vec<f32>,
    pub(crate) element_indices: Option<Vec<i64>>,
    pub(crate) output_fields: Vec<FieldData>,
    pub(crate) output_field_names: Vec<String>,
    pub(crate) primary_field_name: String,
    pub(crate) score_field_name: String,
    pub(crate) highlight_results: Vec<HashMap<String, HighlightResult>>,
}

impl SingleResult {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            ids: Ids::default(),
            scores: Vec::new(),
            element_indices: None,
            output_fields: Vec::new(),
            output_field_names: Vec::new(),
            primary_field_name: String::new(),
            score_field_name: String::new(),
            highlight_results: Vec::new(),
        }
    }

    /// Sets the ids and returns the updated value.
    pub fn ids(mut self, value: Ids) -> Self {
        self.ids = value;
        self
    }

    /// Sets the ids and returns this value for further mutation.
    pub fn set_ids(&mut self, value: Ids) -> &mut Self {
        self.ids = value;
        self
    }

    /// Returns the configured ids.
    pub fn get_ids(&self) -> &Ids {
        &self.ids
    }

    /// Sets the scores and returns the updated value.
    pub fn scores(mut self, value: Vec<f32>) -> Self {
        self.scores = value;
        self
    }

    /// Sets the scores and returns this value for further mutation.
    pub fn set_scores(&mut self, value: Vec<f32>) -> &mut Self {
        self.scores = value;
        self
    }

    /// Returns the configured scores.
    pub fn get_scores(&self) -> &[f32] {
        &self.scores
    }

    /// Element offsets within a matched struct field.
    ///
    /// This is absent for ordinary entity-level searches.
    pub fn element_indices(mut self, value: Option<Vec<i64>>) -> Self {
        self.element_indices = value;
        self
    }

    /// Sets the element indices and returns this value for further mutation.
    pub fn set_element_indices(&mut self, value: Option<Vec<i64>>) -> &mut Self {
        self.element_indices = value;
        self
    }

    /// Returns the configured element indices.
    pub fn get_element_indices(&self) -> Option<&[i64]> {
        self.element_indices.as_deref()
    }

    /// Adds one add element index to the existing values.
    pub fn add_element_index(mut self, value: i64) -> Self {
        self.element_indices
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// Sets the output fields and returns the updated value.
    pub fn output_fields(mut self, value: Vec<FieldData>) -> Self {
        self.output_fields = value;
        self
    }

    /// Sets the output fields and returns this value for further mutation.
    pub fn set_output_fields(&mut self, value: Vec<FieldData>) -> &mut Self {
        self.output_fields = value;
        self
    }

    /// Returns the configured output fields.
    pub fn get_output_fields(&self) -> &[FieldData] {
        &self.output_fields
    }

    /// Performs the output field names operation.
    pub fn output_field_names(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.output_field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the output field names and returns this value for further mutation.
    pub fn set_output_field_names(
        &mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.output_field_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the configured output field names.
    pub fn get_output_field_names(&self) -> &[String] {
        &self.output_field_names
    }

    /// Sets the primary field name and returns the updated value.
    pub fn primary_field_name(mut self, value: impl Into<String>) -> Self {
        self.primary_field_name = value.into();
        self
    }

    /// Sets the primary field name and returns this value for further mutation.
    pub fn set_primary_field_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.primary_field_name = value.into();
        self
    }

    /// Returns the configured primary field name.
    pub fn get_primary_field_name(&self) -> &str {
        &self.primary_field_name
    }

    /// Sets the score field name and returns the updated value.
    pub fn score_field_name(mut self, value: impl Into<String>) -> Self {
        self.score_field_name = value.into();
        self
    }

    /// Sets the score field name and returns this value for further mutation.
    pub fn set_score_field_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.score_field_name = value.into();
        self
    }

    /// Returns the configured score field name.
    pub fn get_score_field_name(&self) -> &str {
        &self.score_field_name
    }

    /// Sets the highlight results and returns the updated value.
    pub fn highlight_results(mut self, value: Vec<HashMap<String, HighlightResult>>) -> Self {
        self.highlight_results = value;
        self
    }

    /// Sets the highlight results and returns this value for further mutation.
    pub fn set_highlight_results(
        &mut self,
        value: Vec<HashMap<String, HighlightResult>>,
    ) -> &mut Self {
        self.highlight_results = value;
        self
    }

    /// Returns the configured highlight results.
    pub fn get_highlight_results(&self) -> &[HashMap<String, HighlightResult>] {
        &self.highlight_results
    }

    /// Returns the len.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns whether empty.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Adds one add score to the existing values.
    pub fn add_score(mut self, value: f32) -> Self {
        self.scores.push(value);
        self
    }

    /// Adds one add output field to the existing values.
    pub fn add_output_field(mut self, value: FieldData) -> Self {
        self.output_fields.push(value);
        self
    }

    /// Returns the configured output field.
    pub fn get_output_field(&self, name: &str) -> Option<&FieldData> {
        self.output_fields.iter().find(|field| field.name() == name)
    }

    /// Adds one add output field name to the existing values.
    pub fn add_output_field_name(mut self, value: impl Into<String>) -> Self {
        self.output_field_names.push(value.into());
        self
    }

    /// Adds one add highlight result to the existing values.
    pub fn add_highlight_result(mut self, value: HashMap<String, HighlightResult>) -> Self {
        self.highlight_results.push(value);
        self
    }

    /// Returns a borrowing iterator over matched rows.
    ///
    /// The iterator reads values directly from the column-oriented result data
    /// without materializing JSON maps for every row.
    pub fn rows(&self) -> Result<ResultRowIter<'_>> {
        self.validate_row_counts()?;
        Ok(ResultRowIter {
            output_fields: &self.output_fields,
            output_field_names: &self.output_field_names,
            ids: Some(&self.ids),
            primary_field_name: Some(&self.primary_field_name),
            scores: Some(&self.scores),
            score_field_name: Some(&self.score_field_name),
            element_indices: self.element_indices.as_deref(),
            next: 0,
            row_count: self.len(),
        })
    }

    /// Materializes all matched rows as owned JSON objects.
    ///
    /// Each object includes the primary key, score, and requested output
    /// fields. Use this when rows must be owned, mutated, serialized, or passed
    /// to a JSON-oriented API. Prefer [`Self::rows`] for borrowing typed access
    /// without allocating a map and JSON values for every row.
    pub fn get_output_rows(&self) -> Result<Vec<EntityRow>> {
        self.validate_row_counts()?;
        (0..self.len())
            .map(|index| self.get_output_row(index))
            .collect()
    }

    /// Materializes one matched row as an owned JSON object.
    ///
    /// The object includes the primary key, score, and requested output
    /// fields. Use this when an owned or serializable row is required. Prefer
    /// [`Self::rows`] when processing result values through borrowing typed
    /// accessors.
    pub fn get_output_row(&self, index: usize) -> Result<EntityRow> {
        self.validate_row_counts()?;
        if index >= self.len() {
            return Err(row_index_error(index, self.len()));
        }
        let mut row = output_row(&self.output_fields, &self.output_field_names, index)?;
        row.insert(self.primary_field_name.clone(), self.ids.value_at(index)?);
        row.insert(
            self.score_field_name.clone(),
            serde_json::to_value(self.scores[index])?,
        );
        Ok(row)
    }

    fn validate_row_counts(&self) -> Result<()> {
        let row_count = self.ids.len();
        if row_count > 0 && (self.primary_field_name.is_empty() || self.score_field_name.is_empty())
        {
            return Err(Error::MalformedResponse(
                "search result primary-key or score field name is empty".into(),
            ));
        }
        if self.scores.len() != row_count
            || self
                .element_indices
                .as_ref()
                .is_some_and(|indices| indices.len() != row_count)
            || self
                .output_fields
                .iter()
                .any(|field| field.len() != row_count)
            || (!self.highlight_results.is_empty() && self.highlight_results.len() != row_count)
        {
            return Err(Error::MalformedResponse(
                "search result fields contain unequal row counts".into(),
            ));
        }
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
// ResultRow
///////////////////////////////////////////////////////////////////////////////
/// Borrowed view of one query or search result row.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ResultRow<'a> {
    output_fields: &'a [FieldData],
    output_field_names: &'a [String],
    ids: Option<&'a Ids>,
    primary_field_name: Option<&'a str>,
    scores: Option<&'a [f32]>,
    score_field_name: Option<&'a str>,
    element_indices: Option<&'a [i64]>,
    index: usize,
}

impl<'a> ResultRow<'a> {
    /// Returns the row's zero-based position within its query result.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the matched element's offset within its parent struct-array or
    /// ArrayOfVector field.
    ///
    /// Ordinary entity-level search rows and query rows return `None`.
    pub fn element_offset(&self) -> Option<i64> {
        self.element_indices
            .and_then(|indices| indices.get(self.index))
            .copied()
    }

    /// Materializes this row as an owned JSON object.
    ///
    /// Use this for generic display, serialization, or APIs that require an
    /// owned map. Prefer the typed getters when the field types are known.
    pub fn to_entity_row(&self) -> Result<EntityRow> {
        let mut row = output_row(self.output_fields, self.output_field_names, self.index)?;
        if let (Some(ids), Some(name)) = (self.ids, self.primary_field_name) {
            row.insert(name.to_owned(), ids.value_at(self.index)?);
        }
        if let (Some(scores), Some(name)) = (self.scores, self.score_field_name) {
            let score = scores
                .get(self.index)
                .copied()
                .ok_or_else(|| row_index_error(self.index, scores.len()))?;
            row.insert(name.to_owned(), serde_json::to_value(score)?);
        }
        Ok(row)
    }

    /// Returns a borrowed, type-preserving value for the named field.
    ///
    /// This is useful for generic result processing when the field type is not
    /// known at compile time. Prefer the typed getters below when the schema is
    /// known because they validate the expected type directly.
    pub fn get(&self, name: &str) -> Result<ResultValue<'a>> {
        self.value(name)
    }

    /// Returns whether the named field is null in this row.
    pub fn is_null(&self, name: &str) -> Result<bool> {
        Ok(matches!(self.value(name)?, ResultValue::Null))
    }

    /// Returns a boolean field.
    pub fn get_bool(&self, name: &str) -> Result<bool> {
        match self.value(name)? {
            ResultValue::Bool(value) => Ok(value),
            ResultValue::Json(Value::Bool(value)) => Ok(*value),
            value => Err(result_row_type_error(name, "boolean", value.kind())),
        }
    }

    /// Returns an Int8 field.
    pub fn get_i8(&self, name: &str) -> Result<i8> {
        match self.value(name)? {
            ResultValue::Int8(value) => Ok(value),
            ResultValue::Json(value) => value
                .as_i64()
                .and_then(|value| i8::try_from(value).ok())
                .ok_or_else(|| result_row_type_error(name, "i8", result_json_kind(value))),
            value => Err(result_row_type_error(name, "i8", value.kind())),
        }
    }

    /// Returns an Int16 field.
    pub fn get_i16(&self, name: &str) -> Result<i16> {
        match self.value(name)? {
            ResultValue::Int16(value) => Ok(value),
            ResultValue::Json(value) => value
                .as_i64()
                .and_then(|value| i16::try_from(value).ok())
                .ok_or_else(|| result_row_type_error(name, "i16", result_json_kind(value))),
            value => Err(result_row_type_error(name, "i16", value.kind())),
        }
    }

    /// Returns an Int32 field.
    pub fn get_i32(&self, name: &str) -> Result<i32> {
        match self.value(name)? {
            ResultValue::Int32(value) => Ok(value),
            ResultValue::Json(value) => value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| result_row_type_error(name, "i32", result_json_kind(value))),
            value => Err(result_row_type_error(name, "i32", value.kind())),
        }
    }

    /// Returns a borrowed string field.
    pub fn get_str(&self, name: &str) -> Result<&'a str> {
        match self.value(name)? {
            ResultValue::String(value) => Ok(value),
            ResultValue::Geometry(value) => Ok(value),
            ResultValue::Timestamptz(value) => Ok(value),
            ResultValue::Json(Value::String(value)) => Ok(value),
            value => Err(result_row_type_error(name, "string", value.kind())),
        }
    }

    /// Returns an integer field as `i64`.
    ///
    /// Int8, Int16, Int32, and Int64 scalar fields are accepted.
    pub fn get_i64(&self, name: &str) -> Result<i64> {
        match self.value(name)? {
            ResultValue::Int8(value) => Ok(i64::from(value)),
            ResultValue::Int16(value) => Ok(i64::from(value)),
            ResultValue::Int32(value) => Ok(i64::from(value)),
            ResultValue::Int64(value) => Ok(value),
            ResultValue::Json(value) => value
                .as_i64()
                .ok_or_else(|| result_row_type_error(name, "integer", result_json_kind(value))),
            value => Err(result_row_type_error(name, "integer", value.kind())),
        }
    }

    /// Returns a single-precision floating-point field.
    pub fn get_f32(&self, name: &str) -> Result<f32> {
        match self.value(name)? {
            ResultValue::Float(value) => Ok(value),
            ResultValue::Json(value) => value
                .as_f64()
                .filter(|value| value.is_finite() && value.abs() <= f64::from(f32::MAX))
                .map(|value| value as f32)
                .ok_or_else(|| result_row_type_error(name, "finite f32", result_json_kind(value))),
            value => Err(result_row_type_error(name, "f32", value.kind())),
        }
    }

    /// Returns a double-precision floating-point field.
    ///
    /// Float and Double scalar fields are accepted because widening `f32` to
    /// `f64` does not lose precision.
    pub fn get_f64(&self, name: &str) -> Result<f64> {
        match self.value(name)? {
            ResultValue::Float(value) => Ok(f64::from(value)),
            ResultValue::Double(value) => Ok(value),
            ResultValue::Json(value) => value
                .as_f64()
                .filter(|value| value.is_finite())
                .ok_or_else(|| result_row_type_error(name, "finite f64", result_json_kind(value))),
            value => Err(result_row_type_error(name, "f64", value.kind())),
        }
    }

    /// Returns a borrowed boolean-array field.
    pub fn get_array_bool(&self, name: &str) -> Result<&'a [bool]> {
        match self.value(name)? {
            ResultValue::ArrayBool(value) => Ok(value),
            value => Err(result_row_type_error(name, "boolean array", value.kind())),
        }
    }

    /// Returns a borrowed Int8-array field.
    pub fn get_array_i8(&self, name: &str) -> Result<&'a [i8]> {
        match self.value(name)? {
            ResultValue::ArrayInt8(value) => Ok(value),
            value => Err(result_row_type_error(name, "i8 array", value.kind())),
        }
    }

    /// Returns a borrowed Int16-array field.
    pub fn get_array_i16(&self, name: &str) -> Result<&'a [i16]> {
        match self.value(name)? {
            ResultValue::ArrayInt16(value) => Ok(value),
            value => Err(result_row_type_error(name, "i16 array", value.kind())),
        }
    }

    /// Returns a borrowed Int32-array field.
    pub fn get_array_i32(&self, name: &str) -> Result<&'a [i32]> {
        match self.value(name)? {
            ResultValue::ArrayInt32(value) => Ok(value),
            value => Err(result_row_type_error(name, "i32 array", value.kind())),
        }
    }

    /// Returns a borrowed Int64-array field.
    pub fn get_array_i64(&self, name: &str) -> Result<&'a [i64]> {
        match self.value(name)? {
            ResultValue::ArrayInt64(value) => Ok(value),
            value => Err(result_row_type_error(name, "i64 array", value.kind())),
        }
    }

    /// Returns a borrowed Float-array field.
    pub fn get_array_f32(&self, name: &str) -> Result<&'a [f32]> {
        match self.value(name)? {
            ResultValue::ArrayFloat(value) => Ok(value),
            value => Err(result_row_type_error(name, "f32 array", value.kind())),
        }
    }

    /// Returns a borrowed Double-array field.
    pub fn get_array_f64(&self, name: &str) -> Result<&'a [f64]> {
        match self.value(name)? {
            ResultValue::ArrayDouble(value) => Ok(value),
            value => Err(result_row_type_error(name, "f64 array", value.kind())),
        }
    }

    /// Returns a borrowed VarChar-array field.
    pub fn get_array_varchar(&self, name: &str) -> Result<&'a [String]> {
        match self.value(name)? {
            ResultValue::ArrayVarChar(value) => Ok(value),
            value => Err(result_row_type_error(name, "string array", value.kind())),
        }
    }

    /// Returns a borrowed struct-array field.
    pub fn get_struct(&self, name: &str) -> Result<&'a [StructValue]> {
        match self.value(name)? {
            ResultValue::Struct(value) => Ok(value),
            value => Err(result_row_type_error(name, "struct array", value.kind())),
        }
    }

    /// Returns a borrowed float-vector field.
    pub fn get_float_vector(&self, name: &str) -> Result<&'a [f32]> {
        match self.value(name)? {
            ResultValue::FloatVector(value) => Ok(value),
            value => Err(result_row_type_error(name, "float vector", value.kind())),
        }
    }

    /// Returns a borrowed binary-vector field.
    pub fn get_binary_vector(&self, name: &str) -> Result<&'a [u8]> {
        match self.value(name)? {
            ResultValue::BinaryVector(value) => Ok(value),
            value => Err(result_row_type_error(name, "binary vector", value.kind())),
        }
    }

    /// Returns a borrowed Float16-vector field as IEEE 754 half-precision bits.
    pub fn get_float16_vector(&self, name: &str) -> Result<&'a [u16]> {
        match self.value(name)? {
            ResultValue::Float16Vector(value) => Ok(value),
            value => Err(result_row_type_error(name, "float16 vector", value.kind())),
        }
    }

    /// Returns a borrowed BFloat16-vector field as bfloat16 bits.
    pub fn get_bfloat16_vector(&self, name: &str) -> Result<&'a [u16]> {
        match self.value(name)? {
            ResultValue::BFloat16Vector(value) => Ok(value),
            value => Err(result_row_type_error(name, "bfloat16 vector", value.kind())),
        }
    }

    /// Returns a borrowed sparse-float-vector field.
    pub fn get_sparse_float_vector(&self, name: &str) -> Result<&'a SparseVector> {
        match self.value(name)? {
            ResultValue::SparseFloatVector(value) => Ok(value),
            value => Err(result_row_type_error(
                name,
                "sparse float vector",
                value.kind(),
            )),
        }
    }

    /// Returns a borrowed Int8-vector field.
    pub fn get_int8_vector(&self, name: &str) -> Result<&'a [i8]> {
        match self.value(name)? {
            ResultValue::Int8Vector(value) => Ok(value),
            value => Err(result_row_type_error(name, "int8 vector", value.kind())),
        }
    }

    /// Returns a borrowed geometry field in the server's text representation.
    pub fn get_geometry(&self, name: &str) -> Result<&'a str> {
        match self.value(name)? {
            ResultValue::Geometry(value) => Ok(value),
            value => Err(result_row_type_error(name, "geometry", value.kind())),
        }
    }

    /// Returns a borrowed timestamptz field in the server's text representation.
    pub fn get_timestamptz(&self, name: &str) -> Result<&'a str> {
        match self.value(name)? {
            ResultValue::Timestamptz(value) => Ok(value),
            value => Err(result_row_type_error(name, "timestamptz", value.kind())),
        }
    }

    /// Returns a borrowed JSON field or dynamic-field value.
    pub fn get_json(&self, name: &str) -> Result<&'a Value> {
        match self.value(name)? {
            ResultValue::Json(value) => Ok(value),
            value => Err(result_row_type_error(name, "JSON", value.kind())),
        }
    }

    fn value(&self, name: &str) -> Result<ResultValue<'a>> {
        if self.primary_field_name == Some(name) {
            let ids = self.ids.ok_or_else(|| {
                Error::MalformedResponse("result row primary-key source is missing".into())
            })?;
            return match ids {
                Ids::Int64(values) => values.get(self.index).copied().map(ResultValue::Int64),
                Ids::VarChar(values) => values
                    .get(self.index)
                    .map(String::as_str)
                    .map(ResultValue::String),
            }
            .ok_or_else(|| row_index_error(self.index, ids.len()));
        }

        if self.score_field_name == Some(name) {
            let scores = self.scores.ok_or_else(|| {
                Error::MalformedResponse("result row score source is missing".into())
            })?;
            return scores
                .get(self.index)
                .copied()
                .map(ResultValue::Float)
                .ok_or_else(|| row_index_error(self.index, scores.len()));
        }

        if let Some(field) = self.output_fields.iter().find(|field| field.name() == name) {
            return result_field_value(field, self.index);
        }

        let dynamic_requested = self
            .output_field_names
            .iter()
            .any(|field| field == "$meta" || field == name);
        if dynamic_requested {
            for field in self
                .output_fields
                .iter()
                .filter(|field| field.name() == "$meta")
            {
                if let ResultValue::Json(Value::Object(values)) =
                    result_field_value(field, self.index)?
                {
                    if let Some(value) = values.get(name) {
                        return Ok(ResultValue::Json(value));
                    }
                }
            }
        }

        Err(Error::validation(
            name.into(),
            "field is not present in the result row".into(),
        ))
    }
}

///////////////////////////////////////////////////////////////////////////////
// ResultRowIter
///////////////////////////////////////////////////////////////////////////////
/// Borrowing iterator over query or search result rows.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ResultRowIter<'a> {
    output_fields: &'a [FieldData],
    output_field_names: &'a [String],
    ids: Option<&'a Ids>,
    primary_field_name: Option<&'a str>,
    scores: Option<&'a [f32]>,
    score_field_name: Option<&'a str>,
    element_indices: Option<&'a [i64]>,
    next: usize,
    row_count: usize,
}

impl<'a> Iterator for ResultRowIter<'a> {
    type Item = ResultRow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.row_count {
            return None;
        }
        let row = ResultRow {
            output_fields: self.output_fields,
            output_field_names: self.output_field_names,
            ids: self.ids,
            primary_field_name: self.primary_field_name,
            scores: self.scores,
            score_field_name: self.score_field_name,
            element_indices: self.element_indices,
            index: self.next,
        };
        self.next += 1;
        Some(row)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.row_count.saturating_sub(self.next);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ResultRowIter<'_> {}

impl std::iter::FusedIterator for ResultRowIter<'_> {}

///////////////////////////////////////////////////////////////////////////////
// ResultValue
///////////////////////////////////////////////////////////////////////////////
/// Borrowed, type-preserving value from one query or search result row.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ResultValue<'a> {
    /// Boolean scalar.
    Bool(bool),
    /// Int8 scalar.
    Int8(i8),
    /// Int16 scalar.
    Int16(i16),
    /// Int32 scalar.
    Int32(i32),
    /// Int64 scalar or integer primary key.
    Int64(i64),
    /// Single-precision scalar or search score.
    Float(f32),
    /// Double-precision scalar.
    Double(f64),
    /// VarChar scalar or string primary key.
    String(&'a str),
    /// Geometry value in the server's text representation.
    Geometry(&'a str),
    /// Timestamptz value in the server's text representation.
    Timestamptz(&'a str),
    /// JSON field or dynamic-field value.
    Json(&'a Value),
    /// Boolean array.
    ArrayBool(&'a [bool]),
    /// Int8 array.
    ArrayInt8(&'a [i8]),
    /// Int16 array.
    ArrayInt16(&'a [i16]),
    /// Int32 array.
    ArrayInt32(&'a [i32]),
    /// Int64 array.
    ArrayInt64(&'a [i64]),
    /// Float array.
    ArrayFloat(&'a [f32]),
    /// Double array.
    ArrayDouble(&'a [f64]),
    /// VarChar array.
    ArrayVarChar(&'a [String]),
    /// Struct-array value.
    Struct(&'a [StructValue]),
    /// Float vector.
    FloatVector(&'a [f32]),
    /// Packed binary vector bytes.
    BinaryVector(&'a [u8]),
    /// Float16 vector represented by IEEE 754 half-precision bits.
    Float16Vector(&'a [u16]),
    /// BFloat16 vector represented by bfloat16 bits.
    BFloat16Vector(&'a [u16]),
    /// Sparse float vector.
    SparseFloatVector(&'a SparseVector),
    /// Int8 vector.
    Int8Vector(&'a [i8]),
    /// Nullable field whose value is null in this row.
    Null,
}

impl ResultValue<'_> {
    fn kind(self) -> &'static str {
        match self {
            Self::Bool(_) => "boolean",
            Self::Int8(_) => "i8",
            Self::Int16(_) => "i16",
            Self::Int32(_) => "i32",
            Self::Int64(_) => "integer",
            Self::Float(_) => "f32",
            Self::Double(_) => "f64",
            Self::String(_) => "string",
            Self::Geometry(_) => "geometry",
            Self::Timestamptz(_) => "timestamptz",
            Self::Json(value) => result_json_kind(value),
            Self::ArrayBool(_) => "boolean array",
            Self::ArrayInt8(_) => "i8 array",
            Self::ArrayInt16(_) => "i16 array",
            Self::ArrayInt32(_) => "i32 array",
            Self::ArrayInt64(_) => "i64 array",
            Self::ArrayFloat(_) => "f32 array",
            Self::ArrayDouble(_) => "f64 array",
            Self::ArrayVarChar(_) => "string array",
            Self::Struct(_) => "struct array",
            Self::FloatVector(_) => "float vector",
            Self::BinaryVector(_) => "binary vector",
            Self::Float16Vector(_) => "float16 vector",
            Self::BFloat16Vector(_) => "bfloat16 vector",
            Self::SparseFloatVector(_) => "sparse float vector",
            Self::Int8Vector(_) => "int8 vector",
            Self::Null => "null",
        }
    }
}

fn result_field_value(field: &FieldData, index: usize) -> Result<ResultValue<'_>> {
    fn at<T>(values: &[T], index: usize) -> Result<&T> {
        values
            .get(index)
            .ok_or_else(|| row_index_error(index, values.len()))
    }

    match field {
        FieldData::Bool { values, .. } => Ok(ResultValue::Bool(*at(values, index)?)),
        FieldData::Int8 { values, .. } => Ok(ResultValue::Int8(*at(values, index)?)),
        FieldData::Int16 { values, .. } => Ok(ResultValue::Int16(*at(values, index)?)),
        FieldData::Int32 { values, .. } => Ok(ResultValue::Int32(*at(values, index)?)),
        FieldData::Int64 { values, .. } => Ok(ResultValue::Int64(*at(values, index)?)),
        FieldData::Float { values, .. } => Ok(ResultValue::Float(*at(values, index)?)),
        FieldData::Double { values, .. } => Ok(ResultValue::Double(*at(values, index)?)),
        FieldData::VarChar { values, .. } => Ok(ResultValue::String(at(values, index)?)),
        FieldData::Json { values, .. } => Ok(ResultValue::Json(at(values, index)?)),
        FieldData::Geometry { values, .. } => Ok(ResultValue::Geometry(at(values, index)?)),
        FieldData::Timestamptz { values, .. } => Ok(ResultValue::Timestamptz(at(values, index)?)),
        FieldData::ArrayBool { values, .. } => {
            Ok(ResultValue::ArrayBool(at(values, index)?.as_slice()))
        }
        FieldData::ArrayInt8 { values, .. } => {
            Ok(ResultValue::ArrayInt8(at(values, index)?.as_slice()))
        }
        FieldData::ArrayInt16 { values, .. } => {
            Ok(ResultValue::ArrayInt16(at(values, index)?.as_slice()))
        }
        FieldData::ArrayInt32 { values, .. } => {
            Ok(ResultValue::ArrayInt32(at(values, index)?.as_slice()))
        }
        FieldData::ArrayInt64 { values, .. } => {
            Ok(ResultValue::ArrayInt64(at(values, index)?.as_slice()))
        }
        FieldData::ArrayFloat { values, .. } => {
            Ok(ResultValue::ArrayFloat(at(values, index)?.as_slice()))
        }
        FieldData::ArrayDouble { values, .. } => {
            Ok(ResultValue::ArrayDouble(at(values, index)?.as_slice()))
        }
        FieldData::ArrayVarChar { values, .. } => {
            Ok(ResultValue::ArrayVarChar(at(values, index)?.as_slice()))
        }
        FieldData::Struct { values, .. } => Ok(ResultValue::Struct(at(values, index)?.as_slice())),
        FieldData::FloatVector { values, .. } => {
            Ok(ResultValue::FloatVector(at(values, index)?.as_slice()))
        }
        FieldData::BinaryVector { values, .. } => {
            Ok(ResultValue::BinaryVector(at(values, index)?.as_slice()))
        }
        FieldData::Float16Vector { values, .. } => {
            Ok(ResultValue::Float16Vector(at(values, index)?.as_slice()))
        }
        FieldData::BFloat16Vector { values, .. } => {
            Ok(ResultValue::BFloat16Vector(at(values, index)?.as_slice()))
        }
        FieldData::SparseFloatVector { values, .. } => {
            Ok(ResultValue::SparseFloatVector(at(values, index)?))
        }
        FieldData::Int8Vector { values, .. } => {
            Ok(ResultValue::Int8Vector(at(values, index)?.as_slice()))
        }
        FieldData::Nullable { data, valid_data } => {
            let valid = *at(valid_data, index)?;
            if !valid {
                return Ok(ResultValue::Null);
            }
            let compact_index = valid_data[..index].iter().filter(|valid| **valid).count();
            result_field_value(data, compact_index)
        }
    }
}

fn result_row_type_error(name: &str, expected: &str, actual: &str) -> Error {
    Error::validation(name.into(), format!("expected {expected}, found {actual}"))
}

fn result_json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "JSON array",
        Value::Object(_) => "JSON object",
    }
}

///////////////////////////////////////////////////////////////////////////////
// AggregationBucketValue
///////////////////////////////////////////////////////////////////////////////
/// Typed grouping-key value of an aggregation bucket.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AggregationBucketValue {
    /// Represents an integer value.
    Int(i64),
    /// Represents a string value.
    String(String),
    /// Represents a boolean value.
    Bool(bool),
}

impl AggregationBucketValue {
    pub(crate) fn from_proto(value: Option<schema::bucket_key_entry::Value>) -> Result<Self> {
        use schema::bucket_key_entry::Value;
        match value {
            Some(Value::IntVal(value)) => Ok(Self::Int(value)),
            Some(Value::StringVal(value)) => Ok(Self::String(value)),
            Some(Value::BoolVal(value)) => Ok(Self::Bool(value)),
            None => Err(Error::MalformedResponse(
                "aggregation bucket key contains no value".into(),
            )),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AggregationMetricValue
///////////////////////////////////////////////////////////////////////////////
/// Typed metric result of an aggregation bucket.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AggregationMetricValue {
    /// Represents an integer value.
    Int(i64),
    /// Represents a floating-point value.
    Double(f64),
    /// Represents a string value.
    String(String),
    /// Represents a boolean value.
    Bool(bool),
}

impl AggregationMetricValue {
    pub(crate) fn from_proto(value: Option<schema::metric_value::Value>) -> Result<Self> {
        use schema::metric_value::Value;
        match value {
            Some(Value::IntVal(value)) => Ok(Self::Int(value)),
            Some(Value::DoubleVal(value)) => Ok(Self::Double(value)),
            Some(Value::StringVal(value)) => Ok(Self::String(value)),
            Some(Value::BoolVal(value)) => Ok(Self::Bool(value)),
            None => Err(Error::MalformedResponse(
                "aggregation bucket metric contains no value".into(),
            )),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AggregationHitFieldValue
///////////////////////////////////////////////////////////////////////////////
/// Typed field value of an aggregation hit.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AggregationHitFieldValue {
    /// Represents an integer value.
    Int(i64),
    /// Represents a boolean value.
    Bool(bool),
    /// Represents a float value.
    Float(f32),
    /// Represents a double value.
    Double(f64),
    /// Represents a string value.
    String(String),
    /// Represents a bytes value.
    Bytes(Vec<u8>),
}

impl AggregationHitFieldValue {
    pub(crate) fn from_proto(value: Option<schema::agg_hit_field::Value>) -> Result<Self> {
        use schema::agg_hit_field::Value;
        match value {
            Some(Value::IntVal(value)) => Ok(Self::Int(value)),
            Some(Value::BoolVal(value)) => Ok(Self::Bool(value)),
            Some(Value::FloatVal(value)) => Ok(Self::Float(value)),
            Some(Value::DoubleVal(value)) => Ok(Self::Double(value)),
            Some(Value::StringVal(value)) => Ok(Self::String(value)),
            Some(Value::BytesVal(value)) => Ok(Self::Bytes(value)),
            None => Err(Error::MalformedResponse(
                "aggregation hit field contains no value".into(),
            )),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// AggregationHitPk
///////////////////////////////////////////////////////////////////////////////
/// Primary key of an aggregation hit.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AggregationHitPk {
    /// Represents an integer primary key.
    Int(i64),
    /// Represents a string primary key.
    String(String),
}

///////////////////////////////////////////////////////////////////////////////
// AggregationHitField
///////////////////////////////////////////////////////////////////////////////
/// One returned field of an aggregation hit.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AggregationHitField {
    pub(crate) field_id: i64,
    pub(crate) field_name: String,
    pub(crate) value: AggregationHitFieldValue,
}

impl AggregationHitField {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            field_id: 0,
            field_name: String::new(),
            value: AggregationHitFieldValue::Int(0),
        }
    }

    /// Sets the field id and returns the updated value.
    pub fn field_id(mut self, value: i64) -> Self {
        self.field_id = value;
        self
    }

    /// Sets the field id and returns this value for further mutation.
    pub fn set_field_id(&mut self, value: i64) -> &mut Self {
        self.field_id = value;
        self
    }

    /// Returns the field id.
    pub fn get_field_id(&self) -> i64 {
        self.field_id
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

    /// Returns the field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// Sets the field value and returns the updated value.
    pub fn value(mut self, value: AggregationHitFieldValue) -> Self {
        self.value = value;
        self
    }

    /// Sets the field value and returns this value for further mutation.
    pub fn set_value(&mut self, value: AggregationHitFieldValue) -> &mut Self {
        self.value = value;
        self
    }

    /// Returns the field value.
    pub fn get_value(&self) -> &AggregationHitFieldValue {
        &self.value
    }

    pub(crate) fn from_proto(value: schema::AggHitField) -> Result<Self> {
        Ok(Self {
            field_id: value.field_id,
            field_name: if value.field_name.is_empty() {
                value.field_id.to_string()
            } else {
                value.field_name
            },
            value: AggregationHitFieldValue::from_proto(value.value)?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// AggregationHit
///////////////////////////////////////////////////////////////////////////////
/// One document inside an aggregation bucket.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AggregationHit {
    pub(crate) pk: Option<AggregationHitPk>,
    pub(crate) score: f32,
    pub(crate) fields: Vec<AggregationHitField>,
}

impl AggregationHit {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            pk: None,
            score: 0.0,
            fields: Vec::new(),
        }
    }

    /// Sets the primary key and returns the updated value.
    pub fn pk(mut self, value: AggregationHitPk) -> Self {
        self.pk = Some(value);
        self
    }

    /// Sets the primary key and returns this value for further mutation.
    pub fn set_pk(&mut self, value: AggregationHitPk) -> &mut Self {
        self.pk = Some(value);
        self
    }

    /// Returns the primary key.
    pub fn get_pk(&self) -> Option<&AggregationHitPk> {
        self.pk.as_ref()
    }

    /// Sets the score and returns the updated value.
    pub fn score(mut self, value: f32) -> Self {
        self.score = value;
        self
    }

    /// Sets the score and returns this value for further mutation.
    pub fn set_score(&mut self, value: f32) -> &mut Self {
        self.score = value;
        self
    }

    /// Returns the score.
    pub fn get_score(&self) -> f32 {
        self.score
    }

    /// Sets the returned fields and returns the updated value.
    pub fn fields(mut self, values: impl IntoIterator<Item = AggregationHitField>) -> Self {
        self.fields = values.into_iter().collect();
        self
    }

    /// Sets the returned fields and returns this value for further mutation.
    pub fn set_fields(
        &mut self,
        values: impl IntoIterator<Item = AggregationHitField>,
    ) -> &mut Self {
        self.fields = values.into_iter().collect();
        self
    }

    /// Returns the returned fields.
    pub fn get_fields(&self) -> &[AggregationHitField] {
        &self.fields
    }

    /// Appends a returned field and returns the updated value.
    pub fn add_field(mut self, value: AggregationHitField) -> Self {
        self.fields.push(value);
        self
    }

    pub(crate) fn from_proto(value: schema::AggHit) -> Result<Self> {
        use schema::agg_hit::Pk;
        let pk = match value.pk {
            Some(Pk::IntPk(value)) => Some(AggregationHitPk::Int(value)),
            Some(Pk::StrPk(value)) => Some(AggregationHitPk::String(value)),
            None => None,
        };
        let fields = value
            .fields
            .into_iter()
            .map(AggregationHitField::from_proto)
            .collect::<Result<_>>()?;
        Ok(Self {
            pk,
            score: value.score,
            fields,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// BucketKeyEntry
///////////////////////////////////////////////////////////////////////////////
/// One entry of a composite grouping key in an aggregation bucket.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct BucketKeyEntry {
    pub(crate) field_id: i64,
    pub(crate) field_name: String,
    pub(crate) value: AggregationBucketValue,
}

impl BucketKeyEntry {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            field_id: 0,
            field_name: String::new(),
            value: AggregationBucketValue::Int(0),
        }
    }

    /// Sets the field id and returns the updated value.
    pub fn field_id(mut self, value: i64) -> Self {
        self.field_id = value;
        self
    }

    /// Sets the field id and returns this value for further mutation.
    pub fn set_field_id(&mut self, value: i64) -> &mut Self {
        self.field_id = value;
        self
    }

    /// Returns the field id.
    pub fn get_field_id(&self) -> i64 {
        self.field_id
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

    /// Returns the field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// Sets the field value and returns the updated value.
    pub fn value(mut self, value: AggregationBucketValue) -> Self {
        self.value = value;
        self
    }

    /// Sets the field value and returns this value for further mutation.
    pub fn set_value(&mut self, value: AggregationBucketValue) -> &mut Self {
        self.value = value;
        self
    }

    /// Returns the field value.
    pub fn get_value(&self) -> &AggregationBucketValue {
        &self.value
    }

    pub(crate) fn from_proto(value: schema::BucketKeyEntry) -> Result<Self> {
        Ok(Self {
            field_id: value.field_id,
            field_name: if value.field_name.is_empty() {
                value.field_id.to_string()
            } else {
                value.field_name
            },
            value: AggregationBucketValue::from_proto(value.value)?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// AggregationBucket
///////////////////////////////////////////////////////////////////////////////
/// One bucket in the hierarchical aggregation result tree.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AggregationBucket {
    pub(crate) key: Vec<BucketKeyEntry>,
    pub(crate) count: i64,
    pub(crate) metrics: HashMap<String, AggregationMetricValue>,
    pub(crate) hits: Vec<AggregationHit>,
    pub(crate) sub_groups: Vec<AggregationBucket>,
}

impl AggregationBucket {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            key: Vec::new(),
            count: 0,
            metrics: HashMap::new(),
            hits: Vec::new(),
            sub_groups: Vec::new(),
        }
    }

    /// Sets the composite grouping key and returns the updated value.
    pub fn key(mut self, values: impl IntoIterator<Item = BucketKeyEntry>) -> Self {
        self.key = values.into_iter().collect();
        self
    }

    /// Sets the composite grouping key and returns this value for further mutation.
    pub fn set_key(&mut self, values: impl IntoIterator<Item = BucketKeyEntry>) -> &mut Self {
        self.key = values.into_iter().collect();
        self
    }

    /// Returns the composite grouping key.
    pub fn get_key(&self) -> &[BucketKeyEntry] {
        &self.key
    }

    /// Appends a grouping-key entry and returns the updated value.
    pub fn add_key(mut self, value: BucketKeyEntry) -> Self {
        self.key.push(value);
        self
    }

    /// Sets the number of documents in this bucket and returns the updated value.
    pub fn count(mut self, value: i64) -> Self {
        self.count = value;
        self
    }

    /// Sets the number of documents in this bucket and returns this value for further mutation.
    pub fn set_count(&mut self, value: i64) -> &mut Self {
        self.count = value;
        self
    }

    /// Returns the number of documents in this bucket.
    pub fn get_count(&self) -> i64 {
        self.count
    }

    /// Sets the metric results keyed by alias and returns the updated value.
    pub fn metrics(
        mut self,
        values: impl IntoIterator<Item = (String, AggregationMetricValue)>,
    ) -> Self {
        self.metrics = values.into_iter().collect();
        self
    }

    /// Sets the metric results keyed by alias and returns this value for further mutation.
    pub fn set_metrics(
        &mut self,
        values: impl IntoIterator<Item = (String, AggregationMetricValue)>,
    ) -> &mut Self {
        self.metrics = values.into_iter().collect();
        self
    }

    /// Returns the metric results keyed by alias.
    pub fn get_metrics(&self) -> &HashMap<String, AggregationMetricValue> {
        &self.metrics
    }

    /// Adds a metric result under `alias` and returns the updated value.
    pub fn add_metric(mut self, alias: impl Into<String>, value: AggregationMetricValue) -> Self {
        self.metrics.insert(alias.into(), value);
        self
    }

    /// Sets the top-hits documents and returns the updated value.
    pub fn hits(mut self, values: impl IntoIterator<Item = AggregationHit>) -> Self {
        self.hits = values.into_iter().collect();
        self
    }

    /// Sets the top-hits documents and returns this value for further mutation.
    pub fn set_hits(&mut self, values: impl IntoIterator<Item = AggregationHit>) -> &mut Self {
        self.hits = values.into_iter().collect();
        self
    }

    /// Returns the top-hits documents.
    pub fn get_hits(&self) -> &[AggregationHit] {
        &self.hits
    }

    /// Appends a top-hits document and returns the updated value.
    pub fn add_hit(mut self, value: AggregationHit) -> Self {
        self.hits.push(value);
        self
    }

    /// Sets the nested child buckets and returns the updated value.
    pub fn sub_groups(mut self, values: impl IntoIterator<Item = AggregationBucket>) -> Self {
        self.sub_groups = values.into_iter().collect();
        self
    }

    /// Sets the nested child buckets and returns this value for further mutation.
    pub fn set_sub_groups(
        &mut self,
        values: impl IntoIterator<Item = AggregationBucket>,
    ) -> &mut Self {
        self.sub_groups = values.into_iter().collect();
        self
    }

    /// Returns the nested child buckets.
    pub fn get_sub_groups(&self) -> &[AggregationBucket] {
        &self.sub_groups
    }

    /// Appends a nested child bucket and returns the updated value.
    pub fn add_sub_group(mut self, value: AggregationBucket) -> Self {
        self.sub_groups.push(value);
        self
    }

    pub(crate) fn from_proto(value: schema::AggBucket) -> Result<Self> {
        let key = value
            .key
            .into_iter()
            .map(BucketKeyEntry::from_proto)
            .collect::<Result<_>>()?;
        let metrics = value
            .metrics
            .into_iter()
            .map(|(alias, metric)| Ok((alias, AggregationMetricValue::from_proto(metric.value)?)))
            .collect::<Result<_>>()?;
        let hits = value
            .hits
            .into_iter()
            .map(AggregationHit::from_proto)
            .collect::<Result<_>>()?;
        let sub_groups = value
            .sub_groups
            .into_iter()
            .map(AggregationBucket::from_proto)
            .collect::<Result<_>>()?;
        Ok(Self {
            key,
            count: value.count,
            metrics,
            hits,
            sub_groups,
        })
    }
}

/// Groups a flat aggregation bucket list into per-query lists using `agg_topks`.
///
/// The server flattens buckets for all query vectors into one `repeated` list and records the
/// number of top-level buckets per query in `agg_topks`. Both lists being empty means no
/// aggregation is present. Otherwise a `MalformedResponse` is returned when `agg_topks` is
/// missing for a multi-query response, when its length does not match `num_queries`, or when the
/// counts are negative, overflow `usize`, or do not add up to the number of parsed buckets. Each
/// query vector receives its own group, which may be empty when a query produced no buckets.
pub(crate) fn group_aggregation_buckets(
    buckets: Vec<schema::AggBucket>,
    topks: Vec<i64>,
    num_queries: i64,
) -> Result<Vec<Vec<AggregationBucket>>> {
    if buckets.is_empty() && topks.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: Vec<AggregationBucket> = buckets
        .into_iter()
        .map(AggregationBucket::from_proto)
        .collect::<Result<_>>()?;
    if topks.is_empty() {
        if num_queries > 1 {
            return Err(Error::MalformedResponse(
                "aggregation buckets returned without agg_topks for a multi-query search".into(),
            ));
        }
        return Ok(vec![parsed]);
    }
    let num_queries = usize::try_from(num_queries).map_err(|_| {
        Error::MalformedResponse(format!(
            "aggregation num_queries value {num_queries} does not fit usize"
        ))
    })?;
    if topks.len() != num_queries {
        return Err(Error::MalformedResponse(format!(
            "aggregation agg_topks length {} does not match num_queries {num_queries}",
            topks.len()
        )));
    }
    let total = parsed.len();
    let mut remaining = parsed;
    let mut groups = Vec::with_capacity(topks.len());
    for topk in topks {
        if topk < 0 {
            return Err(Error::MalformedResponse(format!(
                "aggregation response contains a negative agg_topks value {topk}"
            )));
        }
        let size = usize::try_from(topk).map_err(|_| {
            Error::MalformedResponse(format!(
                "aggregation agg_topks value {topk} does not fit usize"
            ))
        })?;
        if size > remaining.len() {
            return Err(Error::MalformedResponse(format!(
                "aggregation bucket count mismatch: agg_topks sum exceeds {total} parsed buckets"
            )));
        }
        groups.push(remaining.drain(..size).collect());
    }
    if !remaining.is_empty() {
        return Err(Error::MalformedResponse(format!(
            "aggregation bucket count mismatch: agg_topks sum={}, parsed={total}",
            total - remaining.len()
        )));
    }
    Ok(groups)
}

///////////////////////////////////////////////////////////////////////////////
// QueryCursor
///////////////////////////////////////////////////////////////////////////////
/// Resumable cursor for a query iterator, mirroring the Java/Python SDKs' `QueryIteratorCursor`.
///
/// A cursor bundles the MVCC snapshot timestamp (`session_ts`) from the original iterator with
/// the primary-key position after the last page read. Restoring both on resume pins the new
/// iterator to the same data view as the original, so concurrent inserts/deletes between capture
/// and resume cannot cause missed or duplicated rows. For `element_filter` iterators the cursor
/// additionally carries `last_element_offset`, the element position within the last primary key
/// from which the server resumes, matching pymilvus.
///
/// Capture a cursor from [`QueryIterator::cursor`](crate::v2::QueryIterator::cursor) after
/// reading a few pages, then pass it to
/// [`QueryIteratorRequestBuilder::cursor`](crate::v2::request::dql::QueryIteratorRequestBuilder::cursor)
/// to resume pagination in a brand-new iterator.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct QueryCursor {
    pub(crate) session_ts: u64,
    pub(crate) pk: QueryCursorPk,
    pub(crate) last_element_offset: Option<i64>,
}

/// Primary-key position of a [`QueryCursor`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryCursorPk {
    /// Int64 primary-key position.
    Int64(i64),
    /// VarChar primary-key position.
    VarChar(String),
}

impl QueryCursor {
    /// Creates a value initialized with its SDK defaults.
    ///
    /// The default cursor carries an `Int64` primary-key position of `0` and no element offset.
    /// Use [`Self::int64`]/[`Self::varchar`] for the primary construction path, or configure this
    /// default with the fluent and in-place setters.
    ///
    /// # Warning
    ///
    /// The default cursor is **not a valid resume point**: passing it to
    /// [`QueryIteratorRequestBuilder::cursor`](crate::v2::request::dql::QueryIteratorRequestBuilder::cursor)
    /// resumes from strictly after primary key `0`, silently skipping any rows with `pk <= 0`.
    /// Resume only with a cursor captured from
    /// [`QueryIterator::cursor`](crate::v2::QueryIterator::cursor) or constructed with an explicit
    /// primary-key position via [`Self::int64`]/[`Self::varchar`].
    pub fn new() -> Self {
        Self {
            session_ts: 0,
            pk: QueryCursorPk::Int64(0),
            last_element_offset: None,
        }
    }

    /// Creates a cursor with an Int64 primary-key position.
    ///
    /// `session_ts` pins the MVCC snapshot the resumed iterator reads from. Pass `0` when building
    /// a cursor from scratch: the SDK then runs the server-side timestamp probe and derives the
    /// guarantee timestamp, exactly as a fresh iterator does. Pass the timestamp captured from
    /// [`QueryIterator::cursor`](crate::v2::QueryIterator::cursor) (see [`QueryCursor::get_session_ts`])
    /// to pin the resumed iterator to the original data view and skip the probe.
    pub fn int64(session_ts: u64, pk: i64) -> Self {
        Self {
            session_ts,
            pk: QueryCursorPk::Int64(pk),
            last_element_offset: None,
        }
    }

    /// Creates a cursor with a VarChar primary-key position.
    ///
    /// `session_ts` pins the MVCC snapshot the resumed iterator reads from. Pass `0` when building
    /// a cursor from scratch: the SDK then runs the server-side timestamp probe and derives the
    /// guarantee timestamp, exactly as a fresh iterator does. Pass the timestamp captured from
    /// [`QueryIterator::cursor`](crate::v2::QueryIterator::cursor) (see [`QueryCursor::get_session_ts`])
    /// to pin the resumed iterator to the original data view and skip the probe.
    pub fn varchar(session_ts: u64, pk: impl Into<String>) -> Self {
        Self {
            session_ts,
            pk: QueryCursorPk::VarChar(pk.into()),
            last_element_offset: None,
        }
    }

    /// Sets the MVCC snapshot timestamp and returns the updated value.
    pub fn session_ts(mut self, value: u64) -> Self {
        self.session_ts = value;
        self
    }

    /// Sets the MVCC snapshot timestamp and returns this value for further mutation.
    pub fn set_session_ts(&mut self, value: u64) -> &mut Self {
        self.session_ts = value;
        self
    }

    /// Returns the MVCC snapshot timestamp pinned from the original iterator session.
    pub fn get_session_ts(&self) -> u64 {
        self.session_ts
    }

    /// Sets the primary-key position and returns the updated value.
    pub fn pk(mut self, value: QueryCursorPk) -> Self {
        self.pk = value;
        self
    }

    /// Sets the primary-key position and returns this value for further mutation.
    pub fn set_pk(&mut self, value: QueryCursorPk) -> &mut Self {
        self.pk = value;
        self
    }

    /// Returns the primary-key position after the last page read.
    pub fn get_pk(&self) -> &QueryCursorPk {
        &self.pk
    }

    /// Sets the element position within the last primary key and returns the updated value.
    ///
    /// Only meaningful for `element_filter` iterators; ordinary iterators leave it `None`.
    pub fn last_element_offset(mut self, value: i64) -> Self {
        self.last_element_offset = Some(value);
        self
    }

    /// Sets the element position within the last primary key and returns this value for further
    /// mutation.
    ///
    /// Only meaningful for `element_filter` iterators; ordinary iterators leave it `None`.
    pub fn set_last_element_offset(&mut self, value: i64) -> &mut Self {
        self.last_element_offset = Some(value);
        self
    }

    /// Returns the element position within the last primary key, if this is an `element_filter`
    /// cursor.
    pub fn get_last_element_offset(&self) -> Option<i64> {
        self.last_element_offset
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchResults
///////////////////////////////////////////////////////////////////////////////
/// Search results for all query vectors, with one [`SingleResult`] per query vector.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SearchResults {
    pub(crate) results: Vec<SingleResult>,
    pub(crate) recalls: Vec<f32>,
    pub(crate) agg_buckets: Vec<Vec<AggregationBucket>>,
}

impl SearchResults {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            recalls: Vec::new(),
            agg_buckets: Vec::new(),
        }
    }

    /// Sets the results and returns the updated value.
    pub fn results(mut self, value: Vec<SingleResult>) -> Self {
        self.results = value;
        self
    }

    /// Sets the results and returns this value for further mutation.
    pub fn set_results(&mut self, value: Vec<SingleResult>) -> &mut Self {
        self.results = value;
        self
    }

    /// Returns the configured results.
    pub fn get_results(&self) -> &[SingleResult] {
        &self.results
    }

    /// Iterates over the result associated with each query vector.
    pub fn iter(&self) -> std::slice::Iter<'_, SingleResult> {
        self.results.iter()
    }

    /// Sets the recalls and returns the updated value.
    pub fn recalls(mut self, value: Vec<f32>) -> Self {
        self.recalls = value;
        self
    }

    /// Sets the recalls and returns this value for further mutation.
    pub fn set_recalls(&mut self, value: Vec<f32>) -> &mut Self {
        self.recalls = value;
        self
    }

    /// Returns the configured recalls.
    pub fn get_recalls(&self) -> &[f32] {
        &self.recalls
    }

    /// Adds one add result to the existing values.
    pub fn add_result(mut self, value: SingleResult) -> Self {
        self.results.push(value);
        self
    }

    /// Returns the len.
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Returns whether empty.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Adds one add recall to the existing values.
    pub fn add_recall(mut self, value: f32) -> Self {
        self.recalls.push(value);
        self
    }

    /// Sets the hierarchical aggregation buckets, grouped per query vector, and returns the
    /// updated value.
    pub fn agg_buckets(mut self, value: Vec<Vec<AggregationBucket>>) -> Self {
        self.agg_buckets = value;
        self
    }

    /// Sets the hierarchical aggregation buckets, grouped per query vector, and returns this value
    /// for further mutation.
    pub fn set_agg_buckets(&mut self, value: Vec<Vec<AggregationBucket>>) -> &mut Self {
        self.agg_buckets = value;
        self
    }

    /// Returns the hierarchical aggregation buckets, one inner list per query vector, when a
    /// search aggregation was requested.
    pub fn get_agg_buckets(&self) -> &[Vec<AggregationBucket>] {
        &self.agg_buckets
    }

    /// Appends one query vector's aggregation buckets and returns the updated value.
    pub fn add_agg_bucket(mut self, value: Vec<AggregationBucket>) -> Self {
        self.agg_buckets.push(value);
        self
    }
}

impl<'a> IntoIterator for &'a SearchResults {
    type Item = &'a SingleResult;
    type IntoIter = std::slice::Iter<'a, SingleResult>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn output_row_count(fields: &[FieldData]) -> Result<usize> {
    let row_count = fields.first().map_or(0, FieldData::len);
    if fields.iter().any(|field| field.len() != row_count) {
        return Err(Error::MalformedResponse(
            "query result fields contain unequal row counts".into(),
        ));
    }
    Ok(row_count)
}

fn output_row(
    fields: &[FieldData],
    output_field_names: &[String],
    index: usize,
) -> Result<EntityRow> {
    const DYNAMIC_FIELD: &str = "$meta";

    let mut row = EntityRow::new();
    for field in fields {
        let value = field.value_at(index)?;
        if field.name() == DYNAMIC_FIELD {
            let serde_json::Value::Object(values) = value else {
                return Err(Error::MalformedResponse(
                    "dynamic output field is not a JSON object".into(),
                ));
            };
            if output_field_names.iter().any(|name| name == DYNAMIC_FIELD) {
                row.extend(values);
            } else {
                row.extend(
                    values
                        .into_iter()
                        .filter(|(name, _)| output_field_names.iter().any(|item| item == name)),
                );
            }
        } else {
            row.insert(field.name().to_owned(), value);
        }
    }
    Ok(row)
}

fn row_index_error(index: usize, row_count: usize) -> Error {
    Error::validation(
        "index".into(),
        format!("row index {index} is out of bounds for {row_count} rows"),
    )
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod function_tests {
    use super::{
        BoostRerank, DecayRerank, Function, FunctionType, ModelRerank, RRFRerank, WeightedRerank,
    };

    #[test]
    fn concrete_rerankers_convert_to_common_functions() {
        let rrf: Function = RRFRerank::new().k(80).into();
        assert_eq!(rrf.get_function_type().to_owned(), FunctionType::Rerank);
        assert_eq!(rrf.get_params()["strategy"], "rrf");
        assert_eq!(rrf.get_params()["params"], r#"{"k":80}"#);

        let weighted: Function = WeightedRerank::new().weights(vec![0.25, 0.75]).into();
        assert_eq!(weighted.get_params()["strategy"], "weighted");
        assert_eq!(
            weighted.get_params()["params"],
            r#"{"weights":[0.25,0.75]}"#
        );

        let boost: Function = BoostRerank::new()
            .name("boost")
            .filter("category == 1")
            .weight(2.0)
            .random_score_field("id")
            .random_score_seed(42)
            .into();
        assert_eq!(boost.get_params()["reranker"], "boost");
        assert_eq!(boost.get_params()["weight"], "2");
        let random_score: serde_json::Value =
            serde_json::from_str(&boost.get_params()["random_score"]).unwrap();
        assert_eq!(random_score["field"], "id");
        assert_eq!(random_score["seed"], 42);

        let decay: Function = DecayRerank::new()
            .name("freshness")
            .decay_function("gauss")
            .origin(100)
            .offset(5)
            .scale(20)
            .decay(0.5)
            .into();
        assert_eq!(decay.get_params()["reranker"], "decay");
        assert_eq!(decay.get_params()["function"], "gauss");

        let model: Function = ModelRerank::new()
            .name("cross_encoder")
            .provider("tei")
            .queries(["milvus"])
            .endpoint("http://localhost:8080")
            .max_client_batch_size(16)
            .into();
        assert_eq!(model.get_params()["reranker"], "model");
        assert_eq!(model.get_params()["queries"], r#"["milvus"]"#);
        assert_eq!(model.get_params()["max_client_batch_size"], "16");
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod result_row_tests {
    use super::{QueryResults, ResultValue, SearchResults, SingleResult};
    use crate::v2::error::Error;
    use crate::v2::types::{FieldData, Ids, SparseVector};
    use serde_json::json;

    #[test]
    fn query_results_materialize_one_or_all_rows() {
        let nested = |name: &str| {
            json!({ "name": name })
                .as_object()
                .expect("nested struct value")
                .clone()
        };
        let results = QueryResults::new()
            .output_fields(vec![
                FieldData::Int64 {
                    name: "id".into(),
                    values: vec![1, 2],
                },
                FieldData::nullable(
                    FieldData::VarChar {
                        name: "nullable_text".into(),
                        values: vec!["first".into()],
                    },
                    vec![true, false],
                )
                .unwrap(),
                FieldData::ArrayInt16 {
                    name: "numbers".into(),
                    values: vec![vec![1, 2], vec![3]],
                },
                FieldData::Struct {
                    name: "items".into(),
                    values: vec![vec![nested("a")], vec![nested("b")]],
                },
                FieldData::FloatVector {
                    name: "embedding".into(),
                    values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
                },
                FieldData::Json {
                    name: "$meta".into(),
                    values: vec![
                        json!({ "visible": "first", "hidden": 10 }),
                        json!({ "visible": "second", "hidden": 20 }),
                    ],
                },
            ])
            .output_field_names([
                "id",
                "nullable_text",
                "numbers",
                "items",
                "embedding",
                "visible",
            ]);

        let first = results.get_output_row(0).unwrap();
        assert_eq!(first["id"], 1);
        assert_eq!(first["nullable_text"], "first");
        assert_eq!(first["numbers"], json!([1, 2]));
        assert_eq!(first["items"], json!([{ "name": "a" }]));
        let embedding = first["embedding"].as_array().unwrap();
        assert!((embedding[0].as_f64().unwrap() - 0.1).abs() < 1e-6);
        assert!((embedding[1].as_f64().unwrap() - 0.2).abs() < 1e-6);
        assert_eq!(first["visible"], "first");
        assert!(!first.contains_key("hidden"));
        assert!(!first.contains_key("$meta"));

        let rows = results.get_output_rows().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1]["nullable_text"], serde_json::Value::Null);
        assert_eq!(rows[1]["visible"], "second");
        assert!(results.get_output_row(2).is_err());

        let borrowed = results.rows().unwrap().collect::<Vec<_>>();
        assert_eq!(borrowed.len(), 2);
        assert_eq!(borrowed[0].index(), 0);
        assert_eq!(borrowed[0].get_i64("id").unwrap(), 1);
        assert_eq!(borrowed[0].get_str("nullable_text").unwrap(), "first");
        assert_eq!(borrowed[0].get_str("visible").unwrap(), "first");
        assert_eq!(
            borrowed[0].get_float_vector("embedding").unwrap(),
            &[0.1, 0.2]
        );
        assert!(borrowed[1].get_str("nullable_text").is_err());
    }

    #[test]
    fn single_result_rows_include_primary_key_and_score() {
        let result = SingleResult::new()
            .ids(Ids::Int64(vec![10, 20]))
            .scores(vec![0.9, 0.8])
            .output_fields(vec![FieldData::VarChar {
                name: "title".into(),
                values: vec!["first".into(), "second".into()],
            }])
            .output_field_names(["title"])
            .primary_field_name("id")
            .score_field_name("score");

        let rows = result.get_output_rows().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["id"], 10);
        assert_eq!(rows[0]["title"], "first");
        assert!((rows[0]["score"].as_f64().unwrap() - 0.9).abs() < 1e-6);
        assert_eq!(rows[1]["id"], 20);
        assert_eq!(rows[1]["title"], "second");
        assert!((rows[1]["score"].as_f64().unwrap() - 0.8).abs() < 1e-6);
        assert_eq!(result.get_output_row(1).unwrap().to_owned(), rows[1]);
        assert!(result.get_output_row(2).is_err());
    }

    #[test]
    fn borrowed_search_rows_iterate_and_read_typed_values() {
        let result = SingleResult::new()
            .ids(Ids::Int64(vec![10, 20]))
            .scores(vec![0.9, 0.8])
            .output_fields(vec![
                FieldData::VarChar {
                    name: "title".into(),
                    values: vec!["first".into(), "second".into()],
                },
                FieldData::Int32 {
                    name: "count".into(),
                    values: vec![3, 4],
                },
                FieldData::FloatVector {
                    name: "embedding".into(),
                    values: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
                },
                FieldData::Json {
                    name: "payload".into(),
                    values: vec![json!({"active": true}), json!({"active": false})],
                },
                FieldData::Json {
                    name: "$meta".into(),
                    values: vec![json!({"tag": "a"}), json!({"tag": "b"})],
                },
            ])
            .output_field_names(["title", "count", "embedding", "payload", "tag"])
            .primary_field_name("id")
            .score_field_name("score");

        let mut rows = result.rows().unwrap();
        assert_eq!(rows.len(), 2);
        let first = rows.next().unwrap();
        assert_eq!(first.index(), 0);
        assert_eq!(first.get_i64("id").unwrap(), 10);
        assert_eq!(first.get_i64("count").unwrap(), 3);
        assert!((first.get_f32("score").unwrap() - 0.9).abs() < f32::EPSILON);
        assert_eq!(first.get_str("title").unwrap(), "first");
        assert_eq!(first.get_str("tag").unwrap(), "a");
        assert_eq!(first.get_json("payload").unwrap(), &json!({"active": true}));
        assert_eq!(first.get_float_vector("embedding").unwrap(), &[0.1, 0.2]);
        assert!(first.get_i64("title").is_err());
        assert!(first.get_str("missing").is_err());

        let second = rows.next().unwrap();
        assert_eq!(second.index(), 1);
        assert_eq!(second.get_i64("id").unwrap(), 20);
        assert_eq!(second.get_str("tag").unwrap(), "b");
        assert!(rows.next().is_none());

        let results = SearchResults::new().results(vec![result]);
        assert_eq!(results.iter().count(), 1);
        assert_eq!((&results).into_iter().count(), 1);
    }

    #[test]
    fn borrowed_query_rows_read_every_field_data_type() {
        let struct_value = json!({"label": "nested"})
            .as_object()
            .expect("struct value")
            .clone();
        let sparse = SparseVector::from([(1, 0.5), (7, 1.25)]);
        let results = QueryResults::new().output_fields(vec![
            FieldData::boolean("bool", vec![true]),
            FieldData::int8("i8", vec![-8]),
            FieldData::int16("i16", vec![-16]),
            FieldData::int32("i32", vec![-32]),
            FieldData::int64("i64", vec![-64]),
            FieldData::float("f32", vec![1.25]),
            FieldData::double("f64", vec![2.5]),
            FieldData::varchar("text", vec!["value".into()]),
            FieldData::json("json", vec![json!({"key": "value"})]),
            FieldData::geometry("geometry", vec!["POINT (1 2)".into()]),
            FieldData::timestamptz("time", vec!["2026-07-31T12:00:00Z".into()]),
            FieldData::array_bool("bools", vec![vec![true, false]]),
            FieldData::array_int8("i8s", vec![vec![-1, 2]]),
            FieldData::array_int16("i16s", vec![vec![-2, 3]]),
            FieldData::array_int32("i32s", vec![vec![-3, 4]]),
            FieldData::array_int64("i64s", vec![vec![-4, 5]]),
            FieldData::array_float("f32s", vec![vec![0.5, 1.5]]),
            FieldData::array_double("f64s", vec![vec![2.5, 3.5]]),
            FieldData::array_varchar("texts", vec![vec!["a".into(), "b".into()]]),
            FieldData::struct_field("structs", vec![vec![struct_value.clone()]]),
            FieldData::float_vector("float_vector", vec![vec![0.1, 0.2]]),
            FieldData::binary_vector("binary_vector", vec![vec![0b1010_0101]]),
            FieldData::float16_vector("float16_vector", vec![vec![0x3c00, 0x4000]]),
            FieldData::bfloat16_vector("bfloat16_vector", vec![vec![0x3f80, 0x4000]]),
            FieldData::sparse_float_vector("sparse_vector", vec![sparse.clone()]),
            FieldData::int8_vector("int8_vector", vec![vec![-1, 2]]),
        ]);

        let row = results.rows().unwrap().next().unwrap();
        assert!(row.get_bool("bool").unwrap());
        assert_eq!(row.get_i8("i8").unwrap(), -8);
        assert_eq!(row.get_i16("i16").unwrap(), -16);
        assert_eq!(row.get_i32("i32").unwrap(), -32);
        assert_eq!(row.get_i64("i64").unwrap(), -64);
        assert_eq!(row.get_i64("i8").unwrap(), -8);
        assert_eq!(row.get_f32("f32").unwrap(), 1.25);
        assert_eq!(row.get_f64("f32").unwrap(), 1.25);
        assert_eq!(row.get_f64("f64").unwrap(), 2.5);
        assert_eq!(row.get_str("text").unwrap(), "value");
        assert_eq!(row.get_json("json").unwrap(), &json!({"key": "value"}));
        assert_eq!(row.get_geometry("geometry").unwrap(), "POINT (1 2)");
        assert_eq!(row.get_timestamptz("time").unwrap(), "2026-07-31T12:00:00Z");
        assert_eq!(row.get_array_bool("bools").unwrap(), &[true, false]);
        assert_eq!(row.get_array_i8("i8s").unwrap(), &[-1, 2]);
        assert_eq!(row.get_array_i16("i16s").unwrap(), &[-2, 3]);
        assert_eq!(row.get_array_i32("i32s").unwrap(), &[-3, 4]);
        assert_eq!(row.get_array_i64("i64s").unwrap(), &[-4, 5]);
        assert_eq!(row.get_array_f32("f32s").unwrap(), &[0.5, 1.5]);
        assert_eq!(row.get_array_f64("f64s").unwrap(), &[2.5, 3.5]);
        assert_eq!(row.get_array_varchar("texts").unwrap(), &["a", "b"]);
        assert_eq!(
            row.get_struct("structs").unwrap(),
            std::slice::from_ref(&struct_value)
        );
        assert_eq!(row.get_float_vector("float_vector").unwrap(), &[0.1, 0.2]);
        assert_eq!(
            row.get_binary_vector("binary_vector").unwrap(),
            &[0b1010_0101]
        );
        assert_eq!(
            row.get_float16_vector("float16_vector").unwrap(),
            &[0x3c00, 0x4000]
        );
        assert_eq!(
            row.get_bfloat16_vector("bfloat16_vector").unwrap(),
            &[0x3f80, 0x4000]
        );
        assert_eq!(
            row.get_sparse_float_vector("sparse_vector").unwrap(),
            &sparse
        );
        assert_eq!(row.get_int8_vector("int8_vector").unwrap(), &[-1, 2]);
        assert!(!row.is_null("bool").unwrap());
        assert!(row.get_float_vector("binary_vector").is_err());

        assert!(matches!(row.get("bool").unwrap(), ResultValue::Bool(true)));
        assert!(matches!(row.get("i8").unwrap(), ResultValue::Int8(-8)));
        assert!(matches!(row.get("i16").unwrap(), ResultValue::Int16(-16)));
        assert!(matches!(row.get("i32").unwrap(), ResultValue::Int32(-32)));
        assert!(matches!(row.get("i64").unwrap(), ResultValue::Int64(-64)));
        assert!(matches!(row.get("f32").unwrap(), ResultValue::Float(1.25)));
        assert!(matches!(row.get("f64").unwrap(), ResultValue::Double(2.5)));
        assert!(matches!(
            row.get("text").unwrap(),
            ResultValue::String("value")
        ));
        assert!(
            matches!(row.get("json").unwrap(), ResultValue::Json(value) if value == &json!({"key": "value"}))
        );
        assert!(matches!(
            row.get("geometry").unwrap(),
            ResultValue::Geometry("POINT (1 2)")
        ));
        assert!(matches!(
            row.get("time").unwrap(),
            ResultValue::Timestamptz("2026-07-31T12:00:00Z")
        ));
        assert!(
            matches!(row.get("bools").unwrap(), ResultValue::ArrayBool(value) if value == [true, false])
        );
        assert!(
            matches!(row.get("i8s").unwrap(), ResultValue::ArrayInt8(value) if value == [-1, 2])
        );
        assert!(
            matches!(row.get("i16s").unwrap(), ResultValue::ArrayInt16(value) if value == [-2, 3])
        );
        assert!(
            matches!(row.get("i32s").unwrap(), ResultValue::ArrayInt32(value) if value == [-3, 4])
        );
        assert!(
            matches!(row.get("i64s").unwrap(), ResultValue::ArrayInt64(value) if value == [-4, 5])
        );
        assert!(
            matches!(row.get("f32s").unwrap(), ResultValue::ArrayFloat(value) if value == [0.5, 1.5])
        );
        assert!(
            matches!(row.get("f64s").unwrap(), ResultValue::ArrayDouble(value) if value == [2.5, 3.5])
        );
        assert!(
            matches!(row.get("texts").unwrap(), ResultValue::ArrayVarChar(value) if value == ["a", "b"])
        );
        assert!(
            matches!(row.get("structs").unwrap(), ResultValue::Struct(value) if value == std::slice::from_ref(&struct_value))
        );
        assert!(
            matches!(row.get("float_vector").unwrap(), ResultValue::FloatVector(value) if value == [0.1, 0.2])
        );
        assert!(
            matches!(row.get("binary_vector").unwrap(), ResultValue::BinaryVector(value) if value == [0b1010_0101])
        );
        assert!(
            matches!(row.get("float16_vector").unwrap(), ResultValue::Float16Vector(value) if value == [0x3c00, 0x4000])
        );
        assert!(
            matches!(row.get("bfloat16_vector").unwrap(), ResultValue::BFloat16Vector(value) if value == [0x3f80, 0x4000])
        );
        assert!(
            matches!(row.get("sparse_vector").unwrap(), ResultValue::SparseFloatVector(value) if value == &sparse)
        );
        assert!(
            matches!(row.get("int8_vector").unwrap(), ResultValue::Int8Vector(value) if value == [-1, 2])
        );
    }

    #[test]
    fn borrowed_search_rows_report_null_fields() {
        let result = SingleResult::new()
            .ids(Ids::Int64(vec![1, 2]))
            .scores(vec![1.0, 0.5])
            .output_fields(vec![FieldData::nullable(
                FieldData::VarChar {
                    name: "title".into(),
                    values: vec!["present".into()],
                },
                vec![true, false],
            )
            .unwrap()])
            .output_field_names(["title"])
            .primary_field_name("id")
            .score_field_name("score");

        let rows = result.rows().unwrap().collect::<Vec<_>>();
        assert_eq!(rows[0].get_str("title").unwrap(), "present");
        assert!(!rows[0].is_null("title").unwrap());
        assert!(rows[1].is_null("title").unwrap());
        assert!(matches!(rows[1].get("title").unwrap(), ResultValue::Null));
        assert!(matches!(
            rows[1].get_str("title"),
            Err(Error::Validation(error))
                if error.parameter() == "title" && error.reason().contains("null")
        ));
    }

    #[test]
    fn row_materialization_rejects_unequal_field_lengths() {
        let results = QueryResults::new().output_fields(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::VarChar {
                name: "title".into(),
                values: vec!["only one".into()],
            },
        ]);

        assert!(results.get_output_rows().is_err());
        assert!(results.rows().is_err());
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn function_score_default_values() {
        let value = FunctionScore::new();
        let expected_functions: Vec<Function> = Default::default();
        let expected_params: HashMap<String, serde_json::Value> = Default::default();

        assert_eq!(value.get_functions().to_owned(), expected_functions);
        assert_eq!(value.get_params().to_owned(), expected_params);
        assert_eq!(FunctionScore::new(), value);
    }

    #[test]
    fn function_score_populated_values() {
        let functions = vec![Function::new()
            .name("function")
            .function_type(crate::v2::FunctionType::Bm25)];
        let params = HashMap::from([("key-value".to_owned(), serde_json::json!({"key": "value"}))]);
        let value = FunctionScore::new()
            .functions(functions.clone())
            .params(params.clone());

        assert_eq!(value.get_functions().to_owned(), functions);
        assert_eq!(value.get_params().to_owned(), params);
    }

    #[test]
    fn function_score_proto_params_unwrap_strings_and_serialize_other_json_values() {
        let proto = FunctionScore::new()
            .params(HashMap::from([
                ("boost_mode".to_owned(), serde_json::json!("multiply")),
                ("limit".to_owned(), serde_json::json!(10)),
                ("options".to_owned(), serde_json::json!({ "enabled": true })),
            ]))
            .into_proto();
        let params = proto
            .params
            .into_iter()
            .map(|param| (param.key, param.value))
            .collect::<HashMap<_, _>>();

        assert_eq!(params["boost_mode"], "multiply");
        assert_eq!(params["limit"], "10");
        assert_eq!(params["options"], r#"{"enabled":true}"#);
    }

    #[test]
    fn highlighter_constructor_values() {
        let value = Highlighter::new().highlight_type(HighlightType::Lexical);
        let expected_highlight_type = HighlightType::Lexical;
        let expected_params: HashMap<String, String> = Default::default();

        assert_eq!(
            value.get_highlight_type().to_owned(),
            expected_highlight_type
        );
        assert_eq!(value.get_params().to_owned(), expected_params);
        assert_eq!(
            Highlighter::new().highlight_type(HighlightType::Lexical),
            value
        );
    }

    #[test]
    fn highlighter_populated_values() {
        let highlight_type = HighlightType::Lexical;
        let params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = Highlighter::new()
            .highlight_type(highlight_type)
            .params(params.clone());

        assert_eq!(value.get_highlight_type().to_owned(), highlight_type);
        assert_eq!(value.get_params().to_owned(), params);
    }

    #[test]
    fn highlight_query_constructor_values() {
        let value = HighlightQuery::new()
            .query_type("query")
            .field("field")
            .text("text");
        let expected_query_type = "query";
        let expected_field = "field";
        let expected_text = "text";

        assert_eq!(value.get_query_type(), expected_query_type);
        assert_eq!(value.get_field(), expected_field);
        assert_eq!(value.get_text(), expected_text);
    }

    #[test]
    fn highlight_query_populated_values() {
        let query_type = "query_type-value".to_owned();
        let field = "field-value".to_owned();
        let text = "text-value".to_owned();
        let value = HighlightQuery::new()
            .query_type(query_type.clone())
            .field(field.clone())
            .text(text.clone());

        assert_eq!(value.get_query_type().to_owned(), query_type);
        assert_eq!(value.get_field().to_owned(), field);
        assert_eq!(value.get_text().to_owned(), text);
        assert_eq!(
            HighlightQuery::new()
                .query_type(query_type)
                .field(field)
                .text(text),
            value
        );
    }

    #[test]
    fn lexical_highlighter_default_values() {
        let value = LexicalHighlighter::new();
        let expected_highlight_queries: Vec<HighlightQuery> = Default::default();
        let expected_highlight_search_text: bool = false;
        let expected_pre_tags: Vec<String> = Default::default();
        let expected_post_tags: Vec<String> = Default::default();
        let expected_fragment_offset: Option<i64> = None;
        let expected_fragment_size: Option<i64> = None;
        let expected_num_of_fragments: Option<i64> = None;

        assert_eq!(
            value.get_highlight_queries().to_owned(),
            expected_highlight_queries
        );
        assert_eq!(
            value.get_highlight_search_text(),
            expected_highlight_search_text
        );
        assert_eq!(value.get_pre_tags().to_owned(), expected_pre_tags);
        assert_eq!(value.get_post_tags().to_owned(), expected_post_tags);
        assert_eq!(
            value.get_fragment_offset().to_owned(),
            expected_fragment_offset
        );
        assert_eq!(value.get_fragment_size().to_owned(), expected_fragment_size);
        assert_eq!(
            value.get_num_of_fragments().to_owned(),
            expected_num_of_fragments
        );
        assert_eq!(LexicalHighlighter::new(), value);
    }

    #[test]
    fn lexical_highlighter_populated_values() {
        let highlight_queries = vec![HighlightQuery::new()
            .query_type("query")
            .field("field")
            .text("text")];
        let highlight_search_text = true;
        let pre_tags = vec!["pre_tags-value".to_owned()];
        let post_tags = vec!["post_tags-value".to_owned()];
        let fragment_offset = 7;
        let fragment_size = 7;
        let num_of_fragments = 7;
        let value = LexicalHighlighter::new()
            .highlight_queries(highlight_queries.clone())
            .highlight_search_text(highlight_search_text.clone())
            .pre_tags(pre_tags.clone())
            .post_tags(post_tags.clone())
            .fragment_offset(fragment_offset.clone())
            .fragment_size(fragment_size.clone())
            .num_of_fragments(num_of_fragments.clone());

        assert_eq!(value.get_highlight_queries().to_owned(), highlight_queries);
        assert_eq!(
            value.get_highlight_search_text().to_owned(),
            highlight_search_text
        );
        assert_eq!(value.get_pre_tags().to_owned(), pre_tags);
        assert_eq!(value.get_post_tags().to_owned(), post_tags);
        assert_eq!(
            value.get_fragment_offset().to_owned(),
            Some(fragment_offset)
        );
        assert_eq!(value.get_fragment_size().to_owned(), Some(fragment_size));
        assert_eq!(
            value.get_num_of_fragments().to_owned(),
            Some(num_of_fragments)
        );
    }

    #[test]
    fn semantic_highlighter_default_values() {
        let value = SemanticHighlighter::new();
        let expected_queries: Vec<String> = Default::default();
        let expected_input_fields: Vec<String> = Default::default();
        let expected_pre_tags: Vec<String> = Default::default();
        let expected_post_tags: Vec<String> = Default::default();
        let expected_threshold: Option<f32> = None;
        let expected_highlight_only: bool = false;
        let expected_model_deployment_id: String = String::new();
        let expected_max_client_batch_size: Option<i64> = None;

        assert_eq!(value.get_queries().to_owned(), expected_queries);
        assert_eq!(value.get_input_fields().to_owned(), expected_input_fields);
        assert_eq!(value.get_pre_tags().to_owned(), expected_pre_tags);
        assert_eq!(value.get_post_tags().to_owned(), expected_post_tags);
        assert_eq!(value.get_threshold().to_owned(), expected_threshold);
        assert_eq!(
            value.get_highlight_only().to_owned(),
            expected_highlight_only
        );
        assert_eq!(
            value.get_model_deployment_id(),
            &expected_model_deployment_id
        );
        assert_eq!(
            value.get_max_client_batch_size(),
            expected_max_client_batch_size
        );
        assert_eq!(SemanticHighlighter::new(), value);
    }

    #[test]
    fn semantic_highlighter_populated_values() {
        let queries = vec!["queries-value".to_owned()];
        let input_fields = vec!["input_fields-value".to_owned()];
        let pre_tags = vec!["pre_tags-value".to_owned()];
        let post_tags = vec!["post_tags-value".to_owned()];
        let threshold = 1.5;
        let highlight_only = true;
        let model_deployment_id = "model_deployment_id-value".to_owned();
        let max_client_batch_size = 7;
        let value = SemanticHighlighter::new()
            .queries(queries.clone())
            .input_fields(input_fields.clone())
            .pre_tags(pre_tags.clone())
            .post_tags(post_tags.clone())
            .threshold(threshold.clone())
            .highlight_only(highlight_only.clone())
            .model_deployment_id(model_deployment_id.clone())
            .max_client_batch_size(max_client_batch_size.clone());

        assert_eq!(value.get_queries().to_owned(), queries);
        assert_eq!(value.get_input_fields().to_owned(), input_fields);
        assert_eq!(value.get_pre_tags().to_owned(), pre_tags);
        assert_eq!(value.get_post_tags().to_owned(), post_tags);
        assert_eq!(value.get_threshold().to_owned(), Some(threshold));
        assert_eq!(value.get_highlight_only().to_owned(), highlight_only);
        assert_eq!(
            value.get_model_deployment_id().to_owned(),
            model_deployment_id
        );
        assert_eq!(
            value.get_max_client_batch_size(),
            Some(max_client_batch_size)
        );
    }

    #[test]
    fn query_results_default_values() {
        let value = QueryResults::new();
        let expected_output_fields: Vec<FieldData> = Default::default();
        let expected_output_field_names: Vec<String> = Default::default();

        assert_eq!(value.get_output_fields().to_owned(), expected_output_fields);
        assert_eq!(
            value.get_output_field_names().to_owned(),
            expected_output_field_names
        );
    }

    #[test]
    fn query_results_populated_values() {
        let output_fields = vec![FieldData::VarChar {
            name: "field".to_owned(),
            values: vec!["value".to_owned()],
        }];
        let output_field_names = vec!["output_field_names-value".to_owned()];
        let value = QueryResults::new()
            .output_fields(output_fields.clone())
            .output_field_names(output_field_names.clone());

        assert_eq!(value.get_output_fields().to_owned(), output_fields);
        assert_eq!(
            value.get_output_field_names().to_owned(),
            output_field_names
        );
    }

    #[test]
    fn highlight_result_default_values() {
        let value = HighlightResult::new();
        let expected_field_name: String = String::new();
        let expected_fragments: Vec<String> = Default::default();
        let expected_scores: Vec<f32> = Default::default();

        assert_eq!(value.get_field_name().to_owned(), expected_field_name);
        assert_eq!(value.get_fragments().to_owned(), expected_fragments);
        assert_eq!(value.get_scores().to_owned(), expected_scores);
    }

    #[test]
    fn highlight_result_populated_values() {
        let field_name = "field_name-value".to_owned();
        let fragments = vec!["fragments-value".to_owned()];
        let scores = vec![1.5];
        let value = HighlightResult::new()
            .field_name(field_name.clone())
            .fragments(fragments.clone())
            .scores(scores.clone());

        assert_eq!(value.get_field_name().to_owned(), field_name);
        assert_eq!(value.get_fragments().to_owned(), fragments);
        assert_eq!(value.get_scores().to_owned(), scores);
    }

    #[test]
    fn single_result_default_values() {
        let value = SingleResult::new();
        let expected_ids: Ids = Default::default();
        let expected_scores: Vec<f32> = Default::default();
        let expected_element_indices: Option<Vec<i64>> = None;
        let expected_output_fields: Vec<FieldData> = Default::default();
        let expected_output_field_names: Vec<String> = Default::default();
        let expected_primary_field_name: String = String::new();
        let expected_score_field_name: String = String::new();
        let expected_highlight_results: Vec<HashMap<String, HighlightResult>> = Default::default();

        assert_eq!(value.get_ids().to_owned(), expected_ids);
        assert_eq!(value.get_scores().to_owned(), expected_scores);
        assert_eq!(
            value.get_element_indices(),
            expected_element_indices.as_deref()
        );
        assert_eq!(value.get_output_fields().to_owned(), expected_output_fields);
        assert_eq!(
            value.get_output_field_names().to_owned(),
            expected_output_field_names
        );
        assert_eq!(
            value.get_primary_field_name().to_owned(),
            expected_primary_field_name
        );
        assert_eq!(
            value.get_score_field_name().to_owned(),
            expected_score_field_name
        );
        assert_eq!(
            value.get_highlight_results().to_owned(),
            expected_highlight_results
        );
    }

    #[test]
    fn single_result_populated_values() {
        let ids = Ids::VarChar(vec!["id".to_owned()]);
        let scores = vec![1.5];
        let element_indices = Some(vec![3]);
        let output_fields = vec![FieldData::VarChar {
            name: "field".to_owned(),
            values: vec!["value".to_owned()],
        }];
        let output_field_names = vec!["output_field_names-value".to_owned()];
        let primary_field_name = "primary_field_name-value".to_owned();
        let score_field_name = "score_field_name-value".to_owned();
        let highlight_results = vec![HashMap::from([(
            "key-value".to_owned(),
            HighlightResult::new(),
        )])];
        let value = SingleResult::new()
            .ids(ids.clone())
            .scores(scores.clone())
            .element_indices(element_indices.clone())
            .output_fields(output_fields.clone())
            .output_field_names(output_field_names.clone())
            .primary_field_name(primary_field_name.clone())
            .score_field_name(score_field_name.clone())
            .highlight_results(highlight_results.clone());

        assert_eq!(value.get_ids().to_owned(), ids);
        assert_eq!(value.get_scores().to_owned(), scores);
        assert_eq!(value.get_element_indices(), element_indices.as_deref());
        assert_eq!(value.get_output_fields().to_owned(), output_fields);
        assert_eq!(
            value.get_output_field_names().to_owned(),
            output_field_names
        );
        assert_eq!(
            value.get_primary_field_name().to_owned(),
            primary_field_name
        );
        assert_eq!(value.get_score_field_name().to_owned(), score_field_name);
        assert_eq!(value.get_highlight_results().to_owned(), highlight_results);
    }

    #[test]
    fn search_results_default_values() {
        let value = SearchResults::new();
        let expected_results: Vec<SingleResult> = Default::default();
        let expected_recalls: Vec<f32> = Default::default();

        assert_eq!(value.get_results().to_owned(), expected_results);
        assert_eq!(value.get_recalls().to_owned(), expected_recalls);
    }

    #[test]
    fn search_results_populated_values() {
        let results = vec![SingleResult::new()];
        let recalls = vec![1.5];
        let value = SearchResults::new()
            .results(results.clone())
            .recalls(recalls.clone());

        assert_eq!(value.get_results().to_owned(), results);
        assert_eq!(value.get_recalls().to_owned(), recalls);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod constructor_value_tests {
    use super::*;

    #[test]
    fn rrf_rerank_default_values() {
        let value = RRFRerank::new();

        assert!(value.get_name().is_empty());
        assert_eq!(value.get_function_type().to_owned(), FunctionType::Rerank);
        assert_eq!(
            value.get_params().get("strategy").map(String::as_str),
            Some("rrf")
        );
        assert_eq!(
            value.get_params().get("params").map(String::as_str),
            Some(r#"{"k":60}"#)
        );
    }

    #[test]
    fn rrf_rerank_populated_values() {
        let value = RRFRerank::new().k(80);

        assert_eq!(
            value.get_params().get("params").map(String::as_str),
            Some(r#"{"k":80}"#)
        );
        let converted: Function = value.into();
        assert_eq!(
            converted.get_function_type().to_owned(),
            FunctionType::Rerank
        );
    }

    #[test]
    fn weighted_rerank_default_values() {
        let value = WeightedRerank::new().weights(Vec::new());

        assert!(value.get_name().is_empty());
        assert_eq!(value.get_function_type().to_owned(), FunctionType::Rerank);
        assert_eq!(
            value.get_params().get("strategy").map(String::as_str),
            Some("weighted")
        );
        assert_eq!(
            value.get_params().get("params").map(String::as_str),
            Some(r#"{"weights":[]}"#)
        );
    }

    #[test]
    fn weighted_rerank_populated_values() {
        let value = WeightedRerank::new().weights(vec![0.25, 0.75]);

        assert_eq!(
            value.get_params().get("params").map(String::as_str),
            Some(r#"{"weights":[0.25,0.75]}"#)
        );
        let converted: Function = value.into();
        assert_eq!(
            converted.get_params().get("strategy").map(String::as_str),
            Some("weighted")
        );
    }

    #[test]
    fn boost_rerank_default_values() {
        let value = BoostRerank::new().name("");

        assert!(value.get_name().is_empty());
        assert_eq!(
            value.get_params().get("reranker").map(String::as_str),
            Some("boost")
        );
        assert_eq!(value.get_params().len().to_owned(), 1);
    }

    #[test]
    fn boost_rerank_populated_values() {
        let value = BoostRerank::new()
            .name("boost")
            .filter("score > 0")
            .weight(2.0)
            .random_score_field("id")
            .random_score_seed(42);

        assert_eq!(value.get_name().to_owned(), "boost");
        assert_eq!(
            value.get_params().get("filter").map(String::as_str),
            Some("score > 0")
        );
        assert_eq!(
            value.get_params().get("weight").map(String::as_str),
            Some("2")
        );
        let random: serde_json::Value =
            serde_json::from_str(&value.get_params()["random_score"]).expect("random score JSON");
        assert_eq!(random["field"], "id");
        assert_eq!(random["seed"], 42);
    }

    #[test]
    fn decay_rerank_default_values() {
        let value = DecayRerank::new().name("");

        assert!(value.get_name().is_empty());
        assert_eq!(
            value.get_params().get("reranker").map(String::as_str),
            Some("decay")
        );
        assert_eq!(value.get_params().len().to_owned(), 1);
    }

    #[test]
    fn decay_rerank_populated_values() {
        let value = DecayRerank::new()
            .name("freshness")
            .decay_function("gauss")
            .origin(100)
            .offset(5)
            .scale(20)
            .decay(0.5);

        assert_eq!(value.get_name().to_owned(), "freshness");
        assert_eq!(
            value.get_params().get("function").map(String::as_str),
            Some("gauss")
        );
        assert_eq!(
            value.get_params().get("origin").map(String::as_str),
            Some("100")
        );
        assert_eq!(
            value.get_params().get("offset").map(String::as_str),
            Some("5")
        );
        assert_eq!(
            value.get_params().get("scale").map(String::as_str),
            Some("20")
        );
        assert_eq!(
            value.get_params().get("decay").map(String::as_str),
            Some("0.5")
        );
    }

    #[test]
    fn model_rerank_default_values() {
        let value = ModelRerank::new().name("");

        assert!(value.get_name().is_empty());
        assert_eq!(
            value.get_params().get("reranker").map(String::as_str),
            Some("model")
        );
        assert_eq!(value.get_params().len().to_owned(), 1);
    }

    #[test]
    fn model_rerank_populated_values() {
        let value = ModelRerank::new()
            .name("cross-encoder")
            .provider("tei")
            .queries(["milvus"])
            .endpoint("http://localhost:8080")
            .max_client_batch_size(16);

        assert_eq!(value.get_name().to_owned(), "cross-encoder");
        assert_eq!(
            value.get_params().get("provider").map(String::as_str),
            Some("tei")
        );
        assert_eq!(
            value.get_params().get("queries").map(String::as_str),
            Some(r#"["milvus"]"#)
        );
        assert_eq!(
            value.get_params().get("endpoint").map(String::as_str),
            Some("http://localhost:8080")
        );
        assert_eq!(
            value
                .get_params()
                .get("max_client_batch_size")
                .map(String::as_str),
            Some("16")
        );
    }

    #[test]
    fn embedding_list_constructor_values() {
        let value = EmbeddingList::new().vectors(Vec::new());
        assert!(value.get_vectors().is_empty());
    }

    #[test]
    fn embedding_list_populated_values() {
        let vectors = vec![vec![1.0, 2.0]];
        let value = EmbeddingList::new().vectors(vectors.clone());
        assert_eq!(value.get_vectors().to_owned(), vectors);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod enum_conversion_tests {
    use super::*;
    use crate::v2::request::dql::SearchRequest;

    #[test]
    fn search_vectors_convert_through_search_request() {
        let values = [
            SearchVectors::Float(vec![vec![1.0, 2.0]]),
            SearchVectors::Binary(vec![vec![0b1010_1010]]),
            SearchVectors::Float16(vec![vec![0x3c00]]),
            SearchVectors::BFloat16(vec![vec![0x3f80]]),
            SearchVectors::SparseFloat(vec![SparseVector::from([(1, 0.5)])]),
            SearchVectors::Int8(vec![vec![1, 2]]),
            SearchVectors::EmbeddedText(vec!["milvus".to_owned()]),
            SearchVectors::EmbeddingLists(vec![EmbeddingList::new().vectors(vec![vec![1.0, 2.0]])]),
        ];

        for vectors in values {
            let proto = SearchRequest::builder()
                .collection_name("collection")
                .vectors(vectors)
                .build()
                .expect("valid request")
                .into_proto("default", 0)
                .expect("search vector conversion");
            assert!(matches!(
                proto.search_input,
                Some(crate::proto::milvus::search_request::SearchInput::PlaceholderGroup(bytes))
                    if !bytes.is_empty()
            ));
        }
    }

    #[test]
    fn highlight_type_converts_to_proto() {
        let lexical = Highlighter::new()
            .highlight_type(HighlightType::Lexical)
            .into_proto();
        let semantic = Highlighter::new()
            .highlight_type(HighlightType::Semantic)
            .into_proto();

        assert_eq!(lexical.r#type, common::HighlightType::Lexical as i32);
        assert_eq!(semantic.r#type, common::HighlightType::Semantic as i32);
    }
}

#[cfg(test)]
mod aggregation_decoding_tests {
    use super::*;
    use schema::{
        agg_hit, agg_hit_field, bucket_key_entry, metric_value, AggBucket, AggHit, AggHitField,
        BucketKeyEntry, MetricValue,
    };

    fn key(field_id: i64, field_name: &str, value: bucket_key_entry::Value) -> BucketKeyEntry {
        BucketKeyEntry {
            field_id,
            field_name: field_name.to_owned(),
            value: Some(value),
        }
    }

    fn metrics(values: Vec<(String, MetricValue)>) -> HashMap<String, MetricValue> {
        values.into_iter().collect()
    }

    fn metric(alias: &str, value: metric_value::Value) -> (String, MetricValue) {
        (alias.to_owned(), MetricValue { value: Some(value) })
    }

    fn hit(pk: Option<agg_hit::Pk>, fields: Vec<AggHitField>) -> AggHit {
        AggHit {
            pk,
            score: 0.5,
            fields,
        }
    }

    #[test]
    fn aggregation_bucket_decodes_nested_structure() {
        let proto = AggBucket {
            key: vec![
                key(
                    1,
                    "category",
                    bucket_key_entry::Value::StringVal("tech".to_owned()),
                ),
                key(2, "year", bucket_key_entry::Value::IntVal(2026)),
            ],
            count: 42,
            metrics: metrics(vec![
                metric("total", metric_value::Value::DoubleVal(99.5)),
                metric("min_rating", metric_value::Value::IntVal(4)),
                metric("tag", metric_value::Value::StringVal("hot".to_owned())),
            ]),
            hits: vec![hit(
                Some(agg_hit::Pk::IntPk(7)),
                vec![AggHitField {
                    field_id: 1,
                    field_name: "title".to_owned(),
                    value: Some(agg_hit_field::Value::StringVal("milvus".to_owned())),
                }],
            )],
            sub_groups: vec![AggBucket {
                key: vec![key(3, "sub", bucket_key_entry::Value::BoolVal(true))],
                count: 1,
                metrics: metrics(vec![metric("total", metric_value::Value::IntVal(1))]),
                hits: vec![],
                sub_groups: vec![],
            }],
        };

        let bucket = AggregationBucket::from_proto(proto).expect("valid bucket");
        assert_eq!(bucket.get_count(), 42);
        assert_eq!(bucket.get_key().len(), 2);
        assert_eq!(bucket.get_key()[0].get_field_name(), "category");
        assert_eq!(
            bucket.get_key()[0].get_value(),
            &AggregationBucketValue::String("tech".to_owned())
        );
        assert_eq!(
            bucket.get_key()[1].get_value(),
            &AggregationBucketValue::Int(2026)
        );
        assert_eq!(
            bucket.get_metrics().get("total"),
            Some(&AggregationMetricValue::Double(99.5))
        );
        assert_eq!(
            bucket.get_metrics().get("min_rating"),
            Some(&AggregationMetricValue::Int(4))
        );
        assert_eq!(
            bucket.get_metrics().get("tag"),
            Some(&AggregationMetricValue::String("hot".to_owned()))
        );
        assert_eq!(bucket.get_hits().len(), 1);
        assert_eq!(
            bucket.get_hits()[0].get_pk(),
            Some(&AggregationHitPk::Int(7))
        );
        assert_eq!(bucket.get_hits()[0].get_score(), 0.5);
        assert_eq!(
            bucket.get_hits()[0].get_fields()[0].get_value(),
            &AggregationHitFieldValue::String("milvus".to_owned())
        );
        let sub = &bucket.get_sub_groups()[0];
        assert_eq!(sub.get_count(), 1);
        assert_eq!(
            sub.get_key()[0].get_value(),
            &AggregationBucketValue::Bool(true)
        );
    }

    #[test]
    fn aggregation_bucket_rejects_missing_key_value() {
        let proto = AggBucket {
            key: vec![BucketKeyEntry {
                field_id: 1,
                field_name: "category".to_owned(),
                value: None,
            }],
            count: 1,
            metrics: metrics(vec![]),
            hits: vec![],
            sub_groups: vec![],
        };
        let error =
            AggregationBucket::from_proto(proto).expect_err("missing key value must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("bucket key"));
    }

    #[test]
    fn empty_field_names_fall_back_to_field_id() {
        let bucket = AggregationBucket::from_proto(AggBucket {
            key: vec![key(
                5,
                "",
                bucket_key_entry::Value::StringVal("x".to_owned()),
            )],
            count: 1,
            metrics: metrics(vec![]),
            hits: vec![hit(
                Some(agg_hit::Pk::IntPk(7)),
                vec![AggHitField {
                    field_id: 9,
                    field_name: String::new(),
                    value: Some(agg_hit_field::Value::StringVal("v".to_owned())),
                }],
            )],
            sub_groups: vec![],
        })
        .expect("valid bucket");

        // Both the grouping key and hit fields substitute the numeric field id when the
        // server omits the field name, mirroring pymilvus.
        assert_eq!(bucket.get_key()[0].get_field_name(), "5");
        assert_eq!(bucket.get_hits()[0].get_fields()[0].get_field_name(), "9");
        assert_eq!(bucket.get_key()[0].get_field_id(), 5);
        assert_eq!(bucket.get_hits()[0].get_fields()[0].get_field_id(), 9);
    }

    #[test]
    fn aggregation_bucket_rejects_missing_metric_value() {
        let proto = AggBucket {
            key: vec![],
            count: 1,
            metrics: metrics(vec![("total".to_owned(), MetricValue { value: None })]),
            hits: vec![],
            sub_groups: vec![],
        };
        let error = AggregationBucket::from_proto(proto)
            .expect_err("missing metric value must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("metric"));
    }

    #[test]
    fn aggregation_bucket_rejects_missing_hit_field_value() {
        let proto = AggBucket {
            key: vec![],
            count: 1,
            metrics: metrics(vec![]),
            hits: vec![hit(
                Some(agg_hit::Pk::StrPk("doc".to_owned())),
                vec![AggHitField {
                    field_id: 1,
                    field_name: "title".to_owned(),
                    value: None,
                }],
            )],
            sub_groups: vec![],
        };
        let error = AggregationBucket::from_proto(proto)
            .expect_err("missing hit field value must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
    }

    #[test]
    fn search_results_exposes_grouped_aggregation_accessors() {
        let bucket = || {
            AggregationBucket::from_proto(AggBucket {
                key: vec![],
                count: 1,
                metrics: metrics(vec![]),
                hits: vec![],
                sub_groups: vec![],
            })
            .expect("valid bucket")
        };
        let results =
            SearchResults::new().agg_buckets(vec![vec![bucket(), bucket()], vec![bucket()]]);
        assert_eq!(results.get_agg_buckets().len(), 2);
        assert_eq!(results.get_agg_buckets()[0].len(), 2);
        assert_eq!(results.get_agg_buckets()[1].len(), 1);
    }

    fn agg_bucket() -> schema::AggBucket {
        schema::AggBucket {
            key: vec![],
            count: 1,
            metrics: HashMap::new(),
            hits: vec![],
            sub_groups: vec![],
        }
    }

    #[test]
    fn groups_aggregation_buckets_per_query() {
        let grouped = group_aggregation_buckets(
            vec![agg_bucket(), agg_bucket(), agg_bucket()],
            vec![2, 1],
            2,
        )
        .expect("valid grouping");
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].len(), 2);
        assert_eq!(grouped[1].len(), 1);
        assert_eq!(grouped[0][0].get_count(), 1);
    }

    #[test]
    fn grouping_recovers_single_query_without_topks() {
        let grouped =
            group_aggregation_buckets(vec![agg_bucket()], Vec::new(), 1).expect("nq==1 recovery");
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].len(), 1);
    }

    #[test]
    fn grouping_rejects_missing_topks_for_multi_query() {
        let error = group_aggregation_buckets(vec![agg_bucket()], Vec::new(), 2)
            .expect_err("multi-query without agg_topks must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("agg_topks"));
    }

    #[test]
    fn grouping_rejects_negative_topk() {
        let error = group_aggregation_buckets(vec![agg_bucket()], vec![-1], 1)
            .expect_err("negative agg_topks must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
    }

    #[test]
    fn grouping_rejects_count_mismatch() {
        let error = group_aggregation_buckets(vec![agg_bucket(), agg_bucket()], vec![1], 1)
            .expect_err("agg_topks sum below bucket count must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("mismatch"));

        let error = group_aggregation_buckets(vec![agg_bucket()], vec![2], 1)
            .expect_err("agg_topks sum above bucket count must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("mismatch"));
    }

    #[test]
    fn grouping_builds_empty_groups_per_query() {
        let grouped =
            group_aggregation_buckets(Vec::new(), vec![0, 0], 2).expect("zero-bucket grouping");
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].len(), 0);
        assert_eq!(grouped[1].len(), 0);
    }

    #[test]
    fn grouping_treats_both_empty_as_no_aggregation() {
        let grouped = group_aggregation_buckets(Vec::new(), Vec::new(), 2).expect("no aggregation");
        assert_eq!(grouped.len(), 0);
    }

    #[test]
    fn grouping_rejects_topks_arity_mismatch() {
        let error = group_aggregation_buckets(vec![agg_bucket()], vec![1], 2)
            .expect_err("agg_topks arity must match num_queries");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("num_queries"));

        let error = group_aggregation_buckets(Vec::new(), vec![0], 2)
            .expect_err("zero-bucket topks arity must match num_queries");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("num_queries"));
    }

    #[test]
    fn grouping_rejects_zero_bucket_count_exceeding_queries() {
        let error = group_aggregation_buckets(Vec::new(), vec![1], 1)
            .expect_err("agg_topks exceeding buckets must be rejected");
        assert!(matches!(error, Error::MalformedResponse(_)));
        assert!(error.to_string().contains("mismatch"));
    }
}
