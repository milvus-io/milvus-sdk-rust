mod common;
use common::*;
use milvus::client::Client;
use milvus::error::Result;

#[tokio::test]
async fn create_alter_drop_alias() -> Result<()> {
    let alias0 = gen_random_name();
    let alias1 = gen_random_name();

    let client = Client::new(URL).await?;

    let (_, schema1) = create_test_collection(true).await?;
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
    let alias = "test_create_alias";
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.create_alias(schema.name(), alias).await?;
        client.drop_alias(alias).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_alter_alias() -> Result<()> {
    let alias = "test_alter_alias";
    let (client1, schema1) = create_test_collection(true).await?;
    let (client2, schema2) = create_test_collection(true).await?;
    let collection_names = vec![schema1.name().to_string(), schema2.name().to_string()];

    run_with_collection_cleanup(&client1, collection_names, || async {
        client1.create_alias(schema1.name(), alias).await?;
        client2.alter_alias(schema2.name(), alias).await?;
        client2.drop_alias(alias).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_describe_alias() -> Result<()> {
    let alias = "test_describe_alias";
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.create_alias(schema.name(), alias).await?;
        let (alias, collection, db_name) = client.describe_alias(alias).await?;
        assert_eq!(alias, "test_describe_alias");
        assert_eq!(collection, schema.name());
        assert_eq!(db_name, "default");
        client.drop_alias(alias).await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn list_aliases() -> Result<()> {
    let alias1 = "test_list_alias_1";
    let alias2 = "test_list_alias_2";
    let (client, schema) = create_test_collection(true).await?;
    let collection_name = schema.name().to_string();

    run_with_collection_cleanup(&client, vec![collection_name], || async {
        client.create_alias(schema.name(), alias1).await?;
        client.create_alias(schema.name(), alias2).await?;

        let (db_name, collection_name, aliases) = client.list_aliases(schema.name()).await?;

        assert_eq!(db_name, "default");
        assert_eq!(collection_name, schema.name());

        // the result is not in order,so transfer to hashset
        let set1: std::collections::HashSet<_> = aliases.iter().collect();
        let vec = vec![alias1.to_string(), alias2.to_string()];
        let set2: std::collections::HashSet<_> = vec.iter().collect();
        assert_eq!(set1, set2);

        client.drop_alias(alias1).await?;
        client.drop_alias(alias2).await?;
        Ok(())
    })
    .await
}
