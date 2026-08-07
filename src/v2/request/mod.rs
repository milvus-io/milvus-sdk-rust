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

//! V2 request objects, grouped by Milvus functional area.
//!
//! Public requests are SDK domain objects. Generated protobuf messages are
//! created only by crate-private conversion methods in each area module.

pub mod alias;
pub mod cdc;
pub mod collection;
pub mod database;
pub mod dml;
pub mod dql;
pub mod index;
pub mod partition;
pub mod rbac;
pub mod resource_group;
pub mod utility;

mod validation;

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod validation_tests {
    use super::alias::CreateAliasRequest;
    use super::cdc::DumpMessagesRequest;
    use super::collection::{CreateCollectionRequest, CreateSimpleCollectionRequest};
    use super::dml::{InsertRequest, UpsertRequest};
    use super::dql::{QueryRequest, SearchRequest};
    use super::index::CreateIndexRequest;
    use super::partition::LoadPartitionsRequest;
    use super::rbac::{CreateUserRequest, GrantPrivilegeRequest};
    use super::resource_group::{TransferNodeRequest, UpdateResourceGroupsRequest};
    use super::utility::{FlushRequest, GetCompactionStateRequest};
    use crate::v2::types::{DataType, SearchVectors};

    #[test]
    fn builders_reject_missing_required_identifiers() {
        assert!(CreateAliasRequest::builder().build().is_err());
        assert!(CreateCollectionRequest::builder().build().is_err());
        assert!(QueryRequest::builder().build().is_err());
        assert!(CreateIndexRequest::builder().build().is_err());
        assert!(CreateUserRequest::builder().build().is_err());
        assert!(GrantPrivilegeRequest::builder().build().is_err());
    }

    #[test]
    fn builders_reject_invalid_ranges_and_empty_inputs() {
        assert!(CreateSimpleCollectionRequest::builder()
            .collection_name("books")
            .dimension(0)
            .build()
            .is_err());
        assert!(CreateSimpleCollectionRequest::builder()
            .collection_name("books")
            .dimension(128)
            .primary_field_type(DataType::Int32)
            .build()
            .is_err());
        assert!(SearchRequest::builder()
            .collection_name("books")
            .vectors(SearchVectors::Float(Vec::new()))
            .build()
            .is_err());
        assert!(LoadPartitionsRequest::builder()
            .collection_name("books")
            .build()
            .is_err());
        assert!(TransferNodeRequest::builder()
            .source_group("default")
            .target_group("analytics")
            .node_count(0)
            .build()
            .is_err());
        assert!(UpdateResourceGroupsRequest::builder().build().is_err());
        assert!(FlushRequest::builder().build().is_err());
        assert!(GetCompactionStateRequest::builder().build().is_err());
    }

    #[test]
    fn builders_reject_incomplete_dml_and_cdc_requests() {
        assert!(InsertRequest::builder()
            .collection_name("books")
            .build()
            .is_err());
        assert!(UpsertRequest::builder().build().is_err());
        assert!(DumpMessagesRequest::builder()
            .physical_channel("channel")
            .build()
            .is_err());
    }

    #[test]
    fn query_builder_preserves_iterator_unlimited_limit() {
        assert!(QueryRequest::builder()
            .collection_name("books")
            .limit(-1)
            .build()
            .is_ok());
        assert!(QueryRequest::builder()
            .collection_name("books")
            .limit(0)
            .build()
            .is_err());
    }

    #[test]
    fn builders_accept_omitted_or_empty_database_name() {
        let omitted = QueryRequest::builder()
            .collection_name("books")
            .build()
            .expect("omitted database name uses the default database");
        assert_eq!(omitted.database_name(), &None);

        let empty = QueryRequest::builder()
            .database_name("")
            .collection_name("books")
            .build()
            .expect("empty database name uses the default database");
        assert_eq!(empty.database_name().as_deref(), Some(""));
    }
}
