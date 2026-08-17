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

mod alias;
mod bulk_import;
mod cdc;
mod client;
mod collection;
mod common;
mod database;
mod dml;
mod dql;
mod external;
mod index;
mod iterator;
mod partition;
mod rbac;
mod resource_group;
mod snapshot;
mod utility;

use milvus::v2::request::cdc::{DumpMessagesRequest, ReplicateMessageId, WalName};
use milvus::v2::request::collection::{
    CreateCollectionRequest, CreateSimpleCollectionRequest, LoadCollectionRequest,
    RefreshLoadRequest,
};
use milvus::v2::request::database::ListDatabasesRequest;
use milvus::v2::request::dml::InsertRequest;
use milvus::v2::request::dql::{
    GetRequest, Ids, QueryIteratorRequest, QueryRequest, SearchIteratorRequest, SearchRequest,
    SearchVectors,
};
use milvus::v2::request::index::CreateIndexRequest;
use milvus::v2::request::rbac::UpdateUserRequest;
use milvus::v2::request::utility::OptimizeRequest;
use milvus::v2::ClientV2;
use milvus::v2::{
    CollectionSchema, ConsistencyLevel, DataType, FieldData, FieldSchema, IndexParam, IndexType,
    MetricType,
};
use milvus::v2::{CompactionStateCode, IndexStateCode, LoadState, SegmentLevel, SegmentState};
use milvus::v2::{ConnectConfig, RetryConfig};

#[test]
fn create_collection_accepts_full_and_simple_requests() {
    fn compile_calls(
        client: &ClientV2,
        full: CreateCollectionRequest,
        simple: CreateSimpleCollectionRequest,
    ) {
        drop(client.create_collection(full));
        drop(client.create_collection(simple));
    }

    let _ = compile_calls;
}

#[test]
fn v2_prelude_exposes_all_request_and_response_areas() {
    use milvus::v2::prelude::*;

    let _: Option<ClientV2> = None;
    let _: Option<BulkImport> = None;
    let _: Option<BulkImportConfig> = None;
    let _: Option<BulkImportRequest> = None;
    let _: Option<GetImportProgressRequest> = None;
    let _: Option<ListImportJobsRequest> = None;
    let _: Option<BulkImportResponse> = None;
    let _: Option<ConnectConfig> = None;
    let _: Option<FieldData> = None;
    let _: Option<SearchVectors> = None;

    let _: Option<CreateAliasRequest> = None;
    let _: Option<DumpMessagesRequest> = None;
    let _: Option<AddCollectionFunctionRequest> = None;
    let _: Option<AlterDatabasePropertiesRequest> = None;
    let _: Option<InsertRequest> = None;
    let _: Option<HybridSearchRequest> = None;
    let _: Option<AlterIndexPropertiesRequest> = None;
    let _: Option<GetPartitionStatsRequest> = None;
    let _: Option<CreateUserRequest> = None;
    let _: Option<TransferNodeRequest> = None;
    let _: Option<CompactRequest> = None;

    let _: Option<DescribeAliasResponse> = None;
    let _: Option<GetReplicateInfoResponse> = None;
    let _: Option<DescribeCollectionResponse> = None;
    let _: Option<DescribeDatabaseResponse> = None;
    let _: Option<DmlResponse> = None;
    let _: Option<SearchResponse> = None;
    let _: Option<DescribeIndexResponse> = None;
    let _: Option<ListPartitionsResponse> = None;
    let _: Option<ListUsersResponse> = None;
    let _: Option<DescribeResourceGroupResponse> = None;
    let _: Option<GetServerVersionResponse> = None;

    CreateSimpleCollectionRequest::builder()
        .collection_name("books")
        .dimension(4)
        .build()
        .expect("prelude collection request");
    InsertRequest::builder()
        .collection_name("books")
        .columns(vec![FieldData::int64("id", vec![1])])
        .build()
        .expect("prelude insert request");
    SearchRequest::builder()
        .collection_name("books")
        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2, 0.3, 0.4]]))
        .build()
        .expect("prelude search request");
    DropCollectionRequest::builder()
        .collection_name("books")
        .build()
        .expect("prelude drop request");
}

#[test]
fn built_requests_can_be_modified_through_into_builder() {
    let request = QueryRequest::builder()
        .database_name("analytics")
        .collection_name("books")
        .filter("id > 0")
        .output_fields(["title"])
        .limit(10)
        .build()
        .expect("valid query request");

    let request = request
        .into_builder()
        .filter("id > 100")
        .limit(20)
        .build()
        .expect("valid modified query request");

    assert_eq!(request.database_name().as_deref(), Some("analytics"));
    assert_eq!(request.collection_name(), "books");
    assert_eq!(request.filter(), "id > 100");
    assert_eq!(request.output_fields(), ["title"]);
    assert_eq!(request.limit(), Some(20));

    ListDatabasesRequest::builder()
        .build()
        .expect("valid unit request")
        .into_builder()
        .build()
        .expect("valid rebuilt unit request");
}

