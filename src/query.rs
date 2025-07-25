//! Query module for Milvus Rust SDK
//!
//! This module provides comprehensive query functionality for interacting with Milvus collections,
//! including vector search, hybrid search, and data retrieval operations.
//!
//! ## Key Features
//!
//! - **Vector Search**: Perform ANN (Approximate Nearest Neighbor) searches on vector fields
//! - **Hybrid Search**: Combine multiple search requests with ranking algorithms
//! - **Data Query**: Retrieve data using expressions and filters
//! - **Consistency Levels**: Support for different consistency guarantees
//! - **Template Expressions**: Support for parameterized queries with template values
//!
//! ## Examples
//!
//! ```rust
//! use milvus_sdk_rust::client::Client;
//! use milvus_sdk_rust::query::{SearchOptions, QueryOptions, AnnSearchRequest, WeightedRanker};
//! use milvus_sdk_rust::value::Value;
//!
//! // Vector search
//! let options = SearchOptions::new()
//!     .limit(10)
//!     .output_fields(vec!["id".to_string(), "vector".to_string()]);
//!
//! let results = client.search("my_collection", vec![vector_data], Some(options)).await?;
//!
//! // Hybrid search
//! let req1 = AnnSearchRequest::new(vec![vector1], "field1".to_string(), params1, 10);
//! let req2 = AnnSearchRequest::new(vec![vector2], "field2".to_string(), params2, 10);
//! let ranker = WeightedRanker::new(vec![0.7, 0.3]);
//!
//! let results = client.hybrid_search("my_collection", vec![req1, req2], Box::new(ranker), None).await?;
//! ```

use std::collections::HashMap;

use prost::bytes::BytesMut;
use prost::Message;

use crate::client::{Client, ConsistencyLevel};
use crate::collection::{Collection, SearchResult};
use crate::data::FieldColumn;
use crate::error::Error as SuperError;
use crate::proto::common::{
    DslType, KeyValuePair, MsgBase, MsgType, PlaceholderGroup, PlaceholderType, PlaceholderValue,
};
use crate::proto::milvus::{QueryRequest, SearchRequest};
use crate::proto::schema::DataType;
use crate::types::Field;
use crate::utils::status_to_result;
use crate::value::Value;
use crate::{error::*, proto};

/// Timestamp value for Strong consistency level
/// Ensures that all operations are performed with the latest data
const STRONG_TIMESTAMP: u64 = 0;

/// Timestamp value for Bounded consistency level
/// Provides bounded staleness guarantees
const BOUNDED_TIMESTAMP: u64 = 2;

/// Timestamp value for Eventually consistency level
/// Provides eventual consistency guarantees
const EVENTUALLY_TIMESTAMP: u64 = 1;

/// Represents an ANN (Approximate Nearest Neighbor) search request
///
/// This struct encapsulates all the parameters needed for a single vector search operation
/// within a hybrid search. Each request can have its own search parameters, vector field,
/// and limit.
///
/// # Example
///
/// ```rust
/// use milvus_sdk_rust::query::AnnSearchRequest;
/// use milvus_sdk_rust::value::Value;
/// use milvus_sdk_rust::proto::common::KeyValuePair;
///
/// let vector_data = Value::FloatArray(vec![0.1, 0.2, 0.3]);
/// let search_params = vec![
///     KeyValuePair {
///         key: "metric_type".to_string(),
///         value: "L2".to_string(),
///     },
///     KeyValuePair {
///         key: "params".to_string(),
///         value: r#"{"nprobe":16}"#.to_string(),
///     },
/// ];
///
/// let request = AnnSearchRequest::new(
///     vec![vector_data],
///     "vector_field".to_string(),
///     search_params,
///     10
/// );
/// ```
#[derive(Debug, Clone)]
pub struct AnnSearchRequest {
    /// The query vector data
    pub data: Vec<Value<'static>>,
    /// The vector field name to search in
    pub anns_field: String,
    /// Search parameters (metric_type, params, etc.)
    pub param: Vec<KeyValuePair>,
    /// Maximum number of results to return for this request
    pub limit: usize,
    /// Optional expression to filter results
    pub expr: Option<String>,
    /// Optional template values for expression placeholders
    pub expr_params: Option<HashMap<String, crate::proto::schema::TemplateValue>>,
}

impl AnnSearchRequest {
    /// Creates a new ANN search request
    ///
    /// # Arguments
    ///
    /// * `data` - Vector data to search for
    /// * `anns_field` - Name of the vector field to search in
    /// * `param` - Search parameters as a vector of key-value pairs
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A new `AnnSearchRequest` instance
    pub fn new<T: Into<Value<'static>>>(
        data: Vec<T>,
        anns_field: String,
        param: Vec<KeyValuePair>,
        limit: usize,
    ) -> Self {
        Self {
            data: data.into_iter().map(|x| x.into()).collect(),
            anns_field,
            param,
            limit,
            expr: None,
            expr_params: None,
        }
    }

    /// Creates a new ANN search request with a single parameter
    ///
    /// # Arguments
    ///
    /// * `data` - Vector data to search for
    /// * `anns_field` - Name of the vector field to search in
    /// * `param` - Single search parameter
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A new `AnnSearchRequest` instance
    pub fn with_single_param(
        data: Vec<Value<'static>>,
        anns_field: String,
        param: KeyValuePair,
        limit: usize,
    ) -> Self {
        Self {
            data,
            anns_field,
            param: vec![param],
            limit,
            expr: None,
            expr_params: None,
        }
    }

    /// Adds an expression filter to the search request
    ///
    /// # Arguments
    ///
    /// * `expr` - Filter expression string
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_expr(mut self, expr: String) -> Self {
        self.expr = Some(expr);
        self
    }

    /// Adds template values for expression placeholders
    ///
    /// # Arguments
    ///
    /// * `expr_params` - Template values for expression placeholders
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_expr_params(
        mut self,
        expr_params: HashMap<String, crate::proto::schema::TemplateValue>,
    ) -> Self {
        self.expr_params = Some(expr_params);
        self
    }

    /// Adds a search parameter to the request
    ///
    /// # Arguments
    ///
    /// * `key` - Parameter key
    /// * `value` - Parameter value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.param.push(KeyValuePair {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Gets a parameter value by key
    ///
    /// # Arguments
    ///
    /// * `key` - Parameter key to look for
    ///
    /// # Returns
    ///
    /// Optional parameter value
    pub fn get_param(&self, key: &str) -> Option<&str> {
        self.param
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.as_str())
    }

    /// Sets the limit for this request
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Base trait for rankers used in hybrid search
///
/// This trait defines the interface for ranking algorithms that can be used
/// to combine results from multiple search requests in hybrid search operations.
///
/// ## Implementors
///
/// - `WeightedRanker`: Combines results using weighted scoring
/// - `RrfRanker`: Combines results using Reciprocal Rank Fusion algorithm
pub trait BaseRanker: Send + Sync {
    /// Get the ranker parameters as KeyValuePair
    ///
    /// # Returns
    ///
    /// A vector of key-value pairs representing the ranker's configuration
    fn get_params(&self) -> Vec<KeyValuePair>;
}

