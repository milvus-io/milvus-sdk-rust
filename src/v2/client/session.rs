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

//! Cluster-scoped DQL session views over a parent [`ClientV2`].

use super::{ClientV2, QueryIterator, SearchIterator};
use crate::v2::error::{Error, Result};
use crate::v2::request::dql::{
    GetRequest, HybridSearchRequest, QueryIteratorRequest, QueryRequest, SearchIteratorRequest,
    SearchRequest,
};
use crate::v2::response::dql::{GetResponse, QueryResponse, SearchResponse};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A cluster-scoped view of a [`ClientV2`] exposing the DQL surface only.
///
/// Created through [`ClientV2::session`], the session shares the parent
/// client's channel, selected database, RPC settings, and caches, and routes
/// every DQL request to a target global-cluster identifier. It does not expose
/// schema, DML, or administration operations.
///
/// [`MilvusClientV2Session::close`] marks the session closed and makes every
/// subsequent call fail, regardless of which clone of the session is used.
///////////////////////////////////////////////////////////////////////////////
// MilvusClientV2Session
///////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct MilvusClientV2Session {
    client: ClientV2,
    cluster_id: String,
    closed: Arc<AtomicBool>,
}

impl MilvusClientV2Session {
    pub(crate) fn new(client: ClientV2, cluster_id: String) -> Self {
        Self {
            client,
            cluster_id,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the target cluster identifier.
    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    /// Searches vector fields in the target cluster.
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        self.ensure_open()?;
        self.client
            .search_with_cluster(request, &self.cluster_id)
            .await
    }

    /// Executes multiple vector searches combined with a reranking strategy in
    /// the target cluster.
    pub async fn hybrid_search(&self, request: HybridSearchRequest) -> Result<SearchResponse> {
        self.ensure_open()?;
        self.client
            .hybrid_search_with_cluster(request, &self.cluster_id)
            .await
    }

    /// Queries entities in the target cluster.
    pub async fn query(&self, request: QueryRequest) -> Result<QueryResponse> {
        self.ensure_open()?;
        self.client
            .query_with_cluster(request, &self.cluster_id)
            .await
    }

    /// Retrieves entities by their primary-key values in the target cluster.
    pub async fn get(&self, request: GetRequest) -> Result<GetResponse> {
        self.ensure_open()?;
        self.client
            .get_with_cluster(request, &self.cluster_id)
            .await
    }

    /// Creates a query iterator bound to the target cluster.
    ///
    /// Closing the session also stops this iterator's subsequent pages.
    pub async fn query_iterator(&self, request: QueryIteratorRequest) -> Result<QueryIterator> {
        self.ensure_open()?;
        let mut iterator = self
            .client
            .query_iterator_with_cluster(request, &self.cluster_id)
            .await?;
        iterator.bind_session_close(Arc::clone(&self.closed));
        Ok(iterator)
    }

    /// Creates a search iterator bound to the target cluster.
    ///
    /// Closing the session also stops this iterator's subsequent pages.
    pub async fn search_iterator(&self, request: SearchIteratorRequest) -> Result<SearchIterator> {
        self.ensure_open()?;
        let mut iterator = self
            .client
            .search_iterator_with_cluster(request, &self.cluster_id)
            .await?;
        iterator.bind_session_close(Arc::clone(&self.closed));
        Ok(iterator)
    }

    /// Closes this session view without disconnecting the parent client.
    ///
    /// Every subsequent call on this session, including on clones created
    /// before the close, fails with an error.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    fn ensure_open(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            Err(Error::Unexpected(
                "MilvusClientV2 session is closed".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}
