// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Index parameters, descriptions, types, and state values.

use super::common::{pairs, MetricType};
use crate::proto::{common, milvus};
use crate::v2::error::Result;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// IndexStateCode
///////////////////////////////////////////////////////////////////////////////
/// Build state of a Milvus index.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum IndexStateCode {
    #[default]
    /// Represents the None case.
    None,
    /// Represents the Unissued case.
    Unissued,
    /// Represents the InProgress case.
    InProgress,
    /// Represents the Finished case.
    Finished,
    /// Represents the Failed case.
    Failed,
    /// Represents the Retry case.
    Retry,
}

impl IndexStateCode {
    pub(crate) fn from_proto(value: i32) -> Self {
        match common::IndexState::try_from(value).ok() {
            Some(common::IndexState::Unissued) => Self::Unissued,
            Some(common::IndexState::InProgress) => Self::InProgress,
            Some(common::IndexState::Finished) => Self::Finished,
            Some(common::IndexState::Failed) => Self::Failed,
            Some(common::IndexState::Retry) => Self::Retry,
            _ => Self::None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// IndexType
///////////////////////////////////////////////////////////////////////////////
/// Index implementation used for a field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum IndexType {
    #[default]
    /// Represents the Invalid case.
    Invalid,
    /// Represents the Flat case.
    Flat,
    /// Represents the IvfFlat case.
    IvfFlat,
    /// Represents the IvfSq8 case.
    IvfSq8,
    /// Represents the IvfPq case.
    IvfPq,
    /// Represents the Hnsw case.
    Hnsw,
    /// Represents the HnswSq case.
    HnswSq,
    /// Represents the HnswPq case.
    HnswPq,
    /// Represents the HnswPrq case.
    HnswPrq,
    /// Represents the DiskAnn case.
    DiskAnn,
    /// Represents the AutoIndex case.
    AutoIndex,
    /// Represents the Scann case.
    Scann,
    /// Represents the IvfRabitq case.
    IvfRabitq,
    /// Represents the Aisaq case.
    Aisaq,
    /// Represents the GpuIvfFlat case.
    GpuIvfFlat,
    /// Represents the GpuIvfPq case.
    GpuIvfPq,
    /// Represents the GpuBruteForce case.
    GpuBruteForce,
    /// Represents the GpuCagra case.
    GpuCagra,
    /// Represents the BinFlat case.
    BinFlat,
    /// Represents the BinIvfFlat case.
    BinIvfFlat,
    /// Represents the MinhashLsh case.
    MinhashLsh,
    /// Represents the Trie case.
    Trie,
    /// Represents the Ngram case.
    Ngram,
    /// Represents the Rtree case.
    Rtree,
    /// Represents the StlSort case.
    StlSort,
    /// Represents the Inverted case.
    Inverted,
    /// Represents the Bitmap case.
    Bitmap,
    /// Represents the SparseInvertedIndex case.
    SparseInvertedIndex,
    /// Represents the SparseWand case.
    SparseWand,
}

impl IndexType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "INVALID",
            Self::Flat => "FLAT",
            Self::IvfFlat => "IVF_FLAT",
            Self::IvfSq8 => "IVF_SQ8",
            Self::IvfPq => "IVF_PQ",
            Self::Hnsw => "HNSW",
            Self::HnswSq => "HNSW_SQ",
            Self::HnswPq => "HNSW_PQ",
            Self::HnswPrq => "HNSW_PRQ",
            Self::DiskAnn => "DISKANN",
            Self::AutoIndex => "AUTOINDEX",
            Self::Scann => "SCANN",
            Self::IvfRabitq => "IVF_RABITQ",
            Self::Aisaq => "AISAQ",
            Self::GpuIvfFlat => "GPU_IVF_FLAT",
            Self::GpuIvfPq => "GPU_IVF_PQ",
            Self::GpuBruteForce => "GPU_BRUTE_FORCE",
            Self::GpuCagra => "GPU_CAGRA",
            Self::BinFlat => "BIN_FLAT",
            Self::BinIvfFlat => "BIN_IVF_FLAT",
            Self::MinhashLsh => "MINHASH_LSH",
            Self::Trie => "Trie",
            Self::Ngram => "NGRAM",
            Self::Rtree => "RTREE",
            Self::StlSort => "STL_SORT",
            Self::Inverted => "INVERTED",
            Self::Bitmap => "BITMAP",
            Self::SparseInvertedIndex => "SPARSE_INVERTED_INDEX",
            Self::SparseWand => "SPARSE_WAND",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "INVALID" => Self::Invalid,
            "FLAT" => Self::Flat,
            "IVF_FLAT" => Self::IvfFlat,
            "IVF_SQ8" => Self::IvfSq8,
            "IVF_PQ" => Self::IvfPq,
            "HNSW" => Self::Hnsw,
            "HNSW_SQ" => Self::HnswSq,
            "HNSW_PQ" => Self::HnswPq,
            "HNSW_PRQ" => Self::HnswPrq,
            "DISKANN" => Self::DiskAnn,
            "AUTOINDEX" => Self::AutoIndex,
            "SCANN" => Self::Scann,
            "IVF_RABITQ" => Self::IvfRabitq,
            "AISAQ" => Self::Aisaq,
            "GPU_IVF_FLAT" => Self::GpuIvfFlat,
            "GPU_IVF_PQ" => Self::GpuIvfPq,
            "GPU_BRUTE_FORCE" => Self::GpuBruteForce,
            "GPU_CAGRA" => Self::GpuCagra,
            "BIN_FLAT" => Self::BinFlat,
            "BIN_IVF_FLAT" => Self::BinIvfFlat,
            "MINHASH_LSH" => Self::MinhashLsh,
            "TRIE" => Self::Trie,
            "NGRAM" => Self::Ngram,
            "RTREE" => Self::Rtree,
            "STL_SORT" => Self::StlSort,
            "INVERTED" => Self::Inverted,
            "BITMAP" => Self::Bitmap,
            "SPARSE_INVERTED_INDEX" => Self::SparseInvertedIndex,
            "SPARSE_WAND" => Self::SparseWand,
            _ => Self::Invalid,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// IndexParam
///////////////////////////////////////////////////////////////////////////////
/// Parameters used to create an index for a field.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct IndexParam {
    pub(crate) field_name: String,
    pub(crate) index_name: String,
    pub(crate) index_type: IndexType,
    pub(crate) metric_type: Option<MetricType>,
    pub(crate) extra_params: HashMap<String, String>,
}

impl IndexParam {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            field_name: String::new(),
            index_name: String::new(),
            index_type: IndexType::Invalid,
            metric_type: None,
            extra_params: HashMap::new(),
        }
    }

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.field_name = value.into();
        self
    }

    /// Sets the field name and returns this value for further mutation.
    pub fn set_field_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.field_name = value.into();
        self
    }

    /// Returns the configured field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.index_name = value.into();
        self
    }

    /// Sets the index name and returns this value for further mutation.
    pub fn set_index_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.index_name = value.into();
        self
    }

    /// Returns the configured index name.
    pub fn get_index_name(&self) -> &str {
        &self.index_name
    }

    /// Sets the index type and returns the updated value.
    pub fn index_type(mut self, value: IndexType) -> Self {
        self.index_type = value;
        self
    }

    /// Sets the index type and returns this value for further mutation.
    pub fn set_index_type(&mut self, value: IndexType) -> &mut Self {
        self.index_type = value;
        self
    }

    /// Returns the configured index type.
    pub fn get_index_type(&self) -> IndexType {
        self.index_type
    }

    /// Sets the metric type and returns the updated value.
    pub fn metric_type(mut self, value: MetricType) -> Self {
        self.metric_type = Some(value);
        self
    }

    /// Sets the metric type and returns this value for further mutation.
    pub fn set_metric_type(&mut self, value: MetricType) -> &mut Self {
        self.metric_type = Some(value);
        self
    }

    /// Returns the configured metric type.
    pub fn get_metric_type(&self) -> Option<MetricType> {
        self.metric_type
    }

    /// Sets the extra params and returns the updated value.
    pub fn extra_params(mut self, value: HashMap<String, String>) -> Self {
        self.extra_params = value;
        self
    }

    /// Sets the extra params and returns this value for further mutation.
    pub fn set_extra_params(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.extra_params = value;
        self
    }

    /// Returns the configured extra params.
    pub fn get_extra_params(&self) -> &HashMap<String, String> {
        &self.extra_params
    }

    pub(crate) fn into_proto(
        self,
        database_name: String,
        collection_name: String,
    ) -> crate::proto::milvus::CreateIndexRequest {
        let mut extra_params = self.extra_params;
        extra_params.remove("index_type");
        extra_params.remove("metric_type");
        let mut extra_params = pairs(extra_params);
        extra_params.push(common::KeyValuePair {
            key: "index_type".into(),
            value: self.index_type.as_str().into(),
        });
        if let Some(metric_type) = self
            .metric_type
            .filter(|value| *value != MetricType::Default)
        {
            extra_params.push(common::KeyValuePair {
                key: "metric_type".into(),
                value: metric_type.as_str().into(),
            });
        }
        crate::proto::milvus::CreateIndexRequest {
            base: None,
            db_name: database_name,
            collection_name,
            field_name: self.field_name,
            extra_params,
            index_name: self.index_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// IndexDesc
///////////////////////////////////////////////////////////////////////////////
/// Description and state of an existing index.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct IndexDesc {
    pub(crate) index_name: String,
    pub(crate) index_id: i64,
    pub(crate) field_name: String,
    pub(crate) index_type: IndexType,
    pub(crate) metric_type: MetricType,
    pub(crate) extra_params: HashMap<String, String>,
    pub(crate) indexed_rows: i64,
    pub(crate) total_rows: i64,
    pub(crate) pending_rows: i64,
    pub(crate) state: IndexStateCode,
    pub(crate) failure_reason: String,
    pub(crate) min_version: i32,
    pub(crate) max_version: i32,
}

impl IndexDesc {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            index_name: String::new(),
            index_id: 0,
            field_name: String::new(),
            index_type: IndexType::Invalid,
            metric_type: MetricType::Default,
            extra_params: HashMap::new(),
            indexed_rows: 0,
            total_rows: 0,
            pending_rows: 0,
            state: IndexStateCode::None,
            failure_reason: String::new(),
            min_version: 0,
            max_version: 0,
        }
    }

    /// Sets the index name and returns the updated value.
    pub fn index_name(mut self, value: impl Into<String>) -> Self {
        self.index_name = value.into();
        self
    }

    /// Sets the index name and returns this value for further mutation.
    pub fn set_index_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.index_name = value.into();
        self
    }

    /// Returns the configured index name.
    pub fn get_index_name(&self) -> &str {
        &self.index_name
    }

    /// Sets the index id and returns the updated value.
    pub fn index_id(mut self, value: i64) -> Self {
        self.index_id = value;
        self
    }

    /// Sets the index id and returns this value for further mutation.
    pub fn set_index_id(&mut self, value: i64) -> &mut Self {
        self.index_id = value;
        self
    }

    /// Returns the configured index id.
    pub fn get_index_id(&self) -> i64 {
        self.index_id
    }

    /// Sets the field name and returns the updated value.
    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.field_name = value.into();
        self
    }

    /// Sets the field name and returns this value for further mutation.
    pub fn set_field_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.field_name = value.into();
        self
    }

    /// Returns the configured field name.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// Sets the index type and returns the updated value.
    pub fn index_type(mut self, value: IndexType) -> Self {
        self.index_type = value;
        self
    }

    /// Sets the index type and returns this value for further mutation.
    pub fn set_index_type(&mut self, value: IndexType) -> &mut Self {
        self.index_type = value;
        self
    }

    /// Returns the configured index type.
    pub fn get_index_type(&self) -> IndexType {
        self.index_type
    }

    /// Sets the metric type and returns the updated value.
    pub fn metric_type(mut self, value: MetricType) -> Self {
        self.metric_type = value;
        self
    }

    /// Sets the metric type and returns this value for further mutation.
    pub fn set_metric_type(&mut self, value: MetricType) -> &mut Self {
        self.metric_type = value;
        self
    }

    /// Returns the configured metric type.
    pub fn get_metric_type(&self) -> MetricType {
        self.metric_type
    }

    /// Sets the extra params and returns the updated value.
    pub fn extra_params(mut self, value: HashMap<String, String>) -> Self {
        self.extra_params = value;
        self
    }

    /// Sets the extra params and returns this value for further mutation.
    pub fn set_extra_params(&mut self, value: HashMap<String, String>) -> &mut Self {
        self.extra_params = value;
        self
    }

    /// Returns the configured extra params.
    pub fn get_extra_params(&self) -> &HashMap<String, String> {
        &self.extra_params
    }

    /// Sets the indexed rows and returns the updated value.
    pub fn indexed_rows(mut self, value: i64) -> Self {
        self.indexed_rows = value;
        self
    }

    /// Sets the indexed rows and returns this value for further mutation.
    pub fn set_indexed_rows(&mut self, value: i64) -> &mut Self {
        self.indexed_rows = value;
        self
    }

    /// Returns the configured indexed rows.
    pub fn get_indexed_rows(&self) -> i64 {
        self.indexed_rows
    }

    /// Sets the total rows and returns the updated value.
    pub fn total_rows(mut self, value: i64) -> Self {
        self.total_rows = value;
        self
    }

    /// Sets the total rows and returns this value for further mutation.
    pub fn set_total_rows(&mut self, value: i64) -> &mut Self {
        self.total_rows = value;
        self
    }

    /// Returns the configured total rows.
    pub fn get_total_rows(&self) -> i64 {
        self.total_rows
    }

    /// Sets the pending rows and returns the updated value.
    pub fn pending_rows(mut self, value: i64) -> Self {
        self.pending_rows = value;
        self
    }

    /// Sets the pending rows and returns this value for further mutation.
    pub fn set_pending_rows(&mut self, value: i64) -> &mut Self {
        self.pending_rows = value;
        self
    }

    /// Returns the configured pending rows.
    pub fn get_pending_rows(&self) -> i64 {
        self.pending_rows
    }

    /// Sets the state and returns the updated value.
    pub fn state(mut self, value: IndexStateCode) -> Self {
        self.state = value;
        self
    }

    /// Sets the state and returns this value for further mutation.
    pub fn set_state(&mut self, value: IndexStateCode) -> &mut Self {
        self.state = value;
        self
    }

    /// Returns the configured state.
    pub fn get_state(&self) -> IndexStateCode {
        self.state
    }

    /// Sets the failure reason and returns the updated value.
    pub fn failure_reason(mut self, value: impl Into<String>) -> Self {
        self.failure_reason = value.into();
        self
    }

    /// Sets the failure reason and returns this value for further mutation.
    pub fn set_failure_reason(&mut self, value: impl Into<String>) -> &mut Self {
        self.failure_reason = value.into();
        self
    }

    /// Returns the configured failure reason.
    pub fn get_failure_reason(&self) -> &str {
        &self.failure_reason
    }

    /// Sets the min version and returns the updated value.
    pub fn min_version(mut self, value: i32) -> Self {
        self.min_version = value;
        self
    }

    /// Sets the min version and returns this value for further mutation.
    pub fn set_min_version(&mut self, value: i32) -> &mut Self {
        self.min_version = value;
        self
    }

    /// Returns the configured min version.
    pub fn get_min_version(&self) -> i32 {
        self.min_version
    }

    /// Sets the max version and returns the updated value.
    pub fn max_version(mut self, value: i32) -> Self {
        self.max_version = value;
        self
    }

    /// Sets the max version and returns this value for further mutation.
    pub fn set_max_version(&mut self, value: i32) -> &mut Self {
        self.max_version = value;
        self
    }

    /// Returns the configured max version.
    pub fn get_max_version(&self) -> i32 {
        self.max_version
    }

    pub(crate) fn from_proto(v: milvus::IndexDescription) -> Result<Self> {
        let mut index_type = IndexType::Invalid;
        let mut metric_type = MetricType::Default;
        let mut extra_params = HashMap::new();
        for param in v.params {
            match param.key.as_str() {
                "index_type" => index_type = IndexType::from_str(&param.value),
                "metric_type" => metric_type = MetricType::from_str(&param.value),
                "params" => {
                    let values =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(&param.value)?;
                    extra_params.extend(values.into_iter().map(|(key, value)| {
                        let value = value
                            .as_str()
                            .map(ToOwned::to_owned)
                            .unwrap_or_else(|| value.to_string());
                        (key, value)
                    }));
                }
                _ => {
                    extra_params.insert(param.key, param.value);
                }
            }
        }
        Ok(Self {
            index_name: v.index_name,
            index_id: v.index_id,
            field_name: v.field_name,
            index_type,
            metric_type,
            extra_params,
            indexed_rows: v.indexed_rows,
            total_rows: v.total_rows,
            pending_rows: v.pending_index_rows,
            state: IndexStateCode::from_proto(v.state),
            failure_reason: v.index_state_fail_reason,
            min_version: v.min_index_version,
            max_version: v.max_index_version,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod index_param_tests {
    use super::{IndexParam, IndexType, MetricType};
    use std::collections::HashMap;

    #[test]
    fn index_param_converts_fields_to_rpc_params() {
        let param = IndexParam::new()
            .field_name("embedding")
            .index_type(IndexType::Hnsw)
            .metric_type(MetricType::Cosine)
            .index_name("embedding_idx")
            .extra_params(HashMap::from([("M".into(), "16".into())]));

        let proto = param.into_proto("default".into(), "books".into());
        let params: HashMap<_, _> = proto
            .extra_params
            .into_iter()
            .map(|pair| (pair.key, pair.value))
            .collect();
        assert_eq!(proto.field_name, "embedding");
        assert_eq!(proto.index_name, "embedding_idx");
        assert_eq!(params.get("index_type").map(String::as_str), Some("HNSW"));
        assert_eq!(
            params.get("metric_type").map(String::as_str),
            Some("COSINE")
        );
        assert_eq!(params.get("M").map(String::as_str), Some("16"));
    }

    #[test]
    fn default_metric_is_omitted_from_rpc_params() {
        let proto = IndexParam::new()
            .field_name("embedding")
            .index_type(IndexType::AutoIndex)
            .metric_type(MetricType::Default)
            .into_proto("default".into(), "books".into());

        assert!(proto
            .extra_params
            .iter()
            .all(|param| param.key != "metric_type"));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn index_param_constructor_values() {
        let value = IndexParam::new()
            .field_name("field")
            .index_type(IndexType::Invalid)
            .metric_type(MetricType::Default);
        let expected_field_name = "field";
        let expected_index_name: String = String::new();
        let expected_index_type: IndexType = IndexType::Invalid;
        let expected_metric_type = Some(MetricType::Default);
        let expected_extra_params: HashMap<String, String> = Default::default();

        assert_eq!(value.get_field_name(), expected_field_name);
        assert_eq!(value.get_index_name().to_owned(), expected_index_name);
        assert_eq!(value.get_index_type().to_owned(), expected_index_type);
        assert_eq!(value.get_metric_type().to_owned(), expected_metric_type);
        assert_eq!(value.get_extra_params().to_owned(), expected_extra_params);
    }

    #[test]
    fn index_param_populated_values() {
        let field_name = "field_name-value".to_owned();
        let index_name = "index_name-value".to_owned();
        let index_type = IndexType::Flat;
        let metric_type = MetricType::Cosine;
        let extra_params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = IndexParam::new()
            .field_name(field_name.clone())
            .index_type(index_type)
            .metric_type(metric_type)
            .index_name(index_name.clone())
            .extra_params(extra_params.clone());

        assert_eq!(value.get_field_name().to_owned(), field_name);
        assert_eq!(value.get_index_name().to_owned(), index_name);
        assert_eq!(value.get_index_type().to_owned(), index_type);
        assert_eq!(value.get_metric_type().to_owned(), Some(metric_type));
        assert_eq!(value.get_extra_params().to_owned(), extra_params);

        let constructed = IndexParam::new()
            .field_name(field_name.clone())
            .index_type(index_type)
            .metric_type(metric_type);
        assert_eq!(constructed.get_field_name().to_owned(), field_name);
        assert!(constructed.get_index_name().is_empty());
        assert_eq!(constructed.get_index_type().to_owned(), index_type);
        assert_eq!(constructed.get_metric_type().to_owned(), Some(metric_type));
    }

    #[test]
    fn index_desc_default_values() {
        let value = IndexDesc::new();
        let expected_index_name: String = String::new();
        let expected_index_id: i64 = 0;
        let expected_field_name: String = String::new();
        let expected_index_type: IndexType = Default::default();
        let expected_metric_type: MetricType = Default::default();
        let expected_extra_params: HashMap<String, String> = Default::default();
        let expected_indexed_rows: i64 = 0;
        let expected_total_rows: i64 = 0;
        let expected_pending_rows: i64 = 0;
        let expected_state: IndexStateCode = Default::default();
        let expected_failure_reason: String = String::new();
        let expected_min_version: i32 = 0;
        let expected_max_version: i32 = 0;

        assert_eq!(value.get_index_name().to_owned(), expected_index_name);
        assert_eq!(value.get_index_id().to_owned(), expected_index_id);
        assert_eq!(value.get_field_name().to_owned(), expected_field_name);
        assert_eq!(value.get_index_type().to_owned(), expected_index_type);
        assert_eq!(value.get_metric_type().to_owned(), expected_metric_type);
        assert_eq!(value.get_extra_params().to_owned(), expected_extra_params);
        assert_eq!(value.get_indexed_rows().to_owned(), expected_indexed_rows);
        assert_eq!(value.get_total_rows().to_owned(), expected_total_rows);
        assert_eq!(value.get_pending_rows().to_owned(), expected_pending_rows);
        assert_eq!(value.get_state().to_owned(), expected_state);
        assert_eq!(
            value.get_failure_reason().to_owned(),
            expected_failure_reason
        );
        assert_eq!(value.get_min_version().to_owned(), expected_min_version);
        assert_eq!(value.get_max_version().to_owned(), expected_max_version);
    }

    #[test]
    fn index_desc_populated_values() {
        let index_name = "index_name-value".to_owned();
        let index_id = 7;
        let field_name = "field_name-value".to_owned();
        let index_type = IndexType::Flat;
        let metric_type = MetricType::Cosine;
        let extra_params = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let indexed_rows = 7;
        let total_rows = 7;
        let pending_rows = 7;
        let state = IndexStateCode::Finished;
        let failure_reason = "failure_reason-value".to_owned();
        let min_version = 7;
        let max_version = 7;
        let value = IndexDesc::new()
            .index_name(index_name.clone())
            .index_id(index_id.clone())
            .field_name(field_name.clone())
            .index_type(index_type.clone())
            .metric_type(metric_type.clone())
            .extra_params(extra_params.clone())
            .indexed_rows(indexed_rows.clone())
            .total_rows(total_rows.clone())
            .pending_rows(pending_rows.clone())
            .state(state.clone())
            .failure_reason(failure_reason.clone())
            .min_version(min_version.clone())
            .max_version(max_version.clone());

        assert_eq!(value.get_index_name().to_owned(), index_name);
        assert_eq!(value.get_index_id().to_owned(), index_id);
        assert_eq!(value.get_field_name().to_owned(), field_name);
        assert_eq!(value.get_index_type().to_owned(), index_type);
        assert_eq!(value.get_metric_type().to_owned(), metric_type);
        assert_eq!(value.get_extra_params().to_owned(), extra_params);
        assert_eq!(value.get_indexed_rows().to_owned(), indexed_rows);
        assert_eq!(value.get_total_rows().to_owned(), total_rows);
        assert_eq!(value.get_pending_rows().to_owned(), pending_rows);
        assert_eq!(value.get_state().to_owned(), state);
        assert_eq!(value.get_failure_reason().to_owned(), failure_reason);
        assert_eq!(value.get_min_version().to_owned(), min_version);
        assert_eq!(value.get_max_version().to_owned(), max_version);
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod enum_conversion_tests {
    use super::*;

    #[test]
    fn index_state_converts_from_proto() {
        let cases = [
            (common::IndexState::None, IndexStateCode::None),
            (common::IndexState::Unissued, IndexStateCode::Unissued),
            (common::IndexState::InProgress, IndexStateCode::InProgress),
            (common::IndexState::Finished, IndexStateCode::Finished),
            (common::IndexState::Failed, IndexStateCode::Failed),
            (common::IndexState::Retry, IndexStateCode::Retry),
        ];

        for (proto, sdk) in cases {
            assert_eq!(IndexStateCode::from_proto(proto as i32), sdk);
        }
        assert_eq!(IndexStateCode::from_proto(i32::MAX), IndexStateCode::None);
    }

    #[test]
    fn index_type_round_trips_wire_name() {
        let values = [
            IndexType::Invalid,
            IndexType::Flat,
            IndexType::IvfFlat,
            IndexType::IvfSq8,
            IndexType::IvfPq,
            IndexType::Hnsw,
            IndexType::HnswSq,
            IndexType::HnswPq,
            IndexType::HnswPrq,
            IndexType::DiskAnn,
            IndexType::AutoIndex,
            IndexType::Scann,
            IndexType::IvfRabitq,
            IndexType::Aisaq,
            IndexType::GpuIvfFlat,
            IndexType::GpuIvfPq,
            IndexType::GpuBruteForce,
            IndexType::GpuCagra,
            IndexType::BinFlat,
            IndexType::BinIvfFlat,
            IndexType::MinhashLsh,
            IndexType::Trie,
            IndexType::Ngram,
            IndexType::Rtree,
            IndexType::StlSort,
            IndexType::Inverted,
            IndexType::Bitmap,
            IndexType::SparseInvertedIndex,
            IndexType::SparseWand,
        ];

        for value in values {
            assert_eq!(IndexType::from_str(value.as_str()), value);
        }
        assert_eq!(IndexType::from_str("None"), IndexType::Invalid);
        assert_eq!(IndexType::from_str("unknown"), IndexType::Invalid);
    }
}