/// Weighted ranker for hybrid search
///
/// This ranker combines results from multiple search requests by applying
/// different weights to each request's scores.
///
/// ## Example
///
/// ```rust
/// use milvus_sdk_rust::query::WeightedRanker;
///
/// // Give 70% weight to first search, 30% to second search
/// let ranker = WeightedRanker::new(vec![0.7, 0.3]);
/// ```
#[derive(Debug, Clone)]
pub struct WeightedRanker {
    weights: Vec<f64>,
}

impl WeightedRanker {
    /// Creates a new weighted ranker
    ///
    /// # Arguments
    ///
    /// * `weights` - Vector of weights for each search request (should sum to 1.0)
    ///
    /// # Returns
    ///
    /// A new `WeightedRanker` instance
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }
}

impl BaseRanker for WeightedRanker {
    fn get_params(&self) -> Vec<KeyValuePair> {
        let mut params = vec![KeyValuePair {
            key: "ranker_type".to_string(),
            value: "weighted".to_string(),
        }];

        for (i, weight) in self.weights.iter().enumerate() {
            params.push(KeyValuePair {
                key: format!("weight_{}", i),
                value: weight.to_string(),
            });
        }

        params
    }
}

/// RRF (Reciprocal Rank Fusion) ranker for hybrid search
///
/// This ranker combines results using the Reciprocal Rank Fusion algorithm,
/// which is effective for combining ranked lists from different sources.
///
/// ## Example
///
/// ```rust
/// use milvus_sdk_rust::query::RrfRanker;
///
/// // Create RRF ranker with k=60 (typical value)
/// let ranker = RrfRanker::new(60.0);
/// ```
#[derive(Debug, Clone)]
pub struct RrfRanker {
    k: f64,
}

impl RrfRanker {
    /// Creates a new RRF ranker
    ///
    /// # Arguments
    ///
    /// * `k` - The k parameter for RRF algorithm (typically 60)
    ///
    /// # Returns
    ///
    /// A new `RrfRanker` instance
    pub fn new(k: f64) -> Self {
        Self { k }
    }
}

impl BaseRanker for RrfRanker {
    fn get_params(&self) -> Vec<KeyValuePair> {
        vec![
            KeyValuePair {
                key: "ranker_type".to_string(),
                value: "rrf".to_string(),
            },
            KeyValuePair {
                key: "k".to_string(),
                value: self.k.to_string(),
            },
        ]
    }
}

/// Options for hybrid search operation
///
/// Type alias for SearchOptions used in hybrid search operations
pub type HybridSearchOptions = SearchOptions;

/// QueryOptions for client.query()
///
/// This struct provides configuration options for query operations including
/// output fields, partition names, consistency levels, and template values.
///
/// ## Example
///
/// ```rust
/// use milvus_sdk_rust::query::QueryOptions;
///
/// let options = QueryOptions::new()
///     .output_fields(vec!["id".to_string(), "title".to_string()])
///     .partition_names(vec!["partition1".to_string()])
///     .limit(100)
///     .offset(0);
/// ```
#[derive(Debug, Clone)]
pub struct QueryOptions {
    output_fields: Vec<String>,
    partition_names: Vec<String>,
    guarantee_timestamp: u64,
    query_params: Vec<crate::proto::common::KeyValuePair>,
    consistency_level: i32,
    use_default_consistency: bool,
    expr_template_values: HashMap<String, crate::proto::schema::TemplateValue>,
}

// get() shares query()'s options
/// Type alias for GetOptions, which shares the same structure as QueryOptions
pub type GetOptions = QueryOptions;

// For get to use different params
/// Enum representing different ID types for get operations
///
/// This enum allows specifying whether the IDs are integers or strings,
/// which affects how the query expression is constructed.
pub enum IdType {
    /// Integer IDs (i64)
    Int64(Vec<i64>),
    /// String IDs (VarChar)
    VarChar(Vec<String>),
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            output_fields: Vec::new(),
            partition_names: Vec::new(),
            guarantee_timestamp: 0,
            query_params: vec![],
            consistency_level: 0,
            use_default_consistency: false,
            expr_template_values: HashMap::new(),
        }
    }
}

impl QueryOptions {
    /// Creates a new QueryOptions instance with default values
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates QueryOptions with specified output fields
    ///
    /// # Arguments
    ///
    /// * `output_fields` - Vector of field names to return in the query results
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn with_output_fields(output_fields: Vec<String>) -> Self {
        Self::default().output_fields(output_fields)
    }

    /// Creates QueryOptions with specified partition names
    ///
    /// # Arguments
    ///
    /// * `partition_names` - Vector of partition names to query
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn with_partition_names(partition_names: Vec<String>) -> Self {
        Self::default().partition_names(partition_names)
    }

