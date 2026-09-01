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

use milvus::v2::request::alias::{DropAliasRequest, ListAliasesRequest};
use milvus::v2::request::collection::{
    CreateCollectionRequest, DropCollectionRequest, LoadCollectionRequest,
};
use milvus::v2::request::dml::{EntityRow, InsertRequest};
use milvus::v2::request::index::CreateIndexRequest;
use milvus::v2::request::utility::{FlushRequest, GetServerVersionRequest};
use milvus::v2::{
    ClientV2, CollectionSchema, ConnectConfig, ConsistencyLevel, DataType, DefaultValue, FieldData,
    FieldSchema, IndexParam, IndexType, MetricType, StructFieldSchema, StructValue,
};
use serde_json::json;
use std::io::{Read, Write};
use std::net::{TcpListener, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) const ID_FIELD: &str = "id";
pub(super) const VECTOR_FIELD: &str = "vector";
pub(super) const BINARY_VECTOR_FIELD: &str = "binary_vector";
pub(super) const BFLOAT16_VECTOR_FIELD: &str = "bfloat16_vector";
pub(super) const SPARSE_VECTOR_FIELD: &str = "sparse_vector";
pub(super) const GEOMETRY_FIELD: &str = "location";
pub(super) const TIMESTAMPTZ_FIELD: &str = "observed_at";
pub(super) const STRUCT_FIELD: &str = "events";
pub(super) const BOOL_ARRAY_FIELD: &str = "bool_array";
pub(super) const INT8_ARRAY_FIELD: &str = "int8_array";
pub(super) const INT16_ARRAY_FIELD: &str = "int16_array";
pub(super) const INT32_ARRAY_FIELD: &str = "int32_array";
pub(super) const INT64_ARRAY_FIELD: &str = "int64_array";
pub(super) const FLOAT_ARRAY_FIELD: &str = "float_array";
pub(super) const DOUBLE_ARRAY_FIELD: &str = "double_array";
pub(super) const VARCHAR_ARRAY_FIELD: &str = "varchar_array";
const STRUCT_LABEL_FIELD: &str = "label";
const STRUCT_SCORE_FIELD: &str = "score";
pub(super) const VECTOR_DIMENSION: usize = 4;

static COLLECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) struct CollectionCleanup {
    uri: String,
    collections: Vec<(String, String)>,
}

impl CollectionCleanup {
    pub(super) fn new(collection_names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::in_database("default", collection_names)
    }

    pub(super) fn in_database(
        database_name: impl Into<String>,
        collection_names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let database_name = database_name.into();
        Self {
            uri: std::env::var("MILVUS_URI")
                .unwrap_or_else(|_| "http://localhost:29830".to_owned()),
            collections: collection_names
                .into_iter()
                .map(|name| (database_name.clone(), name.into()))
                .collect(),
        }
    }
}

impl Drop for CollectionCleanup {
    fn drop(&mut self) {
        if !std::thread::panicking() || self.collections.is_empty() {
            return;
        }
        let uri = self.uri.clone();
        let collections = std::mem::take(&mut self.collections);
        let cleanup = std::thread::Builder::new().spawn(move || {
            let Ok(runtime) = tokio::runtime::Runtime::new() else {
                return;
            };
            runtime.block_on(async move {
                let config = ConnectConfig::new()
                    .uri(uri)
                    .rpc_timeout(Duration::from_secs(60));
                let Ok(client) = ClientV2::new(&config).await else {
                    return;
                };
                for (database, collection) in collections.into_iter().rev() {
                    if let Ok(aliases) = client
                        .list_aliases(
                            ListAliasesRequest::builder()
                                .database_name(&database)
                                .collection_name(&collection)
                                .build()
                                .expect("valid request"),
                        )
                        .await
                    {
                        for alias in aliases.aliases() {
                            let _ = client
                                .drop_alias(
                                    DropAliasRequest::builder()
                                        .database_name(&database)
                                        .alias(alias)
                                        .build()
                                        .expect("valid request"),
                                )
                                .await;
                        }
                    }
                    let _ = client
                        .drop_collection(
                            DropCollectionRequest::builder()
                                .database_name(database)
                                .collection_name(collection)
                                .build()
                                .expect("valid request"),
                        )
                        .await;
                }
            });
        });
        if let Ok(cleanup) = cleanup {
            let _ = cleanup.join();
        }
    }
}

