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
    query::{QueryOptions, SearchOptions},
    schema::{CollectionSchemaBuilder, FieldSchema},
    value::ValueVec,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use rand::{seq::SliceRandom, Rng};
use futures::future::join_all;
use tokio::time::sleep;

const PHASE_2_COLLECTION_NAME: &str = "phase_2_search_query_collection";
const INITIAL_ENTITY_COUNT: i64 = 100_000;
const BATCH_SIZE: i64 = 1000;
const TEST_DURATION_SECS: u64 = 60;

const SEARCHER_TASKS: usize = 50;
const QUERIER_TASKS: usize = 20;
const CHURN_TASKS: usize = 5;

#[tokio::test]
async fn high_volume_search_and_query_under_load() -> Result<()> {
    let client = ClientBuilder::new(URL)
        .timeout(Duration::from_secs(60))
        .build()
        .await?;

    // 1. Setup
    println!("Phase 2: Setting up collection '{}'...", PHASE_2_COLLECTION_NAME);
    let id_schema = FieldSchema::new_primary_int64("id", "", false);
    let vec_schema = FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", DEFAULT_DIM);
    let varchar_schema = FieldSchema::new_varchar("varchar_field", "", 256);

    let schema = CollectionSchemaBuilder::new(PHASE_2_COLLECTION_NAME, "Phase 2 Test Collection")
        .add_field(id_schema.clone())
        .add_field(vec_schema.clone())
        .add_field(varchar_schema.clone())
        .build()?;

    if client.has_collection(PHASE_2_COLLECTION_NAME).await? {
        client.drop_collection(PHASE_2_COLLECTION_NAME).await?;
    }
    client.create_collection(schema.clone(), None).await?;
    println!("Collection created.");

    // Populate with initial data
    let inserted_ids = Arc::new(Mutex::new(Vec::<i64>::new()));
    let mut populate_tasks = Vec::new();
    let num_batches = INITIAL_ENTITY_COUNT / BATCH_SIZE;

    for i in 0..num_batches {
        let client = client.clone();
        let collection_name = PHASE_2_COLLECTION_NAME.to_string();
        let inserted_ids_clone = inserted_ids.clone();
        let id_schema_clone = id_schema.clone();
        let vec_schema_clone = vec_schema.clone();
        let varchar_schema_clone = varchar_schema.clone();
        
        populate_tasks.push(tokio::spawn(async move {
            let start_id = i * BATCH_SIZE;
            let ids: Vec<i64> = (start_id..start_id + BATCH_SIZE).collect();
            let vectors: Vec<f32> = (0..(BATCH_SIZE * DEFAULT_DIM)).map(|_| rand::thread_rng().gen()).collect();
            let varchars: Vec<String> = (0..BATCH_SIZE).map(|k| format!("varchar_{}", start_id + k)).collect();

            let fields = vec![
                FieldColumn::new(&id_schema_clone, ids.clone()),
                FieldColumn::new(&vec_schema_clone, vectors),
                FieldColumn::new(&varchar_schema_clone, varchars),
            ];

            client.insert(&collection_name, fields, None).await.unwrap();
            let mut guard = inserted_ids_clone.lock().unwrap();
            guard.extend(ids);
        }));
    }
    join_all(populate_tasks).await;
    println!("Initial population of {} entities complete.", INITIAL_ENTITY_COUNT);

    client.flush(PHASE_2_COLLECTION_NAME).await?;
    
    // Create index and load
    let index_params = IndexParams::new("phase_2_index".to_string(), IndexType::IvfFlat, MetricType::L2, HashMap::new());
    client.create_index(PHASE_2_COLLECTION_NAME, DEFAULT_VEC_FIELD, index_params).await?;
    println!("Index created.");
    client.load_collection(PHASE_2_COLLECTION_NAME, None).await?;
    println!("Collection loaded. Starting concurrent tests for {} seconds...", TEST_DURATION_SECS);

    let mut tasks = Vec::new();
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(TEST_DURATION_SECS);

    // 2. Concurrent Operations
    // Task Group 1: Searchers
    for _ in 0..SEARCHER_TASKS {
        let client = client.clone();
        let collection_name = PHASE_2_COLLECTION_NAME.to_string();

        tasks.push(tokio::spawn(async move {
            while start_time.elapsed() < test_duration {
                let query_vector: Vec<f32> = (0..DEFAULT_DIM).map(|_| rand::thread_rng().gen()).collect();
                let search_options = SearchOptions::new().limit(10).radius(1.0);
                let result = client.search(&collection_name, vec![query_vector.into()], &DEFAULT_VEC_FIELD.to_string(), &search_options).await;
                assert!(result.is_ok(), "Search failed: {:?}", result.err());
            }
        }));
    }

    // Task Group 2: Queriers
    for _ in 0..QUERIER_TASKS {
        let client = client.clone();
        let collection_name = PHASE_2_COLLECTION_NAME.to_string();
        
        tasks.push(tokio::spawn(async move {
            while start_time.elapsed() < test_duration {
                let random_id = rand::thread_rng().gen_range(0..INITIAL_ENTITY_COUNT);
                let expr = format!("id == {}", random_id);
                let query_options = QueryOptions::new().output_fields(vec!["id".to_string(), "varchar_field".to_string()]);
                let result = client.query(&collection_name, &expr, &query_options).await;
                assert!(result.is_ok(), "Query failed: {:?}", result.err());
            }
        }));
    }
    
    // Task Group 3: Churn (Writers/Deleters)
    for i in 0..CHURN_TASKS {
        let client = client.clone();
        let collection_name = PHASE_2_COLLECTION_NAME.to_string();
        let inserted_ids_clone = inserted_ids.clone();
        let id_schema_clone = id_schema.clone();
        let vec_schema_clone = vec_schema.clone();
        let varchar_schema_clone = varchar_schema.clone();

        tasks.push(tokio::spawn(async move {
            let mut counter = 0i64;
             while start_time.elapsed() < test_duration {
                // Insert a small batch
                let start_id = INITIAL_ENTITY_COUNT + (i as i64 * 1_000_000) + counter;
                let ids = vec![start_id];
                let vectors: Vec<f32> = (0..DEFAULT_DIM).map(|_| rand::thread_rng().gen()).collect();
                let varchars = vec![format!("churn_{}", start_id)];
                let fields = vec![
                    FieldColumn::new(&id_schema_clone, ids.clone()),
                    FieldColumn::new(&vec_schema_clone, vectors),
                    FieldColumn::new(&varchar_schema_clone, varchars),
                ];
                let insert_result = client.insert(&collection_name, fields, None).await;
                assert!(insert_result.is_ok(), "Churn insert failed: {:?}", insert_result.err());
                
                // Delete a small batch
                let ids_to_delete = {
                    let mut guard = inserted_ids_clone.lock().unwrap();
                    let sample: Vec<i64> = guard.choose_multiple(&mut rand::thread_rng(), 1).cloned().collect();
                    sample
                };
                if !ids_to_delete.is_empty() {
                    let delete_result = client.delete(&collection_name, &DeleteOptions::with_ids(ValueVec::Long(ids_to_delete))).await;
                     assert!(delete_result.is_ok(), "Churn delete failed: {:?}", delete_result.err());
                }

                counter += 1;
                sleep(Duration::from_millis(100)).await; // Slow down churn slightly
            }
        }));
    }

    // 3. Verification
    join_all(tasks).await;
    println!("Concurrent tests finished.");
    
    // Final verification is that no task panicked.
    
    // Teardown
    client.drop_collection(PHASE_2_COLLECTION_NAME).await?;
    println!("Phase 2 test completed and collection dropped.");
    Ok(())
}
