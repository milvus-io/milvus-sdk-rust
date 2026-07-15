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

#![allow(dead_code)]

use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use rand::Rng;

pub const ID_FIELD: &str = "id";
pub const TEXT_FIELD: &str = "text";
pub const VECTOR_FIELD: &str = "embedding";
pub const DIMENSION: usize = 4;

pub fn float_vector(dimension: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dimension).map(|_| rng.gen_range(0.0..1.0)).collect()
}

pub fn binary_vector(dimension: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..dimension / 8).map(|_| rng.gen()).collect()
}

pub fn int8_vector(dimension: usize) -> Vec<i8> {
    let mut rng = rand::thread_rng();
    (0..dimension).map(|_| rng.gen_range(-128..=127)).collect()
}

pub fn print_query_results(results: &QueryResults) -> Result<()> {
    println!("Query results:");
    for row in results.rows()? {
        println!("\t{:?}", row.to_entity_row()?);
    }
    Ok(())
}

pub fn query_count(results: &QueryResults) -> Result<u64> {
    let row = results
        .rows()?
        .next()
        .ok_or_else(|| Error::Unexpected("count query returned no row".into()))?;
    u64::try_from(row.get_i64("count(*)")?)
        .map_err(|_| Error::Unexpected("count query returned a negative value".into()))
}

pub fn print_search_results(results: &SearchResults) -> Result<()> {
    for result in results {
        println!("Result of one target vector:");
        for row in result.rows()? {
            println!("\t{:?}", row.to_entity_row()?);
        }
    }
    Ok(())
}

pub async fn client() -> Result<ClientV2> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let config = ConnectConfig::new().uri(uri).token("root:Milvus");
    ClientV2::new(&config).await
}

pub async fn flush(client: &ClientV2, collection: &str) -> Result<()> {
    client
        .flush(
            FlushRequest::builder()
                .collection_names([collection])
                .wait_flushed_ms(60_000)
                .build()?,
        )
        .await?;
    Ok(())
}

pub async fn drop_collection(client: &ClientV2, collection: &str) {
    let exists = client
        .has_collection(
            HasCollectionRequest::builder()
                .collection_name(collection)
                .build()
                .expect("valid has-collection request"),
        )
        .await
        .map(|response| response.exists())
        .unwrap_or(false);
    if exists {
        let _ = client
            .drop_collection(
                DropCollectionRequest::builder()
                    .collection_name(collection)
                    .build()
                    .expect("valid drop-collection request"),
            )
            .await;
    }
}