pub(super) struct MockEmbeddingServer {
    endpoint: String,
    stopped: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl MockEmbeddingServer {
    pub(super) fn start() -> Self {
        let listener = TcpListener::bind("0.0.0.0:0").expect("bind mock embedding server");
        listener
            .set_nonblocking(true)
            .expect("configure mock embedding server");
        let port = listener.local_addr().expect("mock server address").port();
        let endpoint_host = std::env::var("MILVUS_TEST_PROVIDER_HOST")
            .unwrap_or_else(|_| host_address().to_string());
        let stopped = Arc::new(AtomicBool::new(false));
        let thread_stopped = stopped.clone();
        let thread = thread::spawn(move || {
            while !thread_stopped.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                        let mut request = [0_u8; 8_192];
                        let _ = stream.read(&mut request);
                        let body = "[[0.1,0.2,0.3,0.4]]";
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            endpoint: format!("http://{endpoint_host}:{port}"),
            stopped,
            thread: Some(thread),
        }
    }

    pub(super) fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl Drop for MockEmbeddingServer {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn host_address() -> std::net::IpAddr {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("bind host address probe");
    socket
        .connect("8.8.8.8:80")
        .expect("determine host address for Milvus provider callback");
    socket.local_addr().expect("host address probe result").ip()
}

pub(super) async fn client() -> ClientV2 {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:29830".to_owned());
    let config = ConnectConfig::new().uri(uri);
    ClientV2::new(&config).await.expect("connect to Milvus")
}

/// Returns the connected server's major version. Unparseable version strings (for example
/// master snapshots) are treated as new enough to carry Milvus 3.0 schema-DDL features.
pub(super) async fn server_major_version() -> u32 {
    let client = client().await;
    let request = GetServerVersionRequest::builder()
        .build()
        .expect("valid request");
    match client.server_version(request).await {
        Ok(response) => {
            let version = response.version();
            version
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(3)
        }
        Err(_) => 3,
    }
}

/// Whether the connected server supports function-field and struct-field DDL
/// (`add_function_field` / `drop_collection_field` / `add_collection_struct_field`),
/// which are Milvus 3.0 features.
pub(super) async fn server_supports_schema_ddl() -> bool {
    server_major_version().await >= 3
}

pub(super) fn unique_collection_name(area: &str) -> String {
    unique_name(area)
}

pub(super) fn unique_name(area: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after Unix epoch")
        .as_millis();
    let counter = COLLECTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "rust_v2_st_{area}_{}_{timestamp}_{counter}",
        std::process::id()
    )
}

pub(super) fn advanced_schema() -> CollectionSchema {
    CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("bool_value")
                .data_type(DataType::Bool)
                .nullable(true)
                .default_value(DefaultValue::Bool(true)),
        )
        .add_field(
            FieldSchema::new()
                .name("int8_value")
                .data_type(DataType::Int8)
                .nullable(true)
                .default_value(DefaultValue::Int32(8)),
        )
        .add_field(
            FieldSchema::new()
                .name("int16_value")
                .data_type(DataType::Int16)
                .nullable(true)
                .default_value(DefaultValue::Int32(16)),
        )
        .add_field(
            FieldSchema::new()
                .name("int32_value")
                .data_type(DataType::Int32)
                .nullable(true)
                .default_value(DefaultValue::Int32(32)),
        )
        .add_field(
            FieldSchema::new()
                .name("float_value")
                .data_type(DataType::Float)
                .nullable(true)
                .default_value(DefaultValue::Float(1.5)),
        )
        .add_field(
            FieldSchema::new()
                .name("double_value")
                .data_type(DataType::Double)
                .nullable(true)
                .default_value(DefaultValue::Double(2.5)),
        )
        .add_field(
            FieldSchema::new()
                .name("varchar_value")
                .data_type(DataType::VarChar)
                .max_length(256)
                .nullable(true)
                .default_value(DefaultValue::String("default".into())),
        )
        .add_field(
            FieldSchema::new()
                .name("json_value")
                .data_type(DataType::Json)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(GEOMETRY_FIELD)
                .data_type(DataType::Geometry)
                .nullable(true)
                .default_value(DefaultValue::String("POINT (0 0)".into())),
        )
        .add_field(
            FieldSchema::new()
                .name(TIMESTAMPTZ_FIELD)
                .data_type(DataType::Timestamptz)
                .nullable(true)
                .default_value(DefaultValue::String("2025-01-01T00:00:00+08:00".into())),
        )
        .add_field(
            FieldSchema::new()
                .name(BOOL_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Bool)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(INT8_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Int8)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(INT16_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Int16)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(INT32_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Int32)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(INT64_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Int64)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(FLOAT_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Float)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(DOUBLE_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::Double)
                .max_capacity(16)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(VARCHAR_ARRAY_FIELD)
                .data_type(DataType::Array)
                .element_type(DataType::VarChar)
                .max_capacity(16)
                .max_length(128)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR_FIELD)
                .data_type(DataType::FloatVector)
                .dimension(VECTOR_DIMENSION as u32),
        )
        .add_field(
            FieldSchema::new()
                .name(BINARY_VECTOR_FIELD)
                .data_type(DataType::BinaryVector)
                .dimension(8)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(BFLOAT16_VECTOR_FIELD)
                .data_type(DataType::BFloat16Vector)
                .dimension(VECTOR_DIMENSION as u32)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name(SPARSE_VECTOR_FIELD)
                .data_type(DataType::SparseFloatVector)
                .nullable(true),
        )
        .add_struct_field(
            StructFieldSchema::new()
                .name(STRUCT_FIELD)
                .max_capacity(4)
                .add_field(
                    FieldSchema::new()
                        .name(STRUCT_LABEL_FIELD)
                        .data_type(DataType::VarChar)
                        .max_length(128),
                )
                .add_field(
                    FieldSchema::new()
                        .name(STRUCT_SCORE_FIELD)
                        .data_type(DataType::Int32),
                ),
        )
}

pub(super) fn advanced_load_fields() -> Vec<&'static str> {
    vec![
        ID_FIELD,
        "bool_value",
        "int8_value",
        "int16_value",
        "int32_value",
        "float_value",
        "double_value",
        "varchar_value",
        "json_value",
        BOOL_ARRAY_FIELD,
        INT8_ARRAY_FIELD,
        INT16_ARRAY_FIELD,
        INT32_ARRAY_FIELD,
        INT64_ARRAY_FIELD,
        FLOAT_ARRAY_FIELD,
        DOUBLE_ARRAY_FIELD,
        VARCHAR_ARRAY_FIELD,
        VECTOR_FIELD,
        BINARY_VECTOR_FIELD,
        BFLOAT16_VECTOR_FIELD,
        SPARSE_VECTOR_FIELD,
        GEOMETRY_FIELD,
        TIMESTAMPTZ_FIELD,
        STRUCT_FIELD,
    ]
}

