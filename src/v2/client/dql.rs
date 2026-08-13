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

//! ClientV2 query and search operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::{request, response};

impl ClientV2 {
    /// Queries entities that match a filter expression or a typed primary-key selection.
    ///
    /// Output fields and consistency are controlled by the request. Session consistency uses the
    /// endpoint/database/collection timestamp recorded by successful DML; other consistency levels
    /// use the corresponding Milvus guarantee semantics. The response owns decoded rows, while
    /// its row iterator provides a borrowing traversal for allocation-sensitive callers.
    pub async fn query(
        &self,
        request: request::dql::QueryRequest,
    ) -> Result<response::dql::QueryResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let guarantee = self
            .deduce_guarantee_timestamp(&database, &collection, request.consistency_level)
            .await?;
        let primary_field = if request.ids.is_empty() {
            None
        } else {
            Some(self.primary_field_name(&database, &collection).await?)
        };
        let raw = request.into_proto(&database, primary_field.as_deref(), guarantee)?;
        let response = rpc_with_retry!(self, query, raw)?;
        status_to_result(&response.status)?;
        response::dql::QueryResponse::from_proto(response)
    }

    /// Retrieves entities by their primary-key values.
    ///
    /// The client resolves the collection's primary-key field from its schema, so callers provide
    /// IDs rather than a protobuf field name. The request cannot combine IDs with a filter; use
    /// [`ClientV2::query`] for expression-based selection.
    pub async fn get(
        &self,
        request: request::dql::GetRequest,
    ) -> Result<response::dql::GetResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let guarantee = self
            .deduce_guarantee_timestamp(&database, &collection, request.consistency_level)
            .await?;
        let primary_field = self.primary_field_name(&database, &collection).await?;
        let raw = request.into_proto(&database, &primary_field, guarantee)?;
        let response = rpc_with_retry!(self, query, raw)?;
        status_to_result(&response.status)?;
        response::dql::QueryResponse::from_proto(response)
    }

    /// Searches vector fields and returns ranked hits for each query vector.
    ///
    /// The collection must have a compatible vector index or be loaded according to the server's
    /// search requirements. Search consistency follows the request and the shared DML timestamp
    /// cache; decoded hits expose IDs, scores, and requested output fields.
    pub async fn search(
        &self,
        request: request::dql::SearchRequest,
    ) -> Result<response::dql::SearchResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let guarantee = self
            .deduce_guarantee_timestamp(&database, &collection, request.consistency_level)
            .await?;
        let raw = request.into_proto(&database, guarantee)?;
        let response = rpc_with_retry!(self, search, raw)?;
        status_to_result(&response.status)?;
        response::dql::SearchResponse::from_proto(response)
    }

    /// Executes multiple vector searches and combines them with the requested reranking strategy.
    ///
    /// Each child search must use compatible collection fields and query-vector dimensions. The
    /// returned response preserves one result set per query vector after server-side reranking.
    pub async fn hybrid_search(
        &self,
        request: request::dql::HybridSearchRequest,
    ) -> Result<response::dql::HybridSearchResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection = request.collection_name.clone();
        let guarantee = self
            .deduce_guarantee_timestamp(&database, &collection, request.consistency_level)
            .await?;
        let raw = request.into_proto(&database, guarantee)?;
        let response = rpc_with_retry!(self, hybrid_search, raw)?;
        status_to_result(&response.status)?;
        response::dql::SearchResponse::from_proto(response)
    }

    async fn primary_field_name(&self, database: &str, collection: &str) -> Result<String> {
        let description = self
            .get_collection_description(database, collection)
            .await?;
        description
            .schema
            .as_ref()
            .and_then(|schema| schema.fields.iter().find(|field| field.is_primary_key))
            .map(|field| field.name.clone())
            .ok_or_else(|| Error::MalformedResponse("collection schema has no primary key".into()))
    }
}
