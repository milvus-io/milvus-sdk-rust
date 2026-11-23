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

//! Process-wide endpoint-scoped caches for V2 schemas and DML timestamps.
//!
//! Both caches are bounded LRU caches whose capacity can be configured process-wide through
//! [`CacheConfig`], mirroring pymilvus's default of 4096 entries. Note that the Java and
//! C++ SDKs keep the timestamp cache unbounded; the 4096 bound here is a deliberate pymilvus
//! alignment.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::types::ConsistencyLevel;
use lazy_static::lazy_static;
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tonic::transport::Uri;

type CacheKey = (String, String, String);
static NEXT_SCHEMA_LOAD_SCOPE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub(super) struct SchemaLoadScope(u64);

impl SchemaLoadScope {
    pub(super) fn new() -> Self {
        Self(NEXT_SCHEMA_LOAD_SCOPE.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct LoadKey {
    collection: CacheKey,
    scope: u64,
}

fn cache_key(endpoint: &str, database: &str, collection: &str) -> CacheKey {
    (
        normalize_endpoint(endpoint),
        database_name(database).to_owned(),
        collection.to_owned(),
    )
}

fn normalize_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return String::new();
    }

    let uri = if endpoint.contains("://") {
        endpoint.to_owned()
    } else {
        format!("http://{endpoint}")
    };
    let Ok(uri) = uri.parse::<Uri>() else {
        return endpoint.to_owned();
    };
    let Some(host) = uri.host() else {
        return endpoint.to_owned();
    };
    let host = host.to_ascii_lowercase();
    let host = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host
    };
    let port = uri.port_u16().unwrap_or_else(|| {
        if uri.scheme_str() == Some("https") {
            443
        } else {
            80
        }
    });
    format!("{host}:{port}")
}

fn database_name(database: &str) -> &str {
    if database.is_empty() {
        "default"
    } else {
        database
    }
}

fn cache_with_capacity<T>(capacity: usize) -> LruCache<CacheKey, T> {
    LruCache::new(NonZeroUsize::new(capacity).expect("cache capacity must be greater than zero"))
}

/// Resizes an LRU cache to `capacity`, evicting least-recently-used entries as needed.
fn resize_cache<T>(cache: &RwLock<LruCache<CacheKey, T>>, capacity: usize) -> Result<()> {
    let capacity = NonZeroUsize::new(capacity).ok_or_else(|| {
        Error::validation("cache_capacity".into(), "must be greater than zero".into())
    })?;
    cache.write().resize(capacity);
    Ok(())
}

/// Returns the maximum number of entries an LRU cache can hold.
fn cache_capacity<T>(cache: &RwLock<LruCache<CacheKey, T>>) -> usize {
    cache.read().cap().get()
}

///////////////////////////////////////////////////////////////////////////////
// SchemaCache
///////////////////////////////////////////////////////////////////////////////
/// Process-wide collection-description cache keyed by endpoint, database, and collection.
/// Completed schemas are shared process-wide, while in-flight loads are coalesced only within
/// one client scope so another client's credentials or RPC deadline cannot control the load.
///
/// Reads (`get`/`get_or_load` hits) do not refresh LRU recency, so with more than the configured
/// number of collections a constantly-read-but-seldom-loaded schema can be evicted by loads of
/// other collections and re-fetched from the server on the next access. Re-fetch is always
/// correct and coalesced; this is the deliberate read-contention trade-off of using `peek` under
/// a read lock.
pub(super) struct SchemaCache {
    schemas: RwLock<LruCache<CacheKey, Arc<milvus::DescribeCollectionResponse>>>,
    loading: Mutex<HashMap<LoadKey, Arc<SchemaLoadState>>>,
}

