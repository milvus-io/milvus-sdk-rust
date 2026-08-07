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

//! Partition metadata types.

///////////////////////////////////////////////////////////////////////////////
// PartitionInfo
///////////////////////////////////////////////////////////////////////////////
/// Metadata describing a collection partition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PartitionInfo {
    pub(crate) name: String,
    pub(crate) id: i64,
    pub(crate) created_timestamp: u64,
    pub(crate) created_utc_timestamp: u64,
}

impl PartitionInfo {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            id: 0,
            created_timestamp: 0,
            created_utc_timestamp: 0,
        }
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = value.into();
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn id(mut self, value: i64) -> Self {
        self.id = value;
        self
    }

    pub fn set_id(&mut self, value: i64) -> &mut Self {
        self.id = value;
        self
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn created_timestamp(mut self, value: u64) -> Self {
        self.created_timestamp = value;
        self
    }

    pub fn set_created_timestamp(&mut self, value: u64) -> &mut Self {
        self.created_timestamp = value;
        self
    }

    pub fn get_created_timestamp(&self) -> u64 {
        self.created_timestamp
    }

    pub fn created_utc_timestamp(mut self, value: u64) -> Self {
        self.created_utc_timestamp = value;
        self
    }

    pub fn set_created_utc_timestamp(&mut self, value: u64) -> &mut Self {
        self.created_utc_timestamp = value;
        self
    }

    pub fn get_created_utc_timestamp(&self) -> u64 {
        self.created_utc_timestamp
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod direct_value_tests {
    use super::*;

    #[test]
    fn partition_info_default_values() {
        let value = PartitionInfo::new();
        let expected_name: String = String::new();
        let expected_id: i64 = 0;
        let expected_created_timestamp: u64 = 0;
        let expected_created_utc_timestamp: u64 = 0;

        assert_eq!(value.get_name().to_owned(), expected_name);
        assert_eq!(value.get_id().to_owned(), expected_id);
        assert_eq!(
            value.get_created_timestamp().to_owned(),
            expected_created_timestamp
        );
        assert_eq!(
            value.get_created_utc_timestamp(),
            expected_created_utc_timestamp
        );
    }

    #[test]
    fn partition_info_populated_values() {
        let name = "name-value".to_owned();
        let id = 7;
        let created_timestamp = 7;
        let created_utc_timestamp = 7;
        let value = PartitionInfo::new()
            .name(name.clone())
            .id(id.clone())
            .created_timestamp(created_timestamp.clone())
            .created_utc_timestamp(created_utc_timestamp.clone());

        assert_eq!(value.get_name().to_owned(), name);
        assert_eq!(value.get_id().to_owned(), id);
        assert_eq!(value.get_created_timestamp().to_owned(), created_timestamp);
        assert_eq!(
            value.get_created_utc_timestamp().to_owned(),
            created_utc_timestamp
        );
    }
}