    /// Creates QueryOptions with specified guarantee timestamp
    ///
    /// # Arguments
    ///
    /// * `guarantee_timestamp` - Timestamp for consistency guarantee
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn with_guarantee_timestamp(guarantee_timestamp: u64) -> Self {
        Self::default().guarantee_timestamp(guarantee_timestamp)
    }

    /// Creates QueryOptions with specified query parameters
    ///
    /// # Arguments
    ///
    /// * `query_params` - Vector of key-value pairs for query parameters
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn with_query_params(query_params: Vec<crate::proto::common::KeyValuePair>) -> Self {
        Self::default().query_params(query_params)
    }

    /// Creates QueryOptions with specified consistency level
    ///
    /// # Arguments
    ///
    /// * `consistency_level` - Consistency level as integer
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn with_consistency_level(consistency_level: i32) -> Self {
        Self::default().consistency_level(consistency_level)
    }

    /// Creates QueryOptions with specified template values
    ///
    /// # Arguments
    ///
    /// * `expr_template_values` - HashMap of template key-value pairs
    ///
    /// # Returns
    ///
    /// A new `QueryOptions` instance
    pub fn with_expr_template_values(
        expr_template_values: HashMap<String, crate::proto::schema::TemplateValue>,
    ) -> Self {
        Self::default().expr_template_values(expr_template_values)
    }

    /// Sets the output fields for the query
    ///
    /// # Arguments
    ///
    /// * `output_fields` - Vector of field names to return
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn output_fields(mut self, output_fields: Vec<String>) -> Self {
        self.output_fields = output_fields;
        self
    }

    /// Sets the partition names for the query
    ///
    /// # Arguments
    ///
    /// * `partition_names` - Vector of partition names
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn partition_names(mut self, partition_names: Vec<String>) -> Self {
        self.partition_names = partition_names;
        self
    }

    /// Sets the guarantee timestamp for consistency
    ///
    /// # Arguments
    ///
    /// * `guarantee_timestamp` - Timestamp value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn guarantee_timestamp(mut self, guarantee_timestamp: u64) -> Self {
        self.guarantee_timestamp = guarantee_timestamp;
        self
    }

    /// Sets the query parameters
    ///
    /// # Arguments
    ///
    /// * `query_params` - Vector of key-value pairs
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn query_params(mut self, query_params: Vec<crate::proto::common::KeyValuePair>) -> Self {
        self.query_params = query_params;
        self
    }

    /// Sets the consistency level
    ///
    /// # Arguments
    ///
    /// * `consistency_level` - Consistency level as integer
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn consistency_level(mut self, consistency_level: i32) -> Self {
        self.consistency_level = consistency_level;
        self
    }

    /// Sets whether to use default consistency
    ///
    /// # Arguments
    ///
    /// * `use_default_consistency` - Boolean flag
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn use_default_consistency(mut self, use_default_consistency: bool) -> Self {
        self.use_default_consistency = use_default_consistency;
        self
    }

    /// Sets the template values for expressions
    ///
    /// # Arguments
    ///
    /// * `expr_template_values` - HashMap of template values
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn expr_template_values(
        mut self,
        expr_template_values: HashMap<String, crate::proto::schema::TemplateValue>,
    ) -> Self {
        self.expr_template_values = expr_template_values;
        self
    }

    /// Adds a template value for expression placeholders
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Template value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_value(
        mut self,
        key: String,
        value: crate::proto::schema::TemplateValue,
    ) -> Self {
        self.expr_template_values.insert(key, value);
        self
    }

    /// Adds a boolean template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Boolean value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_bool(mut self, key: String, value: bool) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::BoolVal(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds an int64 template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Int64 value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_int64(mut self, key: String, value: i64) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::Int64Val(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds a float template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Float value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn template_float(mut self, key: String, value: f64) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::FloatVal(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds a string template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - String value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn template_string(mut self, key: String, value: String) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::StringVal(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds collection ID parameter
    ///
    /// # Arguments
    ///
    /// * `collection_id` - Collection ID
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn collection_id(mut self, collection_id: i64) -> Self {
        self.query_params.push(KeyValuePair {
            key: "collection_id".to_string(),
            value: collection_id.to_string(),
        });
        self
    }

    /// Adds limit parameter
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn limit(mut self, limit: i64) -> Self {
        self.query_params.push(KeyValuePair {
            key: "limit".to_string(),
            value: limit.to_string(),
        });
        self
    }

    /// Adds offset parameter
    ///
    /// # Arguments
    ///
    /// * `offset` - Number of results to skip
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn offset(mut self, offset: i64) -> Self {
        self.query_params.push(KeyValuePair {
            key: "offset".to_string(),
            value: offset.to_string(),
        });
        self
    }

    /// Adds ignore_growing parameter
    ///
    /// # Arguments
    ///
    /// * `ignore_growing` - Whether to ignore growing segments
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn ignore_growing(mut self, ignore_growing: bool) -> Self {
        self.query_params.push(KeyValuePair {
            key: "ignore_growing".to_string(),
            value: ignore_growing.to_string(),
        });
        self
    }

    /// Adds iterator parameter for pagination
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn iterator(mut self) -> Self {
        self.query_params.push(KeyValuePair {
            key: "iterator".to_string(),
            value: true.to_string(),
        });
        self
    }

    /// Adds reduce_stop_for_best parameter
    ///
    /// # Arguments
    ///
    /// * `reduce_stop_for_best` - Whether to stop reduction for best results
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn reduce_stop_for_best(mut self, reduce_stop_for_best: bool) -> Self {
        self.query_params.push(KeyValuePair {
            key: "reduce_stop_for_rest".to_string(),
            value: reduce_stop_for_best.to_string(),
        });
        self
    }
}