impl SchemaCache {
    fn new() -> Self {
        Self::with_capacity(CacheConfig::DEFAULT_CAPACITY)
    }

    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            schemas: RwLock::new(cache_with_capacity(capacity)),
            loading: Mutex::new(HashMap::new()),
        }
    }

    /// Resizes the completed-schema cache to `capacity`, evicting least-recently-used entries.
    pub(super) fn set_capacity(&self, capacity: usize) -> Result<()> {
        resize_cache(&self.schemas, capacity)
    }

    /// Returns the maximum number of completed-schema entries the cache can hold.
    pub(super) fn get_capacity(&self) -> usize {
        cache_capacity(&self.schemas)
    }

    #[cfg(test)]
    pub(super) fn get(
        &self,
        endpoint: &str,
        database: &str,
        collection: &str,
    ) -> Option<Arc<milvus::DescribeCollectionResponse>> {
        let key = cache_key(endpoint, database, collection);
        self.schemas.read().peek(&key).cloned()
    }

    pub(super) async fn get_or_load<F, Fut>(
        &self,
        endpoint: &str,
        database: &str,
        collection: &str,
        force_update: bool,
        load_scope: &SchemaLoadScope,
        loader: F,
    ) -> Result<Arc<milvus::DescribeCollectionResponse>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<milvus::DescribeCollectionResponse>>,
    {
        let key = cache_key(endpoint, database, collection);
        let load_key = LoadKey {
            collection: key.clone(),
            scope: load_scope.0,
        };
        let initial = self.schemas.read().peek(&key).cloned();
        if initial.is_some() && !force_update {
            trace_debug!(
                target: "milvus_sdk::schema_cache",
                endpoint = %key.0,
                database = %key.1,
                collection = %key.2,
                "schema cache hit"
            );
            return Ok(initial.expect("cached schema exists"));
        }
        trace_debug!(
            target: "milvus_sdk::schema_cache",
            endpoint = %key.0,
            database = %key.1,
            collection = %key.2,
            force_update,
            "schema cache miss"
        );

        let (state, is_loader) = {
            let mut loading = self.loading.lock();
            if let Some(state) = loading.get(&load_key) {
                (Arc::clone(state), false)
            } else {
                let state = Arc::new(SchemaLoadState::new());
                loading.insert(load_key.clone(), Arc::clone(&state));
                (state, true)
            }
        };
        if !is_loader {
            trace_debug!(
                target: "milvus_sdk::schema_cache",
                endpoint = %key.0,
                database = %key.1,
                collection = %key.2,
                "joining in-flight schema load"
            );
            return state.wait().await;
        }
        let mut load_guard = SchemaLoadGuard::new(self, load_key.clone(), Arc::clone(&state));

        let current = self.schemas.read().peek(&key).cloned();
        if current.is_some()
            && (!force_update
                || match (initial.as_ref(), current.as_ref()) {
                    (Some(initial), Some(current)) => !Arc::ptr_eq(initial, current),
                    (None, Some(_)) => true,
                    _ => false,
                })
        {
            let current = current.expect("cached schema exists");
            load_guard.finish(Ok(Arc::clone(&current)));
            return Ok(current);
        }

        match loader().await {
            Ok(response) => {
                let response = Arc::new(response);
                {
                    let mut schemas = self.schemas.write();
                    if !state.invalidated.load(Ordering::Acquire) {
                        schemas.put(key.clone(), Arc::clone(&response));
                    }
                }
                load_guard.finish(Ok(Arc::clone(&response)));
                trace_debug!(
                    target: "milvus_sdk::schema_cache",
                    endpoint = %key.0,
                    database = %key.1,
                    collection = %key.2,
                    cached = !state.invalidated.load(Ordering::Acquire),
                    "schema load completed"
                );
                Ok(response)
            }
            Err(error) => {
                trace_debug!(
                    target: "milvus_sdk::schema_cache",
                    endpoint = %key.0,
                    database = %key.1,
                    collection = %key.2,
                    "schema load failed"
                );
                load_guard.finish(Err(error.clone()));
                Err(error)
            }
        }
    }

    #[cfg(test)]
    pub(super) fn set(
        &self,
        endpoint: &str,
        database: &str,
        collection: &str,
        schema: milvus::DescribeCollectionResponse,
    ) -> Arc<milvus::DescribeCollectionResponse> {
        let key = cache_key(endpoint, database, collection);
        self.invalidate_load(&key);
        let schema = Arc::new(schema);
        self.schemas.write().put(key, Arc::clone(&schema));
        schema
    }

    pub(super) fn invalidate(&self, endpoint: &str, database: &str, collection: &str) {
        let key = cache_key(endpoint, database, collection);
        trace_debug!(
            target: "milvus_sdk::schema_cache",
            endpoint = %key.0,
            database = %key.1,
            collection = %key.2,
            "invalidating schema cache entry"
        );
        self.invalidate_load(&key);
        self.schemas.write().pop(&key);
    }

    pub(super) fn invalidate_database(&self, endpoint: &str, database: &str) {
        let endpoint = normalize_endpoint(endpoint);
        let database = database_name(database);
        {
            let mut loading = self.loading.lock();
            let keys = loading
                .keys()
                .filter(|key| {
                    let (entry_endpoint, entry_database, _) = &key.collection;
                    entry_endpoint == &endpoint && entry_database == database
                })
                .cloned()
                .collect::<Vec<_>>();
            for key in keys {
                if let Some(state) = loading.remove(&key) {
                    state.invalidated.store(true, Ordering::Release);
                }
            }
        }
        let mut schemas = self.schemas.write();
        let keys = schemas
            .iter()
            .filter(|((entry_endpoint, entry_database, _), _)| {
                entry_endpoint == &endpoint && entry_database == database
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in keys {
            schemas.pop(&key);
        }
    }

    #[allow(dead_code)]
    pub(super) fn clear(&self) {
        for (_, state) in self.loading.lock().drain() {
            state.invalidated.store(true, Ordering::Release);
        }
        self.schemas.write().clear();
    }

    #[allow(dead_code)]
    pub(super) fn size(&self) -> usize {
        self.schemas.read().len()
    }

    fn invalidate_load(&self, key: &CacheKey) {
        let mut loading = self.loading.lock();
        let keys = loading
            .keys()
            .filter(|load_key| &load_key.collection == key)
            .cloned()
            .collect::<Vec<_>>();
        for load_key in keys {
            if let Some(state) = loading.remove(&load_key) {
                state.invalidated.store(true, Ordering::Release);
            }
        }
    }

    fn finish_load(&self, key: &LoadKey, state: &Arc<SchemaLoadState>, result: SchemaLoadResult) {
        state.complete(result);
        let mut loading = self.loading.lock();
        if loading
            .get(key)
            .is_some_and(|current| Arc::ptr_eq(current, state))
        {
            loading.remove(key);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SchemaLoadGuard
///////////////////////////////////////////////////////////////////////////////
/// Removes and completes an elected loader's in-flight state if its future is dropped.
struct SchemaLoadGuard<'a> {
    cache: &'a SchemaCache,
    key: LoadKey,
    state: Arc<SchemaLoadState>,
    finished: bool,
}

impl<'a> SchemaLoadGuard<'a> {
    fn new(cache: &'a SchemaCache, key: LoadKey, state: Arc<SchemaLoadState>) -> Self {
        Self {
            cache,
            key,
            state,
            finished: false,
        }
    }

    fn finish(&mut self, result: SchemaLoadResult) {
        self.cache.finish_load(&self.key, &self.state, result);
        self.finished = true;
    }
}

impl Drop for SchemaLoadGuard<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.cache.finish_load(
                &self.key,
                &self.state,
                Err(Error::Cancelled("schema load".into())),
            );
        }
    }
}

