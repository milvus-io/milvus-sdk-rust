use milvus::client::Client;
use milvus::database::CreateDbOptions;
use milvus::error::Result;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};

mod common;
use common::*;

#[tokio::test]
async fn database_lifecycle_and_context_switching() -> Result<()> {
    let mut client = Client::new(URL).await?;
    let db_a = format!("test_db_a_{}", gen_random_name());
    let db_b = format!("test_db_b_{}", gen_random_name());
    let collection_a = format!("test_collection_a_{}", gen_random_name());
    let collection_b = format!("test_collection_b_{}", gen_random_name());

    let test_result: Result<()> = async {
        let db_options = CreateDbOptions::new().replica_number(1).max_collections(3);
        client
            .create_database(&db_a, Some(db_options.clone()))
            .await?;
        client.create_database(&db_b, Some(db_options)).await?;

        let databases = client.list_databases().await?;
        assert!(databases.contains(&db_a));
        assert!(databases.contains(&db_b));

        let db_info = client.describe_database(&db_a).await?;
        assert_eq!(db_info.db_name, db_a);
        assert!(db_info.properties.iter().any(|property| {
            property.key == "database.max.collections" && property.value == "3"
        }));

        client
            .alter_database_properties(
                &db_a,
                CreateDbOptions::new().replica_number(1).max_collections(2),
            )
            .await?;
        let db_info = client.describe_database(&db_a).await?;
        assert!(db_info.properties.iter().any(|property| {
            property.key == "database.max.collections" && property.value == "2"
        }));

        client
            .drop_database_properties(&db_a, vec!["database.max.collections".to_string()])
            .await?;

        client.using_database(&db_a).await?;
        let schema_a = CollectionSchemaBuilder::new(&collection_a, "database test")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(FieldSchema::new_float_vector("vector", "", 4))
            .build()?;
        client.create_collection(schema_a, None).await?;
        let collections = client.list_collections().await?;
        assert!(collections.contains(&collection_a));

        client.using_database(&db_b).await?;
        let schema_b = CollectionSchemaBuilder::new(&collection_b, "database test")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(FieldSchema::new_float_vector("vector", "", 4))
            .build()?;
        client.create_collection(schema_b, None).await?;
        let collections = client.list_collections().await?;
        assert!(collections.contains(&collection_b));

        Ok(())
    }
    .await;

    let mut cleanup_error = None;

    if let Err(error) = client.using_database(&db_b).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    } else if let Err(error) = client.drop_collection(&collection_b).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }

    if let Err(error) = client.using_database(&db_a).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    } else if let Err(error) = client.drop_collection(&collection_a).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }

    if let Err(error) = client.using_database("default").await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = client.drop_database(&db_a).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = client.drop_database(&db_b).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }

    test_result?;
    if let Some(error) = cleanup_error {
        return Err(error);
    }

    let databases = client.list_databases().await?;
    assert!(!databases.contains(&db_a));
    assert!(!databases.contains(&db_b));

    Ok(())
}
