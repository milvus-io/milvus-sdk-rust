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

pub mod error;
pub mod proto;
pub mod v1;

mod config;
mod utils;

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
