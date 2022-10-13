use strum_macros::{Display, EnumString};

use crate::proto::{
    common::{IndexState, KeyValuePair},
    milvus::IndexDescription,
};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, EnumString, Display)]
pub enum IndexType {
    #[strum(serialize = "FLAT")]
    Flat,
    #[strum(serialize = "BIN_FLAT")]
    BinFlat,
    #[strum(serialize = "IVF_FLAT")]
    IvfFlat,
    #[strum(serialize = "BIN_IVF_FLAT")]
    BinIvfFlat,
    #[strum(serialize = "IVF_PQ")]
    IvfPQ,
    #[strum(serialize = "IVF_SQ8")]
    IvfSQ8,
    #[strum(serialize = "IVF_SQ8_HYBRID")]
    IvfSQ8H,
    #[strum(serialize = "NSG")]
    NSG,
    #[strum(serialize = "HNSW")]
    HNSW,
    #[strum(serialize = "RHNSW_FLAT")]
    RHNSWFlat,
    #[strum(serialize = "RHNSW_PQ")]
    RHNSWPQ,
    #[strum(serialize = "RHNSW_SQ")]
    RHNSWSQ,
    #[strum(serialize = "IVF_HNSW")]
    IvfHNSW,
    #[strum(serialize = "ANNOY")]
    ANNOY,
    #[strum(serialize = "NGT_PANNG")]
    NGTPANNG,
    #[strum(serialize = "NGT_ONNG")]
    NGTONNG,
}

#[derive(Debug, Clone, Copy, EnumString, Display)]
pub enum MetricType {
    L2,
    IP,
    HAMMING,
    JACCARD,
    TANIMOTO,
    SUBSTRUCTURE,
    SUPERSTRUCTURE,
}

#[derive(Debug, Clone)]
pub struct IndexParams {
    name: String,
    index_type: IndexType,
    metric_type: MetricType,
    params: HashMap<String, String>,
}

impl IndexParams {
    pub fn new(
        name: String,
        index_type: IndexType,
        metric_type: MetricType,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            index_type,
            metric_type,
            params,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn index_type(&self) -> IndexType {
        self.index_type
    }

    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn extra_params(&self) -> HashMap<String, String> {
        HashMap::from([
            ("index_type".to_owned(), self.index_type().to_string()),
            ("metric_type".to_owned(), self.metric_type().to_string()),
            (
                "params".to_owned(),
                serde_json::to_string(&self.params()).unwrap(),
            ),
        ])
    }

    pub fn extra_kv_params(&self) -> Vec<KeyValuePair> {
        self.extra_params()
            .into_iter()
            .map(|(k, v)| KeyValuePair { key: k, value: v })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    field_name: String,
    id: i64,
    params: IndexParams,
    state: IndexState,
}

impl IndexInfo {
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn params(&self) -> &IndexParams {
        &self.params
    }

    pub fn state(&self) -> IndexState {
        self.state
    }
}

impl From<IndexDescription> for IndexInfo {
    fn from(description: IndexDescription) -> Self {
        let mut params: HashMap<String, String> = HashMap::from_iter(
            description
                .params
                .iter()
                .map(|kv| (kv.key.clone(), kv.value.clone())),
        );

        let index_type = IndexType::from_str(&params.remove("index_type").unwrap()).unwrap();
        let metric_type = MetricType::from_str(&params.remove("metric_type").unwrap()).unwrap();
        let params = serde_json::from_str(params.get("params").unwrap()).unwrap();

        let params = IndexParams::new(
            description.index_name.clone(),
            index_type,
            metric_type,
            params,
        );
        Self {
            field_name: description.field_name.clone(),
            id: description.index_id,
            params: params,
            state: description.state(),
        }
    }
}
