#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateAliasRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub alias: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropAliasRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub alias: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AlterAliasRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub alias: ::prost::alloc::string::String,
}
/// *
/// Create collection in milvus
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The unique collection name in milvus.(Required)
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The serialized `schema.CollectionSchema`(Required)
    #[prost(bytes = "vec", tag = "4")]
    pub schema: ::prost::alloc::vec::Vec<u8>,
    /// Once set, no modification is allowed (Optional)
    /// <https://github.com/milvus-io/milvus/issues/6690>
    #[prost(int32, tag = "5")]
    pub shards_num: i32,
    /// The consistency level that the collection used, modification is not supported now.
    #[prost(enumeration = "super::common::ConsistencyLevel", tag = "6")]
    pub consistency_level: i32,
    #[prost(message, repeated, tag = "7")]
    pub properties: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
/// *
/// Drop collection in milvus, also will drop data in collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The unique collection name in milvus.(Required)
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
/// *
/// Alter collection in milvus
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AlterCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The unique collection name in milvus.(Required)
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "4")]
    pub collection_id: i64,
    #[prost(message, repeated, tag = "5")]
    pub properties: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
/// *
/// Check collection exist in milvus or not.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HasCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want to check.
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// If time_stamp is not zero, will return true when time_stamp >= created collection timestamp, otherwise will return false.
    #[prost(uint64, tag = "4")]
    pub time_stamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BoolResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(bool, tag = "2")]
    pub value: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
/// *
/// Get collection meta datas like: schema, collectionID, shards number ...
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want to describe, you can pass collection_name or collectionID
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The collection ID you want to describe
    #[prost(int64, tag = "4")]
    pub collection_id: i64,
    /// If time_stamp is not zero, will describe collection success when time_stamp >= created collection timestamp, otherwise will throw error.
    #[prost(uint64, tag = "5")]
    pub time_stamp: u64,
}
/// *
/// DescribeCollection Response
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeCollectionResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// The schema param when you created collection.
    #[prost(message, optional, tag = "2")]
    pub schema: ::core::option::Option<super::schema::CollectionSchema>,
    /// The collection id
    #[prost(int64, tag = "3")]
    pub collection_id: i64,
    /// System design related, users should not perceive
    #[prost(string, repeated, tag = "4")]
    pub virtual_channel_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// System design related, users should not perceive
    #[prost(string, repeated, tag = "5")]
    pub physical_channel_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Hybrid timestamp in milvus
    #[prost(uint64, tag = "6")]
    pub created_timestamp: u64,
    /// The utc timestamp calculated by created_timestamp
    #[prost(uint64, tag = "7")]
    pub created_utc_timestamp: u64,
    /// The shards number you set.
    #[prost(int32, tag = "8")]
    pub shards_num: i32,
    /// The aliases of this collection
    #[prost(string, repeated, tag = "9")]
    pub aliases: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The message ID/posititon when collection is created
    #[prost(message, repeated, tag = "10")]
    pub start_positions: ::prost::alloc::vec::Vec<super::common::KeyDataPair>,
    /// The consistency level that the collection used, modification is not supported now.
    #[prost(enumeration = "super::common::ConsistencyLevel", tag = "11")]
    pub consistency_level: i32,
    /// The collection name
    #[prost(string, tag = "12")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "13")]
    pub properties: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
/// *
/// Load collection data into query nodes, then you can do vector search on this collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoadCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want to load
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The replica number to load, default by 1
    #[prost(int32, tag = "4")]
    pub replica_number: i32,
}
/// *
/// Release collection data from query nodes, then you can't do vector search on this collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReleaseCollectionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want to release
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
/// *
/// Get statistics like row_count.
/// WARNING: This API is experimental and not useful for now.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatisticsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want get statistics
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition names you want get statistics, empty for all partitions
    #[prost(string, repeated, tag = "4")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Not useful for now, reserved for future
    #[prost(uint64, tag = "5")]
    pub guarantee_timestamp: u64,
}
/// *
/// Will return statistics in stats field like \[{key:"row_count",value:"1"}\]
/// WARNING: This API is experimental and not useful for now.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatisticsResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// Collection statistics data
    #[prost(message, repeated, tag = "2")]
    pub stats: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
/// *
/// Get collection statistics like row_count.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCollectionStatisticsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want get statistics
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
/// *
/// Will return collection statistics in stats field like \[{key:"row_count",value:"1"}\]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCollectionStatisticsResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// Collection statistics data
    #[prost(message, repeated, tag = "2")]
    pub stats: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
///
/// List collections
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowCollectionsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// Not useful for now
    #[prost(uint64, tag = "3")]
    pub time_stamp: u64,
    /// Decide return Loaded collections or All collections(Optional)
    #[prost(enumeration = "ShowType", tag = "4")]
    pub r#type: i32,
    /// When type is InMemory, will return these collection's inMemory_percentages.(Optional)
    /// Deprecated: use GetLoadingProgress rpc instead
    #[prost(string, repeated, tag = "5")]
    pub collection_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
