#[path = "../common/mod.rs"]
mod common;

use common::*;
use milvus::{
    client::*,
    collection::*,
    data::FieldColumn,
    error::Result,
    index::{IndexParams, IndexType, MetricType},
    mutate::{DeleteOptions, InsertOptions},
    query::SearchOptions,
    schema::{CollectionSchemaBuilder, FieldSchema},
    proto::schema::DataType,
    value::{Value, ValueVec},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;
use rand::seq::SliceRandom;
use rand::Rng;
use futures::future::join_all;

const AGGRESSIVE_COLLECTION_NAME: &str = "aggressive_hpc_test_collection";
const BATCH_SIZE: i64 = 1000;
const WRITER_TASKS: usize = 20;
const DELETER_TASKS: usize = 5;
const UPSERTER_TASKS: usize = 5;
const TOTAL_INSERTS_PER_TASK: i64 = 10_000;
const DELETE_BATCH_SIZE: usize = 100;

#[tokio::test]
#[ignore]
async fn high_concurrency_crud_and_indexing() -> Result<()> {
    let client = ClientBuilder::new(URL)
        .timeout(Duration::from_secs(60))
        .build()
        .await?;

    // 1. Setup
    let id_schema = FieldSchema::new_primary_int64("id", "", false);
    let vec_schema = FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", DEFAULT_DIM);
    let varchar_schema = FieldSchema::new_varchar("varchar_field", "", 256);

    let schema = CollectionSchemaBuilder::new(AGGRESSIVE_COLLECTION_NAME, "Aggressive HPC test collection")
        .add_field(id_schema.clone())
        .add_field(vec_schema.clone())
        .add_field(varchar_schema.clone())
        .build()?;

    if client.has_collection(AGGRESSIVE_COLLECTION_NAME).await? {
        client.drop_collection(AGGRESSIVE_COLLECTION_NAME).await?;
    }
    client.create_collection(schema.clone(), None).await?;

    let inserted_ids = Arc::new(Mutex::new(Vec::<i64>::new()));
    let mut tasks = Vec::new();

    // 2. Writers
    for i in 0..WRITER_TASKS {
        let client = client.clone();
        let collection_name = AGGRESSIVE_COLLECTION_NAME.to_string();
        let inserted_ids_clone = inserted_ids.clone();
        let id_schema_clone = id_schema.clone();
        let vec_schema_clone = vec_schema.clone();
        let varchar_schema_clone = varchar_schema.clone();

        tasks.push(tokio::spawn(async move {
            for j in 0..(TOTAL_INSERTS_PER_TASK / BATCH_SIZE) {
                let start_id = (i as i64 * TOTAL_INSERTS_PER_TASK) + (j * BATCH_SIZE);
                let ids: Vec<i64> = (start_id..start_id + BATCH_SIZE).collect();
                let vectors: Vec<f32> = (0..(BATCH_SIZE * DEFAULT_DIM)).map(|_| rand::thread_rng().gen()).collect();
                let varchars: Vec<String> = (0..BATCH_SIZE).map(|k| format!("varchar_{}", start_id + k)).collect();

                let fields = vec![
                    FieldColumn::new(&id_schema_clone, ids.clone()),
                    FieldColumn::new(&vec_schema_clone, vectors),
                    FieldColumn::new(&varchar_schema_clone, varchars),
                ];

                let result = client.insert(&collection_name, fields, None).await;
                if let Err(e) = &result {
                    println!("Insert failed with error: {}", e);
                }
                assert!(result.is_ok());
                
                let mut guard = inserted_ids_clone.lock().unwrap();
                guard.extend(ids);
            }
        }));
    }
    join_all(tasks).await;

    client.flush(AGGRESSIVE_COLLECTION_NAME).await?;
    let mut total_deleted = 0;

    // 3. Deleters
    let mut delete_tasks = Vec::new();
    for _ in 0..DELETER_TASKS {
        let client = client.clone();
        let collection_name = AGGRESSIVE_COLLECTION_NAME.to_string();
        let inserted_ids_clone = inserted_ids.clone();

        delete_tasks.push(tokio::spawn(async move {
            let ids_to_delete = {
                let mut guard = inserted_ids_clone.lock().unwrap();
                let mut rng = rand::thread_rng();
                let sample: Vec<i64> = guard.choose_multiple(&mut rng, DELETE_BATCH_SIZE).cloned().collect();
                sample
            };
            if !ids_to_delete.is_empty() {
                let result = client.delete(&collection_name, &DeleteOptions::with_ids(ValueVec::Long(ids_to_delete))).await;
                assert!(result.is_ok(), "Delete failed: {:?}", result.err());
                result.unwrap().delete_cnt
            } else {
                0
            }
        }));
    }
    let delete_results = join_all(delete_tasks).await;
    for result in delete_results {
        total_deleted += result.unwrap();
    }
    
    // 4. Indexer
    let index_params = IndexParams::new("ivf_flat".to_string(), IndexType::IvfFlat, MetricType::L2, HashMap::new());
    client.create_index(AGGRESSIVE_COLLECTION_NAME, DEFAULT_VEC_FIELD, index_params.clone()).await?;

    // 5. Verification
    client.flush(AGGRESSIVE_COLLECTION_NAME).await?;
    sleep(Duration::from_secs(5)).await; // Give time for stats to update
    let stats = client.get_collection_stats(AGGRESSIVE_COLLECTION_NAME).await?;
    let row_count = stats.get("row_count").unwrap().parse::<i64>().unwrap();
    assert_eq!(row_count, (WRITER_TASKS as i64 * TOTAL_INSERTS_PER_TASK) - total_deleted);

    client.drop_collection(AGGRESSIVE_COLLECTION_NAME).await?;
    Ok(())
}
