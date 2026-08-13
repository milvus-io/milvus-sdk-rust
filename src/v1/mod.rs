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

//! Legacy V1 compatibility API.
//!
//! This module contains the original Milvus Rust SDK API. It remains supported for existing
//! applications and receives compatibility, correctness, and security fixes. New SDK features are
//! implemented only in [`crate::v2`].
//!
//! New applications should use [`crate::v2::ClientV2`] and usually start with
//! [`crate::v2::prelude`]. Crate-root modules such as `crate::client`, `crate::collection`, and
//! `crate::query` are compatibility re-exports of modules in this V1 API.
//!
//! V1 is not marked with Rust's `#[deprecated]` attribute because doing so would introduce warnings
//! for existing users. Its maintenance status is documented here so users can choose V2 without a
//! compatibility-breaking change.

pub mod alias;
pub mod authentication;
pub mod cdc;
pub mod client;
pub mod collection;
mod config;
pub mod data;
pub mod database;
pub mod error;
pub mod index;
pub mod iterator;
pub mod mutate;
pub mod options;
pub mod partition;
pub mod query;
pub mod resource_group;
pub mod schema;
pub mod types;
pub mod utility;
pub mod value;