pub(super) async fn create_advanced_collection(client: &ClientV2, collection_name: &str) {
    let _ = drop_collection(client, collection_name).await;
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection_name)
                .schema(advanced_schema())
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create advanced-type collection");
}

pub(super) async fn drop_collection(
    client: &ClientV2,
    collection_name: &str,
) -> milvus::v2::error::Result<()> {
    client
        .drop_collection(
            DropCollectionRequest::builder()
                .collection_name(collection_name)
                .build()
                .expect("valid request"),
        )
        .await
}

pub(super) fn advanced_columns() -> Vec<FieldData> {
    vec![
        FieldData::Int64 {
            name: ID_FIELD.to_owned(),
            values: vec![1, 2],
        },
        FieldData::Bool {
            name: "bool_value".to_owned(),
            values: vec![true, false],
        },
        FieldData::Int8 {
            name: "int8_value".to_owned(),
            values: vec![8, 9],
        },
        FieldData::Int16 {
            name: "int16_value".to_owned(),
            values: vec![16, 17],
        },
        FieldData::Int32 {
            name: "int32_value".to_owned(),
            values: vec![32, 33],
        },
        FieldData::Float {
            name: "float_value".to_owned(),
            values: vec![1.5, 2.5],
        },
        FieldData::Double {
            name: "double_value".to_owned(),
            values: vec![10.25, 20.5],
        },
        FieldData::VarChar {
            name: "varchar_value".to_owned(),
            values: vec!["first".to_owned(), "second".to_owned()],
        },
        FieldData::Json {
            name: "json_value".to_owned(),
            values: vec![json!({"rank": 1}), json!({"rank": 2})],
        },
        FieldData::FloatVector {
            name: VECTOR_FIELD.to_owned(),
            values: vec![vec![0.1, 0.2, 0.3, 0.4], vec![0.4, 0.3, 0.2, 0.1]],
        },
        FieldData::BFloat16Vector {
            name: BFLOAT16_VECTOR_FIELD.to_owned(),
            values: vec![
                bfloat16_vector([0.1, 0.2, 0.3, 0.4]),
                bfloat16_vector([0.4, 0.3, 0.2, 0.1]),
            ],
        },
        FieldData::BinaryVector {
            name: BINARY_VECTOR_FIELD.to_owned(),
            values: vec![vec![0b1010_1010], vec![0b0101_0101]],
        },
        FieldData::Geometry {
            name: GEOMETRY_FIELD.to_owned(),
            values: vec!["POINT (1 1)".to_owned(), "POINT (2 2)".to_owned()],
        },
        FieldData::Timestamptz {
            name: TIMESTAMPTZ_FIELD.to_owned(),
            values: vec![
                "2025-01-01T00:00:00+08:00".to_owned(),
                "2025-01-02T00:00:00+08:00".to_owned(),
            ],
        },
        FieldData::ArrayBool {
            name: BOOL_ARRAY_FIELD.to_owned(),
            values: vec![vec![true, false], vec![false, true]],
        },
        FieldData::ArrayInt8 {
            name: INT8_ARRAY_FIELD.to_owned(),
            values: vec![vec![-8, 8], vec![-9, 9]],
        },
        FieldData::ArrayInt16 {
            name: INT16_ARRAY_FIELD.to_owned(),
            values: vec![vec![-16, 16], vec![-17, 17]],
        },
        FieldData::ArrayInt32 {
            name: INT32_ARRAY_FIELD.to_owned(),
            values: vec![vec![-32, 32], vec![-33, 33]],
        },
        FieldData::ArrayInt64 {
            name: INT64_ARRAY_FIELD.to_owned(),
            values: vec![vec![1, 2], vec![3, 4]],
        },
        FieldData::ArrayFloat {
            name: FLOAT_ARRAY_FIELD.to_owned(),
            values: vec![vec![1.25, 2.5], vec![3.75, 4.5]],
        },
        FieldData::ArrayDouble {
            name: DOUBLE_ARRAY_FIELD.to_owned(),
            values: vec![vec![10.25, 20.5], vec![30.75, 40.125]],
        },
        FieldData::ArrayVarChar {
            name: VARCHAR_ARRAY_FIELD.to_owned(),
            values: vec![
                vec!["first".to_owned(), "second".to_owned()],
                vec!["third".to_owned(), "fourth".to_owned()],
            ],
        },
        FieldData::SparseFloatVector {
            name: SPARSE_VECTOR_FIELD.to_owned(),
            values: vec![
                [(1, 0.5), (7, 0.25)].into_iter().collect(),
                [(2, 0.75), (9, 0.125)].into_iter().collect(),
            ],
        },
        FieldData::Struct {
            name: STRUCT_FIELD.to_owned(),
            values: vec![
                vec![struct_value("created", 10)],
                vec![struct_value("created", 20), struct_value("updated", 21)],
            ],
        },
    ]
}

