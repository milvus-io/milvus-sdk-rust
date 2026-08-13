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

//! Official Rust SDK for Milvus.
//!
//! # Which client API should I use?
//!
//! New applications should use [`v2::ClientV2`]. The V2 API uses validated request builders,
//! SDK-owned response types, typed errors, retry handling, and schema-aware data operations.
//! Import [`v2::prelude`] for the common V2 entry points:
//!
//! ```
//! use milvus::v2::prelude::*;
//!
//! let config = ConnectConfig::new()
//!     .uri("http://localhost:19530")
//!     .token("root:Milvus");
//! ```
//!
//! [`v1`] and the modules re-exported at the crate root are the original compatibility API. They
//! remain available for existing applications, but new features are added only to V2. Existing V1
//! users can continue importing paths such as `milvus::client`; new users should start under
//! `milvus::v2`.
//!
//! # Getting started
//!
//! 1. Create a connection with [`v2::ClientV2::new`] and [`v2::ConnectConfig`].
//! 2. Construct an operation through its validated builder in [`v2::request`].
//! 3. Call the corresponding [`v2::ClientV2`] method and inspect its SDK-owned response.
//!
//! The repository README and standalone tutorials provide complete create, insert, load, search,
//! query, administration, and cleanup workflows.
//!
//! # Optional diagnostics
//!
//! Enable the Cargo feature `tracing`, add `tracing-subscriber` with its `env-filter` feature to
//! the application, and initialize a subscriber before creating the client:
//!
//! ```ignore
//! use tracing_subscriber::EnvFilter;
//!
//! tracing_subscriber::fmt()
//!     .with_env_filter(EnvFilter::from_default_env())
//!     .init();
//! ```
//!
//! Set `RUST_LOG=milvus_sdk=debug` to observe retry, schema-cache, and polling events. More focused
//! targets are `milvus_sdk::retry`, `milvus_sdk::schema_cache`, and `milvus_sdk::polling`.

pub mod error;
#[doc(hidden)]
pub mod proto;
pub mod v1;
pub mod v2;

pub use v1::alias;
pub use v1::authentication;
pub use v1::cdc;
pub use v1::client;
pub use v1::collection;
pub use v1::data;
pub use v1::database;
pub use v1::index;
pub use v1::iterator;
pub use v1::mutate;
pub use v1::options;
pub use v1::partition;
pub use v1::query;
pub use v1::resource_group;
pub use v1::schema;
pub use v1::types;
pub use v1::utility;
pub use v1::value;