/// Search options for vector search operations
///
/// This struct provides configuration options for search operations including
/// filters, limits, output fields, search parameters, and template values.
///
/// ## Example
///
/// ```rust
/// use milvus_sdk_rust::query::SearchOptions;
///
/// let options = SearchOptions::new()
///     .limit(10)
///     .output_fields(vec!["id".to_string(), "vector".to_string()])
///     .filter("age > 18".to_string())
///     .add_param("metric_type", "L2");
/// ```
pub struct SearchOptions {
    pub(crate) filter: String,
    pub(crate) limit: usize,
    pub(crate) output_fields: Vec<String>,
    pub(crate) search_params: Vec<KeyValuePair>,
    pub(crate) partition_names: Vec<String>,
    pub(crate) anns_field: Vec<String>,
    pub(crate) ranker: Option<Box<dyn BaseRanker>>,
    pub(crate) expr_template_values: HashMap<String, proto::schema::TemplateValue>,
    pub(crate) other_params: Option<Vec<KeyValuePair>>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            filter: "".to_string(),
            limit: 10,
            output_fields: Vec::new(),
            search_params: Vec::new(),
            partition_names: Vec::new(),
            anns_field: Vec::new(),
            ranker: None,
            expr_template_values: HashMap::new(),
            other_params: None,
        }
    }
}

impl SearchOptions {
    /// Creates a new SearchOptions instance with default values
    ///
    /// # Returns
    ///
    /// A new `SearchOptions` instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates SearchOptions with specified limit
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A new `SearchOptions` instance
    pub fn with_limit(limit: usize) -> Self {
        Self::default().limit(limit)
    }

    /// Creates SearchOptions with specified output fields
    ///
    /// # Arguments
    ///
    /// * `output_fields` - Vector of field names to return
    ///
    /// # Returns
    ///
    /// A new `SearchOptions` instance
    pub fn with_output_fields(output_fields: Vec<String>) -> Self {
        Self::default().output_fields(output_fields)
    }

    /// Creates SearchOptions with specified partitions
    ///
    /// # Arguments
    ///
    /// * `partitions` - Vector of partition names
    ///
    /// # Returns
    ///
    /// A new `SearchOptions` instance
    pub fn with_partitions(partitions: Vec<String>) -> Self {
        Self::default().partitions(partitions)
    }

    /// Adds radius parameter for range search
    ///
    /// # Arguments
    ///
    /// * `radius` - Search radius value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn radius(self, radius: f32) -> Self {
        self.add_param("radius", radius.to_string())
    }

    /// Sets the filter expression for the search
    ///
    /// # Arguments
    ///
    /// * `filter` - Boolean expression string to filter results
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn filter(mut self, filter: String) -> Self {
        self.filter = filter;
        self
    }

    /// Sets the maximum number of results to return
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Sets the output fields for the search results
    ///
    /// # Arguments
    ///
    /// * `output_fields` - Vector of field names to return
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn output_fields(mut self, output_fields: Vec<String>) -> Self {
        self.output_fields = output_fields;
        self
    }

    /// Sets the partition names for the search
    ///
    /// # Arguments
    ///
    /// * `partitions` - Vector of partition names
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn partitions(mut self, partitions: Vec<String>) -> Self {
        self.partition_names = partitions;
        self
    }

    /// Sets the ANN field names for the search
    ///
    /// # Arguments
    ///
    /// * `anns_field` - Vector of ANN field names
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn anns_field(mut self, anns_field: Vec<String>) -> Self {
        self.anns_field = anns_field;
        self
    }

    /// Adds a search parameter
    ///
    /// # Arguments
    ///
    /// * `key` - Parameter key
    /// * `value` - Parameter value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.search_params.push(KeyValuePair {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Adds a template value for expression placeholders
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Template value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_value(
        mut self,
        key: String,
        value: crate::proto::schema::TemplateValue,
    ) -> Self {
        self.expr_template_values.insert(key, value);
        self
    }

    /// Adds a boolean template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Boolean value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_bool(mut self, key: String, value: bool) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::BoolVal(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds an int64 template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Int64 value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_int64(mut self, key: String, value: i64) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::Int64Val(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds a float template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - Float value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_float(mut self, key: String, value: f64) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::FloatVal(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }

    /// Adds a string template value
    ///
    /// # Arguments
    ///
    /// * `key` - Template key
    /// * `value` - String value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn add_template_string(mut self, key: String, value: String) -> Self {
        let template_value = crate::proto::schema::TemplateValue {
            val: Some(crate::proto::schema::template_value::Val::StringVal(value)),
        };
        self.expr_template_values.insert(key, template_value);
        self
    }
}

impl Client {
    /// Gets the guarantee timestamp from consistency level
    ///
    /// This method converts a consistency level to the appropriate timestamp
    /// value used internally by Milvus for consistency guarantees.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - Name of the collection
    /// * `consistency_level` - Consistency level to convert
    ///
    /// # Returns
    ///
    /// Timestamp value corresponding to the consistency level
    pub(crate) async fn get_gts_from_consistency(
        &self,
        collection_name: &str,
        consistency_level: ConsistencyLevel,
    ) -> u64 {
        match consistency_level {
            ConsistencyLevel::Strong => STRONG_TIMESTAMP,
            ConsistencyLevel::Bounded => BOUNDED_TIMESTAMP,
            ConsistencyLevel::Eventually => EVENTUALLY_TIMESTAMP,
            ConsistencyLevel::Session => self
                .collection_cache
                .get_timestamp(collection_name)
                .unwrap_or(EVENTUALLY_TIMESTAMP),

            // This level not works for now
            ConsistencyLevel::Customized => 0,
        }
    }

