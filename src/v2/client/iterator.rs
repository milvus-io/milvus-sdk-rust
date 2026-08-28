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

//! Query and search iterators for paginated V2 reads.

use super::dql::set_cluster_param;
use super::ClientV2;
use crate::proto::{common, milvus, schema};
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::{request, response};
use crate::v2::{DataType, IndexDesc, MetricType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BATCH_SIZE: usize = 16_384;
const HYBRID_TIMESTAMP_LOGICAL_BITS: u32 = 18;
const ITERATOR_FALLBACK_OFFSET_MILLIS: u64 = 1_000;
const LEGACY_SEARCH_EXTENSION_RATE: usize = 10;
const LEGACY_SEARCH_MAX_TRIES: usize = 20;
const LEGACY_SEARCH_MAX_FILTERED_IDS: usize = 100_000;
const LEGACY_SEARCH_MIN_WIDTH: f64 = 0.05;

struct SearchIteratorCollectionInfo {
    collection_id: i64,
    primary_field_name: String,
    primary_field_type: DataType,
    vector_field_names: Vec<String>,
}

///////////////////////////////////////////////////////////////////////////////
// QueryIterator
///////////////////////////////////////////////////////////////////////////////
/// Iterator that retrieves query results in stable, paginated batches.
pub struct QueryIterator {
    client: ClientV2,
    request: milvus::QueryRequest,
    batch_size: usize,
    remaining: Option<usize>,
    original_filter: String,
    primary_field_name: String,
    primary_field_type: DataType,
    cursor: Option<QueryCursor>,
    cache: Option<response::dql::QueryResponse>,
    finished: bool,
    closed: Option<Arc<AtomicBool>>,
}

impl QueryIterator {
    fn finished(client: ClientV2, batch_size: usize) -> Self {
        Self {
            client,
            request: milvus::QueryRequest::default(),
            batch_size,
            remaining: Some(0),
            original_filter: String::new(),
            primary_field_name: String::new(),
            primary_field_type: DataType::Unknown,
            cursor: None,
            cache: None,
            finished: true,
            closed: None,
        }
    }

    /// Binds this iterator to a session-closed flag so pages stop once the owning session closes.
    pub(crate) fn bind_session_close(&mut self, closed: Arc<AtomicBool>) {
        self.closed = Some(closed);
    }

    fn ensure_session_open(&self) -> Result<()> {
        if self
            .closed
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            Err(Error::Unexpected(
                "MilvusClientV2 session is closed".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    /// Retrieves the next query-result batch, or `None` when iteration is complete.
    ///
    /// Pages advance a primary-key cursor and are decoded on demand rather than materializing the
    /// entire result set.
    pub async fn next(&mut self) -> Result<Option<response::dql::QueryResponse>> {
        self.ensure_session_open()?;
        if self.finished || self.remaining == Some(0) {
            return Ok(None);
        }

        let (mut response, source_exhausted, next_cache) = if self
            .cache
            .as_ref()
            .is_some_and(|cached| query_response_row_count(cached) >= self.batch_size)
        {
            trace_debug!(target: "milvus_sdk::iterator", kind = "query", batch_size = self.batch_size, "serving query iterator page from local cache");
            let cached = self.cache.take().expect("checked query iterator cache");
            let (page, remaining) = cached.split_at(self.batch_size)?;
            (page, false, remaining)
        } else {
            trace_debug!(target: "milvus_sdk::iterator", kind = "query", batch_size = self.batch_size, remaining = ?self.remaining, "requesting next query iterator page");
            self.cache = None;
            self.request.expr = self.next_filter();
            set_param(
                &mut self.request.query_params,
                "limit",
                self.batch_size.to_string(),
            );
            set_param(&mut self.request.query_params, "offset", "0".into());
            set_param(&mut self.request.query_params, "iterator", "true".into());

            let raw = rpc_with_retry!(self.client, query, self.request.clone())?;
            status_to_result(&raw.status)?;
            let decoded = response::dql::QueryResponse::from_proto(raw)?;
            let count = query_response_row_count(&decoded);
            if count == 0 {
                self.finished = true;
                return Ok(None);
            }
            let source_exhausted = count < self.batch_size;
            // With iterator reduce_stop_for_best enabled, Milvus may return more rows than the
            // requested limit. Retain complete surplus batches so subsequent next() calls avoid
            // another RPC while advancing the cursor only through rows delivered to the caller.
            let should_cache = count >= self.batch_size.saturating_mul(2);
            let (page, remaining) = decoded.split_at(self.batch_size)?;
            let next_cache = should_cache.then_some(remaining).flatten();
            trace_debug!(target: "milvus_sdk::iterator", kind = "query", rows = count, source_exhausted, cached_surplus = next_cache.is_some(), "received query iterator page");
            (page, source_exhausted, next_cache)
        };

        if let Some(left) = self.remaining {
            let (limited, _) = response.split_at(left)?;
            response = limited;
        }
        let count = query_response_row_count(&response);
        if count == 0 {
            self.finished = true;
            return Ok(None);
        }
        let cursor =
            query_response_cursor(&response, &self.primary_field_name, self.primary_field_type)?;
        let next_remaining = self.remaining.map(|left| left.saturating_sub(count));
        let next_finished = next_remaining == Some(0) || (source_exhausted && next_cache.is_none());

        self.cursor = Some(cursor);
        self.remaining = next_remaining;
        self.cache = next_cache;
        self.finished = next_finished;
        trace_debug!(target: "milvus_sdk::iterator", kind = "query", rows = count, remaining = ?self.remaining, finished = self.finished, "completed query iterator page");
        Ok(Some(response))
    }

    /// Releases cached rows and marks the query iterator closed.
    pub async fn close(&mut self) -> Result<()> {
        self.cache = None;
        Ok(())
    }

    async fn seek_to_offset(&mut self, mut offset: usize) -> Result<()> {
        while offset > 0 {
            let size = offset.min(MAX_BATCH_SIZE);
            let mut request = self.request.clone();
            request.expr = self.next_filter();
            request.output_fields.clear();
            set_param(&mut request.query_params, "limit", size.to_string());
            set_param(&mut request.query_params, "offset", "0".into());
            set_param(&mut request.query_params, "iterator", "false".into());
            set_param(
                &mut request.query_params,
                "reduce_stop_for_best",
                "false".into(),
            );

            let raw = rpc_with_retry!(self.client, query, request)?;
            status_to_result(&raw.status)?;
            let count = query_row_count(&raw);
            if count == 0 {
                self.finished = true;
                break;
            }
            self.cursor = Some(query_proto_cursor(
                &raw,
                &self.primary_field_name,
                self.primary_field_type,
            )?);
            offset = offset.saturating_sub(count);
            if count < size {
                self.finished = true;
                break;
            }
        }
        Ok(())
    }

    fn next_filter(&self) -> String {
        let Some(cursor) = &self.cursor else {
            return self.original_filter.clone();
        };
        let cursor_filter = match cursor {
            QueryCursor::Int64(value) => format!("{} > {value}", self.primary_field_name),
            QueryCursor::VarChar(value) => format!(
                "{} > {}",
                self.primary_field_name,
                serde_json::to_string(value).expect("strings always serialize to JSON")
            ),
        };
        if self.original_filter.is_empty() {
            cursor_filter
        } else {
            format!("{cursor_filter} and ({})", self.original_filter)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// QueryCursor
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
enum QueryCursor {
    Int64(i64),
    VarChar(String),
}

///////////////////////////////////////////////////////////////////////////////
// SearchIterator
///////////////////////////////////////////////////////////////////////////////
/// Search iterator selected according to server capabilities.
#[non_exhaustive]
pub enum SearchIterator {
    /// Legacy range-search implementation for older Milvus servers.
    V1(SearchIteratorV1),
    /// Token/bound implementation for servers supporting Search Iterator V2.
    V2(SearchIteratorV2),
}

impl SearchIterator {
    /// Binds this iterator to a session-closed flag so pages stop once the owning session closes.
    pub(crate) fn bind_session_close(&mut self, closed: Arc<AtomicBool>) {
        match self {
            Self::V1(iterator) => iterator.closed = Some(closed),
            Self::V2(iterator) => iterator.closed = Some(closed),
        }
    }

    /// Retrieves the next search-result batch, or `None` when iteration is complete.
    pub async fn next(&mut self) -> Result<Option<response::dql::SearchResponse>> {
        match self {
            Self::V1(iterator) => iterator.next().await,
            Self::V2(iterator) => iterator.next().await,
        }
    }

    /// Releases iterator-local search state.
    pub async fn close(&mut self) -> Result<()> {
        match self {
            Self::V1(iterator) => iterator.close().await,
            Self::V2(iterator) => iterator.close().await,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchIteratorV1
///////////////////////////////////////////////////////////////////////////////
/// Legacy range-search iterator used when the server does not support Search Iterator V2.
pub struct SearchIteratorV1 {
    client: ClientV2,
    request: milvus::SearchRequest,
    batch_size: usize,
    remaining: Option<usize>,
    metric: MetricType,
    requested_radius: Option<f64>,
    ef: Option<usize>,
    original_filter: String,
    primary_field_name: String,
    filtered_ids: LegacyFilteredIds,
    filtered_distance: Option<f32>,
    tail_band: f64,
    width: f64,
    cache: Option<response::dql::SearchResponse>,
    finished: bool,
    closed: Option<Arc<AtomicBool>>,
}

impl SearchIteratorV1 {
    fn ensure_session_open(&self) -> Result<()> {
        if self
            .closed
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            Err(Error::Unexpected(
                "MilvusClientV2 session is closed".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    /// Retrieves the next legacy search-result batch, or `None` when iteration is complete.
    pub async fn next(&mut self) -> Result<Option<response::dql::SearchResponse>> {
        self.ensure_session_open()?;
        if self.finished || self.remaining == Some(0) {
            return Ok(None);
        }
        let size = self
            .remaining
            .map_or(self.batch_size, |left| left.min(self.batch_size));
        let mut fetched = false;
        let mut exhausted = false;

        if self.cache_row_count()? < size {
            for coefficient in 1..=LEGACY_SEARCH_MAX_TRIES + 1 {
                trace_debug!(target: "milvus_sdk::iterator", kind = "search_v1", coefficient, batch_size = size, "requesting legacy search iterator expansion page");
                let page = self.fetch_page(coefficient).await?;
                if page.row_count()? == 0 {
                    trace_debug!(target: "milvus_sdk::iterator", kind = "search_v1", coefficient, "legacy search iterator expansion returned no rows");
                    if coefficient > LEGACY_SEARCH_MAX_TRIES {
                        exhausted = true;
                    }
                    continue;
                }
                fetched = true;
                self.update_legacy_cursor(&page)?;
                if let Some(cache) = &mut self.cache {
                    cache.append(page)?;
                } else {
                    self.cache = Some(page);
                }
                if self.cache_row_count()? >= self.batch_size {
                    break;
                }
            }
        }

        let Some(cache) = self.cache.take() else {
            self.finished = true;
            return Ok(None);
        };
        let (page, remaining_cache) = cache.split_at(size)?;
        let count = page.row_count()?;
        if count == 0 {
            self.finished = true;
            return Ok(None);
        }
        if fetched && count == self.batch_size {
            self.width = legacy_page_width(&page, self.metric)?;
        }
        let next_remaining = self.remaining.map(|left| left.saturating_sub(count));
        self.remaining = next_remaining;
        self.cache = remaining_cache;
        self.finished = next_remaining == Some(0) || (exhausted && self.cache.is_none());
        trace_debug!(target: "milvus_sdk::iterator", kind = "search_v1", rows = count, remaining = ?self.remaining, finished = self.finished, exhausted, "completed legacy search iterator page");
        Ok(Some(page))
    }

    /// Releases iterator-local legacy search state.
    pub async fn close(&mut self) -> Result<()> {
        self.cache = None;
        self.finished = true;
        Ok(())
    }

    fn cache_row_count(&self) -> Result<usize> {
        self.cache
            .as_ref()
            .map_or(Ok(0), response::dql::SearchResponse::row_count)
    }

    async fn fetch_page(&self, coefficient: usize) -> Result<response::dql::SearchResponse> {
        let mut request = self.request.clone();
        request.dsl = self.next_filter();
        let coefficient = coefficient.max(1) as f64;
        let next_radius = if metric_distance_increases(self.metric) {
            let next = self.tail_band + self.width * coefficient;
            self.requested_radius
                .map_or(next, |radius| next.min(radius))
        } else {
            let next = self.tail_band - self.width * coefficient;
            self.requested_radius
                .map_or(next, |radius| next.max(radius))
        };
        set_search_numeric_param(&mut request.search_params, "radius", next_radius);
        set_search_numeric_param(&mut request.search_params, "range_filter", self.tail_band);
        let extended = self
            .batch_size
            .saturating_mul(LEGACY_SEARCH_EXTENSION_RATE)
            .min(MAX_BATCH_SIZE);
        let extended = self.ef.map_or(extended, |ef| extended.min(ef));
        set_param(&mut request.search_params, "topk", extended.to_string());

        let mut raw = rpc_with_retry!(self.client, search, request)?;
        status_to_result(&raw.status)?;
        if let Some(results) = &mut raw.results {
            if results.primary_field_name.is_empty() {
                results.primary_field_name = self.primary_field_name.clone();
            }
        }
        response::dql::SearchResponse::from_proto(raw)
    }

    fn next_filter(&self) -> String {
        if self.filtered_ids.is_empty() {
            return self.original_filter.clone();
        }
        let exclusion = format!(
            "{} not in [{}]",
            self.primary_field_name,
            self.filtered_ids.expression_values()
        );
        if self.original_filter.is_empty() {
            exclusion
        } else {
            format!("({}) and {exclusion}", self.original_filter)
        }
    }

    fn update_legacy_cursor(&mut self, page: &response::dql::SearchResponse) -> Result<()> {
        let result = page.results().get_results().first().ok_or_else(|| {
            Error::MalformedResponse("legacy search iterator page has no result".into())
        })?;
        let last_distance = *result.get_scores().last().ok_or_else(|| {
            Error::MalformedResponse("legacy search iterator page has no scores".into())
        })?;
        if self.filtered_distance != Some(last_distance) {
            self.filtered_ids.clear();
            self.filtered_distance = Some(last_distance);
        }
        self.filtered_ids.extend_equal_score(
            result.get_ids(),
            result.get_scores(),
            last_distance,
        )?;
        if self.filtered_ids.len() > LEGACY_SEARCH_MAX_FILTERED_IDS {
            return Err(Error::MalformedResponse(format!(
                "legacy search iterator accumulated more than {LEGACY_SEARCH_MAX_FILTERED_IDS} equal-distance primary keys"
            )));
        }
        self.tail_band = f64::from(last_distance);
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
// SearchIteratorV2
///////////////////////////////////////////////////////////////////////////////
/// Token/bound-based iterator supported by newer Milvus servers.
pub struct SearchIteratorV2 {
    client: ClientV2,
    request: milvus::SearchRequest,
    batch_size: usize,
    remaining: Option<usize>,
    token: Option<String>,
    primary_field_name: String,
    finished: bool,
    closed: Option<Arc<AtomicBool>>,
}

impl SearchIteratorV2 {
    fn finished(client: ClientV2, batch_size: usize) -> Self {
        Self {
            client,
            request: milvus::SearchRequest::default(),
            batch_size,
            remaining: Some(0),
            token: None,
            primary_field_name: String::new(),
            finished: true,
            closed: None,
        }
    }

    fn ensure_session_open(&self) -> Result<()> {
        if self
            .closed
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            Err(Error::Unexpected(
                "MilvusClientV2 session is closed".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    /// Retrieves the next token-based search-result batch, or `None` when iteration is complete.
    pub async fn next(&mut self) -> Result<Option<response::dql::SearchResponse>> {
        self.ensure_session_open()?;
        if self.finished || self.remaining == Some(0) {
            return Ok(None);
        }
        let size = self
            .remaining
            .map_or(self.batch_size, |left| left.min(self.batch_size));
        trace_debug!(target: "milvus_sdk::iterator", kind = "search_v2", batch_size = size, remaining = ?self.remaining, has_token = self.token.is_some(), "requesting token-based search iterator page");
        let mut request = self.request.clone();
        set_param(
            &mut request.search_params,
            "topk",
            self.batch_size.to_string(),
        );
        set_search_extra_param(&mut request.search_params, "iterator", "True");
        set_search_extra_param(&mut request.search_params, "search_iter_v2", "True");
        set_search_extra_param(
            &mut request.search_params,
            "search_iter_batch_size",
            &self.batch_size.to_string(),
        );
        if let Some(token) = &self.token {
            set_search_extra_param(&mut request.search_params, "search_iter_id", token);
        }
        let mut raw = rpc_with_retry!(self.client, search, request.clone())?;
        status_to_result(&raw.status)?;
        let (iterator_token, iterator_bound) = {
            let iterator = search_iterator_v2_metadata(&raw)?;
            (iterator.token.clone(), iterator.last_bound)
        };
        let next_token = self.token.clone().unwrap_or(iterator_token);
        set_search_extra_param(
            &mut request.search_params,
            "search_iter_last_bound",
            &format_iterator_bound(iterator_bound),
        );
        if search_iterator_result_count(&raw) == Some(0) {
            trace_debug!(target: "milvus_sdk::iterator", kind = "search_v2", "token-based search iterator reached end of results");
            self.finished = true;
            return Ok(None);
        }
        if let Some(results) = &mut raw.results {
            if results.primary_field_name.is_empty() {
                results.primary_field_name = self.primary_field_name.clone();
            }
        }
        let response = response::dql::SearchResponse::from_proto_with_row_limit(raw, Some(size))?;
        if response.results().len() != 1 {
            return Err(Error::MalformedResponse(
                "search iterator server response must contain exactly one result".into(),
            ));
        }
        let count = response.results().get_results()[0].len();
        if count == 0 {
            self.finished = true;
            return Ok(None);
        }
        let next_remaining = self.remaining.map(|left| left.saturating_sub(count));
        self.request = request;
        self.token = Some(next_token);
        self.remaining = next_remaining;
        trace_debug!(target: "milvus_sdk::iterator", kind = "search_v2", rows = count, remaining = ?self.remaining, "completed token-based search iterator page");
        Ok(Some(response))
    }

    /// Releases token-based search iterator state.
    pub async fn close(&mut self) -> Result<()> {
        self.finished = true;
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
// LegacyFilteredIds
///////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
enum LegacyFilteredIds {
    Int64(Vec<i64>),
    VarChar(Vec<String>),
}

impl LegacyFilteredIds {
    fn from_ids(ids: &crate::v2::Ids) -> Self {
        match ids {
            crate::v2::Ids::Int64(_) => Self::Int64(Vec::new()),
            crate::v2::Ids::VarChar(_) => Self::VarChar(Vec::new()),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Int64(values) => values.len(),
            Self::VarChar(values) => values.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self) {
        match self {
            Self::Int64(values) => values.clear(),
            Self::VarChar(values) => values.clear(),
        }
    }

    fn extend_equal_score(
        &mut self,
        ids: &crate::v2::Ids,
        scores: &[f32],
        target: f32,
    ) -> Result<()> {
        if ids.len() != scores.len() {
            return Err(Error::MalformedResponse(
                "legacy search iterator IDs and scores have different lengths".into(),
            ));
        }
        match (self, ids) {
            (Self::Int64(filtered), crate::v2::Ids::Int64(values)) => filtered.extend(
                values
                    .iter()
                    .zip(scores)
                    .filter_map(|(value, score)| (*score == target).then_some(*value)),
            ),
            (Self::VarChar(filtered), crate::v2::Ids::VarChar(values)) => filtered.extend(
                values
                    .iter()
                    .zip(scores)
                    .filter_map(|(value, score)| (*score == target).then_some(value.clone())),
            ),
            _ => {
                return Err(Error::MalformedResponse(
                    "legacy search iterator primary-key type changed between pages".into(),
                ))
            }
        }
        Ok(())
    }

    fn expression_values(&self) -> String {
        match self {
            Self::Int64(values) => values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
            Self::VarChar(values) => values
                .iter()
                .map(|value| {
                    serde_json::to_string(value).expect("strings always serialize to JSON")
                })
                .collect::<Vec<_>>()
                .join(","),
        }
    }
}

impl ClientV2 {
    /// Creates an iterator that retrieves query results in bounded batches.
    ///
    /// Pagination uses the collection primary key as a cursor and preserves the request's
    /// consistency semantics across pages. Query iterators do not accept primary-key ID input.
    pub async fn query_iterator(
        &self,
        request: request::dql::QueryIteratorRequest,
    ) -> Result<QueryIterator> {
        self.query_iterator_with_cluster(request, "").await
    }

    pub(super) async fn query_iterator_with_cluster(
        &self,
        request: request::dql::QueryIteratorRequest,
        cluster_id: &str,
    ) -> Result<QueryIterator> {
        let request::dql::QueryIteratorRequest {
            query,
            batch_size,
            reduce_stop_for_best,
        } = request;
        validate_batch_size(batch_size)?;
        if query.limit == Some(0) {
            return Ok(QueryIterator::finished(self.clone(), batch_size));
        }
        let database = self.effective_database(query.database_name.as_deref());
        let collection = query.collection_name.clone();
        let description = self
            .get_collection_description(&database, &collection)
            .await?;
        if description.collection_id <= 0 {
            return Err(Error::MalformedResponse(
                "query iterator collection has no valid collection ID".into(),
            ));
        }
        let collection_schema = description.schema.as_ref().ok_or_else(|| {
            Error::MalformedResponse("collection description has no schema".into())
        })?;
        let primary_field = collection_schema
            .fields
            .iter()
            .find(|field| field.is_primary_key)
            .ok_or_else(|| {
                Error::MalformedResponse(
                    "query iterator collection has no primary-key field".into(),
                )
            })?;
        let primary_field_type = data_type_from_proto(primary_field.data_type)?;
        if !matches!(primary_field_type, DataType::Int64 | DataType::VarChar) {
            return Err(Error::MalformedResponse(
                "query iterator primary key must be Int64 or VarChar".into(),
            ));
        }
        let guarantee = self
            .deduce_guarantee_timestamp(&database, &collection, query.consistency_level)
            .await?;
        let offset = query.offset.unwrap_or(0).max(0) as usize;
        let remaining = match query.limit {
            Some(value) if value >= 0 => Some(value as usize),
            Some(_) | None => None,
        };
        let mut raw = query.into_proto(&database, None, guarantee)?;
        set_cluster_param(&mut raw.query_params, cluster_id);
        let original_filter = raw.expr.clone();
        set_param(&mut raw.query_params, "offset", "0".into());
        set_param(
            &mut raw.query_params,
            "collection_id",
            description.collection_id.to_string(),
        );
        set_param(
            &mut raw.query_params,
            "reduce_stop_for_best",
            if reduce_stop_for_best {
                "True"
            } else {
                "False"
            }
            .into(),
        );

        let mut probe = raw.clone();
        probe.output_fields.clear();
        probe.partition_names.clear();
        set_param(&mut probe.query_params, "limit", "1".into());
        set_param(&mut probe.query_params, "iterator", "true".into());
        let probe_response = rpc_with_retry!(self, query, probe)?;
        status_to_result(&probe_response.status)?;
        raw.guarantee_timestamp = iterator_session_timestamp(probe_response.session_ts);

        let mut iterator = QueryIterator {
            client: self.clone(),
            request: raw,
            batch_size,
            remaining,
            original_filter,
            primary_field_name: primary_field.name.clone(),
            primary_field_type,
            cursor: None,
            cache: None,
            finished: false,
            closed: None,
        };
        iterator.seek_to_offset(offset).await?;
        Ok(iterator)
    }

    /// Creates an iterator that retrieves search results in batches while preserving the server
    /// search token/bound and one MVCC session timestamp across pages.
    pub async fn search_iterator(
        &self,
        request: request::dql::SearchIteratorRequest,
    ) -> Result<SearchIterator> {
        self.search_iterator_with_cluster(request, "").await
    }

    pub(super) async fn search_iterator_with_cluster(
        &self,
        mut request: request::dql::SearchIteratorRequest,
        cluster_id: &str,
    ) -> Result<SearchIterator> {
        validate_batch_size(request.batch_size)?;
        if request.limit == Some(0) {
            return Ok(SearchIterator::V2(SearchIteratorV2::finished(
                self.clone(),
                request.batch_size,
            )));
        }
        let database = self.effective_database(request.search.database_name.as_deref());
        let collection = request.search.collection_name.clone();
        let description = self
            .describe_collection_uncached(&database, &collection)
            .await?;
        let direct_info = search_iterator_collection_info(&description)?;
        let inferred_vector_field = request.search.vector_field.is_empty();
        if request.search.vector_field.is_empty() {
            request.search.vector_field = single_search_iterator_vector_field(&direct_info)?;
        }
        let requested_metric = request
            .search
            .metric_type
            .filter(|metric| *metric != MetricType::Default);
        // Search Iterator V2 lets the server deduce an omitted metric. Resolve a
        // concrete index metric only if the server falls back to the legacy
        // range-search iterator, which needs it to advance distance bounds.
        validate_search_iterator_input(&request.search, request.batch_size)?;
        let batch_size = request.batch_size;
        let remaining = request.limit;
        let mut vector_field = request.search.vector_field.clone();
        let requested_radius = request.search.radius;
        let requested_range_filter = request.search.range_filter;
        let ef = request
            .search
            .extra_params
            .get("ef")
            .and_then(|value| value.parse::<usize>().ok());
        request.search.limit = batch_size as i64;
        let consistency_level = request.search.consistency_level;
        let mut raw = request.search.into_proto(&database, 0)?;
        set_cluster_param(&mut raw.search_params, cluster_id);
        if raw.nq != 1 {
            return Err(Error::validation(
                "vectors".into(),
                "search iterator requires exactly one query vector".into(),
            ));
        }
        set_search_extra_param(
            &mut raw.search_params,
            "collection_id",
            &direct_info.collection_id.to_string(),
        );
        set_search_extra_param(&mut raw.search_params, "iterator", "True");
        set_param(&mut raw.search_params, "topk", batch_size.to_string());
        let original_filter = raw.dsl.clone();

        let mut v2_request = raw.clone();
        set_search_extra_param(&mut v2_request.search_params, "search_iter_v2", "True");
        set_search_extra_param(
            &mut v2_request.search_params,
            "search_iter_batch_size",
            &batch_size.to_string(),
        );
        let mut probe = v2_request.clone();
        probe.guarantee_timestamp = 0;
        set_param(&mut probe.search_params, "topk", "1".into());
        set_search_extra_param(&mut probe.search_params, "search_iter_batch_size", "1");
        let probe = rpc_with_retry!(self, search, probe)?;
        status_to_result(&probe.status)?;
        if search_iterator_v2_metadata(&probe).is_ok() {
            v2_request.guarantee_timestamp = iterator_session_timestamp(probe.session_ts);
            return Ok(SearchIterator::V2(SearchIteratorV2 {
                client: self.clone(),
                request: v2_request,
                batch_size,
                remaining,
                token: None,
                primary_field_name: direct_info.primary_field_name,
                finished: false,
                closed: None,
            }));
        }

        let description = self
            .get_collection_description(&database, &collection)
            .await?;
        let legacy_info = search_iterator_collection_info(&description)?;
        if inferred_vector_field {
            vector_field = single_search_iterator_vector_field(&legacy_info)?;
            set_param(&mut raw.search_params, "anns_field", vector_field.clone());
        }
        set_search_extra_param(
            &mut raw.search_params,
            "collection_id",
            &legacy_info.collection_id.to_string(),
        );

        let metric = match requested_metric {
            Some(metric) => metric,
            None => {
                self.search_iterator_metric(&database, &collection, &vector_field)
                    .await?
            }
        };
        validate_search_iterator_range(requested_radius, requested_range_filter, metric)?;
        set_param(
            &mut raw.search_params,
            "metric_type",
            metric.as_str().into(),
        );
        raw.guarantee_timestamp = self
            .deduce_guarantee_timestamp(&database, &collection, consistency_level)
            .await?;
        let mut initial = rpc_with_retry!(self, search, raw.clone())?;
        status_to_result(&initial.status)?;
        raw.guarantee_timestamp = iterator_session_timestamp(initial.session_ts);
        if let Some(results) = &mut initial.results {
            if results.primary_field_name.is_empty() {
                results.primary_field_name = legacy_info.primary_field_name.clone();
            }
        }
        let initial = response::dql::SearchResponse::from_proto(initial)?;
        let initial_count = initial.row_count()?;
        let (filtered_ids, filtered_distance, tail_band, width) = if initial_count == 0 {
            let filtered_ids = match legacy_info.primary_field_type {
                DataType::Int64 => LegacyFilteredIds::Int64(Vec::new()),
                DataType::VarChar => LegacyFilteredIds::VarChar(Vec::new()),
                _ => unreachable!("validated search iterator primary-key type"),
            };
            (filtered_ids, None, 0.0, LEGACY_SEARCH_MIN_WIDTH)
        } else {
            legacy_initial_state(&initial, metric)?
        };
        let iterator = SearchIteratorV1 {
            client: self.clone(),
            request: raw,
            batch_size,
            remaining,
            metric,
            requested_radius,
            ef,
            original_filter,
            primary_field_name: legacy_info.primary_field_name,
            filtered_ids,
            filtered_distance,
            tail_band,
            width,
            cache: (initial_count > 0).then_some(initial),
            finished: initial_count == 0,
            closed: None,
        };
        Ok(SearchIterator::V1(iterator))
    }

    async fn search_iterator_metric(
        &self,
        database: &str,
        collection: &str,
        vector_field: &str,
    ) -> Result<MetricType> {
        let response = rpc_with_retry!(
            self,
            describe_index,
            milvus::DescribeIndexRequest {
                base: None,
                db_name: database.to_owned(),
                collection_name: collection.to_owned(),
                field_name: vector_field.to_owned(),
                index_name: String::new(),
                timestamp: 0,
                ..Default::default()
            }
        )?;
        status_to_result(&response.status)?;
        let index = response
            .index_descriptions
            .into_iter()
            .find(|index| index.field_name == vector_field)
            .ok_or_else(|| {
                Error::MalformedResponse(format!(
                    "cannot determine metric type from the index on vector field {vector_field:?}"
                ))
            })?;
        let metric = IndexDesc::from_proto(index)?.get_metric_type();
        if metric == MetricType::Default {
            return Err(Error::MalformedResponse(format!(
                "cannot determine metric type from the index on vector field {vector_field:?}"
            )));
        }
        Ok(metric)
    }
}

fn search_iterator_collection_info(
    description: &milvus::DescribeCollectionResponse,
) -> Result<SearchIteratorCollectionInfo> {
    let collection_schema = description
        .schema
        .as_ref()
        .ok_or_else(|| Error::MalformedResponse("collection description has no schema".into()))?;
    let primary_field = collection_schema
        .fields
        .iter()
        .find(|field| field.is_primary_key)
        .ok_or_else(|| {
            Error::MalformedResponse("search iterator collection has no primary-key field".into())
        })?;
    let primary_field_type = data_type_from_proto(primary_field.data_type)?;
    if !matches!(primary_field_type, DataType::Int64 | DataType::VarChar) {
        return Err(Error::MalformedResponse(
            "search iterator primary key must be Int64 or VarChar".into(),
        ));
    }
    if description.collection_id <= 0 {
        return Err(Error::MalformedResponse(
            "search iterator collection has no valid collection ID".into(),
        ));
    }
    Ok(SearchIteratorCollectionInfo {
        collection_id: description.collection_id,
        primary_field_name: primary_field.name.clone(),
        primary_field_type,
        vector_field_names: vector_field_names(collection_schema),
    })
}

fn single_search_iterator_vector_field(info: &SearchIteratorCollectionInfo) -> Result<String> {
    match info.vector_field_names.as_slice() {
        [] => Err(Error::MalformedResponse(
            "search iterator collection has no vector field".into(),
        )),
        [_] => Ok(info.vector_field_names[0].clone()),
        _ => Err(Error::validation(
            "vector_field".into(),
            "must be specified when the collection has multiple vector fields".into(),
        )),
    }
}

fn validate_search_iterator_input(
    search: &request::dql::SearchRequest,
    batch_size: usize,
) -> Result<()> {
    if search.offset != 0 {
        return Err(Error::validation(
            "offset".into(),
            "search iterator does not support a non-zero offset".into(),
        ));
    }
    if let Some(ef) = search
        .extra_params
        .get("ef")
        .and_then(|value| value.parse::<usize>().ok())
    {
        if ef < batch_size {
            return Err(Error::validation(
                "ef".into(),
                "must be greater than or equal to the iterator batch size".into(),
            ));
        }
    }
    if !search.function_chains.is_empty() {
        return Err(Error::validation(
            "function_chains".into(),
            "search iterator does not support function chains".into(),
        ));
    }
    if search.search_aggregation.is_some() {
        return Err(Error::validation(
            "search_aggregation".into(),
            "search iterator does not support search aggregation".into(),
        ));
    }
    Ok(())
}

fn validate_search_iterator_range(
    radius: Option<f64>,
    range_filter: Option<f64>,
    metric: MetricType,
) -> Result<()> {
    if let (Some(radius), Some(range_filter)) = (radius, range_filter) {
        let invalid = if metric_distance_increases(metric) {
            radius <= range_filter
        } else {
            radius >= range_filter
        };
        if invalid {
            return Err(Error::validation(
                "radius".into(),
                "radius and range_filter are invalid for the selected metric".into(),
            ));
        }
    }
    Ok(())
}

fn data_type_from_proto(value: i32) -> Result<DataType> {
    let value = schema::DataType::try_from(value)
        .map_err(|_| Error::conversion(format!("unknown protobuf data type {value}")))?;
    DataType::try_from_proto(value)
}

fn vector_field_names(collection: &schema::CollectionSchema) -> Vec<String> {
    let mut names = collection
        .fields
        .iter()
        .filter_map(|field| {
            data_type_from_proto(field.data_type)
                .ok()
                .filter(|data_type| data_type.is_vector())
                .map(|_| field.name.clone())
        })
        .collect::<Vec<_>>();
    for struct_field in &collection.struct_array_fields {
        for field in &struct_field.fields {
            if data_type_from_proto(field.data_type).is_ok_and(|data_type| data_type.is_vector()) {
                names.push(format!("{}[{}]", struct_field.name, field.name));
            }
        }
    }
    names
}

fn metric_distance_increases(metric: MetricType) -> bool {
    matches!(
        metric,
        MetricType::L2
            | MetricType::Jaccard
            | MetricType::MhJaccard
            | MetricType::Hamming
            | MetricType::MaxSimL2
            | MetricType::MaxSimJaccard
            | MetricType::MaxSimHamming
    )
}

fn metric_distance_is_integer(metric: MetricType) -> bool {
    matches!(metric, MetricType::Hamming | MetricType::MaxSimHamming)
}

fn legacy_initial_state(
    page: &response::dql::SearchResponse,
    metric: MetricType,
) -> Result<(LegacyFilteredIds, Option<f32>, f64, f64)> {
    let result = page.results().get_results().first().ok_or_else(|| {
        Error::MalformedResponse("legacy search iterator page has no result".into())
    })?;
    let last_distance = *result.get_scores().last().ok_or_else(|| {
        Error::MalformedResponse("legacy search iterator page has no scores".into())
    })?;
    let mut filtered_ids = LegacyFilteredIds::from_ids(result.get_ids());
    filtered_ids.extend_equal_score(result.get_ids(), result.get_scores(), last_distance)?;
    Ok((
        filtered_ids,
        Some(last_distance),
        f64::from(last_distance),
        legacy_page_width(page, metric)?,
    ))
}

fn legacy_page_width(page: &response::dql::SearchResponse, metric: MetricType) -> Result<f64> {
    let scores = page
        .results()
        .get_results()
        .first()
        .ok_or_else(|| {
            Error::MalformedResponse("legacy search iterator page has no result".into())
        })?
        .get_scores();
    let first = *scores.first().ok_or_else(|| {
        Error::MalformedResponse("legacy search iterator page has no scores".into())
    })?;
    let last = *scores.last().ok_or_else(|| {
        Error::MalformedResponse("legacy search iterator page has no scores".into())
    })?;
    let width = if metric_distance_increases(metric) {
        f64::from(last - first)
    } else {
        f64::from(first - last)
    };
    let minimum = if metric_distance_is_integer(metric) {
        1.0
    } else {
        LEGACY_SEARCH_MIN_WIDTH
    };
    Ok(width.max(minimum))
}

fn iterator_session_timestamp(server_timestamp: u64) -> u64 {
    if server_timestamp != 0 {
        return server_timestamp;
    }
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    hybrid_timestamp_from_millis(
        u64::try_from(millis).unwrap_or(u64::MAX >> HYBRID_TIMESTAMP_LOGICAL_BITS),
    )
}

fn hybrid_timestamp_from_millis(millis: u64) -> u64 {
    millis
        .saturating_add(ITERATOR_FALLBACK_OFFSET_MILLIS)
        .min(u64::MAX >> HYBRID_TIMESTAMP_LOGICAL_BITS)
        << HYBRID_TIMESTAMP_LOGICAL_BITS
}

fn search_iterator_v2_metadata(
    response: &milvus::SearchResults,
) -> Result<&schema::SearchIteratorV2Results> {
    response
        .results
        .as_ref()
        .and_then(|results| results.search_iterator_v2_results.as_ref())
        .filter(|iterator| !iterator.token.is_empty())
        .ok_or_else(|| {
            Error::MalformedResponse(
                "server does not provide a Search Iterator V2 token; Milvus 2.5.2 or later is required"
                    .into(),
            )
        })
}

fn search_iterator_result_count(response: &milvus::SearchResults) -> Option<usize> {
    let results = response.results.as_ref()?;
    if results.num_queries != 1 {
        return None;
    }
    usize::try_from(*results.topks.first()?).ok()
}

fn format_iterator_bound(bound: f32) -> String {
    format!("{:.15}", f64::from(bound))
}

fn validate_batch_size(batch_size: usize) -> Result<()> {
    if batch_size == 0 || batch_size > MAX_BATCH_SIZE {
        return Err(Error::validation(
            "batch_size".into(),
            format!("must be between 1 and {MAX_BATCH_SIZE}"),
        ));
    }
    Ok(())
}

fn set_param(params: &mut Vec<common::KeyValuePair>, key: &str, value: String) {
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

fn set_search_extra_param(params: &mut Vec<common::KeyValuePair>, key: &str, value: &str) {
    set_param(params, key, value.to_owned());
    let mut nested = params
        .iter()
        .find(|param| param.key == "params")
        .and_then(|param| serde_json::from_str::<serde_json::Value>(&param.value).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    nested.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
    set_param(
        params,
        "params",
        serde_json::Value::Object(nested).to_string(),
    );
}

fn set_search_numeric_param(params: &mut Vec<common::KeyValuePair>, key: &str, value: f64) {
    set_param(params, key, value.to_string());
    let mut nested = params
        .iter()
        .find(|param| param.key == "params")
        .and_then(|param| serde_json::from_str::<serde_json::Value>(&param.value).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    nested.insert(key.to_owned(), serde_json::json!(value));
    set_param(
        params,
        "params",
        serde_json::Value::Object(nested).to_string(),
    );
}

fn query_row_count(response: &milvus::QueryResults) -> usize {
    response
        .fields_data
        .first()
        .map(field_row_count)
        .unwrap_or(0)
}

fn query_response_row_count(response: &response::dql::QueryResponse) -> usize {
    usize::try_from(response.results().get_row_count()).unwrap_or(usize::MAX)
}

fn query_response_cursor(
    response: &response::dql::QueryResponse,
    primary_field_name: &str,
    data_type: DataType,
) -> Result<QueryCursor> {
    let field = response
        .results()
        .get_output_field(primary_field_name)
        .ok_or_else(|| {
            Error::MalformedResponse(format!(
                "primary-key field {primary_field_name:?} is missing from query iterator response"
            ))
        })?;
    match (data_type, field.inner()) {
        (DataType::Int64, crate::v2::FieldData::Int64 { values, .. }) => values
            .last()
            .copied()
            .map(QueryCursor::Int64)
            .ok_or_else(|| {
                Error::MalformedResponse(format!(
                    "primary-key field {primary_field_name:?} has no values"
                ))
            }),
        (DataType::VarChar, crate::v2::FieldData::VarChar { values, .. }) => values
            .last()
            .cloned()
            .map(QueryCursor::VarChar)
            .ok_or_else(|| {
                Error::MalformedResponse(format!(
                    "primary-key field {primary_field_name:?} has no values"
                ))
            }),
        _ => Err(Error::MalformedResponse(format!(
            "primary-key field {primary_field_name:?} does not match its collection schema"
        ))),
    }
}

fn query_proto_cursor(
    response: &milvus::QueryResults,
    primary_field_name: &str,
    data_type: DataType,
) -> Result<QueryCursor> {
    let field = response
        .fields_data
        .iter()
        .find(|field| field.field_name == primary_field_name)
        .ok_or_else(|| {
            Error::MalformedResponse(format!(
                "primary-key field {primary_field_name:?} is missing from query iterator response"
            ))
        })?;
    query_proto_field_cursor(field, data_type)
}

fn query_proto_field_cursor(field: &schema::FieldData, data_type: DataType) -> Result<QueryCursor> {
    use schema::{field_data, scalar_field};

    let scalar = match field.field.as_ref() {
        Some(field_data::Field::Scalars(value)) => value,
        _ => {
            return Err(Error::MalformedResponse(format!(
                "primary-key field {:?} has no scalar data",
                field.field_name
            )))
        }
    };
    match (data_type, scalar.data.as_ref()) {
        (DataType::Int64, Some(scalar_field::Data::LongData(values))) => values
            .data
            .last()
            .copied()
            .map(QueryCursor::Int64)
            .ok_or_else(|| {
                Error::MalformedResponse(format!(
                    "primary-key field {:?} has no values",
                    field.field_name
                ))
            }),
        (DataType::VarChar, Some(scalar_field::Data::StringData(values))) => values
            .data
            .last()
            .cloned()
            .map(QueryCursor::VarChar)
            .ok_or_else(|| {
                Error::MalformedResponse(format!(
                    "primary-key field {:?} has no values",
                    field.field_name
                ))
            }),
        _ => Err(Error::MalformedResponse(format!(
            "primary-key field {:?} does not match its collection schema",
            field.field_name
        ))),
    }
}

fn field_row_count(field: &schema::FieldData) -> usize {
    use schema::{field_data, scalar_field, vector_field};
    if !field.valid_data.is_empty() {
        return field.valid_data.len();
    }
    match field.field.as_ref() {
        Some(field_data::Field::Scalars(scalar)) => match scalar.data.as_ref() {
            Some(scalar_field::Data::BoolData(values)) => values.data.len(),
            Some(scalar_field::Data::IntData(values)) => values.data.len(),
            Some(scalar_field::Data::LongData(values)) => values.data.len(),
            Some(scalar_field::Data::FloatData(values)) => values.data.len(),
            Some(scalar_field::Data::DoubleData(values)) => values.data.len(),
            Some(scalar_field::Data::StringData(values)) => values.data.len(),
            Some(scalar_field::Data::BytesData(values)) => values.data.len(),
            Some(scalar_field::Data::ArrayData(values)) => values.data.len(),
            Some(scalar_field::Data::JsonData(values)) => values.data.len(),
            Some(scalar_field::Data::GeometryData(values)) => values.data.len(),
            Some(scalar_field::Data::TimestamptzData(values)) => values.data.len(),
            Some(scalar_field::Data::GeometryWktData(values)) => values.data.len(),
            Some(scalar_field::Data::MolData(values)) => values.data.len(),
            Some(scalar_field::Data::MolSmilesData(values)) => values.data.len(),
            Some(scalar_field::Data::DateData(values)) => values.data.len(),
            Some(scalar_field::Data::TimeData(values)) => values.data.len(),
            None => 0,
        },
        Some(field_data::Field::Vectors(vector)) => match vector.data.as_ref() {
            Some(vector_field::Data::FloatVector(values)) if vector.dim > 0 => {
                values.data.len() / vector.dim as usize
            }
            Some(vector_field::Data::BinaryVector(values)) if vector.dim > 0 => {
                values.len() * 8 / vector.dim as usize
            }
            Some(vector_field::Data::Float16Vector(values))
            | Some(vector_field::Data::Bfloat16Vector(values))
                if vector.dim > 0 =>
            {
                values.len() / (vector.dim as usize * 2)
            }
            Some(vector_field::Data::Int8Vector(values)) if vector.dim > 0 => {
                values.len() / vector.dim as usize
            }
            Some(vector_field::Data::SparseFloatVector(values)) => values.contents.len(),
            Some(vector_field::Data::VectorArray(values)) => values.data.len(),
            _ => 0,
        },
        Some(field_data::Field::StructArrays(values)) => {
            values.fields.first().map(field_row_count).unwrap_or(0)
        }
        None => 0,
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod search_iterator_v2_tests {
    use super::{
        format_iterator_bound, hybrid_timestamp_from_millis, search_iterator_result_count,
        search_iterator_v2_metadata, set_search_extra_param, set_search_numeric_param,
        validate_search_iterator_input, validate_search_iterator_range, LegacyFilteredIds,
    };
    use crate::proto::{common, milvus, schema};
    use crate::v2::request::dql::{SearchRequest, SearchVectors};
    use crate::v2::types::FunctionChain;
    use crate::v2::MetricType;

    #[test]
    fn last_bound_uses_double_precision_wire_text() {
        assert_eq!(format_iterator_bound(0.1), "0.100000001490116");
        assert_eq!(format_iterator_bound(-1.25), "-1.250000000000000");
    }

    #[test]
    fn fallback_timestamp_is_one_second_after_client_time() {
        assert_eq!(hybrid_timestamp_from_millis(1_234), 2_234_u64 << 18);
    }

    #[test]
    fn legacy_varchar_primary_keys_are_escaped_in_exclusion_filters() {
        let ids = LegacyFilteredIds::VarChar(vec!["plain".into(), "quoted\"value".into()]);
        assert_eq!(ids.expression_values(), r#""plain","quoted\"value""#);
    }

    #[test]
    fn legacy_range_parameters_are_numeric_in_nested_search_params() {
        let mut params = Vec::new();
        set_search_numeric_param(&mut params, "radius", 0.75);
        set_search_numeric_param(&mut params, "range_filter", 0.5);

        let nested: serde_json::Value = serde_json::from_str(
            &params
                .iter()
                .find(|param| param.key == "params")
                .expect("nested search parameters")
                .value,
        )
        .expect("valid nested search parameters");
        assert_eq!(nested["radius"], 0.75);
        assert_eq!(nested["range_filter"], 0.5);
    }

    #[test]
    fn v2_metadata_requires_a_non_empty_server_token() {
        let response = |token: &str| milvus::SearchResults {
            results: Some(schema::SearchResultData {
                search_iterator_v2_results: Some(schema::SearchIteratorV2Results {
                    token: token.into(),
                    last_bound: 0.5,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            search_iterator_v2_metadata(&response("iterator-token"))
                .unwrap()
                .token,
            "iterator-token"
        );
        assert!(search_iterator_v2_metadata(&response("")).is_err());
        assert!(search_iterator_v2_metadata(&milvus::SearchResults::default()).is_err());
    }

    #[test]
    fn zero_row_terminal_page_is_detected_before_field_decoding() {
        let response = milvus::SearchResults {
            results: Some(schema::SearchResultData {
                num_queries: 1,
                topks: vec![0],
                fields_data: vec![schema::FieldData {
                    r#type: schema::DataType::Int16 as i32,
                    field_name: "age".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(search_iterator_result_count(&response), Some(0));
    }

    #[test]
    fn v2_control_params_are_written_for_new_and_compatible_servers() {
        let mut params = vec![common::KeyValuePair {
            key: "params".into(),
            value: r#"{"nprobe":"16"}"#.into(),
            ..Default::default()
        }];
        set_search_extra_param(&mut params, "search_iter_v2", "True");

        assert!(params
            .iter()
            .any(|param| param.key == "search_iter_v2" && param.value == "True"));
        let nested = params
            .iter()
            .find(|param| param.key == "params")
            .and_then(|param| serde_json::from_str::<serde_json::Value>(&param.value).ok())
            .expect("nested search parameters");
        assert_eq!(nested["nprobe"], "16");
        assert_eq!(nested["search_iter_v2"], "True");
    }

    #[test]
    fn v2_input_rejects_offset_and_ef_while_legacy_validates_metric_range() {
        let mut request = SearchRequest::builder()
            .collection_name("books")
            .vectors(SearchVectors::Float(vec![vec![0.0]]))
            .offset(1)
            .build()
            .expect("valid request");
        assert!(validate_search_iterator_input(&request, 100).is_err());

        request.offset = 0;
        request.extra_params.insert("ef".into(), "99".into());
        assert!(validate_search_iterator_input(&request, 100).is_err());

        request.extra_params.clear();
        request.radius = Some(0.2);
        request.range_filter = Some(0.5);
        assert!(validate_search_iterator_input(&request, 100).is_ok());
        assert!(validate_search_iterator_range(
            request.radius,
            request.range_filter,
            MetricType::L2
        )
        .is_err());

        request.radius = Some(0.8);
        assert!(validate_search_iterator_range(
            request.radius,
            request.range_filter,
            MetricType::L2
        )
        .is_ok());
    }

    #[test]
    fn v2_input_rejects_function_chains_and_search_aggregation() {
        let mut request = SearchRequest::builder()
            .collection_name("books")
            .vectors(SearchVectors::Float(vec![vec![0.0]]))
            .build()
            .expect("valid request");
        assert!(validate_search_iterator_input(&request, 100).is_ok());

        request
            .function_chains
            .push(FunctionChain::new().stage(crate::v2::types::FunctionChainStage::L2Rerank));
        assert!(validate_search_iterator_input(&request, 100).is_err());
        request.function_chains.clear();

        request.search_aggregation = Some(
            crate::v2::types::SearchAggregation::new()
                .fields(["category"])
                .size(10),
        );
        assert!(validate_search_iterator_input(&request, 100).is_err());
    }
}
