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

//! Response types returned by index operations.

use crate::proto::milvus;
use crate::v2::error::Result;
pub use crate::v2::types::IndexDesc;

///////////////////////////////////////////////////////////////////////////////
// DescribeIndexResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 describe_index operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DescribeIndexResponse {
    pub(crate) indexes: Vec<IndexDesc>,
}

impl DescribeIndexResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            indexes: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DescribeIndexResponseBuilder {
        DescribeIndexResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the indexes.
    pub fn indexes(&self) -> &[IndexDesc] {
        &self.indexes
    }

    pub(crate) fn from_proto(v: milvus::DescribeIndexResponse) -> Result<Self> {
        Ok(Self {
            indexes: v
                .index_descriptions
                .into_iter()
                .map(IndexDesc::from_proto)
                .collect::<Result<_>>()?,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DescribeIndexResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DescribeIndexResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DescribeIndexResponseBuilder {
    value: DescribeIndexResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DescribeIndexResponseBuilder {
    /// Sets the indexes and returns the updated value.
    pub fn indexes(mut self, value: Vec<IndexDesc>) -> Self {
        self.value.indexes = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> DescribeIndexResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListIndexesResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_indexes operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListIndexesResponse {
    pub(crate) index_names: Vec<String>,
    pub(crate) indexes: Vec<IndexDesc>,
}

impl ListIndexesResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            index_names: Vec::new(),
            indexes: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListIndexesResponseBuilder {
        ListIndexesResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the index names.
    pub fn index_names(&self) -> &[String] {
        &self.index_names
    }

    /// Returns the indexes.
    pub fn indexes(&self) -> &[IndexDesc] {
        &self.indexes
    }

    pub(crate) fn from_proto(v: milvus::GetIndexStatisticsResponse) -> Result<Self> {
        let indexes: Vec<IndexDesc> = v
            .index_descriptions
            .into_iter()
            .map(IndexDesc::from_proto)
            .collect::<Result<_>>()?;
        let index_names = indexes.iter().map(|v| v.index_name.clone()).collect();
        Ok(Self {
            index_names,
            indexes,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListIndexesResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListIndexesResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListIndexesResponseBuilder {
    value: ListIndexesResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListIndexesResponseBuilder {
    /// Sets the index names and returns the updated value.
    pub fn index_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.index_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the indexes and returns the updated value.
    pub fn indexes(mut self, value: Vec<IndexDesc>) -> Self {
        self.value.indexes = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> ListIndexesResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod index_desc_tests {
    use super::IndexDesc;
    use crate::proto::{common, milvus};
    use crate::v2::types::{IndexType, MetricType};

    #[test]
    fn index_desc_separates_typed_and_extra_params() {
        let desc = IndexDesc::from_proto(milvus::IndexDescription {
            field_name: "embedding".into(),
            index_name: "embedding_idx".into(),
            params: vec![
                common::KeyValuePair {
                    key: "index_type".into(),
                    value: "HNSW".into(),
                },
                common::KeyValuePair {
                    key: "metric_type".into(),
                    value: "COSINE".into(),
                },
                common::KeyValuePair {
                    key: "params".into(),
                    value: r#"{"M":16,"efConstruction":"200"}"#.into(),
                },
            ],
            ..Default::default()
        })
        .expect("valid index params JSON");

        assert_eq!(desc.get_index_type().to_owned(), IndexType::Hnsw);
        assert_eq!(desc.get_metric_type().to_owned(), MetricType::Cosine);
        assert_eq!(
            desc.get_extra_params().get("M").map(String::as_str),
            Some("16")
        );
        assert_eq!(
            desc.get_extra_params()
                .get("efConstruction")
                .map(String::as_str),
            Some("200")
        );
    }

    fn malformed_index_description() -> milvus::IndexDescription {
        milvus::IndexDescription {
            field_name: "embedding".into(),
            index_name: "embedding_idx".into(),
            params: vec![common::KeyValuePair {
                key: "params".into(),
                value: "{not valid JSON".into(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn describe_index_rejects_malformed_params_json() {
        assert!(
            super::DescribeIndexResponse::from_proto(milvus::DescribeIndexResponse {
                index_descriptions: vec![malformed_index_description()],
                ..Default::default()
            })
            .is_err()
        );
    }

    #[test]
    fn list_indexes_rejects_malformed_params_json() {
        assert!(
            super::ListIndexesResponse::from_proto(milvus::GetIndexStatisticsResponse {
                index_descriptions: vec![malformed_index_description()],
                ..Default::default()
            })
            .is_err()
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn describe_index_response_default_values() {
        let value = DescribeIndexResponse::builder().build();
        let expected_indexes: Vec<IndexDesc> = Default::default();

        assert_eq!(value.indexes().to_owned(), expected_indexes);
    }

    #[test]
    fn describe_index_response_populated_values() {
        let indexes = vec![IndexDesc::new()];
        let value = DescribeIndexResponse::builder()
            .indexes(indexes.clone())
            .build();

        assert_eq!(value.indexes().to_owned(), indexes);
    }

    #[test]
    fn list_indexes_response_default_values() {
        let value = ListIndexesResponse::builder().build();
        let expected_index_names: Vec<String> = Default::default();
        let expected_indexes: Vec<IndexDesc> = Default::default();

        assert_eq!(value.index_names().to_owned(), expected_index_names);
        assert_eq!(value.indexes().to_owned(), expected_indexes);
    }

    #[test]
    fn list_indexes_response_populated_values() {
        let index_names = vec!["index_names-value".to_owned()];
        let indexes = vec![IndexDesc::new()];
        let value = ListIndexesResponse::builder()
            .index_names(index_names.clone())
            .indexes(indexes.clone())
            .build();

        assert_eq!(value.index_names().to_owned(), index_names);
        assert_eq!(value.indexes().to_owned(), indexes);
    }
}