    /// Performs a query operation on a collection
    ///
    /// This method retrieves data from a collection using a boolean expression
    /// and optional configuration parameters.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - Name of the collection to query
    /// * `expr` - Boolean expression string to filter results
    /// * `options` - Query configuration options
    ///
    /// # Returns
    ///
    /// Vector of field columns containing the query results
    ///
    /// # Example
    ///
    /// ```rust
    /// use milvus_sdk_rust::query::QueryOptions;
    ///
    /// let options = QueryOptions::new()
    ///     .output_fields(vec!["id".to_string(), "title".to_string()])
    ///     .limit(100);
    ///
    /// let results = client.query("my_collection", "age > 18", &options).await?;
    /// ```
    pub async fn query<S>(
        &self,
        collection_name: S,
        expr: &str,
        options: &QueryOptions,
    ) -> Result<Vec<FieldColumn>>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let collection = self.collection_cache.get(&collection_name).await?;

        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(MsgBase::new(MsgType::Retrieve)),
                db_name: "".to_string(),
                collection_name: collection_name.clone(),
                expr: expr.to_string(),
                output_fields: options.output_fields.clone(),
                partition_names: options.partition_names.clone(),
                travel_timestamp: 0,
                guarantee_timestamp: if options.guarantee_timestamp > 0 {
                    options.guarantee_timestamp
                } else {
                    self.get_gts_from_consistency(&collection_name, collection.consistency_level)
                        .await
                },
                query_params: options.query_params.clone(),
                not_return_all_meta: false,
                consistency_level: if options.consistency_level > 0 {
                    options.consistency_level
                } else {
                    0
                },
                use_default_consistency: options.use_default_consistency,
                expr_template_values: options.expr_template_values.clone(),
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res.fields_data.into_iter().map(Into::into).collect())
    }

    /// Performs a vector search operation on a collection
    ///
    /// This method searches for similar vectors in a collection using
    /// approximate nearest neighbor (ANN) algorithms.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - Name of the collection to search
    /// * `data` - Vector data to search for
    /// * `options` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Vector of search results containing scores and field data
    ///
    /// # Example
    ///
    /// ```rust
    /// use milvus_sdk_rust::query::SearchOptions;
    /// use milvus_sdk_rust::value::Value;
    ///
    /// let vector_data = Value::FloatArray(vec![0.1, 0.2, 0.3]);
    /// let options = SearchOptions::new()
    ///     .limit(10)
    ///     .output_fields(vec!["id".to_string(), "vector".to_string()])
    ///     .add_param("metric_type", "L2");
    ///
    /// let results = client.search("my_collection", vec![vector_data], Some(options)).await?;
    /// ```
    pub async fn search<S>(
        &self,
        collection_name: S,
        data: Vec<Value<'_>>,
        options: Option<SearchOptions>,
    ) -> Result<Vec<SearchResult<'_>>>
    where
        S: Into<String>,
    {
        let options = options.unwrap_or_default();
        // check and prepare params
        let mut search_params = options.search_params.clone();
        if let Some(other_params) = &options.other_params {
            search_params.extend(other_params.clone());
        }

        // Add default parameters if not present
        if !search_params.iter().any(|p| p.key == "topk") {
            search_params.push(KeyValuePair {
                key: "topk".to_string(),
                value: options.limit.to_string(),
            });
        }

        if !search_params.iter().any(|p| p.key == "round_decimal") {
            search_params.push(KeyValuePair {
                key: "round_decimal".to_string(),
                value: "-1".to_string(),
            });
        }

        if !search_params.iter().any(|p| p.key == "ignore_growing") {
            search_params.push(KeyValuePair {
                key: "ignore_growing".to_string(),
                value: "false".to_string(),
            });
        }

        // Add anns_field if specified in options
        if !options.anns_field.is_empty() && !search_params.iter().any(|p| p.key == "anns_field") {
            search_params.push(KeyValuePair {
                key: "anns_field".to_string(),
                value: options.anns_field[0].clone(),
            });
        }

        // Merge all parameters into a single params field (similar to Python's get_params)
        let merged_params = get_params(&search_params);
        search_params.push(KeyValuePair {
            key: "params".to_string(),
            value: merged_params,
        });

        let collection_name = collection_name.into();
        let collection = self.collection_cache.get(&collection_name).await?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let res = self
            .client
            .clone()
            .search(SearchRequest {
                base: Some(MsgBase::new(MsgType::Search)),
                db_name: "".to_string(),
                collection_name: collection_name.clone(),
                partition_names: options.partition_names.clone(),
                dsl: options.filter,
                nq: data.len() as _,
                placeholder_group: get_place_holder_group(&data)?,
                dsl_type: DslType::BoolExprV1 as _,
                output_fields: options
                    .output_fields
                    .clone()
                    .into_iter()
                    .map(|f| f.into())
                    .collect(),
                search_params,
                travel_timestamp: 0,
                guarantee_timestamp: self
                    .get_gts_from_consistency(&collection_name, collection.consistency_level)
                    .await,
                not_return_all_meta: false,
                consistency_level: ConsistencyLevel::default() as _,
                use_default_consistency: false,
                search_by_primary_keys: false,
                expr_template_values: options.expr_template_values.clone(),
                sub_reqs: vec![],
                function_score: None,
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        let raw_data = res
            .results
            .ok_or(SuperError::Unexpected("no result for search".to_owned()))?;
        let mut result = Vec::new();
        let mut offset = 0;
        let fields_data = raw_data
            .fields_data
            .into_iter()
            .map(Into::into)
            .collect::<Vec<FieldColumn>>();

        // Handle case where no IDs are returned (empty search results)
        let raw_id = raw_data
            .ids
            .ok_or(SuperError::Unexpected("no IDs in search result".to_owned()))?
            .id_field
            .ok_or(SuperError::Unexpected(
                "no ID field in search result".to_owned(),
            ))?;

        for k in raw_data.topks {
            let k = k as usize;
            let mut score = Vec::new();
            score.extend_from_slice(&raw_data.scores[offset..offset + k]);
            let mut result_data = fields_data
                .iter()
                .map(FieldColumn::copy_with_metadata)
                .collect::<Vec<FieldColumn>>();
            for j in 0..fields_data.len() {
                for i in offset..offset + k {
                    result_data[j].push(fields_data[j].get(i).ok_or(SuperError::Unexpected(
                        "out of range while indexing field data".to_owned(),
                    ))?);
                }
            }

            let id = match raw_id {
                proto::schema::i_ds::IdField::IntId(ref d) => {
                    Vec::<Value>::from_iter(d.data[offset..offset + k].iter().map(|&x| x.into()))
                }
                proto::schema::i_ds::IdField::StrId(ref d) => Vec::<Value>::from_iter(
                    d.data[offset..offset + k].iter().map(|x| x.clone().into()),
                ),
            };

            result.push(SearchResult {
                size: k as i64,
                score,
                field: result_data,
                id,
            });

            offset += k;
        }

        Ok(result)
    }

    /// Performs a hybrid search operation on a collection
    ///
    /// This method combines multiple search requests using a ranking algorithm
    /// to produce unified results from different search strategies.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - Name of the collection to search
    /// * `reqs` - Vector of ANN search requests
    /// * `ranker` - Ranking algorithm to combine results
    /// * `options` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Vector of search results with combined rankings
    ///
    /// # Example
    ///
    /// ```rust
    /// use milvus_sdk_rust::query::{AnnSearchRequest, WeightedRanker, SearchOptions};
    /// use milvus_sdk_rust::value::Value;
    /// use milvus_sdk_rust::proto::common::KeyValuePair;
    ///
    /// let vector1 = Value::FloatArray(vec![0.1, 0.2, 0.3]);
    /// let vector2 = Value::FloatArray(vec![0.4, 0.5, 0.6]);
    ///
    /// let req1 = AnnSearchRequest::new(
    ///     vec![vector1],
    ///     "field1".to_string(),
    ///     KeyValuePair { key: "metric_type".to_string(), value: "L2".to_string() },
    ///     10
    /// );
    ///
    /// let req2 = AnnSearchRequest::new(
    ///     vec![vector2],
    ///     "field2".to_string(),
    ///     KeyValuePair { key: "metric_type".to_string(), value: "IP".to_string() },
    ///     10
    /// );
    ///
    /// let ranker = WeightedRanker::new(vec![0.7, 0.3]);
    /// let results = client.hybrid_search("my_collection", vec![req1, req2], Box::new(ranker), None).await?;
    /// ```
    pub async fn hybrid_search<S>(
        &self,
        collection_name: S,
        reqs: Vec<AnnSearchRequest>,
        ranker: Box<dyn BaseRanker>,
        options: Option<HybridSearchOptions>,
    ) -> Result<Vec<SearchResult<'_>>>
    where
        S: Into<String>,
    {
        let options = options.unwrap_or_default();
        let collection_name = collection_name.into();
        let collection = self.collection_cache.get(&collection_name).await?;

        // Convert AnnSearchRequests to SearchRequests
        let mut search_requests = Vec::new();
        for req in reqs {
            // Create search parameters for this specific request
            let mut search_params = req.param.clone();

            // Add default parameters if not present
            if !search_params.iter().any(|p| p.key == "topk") {
                search_params.push(KeyValuePair {
                    key: "topk".to_string(),
                    value: req.limit.to_string(),
                });
            }

            if !search_params.iter().any(|p| p.key == "round_decimal") {
                search_params.push(KeyValuePair {
                    key: "round_decimal".to_string(),
                    value: "-1".to_string(),
                });
            }

            if !search_params.iter().any(|p| p.key == "ignore_growing") {
                search_params.push(KeyValuePair {
                    key: "ignore_growing".to_string(),
                    value: "false".to_string(),
                });
            }

            // Add anns_field if not present
            if !search_params.iter().any(|p| p.key == "anns_field") {
                search_params.push(KeyValuePair {
                    key: "anns_field".to_string(),
                    value: req.anns_field.clone(),
                });
            }

            // Merge all parameters into a single params field (similar to Python's get_params)
            let merged_params = get_params(&search_params);
            if !search_params.iter().any(|p| p.key == "params") {
                search_params.push(KeyValuePair {
                    key: "params".to_string(),
                    value: merged_params,
                });
            }

            // Create placeholder group for this request
            let placeholder_group = get_place_holder_group(&req.data)?;

            // Create SearchRequest for this AnnSearchRequest
            let search_request = proto::milvus::SearchRequest {
                base: None,
                db_name: "".to_string(),
                collection_name: collection_name.clone(),
                partition_names: options.partition_names.clone(),
                dsl: req.expr.unwrap_or_else(|| "".to_string()),
                placeholder_group,
                dsl_type: proto::common::DslType::BoolExprV1 as i32,
                output_fields: options.output_fields.clone(),
                search_params: search_params.clone(),
                travel_timestamp: 0,
                guarantee_timestamp: self
                    .get_gts_from_consistency(&collection_name, collection.consistency_level)
                    .await,
                nq: req.data.len() as i64,
                not_return_all_meta: false,
                consistency_level: {
                    let level = extract_param(&search_params, "consistency_level", "0");
                    level.parse().unwrap_or(ConsistencyLevel::default() as _)
                },
                use_default_consistency: extract_param(
                    &search_params,
                    "use_default_consistency",
                    "true",
                )
                .parse()
                .unwrap_or(true),
                search_by_primary_keys: false,
                sub_reqs: vec![],
                expr_template_values: req.expr_params.unwrap_or_default(),
                function_score: None,
            };

            search_requests.push(search_request);
        }

        // Prepare ranker parameters
        let rank_params = prepare_rank_params(&vec![], ranker.get_params());

        // Create HybridSearchRequest
        let request = proto::milvus::HybridSearchRequest {
            base: None,
            db_name: "".to_string(),
            collection_name: collection_name.clone(),
            partition_names: options.partition_names,
            requests: search_requests,
            rank_params,
            travel_timestamp: 0,
            guarantee_timestamp: self
                .get_gts_from_consistency(&collection_name, collection.consistency_level)
                .await,
            not_return_all_meta: false,
            output_fields: options.output_fields,
            consistency_level: extract_param(&options.search_params, "consistency_level", "0")
                .parse()
                .unwrap_or(ConsistencyLevel::default() as _),
            use_default_consistency: extract_param(
                &options.search_params,
                "use_default_consistency",
                "true",
            )
            .parse()
            .unwrap_or(true),
            function_score: None,
        };

        let res = self
            .client
            .clone()
            .hybrid_search(request)
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        let raw_data = res.results.ok_or(SuperError::Unexpected(
            "no result for hybrid search".to_owned(),
        ))?;

        let mut result = Vec::new();
        let mut offset = 0;
        let fields_data = raw_data
            .fields_data
            .into_iter()
            .map(Into::into)
            .collect::<Vec<FieldColumn>>();

        // Handle case where no IDs are returned (empty search results)
        let raw_id = raw_data
            .ids
            .ok_or(SuperError::Unexpected(
                "no IDs in hybrid search result".to_owned(),
            ))?
            .id_field
            .ok_or(SuperError::Unexpected(
                "no ID field in hybrid search result".to_owned(),
            ))?;

        for k in raw_data.topks {
            let k = k as usize;
            let mut score = Vec::new();
            score.extend_from_slice(&raw_data.scores[offset..offset + k]);
            let mut result_data = fields_data
                .iter()
                .map(FieldColumn::copy_with_metadata)
                .collect::<Vec<FieldColumn>>();
            for j in 0..fields_data.len() {
                for i in offset..offset + k {
                    result_data[j].push(fields_data[j].get(i).ok_or(SuperError::Unexpected(
                        "out of range while indexing field data".to_owned(),
                    ))?);
                }
            }

            let id = match raw_id {
                proto::schema::i_ds::IdField::IntId(ref d) => {
                    Vec::<Value>::from_iter(d.data[offset..offset + k].iter().map(|&x| x.into()))
                }
                proto::schema::i_ds::IdField::StrId(ref d) => Vec::<Value>::from_iter(
                    d.data[offset..offset + k].iter().map(|x| x.clone().into()),
                ),
            };

            result.push(SearchResult {
                size: k as i64,
                score,
                field: result_data,
                id,
            });

            offset += k;
        }

        Ok(result)
    }

    /// Extracts the primary key field from a collection
    ///
    /// # Arguments
    ///
    /// * `collection` - Collection reference
    ///
    /// # Returns
    ///
    /// Reference to the primary key field
    fn extract_primary_field<'a>(&self, collection: &'a Collection) -> Result<&'a Field> {
        collection
            .fields
            .iter()
            .find(|f| f.is_primary_key)
            .ok_or(Error::Schema(crate::schema::Error::NoPrimaryKey))
    }

    /// Packs primary key IDs into a query expression
    ///
    /// # Arguments
    ///
    /// * `collection` - Collection reference
    /// * `pks` - Vector of primary key strings
    ///
    /// # Returns
    ///
    /// Query expression string for the primary keys
    fn pack_pks_expr(&self, collection: &Collection, pks: Vec<String>) -> Result<String> {
        let primary_field = self.extract_primary_field(collection)?;
        let pk_field_name = primary_field.name.clone();
        let data_type = primary_field.dtype;

        if data_type == DataType::VarChar {
            let ids: Vec<String> = pks.iter().map(|entry| format!("'{}'", entry)).collect();
            let expr = format!("{pk_field_name} in {:?}", ids);
            return Ok(expr);
        } else {
            let mut ids: Vec<i64> = Vec::new();
            for (i, entry) in pks.iter().enumerate() {
                match entry.parse::<i64>() {
                    Ok(id) => ids.push(id),
                    Err(_) => {
                        return Err(SuperError::Unexpected(format!(
                            "Failed to parse primary key '{}' at index {} as integer",
                            entry, i
                        )));
                    }
                }
            }
            let expr = format!("{pk_field_name} in {:?}", ids);
            return Ok(expr);
        }
    }

    /// Gets specific entities by their IDs
    ///
    /// This operation retrieves entities from a collection using their primary key IDs.
    /// It supports both integer and string primary keys, similar to the Python Milvus client's get() method.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to query
    /// * `ids` - The primary key IDs to retrieve. Can be either integer or string IDs
    /// * `options` - Optional configuration for the query operation including output fields, partition names, etc.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `FieldColumn` representing the queried entities.
    /// If no IDs are provided, returns an empty vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use milvus_sdk_rust::query::{GetOptions, IdType};
    ///
    /// // Get by integer IDs
    /// let int_ids = IdType::Int64(vec![1, 2, 3]);
    /// let results = client.get("my_collection", int_ids, None).await?;
    ///
    /// // Get by string IDs
    /// let string_ids = IdType::VarChar(vec!["id1".to_string(), "id2".to_string()]);
    /// let options = GetOptions::new().output_fields(vec!["id".to_string(), "title".to_string()]);
    /// let results = client.get("my_collection", string_ids, Some(options)).await?;
    /// ```
    pub async fn get<S>(
        &self,
        collection_name: S,
        ids: IdType,
        options: Option<GetOptions>,
    ) -> Result<Vec<FieldColumn>>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let ids = match ids {
            IdType::Int64(ids_int64) => ids_int64
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            IdType::VarChar(ids_string) => ids_string,
        };

        let ids: Vec<String> = ids.into_iter().map(|x| x.into()).collect();

        //If ids is empty,return an empty vec
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let collection = self.collection_cache.get(&collection_name).await?;
        let expr = self.pack_pks_expr(&collection, ids)?;
        let option = options.unwrap_or_default();
        Ok(self.query(collection_name, expr.as_str(), &option).await?)
    }
}

