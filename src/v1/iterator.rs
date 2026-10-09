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

use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;

use prost::Message;
use sha2::{Digest, Sha256};

use crate::iterator_cursor::{self, CursorMode};
use crate::proto::common::{KeyValuePair, MsgBase, MsgType};
use crate::proto::milvus::{QueryCursor, QueryRequest};
use crate::proto::schema::DataType;
use crate::v1::client::{Client, ConsistencyLevel};
use crate::v1::data::{slice_field_columns, FieldColumn};
use crate::v1::error::status_to_result;
use crate::v1::error::*;
use crate::v1::value::{Value, ValueVec};

// Constants
const MILVUS_LIMIT: &str = "limit";
const ITERATOR_FIELD: &str = "iterator";
const OFFSET: &str = "offset";
const GUARANTEE_TIMESTAMP: &str = "guarantee_timestamp";
const MAX_BATCH_SIZE: usize = 16384;
const NO_CACHE_ID: i32 = -1;

// Global iterator cache singleton
lazy_static::lazy_static! {
    static ref ITERATOR_CACHE: Arc<Mutex<IteratorCache>> = Arc::new(Mutex::new(IteratorCache::new()));
}

/// Iterator cache implementation
struct IteratorCache {
    cache_id: i32,
    cache_map: HashMap<i32, Vec<Vec<FieldColumn>>>,
}

impl IteratorCache {
    fn new() -> Self {
        Self {
            cache_id: 0,
            cache_map: HashMap::new(),
        }
    }

    fn cache(&mut self, result: Vec<Vec<FieldColumn>>, cache_id: i32) -> i32 {
        let mut id = cache_id;
        if id == NO_CACHE_ID {
            self.cache_id += 1;
            id = self.cache_id;
        }
        self.cache_map.insert(id, result);
        id
    }

    fn fetch_cache(&self, cache_id: i32) -> Option<Vec<Vec<FieldColumn>>> {
        self.cache_map.get(&cache_id).cloned()
    }

    fn release_cache(&mut self, cache_id: i32) {
        self.cache_map.remove(&cache_id);
    }
}

/// Options for query_iterator operation
#[derive(Debug, Clone)]
pub struct QueryIteratorOptions {
    pub batch_size: Option<usize>,
    pub limit: Option<usize>,
    pub filter: String,
    pub output_fields: Vec<String>,
    pub partition_names: Vec<String>,
    pub namespace: Option<String>,
    pub timeout: Option<f64>,
    pub consistency_level: Option<i32>,
    pub guarantee_timestamp: Option<u64>,
    pub graceful_time: Option<u64>,
    pub offset: Option<i64>,
    pub expr_template_values: HashMap<String, crate::proto::schema::TemplateValue>,
    pub iterator_cp_file: Option<String>,
    pub reduce_stop_for_best: Option<bool>,
}

impl Default for QueryIteratorOptions {
    fn default() -> Self {
        Self {
            batch_size: Some(1000),
            limit: None, // UNLIMITED
            filter: "".to_string(),
            output_fields: Vec::new(),
            partition_names: Vec::new(),
            namespace: None,
            timeout: None,
            consistency_level: None,
            guarantee_timestamp: None,
            graceful_time: None,
            offset: Some(0),
            expr_template_values: HashMap::new(),
            iterator_cp_file: None,
            reduce_stop_for_best: Some(true),
        }
    }
}

