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

//! Response types returned by data-manipulation operations.

use crate::proto::milvus;
use crate::v2::error::Result;
pub use crate::v2::types::Ids;

///////////////////////////////////////////////////////////////////////////////
// DmlResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 DML operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DmlResponse {
    pub(crate) ids: Ids,
    pub(crate) succeeded_indices: Vec<u32>,
    pub(crate) failed_indices: Vec<u32>,
    pub(crate) acknowledged: bool,
    pub(crate) insert_count: i64,
    pub(crate) delete_count: i64,
    pub(crate) upsert_count: i64,
    pub(crate) timestamp: u64,
}

impl DmlResponse {
    /// Creates an empty response used when the client short-circuits an empty
    /// mutation before issuing the RPC (matching pymilvus's empty-input behavior).
    pub(crate) fn empty() -> Self {
        Self {
            ids: Ids::default(),
            succeeded_indices: Vec::new(),
            failed_indices: Vec::new(),
            acknowledged: false,
            insert_count: 0,
            delete_count: 0,
            upsert_count: 0,
            timestamp: 0,
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> DmlResponseBuilder {
        DmlResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the ids.
    pub fn ids(&self) -> &Ids {
        &self.ids
    }

    /// Returns the succeeded indices.
    pub fn succeeded_indices(&self) -> &[u32] {
        &self.succeeded_indices
    }

    /// Returns the failed indices.
    pub fn failed_indices(&self) -> &[u32] {
        &self.failed_indices
    }

    /// Returns whether acknowledged.
    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged
    }

    /// Returns the insert count.
    pub fn insert_count(&self) -> i64 {
        self.insert_count
    }

    /// Returns the delete count.
    pub fn delete_count(&self) -> i64 {
        self.delete_count
    }

    /// Returns the upsert count.
    pub fn upsert_count(&self) -> i64 {
        self.upsert_count
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub(crate) fn from_proto(value: milvus::MutationResult) -> Result<Self> {
        Ok(Self {
            ids: Ids::from_proto(value.i_ds)?,
            succeeded_indices: value.succ_index,
            failed_indices: value.err_index,
            acknowledged: value.acknowledged,
            insert_count: value.insert_cnt,
            delete_count: value.delete_cnt,
            upsert_count: value.upsert_cnt,
            timestamp: value.timestamp,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// DmlResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DmlResponse.
#[derive(Debug, Clone)]
///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
pub(crate) struct DmlResponseBuilder {
    value: DmlResponse,
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl DmlResponseBuilder {
    /// Sets the ids and returns the updated value.
    pub fn ids(mut self, value: Ids) -> Self {
        self.value.ids = value;
        self
    }

    /// Sets the succeeded indices and returns the updated value.
    pub fn succeeded_indices(mut self, value: Vec<u32>) -> Self {
        self.value.succeeded_indices = value;
        self
    }

    /// Sets the failed indices and returns the updated value.
    pub fn failed_indices(mut self, value: Vec<u32>) -> Self {
        self.value.failed_indices = value;
        self
    }

    /// Sets the acknowledged and returns the updated value.
    pub fn acknowledged(mut self, value: bool) -> Self {
        self.value.acknowledged = value;
        self
    }

    /// Sets the insert count and returns the updated value.
    pub fn insert_count(mut self, value: i64) -> Self {
        self.value.insert_count = value;
        self
    }

    /// Sets the delete count and returns the updated value.
    pub fn delete_count(mut self, value: i64) -> Self {
        self.value.delete_count = value;
        self
    }

    /// Sets the upsert count and returns the updated value.
    pub fn upsert_count(mut self, value: i64) -> Self {
        self.value.upsert_count = value;
        self
    }

    /// Sets the timestamp and returns the updated value.
    pub fn timestamp(mut self, value: u64) -> Self {
        self.value.timestamp = value;
        self
    }

    /// Validates the configured values and builds the request.
    pub fn build(self) -> DmlResponse {
        self.value
    }
}

/// Response returned by the ClientV2 insert operation.
pub type InsertResponse = DmlResponse;

/// Response returned by the ClientV2 upsert operation.
pub type UpsertResponse = DmlResponse;

/// Response returned by the ClientV2 delete operation.
pub type DeleteResponse = DmlResponse;

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod dml_response_tests {
    use super::Ids;
    use crate::proto::schema;

    #[test]
    fn protobuf_ids_convert_to_shared_ids() {
        assert_eq!(Ids::from_proto(None).unwrap(), Ids::default());
        let int_ids = Ids::from_proto(Some(schema::IDs {
            id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                data: vec![1, 2],
                ..Default::default()
            })),
        }))
        .unwrap();
        assert_eq!(int_ids, Ids::Int64(vec![1, 2]));
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod builder_value_tests {
    use super::*;

    #[test]
    fn dml_response_default_values() {
        let value = DmlResponse::builder().build();
        let expected_ids: Ids = Default::default();
        let expected_succeeded_indices: Vec<u32> = Default::default();
        let expected_failed_indices: Vec<u32> = Default::default();
        let expected_acknowledged: bool = false;
        let expected_insert_count: i64 = 0;
        let expected_delete_count: i64 = 0;
        let expected_upsert_count: i64 = 0;
        let expected_timestamp: u64 = 0;

        assert_eq!(value.ids().to_owned(), expected_ids);
        assert_eq!(
            value.succeeded_indices().to_owned(),
            expected_succeeded_indices
        );
        assert_eq!(value.failed_indices().to_owned(), expected_failed_indices);
        assert_eq!(value.is_acknowledged().to_owned(), expected_acknowledged);
        assert_eq!(value.insert_count().to_owned(), expected_insert_count);
        assert_eq!(value.delete_count().to_owned(), expected_delete_count);
        assert_eq!(value.upsert_count().to_owned(), expected_upsert_count);
        assert_eq!(value.timestamp().to_owned(), expected_timestamp);
    }

    #[test]
    fn dml_response_populated_values() {
        let ids = Ids::VarChar(vec!["id".to_owned()]);
        let succeeded_indices = vec![7];
        let failed_indices = vec![7];
        let acknowledged = true;
        let insert_count = 7;
        let delete_count = 7;
        let upsert_count = 7;
        let timestamp = 7;
        let value = DmlResponse::builder()
            .ids(ids.clone())
            .succeeded_indices(succeeded_indices.clone())
            .failed_indices(failed_indices.clone())
            .acknowledged(acknowledged.clone())
            .insert_count(insert_count.clone())
            .delete_count(delete_count.clone())
            .upsert_count(upsert_count.clone())
            .timestamp(timestamp.clone())
            .build();

        assert_eq!(value.ids().to_owned(), ids);
        assert_eq!(value.succeeded_indices().to_owned(), succeeded_indices);
        assert_eq!(value.failed_indices().to_owned(), failed_indices);
        assert_eq!(value.is_acknowledged().to_owned(), acknowledged);
        assert_eq!(value.insert_count().to_owned(), insert_count);
        assert_eq!(value.delete_count().to_owned(), delete_count);
        assert_eq!(value.upsert_count().to_owned(), upsert_count);
        assert_eq!(value.timestamp().to_owned(), timestamp);
    }
}
