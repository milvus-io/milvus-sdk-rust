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

//! Hierarchical bucket aggregation types for the search interface.
//!
//! [`SearchAggregation`] groups search results by one or more fields, optionally computing
//! [`MetricSpec`] aggregations, ordering the resulting buckets, returning [`TopHitsSpec`]
//! document snapshots, and nesting recursive [`SearchAggregation`] levels, mirroring pymilvus's
//! `SearchAggregation` and the Java SDK's `aggregation` package.

use crate::proto::common;
use crate::v2::error::{Error, Result};
use std::collections::HashMap;

/// Keys that ordering rules may reference without being defined metrics.
const SPECIAL_ORDER_KEYS: [&str; 2] = ["_count", "_key"];

///////////////////////////////////////////////////////////////////////////////
// MetricOp
///////////////////////////////////////////////////////////////////////////////
/// Aggregation operator applied to a field within a bucket.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetricOp {
    #[default]
    /// Represents the Avg case.
    Avg,
    /// Represents the Sum case.
    Sum,
    /// Represents the Count case.
    Count,
    /// Represents the Min case.
    Min,
    /// Represents the Max case.
    Max,
}

impl MetricOp {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Avg => "avg",
            Self::Sum => "sum",
            Self::Count => "count",
            Self::Min => "min",
            Self::Max => "max",
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SortDirection
///////////////////////////////////////////////////////////////////////////////
/// Sort direction used by aggregation order and top-hits rules.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SortDirection {
    /// Represents the Asc case.
    Asc,
    #[default]
    /// Represents the Desc case.
    Desc,
}

impl SortDirection {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// MetricSpec
///////////////////////////////////////////////////////////////////////////////
/// A single metric aggregation over a field, keyed by alias in a [`SearchAggregation`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MetricSpec {
    pub(crate) op: MetricOp,
    pub(crate) field_name: String,
}

