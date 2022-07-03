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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropAliasRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub alias: ::prost::alloc::string::String,
}
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
///*
/// Create collection in milvus
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
}
///*
/// Drop collection in milvus, also will drop data in collection.
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
///*
/// Check collection exist in milvus or not.
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BoolResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(bool, tag = "2")]
    pub value: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
///*
/// Get collection meta datas like: schema, collectionID, shards number ...
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
///*
/// DescribeCollection Response
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
}
///*
/// Load collection data into query nodes, then you can do vector search on this collection.
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
///*
/// Release collection data from query nodes, then you can't do vector search on this collection.
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
///*
/// Get collection statistics like row_count.
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
///*
/// Will return collection statistics in stats field like \[{key:"row_count",value:"1"}\]
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
    #[prost(string, repeated, tag = "5")]
    pub collection_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
///
/// Return basic collection infos.
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
    #[prost(int64, repeated, tag = "6")]
    pub in_memory_percentages: ::prost::alloc::vec::Vec<i64>,
    /// Indicate whether query service is available
    #[prost(bool, repeated, tag = "7")]
    pub query_service_available: ::prost::alloc::vec::Vec<bool>,
}
///
/// Create partition in created collection.
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPartitionStatisticsResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub stats: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
///
/// List all partitions for particular collection
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
    #[prost(enumeration = "ShowType", tag = "6")]
    pub r#type: i32,
}
///
/// List all partitions for particular collection response.
/// The returned datas are all rows, we can format to columns by therir index.
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
    #[prost(int64, repeated, tag = "6")]
    pub in_memory_percentages: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeSegmentRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(int64, tag = "3")]
    pub segment_id: i64,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowSegmentsRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(int64, tag = "3")]
    pub partition_id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowSegmentsResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, repeated, tag = "2")]
    pub segment_i_ds: ::prost::alloc::vec::Vec<i64>,
}
///
/// Create index for vector datas
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
}
///
/// Describe index response
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
///  Get index building progress
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexBuildProgressResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, tag = "2")]
    pub indexed_rows: i64,
    #[prost(int64, tag = "3")]
    pub total_rows: i64,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexStateResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(enumeration = "super::common::IndexState", tag = "2")]
    pub state: i32,
    #[prost(string, tag = "3")]
    pub fail_reason: ::prost::alloc::string::String,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hits {
    #[prost(int64, repeated, tag = "1")]
    pub i_ds: ::prost::alloc::vec::Vec<i64>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub row_data: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(float, repeated, tag = "3")]
    pub scores: ::prost::alloc::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchResults {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, optional, tag = "2")]
    pub results: ::core::option::Option<super::schema::SearchResultData>,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlushRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub collection_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlushResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(map = "string, message", tag = "3")]
    pub coll_seg_i_ds:
        ::std::collections::HashMap<::prost::alloc::string::String, super::schema::LongArray>,
}
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
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryResults {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub fields_data: ::prost::alloc::vec::Vec<super::schema::FieldData>,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VectorsArray {
    #[prost(oneof = "vectors_array::Array", tags = "1, 2")]
    pub array: ::core::option::Option<vectors_array::Array>,
}
/// Nested message and enum types in `VectorsArray`.
pub mod vectors_array {
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
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Array {
        #[prost(message, tag = "2")]
        IntDist(super::super::schema::IntArray),
        #[prost(message, tag = "3")]
        FloatDist(super::super::schema::FloatArray),
    }
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPersistentSegmentInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub infos: ::prost::alloc::vec::Vec<PersistentSegmentInfo>,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetQuerySegmentInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub infos: ::prost::alloc::vec::Vec<QuerySegmentInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DummyRequest {
    #[prost(string, tag = "1")]
    pub request_type: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DummyResponse {
    #[prost(string, tag = "1")]
    pub response: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterLinkRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterLinkResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::common::Address>,
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<super::common::Status>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMetricsRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// request is of jsonic format
    #[prost(string, tag = "2")]
    pub request: ::prost::alloc::string::String,
}
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
///
/// Do load balancing operation from src_nodeID to dst_nodeID.
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ManualCompactionRequest {
    #[prost(int64, tag = "1")]
    pub collection_id: i64,
    #[prost(uint64, tag = "2")]
    pub timetravel: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ManualCompactionResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, tag = "2")]
    pub compaction_id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionStateRequest {
    #[prost(int64, tag = "1")]
    pub compaction_id: i64,
}
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
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionPlansRequest {
    #[prost(int64, tag = "1")]
    pub compaction_id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCompactionPlansResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(enumeration = "super::common::CompactionState", tag = "2")]
    pub state: i32,
    #[prost(message, repeated, tag = "3")]
    pub merge_infos: ::prost::alloc::vec::Vec<CompactionMergeInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactionMergeInfo {
    #[prost(int64, repeated, tag = "1")]
    pub sources: ::prost::alloc::vec::Vec<i64>,
    #[prost(int64, tag = "2")]
    pub target: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetFlushStateRequest {
    #[prost(int64, repeated, tag = "1")]
    pub segment_i_ds: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetFlushStateResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(bool, tag = "2")]
    pub flushed: bool,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// id array of import tasks
    #[prost(int64, repeated, tag = "2")]
    pub tasks: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetImportStateRequest {
    /// id of an import task
    #[prost(int64, tag = "1")]
    pub task: i64,
}
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
    /// A flag indicating whether import data are queryable (i.e. loaded in query nodes)
    #[prost(bool, tag = "7")]
    pub data_queryable: bool,
    /// A flag indicating whether import data are indexed.
    #[prost(bool, tag = "8")]
    pub data_indexed: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListImportTasksRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListImportTasksResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// list of all import tasks
    #[prost(message, repeated, tag = "2")]
    pub tasks: ::prost::alloc::vec::Vec<GetImportStateResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetReplicasRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(int64, tag = "2")]
    pub collection_id: i64,
    #[prost(bool, tag = "3")]
    pub with_shard_nodes: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetReplicasResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(message, repeated, tag = "2")]
    pub replicas: ::prost::alloc::vec::Vec<ReplicaInfo>,
}
/// ReplicaGroup
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
// <https://wiki.lfaidata.foundation/display/MIL/MEP+27+--+Support+Basic+Authentication>

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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCredentialRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// Not useful for now
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListCredUsersResponse {
    /// Contain error_code and reason
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// username array
    #[prost(string, repeated, tag = "2")]
    pub usernames: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListCredUsersRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
}
/// <https://wiki.lfaidata.foundation/display/MIL/MEP+29+--+Support+Role-Based+Access+Control>
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoleEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateRoleRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// role
    #[prost(message, optional, tag = "2")]
    pub entity: ::core::option::Option<RoleEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropRoleRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// role name
    #[prost(string, tag = "2")]
    pub role_name: ::prost::alloc::string::String,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoleResult {
    #[prost(message, optional, tag = "1")]
    pub role: ::core::option::Option<RoleEntity>,
    #[prost(message, repeated, tag = "2")]
    pub users: ::prost::alloc::vec::Vec<UserEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectRoleResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// role result array
    #[prost(message, repeated, tag = "2")]
    pub results: ::prost::alloc::vec::Vec<RoleResult>,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserResult {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<UserEntity>,
    #[prost(message, repeated, tag = "2")]
    pub roles: ::prost::alloc::vec::Vec<RoleEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectUserResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// user result array
    #[prost(message, repeated, tag = "2")]
    pub result: ::prost::alloc::vec::Vec<UserResult>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceEntity {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivilegeEntity {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectResourceRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// resource
    #[prost(message, optional, tag = "2")]
    pub entity: ::core::option::Option<ResourceEntity>,
    /// include privilege info
    #[prost(bool, tag = "3")]
    pub include_privilege_info: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceResult {
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<ResourceEntity>,
    #[prost(message, repeated, tag = "2")]
    pub privileges: ::prost::alloc::vec::Vec<PrivilegeEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectResourceResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// resource result array
    #[prost(message, repeated, tag = "2")]
    pub results: ::prost::alloc::vec::Vec<ResourceResult>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrincipalEntity {
    /// principal type, including user, role
    #[prost(string, tag = "1")]
    pub principal_type: ::prost::alloc::string::String,
    /// principal, including user entity or role entity
    #[prost(oneof = "principal_entity::Principal", tags = "2, 3")]
    pub principal: ::core::option::Option<principal_entity::Principal>,
}
/// Nested message and enum types in `PrincipalEntity`.
pub mod principal_entity {
    /// principal, including user entity or role entity
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Principal {
        #[prost(message, tag = "2")]
        User(super::UserEntity),
        #[prost(message, tag = "3")]
        Role(super::RoleEntity),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantorEntity {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<UserEntity>,
    #[prost(message, optional, tag = "2")]
    pub privilege: ::core::option::Option<PrivilegeEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrantEntity {
    /// principal
    #[prost(message, optional, tag = "1")]
    pub principal: ::core::option::Option<PrincipalEntity>,
    /// resource
    #[prost(message, optional, tag = "2")]
    pub resource: ::core::option::Option<ResourceEntity>,
    /// resource name
    #[prost(string, tag = "3")]
    pub resource_name: ::prost::alloc::string::String,
    /// privilege
    #[prost(message, optional, tag = "4")]
    pub grantor: ::core::option::Option<GrantorEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectGrantRequest {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    /// grant
    #[prost(message, optional, tag = "2")]
    pub entity: ::core::option::Option<GrantEntity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectGrantResponse {
    /// Not useful for now
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// grant info array
    #[prost(message, repeated, tag = "2")]
    pub entities: ::prost::alloc::vec::Vec<GrantEntity>,
}
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MilvusExt {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
}
///
/// This is for ShowCollectionsRequest type field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ShowType {
    /// Will return all colloections
    All = 0,
    /// Will return loaded collections with their inMemory_percentages
    InMemory = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OperateUserRoleType {
    AddUserToRole = 0,
    RemoveUserFromRole = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OperatePrivilegeType {
    Grant = 0,
    Revoke = 1,
}
#[doc = r" Generated client implementations."]
pub mod milvus_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct MilvusServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MilvusServiceClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
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
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MilvusServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            MilvusServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn create_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        ) -> Result<tonic::Response<super::GetCollectionStatisticsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        pub async fn create_partition(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePartitionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        ) -> Result<tonic::Response<super::GetPartitionStatisticsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        pub async fn create_alias(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        pub async fn get_index_state(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIndexStateRequest>,
        ) -> Result<tonic::Response<super::GetIndexStateResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
        pub async fn get_index_build_progress(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIndexBuildProgressRequest>,
        ) -> Result<tonic::Response<super::GetIndexBuildProgressResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Insert");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRequest>,
        ) -> Result<tonic::Response<super::MutationResult>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Delete");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn search(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchRequest>,
        ) -> Result<tonic::Response<super::SearchResults>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Search");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::FlushRequest>,
        ) -> Result<tonic::Response<super::FlushResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Flush");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRequest>,
        ) -> Result<tonic::Response<super::QueryResults>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Query");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn calc_distance(
            &mut self,
            request: impl tonic::IntoRequest<super::CalcDistanceRequest>,
        ) -> Result<tonic::Response<super::CalcDistanceResults>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        ) -> Result<tonic::Response<super::GetPersistentSegmentInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Dummy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " TODO: remove"]
        pub async fn register_link(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterLinkRequest>,
        ) -> Result<tonic::Response<super::RegisterLinkResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+8+--+Add+metrics+for+proxy"]
        pub async fn get_metrics(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMetricsRequest>,
        ) -> Result<tonic::Response<super::GetMetricsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
        pub async fn load_balance(
            &mut self,
            request: impl tonic::IntoRequest<super::LoadBalanceRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+24+--+Support+bulk+load"]
        pub async fn import(
            &mut self,
            request: impl tonic::IntoRequest<super::ImportRequest>,
        ) -> Result<tonic::Response<super::ImportResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/Import");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_import_state(
            &mut self,
            request: impl tonic::IntoRequest<super::GetImportStateRequest>,
        ) -> Result<tonic::Response<super::GetImportStateResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+27+--+Support+Basic+Authentication"]
        pub async fn create_credential(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+29+--+Support+Role-Based+Access+Control"]
        pub async fn create_role(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/milvus.proto.milvus.MilvusService/DropRole");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn operate_user_role(
            &mut self,
            request: impl tonic::IntoRequest<super::OperateUserRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
        pub async fn select_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::SelectResourceRequest>,
        ) -> Result<tonic::Response<super::SelectResourceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/milvus.proto.milvus.MilvusService/SelectResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn operate_privilege(
            &mut self,
            request: impl tonic::IntoRequest<super::OperatePrivilegeRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
            self.inner.ready().await.map_err(|e| {
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
    }
}
#[doc = r" Generated client implementations."]
pub mod proxy_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct ProxyServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ProxyServiceClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
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
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ProxyServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            ProxyServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn register_link(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterLinkRequest>,
        ) -> Result<tonic::Response<super::RegisterLinkResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
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
#[doc = r" Generated server implementations."]
pub mod milvus_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with MilvusServiceServer."]
    #[async_trait]
    pub trait MilvusService: Send + Sync + 'static {
        async fn create_collection(
            &self,
            request: tonic::Request<super::CreateCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn drop_collection(
            &self,
            request: tonic::Request<super::DropCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn has_collection(
            &self,
            request: tonic::Request<super::HasCollectionRequest>,
        ) -> Result<tonic::Response<super::BoolResponse>, tonic::Status>;
        async fn load_collection(
            &self,
            request: tonic::Request<super::LoadCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn release_collection(
            &self,
            request: tonic::Request<super::ReleaseCollectionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn describe_collection(
            &self,
            request: tonic::Request<super::DescribeCollectionRequest>,
        ) -> Result<tonic::Response<super::DescribeCollectionResponse>, tonic::Status>;
        async fn get_collection_statistics(
            &self,
            request: tonic::Request<super::GetCollectionStatisticsRequest>,
        ) -> Result<tonic::Response<super::GetCollectionStatisticsResponse>, tonic::Status>;
        async fn show_collections(
            &self,
            request: tonic::Request<super::ShowCollectionsRequest>,
        ) -> Result<tonic::Response<super::ShowCollectionsResponse>, tonic::Status>;
        async fn create_partition(
            &self,
            request: tonic::Request<super::CreatePartitionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn drop_partition(
            &self,
            request: tonic::Request<super::DropPartitionRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn has_partition(
            &self,
            request: tonic::Request<super::HasPartitionRequest>,
        ) -> Result<tonic::Response<super::BoolResponse>, tonic::Status>;
        async fn load_partitions(
            &self,
            request: tonic::Request<super::LoadPartitionsRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn release_partitions(
            &self,
            request: tonic::Request<super::ReleasePartitionsRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn get_partition_statistics(
            &self,
            request: tonic::Request<super::GetPartitionStatisticsRequest>,
        ) -> Result<tonic::Response<super::GetPartitionStatisticsResponse>, tonic::Status>;
        async fn show_partitions(
            &self,
            request: tonic::Request<super::ShowPartitionsRequest>,
        ) -> Result<tonic::Response<super::ShowPartitionsResponse>, tonic::Status>;
        async fn create_alias(
            &self,
            request: tonic::Request<super::CreateAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn drop_alias(
            &self,
            request: tonic::Request<super::DropAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn alter_alias(
            &self,
            request: tonic::Request<super::AlterAliasRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn create_index(
            &self,
            request: tonic::Request<super::CreateIndexRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn describe_index(
            &self,
            request: tonic::Request<super::DescribeIndexRequest>,
        ) -> Result<tonic::Response<super::DescribeIndexResponse>, tonic::Status>;
        async fn get_index_state(
            &self,
            request: tonic::Request<super::GetIndexStateRequest>,
        ) -> Result<tonic::Response<super::GetIndexStateResponse>, tonic::Status>;
        async fn get_index_build_progress(
            &self,
            request: tonic::Request<super::GetIndexBuildProgressRequest>,
        ) -> Result<tonic::Response<super::GetIndexBuildProgressResponse>, tonic::Status>;
        async fn drop_index(
            &self,
            request: tonic::Request<super::DropIndexRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn insert(
            &self,
            request: tonic::Request<super::InsertRequest>,
        ) -> Result<tonic::Response<super::MutationResult>, tonic::Status>;
        async fn delete(
            &self,
            request: tonic::Request<super::DeleteRequest>,
        ) -> Result<tonic::Response<super::MutationResult>, tonic::Status>;
        async fn search(
            &self,
            request: tonic::Request<super::SearchRequest>,
        ) -> Result<tonic::Response<super::SearchResults>, tonic::Status>;
        async fn flush(
            &self,
            request: tonic::Request<super::FlushRequest>,
        ) -> Result<tonic::Response<super::FlushResponse>, tonic::Status>;
        async fn query(
            &self,
            request: tonic::Request<super::QueryRequest>,
        ) -> Result<tonic::Response<super::QueryResults>, tonic::Status>;
        async fn calc_distance(
            &self,
            request: tonic::Request<super::CalcDistanceRequest>,
        ) -> Result<tonic::Response<super::CalcDistanceResults>, tonic::Status>;
        async fn get_flush_state(
            &self,
            request: tonic::Request<super::GetFlushStateRequest>,
        ) -> Result<tonic::Response<super::GetFlushStateResponse>, tonic::Status>;
        async fn get_persistent_segment_info(
            &self,
            request: tonic::Request<super::GetPersistentSegmentInfoRequest>,
        ) -> Result<tonic::Response<super::GetPersistentSegmentInfoResponse>, tonic::Status>;
        async fn get_query_segment_info(
            &self,
            request: tonic::Request<super::GetQuerySegmentInfoRequest>,
        ) -> Result<tonic::Response<super::GetQuerySegmentInfoResponse>, tonic::Status>;
        async fn get_replicas(
            &self,
            request: tonic::Request<super::GetReplicasRequest>,
        ) -> Result<tonic::Response<super::GetReplicasResponse>, tonic::Status>;
        async fn dummy(
            &self,
            request: tonic::Request<super::DummyRequest>,
        ) -> Result<tonic::Response<super::DummyResponse>, tonic::Status>;
        #[doc = " TODO: remove"]
        async fn register_link(
            &self,
            request: tonic::Request<super::RegisterLinkRequest>,
        ) -> Result<tonic::Response<super::RegisterLinkResponse>, tonic::Status>;
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+8+--+Add+metrics+for+proxy"]
        async fn get_metrics(
            &self,
            request: tonic::Request<super::GetMetricsRequest>,
        ) -> Result<tonic::Response<super::GetMetricsResponse>, tonic::Status>;
        async fn load_balance(
            &self,
            request: tonic::Request<super::LoadBalanceRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn get_compaction_state(
            &self,
            request: tonic::Request<super::GetCompactionStateRequest>,
        ) -> Result<tonic::Response<super::GetCompactionStateResponse>, tonic::Status>;
        async fn manual_compaction(
            &self,
            request: tonic::Request<super::ManualCompactionRequest>,
        ) -> Result<tonic::Response<super::ManualCompactionResponse>, tonic::Status>;
        async fn get_compaction_state_with_plans(
            &self,
            request: tonic::Request<super::GetCompactionPlansRequest>,
        ) -> Result<tonic::Response<super::GetCompactionPlansResponse>, tonic::Status>;
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+24+--+Support+bulk+load"]
        async fn import(
            &self,
            request: tonic::Request<super::ImportRequest>,
        ) -> Result<tonic::Response<super::ImportResponse>, tonic::Status>;
        async fn get_import_state(
            &self,
            request: tonic::Request<super::GetImportStateRequest>,
        ) -> Result<tonic::Response<super::GetImportStateResponse>, tonic::Status>;
        async fn list_import_tasks(
            &self,
            request: tonic::Request<super::ListImportTasksRequest>,
        ) -> Result<tonic::Response<super::ListImportTasksResponse>, tonic::Status>;
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+27+--+Support+Basic+Authentication"]
        async fn create_credential(
            &self,
            request: tonic::Request<super::CreateCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn update_credential(
            &self,
            request: tonic::Request<super::UpdateCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn delete_credential(
            &self,
            request: tonic::Request<super::DeleteCredentialRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn list_cred_users(
            &self,
            request: tonic::Request<super::ListCredUsersRequest>,
        ) -> Result<tonic::Response<super::ListCredUsersResponse>, tonic::Status>;
        #[doc = " https://wiki.lfaidata.foundation/display/MIL/MEP+29+--+Support+Role-Based+Access+Control"]
        async fn create_role(
            &self,
            request: tonic::Request<super::CreateRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn drop_role(
            &self,
            request: tonic::Request<super::DropRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn operate_user_role(
            &self,
            request: tonic::Request<super::OperateUserRoleRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn select_role(
            &self,
            request: tonic::Request<super::SelectRoleRequest>,
        ) -> Result<tonic::Response<super::SelectRoleResponse>, tonic::Status>;
        async fn select_user(
            &self,
            request: tonic::Request<super::SelectUserRequest>,
        ) -> Result<tonic::Response<super::SelectUserResponse>, tonic::Status>;
        async fn select_resource(
            &self,
            request: tonic::Request<super::SelectResourceRequest>,
        ) -> Result<tonic::Response<super::SelectResourceResponse>, tonic::Status>;
        async fn operate_privilege(
            &self,
            request: tonic::Request<super::OperatePrivilegeRequest>,
        ) -> Result<tonic::Response<super::super::common::Status>, tonic::Status>;
        async fn select_grant(
            &self,
            request: tonic::Request<super::SelectGrantRequest>,
        ) -> Result<tonic::Response<super::SelectGrantResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct MilvusServiceServer<T: MilvusService> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: MilvusService> MilvusServiceServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MilvusServiceServer<T>
    where
        T: MilvusService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/milvus.proto.milvus.MilvusService/CreateCollection" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCollectionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::CreateCollectionRequest>
                        for CreateCollectionSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCollectionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_collection(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateCollectionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DropCollection" => {
                    #[allow(non_camel_case_types)]
                    struct DropCollectionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DropCollectionRequest>
                        for DropCollectionSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DropCollectionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).drop_collection(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DropCollectionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/HasCollection" => {
                    #[allow(non_camel_case_types)]
                    struct HasCollectionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::HasCollectionRequest>
                        for HasCollectionSvc<T>
                    {
                        type Response = super::BoolResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HasCollectionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).has_collection(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HasCollectionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/LoadCollection" => {
                    #[allow(non_camel_case_types)]
                    struct LoadCollectionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::LoadCollectionRequest>
                        for LoadCollectionSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LoadCollectionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).load_collection(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoadCollectionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ReleaseCollection" => {
                    #[allow(non_camel_case_types)]
                    struct ReleaseCollectionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::ReleaseCollectionRequest>
                        for ReleaseCollectionSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReleaseCollectionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).release_collection(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReleaseCollectionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DescribeCollection" => {
                    #[allow(non_camel_case_types)]
                    struct DescribeCollectionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::DescribeCollectionRequest>
                        for DescribeCollectionSvc<T>
                    {
                        type Response = super::DescribeCollectionResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DescribeCollectionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).describe_collection(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DescribeCollectionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetCollectionStatistics" => {
                    #[allow(non_camel_case_types)]
                    struct GetCollectionStatisticsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetCollectionStatisticsRequest>
                        for GetCollectionStatisticsSvc<T>
                    {
                        type Response = super::GetCollectionStatisticsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCollectionStatisticsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_collection_statistics(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCollectionStatisticsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ShowCollections" => {
                    #[allow(non_camel_case_types)]
                    struct ShowCollectionsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::ShowCollectionsRequest>
                        for ShowCollectionsSvc<T>
                    {
                        type Response = super::ShowCollectionsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ShowCollectionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).show_collections(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ShowCollectionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/CreatePartition" => {
                    #[allow(non_camel_case_types)]
                    struct CreatePartitionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::CreatePartitionRequest>
                        for CreatePartitionSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePartitionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_partition(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreatePartitionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DropPartition" => {
                    #[allow(non_camel_case_types)]
                    struct DropPartitionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DropPartitionRequest>
                        for DropPartitionSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DropPartitionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).drop_partition(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DropPartitionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/HasPartition" => {
                    #[allow(non_camel_case_types)]
                    struct HasPartitionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::HasPartitionRequest>
                        for HasPartitionSvc<T>
                    {
                        type Response = super::BoolResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HasPartitionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).has_partition(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HasPartitionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/LoadPartitions" => {
                    #[allow(non_camel_case_types)]
                    struct LoadPartitionsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::LoadPartitionsRequest>
                        for LoadPartitionsSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LoadPartitionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).load_partitions(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoadPartitionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ReleasePartitions" => {
                    #[allow(non_camel_case_types)]
                    struct ReleasePartitionsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::ReleasePartitionsRequest>
                        for ReleasePartitionsSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReleasePartitionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).release_partitions(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReleasePartitionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetPartitionStatistics" => {
                    #[allow(non_camel_case_types)]
                    struct GetPartitionStatisticsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetPartitionStatisticsRequest>
                        for GetPartitionStatisticsSvc<T>
                    {
                        type Response = super::GetPartitionStatisticsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPartitionStatisticsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_partition_statistics(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPartitionStatisticsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ShowPartitions" => {
                    #[allow(non_camel_case_types)]
                    struct ShowPartitionsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::ShowPartitionsRequest>
                        for ShowPartitionsSvc<T>
                    {
                        type Response = super::ShowPartitionsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ShowPartitionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).show_partitions(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ShowPartitionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/CreateAlias" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAliasSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::CreateAliasRequest>
                        for CreateAliasSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateAliasRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_alias(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateAliasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DropAlias" => {
                    #[allow(non_camel_case_types)]
                    struct DropAliasSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DropAliasRequest> for DropAliasSvc<T> {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DropAliasRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).drop_alias(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DropAliasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/AlterAlias" => {
                    #[allow(non_camel_case_types)]
                    struct AlterAliasSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::AlterAliasRequest> for AlterAliasSvc<T> {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AlterAliasRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).alter_alias(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AlterAliasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/CreateIndex" => {
                    #[allow(non_camel_case_types)]
                    struct CreateIndexSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::CreateIndexRequest>
                        for CreateIndexSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_index(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DescribeIndex" => {
                    #[allow(non_camel_case_types)]
                    struct DescribeIndexSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DescribeIndexRequest>
                        for DescribeIndexSvc<T>
                    {
                        type Response = super::DescribeIndexResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DescribeIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).describe_index(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DescribeIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetIndexState" => {
                    #[allow(non_camel_case_types)]
                    struct GetIndexStateSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::GetIndexStateRequest>
                        for GetIndexStateSvc<T>
                    {
                        type Response = super::GetIndexStateResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetIndexStateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_index_state(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetIndexStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetIndexBuildProgress" => {
                    #[allow(non_camel_case_types)]
                    struct GetIndexBuildProgressSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetIndexBuildProgressRequest>
                        for GetIndexBuildProgressSvc<T>
                    {
                        type Response = super::GetIndexBuildProgressResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetIndexBuildProgressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_index_build_progress(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetIndexBuildProgressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DropIndex" => {
                    #[allow(non_camel_case_types)]
                    struct DropIndexSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DropIndexRequest> for DropIndexSvc<T> {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DropIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).drop_index(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DropIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Insert" => {
                    #[allow(non_camel_case_types)]
                    struct InsertSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::InsertRequest> for InsertSvc<T> {
                        type Response = super::MutationResult;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InsertRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).insert(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InsertSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Delete" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DeleteRequest> for DeleteSvc<T> {
                        type Response = super::MutationResult;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Search" => {
                    #[allow(non_camel_case_types)]
                    struct SearchSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::SearchRequest> for SearchSvc<T> {
                        type Response = super::SearchResults;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).search(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SearchSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Flush" => {
                    #[allow(non_camel_case_types)]
                    struct FlushSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::FlushRequest> for FlushSvc<T> {
                        type Response = super::FlushResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FlushRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).flush(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FlushSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Query" => {
                    #[allow(non_camel_case_types)]
                    struct QuerySvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::QueryRequest> for QuerySvc<T> {
                        type Response = super::QueryResults;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).query(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = QuerySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/CalcDistance" => {
                    #[allow(non_camel_case_types)]
                    struct CalcDistanceSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::CalcDistanceRequest>
                        for CalcDistanceSvc<T>
                    {
                        type Response = super::CalcDistanceResults;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CalcDistanceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).calc_distance(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CalcDistanceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetFlushState" => {
                    #[allow(non_camel_case_types)]
                    struct GetFlushStateSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::GetFlushStateRequest>
                        for GetFlushStateSvc<T>
                    {
                        type Response = super::GetFlushStateResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetFlushStateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_flush_state(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetFlushStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetPersistentSegmentInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GetPersistentSegmentInfoSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetPersistentSegmentInfoRequest>
                        for GetPersistentSegmentInfoSvc<T>
                    {
                        type Response = super::GetPersistentSegmentInfoResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPersistentSegmentInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_persistent_segment_info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPersistentSegmentInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetQuerySegmentInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GetQuerySegmentInfoSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetQuerySegmentInfoRequest>
                        for GetQuerySegmentInfoSvc<T>
                    {
                        type Response = super::GetQuerySegmentInfoResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetQuerySegmentInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_query_segment_info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetQuerySegmentInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetReplicas" => {
                    #[allow(non_camel_case_types)]
                    struct GetReplicasSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::GetReplicasRequest>
                        for GetReplicasSvc<T>
                    {
                        type Response = super::GetReplicasResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetReplicasRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_replicas(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetReplicasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Dummy" => {
                    #[allow(non_camel_case_types)]
                    struct DummySvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DummyRequest> for DummySvc<T> {
                        type Response = super::DummyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DummyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).dummy(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DummySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/RegisterLink" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterLinkSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::RegisterLinkRequest>
                        for RegisterLinkSvc<T>
                    {
                        type Response = super::RegisterLinkResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RegisterLinkRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).register_link(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RegisterLinkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetMetrics" => {
                    #[allow(non_camel_case_types)]
                    struct GetMetricsSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::GetMetricsRequest> for GetMetricsSvc<T> {
                        type Response = super::GetMetricsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetMetricsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_metrics(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetMetricsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/LoadBalance" => {
                    #[allow(non_camel_case_types)]
                    struct LoadBalanceSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::LoadBalanceRequest>
                        for LoadBalanceSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LoadBalanceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).load_balance(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoadBalanceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetCompactionState" => {
                    #[allow(non_camel_case_types)]
                    struct GetCompactionStateSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetCompactionStateRequest>
                        for GetCompactionStateSvc<T>
                    {
                        type Response = super::GetCompactionStateResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCompactionStateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_compaction_state(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCompactionStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ManualCompaction" => {
                    #[allow(non_camel_case_types)]
                    struct ManualCompactionSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::ManualCompactionRequest>
                        for ManualCompactionSvc<T>
                    {
                        type Response = super::ManualCompactionResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ManualCompactionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).manual_compaction(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ManualCompactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetCompactionStateWithPlans" => {
                    #[allow(non_camel_case_types)]
                    struct GetCompactionStateWithPlansSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::GetCompactionPlansRequest>
                        for GetCompactionStateWithPlansSvc<T>
                    {
                        type Response = super::GetCompactionPlansResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCompactionPlansRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_compaction_state_with_plans(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCompactionStateWithPlansSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/Import" => {
                    #[allow(non_camel_case_types)]
                    struct ImportSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::ImportRequest> for ImportSvc<T> {
                        type Response = super::ImportResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ImportRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).import(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ImportSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/GetImportState" => {
                    #[allow(non_camel_case_types)]
                    struct GetImportStateSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::GetImportStateRequest>
                        for GetImportStateSvc<T>
                    {
                        type Response = super::GetImportStateResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetImportStateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_import_state(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetImportStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ListImportTasks" => {
                    #[allow(non_camel_case_types)]
                    struct ListImportTasksSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::ListImportTasksRequest>
                        for ListImportTasksSvc<T>
                    {
                        type Response = super::ListImportTasksResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListImportTasksRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_import_tasks(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListImportTasksSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/CreateCredential" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCredentialSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::CreateCredentialRequest>
                        for CreateCredentialSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCredentialRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_credential(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateCredentialSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/UpdateCredential" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateCredentialSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::UpdateCredentialRequest>
                        for UpdateCredentialSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateCredentialRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_credential(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateCredentialSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DeleteCredential" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCredentialSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::DeleteCredentialRequest>
                        for DeleteCredentialSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCredentialRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_credential(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteCredentialSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/ListCredUsers" => {
                    #[allow(non_camel_case_types)]
                    struct ListCredUsersSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::ListCredUsersRequest>
                        for ListCredUsersSvc<T>
                    {
                        type Response = super::ListCredUsersResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListCredUsersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_cred_users(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListCredUsersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/CreateRole" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRoleSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::CreateRoleRequest> for CreateRoleSvc<T> {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRoleRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_role(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/DropRole" => {
                    #[allow(non_camel_case_types)]
                    struct DropRoleSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::DropRoleRequest> for DropRoleSvc<T> {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DropRoleRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).drop_role(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DropRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/OperateUserRole" => {
                    #[allow(non_camel_case_types)]
                    struct OperateUserRoleSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::OperateUserRoleRequest>
                        for OperateUserRoleSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperateUserRoleRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).operate_user_role(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OperateUserRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/SelectRole" => {
                    #[allow(non_camel_case_types)]
                    struct SelectRoleSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::SelectRoleRequest> for SelectRoleSvc<T> {
                        type Response = super::SelectRoleResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SelectRoleRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).select_role(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SelectRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/SelectUser" => {
                    #[allow(non_camel_case_types)]
                    struct SelectUserSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::SelectUserRequest> for SelectUserSvc<T> {
                        type Response = super::SelectUserResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SelectUserRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).select_user(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SelectUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/SelectResource" => {
                    #[allow(non_camel_case_types)]
                    struct SelectResourceSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::SelectResourceRequest>
                        for SelectResourceSvc<T>
                    {
                        type Response = super::SelectResourceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SelectResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).select_resource(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SelectResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/OperatePrivilege" => {
                    #[allow(non_camel_case_types)]
                    struct OperatePrivilegeSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService>
                        tonic::server::UnaryService<super::OperatePrivilegeRequest>
                        for OperatePrivilegeSvc<T>
                    {
                        type Response = super::super::common::Status;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OperatePrivilegeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).operate_privilege(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OperatePrivilegeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/milvus.proto.milvus.MilvusService/SelectGrant" => {
                    #[allow(non_camel_case_types)]
                    struct SelectGrantSvc<T: MilvusService>(pub Arc<T>);
                    impl<T: MilvusService> tonic::server::UnaryService<super::SelectGrantRequest>
                        for SelectGrantSvc<T>
                    {
                        type Response = super::SelectGrantResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SelectGrantRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).select_grant(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SelectGrantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: MilvusService> Clone for MilvusServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: MilvusService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: MilvusService> tonic::transport::NamedService for MilvusServiceServer<T> {
        const NAME: &'static str = "milvus.proto.milvus.MilvusService";
    }
}
#[doc = r" Generated server implementations."]
pub mod proxy_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ProxyServiceServer."]
    #[async_trait]
    pub trait ProxyService: Send + Sync + 'static {
        async fn register_link(
            &self,
            request: tonic::Request<super::RegisterLinkRequest>,
        ) -> Result<tonic::Response<super::RegisterLinkResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ProxyServiceServer<T: ProxyService> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ProxyService> ProxyServiceServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ProxyServiceServer<T>
    where
        T: ProxyService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/milvus.proto.milvus.ProxyService/RegisterLink" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterLinkSvc<T: ProxyService>(pub Arc<T>);
                    impl<T: ProxyService> tonic::server::UnaryService<super::RegisterLinkRequest>
                        for RegisterLinkSvc<T>
                    {
                        type Response = super::RegisterLinkResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RegisterLinkRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).register_link(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RegisterLinkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: ProxyService> Clone for ProxyServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ProxyService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ProxyService> tonic::transport::NamedService for ProxyServiceServer<T> {
        const NAME: &'static str = "milvus.proto.milvus.ProxyService";
    }
}