type SchemaLoadResult = Result<Arc<milvus::DescribeCollectionResponse>>;

struct SchemaLoadState {
    invalidated: AtomicBool,
    result: Mutex<Option<SchemaLoadResult>>,
    notify: Notify,
}

impl SchemaLoadState {
    fn new() -> Self {
        Self {
            invalidated: AtomicBool::new(false),
            result: Mutex::new(None),
            notify: Notify::new(),
        }
    }

    fn complete(&self, result: SchemaLoadResult) {
        *self.result.lock() = Some(result);
        self.notify.notify_waiters();
    }

    async fn wait(&self) -> SchemaLoadResult {
        loop {
            if let Some(result) = self.result.lock().clone() {
                return result;
            }
            let notified = self.notify.notified();
            if let Some(result) = self.result.lock().clone() {
                return result;
            }
            notified.await;
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CollectionTsCache
///////////////////////////////////////////////////////////////////////////////
/// Process-wide last-DML-timestamp cache keyed by endpoint, database, and collection.
///
/// LRU-bounded like pymilvus's `CollectionTsCache` (default capacity 4096); least-recently-used
/// entries are evicted when the cache exceeds its capacity.
///
/// Evicting a timestamp drops the client's last-write guarantee: SESSION (and default) consistency
/// reads fall back to `ts=1` for that collection until the next DML records a new timestamp.
/// Unlike the schema cache, an evicted timestamp cannot be reloaded from the server, so
/// applications relying on SESSION consistency across many collections should size the cache with
/// [`CacheConfig::set_capacity`] accordingly.
///
/// `set` keeps the maximum timestamp per key and, unlike pymilvus's `get`-based compare, does not
/// refresh LRU recency when an incoming timestamp is not greater; only a real update marks the key
/// as recently used. Reads (`get`/`guarantee_timestamp`) use `peek` under a read lock and likewise
/// never refresh recency, so a collection that is read constantly under Session consistency but
/// written rarely can be evicted by writes to *other* collections once the cache exceeds its
/// capacity, silently falling back to `ts=1` until the next DML. This is the same read-contention
/// trade-off as the schema cache: actively-read-but-rarely-written collections are not protected
/// from eviction, unlike pymilvus where reads go through the recency-refreshing `LRUCache.get`.
pub(super) struct CollectionTsCache {
    timestamps: RwLock<LruCache<CacheKey, u64>>,
}

impl CollectionTsCache {
    fn new() -> Self {
        Self::with_capacity(CacheConfig::DEFAULT_CAPACITY)
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            timestamps: RwLock::new(cache_with_capacity(capacity)),
        }
    }

    /// Resizes the timestamp cache to `capacity`, evicting least-recently-used entries.
    pub(super) fn set_capacity(&self, capacity: usize) -> Result<()> {
        resize_cache(&self.timestamps, capacity)
    }

    /// Returns the maximum number of timestamp entries the cache can hold.
    #[allow(dead_code)]
    pub(super) fn get_capacity(&self) -> usize {
        cache_capacity(&self.timestamps)
    }

    pub(super) fn set(&self, endpoint: &str, database: &str, collection: &str, timestamp: u64) {
        if timestamp == 0 {
            return;
        }
        let key = cache_key(endpoint, database, collection);
        let mut timestamps = self.timestamps.write();
        let previous = timestamps.peek(&key).copied();
        let updated = previous.is_none_or(|current| timestamp > current);
        if updated {
            #[cfg(feature = "tracing")]
            let evicted = timestamps.push(key, timestamp);
            #[cfg(not(feature = "tracing"))]
            let _ = timestamps.push(key, timestamp);
            // `push` returns a pair both when it replaces an existing key's value and when it
            // evicts the LRU entry for a brand-new key; only a previously-absent key can evict.
            #[cfg(feature = "tracing")]
            if previous.is_none() {
                if let Some((
                    (evicted_endpoint, evicted_database, evicted_collection),
                    evicted_ts,
                )) = evicted
                {
                    trace_warn!(
                        target: "milvus_sdk::timestamp_cache",
                        endpoint = %evicted_endpoint,
                        database = %evicted_database,
                        collection = %evicted_collection,
                        evicted_ts,
                        capacity = timestamps.cap().get(),
                        "timestamp cache entry evicted; SESSION/default reads for this collection fall back to ts=1 until the next DML"
                    );
                }
            }
        }
        trace_debug!(target: "milvus_sdk::timestamp_cache", endpoint = %normalize_endpoint(endpoint), database = %database_name(database), collection, previous = ?previous, timestamp, entries = timestamps.len(), "DML timestamp cache updated");
    }

    pub(super) fn get(&self, endpoint: &str, database: &str, collection: &str) -> Option<u64> {
        let key = cache_key(endpoint, database, collection);
        self.timestamps.read().peek(&key).copied()
    }

    pub(super) fn invalidate(&self, endpoint: &str, database: &str, collection: &str) {
        let key = cache_key(endpoint, database, collection);
        let mut timestamps = self.timestamps.write();
        let _removed = timestamps.pop(&key).is_some();
        trace_debug!(target: "milvus_sdk::timestamp_cache", endpoint = %key.0, database = %key.1, collection = %key.2, removed = _removed, entries = timestamps.len(), "DML timestamp cache entry invalidated");
    }

    pub(super) fn copy(
        &self,
        endpoint: &str,
        database: &str,
        source_collection: &str,
        target_collection: &str,
    ) {
        let source_key = cache_key(endpoint, database, source_collection);
        let target_key = cache_key(endpoint, database, target_collection);
        if source_key == target_key {
            return;
        }

        let mut timestamps = self.timestamps.write();
        let timestamp = timestamps
            .get(&source_key)
            .copied()
            .unwrap_or_default()
            .max(timestamps.get(&target_key).copied().unwrap_or_default());
        if timestamp != 0 {
            timestamps.put(target_key, timestamp);
        }
        trace_debug!(target: "milvus_sdk::timestamp_cache", endpoint = %normalize_endpoint(endpoint), database = %database_name(database), source_collection, target_collection, timestamp, entries = timestamps.len(), "DML timestamp cache entry copied");
    }

    pub(super) fn invalidate_database(&self, endpoint: &str, database: &str) {
        let endpoint = normalize_endpoint(endpoint);
        let database = database_name(database);
        let mut timestamps = self.timestamps.write();
        let keys = timestamps
            .iter()
            .filter(|((entry_endpoint, entry_database, _), _)| {
                entry_endpoint == &endpoint && entry_database == database
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in &keys {
            timestamps.pop(key);
        }
        trace_debug!(target: "milvus_sdk::timestamp_cache", endpoint = %endpoint, database, removed = keys.len(), entries = timestamps.len(), "DML timestamp database entries invalidated");
    }

    pub(super) fn move_ts(
        &self,
        endpoint: &str,
        old_database: &str,
        old_collection: &str,
        new_database: &str,
        new_collection: &str,
    ) {
        let old_key = cache_key(endpoint, old_database, old_collection);
        let new_key = cache_key(endpoint, new_database, new_collection);
        if old_key == new_key {
            return;
        }

        let mut timestamps = self.timestamps.write();
        let timestamp = timestamps
            .get(&old_key)
            .copied()
            .unwrap_or_default()
            .max(timestamps.get(&new_key).copied().unwrap_or_default());
        timestamps.pop(&old_key);
        timestamps.pop(&new_key);
        if timestamp != 0 {
            timestamps.put(new_key, timestamp);
        }
        trace_debug!(target: "milvus_sdk::timestamp_cache", endpoint = %normalize_endpoint(endpoint), old_database = %database_name(old_database), old_collection, new_database = %database_name(new_database), new_collection, timestamp, entries = timestamps.len(), "DML timestamp cache entry moved");
    }

    #[allow(dead_code)]
    pub(super) fn clear(&self) {
        self.timestamps.write().clear();
    }

    #[allow(dead_code)]
    pub(super) fn size(&self) -> usize {
        self.timestamps.read().len()
    }

    pub(super) fn guarantee_timestamp(
        &self,
        endpoint: &str,
        database: &str,
        collection: &str,
        consistency: ConsistencyLevel,
    ) -> u64 {
        #[allow(deprecated)]
        match consistency {
            ConsistencyLevel::Strong | ConsistencyLevel::Customized => 0,
            ConsistencyLevel::Eventually => 1,
            ConsistencyLevel::Bounded => 2,
            ConsistencyLevel::Session => self.get(endpoint, database, collection).unwrap_or(1),
        }
    }
}

lazy_static! {
    pub(super) static ref SCHEMA_CACHE: SchemaCache = SchemaCache::new();
    pub(super) static ref COLLECTION_TS_CACHE: CollectionTsCache = CollectionTsCache::new();
    /// Serializes concurrent [`CacheConfig::set_capacity`] calls so both caches always share one capacity.
    static ref CACHE_CAPACITY_LOCK: Mutex<()> = Mutex::new(());
}

///////////////////////////////////////////////////////////////////////////////
// CacheConfig
///////////////////////////////////////////////////////////////////////////////
/// Static entry points to tune the process-wide schema and last-DML-timestamp caches.
///
/// Both caches are LRU-bounded; the least-recently-used entries are evicted when the cache exceeds
/// the configured capacity. The default is [`CacheConfig::DEFAULT_CAPACITY`] entries, matching
/// pymilvus; the Java and C++ SDKs
/// leave the timestamp cache unbounded, so this bound is a deliberate pymilvus alignment.
///
/// Call [`CacheConfig::set_capacity`] before heavy workloads to raise or lower the bound; shrinking
/// below the current size evicts entries immediately.
pub struct CacheConfig;

impl CacheConfig {
    /// The default capacity of the process-wide schema and last-DML-timestamp caches, in entries.
    pub const DEFAULT_CAPACITY: usize = 4_096;

    /// Sets the capacity of the process-wide schema and last-DML-timestamp caches.
    ///
    /// The two resizes happen atomically under a shared lock, so concurrent callers cannot leave the
    /// caches at different capacities.
    ///
    /// Shrinking evicts least-recently-used timestamp entries; SESSION and default-consistency reads
    /// then fall back to `ts=1` for the evicted collections until the next DML records a new
    /// timestamp, so deployments touching more than `capacity` collections can silently lose
    /// read-your-writes guarantees for the affected collections.
    ///
    /// Returns a [`Error::Validation`] error when `capacity` is zero.
    pub fn set_capacity(capacity: usize) -> Result<()> {
        let _guard = CACHE_CAPACITY_LOCK.lock();
        SCHEMA_CACHE.set_capacity(capacity)?;
        COLLECTION_TS_CACHE.set_capacity(capacity)?;
        Ok(())
    }

    /// Returns the capacity of the process-wide schema and last-DML-timestamp caches.
    ///
    /// Both caches share the same maximum entry count, defaulting to 4096, matching pymilvus.
    pub fn get_capacity() -> usize {
        SCHEMA_CACHE.get_capacity()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
pub(crate) mod tests {
    use super::{
        normalize_endpoint, CollectionTsCache, SchemaCache, SchemaLoadScope, COLLECTION_TS_CACHE,
        SCHEMA_CACHE,
    };
    use crate::proto::milvus;
    use crate::v2::error::{ConversionError, Error};
    use crate::v2::types::ConsistencyLevel;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::Notify;

    /// Serializes tests that mutate the process-wide global caches, which run in parallel.
    pub(crate) static GLOBAL_CACHE_TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

    #[test]
    fn endpoint_normalization_uses_transport_default_ports() {
        assert_eq!(
            normalize_endpoint("http://MILVUS.EXAMPLE.COM"),
            "milvus.example.com:80"
        );
        assert_eq!(
            normalize_endpoint("milvus.example.com"),
            "milvus.example.com:80"
        );
        assert_eq!(
            normalize_endpoint("https://MILVUS.EXAMPLE.COM/path"),
            "milvus.example.com:443"
        );
        assert_eq!(
            normalize_endpoint("http://MILVUS.EXAMPLE.COM:19530/path"),
            "milvus.example.com:19530"
        );
    }

    #[test]
    fn caches_normalize_database_and_isolate_endpoints() {
        let _guard = GLOBAL_CACHE_TEST_LOCK.lock();
        let first_endpoint = "http://CACHE-ISOLATION-A:19530/path";
        let normalized_first_endpoint = "cache-isolation-a:19530";
        let second_endpoint = "cache-isolation-b";
        SCHEMA_CACHE.invalidate(first_endpoint, "", "books");
        SCHEMA_CACHE.invalidate(second_endpoint, "default", "books");
        COLLECTION_TS_CACHE.invalidate(first_endpoint, "", "books");
        COLLECTION_TS_CACHE.invalidate(second_endpoint, "default", "books");

        SCHEMA_CACHE.set(
            first_endpoint,
            "",
            "books",
            milvus::DescribeCollectionResponse {
                collection_id: 1,
                ..Default::default()
            },
        );
        let first_schema = SCHEMA_CACHE
            .get(normalized_first_endpoint, "", "books")
            .expect("cached schema");
        let shared_schema = SCHEMA_CACHE
            .get(normalized_first_endpoint, "default", "books")
            .expect("same normalized cache entry");
        assert!(Arc::ptr_eq(&first_schema, &shared_schema));
        COLLECTION_TS_CACHE.set(first_endpoint, "", "books", 100);
        COLLECTION_TS_CACHE.set(normalized_first_endpoint, "default", "books", 50);
        COLLECTION_TS_CACHE.set(second_endpoint, "default", "books", 200);

        assert_eq!(
            SCHEMA_CACHE
                .get(normalized_first_endpoint, "default", "books")
                .map(|schema| schema.collection_id),
            Some(1)
        );
        assert!(SCHEMA_CACHE
            .get(second_endpoint, "default", "books")
            .is_none());
        assert_eq!(
            COLLECTION_TS_CACHE.get(normalized_first_endpoint, "default", "books"),
            Some(100)
        );
        assert_eq!(
            COLLECTION_TS_CACHE.guarantee_timestamp(
                second_endpoint,
                "default",
                "books",
                ConsistencyLevel::Session,
            ),
            200
        );

        SCHEMA_CACHE.invalidate_database(normalized_first_endpoint, "");
        COLLECTION_TS_CACHE.invalidate_database(normalized_first_endpoint, "");
        assert!(SCHEMA_CACHE
            .get(normalized_first_endpoint, "default", "books")
            .is_none());
        assert_eq!(
            COLLECTION_TS_CACHE.get(normalized_first_endpoint, "default", "books"),
            None
        );
        COLLECTION_TS_CACHE.invalidate(second_endpoint, "default", "books");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn schema_cache_shares_one_inflight_load() {
        let cache = Arc::new(SchemaCache::new());
        let loads = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let scope = Arc::new(SchemaLoadScope::new());
        let mut tasks = Vec::new();

        for _ in 0..8 {
            let cache = Arc::clone(&cache);
            let loads = Arc::clone(&loads);
            let started = Arc::clone(&started);
            let release = Arc::clone(&release);
            let scope = Arc::clone(&scope);
            tasks.push(tokio::spawn(async move {
                cache
                    .get_or_load("host:19530", "db", "books", false, &scope, || async move {
                        loads.fetch_add(1, Ordering::SeqCst);
                        started.notify_one();
                        release.notified().await;
                        Ok(milvus::DescribeCollectionResponse {
                            collection_id: 10,
                            ..Default::default()
                        })
                    })
                    .await
            }));
        }

        started.notified().await;
        tokio::task::yield_now().await;
        release.notify_one();
        for task in tasks {
            assert_eq!(
                task.await.expect("schema load task").unwrap().collection_id,
                10
            );
        }
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn schema_cache_waiters_preserve_conversion_failures() {
        let cache = Arc::new(SchemaCache::new());
        let scope = Arc::new(SchemaLoadScope::new());
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());

        let loader = {
            let cache = Arc::clone(&cache);
            let scope = Arc::clone(&scope);
            let started = Arc::clone(&started);
            let release = Arc::clone(&release);
            tokio::spawn(async move {
                cache
                    .get_or_load("host:19530", "db", "books", false, &scope, || async move {
                        started.notify_one();
                        release.notified().await;
                        let error: Error = serde_json::from_str::<serde_json::Value>("{")
                            .unwrap_err()
                            .into();
                        Err(error)
                    })
                    .await
            })
        };

        started.notified().await;
        let waiter = {
            let cache = Arc::clone(&cache);
            let scope = Arc::clone(&scope);
            tokio::spawn(async move {
                cache
                    .get_or_load("host:19530", "db", "books", false, &scope, || async {
                        panic!("waiter must join the in-flight schema load")
                    })
                    .await
            })
        };
        tokio::task::yield_now().await;
        release.notify_one();

        for result in [loader.await.unwrap(), waiter.await.unwrap()] {
            assert!(matches!(
                result,
                Err(Error::Conversion(ConversionError::Json(_)))
            ));
        }
    }

    #[tokio::test]
    async fn schema_cache_invalidation_during_load_prevents_repopulation() {
        let cache = Arc::new(SchemaCache::new());
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let scope = Arc::new(SchemaLoadScope::new());
        let task = {
            let cache = Arc::clone(&cache);
            let started = Arc::clone(&started);
            let release = Arc::clone(&release);
            let scope = Arc::clone(&scope);
            tokio::spawn(async move {
                cache
                    .get_or_load("host:19530", "db", "books", false, &scope, || async move {
                        started.notify_one();
                        release.notified().await;
                        Ok(milvus::DescribeCollectionResponse {
                            collection_id: 11,
                            ..Default::default()
                        })
                    })
                    .await
            })
        };

        started.notified().await;
        cache.invalidate("host:19530", "db", "books");
        release.notify_one();
        assert_eq!(task.await.unwrap().unwrap().collection_id, 11);
        assert!(cache.get("host:19530", "db", "books").is_none());
    }

    #[tokio::test]
    async fn schema_cache_recovers_after_the_elected_loader_is_aborted() {
        let cache = Arc::new(SchemaCache::new());
        let scope = Arc::new(SchemaLoadScope::new());
        let started = Arc::new(Notify::new());
        let loader = {
            let cache = Arc::clone(&cache);
            let scope = Arc::clone(&scope);
            let started = Arc::clone(&started);
            tokio::spawn(async move {
                cache
                    .get_or_load("host:19530", "db", "books", false, &scope, || async move {
                        started.notify_one();
                        std::future::pending::<Result<milvus::DescribeCollectionResponse, Error>>()
                            .await
                    })
                    .await
            })
        };

        started.notified().await;
        loader.abort();
        assert!(loader
            .await
            .expect_err("loader task must be aborted")
            .is_cancelled());
        assert!(cache.loading.lock().is_empty());

        let recovered = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            cache.get_or_load("host:19530", "db", "books", false, &scope, || async {
                Ok(milvus::DescribeCollectionResponse {
                    collection_id: 12,
                    ..Default::default()
                })
            }),
        )
        .await
        .expect("a later caller must not wait on the aborted loader")
        .unwrap();
        assert_eq!(recovered.collection_id, 12);
    }

    #[tokio::test]
    async fn schema_cache_does_not_share_inflight_loads_across_client_scopes() {
        let cache = Arc::new(SchemaCache::new());
        let first_scope = Arc::new(SchemaLoadScope::new());
        let second_scope = SchemaLoadScope::new();
        let first_started = Arc::new(Notify::new());
        let release_first = Arc::new(Notify::new());

        let first = {
            let cache = Arc::clone(&cache);
            let first_scope = Arc::clone(&first_scope);
            let first_started = Arc::clone(&first_started);
            let release_first = Arc::clone(&release_first);
            tokio::spawn(async move {
                cache
                    .get_or_load(
                        "host:19530",
                        "db",
                        "books",
                        false,
                        &first_scope,
                        || async move {
                            first_started.notify_one();
                            release_first.notified().await;
                            Ok(milvus::DescribeCollectionResponse {
                                collection_id: 100,
                                ..Default::default()
                            })
                        },
                    )
                    .await
            })
        };

        first_started.notified().await;
        let second = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            cache.get_or_load(
                "host:19530",
                "db",
                "books",
                false,
                &second_scope,
                || async {
                    Ok(milvus::DescribeCollectionResponse {
                        collection_id: 200,
                        ..Default::default()
                    })
                },
            ),
        )
        .await
        .expect("a different client scope must run its own loader")
        .unwrap();
        assert_eq!(second.collection_id, 200);

        release_first.notify_one();
        assert_eq!(first.await.unwrap().unwrap().collection_id, 100);
    }

    #[tokio::test]
    async fn schema_cache_force_refreshes_and_preserves_failure_types() {
        let cache = SchemaCache::new();
        let scope = SchemaLoadScope::new();
        let loads = Arc::new(AtomicUsize::new(0));
        let first = cache
            .get_or_load("host:19530", "db", "books", false, &scope, {
                let loads = Arc::clone(&loads);
                || async move {
                    loads.fetch_add(1, Ordering::SeqCst);
                    Ok(milvus::DescribeCollectionResponse {
                        collection_id: 1,
                        ..Default::default()
                    })
                }
            })
            .await
            .unwrap();
        let cached = cache
            .get_or_load(
                "http://HOST:19530/path",
                "db",
                "books",
                false,
                &scope,
                || async { panic!("cached schema must not reload") },
            )
            .await
            .unwrap();
        assert!(Arc::ptr_eq(&first, &cached));

        let refreshed = cache
            .get_or_load("host:19530", "db", "books", true, &scope, {
                let loads = Arc::clone(&loads);
                || async move {
                    loads.fetch_add(1, Ordering::SeqCst);
                    Ok(milvus::DescribeCollectionResponse {
                        collection_id: 2,
                        ..Default::default()
                    })
                }
            })
            .await
            .unwrap();
        assert_eq!(refreshed.collection_id, 2);
        assert_eq!(loads.load(Ordering::SeqCst), 2);

        let failure = cache
            .get_or_load("host:19530", "db", "failed", false, &scope, || async {
                Err(Error::validation("schema".into(), "failed load".into()))
            })
            .await
            .expect_err("schema load must fail");
        assert!(matches!(failure, Error::Validation(_)));
    }

    #[test]
    fn schema_cache_supports_capacity_size_and_clear() {
        let cache = SchemaCache::with_capacity(2);
        for collection_id in 1..=2 {
            cache.set(
                "host:19530",
                "db",
                &format!("collection-{collection_id}"),
                milvus::DescribeCollectionResponse {
                    collection_id,
                    ..Default::default()
                },
            );
        }
        cache.set(
            "host:19530",
            "db",
            "collection-1",
            milvus::DescribeCollectionResponse {
                collection_id: 1,
                ..Default::default()
            },
        );
        cache.set(
            "host:19530",
            "db",
            "collection-3",
            milvus::DescribeCollectionResponse {
                collection_id: 3,
                ..Default::default()
            },
        );

        assert_eq!(cache.size(), 2);
        assert!(cache.get("host:19530", "db", "collection-1").is_some());
        assert!(cache.get("host:19530", "db", "collection-2").is_none());
        assert!(cache.get("host:19530", "db", "collection-3").is_some());
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    #[should_panic(expected = "cache capacity must be greater than zero")]
    fn schema_cache_rejects_zero_capacity() {
        let _ = SchemaCache::with_capacity(0);
    }

    #[test]
    fn collection_timestamps_evict_lru_and_move_atomically() {
        let cache = CollectionTsCache::with_capacity(16);
        for index in 0..16_u64 {
            cache.set(
                "host:19530",
                "db",
                &format!("collection-{index}"),
                index + 1,
            );
        }
        assert_eq!(cache.size(), 16);
        cache.set("host:19530", "db", "collection-16", 17);
        assert_eq!(cache.get("host:19530", "db", "collection-0"), None);
        assert_eq!(cache.get("host:19530", "db", "collection-16"), Some(17));

        cache.set("host:19530", "source", "old", 100);
        cache.set("host:19530", "target", "new", 200);
        cache.move_ts("host:19530", "source", "old", "target", "new");
        assert_eq!(cache.get("host:19530", "source", "old"), None);
        assert_eq!(cache.get("host:19530", "target", "new"), Some(200));

        cache.set("host:19530", "source", "old", 300);
        cache.move_ts("host:19530", "source", "old", "target", "new");
        assert_eq!(cache.get("host:19530", "target", "new"), Some(300));
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn collection_timestamps_default_capacity_matches_pymilvus() {
        let cache = CollectionTsCache::new();
        for index in 0..crate::v2::CacheConfig::DEFAULT_CAPACITY as u64 {
            cache.set(
                "host:19530",
                "db",
                &format!("collection-{index}"),
                index + 1,
            );
        }
        assert_eq!(cache.size(), crate::v2::CacheConfig::DEFAULT_CAPACITY);
        cache.set("host:19530", "db", "collection-4096", 4_097);
        assert_eq!(cache.get("host:19530", "db", "collection-0"), None);
    }

    #[test]
    fn cache_capacity_resizes_and_rejects_zero() {
        let cache = CollectionTsCache::with_capacity(4);
        assert_eq!(cache.get_capacity(), 4);
        for index in 0..4_u64 {
            cache.set(
                "host:19530",
                "db",
                &format!("collection-{index}"),
                index + 1,
            );
        }
        cache.set_capacity(2).expect("resize to two entries");
        assert_eq!(cache.get_capacity(), 2);
        assert_eq!(cache.size(), 2);
        assert!(cache.get("host:19530", "db", "collection-0").is_none());
        assert!(cache.get("host:19530", "db", "collection-1").is_none());
        assert!(cache.get("host:19530", "db", "collection-3").is_some());
        assert!(matches!(cache.set_capacity(0), Err(Error::Validation(_))));
        assert!(matches!(
            crate::v2::CacheConfig::set_capacity(0),
            Err(Error::Validation(_))
        ));
    }

    #[test]
    fn timestamp_set_skips_smaller_values_without_disturbing_lru_order() {
        let cache = CollectionTsCache::with_capacity(3);
        cache.set("host:19530", "db", "a", 1);
        cache.set("host:19530", "db", "b", 2);
        cache.set("host:19530", "db", "c", 3);

        cache.set("host:19530", "db", "a", 10);
        assert_eq!(cache.get("host:19530", "db", "a"), Some(10));

        cache.set("host:19530", "db", "b", 1);
        assert_eq!(cache.size(), 3);

        cache.set("host:19530", "db", "d", 4);
        assert!(cache.get("host:19530", "db", "b").is_none());
        assert_eq!(cache.get("host:19530", "db", "a"), Some(10));
        assert_eq!(cache.get("host:19530", "db", "c"), Some(3));
        assert_eq!(cache.get("host:19530", "db", "d"), Some(4));
    }

    #[test]
    fn public_cache_capacity_roundtrips_the_process_wide_caches() {
        let _guard = GLOBAL_CACHE_TEST_LOCK.lock();
        let original = crate::v2::CacheConfig::get_capacity();
        crate::v2::CacheConfig::set_capacity(8192).expect("grow global caches");
        assert_eq!(crate::v2::CacheConfig::get_capacity(), 8192);
        assert_eq!(SCHEMA_CACHE.get_capacity(), 8192);
        assert_eq!(COLLECTION_TS_CACHE.get_capacity(), 8192);

        crate::v2::CacheConfig::set_capacity(original).expect("restore capacity");
        assert_eq!(crate::v2::CacheConfig::get_capacity(), original);
    }

    #[test]
    fn concurrent_set_capacity_keeps_both_caches_in_lockstep() {
        let _guard = GLOBAL_CACHE_TEST_LOCK.lock();
        let original = crate::v2::CacheConfig::get_capacity();
        let mut handles = Vec::new();
        for thread_index in 0..8 {
            handles.push(std::thread::spawn(move || {
                for round in 0..50 {
                    let capacity = 64 + ((thread_index + round) % 8) * 64;
                    crate::v2::CacheConfig::set_capacity(capacity)
                        .expect("valid capacity during race");
                }
            }));
        }
        for handle in handles {
            handle.join().expect("racing thread");
        }
        assert_eq!(
            SCHEMA_CACHE.get_capacity(),
            COLLECTION_TS_CACHE.get_capacity(),
            "both caches must share the same capacity after all racing set_capacity calls"
        );
        crate::v2::CacheConfig::set_capacity(original).expect("restore capacity");
        assert_eq!(crate::v2::CacheConfig::get_capacity(), original);
    }

    #[test]
    fn evicted_timestamp_falls_back_to_one_for_session_consistency() {
        let cache = CollectionTsCache::with_capacity(1);
        cache.set("host:19530", "db", "kept", 100);
        assert_eq!(
            cache.guarantee_timestamp("host:19530", "db", "kept", ConsistencyLevel::Session),
            100
        );

        cache.set("host:19530", "db", "other", 200);
        assert_eq!(cache.get("host:19530", "db", "kept"), None);
        assert_eq!(
            cache.guarantee_timestamp("host:19530", "db", "kept", ConsistencyLevel::Session),
            1
        );
    }
}
