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

mod alias;
mod cdc;
mod client;
mod collection;
mod common;
mod database;
mod index;
mod iterator;
mod partition;
mod query;
mod rbac;
mod resource_group;
mod utility;

#[path = "aggressivehpctesting/mod.rs"]
mod aggressive_hpc_testing;
#[path = "testsoutsideoftestsuite/client_keepalive.rs"]
mod client_keepalive;
#[path = "testsoutsideoftestsuite/client_search.rs"]
mod client_search;
#[path = "testsoutsideoftestsuite/client_with_timeout.rs"]
mod client_with_timeout;
#[path = "testsoutsideoftestsuite/collection_get_collection_stats.rs"]
mod collection_get_collection_stats;
#[path = "testsoutsideoftestsuite/collection_list_collections.rs"]
mod collection_list_collections;
#[path = "testsoutsideoftestsuite/collection_release_collection.rs"]
mod collection_release_collection;
#[path = "testsoutsideoftestsuite/mutate_delete.rs"]
mod mutate_delete;

#[test]
fn public_v1_paths_remain_available() {
    fn accepts_v1_client(_: Option<milvus::client::Client>) {}
    fn accepts_v1_schema(_: Option<milvus::schema::CollectionSchema>) {}
    fn accepts_v1_value(_: Option<milvus::value::Value>) {}

    accepts_v1_client(None);
    accepts_v1_schema(None);
    accepts_v1_value(None);
}