#[test]
fn public_v2_api_is_composable() {
    let vector_index = IndexParam::new()
        .field_name("embedding")
        .index_type(IndexType::Hnsw)
        .metric_type(MetricType::Cosine)
        .index_name("embedding_idx");
    let scalar_index = IndexParam::new()
        .field_name("title")
        .index_type(IndexType::Inverted)
        .metric_type(MetricType::Default);
    let schema = CollectionSchema::new()
        .description("book embeddings")
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("title")
                .data_type(DataType::VarChar)
                .max_length(512),
        )
        .add_field(
            FieldSchema::new()
                .name("embedding")
                .data_type(DataType::FloatVector)
                .dimension(4),
        );

    let create_collection = CreateCollectionRequest::builder()
        .collection_name("books")
        .schema(schema)
        .num_shards(2)
        .consistency_level(ConsistencyLevel::Session)
        .index_params(vec![vector_index.clone(), scalar_index.clone()])
        .build()
        .expect("valid request");
    let create_index = CreateIndexRequest::builder()
        .collection_name("books")
        .index_params(vec![vector_index, scalar_index])
        .build()
        .expect("valid request");
    let insert = InsertRequest::builder()
        .collection_name("books")
        .columns(vec![
            FieldData::Int64 {
                name: "id".into(),
                values: vec![1, 2],
            },
            FieldData::FloatVector {
                name: "embedding".into(),
                values: vec![vec![0.1, 0.2, 0.3, 0.4], vec![0.4, 0.3, 0.2, 0.1]],
            },
        ])
        .build()
        .expect("valid public insert request");
    let query = QueryRequest::builder()
        .collection_name("books")
        .filter("id > 0")
        .limit(10)
        .build()
        .expect("valid request");
    let search = SearchRequest::builder()
        .collection_name("books")
        .vector_field("embedding")
        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2, 0.3, 0.4]]))
        .limit(5)
        .build()
        .expect("valid request");
    let get = GetRequest::builder()
        .collection_name("books")
        .ids(Ids::Int64(vec![1, 2]))
        .output_fields(["title"])
        .build()
        .expect("valid request");
    let query_iterator = QueryIteratorRequest::builder()
        .query(query.clone())
        .batch_size(128)
        .build()
        .expect("valid request");
    let search_iterator = SearchIteratorRequest::builder()
        .search(search.clone())
        .batch_size(64)
        .limit(1_000)
        .build()
        .expect("valid request");
    let load = LoadCollectionRequest::builder()
        .collection_name("books")
        .sync(true)
        .timeout_ms(5_000)
        .build()
        .expect("valid request");
    let refresh = RefreshLoadRequest::builder()
        .collection_name("books")
        .sync(true)
        .timeout_ms(5_000)
        .build()
        .expect("valid request");
    let update_user = UpdateUserRequest::builder()
        .username("alice")
        .description("data engineer")
        .build()
        .expect("valid request");
    let optimize = OptimizeRequest::builder()
        .collection_name("books")
        .target_size("512MB")
        .async_mode(true)
        .timeout_ms(60_000)
        .build()
        .expect("valid request");
    let dump = DumpMessagesRequest::builder()
        .physical_channel("by-dev-rootcoord-dml_0")
        .start_message_id(
            ReplicateMessageId::new()
                .id("message-1")
                .wal_name(WalName::Pulsar),
        )
        .build()
        .expect("valid request");
    let retry = RetryConfig::new()
        .max_attempts(3)
        .retry_on_rate_limit(false);
    let connect = ConnectConfig::new()
        .uri("http://localhost:19530")
        .database("books")
        .retry(retry);

    fn accepts_public_output_types(
        _: Option<milvus::v2::response::index::IndexDesc>,
        _: Option<milvus::v2::OptimizeTask>,
    ) {
    }
    accepts_public_output_types(None, None);

    let states = (
        LoadState::NotExist,
        IndexStateCode::None,
        SegmentState::Unknown,
        SegmentLevel::Unknown,
        CompactionStateCode::Unknown,
    );
    drop((
        create_collection,
        create_index,
        insert,
        query,
        search,
        get,
        query_iterator,
        search_iterator,
        load,
        refresh,
        update_user,
        optimize,
        dump,
        connect,
        states,
    ));
}
