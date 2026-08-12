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

use milvus::v2::prelude::*;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const DIMENSION: u32 = 4;

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".into());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".into());
    let collection = unique_collection_name();
    // ClientV2::new establishes the connection. `uri` selects the Milvus endpoint and `token`
    // supplies an API key or username/password credential.
    println!("Calling ClientV2::new: connect to {uri}");
    let client = ClientV2::new(&ConnectConfig::new().uri(uri).token(token)).await?;
    println!("ClientV2::new completed");

    let result = async {
        // create_collection defines the collection schema and its vector index. The collection
        // name identifies the resource, while the schema describes field types and dimensions.
        println!("Calling create_collection: create {collection:?} with schema and vector index");
        client
            .create_collection(
                CreateCollectionRequest::builder()
                    .collection_name(&collection)
                    .schema(
                        CollectionSchema::new()
                            .enable_dynamic_field(false)
                            .add_field(
                                FieldSchema::new()
                                    .name("id")
                                    .data_type(DataType::Int64)
                                    .primary_key(true),
                            )
                            .add_field(
                                FieldSchema::new()
                                    .name("text")
                                    .data_type(DataType::VarChar)
                                    .max_length(256),
                            )
                            .add_field(
                                FieldSchema::new()
                                    .name("embedding")
                                    .data_type(DataType::FloatVector)
                                    .dimension(DIMENSION),
                            ),
                    )
                    .index_param(
                        IndexParam::new()
                            .field_name("embedding")
                            .index_type(IndexType::AutoIndex)
                            .metric_type(MetricType::Cosine),
                    )
                    .build()?,
            )
            .await?;
        println!("create_collection completed");
        // insert writes the JSON rows into the selected collection. Each row must contain values
        // compatible with the fields declared above.
        println!("Calling insert: write two JSON rows to {collection:?}");
        client
            .insert(
                InsertRequest::builder()
                    .collection_name(&collection)
                    .rows([
                        json!({"id": 1, "text": "hello", "embedding": [0.1, 0.2, 0.3, 0.4]}),
                        json!({"id": 2, "text": "milvus", "embedding": [0.2, 0.3, 0.4, 0.5]}),
                    ])
                    .build()?,
            )
            .await?;
        println!("insert completed");
        // load_collection prepares the collection for serving queries. `sync(true)` waits for
        // readiness and `timeout_ms` bounds that wait.
        println!("Calling load_collection: load {collection:?} synchronously");
        client
            .load_collection(
                LoadCollectionRequest::builder()
                    .collection_name(&collection)
                    .sync(true)
                    .timeout_ms(60_000)
                    .build()?,
            )
            .await?;
        println!("load_collection completed");
        // search finds nearest vectors. `vector_field` selects the indexed field, `vectors` is the
        // query vector, `output_fields` selects returned data, and `limit` caps matches.
        println!("Calling search: find the two nearest embedding matches");
        let response = client
            .search(
                SearchRequest::builder()
                    .collection_name(&collection)
                    .vector_field("embedding")
                    .vectors(SearchVectors::Float(vec![vec![0.1, 0.2, 0.3, 0.4]]))
                    .output_fields(["text"])
                    .limit(2)
                    .build()?,
            )
            .await?;
        println!("search completed");
        for result in response.results().iter() {
            for row in result.rows()? {
                println!(
                    "id={}, score={:.4}, text={:?}",
                    row.get_i64("id")?,
                    row.get_f32("score")?,
                    row.get_str("text")?
                );
            }
        }
        Ok::<_, milvus::v2::error::Error>(())
    }
    .await;

    // drop_collection removes the tutorial collection, its data, and its indexes.
    println!("Calling drop_collection: remove {collection:?}");
    let cleanup = client
        .drop_collection(
            DropCollectionRequest::builder()
                .collection_name(&collection)
                .build()?,
        )
        .await;
    if cleanup.is_ok() {
        println!("drop_collection completed");
    }
    result?;
    cleanup
}

fn unique_collection_name() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("RUST_V2_QUICKSTART_{millis}_{}", std::process::id())
}