pub(super) fn advanced_rows() -> Vec<EntityRow> {
    vec![
        json!({
            "id": 11,
            "int8_value": 18,
            "int16_value": 116,
            "int32_value": 1116,
            "vector": [0.1, 0.2, 0.3, 0.4],
            "bfloat16_vector": [0.1, 0.2, 0.3, 0.4],
            "location": "POINT (11 11)",
            "observed_at": "2025-02-01T00:00:00+08:00",
            "bool_array": [true, false],
            "int8_array": [-8, 8],
            "int16_array": [-16, 16],
            "int32_array": [-32, 32],
            "int64_array": [11, 12],
            "float_array": [1.25, 2.5],
            "double_array": [10.25, 20.5],
            "varchar_array": ["first", "second"],
            "sparse_vector": {"indices": [1, 7], "values": [0.5, 0.25]},
            "events": [{ "label": "created", "score": 110 }],
        }),
        json!({
            "id": 12,
            "int8_value": 19,
            "int16_value": 117,
            "int32_value": 1117,
            "vector": [0.4, 0.3, 0.2, 0.1],
            "bfloat16_vector": [0.4, 0.3, 0.2, 0.1],
            "location": "POINT (12 12)",
            "observed_at": "2025-02-02T00:00:00+08:00",
            "bool_array": [false, true],
            "int8_array": [-9, 9],
            "int16_array": [-17, 17],
            "int32_array": [-33, 33],
            "int64_array": [12, 13],
            "float_array": [3.75, 4.5],
            "double_array": [30.75, 40.125],
            "varchar_array": ["third", "fourth"],
            "sparse_vector": {"2": 0.75, "9": 0.125},
            "events": [
                { "label": "created", "score": 120 },
                { "label": "updated", "score": 121 }
            ],
        }),
    ]
    .into_iter()
    .map(|value| {
        value
            .as_object()
            .expect("advanced row is a JSON object")
            .clone()
    })
    .collect()
}

