#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SegmentIndexData {
    #[prost(int64, tag = "1")]
    pub segment_id: i64,
    /// data from knownwhere
    #[prost(string, tag = "2")]
    pub index_data: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FederSegmentSearchResult {
    #[prost(int64, tag = "1")]
    pub segment_id: i64,
    #[prost(string, tag = "2")]
    pub visit_info: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListIndexedSegmentRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub index_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListIndexedSegmentResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    #[prost(int64, repeated, tag = "2")]
    pub segment_i_ds: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeSegmentIndexDataRequest {
    #[prost(message, optional, tag = "1")]
    pub base: ::core::option::Option<super::common::MsgBase>,
    #[prost(string, tag = "2")]
    pub collection_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub index_name: ::prost::alloc::string::String,
    #[prost(int64, repeated, tag = "4")]
    pub segments_i_ds: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescribeSegmentIndexDataResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<super::common::Status>,
    /// segmentID => segmentIndexData
    #[prost(map = "int64, message", tag = "2")]
    pub index_data: ::std::collections::HashMap<i64, SegmentIndexData>,
    #[prost(message, repeated, tag = "3")]
    pub index_params: ::prost::alloc::vec::Vec<super::common::KeyValuePair>,
}
