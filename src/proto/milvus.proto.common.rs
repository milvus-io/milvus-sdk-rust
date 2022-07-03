#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Status {
    #[prost(enumeration="ErrorCode", tag="1")]
    pub error_code: i32,
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValuePair {
    #[prost(string, tag="1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyDataPair {
    #[prost(string, tag="1")]
    pub key: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Blob {
    #[prost(bytes="vec", tag="1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Address {
    #[prost(string, tag="1")]
    pub ip: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub port: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBase {
    #[prost(enumeration="MsgType", tag="1")]
    pub msg_type: i32,
    #[prost(int64, tag="2")]
    pub msg_id: i64,
    #[prost(uint64, tag="3")]
    pub timestamp: u64,
    #[prost(int64, tag="4")]
    pub source_id: i64,
}
/// Don't Modify This. @czs
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgHeader {
    #[prost(message, optional, tag="1")]
    pub base: ::core::option::Option<MsgBase>,
}
/// Don't Modify This. @czs
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DmlMsgHeader {
    #[prost(message, optional, tag="1")]
    pub base: ::core::option::Option<MsgBase>,
    #[prost(string, tag="2")]
    pub shard_name: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    UnexpectedError = 1,
    ConnectFailed = 2,
    PermissionDenied = 3,
    CollectionNotExists = 4,
    IllegalArgument = 5,
    IllegalDimension = 7,
    IllegalIndexType = 8,
    IllegalCollectionName = 9,
    IllegalTopk = 10,
    IllegalRowRecord = 11,
    IllegalVectorId = 12,
    IllegalSearchResult = 13,
    FileNotFound = 14,
    MetaFailed = 15,
    CacheFailed = 16,
    CannotCreateFolder = 17,
    CannotCreateFile = 18,
    CannotDeleteFolder = 19,
    CannotDeleteFile = 20,
    BuildIndexError = 21,
    IllegalNlist = 22,
    IllegalMetricType = 23,
    OutOfMemory = 24,
    IndexNotExist = 25,
    EmptyCollection = 26,
    UpdateImportTaskFailure = 27,
    CollectionNameNotFound = 28,
    CreateCredentialFailure = 29,
    UpdateCredentialFailure = 30,
    DeleteCredentialFailure = 31,
    GetCredentialFailure = 32,
    ListCredUsersFailure = 33,
    /// internal error code.
    DdRequestRace = 1000,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum IndexState {
    None = 0,
    Unissued = 1,
    InProgress = 2,
    Finished = 3,
    Failed = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SegmentState {
    None = 0,
    NotExist = 1,
    Growing = 2,
    Sealed = 3,
    Flushed = 4,
    Flushing = 5,
    Dropped = 6,
    Importing = 7,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MsgType {
    Undefined = 0,
    /// DEFINITION REQUESTS: COLLECTION 
    CreateCollection = 100,
    DropCollection = 101,
    HasCollection = 102,
    DescribeCollection = 103,
    ShowCollections = 104,
    GetSystemConfigs = 105,
    LoadCollection = 106,
    ReleaseCollection = 107,
    CreateAlias = 108,
    DropAlias = 109,
    AlterAlias = 110,
    /// DEFINITION REQUESTS: PARTITION 
    CreatePartition = 200,
    DropPartition = 201,
    HasPartition = 202,
    DescribePartition = 203,
    ShowPartitions = 204,
    LoadPartitions = 205,
    ReleasePartitions = 206,
    /// DEFINE REQUESTS: SEGMENT 
    ShowSegments = 250,
    DescribeSegment = 251,
    LoadSegments = 252,
    ReleaseSegments = 253,
    HandoffSegments = 254,
    LoadBalanceSegments = 255,
    DescribeSegments = 256,
    /// DEFINITION REQUESTS: INDEX 
    CreateIndex = 300,
    DescribeIndex = 301,
    DropIndex = 302,
    /// MANIPULATION REQUESTS 
    Insert = 400,
    Delete = 401,
    Flush = 402,
    /// QUERY 
    Search = 500,
    SearchResult = 501,
    GetIndexState = 502,
    GetIndexBuildProgress = 503,
    GetCollectionStatistics = 504,
    GetPartitionStatistics = 505,
    Retrieve = 506,
    RetrieveResult = 507,
    WatchDmChannels = 508,
    RemoveDmChannels = 509,
    WatchQueryChannels = 510,
    RemoveQueryChannels = 511,
    SealedSegmentsChangeInfo = 512,
    WatchDeltaChannels = 513,
    GetShardLeaders = 514,
    GetReplicas = 515,
    /// DATA SERVICE 
    SegmentInfo = 600,
    SystemInfo = 601,
    GetRecoveryInfo = 602,
    GetSegmentState = 603,
    /// SYSTEM CONTROL 
    TimeTick = 1200,
    /// GOOSE TODO: Remove kQueryNodeStats
    QueryNodeStats = 1201,
    LoadIndex = 1202,
    RequestId = 1203,
    RequestTso = 1204,
    AllocateSegment = 1205,
    SegmentStatistics = 1206,
    SegmentFlushDone = 1207,
    DataNodeTt = 1208,
    /// Credential 
    CreateCredential = 1500,
    GetCredential = 1501,
    DeleteCredential = 1502,
    UpdateCredential = 1503,
    ListCredUsernames = 1504,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DslType {
    Dsl = 0,
    BoolExprV1 = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CompactionState {
    UndefiedState = 0,
    Executing = 1,
    Completed = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ConsistencyLevel {
    Strong = 0,
    /// default in PyMilvus
    Session = 1,
    Bounded = 2,
    Eventually = 3,
    /// Users pass their own `guarantee_timestamp`.
    Customized = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ImportState {
    ImportPending = 0,
    ImportFailed = 1,
    ImportStarted = 2,
    ImportDownloaded = 3,
    ImportParsed = 4,
    ImportPersisted = 5,
    DataQueryable = 6,
    DataIndexed = 7,
    ImportCompleted = 8,
}