///
/// Return basic collection infos.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowCollectionsResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// Collection name array
    #[prost(string, repeated, tag = "2")]
    pub collection_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Collection Id array
    #[prost(int64, repeated, tag = "3")]
    pub collection_ids: ::prost::alloc::vec::Vec<i64>,
    /// Hybrid timestamps in milvus
    #[prost(uint64, repeated, tag = "4")]
    pub created_timestamps: ::prost::alloc::vec::Vec<u64>,
    /// The utc timestamp calculated by created_timestamp
    #[prost(uint64, repeated, tag = "5")]
    pub created_utc_timestamps: ::prost::alloc::vec::Vec<u64>,
    /// Load percentage on querynode when type is InMemory
    /// Deprecated: use GetLoadingProgress rpc instead
    #[prost(int64, repeated, tag = "6")]
    pub in_memory_percentages: ::prost::alloc::vec::Vec<i64>,
    /// Indicate whether query service is available
    #[prost(bool, repeated, tag = "7")]
    pub query_service_available: ::prost::alloc::vec::Vec<bool>,
}
///
/// Create partition in created collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePartitionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition name you want to create.
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
}
///
/// Drop partition in created collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropPartitionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition name you want to drop
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
}
///
/// Check if partition exist in collection or not.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HasPartitionRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition name you want to check
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
}
///
/// Load specific partitions data of one collection into query nodes
/// Then you can get these data as result when you do vector search on this collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoadPartitionsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition names you want to load
    #[prost(string, repeated, tag = "4")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The replicas number you would load, 1 by default
    #[prost(int32, tag = "5")]
    pub replica_number: i32,
}
///
/// Release specific partitions data of one collection from query nodes.
/// Then you can not get these data as result when you do vector search on this collection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReleasePartitionsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition names you want to release
    #[prost(string, repeated, tag = "4")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
