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

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::{Error, Result};
use milvus::v2::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use utils::*;

const COLLECTION: &str = "RUST_V2_DML";
const PRIMARY: &str = "pk";
const VECTOR: &str = "vector";
const TEXT: &str = "text";
const DIMENSION: usize = 4;

async fn print_row_count(client: &ClientV2, level: sdk::ConsistencyLevel) -> Result<()> {
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(level)
                .build()?,
        )
        .await?;
    println!("count(*) = {}", query_count(response.results())?);
    Ok(())
}

async fn build_collection(client: &ClientV2, auto_id: bool) -> Result<Vec<i64>> {
    let schema = sdk::CollectionSchema::new()
        .add_field(
            sdk::FieldSchema::new()
                .name(PRIMARY)
                .description("id")
                .data_type(sdk::DataType::Int64)
                .primary_key(true)
                .auto_id(auto_id),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(VECTOR)
                .data_type(sdk::DataType::FloatVector)
                .dimension(DIMENSION as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(TEXT)
                .data_type(sdk::DataType::VarChar)
                .max_length(100),
        );
    drop_collection(client, COLLECTION).await;
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .build()?,
        )
        .await?;
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_params(vec![
                    IndexParam::new()
                        .field_name(VECTOR)
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::L2)
                        .extra_params(HashMap::from([
                            ("M".into(), "64".into()),
                            ("efConstruction".into(), "200".into()),
                        ])),
                    IndexParam::new()
                        .field_name(TEXT)
                        .index_type(IndexType::Inverted),
                ])
                .build()?,
        )
        .await?;
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;

    let mut columns = vec![
        sdk::FieldData::varchar(TEXT, vec!["column-based-1".into(), "column-based-2".into()]),
        sdk::FieldData::float_vector(
            VECTOR,
            vec![float_vector(DIMENSION), float_vector(DIMENSION)],
        ),
    ];
    if !auto_id {
        columns.push(sdk::FieldData::int64(PRIMARY, vec![10_000, 10_001]));
    }
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .columns(columns)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by column-based.", insert.insert_count());

    let rows: Vec<_> = (0..100)
        .map(|id| {
            let mut row =
                json!({TEXT: format!("hello world {id}"), VECTOR: float_vector(DIMENSION)});
            if !auto_id {
                row[PRIMARY] = json!(id);
            }
            row
        })
        .collect();
    let insert = client
        .insert(
            InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(rows)
                .build()?,
        )
        .await?;
    println!("{} rows inserted by row-based.", insert.insert_count());
    print_row_count(client, sdk::ConsistencyLevel::Strong).await?;
    match insert.ids() {
        Ids::Int64(ids) => Ok(ids.clone()),
        _ => Err(Error::Unexpected("expected integer primary keys".into())),
    }
}

fn filter_for(ids: &[i64]) -> String {
    format!(
        "pk in [{}]",
        ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
    )
}

async fn query(client: &ClientV2, filter: &str, level: sdk::ConsistencyLevel) -> Result<()> {
    println!("Query with expression: {filter}");
    let response = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .output_fields([PRIMARY, TEXT, VECTOR])
                .consistency_level(level)
                .build()?,
        )
        .await?;
    println!("Query result count: {}", response.results().get_row_count());
    print_query_results(response.results())
}

async fn do_dml(client: &ClientV2, auto_id: bool) -> Result<()> {
    println!("\n================== auto_id: {auto_id} ==================");
    let ids = build_collection(client, auto_id).await?;
    let old_ids = [ids[1], *ids.last().expect("insert returned IDs")];
    let rows: Vec<_> = old_ids
        .iter()
        .map(|id| {
            json!({PRIMARY: id, TEXT: format!("this row is updated from {id}"), VECTOR: vec![0.88; DIMENSION]})
        })
        .collect();
    let upsert = client
        .upsert(
            UpsertRequest::builder()
                .insert(
                    InsertRequest::builder()
                        .collection_name(COLLECTION)
                        .rows(rows)
                        .build()?,
                )
                .build()?,
        )
        .await?;
    let new_ids = match upsert.ids() {
        Ids::Int64(ids) => ids.clone(),
        _ => return Err(Error::Unexpected("expected integer upsert IDs".into())),
    };
    for (old, new) in old_ids.iter().zip(&new_ids) {
        println!("After upsert, the id {old} has been updated to {new}");
    }
    let mut filter = filter_for(&new_ids);
    query(client, &filter, sdk::ConsistencyLevel::Session).await?;
    print_row_count(client, sdk::ConsistencyLevel::Eventually).await?;

    let partial_rows: Vec<_> = new_ids
        .iter()
        .map(|id| json!({PRIMARY: id, TEXT: "this item is partial updated"}))
        .collect();
    let partial = client
        .upsert(
            UpsertRequest::builder()
                .insert(
                    InsertRequest::builder()
                        .collection_name(COLLECTION)
                        .rows(partial_rows)
                        .build()?,
                )
                .partial_update(true)
                .build()?,
        )
        .await?;
    let updated_ids = match partial.ids() {
        Ids::Int64(ids) => ids.clone(),
        _ => {
            return Err(Error::Unexpected(
                "expected integer partial-upsert IDs".into(),
            ))
        }
    };
    for (old, new) in new_ids.iter().zip(&updated_ids) {
        println!("After partial upsert, the id {old} has been updated to {new}");
    }
    filter = filter_for(&updated_ids);
    query(client, &filter, sdk::ConsistencyLevel::Session).await?;
    print_row_count(client, sdk::ConsistencyLevel::Eventually).await?;

    println!("Delete with expression: {filter}");
    client
        .delete(
            DeleteRequest::builder()
                .collection_name(COLLECTION)
                .filter(&filter)
                .build()?,
        )
        .await?;
    query(client, &filter, sdk::ConsistencyLevel::Session).await?;
    print_row_count(client, sdk::ConsistencyLevel::Eventually).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    do_dml(&client, true).await?;
    do_dml(&client, false).await?;
    drop_collection(&client, COLLECTION).await;
    Ok(())
}