pub(super) async fn prepare_loaded_collection(client: &ClientV2, collection_name: &str) {
    create_advanced_collection(client, collection_name).await;
    let request = InsertRequest::builder()
        .collection_name(collection_name)
        .columns(advanced_columns())
        .build()
        .expect("build insert request");
    client.insert(request).await.expect("insert advanced data");
    client
        .flush(
            FlushRequest::builder()
                .collection_names([collection_name])
                .wait_flushed_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("flush advanced data");
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(collection_name)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::L2),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(BFLOAT16_VECTOR_FIELD)
                        .index_type(IndexType::AutoIndex)
                        .metric_type(MetricType::L2),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(BINARY_VECTOR_FIELD)
                        .index_type(IndexType::BinFlat)
                        .metric_type(MetricType::Hamming),
                )
                .index_param(
                    IndexParam::new()
                        .field_name(SPARSE_VECTOR_FIELD)
                        .index_type(IndexType::SparseInvertedIndex)
                        .metric_type(MetricType::Ip),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create vector index");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection_name)
                .load_fields(advanced_load_fields())
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load advanced-type collection");
}

pub(super) fn bfloat16_vector(values: [f32; VECTOR_DIMENSION]) -> Vec<u16> {
    milvus::v2::array_f32_to_bf16(&values)
}

