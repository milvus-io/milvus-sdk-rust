use crate::proto::common::ConsistencyLevel;

#[derive(Debug, Clone, Copy)]
pub struct CreateCollectionOptions {
    pub(crate) shard_num: i32,
    pub(crate) consistency_level: ConsistencyLevel,
}

impl Default for CreateCollectionOptions {
    fn default() -> Self {
        Self {
            shard_num: 2,
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
