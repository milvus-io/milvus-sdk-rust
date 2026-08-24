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

//! Current request/response-based Milvus API for new applications.
//!
//! Start with [`ClientV2`], [`ConnectConfig`], and the types exported by [`prelude`]:
//!
//! ```
//! use milvus::v2::prelude::*;
//!
//! let config = ConnectConfig::new()
//!     .uri("http://localhost:19530")
//!     .token("root:Milvus");
//! ```
//!
//! Public operations accept validated values from [`request`] and return SDK-owned values from
//! [`response`]. Reusable schemas, enums, search parameters, and data values live in [`types`].
//! Request builders report invalid or conflicting parameters before an RPC is sent.
//!
//! Typical workflow:
//!
//! 1. Connect with [`ClientV2::new`].
//! 2. Build a request with `RequestType::builder().build()?`.
//! 3. Pass the request to the matching client method.
//! 4. Read the response through its accessors or result-row iterators.
//!
//! The V2 API is additive. [`crate::v1`] and crate-root compatibility exports remain available so
//! existing applications continue to compile, while new features are developed here.

#![warn(missing_docs)]

pub mod bulk_import;
pub mod client;
pub mod error;
pub mod prelude;
pub mod request;
pub mod response;
pub mod types;
pub mod utils;

pub use bulk_import::*;
pub use client::{
    new_client_request_id, with_client_request_id, ClientTelemetry, ClientTelemetryCommand,
    ClientTelemetryCommandReply, ClientV2, MilvusClientV2Session, OptimizeTask, QueryIterator,
    SearchIterator, SearchIteratorV1, SearchIteratorV2, TelemetryErrorInfo, TelemetryMetrics,
    TelemetryOperationMetrics, TelemetrySnapshot,
};
pub use types::*;
pub use utils::*;
