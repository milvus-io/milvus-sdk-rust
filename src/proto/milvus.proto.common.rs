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
pub struct PlaceholderValue {
    #[prost(string, tag="1")]
    pub tag: ::prost::alloc::string::String,
    #[prost(enumeration="PlaceholderType", tag="2")]
    pub r#type: i32,
    /// values is a 2d-array, every array contains a vector
    #[prost(bytes="vec", repeated, tag="3")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlaceholderGroup {
    #[prost(message, repeated, tag="1")]
    pub placeholders: ::prost::alloc::vec::Vec<PlaceholderValue>,
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivilegeExt {
    #[prost(enumeration="ObjectType", tag="1")]
    pub object_type: i32,
    #[prost(enumeration="ObjectPrivilege", tag="2")]
    pub object_privilege: i32,
    #[prost(int32, tag="3")]
    pub object_name_index: i32,
    #[prost(int32, tag="4")]
    pub object_name_indexs: i32,
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
    GetUserFailure = 34,
    CreateRoleFailure = 35,
    DropRoleFailure = 36,
    OperateUserRoleFailure = 37,
    SelectRoleFailure = 38,
    SelectUserFailure = 39,
    SelectResourceFailure = 40,
    OperatePrivilegeFailure = 41,
    SelectGrantFailure = 42,
    RefreshPolicyInfoCacheFailure = 43,
    ListPolicyFailure = 44,
    NotShardLeader = 45,
    NoReplicaAvailable = 46,
    SegmentNotFound = 47,
    ForceDeny = 48,
    RateLimit = 49,
    /// Service availability.
    /// NA: Not Available.
    DataCoordNa = 100,
    /// internal error code.
    DdRequestRace = 1000,
}
impl ErrorCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ErrorCode::Success => "Success",
            ErrorCode::UnexpectedError => "UnexpectedError",
            ErrorCode::ConnectFailed => "ConnectFailed",
            ErrorCode::PermissionDenied => "PermissionDenied",
            ErrorCode::CollectionNotExists => "CollectionNotExists",
            ErrorCode::IllegalArgument => "IllegalArgument",
            ErrorCode::IllegalDimension => "IllegalDimension",
            ErrorCode::IllegalIndexType => "IllegalIndexType",
            ErrorCode::IllegalCollectionName => "IllegalCollectionName",
            ErrorCode::IllegalTopk => "IllegalTOPK",
            ErrorCode::IllegalRowRecord => "IllegalRowRecord",
            ErrorCode::IllegalVectorId => "IllegalVectorID",
            ErrorCode::IllegalSearchResult => "IllegalSearchResult",
            ErrorCode::FileNotFound => "FileNotFound",
            ErrorCode::MetaFailed => "MetaFailed",
            ErrorCode::CacheFailed => "CacheFailed",
            ErrorCode::CannotCreateFolder => "CannotCreateFolder",
            ErrorCode::CannotCreateFile => "CannotCreateFile",
            ErrorCode::CannotDeleteFolder => "CannotDeleteFolder",
            ErrorCode::CannotDeleteFile => "CannotDeleteFile",
            ErrorCode::BuildIndexError => "BuildIndexError",
            ErrorCode::IllegalNlist => "IllegalNLIST",
            ErrorCode::IllegalMetricType => "IllegalMetricType",
            ErrorCode::OutOfMemory => "OutOfMemory",
            ErrorCode::IndexNotExist => "IndexNotExist",
            ErrorCode::EmptyCollection => "EmptyCollection",
            ErrorCode::UpdateImportTaskFailure => "UpdateImportTaskFailure",
            ErrorCode::CollectionNameNotFound => "CollectionNameNotFound",
            ErrorCode::CreateCredentialFailure => "CreateCredentialFailure",
            ErrorCode::UpdateCredentialFailure => "UpdateCredentialFailure",
            ErrorCode::DeleteCredentialFailure => "DeleteCredentialFailure",
            ErrorCode::GetCredentialFailure => "GetCredentialFailure",
            ErrorCode::ListCredUsersFailure => "ListCredUsersFailure",
            ErrorCode::GetUserFailure => "GetUserFailure",
            ErrorCode::CreateRoleFailure => "CreateRoleFailure",
            ErrorCode::DropRoleFailure => "DropRoleFailure",
            ErrorCode::OperateUserRoleFailure => "OperateUserRoleFailure",
            ErrorCode::SelectRoleFailure => "SelectRoleFailure",
            ErrorCode::SelectUserFailure => "SelectUserFailure",
            ErrorCode::SelectResourceFailure => "SelectResourceFailure",
            ErrorCode::OperatePrivilegeFailure => "OperatePrivilegeFailure",
            ErrorCode::SelectGrantFailure => "SelectGrantFailure",
            ErrorCode::RefreshPolicyInfoCacheFailure => "RefreshPolicyInfoCacheFailure",
            ErrorCode::ListPolicyFailure => "ListPolicyFailure",
            ErrorCode::NotShardLeader => "NotShardLeader",
            ErrorCode::NoReplicaAvailable => "NoReplicaAvailable",
            ErrorCode::SegmentNotFound => "SegmentNotFound",
            ErrorCode::ForceDeny => "ForceDeny",
            ErrorCode::RateLimit => "RateLimit",
            ErrorCode::DataCoordNa => "DataCoordNA",
            ErrorCode::DdRequestRace => "DDRequestRace",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum IndexState {
    None = 0,
    Unissued = 1,
    InProgress = 2,
    Finished = 3,
    Failed = 4,
    Retry = 5,
}
impl IndexState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            IndexState::None => "IndexStateNone",
            IndexState::Unissued => "Unissued",
            IndexState::InProgress => "InProgress",
            IndexState::Finished => "Finished",
            IndexState::Failed => "Failed",
            IndexState::Retry => "Retry",
        }
    }
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
impl SegmentState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SegmentState::None => "SegmentStateNone",
            SegmentState::NotExist => "NotExist",
            SegmentState::Growing => "Growing",
            SegmentState::Sealed => "Sealed",
            SegmentState::Flushed => "Flushed",
            SegmentState::Flushing => "Flushing",
            SegmentState::Dropped => "Dropped",
            SegmentState::Importing => "Importing",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PlaceholderType {
    None = 0,
    BinaryVector = 100,
    FloatVector = 101,
}
impl PlaceholderType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PlaceholderType::None => "None",
            PlaceholderType::BinaryVector => "BinaryVector",
            PlaceholderType::FloatVector => "FloatVector",
        }
    }
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
    ResendSegmentStats = 403,
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
    UnsubDmChannel = 516,
    GetDistribution = 517,
    SyncDistribution = 518,
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
    /// RBAC 
    CreateRole = 1600,
    DropRole = 1601,
    OperateUserRole = 1602,
    SelectRole = 1603,
    SelectUser = 1604,
    SelectResource = 1605,
    OperatePrivilege = 1606,
    SelectGrant = 1607,
    RefreshPolicyInfoCache = 1608,
    ListPolicy = 1609,
}
impl MsgType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MsgType::Undefined => "Undefined",
            MsgType::CreateCollection => "CreateCollection",
            MsgType::DropCollection => "DropCollection",
            MsgType::HasCollection => "HasCollection",
            MsgType::DescribeCollection => "DescribeCollection",
            MsgType::ShowCollections => "ShowCollections",
            MsgType::GetSystemConfigs => "GetSystemConfigs",
            MsgType::LoadCollection => "LoadCollection",
            MsgType::ReleaseCollection => "ReleaseCollection",
            MsgType::CreateAlias => "CreateAlias",
            MsgType::DropAlias => "DropAlias",
            MsgType::AlterAlias => "AlterAlias",
            MsgType::CreatePartition => "CreatePartition",
            MsgType::DropPartition => "DropPartition",
            MsgType::HasPartition => "HasPartition",
            MsgType::DescribePartition => "DescribePartition",
            MsgType::ShowPartitions => "ShowPartitions",
            MsgType::LoadPartitions => "LoadPartitions",
            MsgType::ReleasePartitions => "ReleasePartitions",
            MsgType::ShowSegments => "ShowSegments",
            MsgType::DescribeSegment => "DescribeSegment",
            MsgType::LoadSegments => "LoadSegments",
            MsgType::ReleaseSegments => "ReleaseSegments",
            MsgType::HandoffSegments => "HandoffSegments",
            MsgType::LoadBalanceSegments => "LoadBalanceSegments",
            MsgType::DescribeSegments => "DescribeSegments",
            MsgType::CreateIndex => "CreateIndex",
            MsgType::DescribeIndex => "DescribeIndex",
            MsgType::DropIndex => "DropIndex",
            MsgType::Insert => "Insert",
            MsgType::Delete => "Delete",
            MsgType::Flush => "Flush",
            MsgType::ResendSegmentStats => "ResendSegmentStats",
            MsgType::Search => "Search",
            MsgType::SearchResult => "SearchResult",
            MsgType::GetIndexState => "GetIndexState",
            MsgType::GetIndexBuildProgress => "GetIndexBuildProgress",
            MsgType::GetCollectionStatistics => "GetCollectionStatistics",
            MsgType::GetPartitionStatistics => "GetPartitionStatistics",
            MsgType::Retrieve => "Retrieve",
            MsgType::RetrieveResult => "RetrieveResult",
            MsgType::WatchDmChannels => "WatchDmChannels",
            MsgType::RemoveDmChannels => "RemoveDmChannels",
            MsgType::WatchQueryChannels => "WatchQueryChannels",
            MsgType::RemoveQueryChannels => "RemoveQueryChannels",
            MsgType::SealedSegmentsChangeInfo => "SealedSegmentsChangeInfo",
            MsgType::WatchDeltaChannels => "WatchDeltaChannels",
            MsgType::GetShardLeaders => "GetShardLeaders",
            MsgType::GetReplicas => "GetReplicas",
            MsgType::UnsubDmChannel => "UnsubDmChannel",
            MsgType::GetDistribution => "GetDistribution",
            MsgType::SyncDistribution => "SyncDistribution",
            MsgType::SegmentInfo => "SegmentInfo",
            MsgType::SystemInfo => "SystemInfo",
            MsgType::GetRecoveryInfo => "GetRecoveryInfo",
            MsgType::GetSegmentState => "GetSegmentState",
            MsgType::TimeTick => "TimeTick",
            MsgType::QueryNodeStats => "QueryNodeStats",
            MsgType::LoadIndex => "LoadIndex",
            MsgType::RequestId => "RequestID",
            MsgType::RequestTso => "RequestTSO",
            MsgType::AllocateSegment => "AllocateSegment",
            MsgType::SegmentStatistics => "SegmentStatistics",
            MsgType::SegmentFlushDone => "SegmentFlushDone",
            MsgType::DataNodeTt => "DataNodeTt",
            MsgType::CreateCredential => "CreateCredential",
            MsgType::GetCredential => "GetCredential",
            MsgType::DeleteCredential => "DeleteCredential",
            MsgType::UpdateCredential => "UpdateCredential",
            MsgType::ListCredUsernames => "ListCredUsernames",
            MsgType::CreateRole => "CreateRole",
            MsgType::DropRole => "DropRole",
            MsgType::OperateUserRole => "OperateUserRole",
            MsgType::SelectRole => "SelectRole",
            MsgType::SelectUser => "SelectUser",
            MsgType::SelectResource => "SelectResource",
            MsgType::OperatePrivilege => "OperatePrivilege",
            MsgType::SelectGrant => "SelectGrant",
            MsgType::RefreshPolicyInfoCache => "RefreshPolicyInfoCache",
            MsgType::ListPolicy => "ListPolicy",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DslType {
    Dsl = 0,
    BoolExprV1 = 1,
}
impl DslType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DslType::Dsl => "Dsl",
            DslType::BoolExprV1 => "BoolExprV1",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CompactionState {
    UndefiedState = 0,
    Executing = 1,
    Completed = 2,
}
impl CompactionState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CompactionState::UndefiedState => "UndefiedState",
            CompactionState::Executing => "Executing",
            CompactionState::Completed => "Completed",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ConsistencyLevel {
    Strong = 0,
    /// default in PyMilvus
    Session = 1,
    Bounded = 2,
    Eventually = 3,
    /// Users pass their own `guarantee_timestamp`. Deprecated
    Customized = 4,
}
impl ConsistencyLevel {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ConsistencyLevel::Strong => "Strong",
            ConsistencyLevel::Session => "Session",
            ConsistencyLevel::Bounded => "Bounded",
            ConsistencyLevel::Eventually => "Eventually",
            ConsistencyLevel::Customized => "Customized",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ImportState {
    /// the task in in pending list of rootCoord, waiting to be executed
    ImportPending = 0,
    /// the task failed for some reason, get detail reason from GetImportStateResponse.infos
    ImportFailed = 1,
    /// the task has been sent to datanode to execute
    ImportStarted = 2,
    /// all data files have been parsed and data already persisted
    ImportPersisted = 5,
    /// all indexes are successfully built and segments are able to be compacted as normal.
    ImportCompleted = 6,
    /// the task failed and all segments it generated are cleaned up.
    ImportFailedAndCleaned = 7,
}
impl ImportState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ImportState::ImportPending => "ImportPending",
            ImportState::ImportFailed => "ImportFailed",
            ImportState::ImportStarted => "ImportStarted",
            ImportState::ImportPersisted => "ImportPersisted",
            ImportState::ImportCompleted => "ImportCompleted",
            ImportState::ImportFailedAndCleaned => "ImportFailedAndCleaned",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ObjectType {
    Collection = 0,
    Global = 1,
    User = 2,
}
impl ObjectType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ObjectType::Collection => "Collection",
            ObjectType::Global => "Global",
            ObjectType::User => "User",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ObjectPrivilege {
    PrivilegeAll = 0,
    PrivilegeCreateCollection = 1,
    PrivilegeDropCollection = 2,
    PrivilegeDescribeCollection = 3,
    PrivilegeShowCollections = 4,
    PrivilegeLoad = 5,
    PrivilegeRelease = 6,
    PrivilegeCompaction = 7,
    PrivilegeInsert = 8,
    PrivilegeDelete = 9,
    PrivilegeGetStatistics = 10,
    PrivilegeCreateIndex = 11,
    PrivilegeIndexDetail = 12,
    PrivilegeDropIndex = 13,
    PrivilegeSearch = 14,
    PrivilegeFlush = 15,
    PrivilegeQuery = 16,
    PrivilegeLoadBalance = 17,
    PrivilegeImport = 18,
    PrivilegeCreateOwnership = 19,
    PrivilegeUpdateUser = 20,
    PrivilegeDropOwnership = 21,
    PrivilegeSelectOwnership = 22,
    PrivilegeManageOwnership = 23,
    PrivilegeSelectUser = 24,
    PrivilegeDescribePartition = 25,
}
impl ObjectPrivilege {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ObjectPrivilege::PrivilegeAll => "PrivilegeAll",
            ObjectPrivilege::PrivilegeCreateCollection => "PrivilegeCreateCollection",
            ObjectPrivilege::PrivilegeDropCollection => "PrivilegeDropCollection",
            ObjectPrivilege::PrivilegeDescribeCollection => "PrivilegeDescribeCollection",
            ObjectPrivilege::PrivilegeShowCollections => "PrivilegeShowCollections",
            ObjectPrivilege::PrivilegeLoad => "PrivilegeLoad",
            ObjectPrivilege::PrivilegeRelease => "PrivilegeRelease",
            ObjectPrivilege::PrivilegeCompaction => "PrivilegeCompaction",
            ObjectPrivilege::PrivilegeInsert => "PrivilegeInsert",
            ObjectPrivilege::PrivilegeDelete => "PrivilegeDelete",
            ObjectPrivilege::PrivilegeGetStatistics => "PrivilegeGetStatistics",
            ObjectPrivilege::PrivilegeCreateIndex => "PrivilegeCreateIndex",
            ObjectPrivilege::PrivilegeIndexDetail => "PrivilegeIndexDetail",
            ObjectPrivilege::PrivilegeDropIndex => "PrivilegeDropIndex",
            ObjectPrivilege::PrivilegeSearch => "PrivilegeSearch",
            ObjectPrivilege::PrivilegeFlush => "PrivilegeFlush",
            ObjectPrivilege::PrivilegeQuery => "PrivilegeQuery",
            ObjectPrivilege::PrivilegeLoadBalance => "PrivilegeLoadBalance",
            ObjectPrivilege::PrivilegeImport => "PrivilegeImport",
            ObjectPrivilege::PrivilegeCreateOwnership => "PrivilegeCreateOwnership",
            ObjectPrivilege::PrivilegeUpdateUser => "PrivilegeUpdateUser",
            ObjectPrivilege::PrivilegeDropOwnership => "PrivilegeDropOwnership",
            ObjectPrivilege::PrivilegeSelectOwnership => "PrivilegeSelectOwnership",
            ObjectPrivilege::PrivilegeManageOwnership => "PrivilegeManageOwnership",
            ObjectPrivilege::PrivilegeSelectUser => "PrivilegeSelectUser",
            ObjectPrivilege::PrivilegeDescribePartition => "PrivilegeDescribePartition",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum StateCode {
    Initializing = 0,
    Healthy = 1,
    Abnormal = 2,
    StandBy = 3,
}
impl StateCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            StateCode::Initializing => "Initializing",
            StateCode::Healthy => "Healthy",
            StateCode::Abnormal => "Abnormal",
            StateCode::StandBy => "StandBy",
        }
    }
}
