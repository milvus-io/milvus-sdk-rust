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

use super::common::*;
use milvus::{
    client::*, data::FieldColumn, error::Result, query::SearchOptions, schema::CollectionSchema,
    value::Value,
};
use std::borrow::Cow;

fn gen_f32_data(size: i64) -> Vec<f32> {
    let mut data = Vec::<f32>::with_capacity(size as usize);
    for i in 0..size {
        data.push(i as f32);
    }
    data
}

fn gen_i64_data(size: i64) -> Vec<i64> {
    let mut data = Vec::<i64>::with_capacity(size as usize);
    for i in 0..size {
        data.push(i as i64);
    }
    data
}

async fn insert_data(
    client: &Client,
    collection: &CollectionSchema,
    count: i64,
) -> Result<(Vec<i64>, Vec<f32>)> {
    let ids = gen_i64_data(count);
    let vectors = gen_f32_data(count * DEFAULT_DIM);

    let mut fields = Vec::new();
    fields.push(FieldColumn::new(
        collection.get_field("id").unwrap(),
        ids.clone(),
    ));
    fields.push(FieldColumn::new(
        collection.get_field(DEFAULT_VEC_FIELD).unwrap(),
        vectors.clone(),
    ));

    client.insert(collection.name(), fields, None).await?;

    Ok((ids, vectors))
}

#[tokio::test]
async fn test_search() -> Result<()> {
    let (client, collection) = create_test_collection(false).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let (_ids, _vectors) = insert_data(&client, &collection, 10).await?;

        client.load_collection(collection.name(), None).await?;

        let search_vectors = vec![Value::FloatArray(Cow::Owned(vec![
            0.0;
            DEFAULT_DIM as usize
        ]))];

        let search_result = client
            .search(
                collection.name(),
                search_vectors,
                Some(SearchOptions::default().limit(5).add_param(
                    "consistency_level",
                    (milvus::client::ConsistencyLevel::Strong as i32).to_string(),
                )),
            )
            .await?;

        assert_eq!(search_result.len(), 1);
        assert_eq!(search_result[0].score.len(), 5);
        Ok(())
    })
    .await
}
