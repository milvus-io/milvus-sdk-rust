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
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const CATEGORY_INDEX: &str = "category_inverted_idx";
const PRICE_INDEX: &str = "price_sort_idx";
const VECTOR_INDEX: &str = "embedding_hnsw_idx";

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".to_owned());
    let collection = tutorial_collection_name();
    // ClientV2::new connects to `uri` and authenticates with `token` for index administration.
    println!("Calling ClientV2::new: connect to Milvus");
    let client = ClientV2::new(&ConnectConfig::new().uri(uri).token(token)).await?;
    println!("ClientV2::new completed");

    let tutorial_result = demonstrate_indexes(&client, &collection).await;
    let cleanup_result = cleanup_collection(&client, &collection).await;
    if let Err(error) = &cleanup_result {
        eprintln!("Failed to clean up {collection:?}: {error}");
    }
    tutorial_result?;
    cleanup_result
}

async fn demonstrate_indexes(client: &ClientV2, collection: &str) -> Result<()> {
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name("id")
                .data_type(DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            FieldSchema::new()
                .name("category")
                .data_type(DataType::VarChar)
                .max_length(128),
        )
        .add_field(FieldSchema::new().name("price").data_type(DataType::Float))
        .add_field(
            FieldSchema::new()
                .name("embedding")
                .data_type(DataType::FloatVector)
                .dimension(8),
        );

    println!("Creating collection {collection:?} without indexes");
    // create_collection creates the fields that the indexes will target. No index is supplied here
    // because this tutorial demonstrates create_index separately.
    println!("Calling create_collection: create {collection:?} without indexes");
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection)
                .schema(schema)
                .build()?,
        )
        .await?;
    println!("create_collection completed");

    println!("Creating one vector index and two scalar indexes");
    // create_index builds all supplied index definitions. Each IndexParam identifies a field,
    // index name/type, and any metric or build parameters; `sync` waits for completion.
    println!("Calling create_index: build vector and scalar indexes");
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(collection)
                .index_params(vec![
                    IndexParam::new()
                        .field_name("embedding")
                        .index_name(VECTOR_INDEX)
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::Cosine)
                        .extra_params(HashMap::from([
                            ("M".to_owned(), "16".to_owned()),
                            ("efConstruction".to_owned(), "100".to_owned()),
                        ])),
                    IndexParam::new()
                        .field_name("category")
                        .index_name(CATEGORY_INDEX)
                        .index_type(IndexType::Inverted),
                    IndexParam::new()
                        .field_name("price")
                        .index_name(PRICE_INDEX)
                        .index_type(IndexType::StlSort),
                ])
                .sync(true)
                .timeout_ms(60_000)
                .build()?,
        )
        .await?;
    println!("create_index completed");

    print_indexes(client, collection, "Created indexes").await?;

    // describe_index returns detailed metadata for the selected field and index name.
    println!("Calling describe_index: inspect {VECTOR_INDEX:?}");
    let response = client
        .describe_index(
            DescribeIndexRequest::builder()
                .collection_name(collection)
                .field_name("embedding")
                .index_name(VECTOR_INDEX)
                .build()?,
        )
        .await?;
    println!("describe_index completed");
    for index in response.indexes() {
        println!(
            "Vector index detail: name={:?}, type={:?}, metric={:?}, state={:?}, params={:?}",
            index.get_index_name(),
            index.get_index_type(),
            index.get_metric_type(),
            index.get_state(),
            index.get_extra_params()
        );
    }

    println!("Dropping scalar index {PRICE_INDEX:?}");
    // drop_index removes only the named index; it does not delete its field or collection data.
    println!("Calling drop_index: remove {PRICE_INDEX:?}");
    client
        .drop_index(
            DropIndexRequest::builder()
                .collection_name(collection)
                .index_name(PRICE_INDEX)
                .build()?,
        )
        .await?;
    println!("drop_index completed");
    print_indexes(client, collection, "Indexes after dropping price index").await
}

async fn print_indexes(client: &ClientV2, collection: &str, heading: &str) -> Result<()> {
    // list_indexes returns every index defined on the selected collection.
    println!("Calling list_indexes: list indexes on {collection:?}");
    let response = client
        .list_indexes(
            ListIndexesRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("list_indexes completed");
    println!("\n{heading}:");
    for index in response.indexes() {
        println!(
            "  name={:<24} field={:<10} type={:?} metric={:?} state={:?}",
            index.get_index_name(),
            index.get_field_name(),
            index.get_index_type(),
            index.get_metric_type(),
            index.get_state()
        );
    }
    Ok(())
}

async fn cleanup_collection(client: &ClientV2, collection: &str) -> Result<()> {
    // has_collection avoids issuing a drop request when cleanup has already occurred.
    println!("Calling has_collection: check {collection:?}");
    let exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?
        .exists();
    println!("has_collection completed");
    if exists {
        println!("\nDropping tutorial collection {collection:?}");
        // drop_collection removes the tutorial collection and any remaining indexes.
        println!("Calling drop_collection: remove {collection:?}");
        client
            .drop_collection(
                DropCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await?;
        println!("drop_collection completed");
    }
    Ok(())
}

fn tutorial_collection_name() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("RUST_V2_INDEX_{timestamp}_{}", std::process::id())
}
