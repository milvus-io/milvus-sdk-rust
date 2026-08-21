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

use milvus::proto::{common, milvus as pb, schema};
use milvus::v2::{ClientV2, ConnectConfig};
use pb::milvus_service_server::{MilvusService, MilvusServiceServer};
use prost::Message;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status};

fn success_status() -> common::Status {
    common::Status::default()
}

fn database_name(value: String) -> String {
    if value.is_empty() {
        "default".into()
    } else {
        value
    }
}

struct MockState {
    calls: HashMap<&'static str, usize>,
    requests: HashMap<&'static str, Vec<String>>,
    transport_failures: HashMap<&'static str, Vec<tonic::Code>>,
    aliases: HashMap<(String, String), String>,
    databases: HashMap<String, HashMap<String, String>>,
    replicate_configuration: Option<common::ReplicateConfiguration>,
    collections: HashMap<(String, String), pb::DescribeCollectionResponse>,
    loaded_collections: HashSet<(String, String)>,
    partitions: HashMap<(String, String), HashMap<String, i64>>,
    indexes: HashMap<(String, String, String), pb::IndexDescription>,
    resource_groups: HashMap<String, pb::ResourceGroup>,
    file_resources: HashMap<String, pb::FileResourceInfo>,
    users: HashMap<String, String>,
    roles: HashMap<String, String>,
    user_roles: HashSet<(String, String)>,
    grants: Vec<pb::GrantEntity>,
    privilege_groups: HashMap<String, HashSet<String>>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            calls: HashMap::new(),
            requests: HashMap::new(),
            transport_failures: HashMap::new(),
            aliases: HashMap::new(),
            databases: HashMap::new(),
            replicate_configuration: None,
            collections: HashMap::from([(
                ("default".into(), "books".into()),
                describe_collection_response(),
            )]),
            loaded_collections: HashSet::from([("default".into(), "books".into())]),
            partitions: HashMap::new(),
            indexes: HashMap::from([(
                ("default".into(), "books".into(), "vector_idx".into()),
                index_description(),
            )]),
            resource_groups: HashMap::from([(
                "default".into(),
                pb::ResourceGroup {
                    name: "default".into(),
                    capacity: 1,
                    num_available_node: 1,
                    ..Default::default()
                },
            )]),
            file_resources: HashMap::new(),
            users: HashMap::new(),
            roles: HashMap::new(),
            user_roles: HashSet::new(),
            grants: Vec::new(),
            privilege_groups: HashMap::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct MockMilvus {
    state: Arc<Mutex<MockState>>,
}

impl MockMilvus {
    fn record_request<T: Debug>(&self, method: &'static str, request: &T) {
        let mut state = self.state.lock().unwrap();
        *state.calls.entry(method).or_default() += 1;
        state
            .requests
            .entry(method)
            .or_default()
            .push(format!("{request:?}"));
    }

    pub fn call_count(&self, method: &'static str) -> usize {
        self.state
            .lock()
            .unwrap()
            .calls
            .get(method)
            .copied()
            .unwrap_or_default()
    }

    pub fn request_text(&self, method: &'static str) -> String {
        self.state
            .lock()
            .unwrap()
            .requests
            .get(method)
            .and_then(|requests| requests.last())
            .cloned()
            .unwrap_or_default()
    }

    pub fn request_texts(&self, method: &'static str) -> Vec<String> {
        self.state
            .lock()
            .unwrap()
            .requests
            .get(method)
            .cloned()
            .unwrap_or_default()
    }

    pub fn fail_next_transport(&self, method: &'static str, code: tonic::Code) {
        self.state
            .lock()
            .unwrap()
            .transport_failures
            .entry(method)
            .or_default()
            .push(code);
    }

    fn take_transport_failure(&self, method: &'static str) -> Option<Status> {
        let mut state = self.state.lock().unwrap();
        let failures = state.transport_failures.get_mut(method)?;
        if failures.is_empty() {
            return None;
        }
        Some(Status::new(
            failures.remove(0),
            "injected mock transport failure",
        ))
    }

    pub fn rename_collection_field(
        &self,
        database: &str,
        collection: &str,
        old_name: &str,
        new_name: &str,
    ) {
        let mut state = self.state.lock().unwrap();
        let description = state
            .collections
            .get_mut(&(database_name(database.to_owned()), collection.to_owned()))
            .expect("mock collection exists");
        let field = description
            .schema
            .as_mut()
            .and_then(|schema| {
                schema
                    .fields
                    .iter_mut()
                    .find(|field| field.name == old_name)
            })
            .expect("mock collection field exists");
        field.name = new_name.to_owned();
        description.update_timestamp = description.update_timestamp.saturating_add(1);
    }

    pub fn set_collection_auto_id(&self, database: &str, collection: &str, auto_id: bool) {
        let mut state = self.state.lock().unwrap();
        let description = state
            .collections
            .get_mut(&(database_name(database.to_owned()), collection.to_owned()))
            .expect("mock collection exists");
        let primary = description
            .schema
            .as_mut()
            .and_then(|schema| schema.fields.iter_mut().find(|field| field.is_primary_key))
            .expect("mock collection primary field exists");
        primary.auto_id = auto_id;
        description.update_timestamp = description.update_timestamp.saturating_add(1);
    }
}

macro_rules! status_method {
    ($name:ident, $request:ty) => {
        fn $name<'life0, 'async_trait>(
            &'life0 self,
            request: Request<$request>,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Response<common::Status>, Status>> + Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let request = request.into_inner();
                self.record_request(stringify!($name), &request);
                if let Some(status) = self.take_transport_failure(stringify!($name)) {
                    return Err(status);
                }
                Ok(Response::new(success_status()))
            })
        }
    };
}

macro_rules! response_method {
    ($name:ident, $request:ty, $response:ty, $value:expr) => {
        fn $name<'life0, 'async_trait>(
            &'life0 self,
            request: Request<$request>,
        ) -> Pin<Box<dyn Future<Output = Result<Response<$response>, Status>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let request = request.into_inner();
                self.record_request(stringify!($name), &request);
                if let Some(status) = self.take_transport_failure(stringify!($name)) {
                    return Err(status);
                }
                Ok(Response::new($value))
            })
        }
    };
}

macro_rules! status_method_with {
    ($name:ident, $request_type:ty, |$service:ident, $request:ident| $body:block) => {
        fn $name<'life0, 'async_trait>(
            &'life0 self,
            request: Request<$request_type>,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Response<common::Status>, Status>> + Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let $request = request.into_inner();
                self.record_request(stringify!($name), &$request);
                let $service = self;
                let status = $body;
                if let Some(status) = self.take_transport_failure(stringify!($name)) {
                    return Err(status);
                }
                Ok(Response::new(status))
            })
        }
    };
}

macro_rules! response_method_with {
    ($name:ident, $request_type:ty, $response_type:ty, |$service:ident, $request:ident| $body:block) => {
        fn $name<'life0, 'async_trait>(
            &'life0 self,
            request: Request<$request_type>,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Response<$response_type>, Status>> + Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let $request = request.into_inner();
                self.record_request(stringify!($name), &$request);
                let $service = self;
                let response = $body;
                Ok(Response::new(response))
            })
        }
    };
}

macro_rules! status_response_method {
    ($name:ident, $request:ty, $response:ty) => {
        response_method!($name, $request, $response, {
            let mut response = <$response>::default();
            response.status = Some(success_status());
            response
        });
    };
}

