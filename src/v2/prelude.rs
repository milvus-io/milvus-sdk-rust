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

//! Convenient imports for ClientV2 workflows.
//!
//! Use `milvus::v2::prelude::*` to import the ClientV2 entry points, errors,
//! reusable types, and all public request and response objects. The functional
//! request and response modules remain available when explicit imports are
//! preferred.

pub use crate::v2::bulk_import::*;
pub use crate::v2::error::{ConversionError, Error, Result, ServerError, ValidationError};
pub use crate::v2::request::{
    alias::*, cdc::*, collection::*, database::*, dml::*, dql::*, index::*, partition::*, rbac::*,
    resource_group::*, snapshot::*, utility::*,
};
pub use crate::v2::response::{
    alias::*, cdc::*, collection::*, database::*, dml::*, dql::*, index::*, partition::*, rbac::*,
    resource_group::*, snapshot::*, utility::*,
};
pub use crate::v2::types::*;
pub use crate::v2::{ClientV2, OptimizeTask, QueryIterator, SearchIterator};