pub(super) fn assert_advanced_fields(fields: &[FieldData], expected_rows: usize) {
    let int8 = fields
        .iter()
        .find(|field| field.name() == "int8_value")
        .expect("int8 output field");
    assert!(matches!(
        int8.inner(),
        FieldData::Int8 { values, .. } if values.len() == expected_rows
    ));

    let int16 = fields
        .iter()
        .find(|field| field.name() == "int16_value")
        .expect("int16 output field");
    assert!(matches!(
        int16.inner(),
        FieldData::Int16 { values, .. } if values.len() == expected_rows
    ));

    let int32 = fields
        .iter()
        .find(|field| field.name() == "int32_value")
        .expect("int32 output field");
    assert!(matches!(
        int32.inner(),
        FieldData::Int32 { values, .. } if values.len() == expected_rows
    ));

    let bool_array = fields
        .iter()
        .find(|field| field.name() == BOOL_ARRAY_FIELD)
        .expect("bool array output field");
    assert!(matches!(
        bool_array.inner(),
        FieldData::ArrayBool { values, .. } if values.len() == expected_rows
    ));

    let int8_array = fields
        .iter()
        .find(|field| field.name() == INT8_ARRAY_FIELD)
        .expect("int8 array output field");
    assert!(matches!(
        int8_array.inner(),
        FieldData::ArrayInt8 { values, .. } if values.len() == expected_rows
    ));

    let int16_array = fields
        .iter()
        .find(|field| field.name() == INT16_ARRAY_FIELD)
        .expect("int16 array output field");
    assert!(matches!(
        int16_array.inner(),
        FieldData::ArrayInt16 { values, .. } if values.len() == expected_rows
    ));

    let int32_array = fields
        .iter()
        .find(|field| field.name() == INT32_ARRAY_FIELD)
        .expect("int32 array output field");
    assert!(matches!(
        int32_array.inner(),
        FieldData::ArrayInt32 { values, .. } if values.len() == expected_rows
    ));

    let int64_array = fields
        .iter()
        .find(|field| field.name() == INT64_ARRAY_FIELD)
        .expect("int64 array output field");
    assert!(matches!(
        int64_array.inner(),
        FieldData::ArrayInt64 { values, .. } if values.len() == expected_rows
    ));

    let float_array = fields
        .iter()
        .find(|field| field.name() == FLOAT_ARRAY_FIELD)
        .expect("float array output field");
    assert!(matches!(
        float_array.inner(),
        FieldData::ArrayFloat { values, .. } if values.len() == expected_rows
    ));

    let double_array = fields
        .iter()
        .find(|field| field.name() == DOUBLE_ARRAY_FIELD)
        .expect("double array output field");
    assert!(matches!(
        double_array.inner(),
        FieldData::ArrayDouble { values, .. } if values.len() == expected_rows
    ));

    let varchar_array = fields
        .iter()
        .find(|field| field.name() == VARCHAR_ARRAY_FIELD)
        .expect("varchar array output field");
    assert!(matches!(
        varchar_array.inner(),
        FieldData::ArrayVarChar { values, .. } if values.len() == expected_rows
    ));

    let float_vector = fields
        .iter()
        .find(|field| field.name() == VECTOR_FIELD)
        .expect("float vector output field");
    assert!(matches!(
        float_vector.inner(),
        FieldData::FloatVector { values, .. } if values.len() == expected_rows
    ));

    let binary_vector = fields
        .iter()
        .find(|field| field.name() == BINARY_VECTOR_FIELD)
        .expect("binary vector output field");
    assert!(matches!(
        binary_vector.inner(),
        FieldData::BinaryVector { values, .. } if values.len() == expected_rows
    ));

    let bfloat16_vector = fields
        .iter()
        .find(|field| field.name() == BFLOAT16_VECTOR_FIELD)
        .expect("bfloat16 vector output field");
    assert!(matches!(
        bfloat16_vector.inner(),
        FieldData::BFloat16Vector { values, .. } if values.len() == expected_rows
    ));

    let sparse_vector = fields
        .iter()
        .find(|field| field.name() == SPARSE_VECTOR_FIELD)
        .expect("sparse vector output field");
    assert!(matches!(
        sparse_vector.inner(),
        FieldData::SparseFloatVector { values, .. } if values.len() == expected_rows
    ));

    let geometry = fields
        .iter()
        .find(|field| field.name() == GEOMETRY_FIELD)
        .expect("geometry output field");
    assert!(matches!(
        geometry.inner(),
        FieldData::Geometry { values, .. } if values.len() == expected_rows
    ));

    let timestamptz = fields
        .iter()
        .find(|field| field.name() == TIMESTAMPTZ_FIELD)
        .expect("timestamptz output field");
    assert!(matches!(
        timestamptz.inner(),
        FieldData::Timestamptz { values, .. } if values.len() == expected_rows
    ));

    let struct_data = fields
        .iter()
        .find(|field| field.name() == STRUCT_FIELD)
        .expect("struct output field");
    let FieldData::Struct { values, .. } = struct_data else {
        panic!("struct output field has the wrong data type");
    };
    assert_eq!(values.len(), expected_rows);
    assert!(values.iter().all(|row| {
        !row.is_empty()
            && row.iter().all(|value| {
                value.contains_key(STRUCT_LABEL_FIELD) && value.contains_key(STRUCT_SCORE_FIELD)
            })
    }));
}

fn struct_value(label: &str, score: i32) -> StructValue {
    json!({
        "label": label,
        "score": score,
    })
    .as_object()
    .expect("struct value is a JSON object")
    .clone()
}