fn mock_schema() -> schema::CollectionSchema {
    schema::CollectionSchema {
        name: "books".into(),
        description: "mock books collection".into(),
        enable_dynamic_field: true,
        fields: vec![
            schema::FieldSchema {
                field_id: 100,
                name: "id".into(),
                is_primary_key: true,
                data_type: schema::DataType::Int64 as i32,
                ..Default::default()
            },
            schema::FieldSchema {
                field_id: 101,
                name: "text".into(),
                data_type: schema::DataType::VarChar as i32,
                type_params: vec![common::KeyValuePair {
                    key: "max_length".into(),
                    value: "128".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            schema::FieldSchema {
                field_id: 102,
                name: "vector".into(),
                data_type: schema::DataType::FloatVector as i32,
                type_params: vec![common::KeyValuePair {
                    key: "dim".into(),
                    value: "2".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            schema::FieldSchema {
                field_id: 103,
                name: "tags".into(),
                data_type: schema::DataType::Array as i32,
                element_type: schema::DataType::VarChar as i32,
                nullable: true,
                type_params: vec![
                    common::KeyValuePair {
                        key: "max_capacity".into(),
                        value: "32".into(),
                        ..Default::default()
                    },
                    common::KeyValuePair {
                        key: "max_length".into(),
                        value: "128".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn describe_collection_response() -> pb::DescribeCollectionResponse {
    pb::DescribeCollectionResponse {
        status: Some(success_status()),
        schema: Some(mock_schema()),
        collection_id: 1,
        collection_name: "books".into(),
        db_name: "default".into(),
        consistency_level: common::ConsistencyLevel::Bounded as i32,
        shards_num: 2,
        aliases: vec!["books_alias".into()],
        created_timestamp: 101,
        created_utc_timestamp: 102,
        update_timestamp: 103,
        num_partitions: 1,
        properties: vec![common::KeyValuePair {
            key: "retention".into(),
            value: "3600".into(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn index_description() -> pb::IndexDescription {
    pb::IndexDescription {
        index_name: "vector_idx".into(),
        field_name: "vector".into(),
        state: common::IndexState::Finished as i32,
        index_id: 10,
        indexed_rows: 1,
        total_rows: 1,
        params: vec![
            common::KeyValuePair {
                key: "index_type".into(),
                value: "HNSW".into(),
                ..Default::default()
            },
            common::KeyValuePair {
                key: "metric_type".into(),
                value: "COSINE".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn int64_field(name: &str, values: Vec<i64>) -> schema::FieldData {
    schema::FieldData {
        r#type: schema::DataType::Int64 as i32,
        field_name: name.into(),
        field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
            valid_data: Vec::new(),
            data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                data: values,
            })),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn string_field(name: &str, values: Vec<&str>) -> schema::FieldData {
    schema::FieldData {
        r#type: schema::DataType::VarChar as i32,
        field_name: name.into(),
        field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
            valid_data: Vec::new(),
            data: Some(schema::scalar_field::Data::StringData(
                schema::StringArray {
                    data: values.into_iter().map(str::to_owned).collect(),
                },
            )),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn search_response() -> pb::SearchResults {
    pb::SearchResults {
        status: Some(common::Status {
            extra_info: [
                ("report_value".into(), "7".into()),
                ("scanned_remote_bytes".into(), "11".into()),
                ("scanned_total_bytes".into(), "13".into()),
                ("cache_hit_ratio".into(), "0.5".into()),
            ]
            .into_iter()
            .collect(),
            ..success_status()
        }),
        results: Some(schema::SearchResultData {
            num_queries: 1,
            top_k: 1,
            topks: vec![1],
            scores: vec![0.9],
            ids: Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                    data: vec![1],
                })),
                ..Default::default()
            }),
            fields_data: vec![string_field("text", vec!["book"])],
            output_fields: vec!["text".into()],
            primary_field_name: "id".into(),
            search_iterator_v2_results: Some(schema::SearchIteratorV2Results {
                token: "mock-search-iterator".into(),
                last_bound: 0.0,
                ..Default::default()
            }),
            ..Default::default()
        }),
        collection_name: "books".into(),
        session_ts: 301,
        ..Default::default()
    }
}

#[allow(deprecated)]
impl MilvusService for MockMilvus {
    response_method!(
        connect,
        pb::ConnectRequest,
        pb::ConnectResponse,
        pb::ConnectResponse {
            status: Some(success_status()),
            server_info: Some(common::ServerInfo {
                build_tags: "mock-2.6-detail".into(),
                build_time: "2026-07-29".into(),
                git_commit: "abcdef".into(),
                go_version: "go1.24".into(),
                deploy_mode: "STANDALONE".into(),
                ..Default::default()
            }),
            identifier: 1,
            ..Default::default()
        }
    );
    status_method_with!(
        create_collection,
        pb::CreateCollectionRequest,
        |service, request| {
            let database = database_name(request.db_name);
            let schema =
                schema::CollectionSchema::decode(request.schema.as_slice()).unwrap_or_default();
            let collection_id = if request.collection_name == "books" {
                1
            } else {
                2
            };
            let response = pb::DescribeCollectionResponse {
                status: Some(success_status()),
                schema: Some(schema),
                collection_id,
                collection_name: request.collection_name.clone(),
                db_name: database.clone(),
                consistency_level: request.consistency_level,
                shards_num: request.shards_num,
                created_timestamp: 101 + collection_id as u64,
                created_utc_timestamp: 201 + collection_id as u64,
                update_timestamp: 301 + collection_id as u64,
                num_partitions: request.num_partitions,
                properties: request.properties,
                ..Default::default()
            };
            service
                .state
                .lock()
                .unwrap()
                .collections
                .insert((database, request.collection_name), response);
            success_status()
        }
    );
    status_method_with!(
        drop_collection,
        pb::DropCollectionRequest,
        |service, request| {
            let database = database_name(request.db_name);
            let key = (database, request.collection_name);
            let mut state = service.state.lock().unwrap();
            state.collections.remove(&key);
            state.loaded_collections.remove(&key);
            state.partitions.remove(&key);
            success_status()
        }
    );
    status_method_with!(
        load_collection,
        pb::LoadCollectionRequest,
        |service, request| {
            let database = database_name(request.db_name);
            service
                .state
                .lock()
                .unwrap()
                .loaded_collections
                .insert((database, request.collection_name));
            success_status()
        }
    );
    status_method_with!(
        release_collection,
        pb::ReleaseCollectionRequest,
        |service, request| {
            let database = database_name(request.db_name);
            service
                .state
                .lock()
                .unwrap()
                .loaded_collections
                .remove(&(database, request.collection_name));
            success_status()
        }
    );
    status_method_with!(
        alter_collection,
        pb::AlterCollectionRequest,
        |service, request| {
            let database = database_name(request.db_name.clone());
            if let Some(description) = service
                .state
                .lock()
                .unwrap()
                .collections
                .get_mut(&(database, request.collection_name.clone()))
            {
                let update = |values: &mut Vec<common::KeyValuePair>| {
                    for property in &request.properties {
                        if let Some(current) =
                            values.iter_mut().find(|item| item.key == property.key)
                        {
                            current.value = property.value.clone();
                        } else {
                            values.push(property.clone());
                        }
                    }
                    values.retain(|item| !request.delete_keys.contains(&item.key));
                };
                update(&mut description.properties);
                if let Some(schema) = &mut description.schema {
                    update(&mut schema.properties);
                }
                description.update_timestamp = description.update_timestamp.saturating_add(1);
            }
            success_status()
        }
    );
    status_method!(alter_collection_field, pb::AlterCollectionFieldRequest);
    status_method!(add_collection_field, pb::AddCollectionFieldRequest);
    status_method!(add_collection_function, pb::AddCollectionFunctionRequest);
    status_method!(
        alter_collection_function,
        pb::AlterCollectionFunctionRequest
    );
    status_method!(drop_collection_function, pb::DropCollectionFunctionRequest);
    status_method_with!(
        create_partition,
        pb::CreatePartitionRequest,
        |service, request| {
            let database = database_name(request.db_name);
            service
                .state
                .lock()
                .unwrap()
                .partitions
                .entry((database, request.collection_name))
                .or_default()
                .insert(request.partition_name, 1);
            success_status()
        }
    );
    status_method_with!(
        drop_partition,
        pb::DropPartitionRequest,
        |service, request| {
            let database = database_name(request.db_name);
            if let Some(partitions) = service
                .state
                .lock()
                .unwrap()
                .partitions
                .get_mut(&(database, request.collection_name))
            {
                partitions.remove(&request.partition_name);
            }
            success_status()
        }
    );
    status_method!(load_partitions, pb::LoadPartitionsRequest);
    status_method!(release_partitions, pb::ReleasePartitionsRequest);
    status_method_with!(create_alias, pb::CreateAliasRequest, |service, request| {
        let database = if request.db_name.is_empty() {
            "default".to_owned()
        } else {
            request.db_name
        };
        service
            .state
            .lock()
            .unwrap()
            .aliases
            .insert((database, request.alias), request.collection_name);
        success_status()
    });
    status_method_with!(drop_alias, pb::DropAliasRequest, |service, request| {
        let database = if request.db_name.is_empty() {
            "default".to_owned()
        } else {
            request.db_name
        };
        service
            .state
            .lock()
            .unwrap()
            .aliases
            .remove(&(database, request.alias));
        success_status()
    });
    status_method_with!(alter_alias, pb::AlterAliasRequest, |service, request| {
        let database = if request.db_name.is_empty() {
            "default".to_owned()
        } else {
            request.db_name
        };
        service
            .state
            .lock()
            .unwrap()
            .aliases
            .insert((database, request.alias), request.collection_name);
        success_status()
    });
    status_method_with!(create_index, pb::CreateIndexRequest, |service, request| {
        let database = database_name(request.db_name);
        let index_name = if request.index_name.is_empty() {
            format!("{}_idx", request.field_name)
        } else {
            request.index_name
        };
        let description = pb::IndexDescription {
            index_name: index_name.clone(),
            index_id: 10,
            field_name: request.field_name,
            params: request.extra_params,
            indexed_rows: 1,
            total_rows: 1,
            state: common::IndexState::Finished as i32,
            ..Default::default()
        };
        service
            .state
            .lock()
            .unwrap()
            .indexes
            .insert((database, request.collection_name, index_name), description);
        success_status()
    });
    status_method!(alter_index, pb::AlterIndexRequest);
    status_method_with!(drop_index, pb::DropIndexRequest, |service, request| {
        let database = database_name(request.db_name);
        service.state.lock().unwrap().indexes.remove(&(
            database,
            request.collection_name,
            request.index_name,
        ));
        success_status()
    });
    status_method_with!(
        create_credential,
        pb::CreateCredentialRequest,
        |service, request| {
            service
                .state
                .lock()
                .unwrap()
                .users
                .insert(request.username, request.description.unwrap_or_default());
            success_status()
        }
    );
    status_method_with!(
        update_credential,
        pb::UpdateCredentialRequest,
        |service, request| {
            if let Some(description) = request.description {
                service
                    .state
                    .lock()
                    .unwrap()
                    .users
                    .insert(request.username, description);
            }
            success_status()
        }
    );
    status_method_with!(
        delete_credential,
        pb::DeleteCredentialRequest,
        |service, request| {
            let mut state = service.state.lock().unwrap();
            state.users.remove(&request.username);
            state
                .user_roles
                .retain(|(username, _)| username != &request.username);
            success_status()
        }
    );
    status_method_with!(create_role, pb::CreateRoleRequest, |service, request| {
        let role = request.entity.unwrap_or_default();
        service
            .state
            .lock()
            .unwrap()
            .roles
            .insert(role.name, role.description);
        success_status()
    });
    status_method_with!(alter_role, pb::AlterRoleRequest, |service, request| {
        service
            .state
            .lock()
            .unwrap()
            .roles
            .insert(request.role_name, request.description);
        success_status()
    });
    status_method_with!(drop_role, pb::DropRoleRequest, |service, request| {
        let mut state = service.state.lock().unwrap();
        state.roles.remove(&request.role_name);
        state
            .user_roles
            .retain(|(_, role_name)| role_name != &request.role_name);
        state
            .grants
            .retain(|grant| grant.role.as_ref().map(|role| &role.name) != Some(&request.role_name));
        success_status()
    });
    status_method_with!(
        operate_user_role,
        pb::OperateUserRoleRequest,
        |service, request| {
            let key = (request.username, request.role_name);
            let mut state = service.state.lock().unwrap();
            if request.r#type == pb::OperateUserRoleType::AddUserToRole as i32 {
                state.user_roles.insert(key);
            } else {
                state.user_roles.remove(&key);
            }
            success_status()
        }
    );
    status_method_with!(
        operate_privilege_v2,
        pb::OperatePrivilegeV2Request,
        |service, request| {
            let role = request.role.unwrap_or_default();
            let grantor = request.grantor.unwrap_or_default();
            let entity = pb::GrantEntity {
                role: Some(role),
                object: Some(pb::ObjectEntity {
                    name: "Collection".into(),
                }),
                object_name: request.collection_name,
                grantor: Some(grantor),
                db_name: request.db_name,
            };
            let mut state = service.state.lock().unwrap();
            if request.r#type == pb::OperatePrivilegeType::Grant as i32 {
                state.grants.push(entity);
            } else {
                state.grants.retain(|grant| grant != &entity);
            }
            success_status()
        }
    );
    status_method_with!(
        create_privilege_group,
        pb::CreatePrivilegeGroupRequest,
        |service, request| {
            service
                .state
                .lock()
                .unwrap()
                .privilege_groups
                .entry(request.group_name)
                .or_default();
            success_status()
        }
    );
    status_method_with!(
        drop_privilege_group,
        pb::DropPrivilegeGroupRequest,
        |service, request| {
            service
                .state
                .lock()
                .unwrap()
                .privilege_groups
                .remove(&request.group_name);
            success_status()
        }
    );
    status_method_with!(
        operate_privilege_group,
        pb::OperatePrivilegeGroupRequest,
        |service, request| {
            let mut state = service.state.lock().unwrap();
            let privileges = state
                .privilege_groups
                .entry(request.group_name)
                .or_default();
            for privilege in request.privileges {
                if request.r#type == pb::OperatePrivilegeGroupType::AddPrivilegesToGroup as i32 {
                    privileges.insert(privilege.name);
                } else {
                    privileges.remove(&privilege.name);
                }
            }
            success_status()
        }
    );
    status_method_with!(
        create_resource_group,
        pb::CreateResourceGroupRequest,
        |service, request| {
            service.state.lock().unwrap().resource_groups.insert(
                request.resource_group.clone(),
                pb::ResourceGroup {
                    name: request.resource_group,
                    capacity: 2,
                    num_available_node: 1,
                    num_loaded_replica: HashMap::from([("books".into(), 1)]),
                    num_outgoing_node: HashMap::from([("default".into(), 1)]),
                    num_incoming_node: HashMap::from([("backup".into(), 1)]),
                    config: request.config,
                    nodes: vec![common::NodeInfo {
                        node_id: 8,
                        address: "127.0.0.1:21123".into(),
                        hostname: "query-node".into(),
                        ..Default::default()
                    }],
                },
            );
            success_status()
        }
    );
    status_method_with!(
        drop_resource_group,
        pb::DropResourceGroupRequest,
        |service, request| {
            service
                .state
                .lock()
                .unwrap()
                .resource_groups
                .remove(&request.resource_group);
            success_status()
        }
    );
    status_method_with!(
        update_resource_groups,
        pb::UpdateResourceGroupsRequest,
        |service, request| {
            let mut state = service.state.lock().unwrap();
            for (name, config) in request.resource_groups {
                state
                    .resource_groups
                    .entry(name.clone())
                    .or_insert_with(|| pb::ResourceGroup {
                        name,
                        ..Default::default()
                    })
                    .config = Some(config);
            }
            success_status()
        }
    );
    status_method!(transfer_node, pb::TransferNodeRequest);
    status_method!(transfer_replica, pb::TransferReplicaRequest);
    status_method_with!(
        rename_collection,
        pb::RenameCollectionRequest,
        |service, request| {
            let old_database = database_name(request.db_name);
            let new_database = if request.new_db_name.is_empty() {
                old_database.clone()
            } else {
                request.new_db_name
            };
            let mut state = service.state.lock().unwrap();
            if let Some(mut description) = state
                .collections
                .remove(&(old_database.clone(), request.old_name.clone()))
            {
                description.db_name = new_database.clone();
                description.collection_name = request.new_name.clone();
                state.collections.insert(
                    (new_database.clone(), request.new_name.clone()),
                    description,
                );
            }
            if state
                .loaded_collections
                .remove(&(old_database, request.old_name))
            {
                state
                    .loaded_collections
                    .insert((new_database, request.new_name));
            }
            success_status()
        }
    );
    status_method_with!(
        create_database,
        pb::CreateDatabaseRequest,
        |service, request| {
            service.state.lock().unwrap().databases.insert(
                request.db_name,
                request
                    .properties
                    .into_iter()
                    .map(|pair| (pair.key, pair.value))
                    .collect(),
            );
            success_status()
        }
    );
    status_method_with!(
        drop_database,
        pb::DropDatabaseRequest,
        |service, request| {
            service
                .state
                .lock()
                .unwrap()
                .databases
                .remove(&request.db_name);
            success_status()
        }
    );
    status_method_with!(
        alter_database,
        pb::AlterDatabaseRequest,
        |service, request| {
            let mut state = service.state.lock().unwrap();
            let properties = state.databases.entry(request.db_name).or_default();
            for pair in request.properties {
                properties.insert(pair.key, pair.value);
            }
            for key in request.delete_keys {
                properties.remove(&key);
            }
            success_status()
        }
    );
    status_method_with!(
        add_file_resource,
        pb::AddFileResourceRequest,
        |service, request| {
            service.state.lock().unwrap().file_resources.insert(
                request.name.clone(),
                pb::FileResourceInfo {
                    id: 30,
                    name: request.name,
                    path: request.path,
                },
            );
            success_status()
        }
    );
    status_method_with!(
        remove_file_resource,
        pb::RemoveFileResourceRequest,
        |service, request| {
            service
                .state
                .lock()
                .unwrap()
                .file_resources
                .remove(&request.name);
            success_status()
        }
    );
    status_method_with!(
        update_replicate_configuration,
        pb::UpdateReplicateConfigurationRequest,
        |service, request| {
            service.state.lock().unwrap().replicate_configuration = request.replicate_configuration;
            success_status()
        }
    );
    response_method!(
        refresh_external_collection,
        pb::RefreshExternalCollectionRequest,
        pb::RefreshExternalCollectionResponse,
        pb::RefreshExternalCollectionResponse {
            status: Some(success_status()),
            job_id: 7,
        }
    );
    response_method!(
        get_refresh_external_collection_progress,
        pb::GetRefreshExternalCollectionProgressRequest,
        pb::GetRefreshExternalCollectionProgressResponse,
        pb::GetRefreshExternalCollectionProgressResponse {
            status: Some(success_status()),
            job_info: Some(pb::RefreshExternalCollectionJobInfo {
                job_id: 7,
                collection_name: "books".into(),
                state: pb::RefreshExternalCollectionState::RefreshCompleted as i32,
                progress: 100,
                reason: String::new(),
                external_source: "s3://bucket/path".into(),
                start_time: 100,
                end_time: 200,
                external_spec: String::new(),
            }),
        }
    );
    response_method!(
        list_refresh_external_collection_jobs,
        pb::ListRefreshExternalCollectionJobsRequest,
        pb::ListRefreshExternalCollectionJobsResponse,
        pb::ListRefreshExternalCollectionJobsResponse {
            status: Some(success_status()),
            jobs: vec![pb::RefreshExternalCollectionJobInfo {
                job_id: 7,
                collection_name: "books".into(),
                state: pb::RefreshExternalCollectionState::RefreshInProgress as i32,
                progress: 50,
                reason: String::new(),
                external_source: "s3://bucket/path".into(),
                start_time: 100,
                end_time: 0,
                external_spec: String::new(),
            }],
        }
    );

    response_method_with!(
        has_collection,
        pb::HasCollectionRequest,
        pb::BoolResponse,
        |service, request| {
            let database = database_name(request.db_name);
            pb::BoolResponse {
                status: Some(success_status()),
                value: service
                    .state
                    .lock()
                    .unwrap()
                    .collections
                    .contains_key(&(database, request.collection_name)),
            }
        }
    );
    response_method_with!(
        describe_collection,
        pb::DescribeCollectionRequest,
        pb::DescribeCollectionResponse,
        |service, request| {
            let database = database_name(request.db_name);
            let state = service.state.lock().unwrap();
            let collection = state
                .aliases
                .get(&(database.clone(), request.collection_name.clone()))
                .cloned()
                .unwrap_or(request.collection_name);
            state
                .collections
                .get(&(database, collection))
                .cloned()
                .unwrap_or_else(|| pb::DescribeCollectionResponse {
                    status: Some(success_status()),
                    ..Default::default()
                })
        }
    );
    response_method_with!(
        batch_describe_collection,
        pb::BatchDescribeCollectionRequest,
        pb::BatchDescribeCollectionResponse,
        |service, request| {
            let database = database_name(request.db_name);
            let state = service.state.lock().unwrap();
            let responses = request
                .collection_name
                .into_iter()
                .filter_map(|name| state.collections.get(&(database.clone(), name)).cloned())
                .chain(request.collection_id.into_iter().filter_map(|id| {
                    state
                        .collections
                        .values()
                        .find(|description| {
                            description.db_name == database && description.collection_id == id
                        })
                        .cloned()
                }))
                .collect();
            pb::BatchDescribeCollectionResponse {
                status: Some(success_status()),
                responses,
            }
        }
    );
    response_method!(
        get_collection_statistics,
        pb::GetCollectionStatisticsRequest,
        pb::GetCollectionStatisticsResponse,
        pb::GetCollectionStatisticsResponse {
            status: Some(success_status()),
            stats: vec![common::KeyValuePair {
                key: "row_count".into(),
                value: "1".into(),
                ..Default::default()
            }],
        }
    );
    response_method_with!(
        show_collections,
        pb::ShowCollectionsRequest,
        pb::ShowCollectionsResponse,
        |service, request| {
            let database = database_name(request.db_name);
            let state = service.state.lock().unwrap();
            let mut descriptions: Vec<_> = state
                .collections
                .values()
                .filter(|description| description.db_name == database)
                .cloned()
                .collect();
            descriptions.sort_by(|left, right| left.collection_name.cmp(&right.collection_name));
            pb::ShowCollectionsResponse {
                status: Some(success_status()),
                collection_names: descriptions
                    .iter()
                    .map(|description| description.collection_name.clone())
                    .collect(),
                collection_ids: descriptions
                    .iter()
                    .map(|description| description.collection_id)
                    .collect(),
                created_timestamps: descriptions
                    .iter()
                    .map(|description| description.created_timestamp)
                    .collect(),
                created_utc_timestamps: descriptions
                    .iter()
                    .map(|description| description.created_utc_timestamp)
                    .collect(),
                query_service_available: descriptions
                    .iter()
                    .map(|description| {
                        state.loaded_collections.contains(&(
                            description.db_name.clone(),
                            description.collection_name.clone(),
                        ))
                    })
                    .collect(),
                shards_num: descriptions
                    .iter()
                    .map(|description| description.shards_num)
                    .collect(),
                ..Default::default()
            }
        }
    );
    status_response_method!(
        truncate_collection,
        pb::TruncateCollectionRequest,
        pb::TruncateCollectionResponse
    );
    response_method_with!(
        has_partition,
        pb::HasPartitionRequest,
        pb::BoolResponse,
        |service, request| {
            let database = database_name(request.db_name);
            pb::BoolResponse {
                status: Some(success_status()),
                value: service
                    .state
                    .lock()
                    .unwrap()
                    .partitions
                    .get(&(database, request.collection_name))
                    .is_some_and(|partitions| partitions.contains_key(&request.partition_name)),
            }
        }
    );
    response_method_with!(
        show_partitions,
        pb::ShowPartitionsRequest,
        pb::ShowPartitionsResponse,
        |service, request| {
            let database = database_name(request.db_name);
            let state = service.state.lock().unwrap();
            let mut partitions: Vec<_> = state
                .partitions
                .get(&(database, request.collection_name))
                .into_iter()
                .flat_map(|partitions| partitions.iter())
                .map(|(name, id)| (name.clone(), *id))
                .collect();
            partitions.sort();
            pb::ShowPartitionsResponse {
                status: Some(success_status()),
                partition_names: partitions.iter().map(|(name, _)| name.clone()).collect(),
                partition_i_ds: partitions.iter().map(|(_, id)| *id).collect(),
                created_timestamps: vec![201; partitions.len()],
                created_utc_timestamps: vec![202; partitions.len()],
                ..Default::default()
            }
        }
    );
    response_method!(
        get_partition_statistics,
        pb::GetPartitionStatisticsRequest,
        pb::GetPartitionStatisticsResponse,
        pb::GetPartitionStatisticsResponse {
            status: Some(success_status()),
            stats: vec![common::KeyValuePair {
                key: "row_count".into(),
                value: "1".into(),
                ..Default::default()
            }],
        }
    );
    response_method_with!(
        get_loading_progress,
        pb::GetLoadingProgressRequest,
        pb::GetLoadingProgressResponse,
        |_service, request| {
            if request.collection_name == "stalled_loading_progress" {
                std::future::pending::<()>().await;
            }
            pb::GetLoadingProgressResponse {
                status: Some(success_status()),
                progress: 100,
                refresh_progress: 100,
            }
        }
    );
    response_method_with!(
        get_load_state,
        pb::GetLoadStateRequest,
        pb::GetLoadStateResponse,
        |service, request| {
            let database = database_name(request.db_name);
            let loaded = service
                .state
                .lock()
                .unwrap()
                .loaded_collections
                .contains(&(database, request.collection_name));
            pb::GetLoadStateResponse {
                status: Some(success_status()),
                state: if loaded {
                    common::LoadState::Loaded
                } else {
                    common::LoadState::NotLoad
                } as i32,
            }
        }
    );
    response_method_with!(
        describe_alias,
        pb::DescribeAliasRequest,
        pb::DescribeAliasResponse,
        |service, request| {
            let database = if request.db_name.is_empty() {
                "default".to_owned()
            } else {
                request.db_name
            };
            let collection = service
                .state
                .lock()
                .unwrap()
                .aliases
                .get(&(database.clone(), request.alias.clone()))
                .cloned()
                .unwrap_or_default();
            pb::DescribeAliasResponse {
                status: Some(success_status()),
                db_name: database,
                alias: request.alias,
                collection,
            }
        }
    );
    response_method_with!(
        list_aliases,
        pb::ListAliasesRequest,
        pb::ListAliasesResponse,
        |service, request| {
            let database = if request.db_name.is_empty() {
                "default".to_owned()
            } else {
                request.db_name
            };
            let mut aliases: Vec<String> = service
                .state
                .lock()
                .unwrap()
                .aliases
                .iter()
                .filter_map(|((db, alias), collection)| {
                    (db == &database && collection == &request.collection_name)
                        .then_some(alias.clone())
                })
                .collect();
            aliases.sort();
            pb::ListAliasesResponse {
                status: Some(success_status()),
                db_name: database,
                collection_name: request.collection_name,
                aliases,
            }
        }
    );
    response_method_with!(
        describe_index,
        pb::DescribeIndexRequest,
        pb::DescribeIndexResponse,
        |service, request| {
            if request.collection_name == "stalled_index_poll" {
                std::future::pending::<()>().await;
            }
            let first_delayed_poll = request.collection_name == "delayed_index_progress"
                && service.call_count("describe_index") == 1;
            let database = database_name(request.db_name);
            let state = service.state.lock().unwrap();
            let mut indexes: Vec<_> = state
                .indexes
                .iter()
                .filter_map(|((db, collection, name), description)| {
                    (db == &database
                        && collection == &request.collection_name
                        && (request.index_name.is_empty() || name == &request.index_name)
                        && (request.field_name.is_empty()
                            || description.field_name == request.field_name))
                        .then_some(description.clone())
                })
                .collect();
            if first_delayed_poll {
                for index in &mut indexes {
                    index.state = common::IndexState::InProgress as i32;
                }
            }
            pb::DescribeIndexResponse {
                status: Some(success_status()),
                index_descriptions: indexes,
            }
        }
    );
    response_method_with!(
        get_index_statistics,
        pb::GetIndexStatisticsRequest,
        pb::GetIndexStatisticsResponse,
        |service, request| {
            let database = database_name(request.db_name);
            let state = service.state.lock().unwrap();
            let indexes = state
                .indexes
                .iter()
                .filter_map(|((db, collection, name), description)| {
                    (db == &database
                        && collection == &request.collection_name
                        && (request.index_name.is_empty() || name == &request.index_name))
                        .then_some(description.clone())
                })
                .collect();
            pb::GetIndexStatisticsResponse {
                status: Some(success_status()),
                index_descriptions: indexes,
            }
        }
    );
    response_method!(
        insert,
        pb::InsertRequest,
        pb::MutationResult,
        pb::MutationResult {
            status: Some(success_status()),
            i_ds: Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                    data: vec![1],
                })),
                ..Default::default()
            }),
            succ_index: vec![0],
            acknowledged: true,
            insert_cnt: 1,
            timestamp: 10,
            ..Default::default()
        }
    );
    response_method!(
        upsert,
        pb::UpsertRequest,
        pb::MutationResult,
        pb::MutationResult {
            status: Some(success_status()),
            i_ds: Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                    data: vec![1],
                })),
                ..Default::default()
            }),
            succ_index: vec![0],
            acknowledged: true,
            upsert_cnt: 1,
            timestamp: 11,
            ..Default::default()
        }
    );
    response_method!(
        delete,
        pb::DeleteRequest,
        pb::MutationResult,
        pb::MutationResult {
            status: Some(success_status()),
            i_ds: Some(schema::IDs {
                id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                    data: vec![1],
                })),
                ..Default::default()
            }),
            succ_index: vec![0],
            acknowledged: true,
            delete_cnt: 1,
            timestamp: 12,
            ..Default::default()
        }
    );
    response_method_with!(
        query,
        pb::QueryRequest,
        pb::QueryResults,
        |_service, request| {
            if request.expr.contains("over_return_query_iterator") {
                let start = request
                    .expr
                    .split_once("id > ")
                    .and_then(|(_, value)| value.split_whitespace().next())
                    .and_then(|value| value.parse::<i64>().ok())
                    .map_or(0, |value| value + 1);
                let end = if request.output_fields.is_empty() {
                    (start + 1).min(6)
                } else {
                    6
                };
                pb::QueryResults {
                    status: Some(success_status()),
                    fields_data: vec![int64_field("id", (start..end).collect())],
                    collection_name: "books".into(),
                    output_fields: vec!["id".into()],
                    session_ts: 300,
                    primary_field_name: "id".into(),
                    ..Default::default()
                }
            } else if request.expr.contains("unlimited_query_iterator") {
                let start = request
                    .expr
                    .split_once("id > ")
                    .and_then(|(_, value)| value.split_whitespace().next())
                    .and_then(|value| value.parse::<i64>().ok())
                    .map_or(0, |value| value + 1);
                let end = (start + 1).min(3);
                pb::QueryResults {
                    status: Some(success_status()),
                    fields_data: vec![int64_field("id", (start..end).collect())],
                    collection_name: "books".into(),
                    output_fields: vec!["id".into()],
                    session_ts: 300,
                    primary_field_name: "id".into(),
                    ..Default::default()
                }
            } else if request.expr.contains("element_filter_query_iterator") {
                let start = request
                    .expr
                    .split_once("id > ")
                    .and_then(|(_, value)| value.split_whitespace().next())
                    .and_then(|value| value.parse::<i64>().ok())
                    .map_or(0, |value| value + 1);
                let end = (start + 1).min(3);
                pb::QueryResults {
                    status: Some(success_status()),
                    fields_data: vec![int64_field("id", (start..end).collect())],
                    collection_name: "books".into(),
                    output_fields: vec!["id".into()],
                    session_ts: 300,
                    primary_field_name: "id".into(),
                    ..Default::default()
                }
            } else if request.expr.contains("decode_failure_query_iterator")
                && !request.output_fields.is_empty()
            {
                pb::QueryResults {
                    status: Some(success_status()),
                    fields_data: vec![
                        int64_field("id", vec![1]),
                        schema::FieldData {
                            r#type: schema::DataType::Json as i32,
                            field_name: "invalid_json".into(),
                            field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                                valid_data: Vec::new(),
                                data: Some(schema::scalar_field::Data::JsonData(
                                    schema::JsonArray {
                                        data: vec![b"not-json".to_vec()],
                                    },
                                )),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    ],
                    collection_name: "books".into(),
                    output_fields: vec!["id".into(), "invalid_json".into()],
                    session_ts: 300,
                    primary_field_name: "id".into(),
                    ..Default::default()
                }
            } else if request.expr.contains("large_query_iterator") {
                let limit = request
                    .query_params
                    .iter()
                    .find(|param| param.key == "limit")
                    .and_then(|param| param.value.parse::<i64>().ok())
                    .unwrap_or(1)
                    .max(0);
                let start = request
                    .expr
                    .split_once("id > ")
                    .and_then(|(_, value)| value.split_whitespace().next())
                    .and_then(|value| value.parse::<i64>().ok())
                    .map_or(0, |value| value + 1);
                let end = (start + limit).min(17_000);
                pb::QueryResults {
                    status: Some(success_status()),
                    fields_data: vec![int64_field("id", (start..end).collect())],
                    collection_name: "books".into(),
                    output_fields: vec!["id".into()],
                    session_ts: 300,
                    primary_field_name: "id".into(),
                    ..Default::default()
                }
            } else {
                pb::QueryResults {
                    status: Some(success_status()),
                    fields_data: vec![
                        int64_field("id", vec![1]),
                        string_field("text", vec!["book"]),
                    ],
                    collection_name: "books".into(),
                    output_fields: vec!["id".into(), "text".into()],
                    session_ts: if request.expr == "zero_session_ts" {
                        0
                    } else {
                        300
                    },
                    primary_field_name: "id".into(),
                    ..Default::default()
                }
            }
        }
    );
    response_method_with!(
        search,
        pb::SearchRequest,
        pb::SearchResults,
        |service, request| {
            let mut response = search_response();
            if request.dsl.contains("legacy_hamming_gap_iterator") {
                let radius = request
                    .search_params
                    .iter()
                    .find(|param| param.key == "radius")
                    .and_then(|param| param.value.parse::<f64>().ok());
                let (ids, scores) = match radius {
                    None => (vec![1], vec![0.0]),
                    Some(radius) if radius >= 2.0 => (vec![2], vec![2.0]),
                    Some(_) => (Vec::new(), Vec::new()),
                };
                if let Some(results) = response.results.as_mut() {
                    results.top_k = ids.len() as i64;
                    results.topks = vec![ids.len() as i64];
                    results.scores = scores;
                    results.ids = Some(schema::IDs {
                        id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                            data: ids,
                        })),
                        ..Default::default()
                    });
                    results.fields_data = vec![string_field(
                        "text",
                        if radius.is_some() && results.top_k == 0 {
                            Vec::<&str>::new()
                        } else {
                            vec!["book"]
                        },
                    )];
                    results.search_iterator_v2_results = None;
                }
            } else if request.dsl.contains("legacy_search_iterator") {
                let is_range_page = request
                    .search_params
                    .iter()
                    .any(|param| param.key == "range_filter");
                let (ids, scores) = if is_range_page {
                    (vec![3, 4], vec![0.7, 0.6])
                } else {
                    (vec![1, 2], vec![0.9, 0.8])
                };
                if let Some(results) = response.results.as_mut() {
                    results.top_k = ids.len() as i64;
                    results.topks = vec![ids.len() as i64];
                    results.scores = scores;
                    results.ids = Some(schema::IDs {
                        id_field: Some(schema::i_ds::IdField::IntId(schema::LongArray {
                            data: ids,
                        })),
                        ..Default::default()
                    });
                    results.fields_data = vec![string_field("text", vec!["book", "book"])];
                    results.search_iterator_v2_results = None;
                }
            }
            if request.dsl == "zero_session_ts" {
                response.session_ts = 0;
            }
            if request.dsl == "decode_failure_search_iterator" && service.call_count("search") == 2
            {
                if let Some(results) = response.results.as_mut() {
                    results.fields_data = vec![schema::FieldData {
                        r#type: schema::DataType::Json as i32,
                        field_name: "invalid_json".into(),
                        field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
                            valid_data: Vec::new(),
                            data: Some(schema::scalar_field::Data::JsonData(schema::JsonArray {
                                data: vec![b"not-json".to_vec()],
                            })),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }];
                    results.output_fields = vec!["invalid_json".into()];
                }
            }
            if request
                .search_params
                .iter()
                .any(|param| param.key == "search_iter_id")
            {
                if let Some(results) = response.results.as_mut() {
                    results.top_k = 0;
                    results.topks = vec![0];
                    results.scores.clear();
                    results.ids = Some(schema::IDs::default());
                    results.fields_data = vec![schema::FieldData {
                        r#type: schema::DataType::VarChar as i32,
                        field_name: "text".into(),
                        ..Default::default()
                    }];
                }
            }
            if request.collection_name == "missing_primary_field" {
                if let Some(results) = response.results.as_mut() {
                    results.primary_field_name.clear();
                }
            }
            response
        }
    );
    response_method_with!(
        hybrid_search,
        pb::HybridSearchRequest,
        pb::SearchResults,
        |_service, request| {
            let mut response = search_response();
            if request.collection_name == "missing_primary_field" {
                if let Some(results) = response.results.as_mut() {
                    results.primary_field_name.clear();
                }
            }
            response
        }
    );
    response_method_with!(
        flush,
        pb::FlushRequest,
        pb::FlushResponse,
        |_service, request| {
            let collection = request
                .collection_names
                .first()
                .cloned()
                .unwrap_or_default();
            let segment_id = match collection.as_str() {
                "stalled_flush_state" => Some(900),
                "short_flush_visibility_delay" => Some(901),
                "missing_flush_timestamp" => Some(902),
                _ => None,
            };
            pb::FlushResponse {
                status: Some(success_status()),
                db_name: database_name(request.db_name),
                coll_seg_i_ds: segment_id
                    .map(|id| {
                        HashMap::from([(
                            collection.clone(),
                            schema::LongArray {
                                data: vec![id],
                                ..Default::default()
                            },
                        )])
                    })
                    .unwrap_or_default(),
                coll_flush_ts: (collection != "missing_flush_timestamp")
                    .then(|| HashMap::from([(collection, 400)]))
                    .unwrap_or_default(),
                ..Default::default()
            }
        }
    );
    response_method_with!(
        flush_all,
        pb::FlushAllRequest,
        pb::FlushAllResponse,
        |_service, request| {
            pb::FlushAllResponse {
                status: Some(success_status()),
                flush_all_ts: if request.db_name == "stalled_flush_all_state" {
                    902
                } else {
                    401
                },
                ..Default::default()
            }
        }
    );
    response_method_with!(
        get_flush_state,
        pb::GetFlushStateRequest,
        pb::GetFlushStateResponse,
        |_service, request| {
            if request.segment_i_ds.contains(&900) {
                std::future::pending::<()>().await;
            }
            pb::GetFlushStateResponse {
                status: Some(success_status()),
                flushed: true,
            }
        }
    );
    response_method_with!(
        get_flush_all_state,
        pb::GetFlushAllStateRequest,
        pb::GetFlushAllStateResponse,
        |_service, request| {
            if request.flush_all_ts == 902 {
                std::future::pending::<()>().await;
            }
            pb::GetFlushAllStateResponse {
                status: Some(success_status()),
                flushed: true,
                flush_states: Vec::new(),
            }
        }
    );
    response_method!(
        get_persistent_segment_info,
        pb::GetPersistentSegmentInfoRequest,
        pb::GetPersistentSegmentInfoResponse,
        pb::GetPersistentSegmentInfoResponse {
            status: Some(success_status()),
            infos: vec![pb::PersistentSegmentInfo {
                segment_id: 40,
                collection_id: 1,
                partition_id: 1,
                num_rows: 1,
                state: common::SegmentState::Flushed as i32,
                level: common::SegmentLevel::L1 as i32,
                is_sorted: true,
                storage_version: 2,
            }],
        }
    );
    response_method!(
        get_query_segment_info,
        pb::GetQuerySegmentInfoRequest,
        pb::GetQuerySegmentInfoResponse,
        pb::GetQuerySegmentInfoResponse {
            status: Some(success_status()),
            infos: vec![pb::QuerySegmentInfo {
                segment_id: 41,
                collection_id: 1,
                partition_id: 1,
                mem_size: 1024,
                num_rows: 1,
                index_name: "vector_idx".into(),
                index_id: 10,
                node_ids: vec![8, 9],
                state: common::SegmentState::Flushed as i32,
                level: common::SegmentLevel::L1 as i32,
                is_sorted: true,
                storage_version: 2,
                ..Default::default()
            }],
        }
    );
    response_method!(
        get_replicas,
        pb::GetReplicasRequest,
        pb::GetReplicasResponse,
        pb::GetReplicasResponse {
            status: Some(success_status()),
            replicas: vec![pb::ReplicaInfo {
                replica_id: 7,
                collection_id: 1,
                partition_ids: vec![1],
                shard_replicas: vec![pb::ShardReplica {
                    leader_id: 8,
                    leader_addr: "127.0.0.1:21123".into(),
                    dm_channel_name: "channel-1".into(),
                    node_ids: vec![8, 9],
                }],
                node_ids: vec![8, 9],
                resource_group_name: "default".into(),
                num_outbound_node: HashMap::from([("backup".into(), 1)]),
            }],
        }
    );
    response_method_with!(
        manual_compaction,
        pb::ManualCompactionRequest,
        pb::ManualCompactionResponse,
        |_service, request| {
            if request.target_size == 2 {
                pb::ManualCompactionResponse {
                    status: Some(success_status()),
                    compaction_id: -1,
                    compaction_plan_count: 0,
                    ..Default::default()
                }
            } else {
                pb::ManualCompactionResponse {
                    status: Some(success_status()),
                    compaction_id: 1,
                    compaction_plan_count: 1,
                    ..Default::default()
                }
            }
        }
    );
    response_method!(
        get_compaction_state,
        pb::GetCompactionStateRequest,
        pb::GetCompactionStateResponse,
        pb::GetCompactionStateResponse {
            status: Some(success_status()),
            state: common::CompactionState::Completed as i32,
            completed_plan_no: 1,
            ..Default::default()
        }
    );
    response_method!(
        get_compaction_state_with_plans,
        pb::GetCompactionPlansRequest,
        pb::GetCompactionPlansResponse,
        pb::GetCompactionPlansResponse {
            status: Some(success_status()),
            state: common::CompactionState::Completed as i32,
            merge_infos: vec![pb::CompactionMergeInfo {
                sources: vec![1, 2],
                target: 3,
            }],
        }
    );
    response_method_with!(
        list_cred_users,
        pb::ListCredUsersRequest,
        pb::ListCredUsersResponse,
        |service, _request| {
            let mut usernames: Vec<_> = service
                .state
                .lock()
                .unwrap()
                .users
                .keys()
                .cloned()
                .collect();
            usernames.sort();
            pb::ListCredUsersResponse {
                status: Some(success_status()),
                usernames,
            }
        }
    );
    response_method_with!(
        select_role,
        pb::SelectRoleRequest,
        pb::SelectRoleResponse,
        |service, request| {
            let requested = request.role.map(|role| role.name);
            let state = service.state.lock().unwrap();
            let mut results: Vec<_> = state
                .roles
                .iter()
                .filter(|(name, _)| {
                    requested
                        .as_ref()
                        .map_or(true, |requested| requested == *name)
                })
                .map(|(name, description)| pb::RoleResult {
                    role: Some(pb::RoleEntity {
                        name: name.clone(),
                        description: description.clone(),
                    }),
                    users: state
                        .user_roles
                        .iter()
                        .filter_map(|(username, role_name)| {
                            (role_name == name).then_some(pb::UserEntity {
                                name: username.clone(),
                            })
                        })
                        .collect(),
                })
                .collect();
            results.sort_by(|left, right| {
                left.role
                    .as_ref()
                    .map(|role| &role.name)
                    .cmp(&right.role.as_ref().map(|role| &role.name))
            });
            pb::SelectRoleResponse {
                status: Some(success_status()),
                results,
            }
        }
    );
    response_method_with!(
        select_user,
        pb::SelectUserRequest,
        pb::SelectUserResponse,
        |service, request| {
            let requested = request.user.map(|user| user.name);
            let state = service.state.lock().unwrap();
            let mut results: Vec<_> = state
                .users
                .iter()
                .filter(|(name, _)| {
                    requested
                        .as_ref()
                        .map_or(true, |requested| requested == *name)
                })
                .map(|(name, description)| pb::UserResult {
                    user: Some(pb::UserEntity { name: name.clone() }),
                    roles: state
                        .user_roles
                        .iter()
                        .filter_map(|(username, role_name)| {
                            (username == name).then(|| pb::RoleEntity {
                                name: role_name.clone(),
                                description: state
                                    .roles
                                    .get(role_name)
                                    .cloned()
                                    .unwrap_or_default(),
                            })
                        })
                        .collect(),
                    description: description.clone(),
                })
                .collect();
            results.sort_by(|left, right| {
                left.user
                    .as_ref()
                    .map(|user| &user.name)
                    .cmp(&right.user.as_ref().map(|user| &user.name))
            });
            pb::SelectUserResponse {
                status: Some(success_status()),
                results,
            }
        }
    );
    response_method_with!(
        select_grant,
        pb::SelectGrantRequest,
        pb::SelectGrantResponse,
        |service, request| {
            let requested_role = request
                .entity
                .and_then(|entity| entity.role)
                .map(|role| role.name);
            let entities = service
                .state
                .lock()
                .unwrap()
                .grants
                .iter()
                .filter(|grant| {
                    requested_role.as_ref().map_or(true, |requested| {
                        grant.role.as_ref().map(|role| &role.name) == Some(requested)
                    })
                })
                .cloned()
                .collect();
            pb::SelectGrantResponse {
                status: Some(success_status()),
                entities,
            }
        }
    );
    response_method!(
        get_version,
        pb::GetVersionRequest,
        pb::GetVersionResponse,
        pb::GetVersionResponse {
            status: Some(success_status()),
            version: "mock-2.6".into(),
        }
    );
    response_method!(
        check_health,
        pb::CheckHealthRequest,
        pb::CheckHealthResponse,
        pb::CheckHealthResponse {
            status: Some(success_status()),
            is_healthy: true,
            reasons: vec![],
            quota_states: vec![pb::QuotaState::ReadLimited as i32],
            ..Default::default()
        }
    );
    response_method_with!(
        list_resource_groups,
        pb::ListResourceGroupsRequest,
        pb::ListResourceGroupsResponse,
        |service, _request| {
            let mut groups: Vec<String> = service
                .state
                .lock()
                .unwrap()
                .resource_groups
                .keys()
                .cloned()
                .collect();
            groups.sort();
            pb::ListResourceGroupsResponse {
                status: Some(success_status()),
                resource_groups: groups,
            }
        }
    );
    response_method_with!(
        describe_resource_group,
        pb::DescribeResourceGroupRequest,
        pb::DescribeResourceGroupResponse,
        |service, request| {
            pb::DescribeResourceGroupResponse {
                status: Some(success_status()),
                resource_group: service
                    .state
                    .lock()
                    .unwrap()
                    .resource_groups
                    .get(&request.resource_group)
                    .cloned(),
            }
        }
    );
    response_method!(
        alloc_timestamp,
        pb::AllocTimestampRequest,
        pb::AllocTimestampResponse,
        pb::AllocTimestampResponse {
            status: Some(success_status()),
            timestamp: 100,
        }
    );
    response_method_with!(
        list_databases,
        pb::ListDatabasesRequest,
        pb::ListDatabasesResponse,
        |service, _request| {
            let mut names = vec!["default".to_owned()];
            names.extend(service.state.lock().unwrap().databases.keys().cloned());
            names.sort();
            let count = names.len();
            pb::ListDatabasesResponse {
                status: Some(success_status()),
                db_names: names,
                created_timestamp: (0..count).map(|index| 200 + index as u64).collect(),
                db_ids: (0..count).map(|index| 20 + index as i64).collect(),
            }
        }
    );
    response_method_with!(
        describe_database,
        pb::DescribeDatabaseRequest,
        pb::DescribeDatabaseResponse,
        |service, request| {
            let properties = service
                .state
                .lock()
                .unwrap()
                .databases
                .get(&request.db_name)
                .cloned()
                .unwrap_or_default();
            pb::DescribeDatabaseResponse {
                status: Some(success_status()),
                db_name: request.db_name,
                db_id: 20,
                created_timestamp: 200,
                properties: properties
                    .into_iter()
                    .map(|(key, value)| common::KeyValuePair {
                        key,
                        value,
                        ..Default::default()
                    })
                    .collect(),
            }
        }
    );
    response_method_with!(
        list_privilege_groups,
        pb::ListPrivilegeGroupsRequest,
        pb::ListPrivilegeGroupsResponse,
        |service, _request| {
            let state = service.state.lock().unwrap();
            let mut groups: Vec<_> = state
                .privilege_groups
                .iter()
                .map(|(name, privileges)| {
                    let mut privileges: Vec<_> = privileges
                        .iter()
                        .map(|name| pb::PrivilegeEntity { name: name.clone() })
                        .collect();
                    privileges.sort_by(|left, right| left.name.cmp(&right.name));
                    pb::PrivilegeGroupInfo {
                        group_name: name.clone(),
                        privileges,
                    }
                })
                .collect();
            groups.sort_by(|left, right| left.group_name.cmp(&right.group_name));
            pb::ListPrivilegeGroupsResponse {
                status: Some(success_status()),
                privilege_groups: groups,
            }
        }
    );
    response_method!(
        run_analyzer,
        pb::RunAnalyzerRequest,
        pb::RunAnalyzerResponse,
        pb::RunAnalyzerResponse {
            status: Some(success_status()),
            results: vec![pb::AnalyzerResult {
                tokens: vec![pb::AnalyzerToken {
                    token: "hello".into(),
                    start_offset: 0,
                    end_offset: 5,
                    position: 0,
                    position_length: 1,
                    hash: 123,
                }],
            }],
        }
    );
    response_method_with!(
        list_file_resources,
        pb::ListFileResourcesRequest,
        pb::ListFileResourcesResponse,
        |service, _request| {
            let mut resources: Vec<_> = service
                .state
                .lock()
                .unwrap()
                .file_resources
                .values()
                .cloned()
                .collect();
            resources.sort_by(|left, right| left.name.cmp(&right.name));
            pb::ListFileResourcesResponse {
                status: Some(success_status()),
                resources,
            }
        }
    );
    response_method_with!(
        get_replicate_configuration,
        pb::GetReplicateConfigurationRequest,
        pb::GetReplicateConfigurationResponse,
        |service, _request| {
            pb::GetReplicateConfigurationResponse {
                status: Some(success_status()),
                configuration: service
                    .state
                    .lock()
                    .unwrap()
                    .replicate_configuration
                    .clone(),
            }
        }
    );
    response_method_with!(
        get_replicate_info,
        pb::GetReplicateInfoRequest,
        pb::GetReplicateInfoResponse,
        |_service, request| {
            let checkpoint = common::ReplicateCheckpoint {
                cluster_id: request.source_cluster_id,
                pchannel: request.target_pchannel,
                message_id: Some(common::MessageId {
                    id: "message-1".into(),
                    wal_name: common::WalName::Pulsar as i32,
                }),
                time_tick: 500,
                ..Default::default()
            };
            pb::GetReplicateInfoResponse {
                checkpoint: Some(checkpoint.clone()),
                salvage_checkpoint: Some(common::ReplicateCheckpoint {
                    time_tick: 400,
                    ..checkpoint
                }),
            }
        }
    );

    fn dump_messages<'life0, 'async_trait>(
        &'life0 self,
        request: Request<pb::DumpMessagesRequest>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Response<tonic::codegen::BoxStream<pb::DumpMessagesResponse>>,
                        Status,
                    >,
                > + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let request = request.into_inner();
            self.record_request("dump_messages", &request);
            if let Some(status) = self.take_transport_failure("dump_messages") {
                return Err(status);
            }
            let response = pb::DumpMessagesResponse {
                response: Some(pb::dump_messages_response::Response::Message(
                    common::ImmutableMessage {
                        id: (request.pchannel != "missing-message-id").then_some(
                            common::MessageId {
                                id: "message-1".into(),
                                wal_name: common::WalName::Pulsar as i32,
                            },
                        ),
                        payload: vec![1, 2, 3],
                        properties: HashMap::from([("key".into(), "value".into())]),
                        ..Default::default()
                    },
                )),
            };
            let stream: tonic::codegen::BoxStream<pb::DumpMessagesResponse> =
                if request.pchannel == "delayed-channel" {
                    let (sender, receiver) = tokio::sync::mpsc::channel(1);
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(75)).await;
                        let _ = sender.send(Ok(response)).await;
                    });
                    Box::pin(tokio_stream::wrappers::ReceiverStream::new(receiver))
                } else {
                    Box::pin(tokio_stream::iter([Ok(response)]))
                };
            Ok(Response::new(stream))
        })
    }
    status_method!(create_snapshot, pb::CreateSnapshotRequest);
    status_method!(drop_snapshot, pb::DropSnapshotRequest);
    response_method!(
        list_snapshots,
        pb::ListSnapshotsRequest,
        pb::ListSnapshotsResponse,
        pb::ListSnapshotsResponse {
            status: Some(success_status()),
            snapshots: vec!["snap-1".into(), "snap-2".into()],
        }
    );
    response_method!(
        describe_snapshot,
        pb::DescribeSnapshotRequest,
        pb::DescribeSnapshotResponse,
        pb::DescribeSnapshotResponse {
            status: Some(success_status()),
            name: "snap-1".into(),
            description: "backup".into(),
            collection_name: "books".into(),
            partition_names: vec!["p1".into(), "p2".into()],
            create_ts: 123,
            s3_location: "s3://bucket/export".into(),
        }
    );
    response_method!(
        restore_snapshot,
        pb::RestoreSnapshotRequest,
        pb::RestoreSnapshotResponse,
        pb::RestoreSnapshotResponse {
            status: Some(success_status()),
            job_id: 7,
        }
    );
    response_method!(
        get_restore_snapshot_state,
        pb::GetRestoreSnapshotStateRequest,
        pb::GetRestoreSnapshotStateResponse,
        pb::GetRestoreSnapshotStateResponse {
            status: Some(success_status()),
            info: Some(pb::RestoreSnapshotInfo {
                job_id: 7,
                snapshot_name: "snap-1".into(),
                db_name: "default".into(),
                collection_name: "books".into(),
                state: pb::RestoreSnapshotState::RestoreSnapshotExecuting as i32,
                progress: 50,
                reason: String::new(),
                start_time: 100,
                time_cost: 5,
            }),
        }
    );
    response_method!(
        list_restore_snapshot_jobs,
        pb::ListRestoreSnapshotJobsRequest,
        pb::ListRestoreSnapshotJobsResponse,
        pb::ListRestoreSnapshotJobsResponse {
            status: Some(success_status()),
            jobs: vec![pb::RestoreSnapshotInfo {
                job_id: 7,
                snapshot_name: "snap-1".into(),
                db_name: "default".into(),
                collection_name: "books".into(),
                state: pb::RestoreSnapshotState::RestoreSnapshotCompleted as i32,
                progress: 100,
                reason: String::new(),
                start_time: 100,
                time_cost: 5,
            }],
        }
    );
    response_method!(
        pin_snapshot_data,
        pb::PinSnapshotDataRequest,
        pb::PinSnapshotDataResponse,
        pb::PinSnapshotDataResponse {
            status: Some(success_status()),
            pin_id: 42,
        }
    );
    status_method!(unpin_snapshot_data, pb::UnpinSnapshotDataRequest);
}

