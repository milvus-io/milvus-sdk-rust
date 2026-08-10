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

use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::types::ConsistencyLevel;
use lazy_static::lazy_static;
use lru::LruCache;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tonic::transport::Uri;

const DEFAULT_CAPACITY: usize = 4_096;
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

///////////////////////////////////////////////////////////////////////////////
// SchemaCache
///////////////////////////////////////////////////////////////////////////////
/// Process-wide collection-description cache keyed by endpoint, database, and collection.
/// Completed schemas are shared process-wide, while in-flight loads are coalesced only within
/// one client scope so another client's credentials or RPC deadline cannot control the load.
pub(super) struct SchemaCache {
    schemas: Mutex<LruCache<CacheKey, Arc<milvus::DescribeCollectionResponse>>>,
    loading: Mutex<HashMap<LoadKey, Arc<SchemaLoadState>>>,
}

impl SchemaCache {
    fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            schemas: Mutex::new(cache_with_capacity(capacity)),
            loading: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    pub(super) fn get(
        &self,
        endpoint: &str,
        database: &str,
        collection: &str,
    ) -> Option<Arc<milvus::DescribeCollectionResponse>> {
        let key = cache_key(endpoint, database, collection);
        self.schemas.lock().get(&key).cloned()
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
        let initial = self.schemas.lock().get(&key).cloned();
        if initial.is_some() && !force_update {
            return Ok(initial.expect("cached schema exists"));
        }

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
            return state.wait().await;
        }
        let mut load_guard = SchemaLoadGuard::new(self, load_key.clone(), Arc::clone(&state));

        let current = self.schemas.lock().get(&key).cloned();
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
                    let mut schemas = self.schemas.lock();
                    if !state.invalidated.load(Ordering::Acquire) {
                        schemas.put(key.clone(), Arc::clone(&response));
                    }
                }
                load_guard.finish(Ok(Arc::clone(&response)));
                Ok(response)
            }
            Err(error) => {
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
        self.schemas.lock().put(key, Arc::clone(&schema));
        schema
    }

    pub(super) fn invalidate(&self, endpoint: &str, database: &str, collection: &str) {
        let key = cache_key(endpoint, database, collection);
        self.invalidate_load(&key);
        self.schemas.lock().pop(&key);
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
        let mut schemas = self.schemas.lock();
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
        self.schemas.lock().clear();
    }

    #[allow(dead_code)]
    pub(super) fn size(&self) -> usize {
        self.schemas.lock().len()
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
pub(super) struct CollectionTsCache {
    timestamps: Mutex<HashMap<CacheKey, u64>>,
}

impl CollectionTsCache {
    fn new() -> Self {
        Self {
            timestamps: Mutex::new(HashMap::new()),
        }
    }

    pub(super) fn set(&self, endpoint: &str, database: &str, collection: &str, timestamp: u64) {
        if timestamp == 0 {
            return;
        }
        let key = cache_key(endpoint, database, collection);
        let mut timestamps = self.timestamps.lock();
        timestamps
            .entry(key)
            .and_modify(|current| *current = (*current).max(timestamp))
            .or_insert(timestamp);
    }

    pub(super) fn get(&self, endpoint: &str, database: &str, collection: &str) -> Option<u64> {
        let key = cache_key(endpoint, database, collection);
        self.timestamps.lock().get(&key).copied()
    }

    pub(super) fn invalidate(&self, endpoint: &str, database: &str, collection: &str) {
        let key = cache_key(endpoint, database, collection);
        self.timestamps.lock().remove(&key);
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

        let mut timestamps = self.timestamps.lock();
        let timestamp = timestamps
            .get(&source_key)
            .copied()
            .unwrap_or_default()
            .max(timestamps.get(&target_key).copied().unwrap_or_default());
        if timestamp != 0 {
            timestamps.insert(target_key, timestamp);
        }
    }

    pub(super) fn invalidate_database(&self, endpoint: &str, database: &str) {
        let endpoint = normalize_endpoint(endpoint);
        let database = database_name(database);
        let mut timestamps = self.timestamps.lock();
        timestamps.retain(|(entry_endpoint, entry_database, _), _| {
            entry_endpoint != &endpoint || entry_database != database
        });
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

        let mut timestamps = self.timestamps.lock();
        let timestamp = timestamps
            .get(&old_key)
            .copied()
            .unwrap_or_default()
            .max(timestamps.get(&new_key).copied().unwrap_or_default());
        timestamps.remove(&old_key);
        timestamps.remove(&new_key);
        if timestamp != 0 {
            timestamps.insert(new_key, timestamp);
        }
    }

    #[allow(dead_code)]
    pub(super) fn clear(&self) {
        self.timestamps.lock().clear();
    }

    #[allow(dead_code)]
    pub(super) fn size(&self) -> usize {
        self.timestamps.lock().len()
    }

    pub(super) fn guarantee_timestamp(
        &self,
        endpoint: &str,
        database: &str,
        collection: &str,
        consistency: ConsistencyLevel,
    ) -> u64 {
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
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
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
        cache
            .get("host:19530", "db", "collection-1")
            .expect("touch first schema");
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
    fn collection_timestamps_do_not_evict_and_move_atomically() {
        let cache = CollectionTsCache::new();
        for index in 0..5_000_u64 {
            cache.set(
                "host:19530",
                "db",
                &format!("collection-{index}"),
                index + 1,
            );
        }
        assert_eq!(cache.get("host:19530", "db", "collection-0"), Some(1));
        assert_eq!(
            cache.get("host:19530", "db", "collection-4999"),
            Some(5_000)
        );

        cache.set("host:19530", "source", "old", 100);
        cache.set("host:19530", "target", "new", 200);
        cache.move_ts("host:19530", "source", "old", "target", "new");
        assert_eq!(cache.get("host:19530", "source", "old"), None);
        assert_eq!(cache.get("host:19530", "target", "new"), Some(200));

        cache.set("host:19530", "source", "old", 300);
        cache.move_ts("host:19530", "source", "old", "target", "new");
        assert_eq!(cache.get("host:19530", "target", "new"), Some(300));
        assert_eq!(cache.size(), 5_001);
        cache.clear();
        assert_eq!(cache.size(), 0);
    }
}
