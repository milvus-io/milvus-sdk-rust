use std::collections::HashMap;

use milvus::{
    client::Client,
    data::FieldColumn,
    error::Result,
    index::{IndexParams, IndexType, MetricType},
    proto::common::LoadState,
    query::QueryOptions,
    schema::{CollectionSchemaBuilder, FieldSchema},
};

mod common;
use common::*;

#[tokio::test]
async fn alternative_database() -> Result<()> {
    let mut client = Client::new(URL).await?;
    let db_name = format!("test_database_{}", gen_random_name());

    client.create_database(&db_name, HashMap::new()).await?;
    client.use_database(&db_name).await?;

    let collection_name = format!("test_collection_{}", gen_random_name());

    let schema = CollectionSchemaBuilder::new(&collection_name, "")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("vector", "", 2))
        .build()?;

    let column = FieldColumn::new(
        schema.get_field("vector").unwrap(),
        gen_random_f32_vector(2),
    );
    client.create_collection(schema, None).await?;
    client
        .create_index(
            &collection_name,
            "id",
            IndexParams::new(
                "AUTOINDEX".into(),
                IndexType::AUTOINDEX,
                MetricType::None,
                HashMap::new(),
            ),
        )
        .await?;
    client
        .create_index(
            &collection_name,
            "vector",
            IndexParams::new(
                "AUTOINDEX_V".into(),
                IndexType::AUTOINDEX,
                MetricType::COSINE,
                HashMap::new(),
            ),
        )
        .await?;
    client.load_collection(&collection_name, None).await?;
    let state = client.get_load_state(&collection_name, None).await?;
    assert_eq!(state, LoadState::Loaded);

    client.insert(&collection_name, vec![column], None).await?;

    let result = client
        .query(&collection_name, "id > 0", &QueryOptions::default())
        .await?;
    assert_eq!(result.len(), 2, "{:#?}", result);

    client.release_collection(&collection_name).await?;
    let state = client.get_load_state(&collection_name, None).await?;
    assert_eq!(state, LoadState::NotLoad);

    client.drop_collection(&collection_name).await?;
    client.drop_database(&db_name).await?;

    let dbs = client.list_databases().await?;
    assert!(!dbs.contains(&db_name));

    Ok(())
}