impl QueryIteratorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_filter(filter: String) -> Self {
        Self::default().filter(filter)
    }

    pub fn with_batch_size(batch_size: usize) -> Self {
        Self::default().batch_size(batch_size)
    }

    pub fn with_limit(limit: usize) -> Self {
        Self::default().limit(limit)
    }

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn filter(mut self, filter: String) -> Self {
        self.filter = filter;
        self
    }

    pub fn output_fields(mut self, output_fields: Vec<String>) -> Self {
        self.output_fields = output_fields;
        self
    }

    pub fn partition_names(mut self, partition_names: Vec<String>) -> Self {
        self.partition_names = partition_names;
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn timeout(mut self, timeout: Option<f64>) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn consistency_level(mut self, consistency_level: i32) -> Self {
        self.consistency_level = Some(consistency_level);
        self
    }

    pub fn guarantee_timestamp(mut self, guarantee_timestamp: u64) -> Self {
        self.guarantee_timestamp = Some(guarantee_timestamp);
        self
    }

    pub fn graceful_time(mut self, graceful_time: u64) -> Self {
        self.graceful_time = Some(graceful_time);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn add_template_value(
        mut self,
        key: String,
        value: crate::proto::schema::TemplateValue,
    ) -> Self {
        self.expr_template_values.insert(key, value);
        self
    }

    pub fn iterator_cp_file(mut self, cp_file: Option<String>) -> Self {
        self.iterator_cp_file = cp_file;
        self
    }

    pub fn reduce_stop_for_best(mut self, reduce_stop: bool) -> Self {
        self.reduce_stop_for_best = Some(reduce_stop);
        self
    }
}

/// Options for search_iterator operation
#[derive(Debug, Clone)]
pub struct SearchIteratorOptions {
    pub batch_size: Option<usize>,
    pub limit: Option<usize>,
    pub filter: String,
    pub output_fields: Vec<String>,
    pub partition_names: Vec<String>,
    pub namespace: Option<String>,
    pub timeout: Option<f64>,
    pub consistency_level: Option<i32>,
    pub guarantee_timestamp: Option<u64>,
    pub graceful_time: Option<u64>,
    pub offset: Option<i64>,
    pub expr_template_values: HashMap<String, crate::proto::schema::TemplateValue>,
    /// Optional checkpoint path. Negotiated PK cursors use a versioned full-state
    /// record; historical UUID-only records remain readable but cannot provide
    /// exact distance/PK resume. New records keep a trailing UUID readable by older
    /// SDKs, which continue their historical UUID-only resume semantics. Full PK
    /// records are bound to the resolved collection ID and the search definition.
    pub iterator_cp_file: Option<String>,
    pub reduce_stop_for_best: Option<bool>,
    pub anns_field: Option<String>,
    pub search_params: HashMap<String, String>,
    pub round_decimal: Option<i32>,
}

impl Default for SearchIteratorOptions {
    fn default() -> Self {
        Self {
            batch_size: Some(1000),
            limit: None, // UNLIMITED
            filter: "".to_string(),
            output_fields: Vec::new(),
            partition_names: Vec::new(),
            namespace: None,
            timeout: None,
            consistency_level: None,
            guarantee_timestamp: None,
            graceful_time: None,
            offset: Some(0),
            expr_template_values: HashMap::new(),
            iterator_cp_file: None,
            reduce_stop_for_best: Some(true),
            anns_field: None,
            search_params: HashMap::new(),
            round_decimal: Some(-1),
        }
    }
}

impl SearchIteratorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_filter(filter: String) -> Self {
        Self::default().filter(filter)
    }

    pub fn with_batch_size(batch_size: usize) -> Self {
        Self::default().batch_size(batch_size)
    }

    pub fn with_limit(limit: usize) -> Self {
        Self::default().limit(limit)
    }

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn filter(mut self, filter: String) -> Self {
        self.filter = filter;
        self
    }

    pub fn output_fields(mut self, output_fields: Vec<String>) -> Self {
        self.output_fields = output_fields;
        self
    }

    pub fn partition_names(mut self, partition_names: Vec<String>) -> Self {
        self.partition_names = partition_names;
        self
    }

    pub fn timeout(mut self, timeout: Option<f64>) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn consistency_level(mut self, consistency_level: i32) -> Self {
        self.consistency_level = Some(consistency_level);
        self
    }

    pub fn guarantee_timestamp(mut self, guarantee_timestamp: u64) -> Self {
        self.guarantee_timestamp = Some(guarantee_timestamp);
        self
    }

    pub fn graceful_time(mut self, graceful_time: u64) -> Self {
        self.graceful_time = Some(graceful_time);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn add_template_value(
        mut self,
        key: String,
        value: crate::proto::schema::TemplateValue,
    ) -> Self {
        self.expr_template_values.insert(key, value);
        self
    }

    pub fn iterator_cp_file(mut self, cp_file: Option<String>) -> Self {
        self.iterator_cp_file = cp_file;
        self
    }

    pub fn reduce_stop_for_best(mut self, reduce_stop: bool) -> Self {
        self.reduce_stop_for_best = Some(reduce_stop);
        self
    }

    pub fn anns_field(mut self, anns_field: String) -> Self {
        self.anns_field = Some(anns_field);
        self
    }

    pub fn add_search_param(mut self, key: String, value: String) -> Self {
        self.search_params.insert(key, value);
        self
    }

    pub fn round_decimal(mut self, round_decimal: i32) -> Self {
        self.round_decimal = Some(round_decimal);
        self
    }
}

impl Client {
    /// Conducts a scalar filtering with a specified boolean expression using iterator pattern.
    ///
    /// This operation performs paginated query to handle large datasets efficiently.
    /// It returns results in batches to avoid memory issues with large result sets.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to query.
    /// * `options` - Configuration for the query operation including filter, batch_size, limit, etc.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `QueryIterator` for streaming query results.
    pub async fn query_iterator<S>(
        &self,
        collection_name: S,
        options: QueryIteratorOptions,
    ) -> Result<QueryIterator>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let collection = self
            .collection_cache
            .get(
                crate::v1::collection::current_db_name(self),
                &collection_name,
            )
            .await?;

        Ok(QueryIterator::new(
            self.client.clone(),
            collection_name.to_string(),
            collection.consistency_level,
            options,
        ))
    }

    /// Conducts a vector similarity search with iterator pattern.
    ///
    /// This operation performs paginated search to handle large datasets efficiently.
    /// It returns results in batches to avoid memory issues with large result sets.
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection to search.
    /// * `data` - The vector data to search for.
    /// * `options` - Configuration for the search operation including filter, batch_size, limit, etc.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `SearchIterator` for streaming search results.
    pub async fn search_iterator<S>(
        &self,
        collection_name: S,
        data: Vec<crate::v1::value::Value<'_>>,
        options: SearchIteratorOptions,
    ) -> Result<SearchIterator>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let collection = self
            .collection_cache
            .get(
                crate::v1::collection::current_db_name(self),
                &collection_name,
            )
            .await?;

        Ok(SearchIterator::new(
            self.client.clone(),
            collection_name.to_string(),
            collection.consistency_level,
            data,
            options,
        ))
    }
}

/// Iterator for querying large datasets in batches
pub struct QueryIterator {
    client: crate::proto::milvus::milvus_service_client::MilvusServiceClient<
        tonic::service::interceptor::InterceptedService<
            tonic::transport::Channel,
            crate::v1::client::CombinedInterceptor,
        >,
    >,
    collection_name: String,
    options: QueryIteratorOptions,
    current_offset: i64,
    has_more: bool,
    current_batch: Option<Vec<FieldColumn>>,
    current_cursor: Option<QueryCursor>,
    returned_count: usize,
    next_id: Option<String>,
    pk_field_name: Option<String>,
    pk_is_string: bool,
    session_ts: u64,
    cache_id_in_use: i32,
    cp_file_handler: Option<File>,
    cp_file_path: Option<String>,
    need_save_cp: bool,
    buffer_cursor_lines_number: usize,
    collection_id: u64,
}

impl QueryIterator {
    pub fn new(
        client: crate::proto::milvus::milvus_service_client::MilvusServiceClient<
            tonic::service::interceptor::InterceptedService<
                tonic::transport::Channel,
                crate::v1::client::CombinedInterceptor,
            >,
        >,
        collection_name: String,
        _consistency_level: ConsistencyLevel,
        options: QueryIteratorOptions,
    ) -> Self {
        Self {
            client,
            collection_name,
            options,
            current_offset: 0,
            has_more: true,
            current_batch: None,
            current_cursor: None,
            returned_count: 0,
            next_id: None,
            pk_field_name: None,
            pk_is_string: false,
            session_ts: 0,
            cache_id_in_use: NO_CACHE_ID,
            cp_file_handler: None,
            cp_file_path: None,
            need_save_cp: false,
            buffer_cursor_lines_number: 0,
            collection_id: 0,
        }
    }

    async fn setup_collection_id(&mut self) -> Result<()> {
        let _res = self
            .client
            .clone()
            .describe_collection(crate::proto::milvus::DescribeCollectionRequest {
                base: Some(MsgBase::new(MsgType::DescribeCollection)),
                db_name: "".to_string(),
                collection_name: self.collection_name.clone(),
                collection_id: 0,
                time_stamp: 0,
                ..Default::default()
            })
            .await?
            .into_inner();

        // Extract collection_id from response (implementation may vary based on actual response structure)
        // For now, we'll use a placeholder
        self.collection_id = 0; // This should be extracted from the response
        Ok(())
    }

    async fn setup_pk_prop(&mut self) -> Result<()> {
        let collection = self
            .client
            .clone()
            .describe_collection(crate::proto::milvus::DescribeCollectionRequest {
                base: Some(MsgBase::new(MsgType::DescribeCollection)),
                db_name: "".to_string(),
                collection_name: self.collection_name.clone(),
                collection_id: 0,
                time_stamp: 0,
                ..Default::default()
            })
            .await?
            .into_inner();

        for field in collection.schema.unwrap().fields {
            if field.is_primary_key {
                self.pk_field_name = Some(field.name.clone());
                self.pk_is_string = field.data_type == DataType::VarChar as i32;
                break;
            }
        }

        if self.pk_field_name.is_none() {
            return Err(Error::Schema(crate::schema::Error::NoPrimaryKey));
        }

        Ok(())
    }

