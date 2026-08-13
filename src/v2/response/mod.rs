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

//! V2 response objects, grouped by Milvus functional area.
//!
//! Responses expose only SDK-owned domain data. Protobuf-to-domain conversion is crate-private and
//! lives beside the public response type. Read response fields through accessors; response values
//! are obtained from [`crate::v2::ClientV2`] methods and are not constructed by applications.
//!
//! For query and search operations, the recommended path is to iterate the response's result groups
//! and then each row. Use typed field accessors when the schema is known, or generic value accessors
//! when processing dynamic output.

use crate::v2::error::{Error, Result};

pub(crate) fn validate_parallel_array_len(
    response: &str,
    anchor_field: &str,
    expected: usize,
    field: &str,
    actual: usize,
) -> Result<()> {
    if actual == expected {
        return Ok(());
    }

    Err(Error::MalformedResponse(format!(
        "{response} has mismatched parallel arrays: {anchor_field} has {expected} entries but {field} has {actual}"
    )))
}

pub(crate) fn validate_optional_parallel_array_len(
    response: &str,
    anchor_field: &str,
    expected: usize,
    field: &str,
    actual: usize,
) -> Result<()> {
    if actual == 0 || actual == expected {
        return Ok(());
    }

    Err(Error::MalformedResponse(format!(
        "{response} has mismatched optional parallel array: {anchor_field} has {expected} entries but {field} has {actual}"
    )))
}

pub mod alias;
pub mod cdc;
pub mod collection;
pub mod database;
pub mod dml;
pub mod dql;
pub mod index;
pub mod partition;
pub mod rbac;
pub mod resource_group;
pub mod utility;