///
/// Get partition statistics like row_count.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPartitionStatisticsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The partition name you want to collect statistics
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPartitionStatisticsResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub stats: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
///
/// List all partitions for particular collection
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowPartitionsRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name you want to describe, you can pass collection_name or collectionID
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The collection id in milvus
    #[prost(int64, tag = "4")]
    pub collection_id: i64,
    /// When type is InMemory, will return these patitions's inMemory_percentages.(Optional)
    #[prost(string, repeated, tag = "5")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Decide return Loaded partitions or All partitions(Optional)
    /// Deprecated: use GetLoadingProgress rpc instead
    #[prost(enumeration = "ShowType", tag = "6")]
    pub r#type: i32,
}
///
/// List all partitions for particular collection response.
/// The returned datas are all rows, we can format to columns by therir index.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowPartitionsResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// All partition names for this collection
    #[prost(string, repeated, tag = "2")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// All partition ids for this collection
    #[prost(int64, repeated, tag = "3")]
    pub partition_i_ds: ::prost::alloc::vec::Vec<i64>,
    /// All hybrid timestamps
    #[prost(uint64, repeated, tag = "4")]
    pub created_timestamps: ::prost::alloc::vec::Vec<u64>,
    /// All utc timestamps calculated by created_timestamps
    #[prost(uint64, repeated, tag = "5")]
    pub created_utc_timestamps: ::prost::alloc::vec::Vec<u64>,
    /// Load percentage on querynode
    /// Deprecated: use GetLoadingProgress rpc instead
    #[prost(int64, repeated, tag = "6")]
    pub in_memory_percentages: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeSegmentRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(int64, tag = "3")]
    pub segment_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeSegmentResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, tag = "2")]
    pub index_id: i64,
    #[prost(int64, tag = "3")]
    pub build_id: i64,
    #[prost(bool, tag = "4")]
    pub enable_index: bool,
    #[prost(int64, tag = "5")]
    pub field_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowSegmentsRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(int64, tag = "3")]
    pub partition_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowSegmentsResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, repeated, tag = "2")]
    pub segment_i_ds: ::prost::alloc::vec::Vec<i64>,
}
///
/// Create index for vector datas
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateIndexRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The particular collection name you want to create index.
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The vector field name in this particular collection
    #[prost(string, tag = "4")]
    pub field_name: ::prost::alloc::string::String,
    /// Support keys: index_type,metric_type, params. Different index_type may has different params.
    #[prost(message, repeated, tag = "5")]
    pub extra_params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
    /// Version before 2.0.2 doesn't contain index_name, we use default index name.
    #[prost(string, tag = "6")]
    pub index_name: ::prost::alloc::string::String,
}
///
/// Get created index information.
/// Current release of Milvus only supports showing latest built index.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeIndexRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The particular collection name in Milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The vector field name in this particular collection
    #[prost(string, tag = "4")]
    pub field_name: ::prost::alloc::string::String,
    /// No need to set up for now @2021.06.30
    #[prost(string, tag = "5")]
    pub index_name: ::prost::alloc::string::String,
}
///
/// Index informations
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexDescription {
    /// Index name
    #[prost(string, tag = "1")]
    pub index_name: ::prost::alloc::string::String,
    /// Index id
    #[prost(int64, tag = "2")]
    pub index_id: i64,
    /// Will return index_type, metric_type, params(like nlist).
    #[prost(message, repeated, tag = "3")]
    pub params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
    /// The vector field name
    #[prost(string, tag = "4")]
    pub field_name: ::prost::alloc::string::String,
    /// index build progress
    #[prost(int64, tag = "5")]
    pub indexed_rows: i64,
    #[prost(int64, tag = "6")]
    pub total_rows: i64,
    /// index state
    #[prost(enumeration = "super::common::IndexState", tag = "7")]
    pub state: i32,
    #[prost(string, tag = "8")]
    pub index_state_fail_reason: ::prost::alloc::string::String,
}
///
/// Describe index response
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeIndexResponse {
    /// Response status
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// All index informations, for now only return tha latest index you created for the collection.
    #[prost(message, repeated, tag = "2")]
    pub index_descriptions: ::prost::alloc::vec::Vec<IndexDescription>,
}
///
///   Get index building progress
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexBuildProgressRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// The collection name in milvus
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// The vector field name in this collection
    #[prost(string, tag = "4")]
    pub field_name: ::prost::alloc::string::String,
    /// Not useful for now
    #[prost(string, tag = "5")]
    pub index_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexBuildProgressResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, tag = "2")]
    pub indexed_rows: i64,
    #[prost(int64, tag = "3")]
    pub total_rows: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexStateRequest {
    /// must
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// must
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub field_name: ::prost::alloc::string::String,
    /// No need to set up for now @2021.06.30
    #[prost(string, tag = "5")]
    pub index_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexStateResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(enumeration = "super::common::IndexState", tag = "2")]
    pub state: i32,
    #[prost(string, tag = "3")]
    pub fail_reason: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropIndexRequest {
    /// must
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// must
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub field_name: ::prost::alloc::string::String,
    /// No need to set up for now @2021.06.30
    #[prost(string, tag = "5")]
    pub index_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "5")]
    pub fields_data: ::prost::alloc::vec::Vec<super::schema::FieldData>,
    #[prost(uint32, repeated, tag = "6")]
    pub hash_keys: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, tag = "7")]
    pub num_rows: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutationResult {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// required for insert, delete
    #[prost(message, optional, tag = "2")]
    pub i_ds: ::core::option::Option<super::schema::IDs>,
    /// error indexes indicate
    #[prost(uint32, repeated, tag = "3")]
    pub succ_index: ::prost::alloc::vec::Vec<u32>,
    /// error indexes indicate
    #[prost(uint32, repeated, tag = "4")]
    pub err_index: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, tag = "5")]
    pub acknowledged: bool,
    #[prost(int64, tag = "6")]
    pub insert_cnt: i64,
    #[prost(int64, tag = "7")]
    pub delete_cnt: i64,
    #[prost(int64, tag = "8")]
    pub upsert_cnt: i64,
    #[prost(uint64, tag = "9")]
    pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub expr: ::prost::alloc::string::String,
    #[prost(uint32, repeated, tag = "6")]
    pub hash_keys: ::prost::alloc::vec::Vec<u32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchRequest {
    /// must
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// must
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    /// must
    #[prost(string, repeated, tag = "4")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// must
    #[prost(string, tag = "5")]
    pub dsl: ::prost::alloc::string::String,
    /// serialized `PlaceholderGroup`
    ///
    /// must
    #[prost(bytes = "vec", tag = "6")]
    pub placeholder_group: ::prost::alloc::vec::Vec<u8>,
    /// must
    #[prost(enumeration = "super::common::DslType", tag = "7")]
    pub dsl_type: i32,
    #[prost(string, repeated, tag = "8")]
    pub output_fields: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// must
    #[prost(message, repeated, tag = "9")]
    pub search_params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
    #[prost(uint64, tag = "10")]
    pub travel_timestamp: u64,
    /// guarantee_timestamp
    #[prost(uint64, tag = "11")]
    pub guarantee_timestamp: u64,
    #[prost(int64, tag = "12")]
    pub nq: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hits {
    #[prost(int64, repeated, tag = "1")]
    pub i_ds: ::prost::alloc::vec::Vec<i64>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub row_data: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(float, repeated, tag = "3")]
    pub scores: ::prost::alloc::vec::Vec<f32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchResults {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag = "2")]
    pub results: ::core::option::Option<super::schema::SearchResultData>,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlushRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub collection_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlushResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(map = "string, message", tag = "3")]
    pub coll_seg_i_ds: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        super::schema::LongArray,
    >,
    #[prost(map = "string, message", tag = "4")]
    pub flush_coll_seg_i_ds: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        super::schema::LongArray,
    >,
    #[prost(map = "string, int64", tag = "5")]
    pub coll_seal_times: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        i64,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub expr: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "5")]
    pub output_fields: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "6")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(uint64, tag = "7")]
    pub travel_timestamp: u64,
    /// guarantee_timestamp
    #[prost(uint64, tag = "8")]
    pub guarantee_timestamp: u64,
    /// optional
    #[prost(message, repeated, tag = "9")]
    pub query_params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryResults {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub fields_data: ::prost::alloc::vec::Vec<super::schema::FieldData>,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VectorIDs {
    #[prost(string, tag = "1")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub field_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub id_array: ::core::option::Option<super::schema::IDs>,
    #[prost(string, repeated, tag = "4")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VectorsArray {
    #[prost(oneof = "vectors_array::Array", tags = "1, 2")]
    pub array: ::core::option::Option<vectors_array::Array>,
}
/// Nested message and enum types in `VectorsArray`.
pub mod vectors_array {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Array {
        /// vector ids
        #[prost(message, tag = "1")]
        IdArray(super::VectorIDs),
        /// vectors data
        #[prost(message, tag = "2")]
        DataArray(super::super::schema::VectorField),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CalcDistanceRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// vectors on the left of operator
    #[prost(message, optional, tag = "2")]
    pub op_left: ::core::option::Option<VectorsArray>,
    /// vectors on the right of operator
    #[prost(message, optional, tag = "3")]
    pub op_right: ::core::option::Option<VectorsArray>,
    /// "metric":"L2"/"IP"/"HAMMIN"/"TANIMOTO"
    #[prost(message, repeated, tag = "4")]
    pub params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CalcDistanceResults {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// num(op_left)*num(op_right) distance values, "HAMMIN" return integer distance
    #[prost(oneof = "calc_distance_results::Array", tags = "2, 3")]
    pub array: ::core::option::Option<calc_distance_results::Array>,
}
/// Nested message and enum types in `CalcDistanceResults`.
pub mod calc_distance_results {
    /// num(op_left)*num(op_right) distance values, "HAMMIN" return integer distance
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Array {
        #[prost(message, tag = "2")]
        IntDist(super::super::schema::IntArray),
        #[prost(message, tag = "3")]
        FloatDist(super::super::schema::FloatArray),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PersistentSegmentInfo {
    #[prost(int64, tag = "1")]
    pub segment_id: i64,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(int64, tag = "3")]
    pub partition_id: i64,
    #[prost(int64, tag = "4")]
    pub num_rows: i64,
    #[prost(enumeration = "super::common::SegmentState", tag = "5")]
    pub state: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPersistentSegmentInfoRequest {
    /// must
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// must
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPersistentSegmentInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub infos: ::prost::alloc::vec::Vec<PersistentSegmentInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySegmentInfo {
    #[prost(int64, tag = "1")]
    pub segment_id: i64,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(int64, tag = "3")]
    pub partition_id: i64,
    #[prost(int64, tag = "4")]
    pub mem_size: i64,
    #[prost(int64, tag = "5")]
    pub num_rows: i64,
    #[prost(string, tag = "6")]
    pub index_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "7")]
    pub index_id: i64,
    /// deprecated, check node_ids(NodeIds) field
    #[prost(int64, tag = "8")]
    pub node_id: i64,
    #[prost(enumeration = "super::common::SegmentState", tag = "9")]
    pub state: i32,
    #[prost(int64, repeated, tag = "10")]
    pub node_ids: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetQuerySegmentInfoRequest {
    /// must
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    /// must
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetQuerySegmentInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub infos: ::prost::alloc::vec::Vec<QuerySegmentInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DummyRequest {
    #[prost(string, tag = "1")]
    pub request_type: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DummyResponse {
    #[prost(string, tag = "1")]
    pub response: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterLinkRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterLinkResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::common::Address>,
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMetricsRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// request is of jsonic format
    #[prost(string, tag = "2")]
    pub request: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMetricsResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// response is of jsonic format
    #[prost(string, tag = "2")]
    pub response: ::prost::alloc::string::String,
    /// metrics from which component
    #[prost(string, tag = "3")]
    pub component_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComponentInfo {
    #[prost(int64, tag = "1")]
    pub node_id: i64,
    #[prost(string, tag = "2")]
    pub role: ::prost::alloc::string::String,
    #[prost(enumeration = "super::common::StateCode", tag = "3")]
    pub state_code: i32,
    #[prost(message, repeated, tag = "4")]
    pub extra_info: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComponentStates {
    #[prost(message, optional, tag = "1")]
    pub state: ::core::option::Option<ComponentInfo>,
    #[prost(message, repeated, tag = "2")]
    pub subcomponent_states: ::prost::alloc::vec::Vec<ComponentInfo>,
    #[prost(message, optional, tag = "3")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetComponentStatesRequest {}
///
/// Do load balancing operation from src_nodeID to dst_nodeID.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoadBalanceRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub src_node_id: i64,
    #[prost(int64, repeated, tag = "3")]
    pub dst_node_i_ds: ::prost::alloc::vec::Vec<i64>,
    #[prost(int64, repeated, tag = "4")]
    pub sealed_segment_i_ds: ::prost::alloc::vec::Vec<i64>,
    #[prost(string, tag = "5")]
    pub collection_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ManualCompactionRequest {
    #[prost(int64, tag = "1")]
    pub collection_id: i64,
    #[prost(uint64, tag = "2")]
    pub timetravel: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ManualCompactionResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, tag = "2")]
    pub compaction_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionStateRequest {
    #[prost(int64, tag = "1")]
    pub compaction_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionStateResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(enumeration = "super::common::CompactionState", tag = "2")]
    pub state: i32,
    #[prost(int64, tag = "3")]
    pub executing_plan_no: i64,
    #[prost(int64, tag = "4")]
    pub timeout_plan_no: i64,
    #[prost(int64, tag = "5")]
    pub completed_plan_no: i64,
    #[prost(int64, tag = "6")]
    pub failed_plan_no: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionPlansRequest {
    #[prost(int64, tag = "1")]
    pub compaction_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionPlansResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(enumeration = "super::common::CompactionState", tag = "2")]
    pub state: i32,
    #[prost(message, repeated, tag = "3")]
    pub merge_infos: ::prost::alloc::vec::Vec<CompactionMergeInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactionMergeInfo {
    #[prost(int64, repeated, tag = "1")]
    pub sources: ::prost::alloc::vec::Vec<i64>,
    #[prost(int64, tag = "2")]
    pub target: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetFlushStateRequest {
    #[prost(int64, repeated, tag = "1")]
    pub segment_i_ds: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetFlushStateResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(bool, tag = "2")]
    pub flushed: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportRequest {
    /// target collection
    #[prost(string, tag = "1")]
    pub collection_name: ::prost::alloc::string::String,
    /// target partition
    #[prost(string, tag = "2")]
    pub partition_name: ::prost::alloc::string::String,
    /// channel names for the collection
    #[prost(string, repeated, tag = "3")]
    pub channel_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// the file is row-based or column-based
    #[prost(bool, tag = "4")]
    pub row_based: bool,
    /// file paths to be imported
    #[prost(string, repeated, tag = "5")]
    pub files: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// import options, bucket, etc.
    #[prost(message, repeated, tag = "6")]
    pub options: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// id array of import tasks
    #[prost(int64, repeated, tag = "2")]
    pub tasks: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetImportStateRequest {
    /// id of an import task
    #[prost(int64, tag = "1")]
    pub task: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetImportStateResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// is this import task finished or not
    #[prost(enumeration = "super::common::ImportState", tag = "2")]
    pub state: i32,
    /// if the task is finished, this value is how many rows are imported. if the task is not finished, this value is how many rows are parsed. return 0 if failed.
    #[prost(int64, tag = "3")]
    pub row_count: i64,
    /// auto generated ids if the primary key is autoid
    #[prost(int64, repeated, tag = "4")]
    pub id_list: ::prost::alloc::vec::Vec<i64>,
    /// more information about the task, progress percent, file path, failed reason, etc.
    #[prost(message, repeated, tag = "5")]
    pub infos: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
    /// id of an import task
    #[prost(int64, tag = "6")]
    pub id: i64,
    /// collection ID of the import task.
    #[prost(int64, tag = "7")]
    pub collection_id: i64,
    /// a list of segment IDs created by the import task.
    #[prost(int64, repeated, tag = "8")]
    pub segment_ids: ::prost::alloc::vec::Vec<i64>,
    /// timestamp when the import task is created.
    #[prost(int64, tag = "9")]
    pub create_ts: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListImportTasksRequest {
    /// target collection, list all tasks if the name is empty
    #[prost(string, tag = "1")]
    pub collection_name: ::prost::alloc::string::String,
    /// maximum number of tasks returned, list all tasks if the value is 0
    #[prost(int64, tag = "2")]
    pub limit: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListImportTasksResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// list of all import tasks
    #[prost(message, repeated, tag = "2")]
    pub tasks: ::prost::alloc::vec::Vec<GetImportStateResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetReplicasRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(bool, tag = "3")]
    pub with_shard_nodes: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetReplicasResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub replicas: ::prost::alloc::vec::Vec<ReplicaInfo>,
}
/// ReplicaGroup
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReplicaInfo {
    #[prost(int64, tag = "1")]
    pub replica_id: i64,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    /// empty indicates to load collection
    #[prost(int64, repeated, tag = "3")]
    pub partition_ids: ::prost::alloc::vec::Vec<i64>,
    #[prost(message, repeated, tag = "4")]
    pub shard_replicas: ::prost::alloc::vec::Vec<ShardReplica>,
    /// include leaders
    #[prost(int64, repeated, tag = "5")]
    pub node_ids: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardReplica {
    #[prost(int64, tag = "1")]
    pub leader_id: i64,
    /// IP:port
    #[prost(string, tag = "2")]
    pub leader_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub dm_channel_name: ::prost::alloc::string::String,
    /// optional, DO NOT save it in meta, set it only for GetReplicas()
    /// if with_shard_nodes is true
    #[prost(int64, repeated, tag = "4")]
    pub node_ids: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCredentialRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// username
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
    /// ciphertext password
    #[prost(string, tag = "3")]
    pub password: ::prost::alloc::string::String,
    /// create time
    #[prost(uint64, tag = "4")]
    pub created_utc_timestamps: u64,
    /// modify time
    #[prost(uint64, tag = "5")]
    pub modified_utc_timestamps: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCredentialRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// username
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
    /// old password
    #[prost(string, tag = "3")]
    pub old_password: ::prost::alloc::string::String,
    /// new password
    #[prost(string, tag = "4")]
    pub new_password: ::prost::alloc::string::String,
    /// create time
    #[prost(uint64, tag = "5")]
    pub created_utc_timestamps: u64,
    /// modify time
    #[prost(uint64, tag = "6")]
    pub modified_utc_timestamps: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCredentialRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListCredUsersResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// username array
    #[prost(string, repeated, tag = "2")]
    pub usernames: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListCredUsersRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
}
/// <https://wiki.lfaidata.foundation/display/MIL/MEP+29+--+Support+Role-Based+Access+Control>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoleEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateRoleRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// role
    #[prost(message, optional, tag = "2")]
    pub entity: ::core::option::Option<RoleEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropRoleRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// role name
    #[prost(string, tag = "2")]
    pub role_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperateUserRoleRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// username
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
    /// role name
    #[prost(string, tag = "3")]
    pub role_name: ::prost::alloc::string::String,
    /// operation type
    #[prost(enumeration = "OperateUserRoleType", tag = "4")]
    pub r#type: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectRoleRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// role
    #[prost(message, optional, tag = "2")]
    pub role: ::core::option::Option<RoleEntity>,
    /// include user info
    #[prost(bool, tag = "3")]
    pub include_user_info: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoleResult {
    #[prost(message, optional, tag = "1")]
    pub role: ::core::option::Option<RoleEntity>,
    #[prost(message, repeated, tag = "2")]
    pub users: ::prost::alloc::vec::Vec<UserEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectRoleResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// role result array
    #[prost(message, repeated, tag = "2")]
    pub results: ::prost::alloc::vec::Vec<RoleResult>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectUserRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// user
    #[prost(message, optional, tag = "2")]
    pub user: ::core::option::Option<UserEntity>,
    /// include user info
    #[prost(bool, tag = "3")]
    pub include_role_info: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserResult {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<UserEntity>,
    #[prost(message, repeated, tag = "2")]
    pub roles: ::prost::alloc::vec::Vec<RoleEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectUserResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// user result array
    #[prost(message, repeated, tag = "2")]
    pub results: ::prost::alloc::vec::Vec<UserResult>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivilegeEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantorEntity {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<UserEntity>,
    #[prost(message, optional, tag = "2")]
    pub privilege: ::core::option::Option<PrivilegeEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantPrivilegeEntity {
    #[prost(message, repeated, tag = "1")]
    pub entities: ::prost::alloc::vec::Vec<GrantorEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantEntity {
    /// role
    #[prost(message, optional, tag = "1")]
    pub role: ::core::option::Option<RoleEntity>,
    /// object
    #[prost(message, optional, tag = "2")]
    pub object: ::core::option::Option<ObjectEntity>,
    /// object name
    #[prost(string, tag = "3")]
    pub object_name: ::prost::alloc::string::String,
    /// privilege
    #[prost(message, optional, tag = "4")]
    pub grantor: ::core::option::Option<GrantorEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectGrantRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// grant
    #[prost(message, optional, tag = "2")]
    pub entity: ::core::option::Option<GrantEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectGrantResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// grant info array
    #[prost(message, repeated, tag = "2")]
    pub entities: ::prost::alloc::vec::Vec<GrantEntity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperatePrivilegeRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// grant
    #[prost(message, optional, tag = "2")]
    pub entity: ::core::option::Option<GrantEntity>,
    /// operation type
    #[prost(enumeration = "OperatePrivilegeType", tag = "3")]
    pub r#type: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLoadingProgressRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub partition_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLoadingProgressResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, tag = "2")]
    pub progress: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MilvusExt {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetVersionRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetVersionResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckHealthRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckHealthResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(bool, tag = "2")]
    pub is_healthy: bool,
    #[prost(string, repeated, tag = "3")]
    pub reasons: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Deprecated: use GetLoadingProgress rpc instead
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ShowType {
    /// Will return all collections
    All = 0,
    /// Will return loaded collections with their inMemory_percentages
    InMemory = 1,
}
impl ShowType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ShowType::All => "All",
            ShowType::InMemory => "InMemory",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "All" => Some(Self::All),
            "InMemory" => Some(Self::InMemory),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OperateUserRoleType {
    AddUserToRole = 0,
    RemoveUserFromRole = 1,
}
impl OperateUserRoleType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OperateUserRoleType::AddUserToRole => "AddUserToRole",
            OperateUserRoleType::RemoveUserFromRole => "RemoveUserFromRole",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AddUserToRole" => Some(Self::AddUserToRole),
            "RemoveUserFromRole" => Some(Self::RemoveUserFromRole),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OperatePrivilegeType {
    Grant = 0,
    Revoke = 1,
}
impl OperatePrivilegeType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OperatePrivilegeType::Grant => "Grant",
            OperatePrivilegeType::Revoke => "Revoke",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Grant" => Some(Self::Grant),
            "Revoke" => Some(Self::Revoke),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod milvus_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct MilvusServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MilvusServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MilvusServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MilvusServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MilvusServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn create_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CreateCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::DropCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DropCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn has_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::HasCollectionRequest>,
        ) -> Result<tonic::Response<super::BoolResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/HasCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn load_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::LoadCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/LoadCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn release_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::ReleaseCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ReleaseCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn describe_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::DescribeCollectionRequest>,
        ) -> Result<tonic::Response<super::DescribeCollectionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DescribeCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_collection_statistics(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCollectionStatisticsRequest>,
        ) -> Result<
            tonic::Response<super::GetCollectionStatisticsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetCollectionStatistics",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn show_collections(
            &mut self,
            request: impl tonic::IntoRequest<super::ShowCollectionsRequest>,
        ) -> Result<tonic::Response<super::ShowCollectionsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ShowCollections",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn alter_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::AlterCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/AlterCollection",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_partition(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePartitionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CreatePartition",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_partition(
            &mut self,
            request: impl tonic::IntoRequest<super::DropPartitionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DropPartition",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn has_partition(
            &mut self,
            request: impl tonic::IntoRequest<super::HasPartitionRequest>,
        ) -> Result<tonic::Response<super::BoolResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/HasPartition",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn load_partitions(
            &mut self,
            request: impl tonic::IntoRequest<super::LoadPartitionsRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/LoadPartitions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn release_partitions(
            &mut self,
            request: impl tonic::IntoRequest<super::ReleasePartitionsRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ReleasePartitions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_partition_statistics(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPartitionStatisticsRequest>,
        ) -> Result<
            tonic::Response<super::GetPartitionStatisticsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetPartitionStatistics",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn show_partitions(
            &mut self,
            request: impl tonic::IntoRequest<super::ShowPartitionsRequest>,
        ) -> Result<tonic::Response<super::ShowPartitionsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ShowPartitions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_loading_progress(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLoadingProgressRequest>,
        ) -> Result<tonic::Response<super::GetLoadingProgressResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetLoadingProgress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_alias(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CreateAlias",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_alias(
            &mut self,
            request: impl tonic::IntoRequest<super::DropAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DropAlias",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn alter_alias(
            &mut self,
            request: impl tonic::IntoRequest<super::AlterAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/AlterAlias",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_index(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateIndexRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CreateIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn describe_index(
            &mut self,
            request: impl tonic::IntoRequest<super::DescribeIndexRequest>,
        ) -> Result<tonic::Response<super::DescribeIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DescribeIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deprecated: use DescribeIndex instead
        pub async fn get_index_state(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIndexStateRequest>,
        ) -> Result<tonic::Response<super::GetIndexStateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetIndexState",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deprecated: use DescribeIndex instead
        pub async fn get_index_build_progress(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIndexBuildProgressRequest>,
        ) -> Result<
            tonic::Response<super::GetIndexBuildProgressResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetIndexBuildProgress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_index(
            &mut self,
            request: impl tonic::IntoRequest<super::DropIndexRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DropIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn insert(
            &mut self,
            request: impl tonic::IntoRequest<super::InsertRequest>,
        ) -> Result<tonic::Response<super::MutationResult>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Insert",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRequest>,
        ) -> Result<tonic::Response<super::MutationResult>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Delete",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn search(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchRequest>,
        ) -> Result<tonic::Response<super::SearchResults>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Search",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::FlushRequest>,
        ) -> Result<tonic::Response<super::FlushResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Flush",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRequest>,
        ) -> Result<tonic::Response<super::QueryResults>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Query",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn calc_distance(
            &mut self,
            request: impl tonic::IntoRequest<super::CalcDistanceRequest>,
        ) -> Result<tonic::Response<super::CalcDistanceResults>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CalcDistance",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_flush_state(
            &mut self,
            request: impl tonic::IntoRequest<super::GetFlushStateRequest>,
        ) -> Result<tonic::Response<super::GetFlushStateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetFlushState",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_persistent_segment_info(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPersistentSegmentInfoRequest>,
        ) -> Result<
            tonic::Response<super::GetPersistentSegmentInfoResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetPersistentSegmentInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_query_segment_info(
            &mut self,
            request: impl tonic::IntoRequest<super::GetQuerySegmentInfoRequest>,
        ) -> Result<tonic::Response<super::GetQuerySegmentInfoResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetQuerySegmentInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_replicas(
            &mut self,
            request: impl tonic::IntoRequest<super::GetReplicasRequest>,
        ) -> Result<tonic::Response<super::GetReplicasResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetReplicas",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn dummy(
            &mut self,
            request: impl tonic::IntoRequest<super::DummyRequest>,
        ) -> Result<tonic::Response<super::DummyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Dummy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// TODO: remove
        pub async fn register_link(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterLinkRequest>,
        ) -> Result<tonic::Response<super::RegisterLinkResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/RegisterLink",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// https://wiki.lfaidata.foundation/display/MIL/MEP+8+--+Add+metrics+for+proxy
        pub async fn get_metrics(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMetricsRequest>,
        ) -> Result<tonic::Response<super::GetMetricsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetMetrics",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_component_states(
            &mut self,
            request: impl tonic::IntoRequest<super::GetComponentStatesRequest>,
        ) -> Result<tonic::Response<super::ComponentStates>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetComponentStates",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn load_balance(
            &mut self,
            request: impl tonic::IntoRequest<super::LoadBalanceRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/LoadBalance",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_compaction_state(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCompactionStateRequest>,
        ) -> Result<tonic::Response<super::GetCompactionStateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetCompactionState",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn manual_compaction(
            &mut self,
            request: impl tonic::IntoRequest<super::ManualCompactionRequest>,
        ) -> Result<tonic::Response<super::ManualCompactionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ManualCompaction",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_compaction_state_with_plans(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCompactionPlansRequest>,
        ) -> Result<tonic::Response<super::GetCompactionPlansResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetCompactionStateWithPlans",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// https://wiki.lfaidata.foundation/display/MIL/MEP+24+--+Support+bulk+load
        pub async fn import(
            &mut self,
            request: impl tonic::IntoRequest<super::ImportRequest>,
        ) -> Result<tonic::Response<super::ImportResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/Import",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_import_state(
            &mut self,
            request: impl tonic::IntoRequest<super::GetImportStateRequest>,
        ) -> Result<tonic::Response<super::GetImportStateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetImportState",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_import_tasks(
            &mut self,
            request: impl tonic::IntoRequest<super::ListImportTasksRequest>,
        ) -> Result<tonic::Response<super::ListImportTasksResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ListImportTasks",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// https://wiki.lfaidata.foundation/display/MIL/MEP+27+--+Support+Basic+Authentication
        pub async fn create_credential(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CreateCredential",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_credential(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/UpdateCredential",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_credential(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DeleteCredential",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_cred_users(
            &mut self,
            request: impl tonic::IntoRequest<super::ListCredUsersRequest>,
        ) -> Result<tonic::Response<super::ListCredUsersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/ListCredUsers",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// https://wiki.lfaidata.foundation/display/MIL/MEP+29+--+Support+Role-Based+Access+Control
        pub async fn create_role(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CreateRole",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn drop_role(
            &mut self,
            request: impl tonic::IntoRequest<super::DropRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/DropRole",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn operate_user_role(
            &mut self,
            request: impl tonic::IntoRequest<super::OperateUserRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/OperateUserRole",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn select_role(
            &mut self,
            request: impl tonic::IntoRequest<super::SelectRoleRequest>,
        ) -> Result<tonic::Response<super::SelectRoleResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/SelectRole",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn select_user(
            &mut self,
            request: impl tonic::IntoRequest<super::SelectUserRequest>,
        ) -> Result<tonic::Response<super::SelectUserResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/SelectUser",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn operate_privilege(
            &mut self,
            request: impl tonic::IntoRequest<super::OperatePrivilegeRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/OperatePrivilege",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn select_grant(
            &mut self,
            request: impl tonic::IntoRequest<super::SelectGrantRequest>,
        ) -> Result<tonic::Response<super::SelectGrantResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/SelectGrant",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_version(
            &mut self,
            request: impl tonic::IntoRequest<super::GetVersionRequest>,
        ) -> Result<tonic::Response<super::GetVersionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/GetVersion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn check_health(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckHealthRequest>,
        ) -> Result<tonic::Response<super::CheckHealthResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/CheckHealth",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod proxy_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ProxyServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ProxyServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ProxyServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ProxyServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ProxyServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn register_link(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterLinkRequest>,
        ) -> Result<tonic::Response<super::RegisterLinkResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.ProxyService/RegisterLink",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
