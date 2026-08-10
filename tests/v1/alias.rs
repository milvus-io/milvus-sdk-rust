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
use milvus::client::Client;
use milvus::error::Result;

#[tokio::test]
async fn create_alter_drop_alias() -> Result<()> {
    let alias0 = gen_random_name();
    let alias1 = gen_random_name();

    let client = Client::new(URL).await?;

    let (_, schema1) = create_test_collection(true).await?;
    let _first_cleanup = CollectionCleanup::new([schema1.name()]);
    let (_, schema2) = create_test_collection(true).await?;
    let collection_names = vec![schema1.name().to_string(), schema2.name().to_string()];

    run_with_collection_cleanup(&client, collection_names, || async {
        client.create_alias(schema1.name(), &alias0).await?;
        assert!(client.has_collection(alias0).await?);

        client.create_alias(schema2.name(), &alias1).await?;

        client.alter_alias(schema1.name(), &alias1).await?;

        client.drop_collection(schema2.name()).await?;
        assert!(client.has_collection(alias1).await?);

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_create_alias() -> Result<()> {
    let alias = format!("test_create_alias_{}", gen_random_name());
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.create_alias(schema.name(), &alias).await?;
        client.drop_alias(&alias).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_alter_alias() -> Result<()> {
    let alias = format!("test_alter_alias_{}", gen_random_name());
    let (client1, schema1) = create_test_collection(true).await?;
    let _first_cleanup = CollectionCleanup::new([schema1.name()]);
    let (client2, schema2) = create_test_collection(true).await?;
    let collection_names = vec![schema1.name().to_string(), schema2.name().to_string()];

    run_with_collection_cleanup(&client1, collection_names, || async {
        client1.create_alias(schema1.name(), &alias).await?;
        client2.alter_alias(schema2.name(), &alias).await?;
        client2.drop_alias(&alias).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_describe_alias() -> Result<()> {
    let alias = format!("test_describe_alias_{}", gen_random_name());
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.create_alias(schema.name(), &alias).await?;
        let (described_alias, collection, db_name) = client.describe_alias(&alias).await?;
        assert_eq!(described_alias, alias);
        assert_eq!(collection, schema.name());
        assert_eq!(db_name, "default");
        client.drop_alias(&described_alias).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn list_aliases() -> Result<()> {
    let alias1 = format!("test_list_alias_1_{}", gen_random_name());
    let alias2 = format!("test_list_alias_2_{}", gen_random_name());
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.create_alias(schema.name(), &alias1).await?;
        client.create_alias(schema.name(), &alias2).await?;

        let (db_name, collection_name, aliases) = client.list_aliases(schema.name()).await?;

        assert_eq!(db_name, "default");
        assert_eq!(collection_name, schema.name());

        let set1: std::collections::HashSet<_> = aliases.iter().collect();
        let vec = vec![alias1.clone(), alias2.clone()];
        let set2: std::collections::HashSet<_> = vec.iter().collect();
        assert_eq!(set1, set2);

        client.drop_alias(&alias1).await?;
        client.drop_alias(&alias2).await?;
        Ok(())
    })
    .await
}
