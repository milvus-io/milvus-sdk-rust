pub mod utils;

use strum_macros::{Display, EnumString};

use crate::proto::{
    common::{IndexState, KeyValuePair},
    milvus::IndexDescription,
};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, EnumString, Display)]
pub enum IndexType {
    #[strum(serialize = "INVALID")]
    INVALID,
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
    #[strum(serialize = "HNSW")]
    HNSW,
    #[strum(serialize = "HNSW_SQ")]
    HNSWSQ,
    #[strum(serialize = "HNSW_PQ")]
    HNSWPQ,
    #[strum(serialize = "HNSW_PRQ")]
    HNSWPRQ,
    #[strum(serialize = "Trie")]
    Trie,
    #[strum(serialize = "BITMAP")]
    Bitmap,
    #[strum(serialize = "INVERTED")]
    Inverted,
    #[strum(serialize = "SPARSE_INVERTED_INDEX")]
    SparseInvertedIndex,
    #[strum(serialize = "SPARSE_WAND")]
    SparseWand,
    #[strum(serialize = "RTREE")]
    RTree,
    #[strum(serialize = "AUTOINDEX")]
    AutoIndex,
    #[strum(serialize = "DISKANN")]
    DiskANN,
    #[strum(serialize = "SCANN")]
    Scann,
    #[strum(serialize = "IVF_RABITQ")]
    IvfRabitQ,
    #[strum(serialize = "AISAQ")]
    Aisaq,
    #[strum(serialize = "GPU_IVF_FLAT")]
    GpuIvfFlat,
    #[strum(serialize = "GPU_IVF_PQ")]
    GpuIvfPQ,
    #[strum(serialize = "GPU_BRUTE_FORCE")]
    GpuBruteForce,
    #[strum(serialize = "GPU_CAGRA")]
    GpuCagra,
    #[strum(serialize = "MINHASH_LSH")]
    MinhashLsh,
    #[strum(serialize = "NGRAM")]
    Ngram,
    #[strum(serialize = "STL_SORT")]
    StlSort,
}

#[derive(Debug, Clone, Copy, EnumString, Display)]
pub enum MetricType {
    #[strum(serialize = "INVALID")]
    INVALID,
    L2,
    IP,
    COSINE,
    HAMMING,
    JACCARD,
    MHJACCARD,
    BM25,
    #[strum(serialize = "MAX_SIM", serialize = "MAX_SIM_COSINE")]
    MAX_SIM_COSINE,
    MAX_SIM_IP,
    MAX_SIM_L2,
    MAX_SIM_JACCARD,
    MAX_SIM_HAMMING,
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
    index_name: String,
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

    pub fn index_name(&self) -> &str {
        &self.index_name
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

        let index_type = params
            .remove("index_type")
            .and_then(|s| IndexType::from_str(&s).ok())
            .unwrap_or(IndexType::INVALID);
        let metric_type = params
            .remove("metric_type")
            .and_then(|s| MetricType::from_str(&s).ok())
            .unwrap_or(MetricType::INVALID);
        let extra_params = params
            .get("params")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let params = IndexParams::new(
            description.index_name.clone(),
            index_type,
            metric_type,
            extra_params,
        );
        Self {
            index_name: description.index_name.clone(),
            field_name: description.field_name.clone(),
            id: description.index_id,
            params,
            state: description.state(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexType, MetricType};
    use std::str::FromStr;

    #[test]
    fn index_type_round_trip_for_supported_variants() {
        let cases = [
            (IndexType::INVALID, "INVALID"),
            (IndexType::HNSWSQ, "HNSW_SQ"),
            (IndexType::HNSWPQ, "HNSW_PQ"),
            (IndexType::HNSWPRQ, "HNSW_PRQ"),
            (IndexType::Scann, "SCANN"),
            (IndexType::IvfRabitQ, "IVF_RABITQ"),
            (IndexType::Aisaq, "AISAQ"),
            (IndexType::GpuBruteForce, "GPU_BRUTE_FORCE"),
            (IndexType::GpuCagra, "GPU_CAGRA"),
            (IndexType::MinhashLsh, "MINHASH_LSH"),
            (IndexType::Ngram, "NGRAM"),
            (IndexType::StlSort, "STL_SORT"),
        ];

        for (variant, expected) in cases {
            assert_eq!(variant.to_string(), expected);
            assert_eq!(IndexType::from_str(expected).unwrap().to_string(), expected);
        }
    }

    #[test]
    fn metric_type_round_trip_for_supported_variants() {
        let cases = [
            (MetricType::INVALID, "INVALID"),
            (MetricType::MHJACCARD, "MHJACCARD"),
            (MetricType::MAX_SIM_COSINE, "MAX_SIM_COSINE"),
            (MetricType::MAX_SIM_IP, "MAX_SIM_IP"),
            (MetricType::MAX_SIM_L2, "MAX_SIM_L2"),
            (MetricType::MAX_SIM_JACCARD, "MAX_SIM_JACCARD"),
            (MetricType::MAX_SIM_HAMMING, "MAX_SIM_HAMMING"),
        ];

        for (variant, expected) in cases {
            assert_eq!(variant.to_string(), expected);
            assert_eq!(MetricType::from_str(expected).unwrap().to_string(), expected);
        }

        assert_eq!(
            MetricType::from_str("MAX_SIM").unwrap().to_string(),
            MetricType::MAX_SIM_COSINE.to_string()
        );
        assert_eq!(MetricType::MAX_SIM_COSINE.to_string(), "MAX_SIM_COSINE");
    }

}
