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
    client::*, data::FieldColumn, error::Result, mutate::DeleteOptions, schema::CollectionSchema,
};

async fn insert_data(
    client: &Client,
    collection: &CollectionSchema,
    count: i64,
) -> Result<Vec<i64>> {
    let ids = (0..count).collect::<Vec<_>>();
    let vectors = (0..count * DEFAULT_DIM)
        .map(|i| i as f32)
        .collect::<Vec<_>>();

    let mut fields = Vec::new();
    fields.push(FieldColumn::new(
        collection.get_field("id").unwrap(),
        ids.clone(),
    ));
    fields.push(FieldColumn::new(
        collection.get_field(DEFAULT_VEC_FIELD).unwrap(),
        vectors,
    ));

    client.insert(collection.name(), fields, None).await?;

    Ok(ids)
}

#[tokio::test]
async fn test_delete() -> Result<()> {
    let (client, collection) = create_test_collection(false).await?;
    let collection_name = collection.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        let ids = insert_data(&client, &collection, 10).await?;

        client
            .delete(
                collection.name().to_string(),
                &DeleteOptions::with_ids(ids.into()),
            )
            .await?;

        Ok(())
    })
    .await
}