/// Converts vector data to placeholder group format
///
/// This function serializes vector data into the format required by Milvus
/// for search operations.
///
/// # Arguments
///
/// * `vectors` - Vector of Value objects containing vector data
///
/// # Returns
///
/// Serialized placeholder group as byte vector
///
/// # Errors
///
/// Returns an error if the vector data is invalid or unsupported
pub fn get_place_holder_group(vectors: &Vec<Value>) -> Result<Vec<u8>> {
    let group = PlaceholderGroup {
        placeholders: vec![get_place_holder_value(vectors)?],
    };
    let mut buf = BytesMut::new();
    group.encode(&mut buf).map_err(|e| {
        SuperError::Unexpected(format!("Failed to encode placeholder group: {}", e))
    })?;
    return Ok(buf.to_vec());
}

/// Converts vector data to placeholder value format
///
/// This function creates a PlaceholderValue from vector data, handling
/// both float and binary vector types.
///
/// # Arguments
///
/// * `vectors` - Vector of Value objects containing vector data
///
/// # Returns
///
/// PlaceholderValue with appropriate type and serialized data
///
/// # Errors
///
/// Returns an error if the vector data is invalid or unsupported
fn get_place_holder_value(vectors: &Vec<Value>) -> Result<PlaceholderValue> {
    let mut place_holder = PlaceholderValue {
        tag: "$0".to_string(),
        r#type: PlaceholderType::None as _,
        values: Vec::new(),
    };
    // if no vectors, return an empty one
    if vectors.len() == 0 {
        return Ok(place_holder);
    };

    match vectors[0] {
        Value::FloatArray(_) => place_holder.r#type = PlaceholderType::FloatVector as _,
        Value::Binary(_) => place_holder.r#type = PlaceholderType::BinaryVector as _,
        _ => {
            return Err(SuperError::from(crate::collection::Error::IllegalType(
                "place holder".to_string(),
                vec![DataType::BinaryVector, DataType::FloatVector],
            )))
        }
    };

    for v in vectors {
        match (v, &vectors[0]) {
            (Value::FloatArray(d), Value::FloatArray(_)) => {
                let mut bytes = Vec::<u8>::with_capacity(d.len() * 4);
                for f in d.iter() {
                    bytes.extend_from_slice(&f.to_le_bytes());
                }
                place_holder.values.push(bytes)
            }
            (Value::Binary(d), Value::Binary(_)) => place_holder.values.push(d.to_vec()),
            _ => {
                return Err(SuperError::from(crate::collection::Error::IllegalType(
                    "place holder".to_string(),
                    vec![DataType::BinaryVector, DataType::FloatVector],
                )))
            }
        };
    }
    return Ok(place_holder);
}