    fn setup_expr(&mut self) -> String {
        if !self.options.filter.is_empty() {
            self.options.filter.clone()
        } else if self.pk_is_string {
            format!("{} != \"\"", self.pk_field_name.as_ref().unwrap())
        } else {
            format!("{} < {}", self.pk_field_name.as_ref().unwrap(), i64::MAX)
        }
    }

    async fn setup_session_ts(&mut self) -> Result<()> {
        let mut init_ts_params = vec![
            KeyValuePair {
                key: OFFSET.to_string(),
                value: "0".to_string(),
            },
            KeyValuePair {
                key: MILVUS_LIMIT.to_string(),
                value: "1".to_string(),
            },
        ];

        if let Some(consistency_level) = self.options.consistency_level {
            init_ts_params.push(KeyValuePair {
                key: "consistency_level".to_string(),
                value: consistency_level.to_string(),
            });
        }

        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(MsgBase::new(MsgType::Retrieve)),
                db_name: "".to_string(),
                collection_name: self.collection_name.clone(),
                expr: self.setup_expr(),
                output_fields: vec![],
                partition_names: self.options.partition_names.clone(),
                travel_timestamp: 0,
                guarantee_timestamp: 0,
                query_params: init_ts_params,
                not_return_all_meta: false,
                consistency_level: self.options.consistency_level.unwrap_or(0),
                use_default_consistency: self.options.consistency_level.is_none(),
                expr_template_values: self.options.expr_template_values.clone(),
                namespace: self.options.namespace.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        self.session_ts = res.session_ts;

        Ok(())
    }

    async fn setup_ts_cp(&mut self) -> Result<()> {
        self.buffer_cursor_lines_number = 0;

        if let Some(cp_file_path) = &self.options.iterator_cp_file {
            self.need_save_cp = true;
            self.cp_file_path = Some(cp_file_path.clone());

            if let Ok(file) = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(cp_file_path)
            {
                self.cp_file_handler = Some(file);

                // Try to read existing checkpoint
                if let Some(ref mut file) = self.cp_file_handler {
                    file.seek(SeekFrom::Start(0))?;
                    let reader = BufReader::new(file);
                    let lines: Vec<String> = reader
                        .lines()
                        .map(|line| line.map_err(|e| Error::Io(e)))
                        .collect::<Result<Vec<String>>>()?;

                    if lines.len() >= 2 {
                        self.session_ts = lines[0].parse::<u64>()?;
                        self.next_id = Some(lines[lines.len() - 1].clone());
                        self.buffer_cursor_lines_number = lines.len() - 1;
                    } else {
                        // Empty file, setup session_ts by request
                        self.setup_session_ts().await?;
                        self.save_mvcc_ts()?;
                    }
                }
            } else {
                // Failed to open file, setup session_ts by request
                self.setup_session_ts().await?;
            }
        } else {
            // No checkpoint file specified
            self.need_save_cp = false;
            self.setup_session_ts().await?;
        }

        Ok(())
    }

    fn save_mvcc_ts(&mut self) -> Result<()> {
        if let Some(ref mut file) = self.cp_file_handler {
            file.seek(SeekFrom::Start(0))?;
            writeln!(file, "{}", self.session_ts)?;
            file.flush()?;
        }
        Ok(())
    }

    fn save_pk_cursor(&mut self) -> Result<()> {
        if !self.need_save_cp || self.next_id.is_none() {
            return Ok(());
        }

        if let Some(ref mut file) = self.cp_file_handler {
            if self.buffer_cursor_lines_number >= 100 {
                file.seek(SeekFrom::Start(0))?;
                file.set_len(0)?;
                self.buffer_cursor_lines_number = 0;
                let session_ts = self.session_ts;
                writeln!(file, "{}", session_ts)?;
            }

            writeln!(file, "{}", self.next_id.as_ref().unwrap())?;
            file.flush()?;
            self.buffer_cursor_lines_number += 1;
        }

        Ok(())
    }

    fn check_set_batch_size(&mut self) -> Result<()> {
        let batch_size = self.options.batch_size.unwrap_or(1000);
        if batch_size > MAX_BATCH_SIZE {
            return Err(Error::Param(format!(
                "batch size cannot be larger than {}",
                MAX_BATCH_SIZE
            )));
        }
        Ok(())
    }

    async fn seek_to_offset(&mut self) -> Result<()> {
        if self.next_id.is_some() {
            return Ok(());
        }

        let offset = self.options.offset.unwrap_or(0);
        if offset > 0 {
            let mut current_offset = offset;

            while current_offset > 0 {
                let batch_size = std::cmp::min(MAX_BATCH_SIZE, current_offset as usize);
                let next_expr = self.build_next_expr();

                let advanced_count = self.seek_offset_by_batch(batch_size, &next_expr).await?;
                if advanced_count == 0 {
                    break;
                }
                current_offset -= advanced_count as i64;
            }
        }

        Ok(())
    }

