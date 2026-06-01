use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::proto::common::KeyValuePair;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn index_management_lifecycle() -> Result<()> {
    let client = Client::new(URL).await?;
    let collection_name = format!("test_index_{}", gen_random_name());

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let vector_field = "embedding".to_string();
        let scalar_field = "category".to_string();
        let vector_index = "embedding_index".to_string();
        let scalar_index = "category_index".to_string();

        let schema = CollectionSchemaBuilder::new(&collection_name, "index test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(FieldSchema::new_float_vector("embedding", "", 8))
            .add_field(FieldSchema::new_varchar("category", "", 128))
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let row_count = 30;
        let ids: Vec<i64> = (0..row_count).collect();
        let vectors = gen_random_f32_vector_custom(row_count, 8);
        let categories: Vec<String> = (0..row_count).map(|i| format!("cat_{}", i % 3)).collect();

        client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("id").unwrap(), ids),
                    FieldColumn::new(schema.get_field("embedding").unwrap(), vectors),
                    FieldColumn::new(schema.get_field("category").unwrap(), categories),
                ],
                None,
            )
            .await?;
        client.flush(&collection_name).await?;

        let vector_params = IndexParams::new(
            vector_index.clone(),
            IndexType::IvfFlat,
            MetricType::L2,
            HashMap::from([("nlist".to_string(), "32".to_string())]),
        );
        client
            .create_index(collection_name.clone(), vector_field.clone(), vector_params)
            .await?;

        let scalar_params = IndexParams::new(
            scalar_index.clone(),
            IndexType::Trie,
            MetricType::L2,
            HashMap::new(),
        );
        client
            .create_index(collection_name.clone(), scalar_field, scalar_params)
            .await?;

        let all_indexes = client.list_indexes(&collection_name, None).await?;
        assert!(all_indexes.contains(&vector_index));
        assert!(all_indexes.contains(&scalar_index));

        let vector_indexes = client
            .list_indexes(collection_name.clone(), Some(vector_field.clone()))
            .await?;
        assert_eq!(vector_indexes, vec![vector_index.clone()]);

        let descriptions = client
            .describe_index(collection_name.clone(), vector_field)
            .await?;
        assert!(descriptions
            .iter()
            .any(|index| index.params().name() == &vector_index));

        client
            .alter_index_properties(
                collection_name.clone(),
                vector_index.clone(),
                vec![KeyValuePair {
                    key: "mmap.enabled".to_string(),
                    value: "false".to_string(),
                }],
            )
            .await?;
        client
            .drop_index_properties(
                collection_name.clone(),
                vector_index.clone(),
                vec!["mmap.enabled".to_string()],
            )
            .await?;

        client
            .drop_index(collection_name.clone(), scalar_index)
            .await?;
        client
            .drop_index(collection_name.clone(), vector_index)
            .await?;

        Ok(())
    })
    .await
}