/// Extracts a parameter value from search parameters with a default fallback
///
/// This helper function searches for a parameter in the search parameters
/// and returns its value, or a default value if not found.
///
/// # Arguments
///
/// * `search_params` - Vector of key-value pairs to search in
/// * `key` - Parameter key to look for
/// * `default` - Default value to return if key is not found
///
/// # Returns
///
/// Parameter value as string, or default value if not found
fn extract_param(search_params: &Vec<KeyValuePair>, key: &str, default: &str) -> String {
    search_params
        .iter()
        .find(|param| param.key == key)
        .map(|param| param.value.clone())
        .unwrap_or_else(|| default.to_string())
}

/// Merges search parameters similar to Python's get_params function
///
/// This function combines all search parameters into a single params field,
/// similar to how Python SDK handles search parameters.
///
/// # Arguments
///
/// * `search_params` - Vector of key-value pairs to merge
///
/// # Returns
///
/// Merged parameters as a JSON string
fn get_params(search_params: &Vec<KeyValuePair>) -> String {
    use serde_json;
    use std::collections::HashMap;

    // Start with existing params if any
    let mut params: HashMap<String, serde_json::Value> = HashMap::new();

    // Find existing params field and parse it
    if let Some(params_param) = search_params.iter().find(|p| p.key == "params") {
        if let Ok(parsed_params) =
            serde_json::from_str::<HashMap<String, serde_json::Value>>(&params_param.value)
        {
            params.extend(parsed_params);
        }
    }

    // Merge all other parameters
    for param in search_params {
        if param.key != "params" {
            // Try to parse as JSON first, fallback to string
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&param.value) {
                params.insert(param.key.clone(), json_value);
            } else {
                params.insert(
                    param.key.clone(),
                    serde_json::Value::String(param.value.clone()),
                );
            }
        }
    }

    // Serialize back to JSON string
    serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
}

