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

#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub(crate) replica_number: i32,
    pub(crate) resource_groups: Vec<String>,
    pub(crate) refresh: bool,
    pub(crate) load_fields: Vec<String>,
    pub(crate) skip_load_dynamic_field: bool,
    pub(crate) load_params: std::collections::HashMap<String, String>,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            replica_number: 1,
            resource_groups: vec![],
            refresh: false,
            load_fields: vec![],
            skip_load_dynamic_field: false,
            load_params: std::collections::HashMap::new(),
        }
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

    pub fn resource_groups(mut self, resource_groups: Vec<String>) -> Self {
        self.resource_groups = resource_groups;
        self
    }

    pub fn refresh(mut self, refresh: bool) -> Self {
        self.refresh = refresh;
        self
    }

    pub fn load_fields(mut self, load_fields: Vec<String>) -> Self {
        self.load_fields = load_fields;
        self
    }

    pub fn skip_load_dynamic_field(mut self, skip_load_dynamic_field: bool) -> Self {
        self.skip_load_dynamic_field = skip_load_dynamic_field;
        self
    }

    pub fn load_params(mut self, load_params: std::collections::HashMap<String, String>) -> Self {
        self.load_params = load_params;
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
