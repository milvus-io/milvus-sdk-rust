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

use super::super::common::*;
use milvus::{
    client::*,
    data::FieldColumn,
    error::Result,
    index::{IndexParams, IndexType, MetricType},
    mutate::{DeleteOptions, InsertOptions},
    proto,
    query::{QueryOptions, SearchOptions},
    schema::{CollectionSchemaBuilder, FieldSchema},
    value::ValueVec,
};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;

const SOAK_TEST_DURATION_MINS: u64 = 60;
const SOAK_COLLECTION_NAME: &str = "soak_test_collection";

#[tokio::test]
#[ignore = "60-minute soak test is intended for explicit stability runs"]
async fn soak_test_for_stability_and_memory_leaks() -> Result<()> {
    let client = ClientBuilder::new(URL)
        .timeout(Duration::from_secs(60))
        .build()
        .await?;

    let test_duration = Duration::from_secs(SOAK_TEST_DURATION_MINS * 60);
    let start_time = tokio::time::Instant::now();
    let mut cycle_count = 0;

    println!(
        "Starting soak test for {} minutes...",
        SOAK_TEST_DURATION_MINS
    );

    while start_time.elapsed() < test_duration {
        cycle_count += 1;
        println!("\n--- Starting Soak Test Cycle {} ---", cycle_count);

        let collection_name = format!("{}_{}", SOAK_COLLECTION_NAME, cycle_count);
        let mut cleanup = CollectionCleanup::new([&collection_name]);
        let partition_name = "soak_partition".to_string();

        // Wrap a full cycle in an error-checked block
        let cycle_result = run_full_sdk_cycle(&client, &collection_name, &partition_name).await;

        if let Err(e) = cycle_result {
            eprintln!(
                "Soak test cycle {} failed with error: {:?}. Aborting test.",
                cycle_count, e
            );
            // In a real CI/CD, this would fail the test. For local runs, we panic.
            panic!("Soak test cycle failed.");
        } else {
            println!(
                "--- Soak Test Cycle {} Completed Successfully ---",
                cycle_count
            );
        }

        // Clean up collection even if it failed, to avoid leaving residue
        if client.has_collection(&collection_name).await? {
            client.drop_collection(&collection_name).await?;
            println!("Cleaned up collection '{}'", collection_name);
        }
        cleanup.disarm();

        sleep(Duration::from_secs(5)).await; // Small delay between cycles
    }

    println!(
        "\nSoak test completed successfully after {} minutes and {} cycles.",
        SOAK_TEST_DURATION_MINS, cycle_count
    );

    Ok(())
}

async fn run_full_sdk_cycle(
    client: &Client,
    collection_name: &str,
    partition_name: &str,
) -> Result<()> {
    // 1. Create Collection & Partition
    let id_schema = FieldSchema::new_primary_int64("id", "", true);
    let vec_schema = FieldSchema::new_float_vector(DEFAULT_VEC_FIELD, "", DEFAULT_DIM);
    let schema = CollectionSchemaBuilder::new(collection_name, "Soak Test Collection")
        .add_field(id_schema.clone())
        .add_field(vec_schema.clone())
        .build()?;

    client.create_collection(schema.clone(), None).await?;
    println!("Cycle: Created collection '{}'", collection_name);

    client
        .create_partition(collection_name.to_string(), partition_name.to_string())
        .await?;
    println!("Cycle: Created partition '{}'", partition_name);

    // 2. Insert Data
    let insert_count = 1000;
    let vectors: Vec<f32> = (0..(insert_count * DEFAULT_DIM))
        .map(|_| rand::random())
        .collect();
    let fields = vec![FieldColumn::new(&vec_schema, vectors)];
    let insert_result = client
        .insert(
            collection_name,
            fields,
            Some(InsertOptions::new().partition_name(partition_name.to_string())),
        )
        .await?;
    println!("Cycle: Inserted {} entities", insert_count);
    client.flush(collection_name).await?;

    // 3. Create Index & Load
    let index_params = IndexParams::new(
        "soak_index".to_string(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::new(),
    );
    client
        .create_index(collection_name, DEFAULT_VEC_FIELD, index_params)
        .await?;
    println!("Cycle: Created index");

    client.load_collection(collection_name, None).await?;
    println!("Cycle: Loaded collection");

    // 4. Search & Query
    let query_vector: Vec<f32> = (0..DEFAULT_DIM).map(|_| rand::random()).collect();
    client
        .search(
            collection_name,
            vec![query_vector.into()],
            Some(
                SearchOptions::new()
                    .limit(10)
                    .add_param("anns_field", DEFAULT_VEC_FIELD),
            ),
        )
        .await?;
    println!("Cycle: Performed search");

    let ids = match insert_result.i_ds {
        Some(proto::schema::IDs {
            id_field: Some(proto::schema::i_ds::IdField::IntId(proto::schema::LongArray { data })),
        }) => data,
        _ => {
            return Err(milvus::error::Error::Unexpected(
                "Expected int IDs".to_string(),
            ))
        }
    };
    let ids_to_query = &ids[0..5];
    let expr = format!("id in {:?}", ids_to_query);
    client
        .query(collection_name, &expr, &QueryOptions::new())
        .await?;
    println!("Cycle: Performed query");

    // 5. Release & Drop Index
    client.release_collection(collection_name).await?;
    println!("Cycle: Released collection");

    client
        .drop_index(collection_name, DEFAULT_VEC_FIELD)
        .await?;
    println!("Cycle: Dropped index");

    // 6. Delete
    let ids_to_delete = &ids[10..20];
    client
        .delete(
            collection_name,
            &DeleteOptions::with_ids(ValueVec::Long(ids_to_delete.to_vec())),
        )
        .await?;
    println!("Cycle: Deleted entities");

    // 7. Drop Partition
    client
        .drop_partition(collection_name.to_string(), partition_name.to_string())
        .await?;
    println!("Cycle: Dropped partition");

    Ok(())
}