/// Prepares rank parameters for hybrid search
///
/// This function combines ranker-specific parameters with common search parameters
/// to create the final rank parameters for hybrid search operations.
///
/// # Arguments
///
/// * `search_params` - Vector of search parameters
/// * `rank_params` - Vector of ranker-specific parameters
///
/// # Returns
///
/// Combined rank parameters with defaults and optional parameters
fn prepare_rank_params(
    search_params: &Vec<KeyValuePair>,
    rank_params: Vec<KeyValuePair>,
) -> Vec<KeyValuePair> {
    let mut final_rank_params = rank_params;

    // Parameters with default values
    let limit = extract_param(&search_params, "limit", "10");
    let round_decimal = extract_param(&search_params, "round_decimal", "-1");
    let offset = extract_param(&search_params, "offset", "0");

    final_rank_params.push(KeyValuePair {
        key: "limit".to_string(),
        value: limit,
    });
    final_rank_params.push(KeyValuePair {
        key: "round_decimal".to_string(),
        value: round_decimal,
    });
    final_rank_params.push(KeyValuePair {
        key: "offset".to_string(),
        value: offset,
    });

    // Parameters without default values - only add if present in search_params
    let optional_params = vec![
        "rank_group_scorer",
        "group_by_field",
        "group_size",
        "strict_group_size",
    ];

    for param_name in optional_params {
        if let Some(param) = search_params.iter().find(|p| p.key == param_name) {
            final_rank_params.push(KeyValuePair {
                key: param.key.clone(),
                value: param.value.clone(),
            });
        }
    }

    final_rank_params
}