    async fn seek_offset_by_batch(&mut self, batch: usize, expr: &str) -> Result<usize> {
        let mut seek_params = vec![KeyValuePair {
            key: MILVUS_LIMIT.to_string(),
            value: batch.to_string(),
        }];

        if let Some(consistency_level) = self.options.consistency_level {
            seek_params.push(KeyValuePair {
                key: "consistency_level".to_string(),
                value: consistency_level.to_string(),
            });
        }

        let res = self
            .client
            .clone()
            .query(QueryRequest {
                base: Some(MsgBase::new(MsgType::Retrieve)),
                db_name: "".to_string(),
                collection_name: self.collection_name.clone(),
                expr: expr.to_string(),
                output_fields: vec![],
                partition_names: self.options.partition_names.clone(),
                travel_timestamp: 0,
                guarantee_timestamp: self.session_ts,
                query_params: seek_params,
                not_return_all_meta: false,
                consistency_level: self.options.consistency_level.unwrap_or(0),
                use_default_consistency: self.options.consistency_level.is_none(),
                expr_template_values: self.options.expr_template_values.clone(),
                namespace: self.options.namespace.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        let results: Vec<FieldColumn> = res.fields_data.into_iter().map(Into::into).collect();
        self.update_cursor(&results);

        Ok(results.len())
    }

    fn build_next_expr(&self) -> String {
        if self.next_id.is_none() {
            return self.options.filter.clone();
        }

        let base_expr = if self.options.filter.is_empty() {
            "".to_string()
        } else {
            self.options.filter.clone()
        };

        let pk_filter = if self.pk_is_string {
            format!(
                "{} > \"{}\"",
                self.pk_field_name.as_ref().unwrap(),
                self.next_id.as_ref().unwrap()
            )
        } else {
            format!(
                "{} > {}",
                self.pk_field_name.as_ref().unwrap(),
                self.next_id.as_ref().unwrap()
            )
        };

        if base_expr.is_empty() {
            pk_filter
        } else {
            format!("({}) and {}", base_expr, pk_filter)
        }
    }

    fn update_cursor(&mut self, results: &[FieldColumn]) {
        if results.is_empty() {
            return;
        }

        for field_column in results {
            if field_column.name == *self.pk_field_name.as_ref().unwrap() {
                if field_column.len() == 0 {
                    continue;
                }
                if let Some(last_value) = field_column.get(field_column.len() - 1) {
                    self.next_id = Some(match last_value {
                        Value::Long(id) => id.to_string(),
                        Value::String(id) => id.to_string(),
                        _ => return,
                    });
                }
                break;
            }
        }
    }

    fn check_reached_limit(&self, results: &[FieldColumn]) -> Vec<FieldColumn> {
        if self.options.limit.is_none() {
            return results.to_vec();
        }

        let limit = self.options.limit.unwrap();
        let left_count = limit.saturating_sub(self.returned_count);

        if left_count >= results.len() {
            results.to_vec()
        } else {
            let safe_count = std::cmp::min(left_count, results.len());
            results[..safe_count].to_vec()
        }
    }

    fn is_res_sufficient(&self, cached_res: &Option<Vec<Vec<FieldColumn>>>) -> bool {
        if let Some(res) = cached_res {
            !res.is_empty() && res[0].len() >= self.options.batch_size.unwrap_or(1000)
        } else {
            false
        }
    }

    fn maybe_cache(&mut self, result: &[FieldColumn]) {
        let batch_size = self.options.batch_size.unwrap_or(1000);
        if result.len() < 2 * batch_size {
            return;
        }

        let start = batch_size;
        let cache_result = vec![result[start..].to_vec()];

        if let Ok(mut cache) = ITERATOR_CACHE.lock() {
            self.cache_id_in_use = cache.cache(cache_result, self.cache_id_in_use);
        }
    }

    /// Get the next batch of results
    pub async fn next(&mut self) -> Result<Option<Vec<FieldColumn>>> {
        if !self.has_more {
            return Ok(None);
        }

        if self.pk_field_name.is_none() {
            self.setup_collection_id().await?;
            self.setup_pk_prop().await?;
            self.check_set_batch_size()?;
            self.setup_ts_cp().await?;
            self.seek_to_offset().await?;
        }

        let cached_res = {
            if let Ok(cache) = ITERATOR_CACHE.lock() {
                cache.fetch_cache(self.cache_id_in_use)
            } else {
                None
            }
        };

        let ret = if self.is_res_sufficient(&cached_res) {
            let mut cache = ITERATOR_CACHE.lock().unwrap();
            let cached_data = cache.fetch_cache(self.cache_id_in_use).unwrap();

            let result = cached_data[0].clone();
            let res_to_cache = if cached_data.len() > 1 {
                cached_data[1..].to_vec()
            } else {
                vec![]
            };

            if res_to_cache.is_empty() {
                cache.release_cache(self.cache_id_in_use);
                self.cache_id_in_use = NO_CACHE_ID;
            } else {
                cache.cache(res_to_cache, self.cache_id_in_use);
            }
            result
        } else {
            if let Ok(mut cache) = ITERATOR_CACHE.lock() {
                cache.release_cache(self.cache_id_in_use);
            }
            self.cache_id_in_use = NO_CACHE_ID;

            if let Some(limit) = self.options.limit {
                if self.returned_count >= limit {
                    self.has_more = false;
                    return Ok(None);
                }
            }

            let batch_size = self.options.batch_size.unwrap_or(1000);
            let remaining_limit = if let Some(limit) = self.options.limit {
                if self.returned_count >= limit {
                    self.has_more = false;
                    return Ok(None);
                }
                let remaining = limit.saturating_sub(self.returned_count);
                std::cmp::min(remaining, batch_size)
            } else {
                batch_size
            };

            let expr = self.build_next_expr();

            let mut query_params = vec![
                KeyValuePair {
                    key: MILVUS_LIMIT.to_string(),
                    value: remaining_limit.to_string(),
                },
                KeyValuePair {
                    key: "topk".to_string(),
                    value: remaining_limit.to_string(),
                },
                KeyValuePair {
                    key: ITERATOR_FIELD.to_string(),
                    value: "true".to_string(),
                },
                KeyValuePair {
                    key: "search_iter_v2".to_string(),
                    value: "true".to_string(),
                },
                KeyValuePair {
                    key: "search_iter_batch_size".to_string(),
                    value: batch_size.to_string(),
                },
            ];

            if let Some(reduce_stop_for_best) = self.options.reduce_stop_for_best {
                query_params.push(KeyValuePair {
                    key: "reduce_stop_for_best".to_string(),
                    value: if reduce_stop_for_best {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                });
            }

            if let Some(consistency_level) = self.options.consistency_level {
                query_params.push(KeyValuePair {
                    key: "consistency_level".to_string(),
                    value: consistency_level.to_string(),
                });
            }

            if self.session_ts > 0 {
                query_params.push(KeyValuePair {
                    key: GUARANTEE_TIMESTAMP.to_string(),
                    value: self.session_ts.to_string(),
                });
            }

            if let Some(graceful_time) = self.options.graceful_time {
                query_params.push(KeyValuePair {
                    key: "graceful_time".to_string(),
                    value: graceful_time.to_string(),
                });
            }

            let res = self
                .client
                .clone()
                .query(QueryRequest {
                    base: Some(MsgBase::new(MsgType::Retrieve)),
                    db_name: "".to_string(),
                    collection_name: self.collection_name.clone(),
                    expr,
                    output_fields: self.options.output_fields.clone(),
                    partition_names: self.options.partition_names.clone(),
                    travel_timestamp: 0,
                    guarantee_timestamp: self.session_ts,
                    query_params,
                    not_return_all_meta: false,
                    consistency_level: self.options.consistency_level.unwrap_or(0),
                    use_default_consistency: self.options.consistency_level.is_none(),
                    expr_template_values: self.options.expr_template_values.clone(),
                    namespace: self.options.namespace.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner();

            status_to_result(&res.status)?;

            self.current_cursor = Some(QueryCursor {
                session_ts: res.session_ts,
                cursor_pk: None,
            });

            let results: Vec<FieldColumn> = res.fields_data.into_iter().map(Into::into).collect();

            if results.is_empty() {
                self.has_more = false;
            } else {
                let actual_returned = results.iter().map(|r| r.len() as usize).sum::<usize>();
                if actual_returned < remaining_limit {
                    self.has_more = false;
                }
            }

            self.update_cursor(&results);

            self.maybe_cache(&results);

            let min_len = if results.is_empty() {
                0
            } else {
                let min_data_len = results.iter().map(|field| field.len()).min().unwrap_or(0);
                std::cmp::min(batch_size, min_data_len)
            };

            if min_len == 0 {
                vec![]
            } else {
                results
                    .into_iter()
                    .map(|field| {
                        let mut new_field = field.clone();
                        match &mut new_field.value {
                            ValueVec::None => {}
                            ValueVec::Bool(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Int(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Long(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Float(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Double(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::String(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Json(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Binary(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Array(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Geometry(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::GeometryWkt(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            ValueVec::Timestamptz(v) => {
                                if v.len() >= min_len {
                                    *v = v[..min_len].to_vec();
                                }
                            }
                            // SparseFloat, StructArray, VectorArray are complex types
                            // that don't support simple truncation
                            ValueVec::SparseFloat(_)
                            | ValueVec::StructArray(_)
                            | ValueVec::VectorArray(_) => {}
                        }
                        new_field
                    })
                    .collect()
            }
        };

        let final_result = self.check_reached_limit(&ret);

        self.save_pk_cursor()?;

        self.returned_count += final_result.len();

        if final_result.is_empty() {
            self.has_more = false;
            Ok(None)
        } else {
            Ok(Some(final_result))
        }
    }

    pub fn get_cursor(&self) -> Option<&QueryCursor> {
        self.current_cursor.as_ref()
    }

    pub fn close(&mut self) {
        if let Ok(mut cache) = ITERATOR_CACHE.lock() {
            cache.release_cache(self.cache_id_in_use);
        }

        if let Some(ref mut file) = self.cp_file_handler {
            let _ = file.flush();
        }

        self.has_more = false;
        self.current_batch = None;
        self.current_cursor = None;
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }

    pub fn current_offset(&self) -> i64 {
        self.current_offset
    }

    pub fn returned_count(&self) -> usize {
        self.returned_count
    }
}

/// Iterator for searching large datasets in batches.
///
/// Set the generic search parameter `search_iter_cursor_version` to `2` to opt
/// into `(score, primary key)` pagination. Distance-only pagination is the default.
/// Older V2 servers retain distance-only pagination; servers without V2 return
/// an explicit unsupported error. Use `ClientV2` for genuine legacy range fallback.
pub struct SearchIterator {
    client: crate::proto::milvus::milvus_service_client::MilvusServiceClient<
        tonic::service::interceptor::InterceptedService<
            tonic::transport::Channel,
            crate::v1::client::CombinedInterceptor,
        >,
    >,
    collection_name: String,
    data: Vec<crate::v1::value::Value<'static>>,
    options: SearchIteratorOptions,
    current_offset: i64,
    has_more: bool,
    current_batch: Option<Vec<crate::v1::collection::SearchResult<'static>>>,
    returned_count: usize,
    session_ts: u64,
    cache_id_in_use: i32,
    cp_file_handler: Option<File>,
    cp_file_path: Option<String>,
    need_save_cp: bool,
    buffer_cursor_lines_number: usize,
    collection_id: u64,
    iterator_token: Option<String>,
    last_bound: Option<String>,
    primary_key_type: Option<String>,
    initialized: bool,
    cursor_mode: Option<CursorMode>,
    last_pk: Option<String>,
    accepted_pks: HashSet<String>,
}

impl SearchIterator {
    pub fn new(
        client: crate::proto::milvus::milvus_service_client::MilvusServiceClient<
            tonic::service::interceptor::InterceptedService<
                tonic::transport::Channel,
                crate::v1::client::CombinedInterceptor,
            >,
        >,
        collection_name: String,
        _consistency_level: ConsistencyLevel,
        data: Vec<crate::v1::value::Value<'_>>,
        options: SearchIteratorOptions,
    ) -> Self {
        Self {
            client,
            collection_name,
            data: data.into_iter().map(|v| v.into_owned()).collect(),
            options,
            current_offset: 0,
            has_more: true,
            current_batch: None,
            returned_count: 0,
            session_ts: 0,
            cache_id_in_use: NO_CACHE_ID,
            cp_file_handler: None,
            cp_file_path: None,
            need_save_cp: false,
            buffer_cursor_lines_number: 0,
            collection_id: 0,
            iterator_token: None,
            last_bound: None,
            primary_key_type: None,
            initialized: false,
            cursor_mode: None,
            last_pk: None,
            accepted_pks: HashSet::new(),
        }
    }

    async fn setup_collection_id(&mut self) -> Result<()> {
        let res = self
            .client
            .clone()
            .describe_collection(crate::proto::milvus::DescribeCollectionRequest {
                base: Some(MsgBase::new(MsgType::DescribeCollection)),
                db_name: "".to_string(),
                collection_name: self.collection_name.clone(),
                collection_id: 0,
                time_stamp: 0,
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;
        self.collection_id = u64::try_from(res.collection_id)
            .map_err(|_| Error::Unexpected("negative collection ID".into()))?;
        let primary_type = res
            .schema
            .as_ref()
            .and_then(|schema| schema.fields.iter().find(|field| field.is_primary_key))
            .map(|field| field.data_type);
        self.primary_key_type = Some(match primary_type {
            Some(value) if value == DataType::Int64 as i32 => "int64".into(),
            Some(value) if value == DataType::VarChar as i32 => "varchar".into(),
            _ => {
                return Err(Error::Unexpected(
                    "unsupported search iterator primary key schema".into(),
                ))
            }
        });
        Ok(())
    }

    fn check_set_batch_size(&mut self) -> Result<()> {
        let batch_size = self.options.batch_size.unwrap_or(1000);
        if batch_size == 0 || batch_size > MAX_BATCH_SIZE {
            return Err(Error::Param(format!(
                "batch size cannot be larger than {}",
                MAX_BATCH_SIZE
            )));
        }
        Ok(())
    }

    async fn setup_session_ts(&mut self) -> Result<()> {
        self.session_ts = self.options.guarantee_timestamp.unwrap_or(0);
        Ok(())
    }

    async fn setup_ts_cp(&mut self) -> Result<()> {
        self.buffer_cursor_lines_number = 0;

        if let Some(cp_file_path) = &self.options.iterator_cp_file {
            self.need_save_cp = true;
            self.cp_file_path = Some(cp_file_path.clone());

            if let Ok(file) = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(cp_file_path)
            {
                self.cp_file_handler = Some(file);

                // Try to read existing checkpoint
                if let Some(ref mut file) = self.cp_file_handler {
                    file.seek(SeekFrom::Start(0))?;
                    let reader = BufReader::new(file);
                    let lines: Vec<String> = reader
                        .lines()
                        .map(|line| line.map_err(|e| Error::Io(e)))
                        .collect::<Result<Vec<String>>>()?;

                    if lines.len() >= 2 {
                        self.session_ts = lines[0].parse::<u64>()?;
                        let trailing_token = &lines[lines.len() - 1];
                        let cursor = if lines.len() >= 3 && lines[lines.len() - 2].starts_with('{')
                        {
                            &lines[lines.len() - 2]
                        } else {
                            trailing_token
                        };
                        if cursor.starts_with('{') {
                            let saved: serde_json::Value = serde_json::from_str(cursor)?;
                            if saved["format_version"].as_u64() != Some(2) || self.session_ts == 0 {
                                return Err(Error::Unexpected(
                                    "invalid PK search iterator checkpoint version or snapshot"
                                        .into(),
                                ));
                            }
                            if self.options.guarantee_timestamp.is_some_and(|timestamp| {
                                timestamp > 0 && timestamp != self.session_ts
                            }) {
                                return Err(Error::Unexpected(
                                    "PK checkpoint snapshot does not match explicit guarantee timestamp"
                                        .into(),
                                ));
                            }
                            let required = |key: &str| {
                                saved[key].as_str().map(str::to_owned).ok_or_else(|| {
                                    Error::Unexpected(format!("PK checkpoint is missing {key}"))
                                })
                            };
                            if saved["collection_id"].as_u64() != Some(self.collection_id) {
                                return Err(Error::Unexpected(
                                    "PK checkpoint collection ID does not match collection".into(),
                                ));
                            }
                            if required("request_fingerprint")? != self.checkpoint_fingerprint()? {
                                return Err(Error::Unexpected(
                                    "PK checkpoint search definition does not match request".into(),
                                ));
                            }
                            let token = required("token")?;
                            if lines.len() >= 3
                                && cursor != trailing_token
                                && token != *trailing_token
                            {
                                return Err(Error::Unexpected(
                                    "PK checkpoint UUID does not match full cursor".into(),
                                ));
                            }
                            self.iterator_token = Some(token);
                            self.last_bound = Some(required("last_bound")?);
                            self.last_pk = Some(required("last_pk")?);
                            if required("last_pk_type")?
                                != self
                                    .primary_key_type
                                    .as_deref()
                                    .expect("schema initialized")
                            {
                                return Err(Error::Unexpected(
                                    "PK checkpoint schema does not match collection".into(),
                                ));
                            }
                            self.returned_count =
                                usize::try_from(saved["returned_count"].as_u64().ok_or_else(
                                    || Error::Unexpected("PK checkpoint has no hit count".into()),
                                )?)
                                .map_err(|_| {
                                    Error::Unexpected("PK checkpoint hit count is too large".into())
                                })?;
                            self.cursor_mode = Some(CursorMode::PrimaryKey);
                            self.accepted_pks = serde_json::from_value(
                                saved.get("accepted_pks").cloned().ok_or_else(|| {
                                    Error::Unexpected(
                                        "PK checkpoint has no accepted primary keys".into(),
                                    )
                                })?,
                            )?;
                            if self.accepted_pks.len() != self.returned_count {
                                return Err(Error::Unexpected(
                                    "PK checkpoint primary keys do not match consumed hit count"
                                        .into(),
                                ));
                            }
                        } else {
                            // Preserve the historical UUID-only format. It does not
                            // contain a distance/PK cursor and cannot guarantee exact resume.
                            self.iterator_token = Some(cursor.clone());
                            self.cursor_mode = Some(CursorMode::Distance);
                        }
                        self.buffer_cursor_lines_number = lines.len() - 1;
                    } else {
                        // Empty file, setup session_ts by request
                        self.setup_session_ts().await?;
                        self.save_mvcc_ts()?;
                    }
                }
            } else {
                // Failed to open file, setup session_ts by request
                self.setup_session_ts().await?;
            }
        } else {
            // No checkpoint file specified
            self.need_save_cp = false;
            self.setup_session_ts().await?;
        }

        Ok(())
    }

    fn save_mvcc_ts(&mut self) -> Result<()> {
        if let Some(ref mut file) = self.cp_file_handler {
            file.seek(SeekFrom::Start(0))?;
            writeln!(file, "{}", self.session_ts)?;
            file.flush()?;
        }
        Ok(())
    }

    fn save_iterator_token(&mut self) -> Result<()> {
        if !self.need_save_cp || self.iterator_token.is_none() {
            return Ok(());
        }

        if let Some(ref mut file) = self.cp_file_handler {
            if self.buffer_cursor_lines_number >= 100 {
                file.seek(SeekFrom::Start(0))?;
                file.set_len(0)?;
                self.buffer_cursor_lines_number = 0;
                let session_ts = self.session_ts;
                writeln!(file, "{}", session_ts)?;
            }

            writeln!(file, "{}", self.iterator_token.as_ref().unwrap())?;
            file.flush()?;
            self.buffer_cursor_lines_number += 1;
        }

        Ok(())
    }

    fn checkpoint_fingerprint(&self) -> Result<String> {
        fn canonical(value: serde_json::Value) -> serde_json::Value {
            match value {
                serde_json::Value::Object(values) => {
                    let sorted: std::collections::BTreeMap<_, _> = values
                        .into_iter()
                        .map(|(key, value)| (key, canonical(value)))
                        .collect();
                    serde_json::Value::Object(sorted.into_iter().collect())
                }
                serde_json::Value::Array(values) => {
                    serde_json::Value::Array(values.into_iter().map(canonical).collect())
                }
                value => value,
            }
        }
        fn volatile_param(key: &str) -> bool {
            matches!(
                key,
                "search_iter_id"
                    | "search_iter_last_bound"
                    | "search_iter_cursor_version"
                    | "search_iter_last_pk_type"
                    | "search_iter_last_pk"
                    | "search_iter_batch_size"
                    | "search_iter_v2"
                    | "iterator"
                    | "collection_id"
                    | "topk"
                    | "limit"
            )
        }
        let params: std::collections::BTreeMap<_, _> = self
            .options
            .search_params
            .iter()
            .filter(|(key, _)| !volatile_param(key))
            .map(|(key, value)| {
                let mut value = serde_json::from_str(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
                if key == "params" {
                    if let serde_json::Value::Object(ref mut values) = value {
                        for cursor_key in [
                            iterator_cursor::CURSOR_VERSION,
                            iterator_cursor::LAST_PK_TYPE,
                            iterator_cursor::LAST_PK,
                        ] {
                            values.remove(cursor_key);
                        }
                    }
                }
                (key, canonical(value))
            })
            .collect();
        let templates: std::collections::BTreeMap<_, _> = self
            .options
            .expr_template_values
            .iter()
            .map(|(key, value)| (key, value.encode_to_vec()))
            .collect();
        // Output projection, timeout, total limit and snapshot do not change the
        // search candidate ordering. The checkpoint stores snapshot/count itself;
        // batch size is fixed so resuming preserves the same page configuration.
        let definition = canonical(serde_json::json!({
            "placeholder_group": crate::v1::query::get_place_holder_group(&self.data)?,
            "filter": self.options.filter,
            "namespace": self.options.namespace,
            "partition_names": self.options.partition_names,
            "anns_field": self.options.anns_field,
            "batch_size": self.options.batch_size.unwrap_or(1000),
            "search_params": params,
            "expr_template_values": templates,
            "round_decimal": self.options.round_decimal,
        }));
        let digest = Sha256::digest(serde_json::to_vec(&definition)?);
        Ok(format!("{digest:x}"))
    }

    fn save_pk_checkpoint(
        &self,
        request: &crate::proto::milvus::SearchRequest,
        returned_count: usize,
        added_pks: &HashSet<String>,
    ) -> Result<()> {
        if !self.need_save_cp {
            return Ok(());
        }
        let Some(path) = &self.cp_file_path else {
            return Ok(());
        };
        let Some(last_pk) = iterator_cursor::param(request, iterator_cursor::LAST_PK) else {
            return Ok(());
        };
        let saved = serde_json::json!({
            "format_version":2,
            "collection_id":self.collection_id,
            "request_fingerprint":self.checkpoint_fingerprint()?,
            "token":iterator_cursor::param(request,"search_iter_id"),
            "last_bound":iterator_cursor::param(request,"search_iter_last_bound"),
            "last_pk_type":iterator_cursor::param(request,iterator_cursor::LAST_PK_TYPE),
            "last_pk":last_pk,
            "returned_count":returned_count,
            "accepted_pks":self.accepted_pks.iter().chain(added_pks).collect::<Vec<_>>(),
        });
        let temporary = format!("{path}.tmp.{}", uuid::Uuid::new_v4());
        let write_result = (|| -> std::io::Result<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            writeln!(file, "{}", request.guarantee_timestamp)?;
            writeln!(file, "{saved}")?;
            writeln!(
                file,
                "{}",
                iterator_cursor::param(request, "search_iter_id")
                    .expect("validated cursor has token")
            )?;
            file.sync_all()?;
            std::fs::rename(&temporary, path)
        })();
        if write_result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        write_result?;
        Ok(())
    }

    /// Get the next batch of search results
    pub async fn next(
        &mut self,
    ) -> Result<Option<Vec<crate::v1::collection::SearchResult<'static>>>> {
        if !self.has_more {
            return Ok(None);
        }

        if self.data.len() != 1 {
            return Err(Error::Param(
                "search iterator requires exactly one query vector".into(),
            ));
        }
        if !self.initialized {
            self.setup_collection_id().await?;
            self.check_set_batch_size()?;
            self.setup_ts_cp().await?;
            self.initialized = true;
        }

        if let Some(limit) = self.options.limit {
            if self.returned_count >= limit {
                self.has_more = false;
                return Ok(None);
            }
        }

        let batch_size = self.options.batch_size.unwrap_or(1000);
        let remaining_limit = if let Some(limit) = self.options.limit {
            if self.returned_count >= limit {
                self.has_more = false;
                return Ok(None);
            }
            let remaining = limit.saturating_sub(self.returned_count);
            std::cmp::min(remaining, batch_size)
        } else {
            batch_size
        };

        loop {
            let result = self.next_page(remaining_limit).await?;
            if result.is_some()
                || !self.has_more
                || self.cursor_mode != Some(CursorMode::PrimaryKey)
            {
                return Ok(result);
            }
        }
    }

    async fn next_page(
        &mut self,
        remaining_limit: usize,
    ) -> Result<Option<Vec<crate::v1::collection::SearchResult<'static>>>> {
        let mut search_params = vec![
            KeyValuePair {
                key: MILVUS_LIMIT.to_string(),
                value: remaining_limit.to_string(),
            },
            KeyValuePair {
                key: "topk".to_string(),
                value: remaining_limit.to_string(),
            },
            KeyValuePair {
                key: ITERATOR_FIELD.to_string(),
                value: "true".to_string(),
            },
            KeyValuePair {
                key: "search_iter_v2".to_string(),
                value: "true".to_string(),
            },
            KeyValuePair {
                key: "search_iter_batch_size".to_string(),
                value: remaining_limit.to_string(),
            },
        ];

        if let Some(ref token) = self.iterator_token {
            search_params.push(KeyValuePair {
                key: "search_iter_id".to_string(),
                value: token.clone(),
            });
        }

        if let Some(ref last_bound) = self.last_bound {
            search_params.push(KeyValuePair {
                key: "search_iter_last_bound".to_string(),
                value: last_bound.clone(),
            });
        }

        if let Some(ref anns_field) = self.options.anns_field {
            search_params.push(KeyValuePair {
                key: "anns_field".to_string(),
                value: anns_field.clone(),
            });
        }

        for (key, value) in &self.options.search_params {
            search_params.push(KeyValuePair {
                key: key.clone(),
                value: value.clone(),
            });
        }

        let mut request = crate::proto::milvus::SearchRequest {
            base: Some(MsgBase::new(MsgType::Search)),
            db_name: "".to_string(),
            collection_name: self.collection_name.clone(),
            partition_names: self.options.partition_names.clone(),
            dsl: self.options.filter.clone(),
            nq: self.data.len() as _,
            search_input: Some(
                crate::proto::milvus::search_request::SearchInput::PlaceholderGroup(
                    crate::v1::query::get_place_holder_group(&self.data)?,
                ),
            ),
            dsl_type: crate::proto::common::DslType::BoolExprV1 as _,
            output_fields: self.options.output_fields.clone(),
            search_params,
            travel_timestamp: 0,
            guarantee_timestamp: self.session_ts,
            not_return_all_meta: false,
            consistency_level: self.options.consistency_level.unwrap_or(0),
            use_default_consistency: self.options.consistency_level.is_none(),
            #[allow(deprecated)]
            search_by_primary_keys: false,
            expr_template_values: self.options.expr_template_values.clone(),
            sub_reqs: vec![],
            function_score: None,
            namespace: self.options.namespace.clone(),
            highlighter: None,
            ..Default::default()
        };
        iterator_cursor::set_param(
            &mut request,
            "collection_id",
            self.collection_id.to_string(),
        );
        for (key, value) in [
            ("topk", remaining_limit.to_string()),
            (MILVUS_LIMIT, remaining_limit.to_string()),
            ("search_iter_batch_size", remaining_limit.to_string()),
            (ITERATOR_FIELD, "true".into()),
            ("search_iter_v2", "true".into()),
        ] {
            iterator_cursor::set_param(&mut request, key, value);
        }
        // Caller-owned initial cursors are used once. Committed iterator state
        // must replace them when each later request is rebuilt from options.
        if let Some(token) = &self.iterator_token {
            iterator_cursor::set_param(&mut request, "search_iter_id", token.clone());
        }
        if let Some(bound) = &self.last_bound {
            iterator_cursor::set_param(&mut request, "search_iter_last_bound", bound.clone());
        }
        let previous_mode =
            iterator_cursor::configure(&mut request, self.cursor_mode).map_err(Error::Param)?;
        if previous_mode == Some(CursorMode::PrimaryKey) {
            if let Some(last_pk) = &self.last_pk {
                iterator_cursor::set_param(
                    &mut request,
                    iterator_cursor::LAST_PK_TYPE,
                    self.primary_key_type.clone().expect("schema initialized"),
                );
                iterator_cursor::set_param(&mut request, iterator_cursor::LAST_PK, last_pk.clone());
            }
        }
        let res = self
            .client
            .clone()
            .search(request.clone())
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        let (next_request, next_mode) = iterator_cursor::advance(
            &request,
            &res,
            previous_mode,
            self.primary_key_type
                .as_deref()
                .expect("schema initialized"),
            remaining_limit,
        )
        .map_err(Error::Unexpected)?;
        let raw_data = res
            .results
            .ok_or_else(|| Error::Unexpected("no result for search".into()))?;
        let mut result = Vec::new();
        let mut offset = 0;
        let fields_data = raw_data
            .fields_data
            .into_iter()
            .map(Into::into)
            .collect::<Vec<FieldColumn>>();

        let raw_id = raw_data
            .ids
            .and_then(|ids| ids.id_field)
            .unwrap_or_else(|| {
                if self.primary_key_type.as_deref() == Some("varchar") {
                    crate::proto::schema::i_ds::IdField::StrId(
                        crate::proto::schema::StringArray::default(),
                    )
                } else {
                    crate::proto::schema::i_ds::IdField::IntId(
                        crate::proto::schema::LongArray::default(),
                    )
                }
            });

        for k in raw_data.topks {
            let k = k as usize;

            if offset + k > raw_data.scores.len() {
                return Err(Error::Unexpected(format!(
                    "scores array bounds exceeded: offset={}, k={}, scores_len={}",
                    offset,
                    k,
                    raw_data.scores.len()
                )));
            }

            let mut score = Vec::new();
            score.extend_from_slice(&raw_data.scores[offset..offset + k]);
            let result_data =
                slice_field_columns(&fields_data, offset, k).map_err(Error::Unexpected)?;

            let id = match raw_id {
                crate::proto::schema::i_ds::IdField::IntId(ref d) => {
                    if offset + k > d.data.len() {
                        return Err(Error::Unexpected(format!(
                            "int id array bounds exceeded: offset={}, k={}, id_len={}",
                            offset,
                            k,
                            d.data.len()
                        )));
                    }
                    Vec::<Value>::from_iter(d.data[offset..offset + k].iter().map(|&x| x.into()))
                }
                crate::proto::schema::i_ds::IdField::StrId(ref d) => {
                    if offset + k > d.data.len() {
                        return Err(Error::Unexpected(format!(
                            "string id array bounds exceeded: offset={}, k={}, id_len={}",
                            offset,
                            k,
                            d.data.len()
                        )));
                    }
                    Vec::<Value>::from_iter(
                        d.data[offset..offset + k].iter().map(|x| x.clone().into()),
                    )
                }
                crate::proto::schema::i_ds::IdField::UuidId(_) => {
                    return Err(Error::Unexpected(
                        "uuid primary keys are not supported by V1 search".to_string(),
                    ));
                }
            };

            result.push(crate::v1::collection::SearchResult {
                size: k as i64,
                score,
                field: result_data,
                id,
                highlight_results: vec![],
            });

            offset += k;
        }

        let raw_count = result
            .iter()
            .map(|result| result.size as usize)
            .sum::<usize>();
        let mut added_pks = HashSet::new();
        if next_mode == CursorMode::PrimaryKey {
            for page in &mut result {
                let mut keep = Vec::new();
                for (index, id) in page.id.iter().enumerate() {
                    let key = match id {
                        Value::Long(value) => value.to_string(),
                        Value::String(value) => value.to_string(),
                        _ => {
                            return Err(Error::Unexpected(
                                "invalid PK search iterator primary key".into(),
                            ))
                        }
                    };
                    if !self.accepted_pks.contains(&key) && added_pks.insert(key) {
                        keep.push(index);
                    }
                }
                let mut fields = page
                    .field
                    .iter()
                    .map(FieldColumn::copy_with_metadata)
                    .collect::<Vec<_>>();
                for (source, target) in page.field.iter().zip(&mut fields) {
                    for &index in &keep {
                        target.push(source.get(index).ok_or_else(|| {
                            Error::Unexpected("PK search iterator output field is too short".into())
                        })?);
                    }
                }
                page.id = keep.iter().map(|&index| page.id[index].clone()).collect();
                page.score = keep.iter().map(|&index| page.score[index]).collect();
                page.field = fields;
                page.size = keep.len() as i64;
            }
        }
        let actual_returned = result
            .iter()
            .map(|result| result.size as usize)
            .sum::<usize>();
        if next_mode == CursorMode::PrimaryKey {
            self.accepted_pks
                .try_reserve(added_pks.len())
                .map_err(|error| {
                    Error::Unexpected(format!(
                        "search iterator primary-key cache allocation failed: {error}"
                    ))
                })?;
            self.save_pk_checkpoint(
                &next_request,
                self.returned_count + actual_returned,
                &added_pks,
            )?;
        }
        // Commit only after the full payload has been decoded. Later server replies
        // normally contain session_ts=0, so retain the first positive snapshot.
        self.session_ts = next_request.guarantee_timestamp;
        self.iterator_token =
            iterator_cursor::param(&next_request, "search_iter_id").map(str::to_owned);
        self.last_bound =
            iterator_cursor::param(&next_request, "search_iter_last_bound").map(str::to_owned);
        self.last_pk =
            iterator_cursor::param(&next_request, iterator_cursor::LAST_PK).map(str::to_owned);
        self.cursor_mode = Some(next_mode);
        self.returned_count += actual_returned;
        self.accepted_pks.extend(added_pks);
        self.has_more = raw_count != 0
            && self
                .options
                .limit
                .is_none_or(|limit| self.returned_count < limit);
        if next_mode == CursorMode::Distance {
            self.save_iterator_token()?;
        }
        if actual_returned == 0 {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub fn close(&mut self) {
        if let Ok(mut cache) = ITERATOR_CACHE.lock() {
            cache.release_cache(self.cache_id_in_use);
        }

        if let Some(ref mut file) = self.cp_file_handler {
            let _ = file.flush();
        }

        self.has_more = false;
        self.current_batch = None;
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }

    pub fn current_offset(&self) -> i64 {
        self.current_offset
    }

    pub fn returned_count(&self) -> usize {
        self.returned_count
    }
}