pub struct MockServer {
    pub client: ClientV2,
    pub service: MockMilvus,
    pub uri: String,
    shutdown: Option<oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
    _topology_shutdown: Option<oneshot::Sender<()>>,
    _topology_task: Option<tokio::task::JoinHandle<()>>,
}

impl MockServer {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let incoming = TcpListenerStream::new(listener);
        let (shutdown, shutdown_rx) = oneshot::channel();
        let service = MockMilvus::default();
        let server_service = service.clone();
        let task = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(MilvusServiceServer::new(server_service))
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });
        let uri = format!("http://{address}");
        let config = ConnectConfig::new().uri(&uri).database("default");
        let client = ClientV2::new(&config).await.unwrap();
        Self {
            client,
            service,
            uri,
            shutdown: Some(shutdown),
            task,
            _topology_shutdown: None,
            _topology_task: None,
        }
    }

    /// Starts a mock server whose client connects through a global-cluster endpoint.
    ///
    /// A topology REST server advertises a single writable primary pointing at the mock gRPC
    /// server, so `ClientV2::new` exercises the real discovery + `wait_for_server` handshake
    /// path and any DQL/DML call routes to the mock gRPC server through the primary endpoint.
    pub async fn start_global(topology: String) -> Self {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let incoming = TcpListenerStream::new(listener);
        let (shutdown, shutdown_rx) = oneshot::channel();
        let service = MockMilvus::default();
        let server_service = service.clone();
        let grpc_task = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(MilvusServiceServer::new(server_service))
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });
        let grpc_uri = format!("http://{address}");

        // Serve the global-cluster topology REST endpoint. The topology body is parameterized
        // with the mock gRPC server's address as the primary endpoint.
        let topology_body = topology.replace("{endpoint}", &grpc_uri);
        let topology_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let topology_address = topology_listener.local_addr().unwrap();
        let (topology_shutdown, topology_shutdown_rx) = oneshot::channel::<()>();
        let topology_task = tokio::spawn(async move {
            let mut topology_shutdown_rx = topology_shutdown_rx;
            loop {
                tokio::select! {
                    _ = &mut topology_shutdown_rx => {
                        break;
                    }
                    accepted = topology_listener.accept() => {
                        let (mut stream, _) = accepted.expect("accept topology request");
                        let mut buffer = Vec::new();
                        loop {
                            let mut chunk = [0_u8; 4096];
                            let count = stream.read(&mut chunk).await.expect("read topology request");
                            if count == 0 {
                                break;
                            }
                            buffer.extend_from_slice(&chunk[..count]);
                            if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                                break;
                            }
                        }
                        let body = topology_body.clone();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                }
            }
        });

        let global_uri = format!("http://{topology_address}/global-cluster");
        let config = ConnectConfig::new().uri(&global_uri).database("default");
        let client = ClientV2::new(&config).await.unwrap();
        Self {
            client,
            service,
            uri: grpc_uri,
            shutdown: Some(shutdown),
            task: grpc_task,
            // Keep the topology server alive for the duration of the test; it is detached and
            // stops when the runtime is torn down.
            _topology_shutdown: Some(topology_shutdown),
            _topology_task: Some(topology_task),
        }
    }

    pub fn assert_called(&self, method: &'static str) {
        assert!(
            self.service.call_count(method) > 0,
            "mock RPC {method} was not called"
        );
    }

    pub fn assert_request_contains(&self, method: &'static str, expected: &[&str]) {
        let request = self.service.request_text(method);
        assert!(!request.is_empty(), "mock RPC {method} captured no request");
        for value in expected {
            assert!(
                request.contains(value),
                "mock RPC {method} request did not contain {value:?}: {request}"
            );
        }
    }

    pub fn assert_any_request_contains(&self, method: &'static str, expected: &[&str]) {
        let requests = self.service.request_texts(method);
        assert!(
            !requests.is_empty(),
            "mock RPC {method} captured no request"
        );
        assert!(
            requests
                .iter()
                .any(|request| expected.iter().all(|value| request.contains(value))),
            "no mock RPC {method} request contained {expected:?}: {requests:?}"
        );
    }

    pub async fn shutdown(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(topology_shutdown) = self._topology_shutdown.take() {
            let _ = topology_shutdown.send(());
        }
        self.task.await.unwrap();
        if let Some(topology_task) = self._topology_task.take() {
            topology_task.await.unwrap();
        }
    }
}
