#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub partition_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "6")]
    pub db_id: i64,
    #[prost(int64, tag = "7")]
    pub collection_id: i64,
    #[prost(int64, tag = "8")]
    pub partition_id: i64,
    #[prost(int64, tag = "9")]
    pub segment_id: i64,
    #[prost(uint64, repeated, tag = "10")]
    pub timestamps: ::prost::alloc::vec::Vec<u64>,
    #[prost(int64, repeated, tag = "11")]
    pub row_i_ds: ::prost::alloc::vec::Vec<i64>,
    /// row_data was reserved for compatibility
    #[prost(message, repeated, tag = "12")]
    pub row_data: ::prost::alloc::vec::Vec<super::common::Blob>,
    #[prost(message, repeated, tag = "13")]
    pub fields_data: ::prost::alloc::vec::Vec<super::schema::FieldData>,
    #[prost(uint64, tag = "14")]
    pub num_rows: u64,
    #[prost(enumeration = "InsertDataVersion", tag = "15")]
    pub version: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub partition_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "6")]
    pub db_id: i64,
    #[prost(int64, tag = "7")]
    pub collection_id: i64,
    #[prost(int64, tag = "8")]
    pub partition_id: i64,
    /// deprecated
    #[prost(int64, repeated, tag = "9")]
    pub int64_primary_keys: ::prost::alloc::vec::Vec<i64>,
    #[prost(uint64, repeated, tag = "10")]
    pub timestamps: ::prost::alloc::vec::Vec<u64>,
    #[prost(int64, tag = "11")]
    pub num_rows: i64,
    #[prost(message, optional, tag = "12")]
    pub primary_keys: ::core::option::Option<super::schema::IDs>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPosition {
    #[prost(string, tag = "1")]
    pub channel_name: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub msg_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub msg_group: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCollectionRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
    /// `schema` is the serialized `schema.CollectionSchema`
    #[prost(int64, tag = "5")]
    pub db_id: i64,
    #[prost(int64, tag = "6")]
    pub collection_id: i64,
    /// deprecated
    #[prost(int64, tag = "7")]
    pub partition_id: i64,
    #[prost(bytes = "vec", tag = "8")]
    pub schema: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, repeated, tag = "9")]
    pub virtual_channel_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "10")]
    pub physical_channel_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(int64, repeated, tag = "11")]
    pub partition_i_ds: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropCollectionRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "4")]
    pub db_id: i64,
    #[prost(int64, tag = "5")]
    pub collection_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePartitionRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub db_id: i64,
    #[prost(int64, tag = "6")]
    pub collection_id: i64,
    #[prost(int64, tag = "7")]
    pub partition_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropPartitionRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub db_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub partition_name: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub db_id: i64,
    #[prost(int64, tag = "6")]
    pub collection_id: i64,
    #[prost(int64, tag = "7")]
    pub partition_id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeTickMsg {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataNodeTtMsg {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub channel_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    #[prost(message, repeated, tag = "4")]
    pub segments_stats: ::prost::alloc::vec::Vec<super::common::SegmentStats>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum InsertDataVersion {
    /// 0 must refer to row-based format, since it's the first version in Milvus.
    RowBased = 0,
    ColumnBased = 1,
}
impl InsertDataVersion {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            InsertDataVersion::RowBased => "RowBased",
            InsertDataVersion::ColumnBased => "ColumnBased",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RowBased" => Some(Self::RowBased),
            "ColumnBased" => Some(Self::ColumnBased),
            _ => None,
        }
    }
}