impl MetricSpec {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            op: MetricOp::Avg,
            field_name: String::new(),
        }
    }

    /// Sets the operator and returns the updated value.
    pub fn op(mut self, value: MetricOp) -> Self {
        self.op = value;
        self
    }

    /// Sets the operator and returns this value for further mutation.
    pub fn set_op(&mut self, value: MetricOp) -> &mut Self {
        self.op = value;
        self
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

    /// Returns the operator.
    pub fn get_op(&self) -> MetricOp {
        self.op
    }

    /// Returns the field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    pub(crate) fn into_proto(self) -> common::MetricAggSpec {
        common::MetricAggSpec {
            op: self.op.as_str().to_owned(),
            field_name: self.field_name,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SortSpec
///////////////////////////////////////////////////////////////////////////////
/// Field sort rule used by a top-hits snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SortSpec {
    pub(crate) field_name: String,
    pub(crate) direction: SortDirection,
    pub(crate) null_first: bool,
}

impl SortSpec {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            field_name: String::new(),
            direction: SortDirection::Desc,
            null_first: false,
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

    /// Sets the direction and returns the updated value.
    pub fn direction(mut self, value: SortDirection) -> Self {
        self.direction = value;
        self
    }

    /// Sets the direction and returns this value for further mutation.
    pub fn set_direction(&mut self, value: SortDirection) -> &mut Self {
        self.direction = value;
        self
    }

    /// Sets whether null values sort first and returns the updated value.
    pub fn null_first(mut self, value: bool) -> Self {
        self.null_first = value;
        self
    }

    /// Sets whether null values sort first and returns this value for further mutation.
    pub fn set_null_first(&mut self, value: bool) -> &mut Self {
        self.null_first = value;
        self
    }

    /// Returns the field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// Returns the direction.
    pub fn get_direction(&self) -> SortDirection {
        self.direction
    }

    /// Returns whether null values sort first.
    pub fn is_null_first(&self) -> bool {
        self.null_first
    }

    pub(crate) fn into_proto(self) -> common::SortSpec {
        common::SortSpec {
            field_name: self.field_name,
            direction: self.direction.as_str().to_owned(),
            null_first: self.null_first,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// OrderSpec
///////////////////////////////////////////////////////////////////////////////
/// Ordering rule applied to the buckets at an aggregation level.
///
/// `key` is a metric alias or one of the special keys `_count`/`_key`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OrderSpec {
    pub(crate) key: String,
    pub(crate) direction: SortDirection,
    pub(crate) null_first: bool,
}

impl OrderSpec {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            key: String::new(),
            direction: SortDirection::Desc,
            null_first: false,
        }
    }

    /// Sets the key and returns the updated value.
    pub fn key(mut self, value: impl Into<String>) -> Self {
        self.key = value.into();
        self
    }

    /// Sets the key and returns this value for further mutation.
    pub fn set_key(&mut self, value: impl Into<String>) -> &mut Self {
        self.key = value.into();
        self
    }

    /// Sets the direction and returns the updated value.
    pub fn direction(mut self, value: SortDirection) -> Self {
        self.direction = value;
        self
    }

    /// Sets the direction and returns this value for further mutation.
    pub fn set_direction(&mut self, value: SortDirection) -> &mut Self {
        self.direction = value;
        self
    }

    /// Sets whether null values sort first and returns the updated value.
    pub fn null_first(mut self, value: bool) -> Self {
        self.null_first = value;
        self
    }

    /// Sets whether null values sort first and returns this value for further mutation.
    pub fn set_null_first(&mut self, value: bool) -> &mut Self {
        self.null_first = value;
        self
    }

    /// Returns the key.
    pub fn get_key(&self) -> &str {
        &self.key
    }

    /// Returns the direction.
    pub fn get_direction(&self) -> SortDirection {
        self.direction
    }

    /// Returns whether null values sort first.
    pub fn is_null_first(&self) -> bool {
        self.null_first
    }

    pub(crate) fn into_proto(self) -> common::OrderSpec {
        common::OrderSpec {
            key: self.key,
            direction: self.direction.as_str().to_owned(),
            null_first: self.null_first,
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// TopHitsSpec
///////////////////////////////////////////////////////////////////////////////
/// Document snapshot returned for each bucket at an aggregation level.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TopHitsSpec {
    pub(crate) size: i64,
    pub(crate) sort: Vec<SortSpec>,
}

impl TopHitsSpec {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            size: 0,
            sort: Vec::new(),
        }
    }

    /// Sets the size and returns the updated value.
    pub fn size(mut self, value: i64) -> Self {
        self.size = value;
        self
    }

    /// Sets the size and returns this value for further mutation.
    pub fn set_size(&mut self, value: i64) -> &mut Self {
        self.size = value;
        self
    }

    /// Sets the sort rules and returns the updated value.
    pub fn sort(mut self, values: impl IntoIterator<Item = SortSpec>) -> Self {
        self.sort = values.into_iter().collect();
        self
    }

    /// Sets the sort rules and returns this value for further mutation.
    pub fn set_sort(&mut self, values: impl IntoIterator<Item = SortSpec>) -> &mut Self {
        self.sort = values.into_iter().collect();
        self
    }

    /// Appends a sort rule and returns the updated value.
    pub fn add_sort(mut self, value: SortSpec) -> Self {
        self.sort.push(value);
        self
    }

    /// Returns the size.
    pub fn get_size(&self) -> i64 {
        self.size
    }

    /// Returns the sort rules.
    pub fn get_sort(&self) -> &[SortSpec] {
        &self.sort
    }

    pub(crate) fn into_proto(self) -> common::TopHitsSpec {
        common::TopHitsSpec {
            size: self.size,
            sort: self.sort.into_iter().map(SortSpec::into_proto).collect(),
            ..Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchAggregation
///////////////////////////////////////////////////////////////////////////////
/// One level of hierarchical bucket aggregation, recursively nestable.
///
/// Attach the aggregation to a search request with
/// [`SearchRequestBuilder::search_aggregation`](crate::v2::request::dql::SearchRequestBuilder::search_aggregation).
/// It is mutually exclusive with `group_by_field`; when set, the search `limit` is ignored and
/// [`Self::size`] controls the top-level bucket count.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SearchAggregation {
    pub(crate) fields: Vec<String>,
    pub(crate) size: i64,
    pub(crate) metrics: HashMap<String, MetricSpec>,
    pub(crate) order: Vec<OrderSpec>,
    pub(crate) top_hits: Option<TopHitsSpec>,
    pub(crate) sub_aggregation: Option<Box<SearchAggregation>>,
}

impl SearchAggregation {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            size: 0,
            metrics: HashMap::new(),
            order: Vec::new(),
            top_hits: None,
            sub_aggregation: None,
        }
    }

    /// Sets the group-by fields and returns the updated value.
    pub fn fields(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the group-by fields and returns this value for further mutation.
    pub fn set_fields(&mut self, values: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.fields = values.into_iter().map(Into::into).collect();
        self
    }

    /// Returns the group-by fields.
    pub fn get_fields(&self) -> &[String] {
        &self.fields
    }

    /// Appends a group-by field and returns the updated value.
    pub fn add_field(mut self, value: impl Into<String>) -> Self {
        self.fields.push(value.into());
        self
    }

    /// Sets the maximum number of buckets returned at this level and returns the updated value.
    pub fn size(mut self, value: i64) -> Self {
        self.size = value;
        self
    }

    /// Sets the maximum number of buckets returned at this level and returns this value for
    /// further mutation.
    pub fn set_size(&mut self, value: i64) -> &mut Self {
        self.size = value;
        self
    }

    /// Returns the maximum number of buckets returned at this level.
    pub fn get_size(&self) -> i64 {
        self.size
    }

    /// Sets the metric aggregations keyed by alias and returns the updated value.
    pub fn metrics(mut self, values: impl IntoIterator<Item = (String, MetricSpec)>) -> Self {
        self.metrics = values.into_iter().collect();
        self
    }

    /// Sets the metric aggregations keyed by alias and returns this value for further mutation.
    pub fn set_metrics(
        &mut self,
        values: impl IntoIterator<Item = (String, MetricSpec)>,
    ) -> &mut Self {
        self.metrics = values.into_iter().collect();
        self
    }

    /// Returns the metric aggregations keyed by alias.
    pub fn get_metrics(&self) -> &HashMap<String, MetricSpec> {
        &self.metrics
    }

    /// Adds a metric aggregation under `alias` and returns the updated value.
    pub fn add_metric(mut self, alias: impl Into<String>, value: MetricSpec) -> Self {
        self.metrics.insert(alias.into(), value);
        self
    }

    /// Sets the ordering rules and returns the updated value.
    pub fn order(mut self, values: impl IntoIterator<Item = OrderSpec>) -> Self {
        self.order = values.into_iter().collect();
        self
    }

    /// Sets the ordering rules and returns this value for further mutation.
    pub fn set_order(&mut self, values: impl IntoIterator<Item = OrderSpec>) -> &mut Self {
        self.order = values.into_iter().collect();
        self
    }

    /// Returns the ordering rules.
    pub fn get_order(&self) -> &[OrderSpec] {
        &self.order
    }

    /// Appends an ordering rule and returns the updated value.
    pub fn add_order(mut self, value: OrderSpec) -> Self {
        self.order.push(value);
        self
    }

    /// Sets the top-hits snapshot and returns the updated value.
    pub fn top_hits(mut self, value: TopHitsSpec) -> Self {
        self.top_hits = Some(value);
        self
    }

    /// Sets the top-hits snapshot and returns this value for further mutation.
    pub fn set_top_hits(&mut self, value: TopHitsSpec) -> &mut Self {
        self.top_hits = Some(value);
        self
    }

    /// Returns the top-hits snapshot, if configured.
    pub fn get_top_hits(&self) -> Option<&TopHitsSpec> {
        self.top_hits.as_ref()
    }

    /// Sets the nested sub-aggregation and returns the updated value.
    pub fn sub_aggregation(mut self, value: SearchAggregation) -> Self {
        self.sub_aggregation = Some(Box::new(value));
        self
    }

    /// Sets the nested sub-aggregation and returns this value for further mutation.
    pub fn set_sub_aggregation(&mut self, value: SearchAggregation) -> &mut Self {
        self.sub_aggregation = Some(Box::new(value));
        self
    }

    /// Returns the nested sub-aggregation, if configured.
    pub fn get_sub_aggregation(&self) -> Option<&SearchAggregation> {
        self.sub_aggregation.as_deref()
    }

    /// Validates the aggregation spec before it is attached to a search request.
    pub fn validate(&self) -> Result<()> {
        if self.fields.is_empty() {
            return Err(Error::validation(
                "search_aggregation".into(),
                "fields must contain at least one field".into(),
            ));
        }
        if self.fields.iter().any(String::is_empty) {
            return Err(Error::validation(
                "search_aggregation".into(),
                "fields must not contain empty values".into(),
            ));
        }
        if self.size <= 0 {
            return Err(Error::validation(
                "search_aggregation".into(),
                "size must be a positive integer".into(),
            ));
        }
        for (alias, metric) in &self.metrics {
            if alias.is_empty() {
                return Err(Error::validation(
                    "search_aggregation".into(),
                    "metric aliases must not be empty".into(),
                ));
            }
            if metric.field_name.is_empty() {
                return Err(Error::validation(
                    "search_aggregation".into(),
                    format!("metric {alias:?} must specify a non-empty field name"),
                ));
            }
            if metric.field_name == "*" && metric.op != MetricOp::Count {
                return Err(Error::validation(
                    "search_aggregation".into(),
                    format!(
                        "metric {alias:?}: field \"*\" is only supported with the count operator"
                    ),
                ));
            }
        }
        if let Some(top_hits) = &self.top_hits {
            if top_hits.size <= 0 {
                return Err(Error::validation(
                    "search_aggregation".into(),
                    "top_hits.size must be a positive integer".into(),
                ));
            }
            for sort in &top_hits.sort {
                if sort.field_name.is_empty() {
                    return Err(Error::validation(
                        "search_aggregation".into(),
                        "top_hits sort rules must not use an empty field name".into(),
                    ));
                }
            }
        }
        let allowed_keys: std::collections::HashSet<&str> = self
            .metrics
            .keys()
            .map(String::as_str)
            .chain(SPECIAL_ORDER_KEYS.iter().copied())
            .collect();
        for order in &self.order {
            if !allowed_keys.contains(order.key.as_str()) {
                return Err(Error::validation(
                    "search_aggregation".into(),
                    format!(
                        "order key {:?} must be a metric alias or one of {SPECIAL_ORDER_KEYS:?}",
                        order.key
                    ),
                ));
            }
        }
        if let Some(sub) = &self.sub_aggregation {
            sub.validate()?;
        }
        Ok(())
    }

    pub(crate) fn into_proto(self) -> Result<common::SearchAggregationSpec> {
        self.validate()?;
        Ok(common::SearchAggregationSpec {
            fields: self.fields,
            size: self.size,
            metrics: self
                .metrics
                .into_iter()
                .map(|(alias, metric)| (alias, metric.into_proto()))
                .collect(),
            order: self.order.into_iter().map(OrderSpec::into_proto).collect(),
            top_hits: self.top_hits.map(TopHitsSpec::into_proto),
            sub_aggregation: self
                .sub_aggregation
                .map(|sub| sub.into_proto())
                .transpose()?
                .map(Box::new),
            ..Default::default()
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spec() -> SearchAggregation {
        SearchAggregation::new()
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
            )
    }

    #[test]
    fn validate_accepts_valid_spec() {
        valid_spec().validate().expect("valid aggregation spec");
        valid_spec()
            .add_metric(
                "distinct",
                MetricSpec::new().op(MetricOp::Count).field_name("*"),
            )
            .validate()
            .expect("count over * is valid");
    }

    #[test]
    fn validate_rejects_empty_fields() {
        let error = SearchAggregation::new()
            .size(10)
            .validate()
            .expect_err("empty fields must be rejected");
        assert!(error.to_string().contains("fields"));
    }

    #[test]
    fn validate_rejects_non_positive_size() {
        let error = SearchAggregation::new()
            .fields(["category"])
            .size(0)
            .validate()
            .expect_err("non-positive size must be rejected");
        assert!(error.to_string().contains("size"));
    }

    #[test]
    fn validate_rejects_unknown_order_key() {
        let error = valid_spec()
            .add_order(OrderSpec::new().key("bogus").direction(SortDirection::Asc))
            .validate()
            .expect_err("unknown order key must be rejected");
        assert!(error.to_string().contains("order key"));
    }

    #[test]
    fn validate_rejects_empty_metric_field_name() {
        let error = valid_spec()
            .add_metric("empty", MetricSpec::new().op(MetricOp::Avg).field_name(""))
            .validate()
            .expect_err("empty metric field name must be rejected");
        assert!(error.to_string().contains("field name"));
    }

    #[test]
    fn validate_rejects_empty_metric_alias() {
        let error = valid_spec()
            .add_metric("", MetricSpec::new().op(MetricOp::Avg).field_name("price"))
            .validate()
            .expect_err("empty metric alias must be rejected");
        assert!(error.to_string().contains("alias"));
    }

    #[test]
    fn validate_rejects_non_positive_top_hits_size() {
        let error = valid_spec()
            .top_hits(TopHitsSpec::new().size(0))
            .validate()
            .expect_err("non-positive top-hits size must be rejected");
        assert!(error.to_string().contains("top_hits"));
    }

    #[test]
    fn validate_rejects_empty_top_hits_sort_field() {
        let error = valid_spec()
            .top_hits(
                TopHitsSpec::new()
                    .size(3)
                    .add_sort(SortSpec::new().field_name("").direction(SortDirection::Asc)),
            )
            .validate()
            .expect_err("empty top-hits sort field must be rejected");
        assert!(error.to_string().contains("sort"));
    }

    #[test]
    fn validate_rejects_wildcard_with_non_count_metric() {
        let error = valid_spec()
            .add_metric("bad", MetricSpec::new().op(MetricOp::Sum).field_name("*"))
            .validate()
            .expect_err("wildcard with non-count metric must be rejected");
        assert!(error.to_string().contains("count"));
    }

    #[test]
    fn validate_rejects_invalid_sub_aggregation() {
        let error = valid_spec()
            .sub_aggregation(SearchAggregation::new().size(5))
            .validate()
            .expect_err("invalid sub-aggregation must be rejected");
        assert!(error.to_string().contains("fields"));
    }

    #[test]
    fn metric_spec_encodes_to_proto() {
        let proto = MetricSpec::new()
            .op(MetricOp::Sum)
            .field_name("price")
            .into_proto();
        assert_eq!(proto.op, "sum");
        assert_eq!(proto.field_name, "price");
    }

    #[test]
    fn search_aggregation_encodes_to_proto() {
        let proto = valid_spec()
            .add_metric(
                "total",
                MetricSpec::new().op(MetricOp::Avg).field_name("rating"),
            )
            .top_hits(
                TopHitsSpec::new().size(3).add_sort(
                    SortSpec::new()
                        .field_name("price")
                        .direction(SortDirection::Asc),
                ),
            )
            .sub_aggregation(
                SearchAggregation::new()
                    .fields(["region"])
                    .size(5)
                    .add_metric("cnt", MetricSpec::new().op(MetricOp::Count).field_name("*")),
            )
            .into_proto()
            .expect("valid aggregation spec");
        assert_eq!(proto.fields, ["category"]);
        assert_eq!(proto.size, 10);
        assert_eq!(
            proto.metrics.get("total").map(|m| m.op.as_str()),
            Some("avg")
        );
        assert_eq!(proto.order[0].key, "_count");
        assert_eq!(proto.order[0].direction, "desc");
        let top_hits = proto.top_hits.expect("top hits configured");
        assert_eq!(top_hits.size, 3);
        assert_eq!(top_hits.sort[0].field_name, "price");
        assert_eq!(top_hits.sort[0].direction, "asc");
        let sub = proto.sub_aggregation.expect("sub-aggregation configured");
        assert_eq!(sub.fields, ["region"]);
        assert_eq!(sub.metrics.get("cnt").map(|m| m.op.as_str()), Some("count"));
    }
}
