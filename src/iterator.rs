use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;

use crate::client::{Client, ConsistencyLevel};
use crate::data::FieldColumn;
use crate::error::*;
use crate::proto::common::{KeyValuePair, MsgBase, MsgType};
use crate::proto::milvus::{QueryCursor, QueryRequest};
use crate::proto::schema::DataType;
use crate::utils::status_to_result;
use crate::value::{Value, ValueVec};

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
    pub timeout: Option<f64>,
    pub consistency_level: Option<i32>,
    pub guarantee_timestamp: Option<u64>,
    pub graceful_time: Option<u64>,
    pub offset: Option<i64>,
    pub expr_template_values: HashMap<String, crate::proto::schema::TemplateValue>,
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
        let collection = self.collection_cache.get(&collection_name).await?;

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
        data: Vec<crate::value::Value<'_>>,
        options: SearchIteratorOptions,
    ) -> Result<SearchIterator>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let collection = self.collection_cache.get(&collection_name).await?;

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
            crate::client::CombinedInterceptor,
        >,
    >,
    collection_name: String,
    consistency_level: ConsistencyLevel,
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
                crate::client::CombinedInterceptor,
            >,
        >,
        collection_name: String,
        consistency_level: ConsistencyLevel,
        options: QueryIteratorOptions,
    ) -> Self {
        Self {
            client,
            collection_name,
            consistency_level,
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

                let seeked_count = self.seek_offset_by_batch(batch_size, &next_expr).await?;
                if seeked_count == 0 {
                    break;
                }
                current_offset -= seeked_count as i64;
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

/// Iterator for searching large datasets in batches
pub struct SearchIterator {
    client: crate::proto::milvus::milvus_service_client::MilvusServiceClient<
        tonic::service::interceptor::InterceptedService<
            tonic::transport::Channel,
            crate::client::CombinedInterceptor,
        >,
    >,
    collection_name: String,
    consistency_level: ConsistencyLevel,
    data: Vec<crate::value::Value<'static>>,
    options: SearchIteratorOptions,
    current_offset: i64,
    has_more: bool,
    current_batch: Option<Vec<crate::collection::SearchResult<'static>>>,
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
}

impl SearchIterator {
    pub fn new(
        client: crate::proto::milvus::milvus_service_client::MilvusServiceClient<
            tonic::service::interceptor::InterceptedService<
                tonic::transport::Channel,
                crate::client::CombinedInterceptor,
            >,
        >,
        collection_name: String,
        consistency_level: ConsistencyLevel,
        data: Vec<crate::value::Value<'_>>,
        options: SearchIteratorOptions,
    ) -> Self {
        Self {
            client,
            collection_name,
            consistency_level,
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
            })
            .await?
            .into_inner();

        // Extract collection_id from response (implementation may vary based on actual response structure)
        // For now, we'll use a placeholder
        self.collection_id = 0; // This should be extracted from the response
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

    async fn setup_session_ts(&mut self) -> Result<()> {
        // For search iterator, we'll use a simple approach to get session timestamp
        // This might need to be adjusted based on actual server implementation
        self.session_ts = 0;
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
                        self.iterator_token = Some(lines[lines.len() - 1].clone());
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

    fn check_reached_limit(
        &self,
        results: &[crate::collection::SearchResult<'static>],
    ) -> Vec<crate::collection::SearchResult<'static>> {
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

    /// Get the next batch of search results
    pub async fn next(&mut self) -> Result<Option<Vec<crate::collection::SearchResult<'static>>>> {
        if !self.has_more {
            return Ok(None);
        }

        if self.collection_id == 0 {
            self.setup_collection_id().await?;
            self.check_set_batch_size()?;
            self.setup_ts_cp().await?;
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
                value: batch_size.to_string(),
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

        let res = self
            .client
            .clone()
            .search(crate::proto::milvus::SearchRequest {
                base: Some(MsgBase::new(MsgType::Search)),
                db_name: "".to_string(),
                collection_name: self.collection_name.clone(),
                partition_names: self.options.partition_names.clone(),
                dsl: self.options.filter.clone(),
                nq: self.data.len() as _,
                placeholder_group: crate::query::get_place_holder_group(&self.data)?,
                dsl_type: crate::proto::common::DslType::BoolExprV1 as _,
                output_fields: self.options.output_fields.clone(),
                search_params,
                travel_timestamp: 0,
                guarantee_timestamp: self.session_ts,
                not_return_all_meta: false,
                consistency_level: self.options.consistency_level.unwrap_or(0),
                use_default_consistency: self.options.consistency_level.is_none(),
                search_by_primary_keys: false,
                expr_template_values: self.options.expr_template_values.clone(),
                sub_reqs: vec![],
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        self.session_ts = res.session_ts;

        let raw_data = res
            .results
            .ok_or(Error::Unexpected("no result for search".to_owned()))?;

        if let Some(iter_v2_results) = raw_data.search_iterator_v2_results {
            if self.iterator_token.is_none() {
                self.iterator_token = Some(iter_v2_results.token);
            }
            self.last_bound = Some(iter_v2_results.last_bound.to_string());
        } else {
            if self.iterator_token.is_none() {
                self.iterator_token = Some(format!("token_{}", self.session_ts));
            }
        }

        let mut result = Vec::new();
        let mut offset = 0;
        let fields_data = raw_data
            .fields_data
            .into_iter()
            .map(Into::into)
            .collect::<Vec<FieldColumn>>();

        let raw_id = raw_data
            .ids
            .ok_or(Error::Unexpected("no ids in search result".to_owned()))?
            .id_field
            .ok_or(Error::Unexpected("no id_field in search result".to_owned()))?;

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
            let mut result_data = fields_data
                .iter()
                .map(FieldColumn::copy_with_metadata)
                .collect::<Vec<FieldColumn>>();

            for j in 0..fields_data.len() {
                for i in offset..offset + k {
                    if i >= fields_data[j].len() {
                        return Err(Error::Unexpected(format!(
                            "field data bounds exceeded: field={}, index={}, field_len={}",
                            fields_data[j].name,
                            i,
                            fields_data[j].len()
                        )));
                    }
                    result_data[j].push(fields_data[j].get(i).ok_or(Error::Unexpected(
                        "out of range while indexing field data".to_owned(),
                    ))?);
                }
            }

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
            };

            result.push(crate::collection::SearchResult {
                size: k as i64,
                score,
                field: result_data,
                id,
            });

            offset += k;
        }

        if result.is_empty() {
            self.has_more = false;
        } else {
            let actual_returned = result.iter().map(|r| r.size as usize).sum::<usize>();
            if actual_returned < remaining_limit {
                self.has_more = false;
            }
        }

        let final_result = self.check_reached_limit(&result);

        self.save_iterator_token()?;

        self.returned_count += final_result.len();

        if final_result.is_empty() {
            self.has_more = false;
            Ok(None)
        } else {
            Ok(Some(final_result))
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
