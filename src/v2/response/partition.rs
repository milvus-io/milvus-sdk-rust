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

//! Response types returned by partition operations.

use crate::proto::milvus;
use crate::v2::error::Result;
use crate::v2::response::validate_parallel_array_len;
pub use crate::v2::types::PartitionInfo;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// GetPartitionStatsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_partition_stats operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetPartitionStatsResponse {
    pub(crate) statistics: HashMap<String, String>,
}

impl GetPartitionStatsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            statistics: HashMap::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetPartitionStatsResponseBuilder {
        GetPartitionStatsResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn statistics(&self) -> &HashMap<String, String> {
        &self.statistics
    }

    pub(crate) fn from_proto(value: milvus::GetPartitionStatisticsResponse) -> Self {
        Self {
            statistics: value.stats.into_iter().map(|v| (v.key, v.value)).collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetPartitionStatsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetPartitionStatsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct GetPartitionStatsResponseBuilder {
    value: GetPartitionStatsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl GetPartitionStatsResponseBuilder {
    pub fn statistics(mut self, value: HashMap<String, String>) -> Self {
        self.value.statistics = value;
        self
    }

    pub fn build(self) -> GetPartitionStatsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPartitionsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_partitions operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListPartitionsResponse {
    pub(crate) partition_names: Vec<String>,
    pub(crate) partitions: Vec<PartitionInfo>,
}

impl ListPartitionsResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self {
            partition_names: Vec::new(),
            partitions: Vec::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListPartitionsResponseBuilder {
        ListPartitionsResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn partition_names(&self) -> &[String] {
        &self.partition_names
    }

    pub fn partitions(&self) -> &[PartitionInfo] {
        &self.partitions
    }

    pub(crate) fn from_proto(value: milvus::ShowPartitionsResponse) -> Result<Self> {
        let count = value.partition_names.len();
        validate_parallel_array_len(
            "ShowPartitionsResponse",
            "partition_names",
            count,
            "partition_i_ds",
            value.partition_i_ds.len(),
        )?;
        validate_parallel_array_len(
            "ShowPartitionsResponse",
            "partition_names",
            count,
            "created_timestamps",
            value.created_timestamps.len(),
        )?;
        validate_parallel_array_len(
            "ShowPartitionsResponse",
            "partition_names",
            count,
            "created_utc_timestamps",
            value.created_utc_timestamps.len(),
        )?;

        let names = value.partition_names;
        let partitions = names
            .iter()
            .enumerate()
            .map(|(i, name)| PartitionInfo {
                name: name.clone(),
                id: value.partition_i_ds[i],
                created_timestamp: value.created_timestamps[i],
                created_utc_timestamp: value.created_utc_timestamps[i],
            })
            .collect();
        Ok(Self {
            partition_names: names,
            partitions,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListPartitionsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListPartitionsResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct ListPartitionsResponseBuilder {
    value: ListPartitionsResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl ListPartitionsResponseBuilder {
    pub fn partition_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.partition_names = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn partitions(mut self, value: Vec<PartitionInfo>) -> Self {
        self.value.partitions = value;
        self
    }

    pub fn build(self) -> ListPartitionsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasPartitionResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 has_partition operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HasPartitionResponse(pub(crate) bool);

impl HasPartitionResponse {
    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    fn empty() -> Self {
        Self(false)
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> HasPartitionResponseBuilder {
        HasPartitionResponseBuilder {
            value: Self::empty(),
        }
    }

    pub fn exists(&self) -> bool {
        self.0
    }
}

///////////////////////////////////////////////////////////////////////////////
// HasPartitionResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for HasPartitionResponse.
#[derive(Debug, Clone, Copy)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct HasPartitionResponseBuilder {
    value: HasPartitionResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl HasPartitionResponseBuilder {
    pub fn has(mut self, value: bool) -> Self {
        self.value.0 = value;
        self
    }

    pub fn build(self) -> HasPartitionResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::common::KeyValuePair;

    #[test]
    fn partition_response_methods_and_conversions() {
        let stats = GetPartitionStatsResponse::builder()
            .statistics(HashMap::from([("row_count".into(), "12".into())]))
            .build();
        assert_eq!(stats.statistics()["row_count"], "12");
        let stats = GetPartitionStatsResponse::from_proto(milvus::GetPartitionStatisticsResponse {
            stats: vec![KeyValuePair {
                key: "row_count".into(),
                value: "13".into(),
            }],
            ..Default::default()
        });
        assert_eq!(stats.statistics()["row_count"], "13");

        let info = PartitionInfo::new().name("p1").id(10);
        let list = ListPartitionsResponse::builder()
            .partition_names(["p1"])
            .partitions(vec![info])
            .build();
        assert_eq!(list.partition_names().to_owned(), ["p1"]);
        assert_eq!(list.partitions()[0].get_id().to_owned(), 10);
        let list = ListPartitionsResponse::from_proto(milvus::ShowPartitionsResponse {
            partition_names: vec!["p2".into()],
            partition_i_ds: vec![20],
            created_timestamps: vec![30],
            created_utc_timestamps: vec![40],
            ..Default::default()
        })
        .expect("aligned partition metadata");
        assert_eq!(list.partition_names().to_owned(), ["p2"]);
        assert_eq!(list.partitions()[0].get_id().to_owned(), 20);

        let has = HasPartitionResponse::builder().has(true).build();
        assert!(has.exists());
    }

    #[test]
    fn list_partitions_rejects_short_parallel_array() {
        let error = ListPartitionsResponse::from_proto(milvus::ShowPartitionsResponse {
            partition_names: vec!["p1".into(), "p2".into()],
            partition_i_ds: vec![1, 2],
            created_timestamps: vec![10],
            created_utc_timestamps: vec![100, 200],
            ..Default::default()
        })
        .expect_err("short partition timestamps must be rejected");

        assert!(matches!(
            error,
            crate::v2::error::Error::MalformedResponse(message) if message.contains("created_timestamps has 1")
        ));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn has_partition_response_default_values() {
        let value = HasPartitionResponse::builder().build();
        assert!(!value.exists());
    }

    #[test]
    fn has_partition_response_populated_values() {
        let value = HasPartitionResponse::builder().has(true).build();
        assert!(value.exists());
    }

    #[test]
    fn get_partition_stats_response_default_values() {
        let value = GetPartitionStatsResponse::builder().build();
        let expected_statistics: HashMap<String, String> = Default::default();

        assert_eq!(value.statistics().to_owned(), expected_statistics);
    }

    #[test]
    fn get_partition_stats_response_populated_values() {
        let statistics = HashMap::from([("key-value".to_owned(), "value-value".to_owned())]);
        let value = GetPartitionStatsResponse::builder()
            .statistics(statistics.clone())
            .build();

        assert_eq!(value.statistics().to_owned(), statistics);
    }

    #[test]
    fn list_partitions_response_default_values() {
        let value = ListPartitionsResponse::builder().build();
        let expected_partition_names: Vec<String> = Default::default();
        let expected_partitions: Vec<PartitionInfo> = Default::default();

        assert_eq!(value.partition_names().to_owned(), expected_partition_names);
        assert_eq!(value.partitions().to_owned(), expected_partitions);
    }

    #[test]
    fn list_partitions_response_populated_values() {
        let partition_names = vec!["partition_names-value".to_owned()];
        let partitions = vec![PartitionInfo::new()];
        let value = ListPartitionsResponse::builder()
            .partition_names(partition_names.clone())
            .partitions(partitions.clone())
            .build();

        assert_eq!(value.partition_names().to_owned(), partition_names);
        assert_eq!(value.partitions().to_owned(), partitions);
    }
}
