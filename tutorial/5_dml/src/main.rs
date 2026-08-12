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

const DIMENSION: usize = 4;

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".to_owned());
    let collection = tutorial_collection_name();
    // ClientV2::new connects to the Milvus endpoint and authenticates all following DML requests.
    println!("Calling ClientV2::new: connect to Milvus");
    let client = ClientV2::new(&ConnectConfig::new().uri(uri).token(token)).await?;
    println!("ClientV2::new completed");

    let tutorial_result = demonstrate_dml(&client, &collection).await;
    let cleanup_result = cleanup_collection(&client, &collection).await;
    if let Err(error) = &cleanup_result {
        eprintln!("Failed to clean up {collection:?}: {error}");
    }
    tutorial_result?;
    cleanup_result
}

async fn demonstrate_dml(client: &ClientV2, collection: &str) -> Result<()> {
    create_and_load_collection(client, collection).await?;

    // insert writes row-oriented JSON entities. `collection_name` selects the destination and each
    // `row` must follow its schema; the response reports how many entities were accepted.
    println!("Calling insert: write two row-oriented entities");
    let inserted = client
        .insert(
            InsertRequest::builder()
                .collection_name(collection)
                .row(json!({
                    "id": 1,
                    "title": "Rust in Action",
                    "price": 35.0,
                    "embedding": [0.1, 0.2, 0.3, 0.4]
                }))
                .row(json!({
                    "id": 2,
                    "title": "Vector Search",
                    "price": 25.0,
                    "embedding": [0.2, 0.3, 0.4, 0.5]
                }))
                .build()?,
        )
        .await?;
    println!("insert completed");
    println!("Inserted {} rows with row input", inserted.insert_count());

    // insert also accepts column-oriented FieldData. Every column names a schema field and all
    // columns must contain the same non-zero number of values.
    println!("Calling insert: write two column-oriented entities");
    let inserted = client
        .insert(
            InsertRequest::builder()
                .collection_name(collection)
                .columns(vec![
                    FieldData::int64("id", vec![3, 4]),
                    FieldData::varchar(
                        "title",
                        vec!["Milvus Guide".to_owned(), "Database Systems".to_owned()],
                    ),
                    FieldData::float("price", vec![30.0, 40.0]),
                    FieldData::float_vector(
                        "embedding",
                        vec![vec![0.3, 0.4, 0.5, 0.6], vec![0.4, 0.5, 0.6, 0.7]],
                    ),
                ])
                .build()?,
        )
        .await?;
    println!("insert completed");
    println!(
        "Inserted {} rows with column input",
        inserted.insert_count()
    );

    // upsert replaces an existing entity or inserts it when the primary key is absent. The nested
    // InsertRequest supplies the collection and complete replacement row.
    println!("Calling upsert: fully replace entity id=2");
    let upserted = client
        .upsert(
            UpsertRequest::builder()
                .insert(
                    InsertRequest::builder()
                        .collection_name(collection)
                        .row(json!({
                            "id": 2,
                            "title": "Practical Vector Search",
                            "price": 27.5,
                            "embedding": [0.25, 0.35, 0.45, 0.55]
                        }))
                        .build()?,
                )
                .build()?,
        )
        .await?;
    println!("upsert completed");
    println!("Fully upserted {} row", upserted.upsert_count());

    // upsert with `partial_update(true)` changes only supplied non-primary fields. The primary key
    // still identifies which entity to update.
    println!("Calling upsert: partially update entity id=3");
    let upserted = client
        .upsert(
            UpsertRequest::builder()
                .insert(
                    InsertRequest::builder()
                        .collection_name(collection)
                        .row(json!({"id": 3, "title": "The Milvus Guide"}))
                        .build()?,
                )
                .partial_update(true)
                .build()?,
        )
        .await?;
    println!("upsert completed");
    println!("Partially upserted {} row", upserted.upsert_count());

    // delete with `ids` removes entities by primary key from the selected collection.
    println!("Calling delete: remove entity id=1");
    let deleted = client
        .delete(
            DeleteRequest::builder()
                .collection_name(collection)
                .ids(Ids::Int64(vec![1]))
                .build()?,
        )
        .await?;
    println!("delete completed");
    println!("Deleted {} row by primary key", deleted.delete_count());

    // delete with `filter` removes all entities matching the Milvus boolean expression.
    println!("Calling delete: remove entities matching id >= 4");
    let deleted = client
        .delete(
            DeleteRequest::builder()
                .collection_name(collection)
                .filter("id >= 4")
                .build()?,
        )
        .await?;
    println!("delete completed");
    println!("Deleted {} row by filter", deleted.delete_count());

    print_remaining_rows(client, collection).await
}

async fn create_and_load_collection(client: &ClientV2, collection: &str) -> Result<()> {
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
                .name("title")
                .data_type(DataType::VarChar)
                .max_length(256),
        )
        .add_field(FieldSchema::new().name("price").data_type(DataType::Float))
        .add_field(
            FieldSchema::new()
                .name("embedding")
                .data_type(DataType::FloatVector)
                .dimension(DIMENSION as u32),
        );
    // create_collection creates the DML target. `schema` defines valid input fields and
    // `index_param` makes the vector field searchable.
    println!("Calling create_collection: create {collection:?}");
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection)
                .schema(schema)
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
    // load_collection prepares the collection for the verification query. `sync(true)` waits until
    // serving is ready, bounded by `timeout_ms`.
    println!("Calling load_collection: load {collection:?}");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection)
                .sync(true)
                .timeout_ms(60_000)
                .build()?,
        )
        .await?;
    println!("load_collection completed");
    Ok(())
}

async fn print_remaining_rows(client: &ClientV2, collection: &str) -> Result<()> {
    // query reads the remaining entities. `filter` selects rows, `output_fields` chooses returned
    // columns, and Strong consistency makes the preceding mutations visible.
    println!("Calling query: verify remaining rows");
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(collection)
                .filter("id >= 0")
                .output_fields(["id", "title", "price"])
                .consistency_level(ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("query completed");
    println!("\nRemaining rows:");
    for row in response.results().rows()? {
        println!(
            "  id={}, title={:?}, price={}",
            row.get_i64("id")?,
            row.get_str("title")?,
            row.get_f32("price")?
        );
    }
    Ok(())
}

async fn cleanup_collection(client: &ClientV2, collection: &str) -> Result<()> {
    // has_collection checks whether cleanup is necessary.
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
        // release_collection is best-effort cleanup that removes the collection from serving memory.
        println!("Calling release_collection: cleanup {collection:?}");
        let release_result = client
            .release_collection(
                ReleaseCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await;
        println!(
            "release_collection completed: {}",
            if release_result.is_ok() {
                "ok"
            } else {
                "ignored error"
            }
        );
        // drop_collection permanently removes the tutorial collection and its remaining data.
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
    format!("RUST_V2_DML_{timestamp}_{}", std::process::id())
}
