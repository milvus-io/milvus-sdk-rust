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

//! Response types returned by collection operations.

use crate::proto::milvus;
use crate::v2::error::status_to_result;
use crate::v2::error::{Error, Result};
use crate::v2::response::{validate_optional_parallel_array_len, validate_parallel_array_len};
use crate::v2::types::LoadState;
pub use crate::v2::types::{CollectionDesc, CollectionInfo, ReplicaInfo, ShardReplica};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// DescribeCollectionResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_collection operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DescribeCollectionResponse {
    pub(crate) description: CollectionDesc,
}

impl DescribeCollectionResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            description: CollectionDesc::new(),
        }
    }
}

impl DescribeCollectionResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeCollectionResponseBuilder {
        DescribeCollectionResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the description.
    pub fn description(&self) -> &CollectionDesc {
        &self.description
    }

    pub(crate) fn from_proto(value: milvus::DescribeCollectionResponse) -> Result<Self> {
        Ok(Self {
            description: CollectionDesc::from_proto(value)?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeCollectionResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeCollectionResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeCollectionResponseBuilder {
    value: DescribeCollectionResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeCollectionResponseBuilder {
    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: CollectionDesc) -> Self {
        self.value.description = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> DescribeCollectionResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// BatchDescribeCollectionsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 batch_describe_collections operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct BatchDescribeCollectionsResponse {
    pub(crate) descriptions: Vec<CollectionDesc>,
}

impl BatchDescribeCollectionsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            descriptions: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> BatchDescribeCollectionsResponseBuilder {
        BatchDescribeCollectionsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the descriptions.
    pub fn descriptions(&self) -> &[CollectionDesc] {
        &self.descriptions
    }

    pub(crate) fn from_proto(value: milvus::BatchDescribeCollectionResponse) -> Result<Self> {
        let descriptions = value
            .responses
            .into_iter()
            .map(|response| {
                status_to_result(&response.status)?;
                if let Some(status) = &response.status {
                    if status.code != 0 {
                        return Err(Error::MalformedResponse(format!(
                            "server error code {}: {}",
                            status.code, status.reason
                        )));
                    }
                }
                CollectionDesc::from_proto(response)
            })
            .collect::<Result<_>>()?;
        Ok(Self { descriptions })
    }
}

///////////////////////////////////////////////////////////////////////////////
// BatchDescribeCollectionsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for BatchDescribeCollectionsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct BatchDescribeCollectionsResponseBuilder {
    value: BatchDescribeCollectionsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl BatchDescribeCollectionsResponseBuilder {
    /// Sets the descriptions and returns the updated value.
    pub fn descriptions(mut self, value: Vec<CollectionDesc>) -> Self {
        self.value.descriptions = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> BatchDescribeCollectionsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeReplicasResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_replicas operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeReplicasResponse {
    pub(crate) replicas: Vec<ReplicaInfo>,
}

impl DescribeReplicasResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            replicas: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeReplicasResponseBuilder {
        DescribeReplicasResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the replicas.
    pub fn replicas(&self) -> &[ReplicaInfo] {
        &self.replicas
    }

    pub(crate) fn from_proto(value: milvus::GetReplicasResponse) -> Self {
        Self {
            replicas: value
                .replicas
                .into_iter()
                .map(|v| ReplicaInfo {
                    replica_id: v.replica_id,
                    collection_id: v.collection_id,
                    partition_ids: v.partition_ids,
                    shards: v
                        .shard_replicas
                        .into_iter()
                        .map(|s| ShardReplica {
                            leader_id: s.leader_id,
                            leader_address: s.leader_addr,
                            channel_name: s.dm_channel_name,
                            node_ids: s.node_ids,
                        })
                        .collect(),
                    node_ids: v.node_ids,
                    resource_group: v.resource_group_name,
                    outbound_nodes: v.num_outbound_node,
                })
                .collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeReplicasResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeReplicasResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeReplicasResponseBuilder {
    value: DescribeReplicasResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeReplicasResponseBuilder {
    /// Sets the replicas and returns the updated value.
    pub fn replicas(mut self, value: Vec<ReplicaInfo>) -> Self {
        self.value.replicas = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> DescribeReplicasResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCollectionStatsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_collection_stats operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetCollectionStatsResponse {
    pub(crate) statistics: HashMap<String, String>,
}

impl GetCollectionStatsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            statistics: HashMap::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetCollectionStatsResponseBuilder {
        GetCollectionStatsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the statistics.
    pub fn statistics(&self) -> &HashMap<String, String> {
        &self.statistics
    }

    pub(crate) fn from_proto(value: milvus::GetCollectionStatisticsResponse) -> Self {
        Self {
            statistics: value.stats.into_iter().map(|v| (v.key, v.value)).collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetCollectionStatsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetCollectionStatsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetCollectionStatsResponseBuilder {
    value: GetCollectionStatsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetCollectionStatsResponseBuilder {
    /// Sets the statistics and returns the updated value.
    pub fn statistics(mut self, value: HashMap<String, String>) -> Self {
        self.value.statistics = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> GetCollectionStatsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetLoadStateResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_load_state operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetLoadStateResponse {
    pub(crate) state: LoadState,
    pub(crate) progress: i64,
}

impl GetLoadStateResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            state: LoadState::default(),
            progress: 0,
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetLoadStateResponseBuilder {
        GetLoadStateResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the state.
    pub fn state(&self) -> LoadState {
        self.state
    }

    /// Returns the progress.
    pub fn progress(&self) -> i64 {
        self.progress
    }

    pub(crate) fn from_proto(value: milvus::GetLoadStateResponse, progress: i64) -> Self {
        Self {
            state: LoadState::from_proto(value.state),
            progress,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetLoadStateResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetLoadStateResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetLoadStateResponseBuilder {
    value: GetLoadStateResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetLoadStateResponseBuilder {
    /// Sets the state and returns the updated value.
    pub fn state(mut self, value: LoadState) -> Self {
        self.value.state = value;
        self
    }

    /// Sets the progress and returns the updated value.
    pub fn progress(mut self, value: i64) -> Self {
        self.value.progress = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> GetLoadStateResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListCollectionsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_collections operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListCollectionsResponse {
    pub(crate) collection_names: Vec<String>,
    pub(crate) collections: Vec<CollectionInfo>,
}

impl ListCollectionsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            collection_names: Vec::new(),
            collections: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListCollectionsResponseBuilder {
        ListCollectionsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the collection names.
    pub fn collection_names(&self) -> &[String] {
        &self.collection_names
    }

    /// Returns the collections.
    pub fn collections(&self) -> &[CollectionInfo] {
        &self.collections
    }

    pub(crate) fn from_proto(value: milvus::ShowCollectionsResponse) -> Result<Self> {
        let count = value.collection_names.len();
        validate_parallel_array_len(
            "ShowCollectionsResponse",
            "collection_names",
            count,
            "collection_ids",
            value.collection_ids.len(),
        )?;
        validate_parallel_array_len(
            "ShowCollectionsResponse",
            "collection_names",
            count,
            "created_timestamps",
            value.created_timestamps.len(),
        )?;
        validate_parallel_array_len(
            "ShowCollectionsResponse",
            "collection_names",
            count,
            "created_utc_timestamps",
            value.created_utc_timestamps.len(),
        )?;
        validate_optional_parallel_array_len(
            "ShowCollectionsResponse",
            "collection_names",
            count,
            "query_service_available",
            value.query_service_available.len(),
        )?;
        validate_optional_parallel_array_len(
            "ShowCollectionsResponse",
            "collection_names",
            count,
            "shards_num",
            value.shards_num.len(),
        )?;

        let names = value.collection_names;
        let collections = names
            .iter()
            .enumerate()
            .map(|(i, name)| CollectionInfo {
                name: name.clone(),
                id: value.collection_ids[i],
                created_timestamp: value.created_timestamps[i],
                created_utc_timestamp: value.created_utc_timestamps[i],
                query_service_available: value.query_service_available.get(i).copied(),
                shard_count: value.shards_num.get(i).copied(),
            })
            .collect();
        Ok(Self {
            collection_names: names,
            collections,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListCollectionsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListCollectionsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListCollectionsResponseBuilder {
    value: ListCollectionsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListCollectionsResponseBuilder {
    /// Sets the collection names and returns the updated value.
    pub fn collection_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.collection_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the collections and returns the updated value.
    pub fn collections(mut self, value: Vec<CollectionInfo>) -> Self {
        self.value.collections = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> ListCollectionsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasCollectionResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 has_collection operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HasCollectionResponse(pub(crate) bool);

impl HasCollectionResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self(false)
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> HasCollectionResponseBuilder {
        HasCollectionResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the exists.
    pub fn exists(&self) -> bool {
        self.0
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasCollectionResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for HasCollectionResponse.
#[derive(Debug, Clone, Copy)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct HasCollectionResponseBuilder {
    value: HasCollectionResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl HasCollectionResponseBuilder {
    /// Sets the has and returns the updated value.
    pub fn has(mut self, value: bool) -> Self {
        self.value.0 = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> HasCollectionResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{
        BatchDescribeCollectionsResponse, CollectionDesc, DescribeCollectionResponse,
        GetLoadStateResponse, ListCollectionsResponse,
    };
    use crate::proto::{common, milvus, schema};
    use crate::v2::error::Error;
    use crate::v2::types::{ConsistencyLevel, DataType, LoadState};

    #[test]
    fn describe_collection_preserves_geometry_timestamptz_and_struct_schema() {
        let description = CollectionDesc::from_proto(milvus::DescribeCollectionResponse {
            db_name: "bookstore".into(),
            collection_name: "books".into(),
            shards_num: 3,
            num_partitions: 2,
            collection_id: 42,
            aliases: vec!["current_books".into()],
            created_timestamp: 100,
            created_utc_timestamp: 150,
            update_timestamp: 200,
            consistency_level: common::ConsistencyLevel::Strong as i32,
            properties: vec![common::KeyValuePair {
                key: "collection.ttl.seconds".into(),
                value: "60".into(),
            }],
            schema: Some(schema::CollectionSchema {
                description: "book catalog".into(),
                enable_dynamic_field: true,
                fields: vec![
                    schema::FieldSchema {
                        name: "id".into(),
                        data_type: schema::DataType::Int64 as i32,
                        is_primary_key: true,
                        auto_id: true,
                        ..Default::default()
                    },
                    schema::FieldSchema {
                        name: "location".into(),
                        data_type: schema::DataType::Geometry as i32,
                        ..Default::default()
                    },
                    schema::FieldSchema {
                        name: "observed_at".into(),
                        data_type: schema::DataType::Timestamptz as i32,
                        ..Default::default()
                    },
                ],
                struct_array_fields: vec![schema::StructArrayFieldSchema {
                    name: "events".into(),
                    fields: vec![
                        schema::FieldSchema {
                            name: "location".into(),
                            data_type: schema::DataType::Array as i32,
                            element_type: schema::DataType::Geometry as i32,
                            type_params: vec![common::KeyValuePair {
                                key: "max_capacity".into(),
                                value: "16".into(),
                            }],
                            ..Default::default()
                        },
                        schema::FieldSchema {
                            name: "embedding".into(),
                            data_type: schema::DataType::ArrayOfVector as i32,
                            element_type: schema::DataType::FloatVector as i32,
                            type_params: vec![
                                common::KeyValuePair {
                                    key: "max_capacity".into(),
                                    value: "16".into(),
                                },
                                common::KeyValuePair {
                                    key: "dim".into(),
                                    value: "4".into(),
                                },
                            ],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("valid collection description");

        assert_eq!(description.get_database_name().to_owned(), "bookstore");
        assert_eq!(description.get_collection_name().to_owned(), "books");
        assert_eq!(description.get_description().to_owned(), "book catalog");
        assert_eq!(description.get_num_partitions().to_owned(), 2);
        assert_eq!(
            description.get_field_names(),
            &["id", "location", "observed_at", "events"]
        );
        assert_eq!(
            description.get_vector_field_names().to_owned(),
            ["events[embedding]"]
        );
        assert_eq!(description.get_primary_field_name().to_owned(), "id");
        assert!(description.is_dynamic_field_enabled());
        assert!(description.get_auto_id());
        assert_eq!(description.get_num_shards().to_owned(), 3);
        assert_eq!(description.get_collection_id().to_owned(), 42);
        assert_eq!(description.get_aliases().to_owned(), ["current_books"]);
        assert_eq!(description.get_created_time().to_owned(), 100);
        assert_eq!(description.get_created_utc_time().to_owned(), 150);
        assert_eq!(description.get_update_time().to_owned(), 200);
        assert_eq!(
            description.get_consistency_level(),
            ConsistencyLevel::Strong
        );
        assert_eq!(
            description.get_properties().get("collection.ttl.seconds"),
            Some(&"60".to_owned())
        );
        let schema = description.get_schema();
        assert_eq!(
            schema.get_fields()[1].get_data_type().to_owned(),
            DataType::Geometry
        );
        assert_eq!(
            schema.get_fields()[2].get_data_type(),
            DataType::Timestamptz
        );
        assert_eq!(schema.get_struct_fields().len().to_owned(), 1);
        assert_eq!(
            schema.get_struct_fields()[0].get_fields()[0].get_data_type(),
            DataType::Geometry
        );
        assert_eq!(
            schema.get_struct_fields()[0].get_fields()[1].get_data_type(),
            DataType::FloatVector
        );
    }

    #[test]
    fn describe_collection_rejects_a_missing_schema() {
        let error =
            DescribeCollectionResponse::from_proto(milvus::DescribeCollectionResponse::default())
                .unwrap_err();

        assert!(matches!(
            error,
            Error::MalformedResponse(message) if message.contains("returned no schema")
        ));
    }

    #[test]
    #[allow(deprecated)]
    fn batch_describe_collection_rejects_failed_nested_response() {
        let response = milvus::BatchDescribeCollectionResponse {
            responses: vec![milvus::DescribeCollectionResponse {
                status: Some(common::Status {
                    error_code: common::ErrorCode::CollectionNotExists as i32,
                    reason: "collection does not exist".into(),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        assert!(BatchDescribeCollectionsResponse::from_proto(response).is_err());
    }

    #[test]
    fn batch_describe_collection_rejects_modern_nested_error_code() {
        let response = milvus::BatchDescribeCollectionResponse {
            responses: vec![milvus::DescribeCollectionResponse {
                status: Some(common::Status {
                    code: 1100,
                    reason: "invalid collection".into(),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        assert!(BatchDescribeCollectionsResponse::from_proto(response).is_err());
    }

    #[test]
    fn get_load_state_preserves_calculated_progress() {
        let response = GetLoadStateResponse::from_proto(
            milvus::GetLoadStateResponse {
                state: common::LoadState::Loading as i32,
                ..Default::default()
            },
            37,
        );

        assert_eq!(response.state().to_owned(), LoadState::Loading);
        assert_eq!(response.progress().to_owned(), 37);
    }

    #[test]
    fn list_collections_rejects_short_required_parallel_array() {
        let error = ListCollectionsResponse::from_proto(milvus::ShowCollectionsResponse {
            collection_names: vec!["books".into(), "articles".into()],
            collection_ids: vec![1],
            created_timestamps: vec![10, 20],
            created_utc_timestamps: vec![100, 200],
            ..Default::default()
        })
        .expect_err("short collection IDs must be rejected");

        assert!(matches!(
            error,
            Error::MalformedResponse(message) if message.contains("collection_ids has 1")
        ));
    }

    #[test]
    fn list_collections_rejects_short_optional_parallel_array() {
        let error = ListCollectionsResponse::from_proto(milvus::ShowCollectionsResponse {
            collection_names: vec!["books".into(), "articles".into()],
            collection_ids: vec![1, 2],
            created_timestamps: vec![10, 20],
            created_utc_timestamps: vec![100, 200],
            query_service_available: vec![true],
            ..Default::default()
        })
        .expect_err("partial optional metadata must be rejected");

        assert!(matches!(
            error,
            Error::MalformedResponse(message) if message.contains("query_service_available has 1")
        ));
    }

    #[test]
    fn list_collections_preserves_absent_optional_metadata() {
        let response = ListCollectionsResponse::from_proto(milvus::ShowCollectionsResponse {
            collection_names: vec!["books".into()],
            collection_ids: vec![1],
            created_timestamps: vec![10],
            created_utc_timestamps: vec![100],
            ..Default::default()
        })
        .expect("omitted optional metadata is valid");

        assert_eq!(
            response.collections()[0].get_query_service_available(),
            None
        );
        assert_eq!(response.collections()[0].get_shard_count(), None);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn has_collection_response_default_values() {
        let value = HasCollectionResponse::builder().build();
        assert!(!value.exists());
    }

    #[test]
    fn has_collection_response_populated_values() {
        let value = HasCollectionResponse::builder().has(true).build();
        assert!(value.exists());
    }

    #[test]
    fn describe_collection_response_default_values() {
        let value = DescribeCollectionResponse::builder().build();
        let expected_description = CollectionDesc::new();

        assert_eq!(value.description().to_owned(), expected_description);
    }

    #[test]
    fn describe_collection_response_populated_values() {
        let description = CollectionDesc::new();
        let value = DescribeCollectionResponse::builder()
            .description(description.clone())
            .build();

        assert_eq!(value.description().to_owned(), description);
    }

    #[test]
    fn batch_describe_collections_response_default_values() {
        let value = BatchDescribeCollectionsResponse::builder().build();
        let expected_descriptions: Vec<CollectionDesc> = Default::default();

        assert_eq!(value.descriptions().to_owned(), expected_descriptions);
    }

    #[test]
    fn batch_describe_collections_response_populated_values() {
        let descriptions = vec![CollectionDesc::new()];
        let value = BatchDescribeCollectionsResponse::builder()
            .descriptions(descriptions.clone())
            .build();

        assert_eq!(value.descriptions().to_owned(), descriptions);
    }

    #[test]
    fn describe_replicas_response_default_values() {
        let value = DescribeReplicasResponse::builder().build();
        let expected_replicas: Vec<ReplicaInfo> = Default::default();

        assert_eq!(value.replicas().to_owned(), expected_replicas);
    }

    #[test]
    fn describe_replicas_response_populated_values() {
        let replicas = vec![ReplicaInfo::new()];
        let value = DescribeReplicasResponse::builder()
            .replicas(replicas.clone())
            .build();

        assert_eq!(value.replicas().to_owned(), replicas);
    }

    #[test]
    fn get_collection_stats_response_default_values() {
        let value = GetCollectionStatsResponse::builder().build();
        let expected_statistics: HashMap<String, String> = Default::default();

        assert_eq!(value.statistics().to_owned(), expected_statistics);
    }

    #[test]
    fn get_collection_stats_response_populated_values() {
        let statistics = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = GetCollectionStatsResponse::builder()
            .statistics(statistics.clone())
            .build();

        assert_eq!(value.statistics().to_owned(), statistics);
    }

    #[test]
    fn get_load_state_response_default_values() {
        let value = GetLoadStateResponse::builder().build();
        let expected_state: LoadState = Default::default();
        let expected_progress: i64 = 0;

        assert_eq!(value.state().to_owned(), expected_state);
        assert_eq!(value.progress().to_owned(), expected_progress);
    }

    #[test]
    fn get_load_state_response_populated_values() {
        let state = LoadState::Loaded;
        let progress = 7;
        let value = GetLoadStateResponse::builder()
            .state(state.clone())
            .progress(progress.clone())
            .build();

        assert_eq!(value.state().to_owned(), state);
        assert_eq!(value.progress().to_owned(), progress);
    }

    #[test]
    fn list_collections_response_default_values() {
        let value = ListCollectionsResponse::builder().build();
        let expected_collection_names: Vec<String> = Default::default();
        let expected_collections: Vec<CollectionInfo> = Default::default();

        assert_eq!(
            value.collection_names().to_owned(),
            expected_collection_names
        );
        assert_eq!(value.collections().to_owned(), expected_collections);
    }

    #[test]
    fn list_collections_response_populated_values() {
        let collection_names = vec!["collection_names-value".to_owned()];
        let collections = vec![CollectionInfo::new()];
        let value = ListCollectionsResponse::builder()
            .collection_names(collection_names.clone())
            .collections(collections.clone())
            .build();

        assert_eq!(value.collection_names().to_owned(), collection_names);
        assert_eq!(value.collections().to_owned(), collections);
    }
}
