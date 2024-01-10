use crate::proto::common::ConsistencyLevel;

#[derive(Debug, Clone, Copy)]
pub struct CreateCollectionOptions {
    pub(crate) shard_num: i32,
    pub(crate) consistency_level: ConsistencyLevel,
}

impl Default for CreateCollectionOptions {
    fn default() -> Self {
        Self {
            shard_num: 0,
            consistency_level: ConsistencyLevel::Bounded,
        }
    }
}

impl CreateCollectionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shard_num(shard_num: i32) -> Self {
        Self::default().shard_num(shard_num)
    }

    pub fn with_consistency_level(consistency_level: ConsistencyLevel) -> Self {
        Self::default().consistency_level(consistency_level)
    }

    pub fn shard_num(mut self, shard_num: i32) -> Self {
        self.shard_num = shard_num;
        self
    }

    pub fn consistency_level(mut self, consistency_level: ConsistencyLevel) -> Self {
        self.consistency_level = consistency_level;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoadOptions {
    pub(crate) replica_number: i32,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self { replica_number: 1 }
    }
}

impl LoadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_replica_number(replica_number: i32) -> Self {
        Self::default().replica_number(replica_number)
    }

    pub fn replica_number(mut self, replica_number: i32) -> Self {
        self.replica_number = replica_number;
        self
    }
}

#[derive(Debug, Clone)]
pub struct GetLoadStateOptions {
    pub(crate) partition_names: Vec<String>,
}

impl Default for GetLoadStateOptions {
    fn default() -> Self {
        Self {
            partition_names: vec![],
        }
    }
}

impl GetLoadStateOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_partition_names(partition_names: Vec<String>) -> Self {
        Self::default().partition_names(partition_names)
    }

    pub fn partition_names(mut self, partition_names: Vec<String>) -> Self {
        self.partition_names = partition_names;
        self
    }
}