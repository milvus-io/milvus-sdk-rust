// Integration tests for Milvus 2.6 features.
// These tests require a running Milvus 2.6+ server.

use milvus::client::*;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::CreateCollectionOptions;
use milvus::query::SearchOptions;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::ValueVec;
use std::collections::HashMap;

mod common;
use common::*;

// -- New data type tests --

#[tokio::test]
async fn int8_vector_collection() -> Result<()> {
    let collection_name = format!("test_int8vec_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let dim: i64 = 8;

    let schema = CollectionSchemaBuilder::new(&collection_name, "int8 vector test")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_int8_vector("embedding", "", dim))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    assert!(client.has_collection(&collection_name).await?);

    // Insert int8 vector data (represented as bytes, dim values per vector)
    let num_rows: usize = 10;
    let id_data: Vec<i64> = (0..num_rows as i64).collect();
    let id_col = FieldColumn::new(schema.get_field("id").unwrap(), id_data);

    // Int8Vector is stored as Binary(Vec<u8>) with dim bytes per vector
    let mut vec_data: Vec<u8> = Vec::with_capacity(num_rows * dim as usize);
    for i in 0..num_rows {
        for d in 0..dim as usize {
            vec_data.push(((i * dim as usize + d) % 128) as u8);
        }
    }
    let vec_col = FieldColumn::new(schema.get_field("embedding").unwrap(), vec_data);

    client
        .insert(&collection_name, vec![id_col, vec_col], None)
        .await?;

    client.drop_collection(&collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn timestamptz_field() -> Result<()> {
    let collection_name = format!("test_tstz_{}", gen_random_name());
    let client = Client::new(URL).await?;

    let schema = CollectionSchemaBuilder::new(&collection_name, "timestamptz test")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .add_field(FieldSchema::new_timestamptz("created_at", ""))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    assert!(client.has_collection(&collection_name).await?);

    // Insert data with timestamptz
    let ids: Vec<i64> = vec![1, 2, 3];
    let id_col = FieldColumn::new(schema.get_field("id").unwrap(), ids);
    let vecs: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
    let vec_col = FieldColumn::new(schema.get_field("embedding").unwrap(), vecs);
    // Timestamps in microseconds
    let timestamps: Vec<i64> = vec![1700000000000000, 1700000001000000, 1700000002000000];
    let ts_col = FieldColumn::new(schema.get_field("created_at").unwrap(), timestamps);

    client
        .insert(&collection_name, vec![id_col, vec_col, ts_col], None)
        .await?;

    client.drop_collection(&collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn float16_vector_collection() -> Result<()> {
    let collection_name = format!("test_f16vec_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let dim: i64 = 4;

    let schema = CollectionSchemaBuilder::new(&collection_name, "float16 vector test")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float16_vector("embedding", "", dim))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    assert!(client.has_collection(&collection_name).await?);
    client.drop_collection(&collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn bfloat16_vector_collection() -> Result<()> {
    let collection_name = format!("test_bf16vec_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let dim: i64 = 4;

    let schema = CollectionSchemaBuilder::new(&collection_name, "bfloat16 vector test")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_bfloat16_vector("embedding", "", dim))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    assert!(client.has_collection(&collection_name).await?);
    client.drop_collection(&collection_name).await?;
    Ok(())
}

// -- New RPC tests --

#[tokio::test]
async fn truncate_collection() -> Result<()> {
    let (client, schema) = create_test_collection(true).await?;

    // Verify data exists
    let options =
        milvus::query::QueryOptions::default().output_fields(vec![String::from("count(*)")]);
    let result = client.query(schema.name(), "", &options).await?;
    if let ValueVec::Long(vec) = &result[0].value {
        assert!(*vec.first().unwrap() > 0, "collection should have data");
    }

    // Truncate
    client.truncate_collection(schema.name()).await?;

    // After truncate, data should eventually be gone (may take a moment)
    // We just verify the RPC succeeded without error
    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn batch_describe_collections() -> Result<()> {
    let (client, schema1) = create_test_collection(true).await?;
    let (_, schema2) = create_test_collection(true).await?;

    let collections = client
        .batch_describe_collections(vec![schema1.name().to_string(), schema2.name().to_string()])
        .await?;

    assert_eq!(collections.len(), 2);

    client.drop_collection(schema1.name()).await?;
    client.drop_collection(schema2.name()).await?;
    Ok(())
}

#[tokio::test]
async fn add_collection_field_schema_evolution() -> Result<()> {
    let collection_name = format!("test_schema_evo_{}", gen_random_name());
    let client = Client::new(URL).await?;

    let schema = CollectionSchemaBuilder::new(&collection_name, "schema evolution test")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    let new_field = FieldSchema::new_varchar("description", "added later", 256).set_nullable(true);

    client
        .add_collection_field(&collection_name, new_field)
        .await?;

    let updated = client.describe_collection(&collection_name).await?;
    assert!(updated
        .schema
        .fields
        .iter()
        .any(|field| field.name == "description" && field.nullable),);

    client.drop_collection(&collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn search_with_cosine_metric() -> Result<()> {
    let collection_name = format!("test_cosine_{}", gen_random_name());
    let client = Client::new(URL).await?;
    let dim: i64 = 16;

    let schema = CollectionSchemaBuilder::new(&collection_name, "cosine test")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("embedding", "", dim))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Insert data
    let embed_data = gen_random_f32_vector_custom(100, dim);
    let embed_col = FieldColumn::new(schema.get_field("embedding").unwrap(), embed_data);
    client
        .insert(&collection_name, vec![embed_col], None)
        .await?;
    client.flush(&collection_name).await?;

    // Create index with COSINE metric
    let index_params = IndexParams::new(
        "cosine_idx".to_owned(),
        IndexType::HNSW,
        MetricType::COSINE,
        HashMap::from([
            ("M".to_owned(), "16".to_owned()),
            ("efConstruction".to_owned(), "64".to_owned()),
        ]),
    );
    client
        .create_index(&collection_name, "embedding", index_params)
        .await?;

    client.load_collection(&collection_name, None).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Search with COSINE
    let query_vec = gen_random_f32_vector_custom(1, dim);
    let option = SearchOptions::with_limit(5)
        .output_fields(vec!["id".to_owned()])
        .add_param("ef", "64");

    let result = client
        .search(&collection_name, vec![query_vec.into()], Some(option))
        .await?;

    assert!(!result.is_empty());
    assert!(result[0].size > 0);

    client.drop_collection(&collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn upsert_with_partial_update() -> Result<()> {
    let collection_name = format!("test_partial_{}", gen_random_name());
    let client = Client::new(URL).await?;

    let schema = CollectionSchemaBuilder::new(&collection_name, "partial upsert test")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .add_field(FieldSchema::new_varchar("name", "", 256))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Initial insert
    let ids: Vec<i64> = vec![1, 2, 3];
    let id_col = FieldColumn::new(schema.get_field("id").unwrap(), ids.clone());
    let vecs: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
    let vec_col = FieldColumn::new(schema.get_field("embedding").unwrap(), vecs);
    let names: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
    let name_col = FieldColumn::new(schema.get_field("name").unwrap(), names);

    client
        .insert(&collection_name, vec![id_col, vec_col, name_col], None)
        .await?;

    // Partial upsert - only update the name field
    let upsert_ids: Vec<i64> = vec![1, 2];
    let id_col = FieldColumn::new(schema.get_field("id").unwrap(), upsert_ids);
    let new_names: Vec<String> = vec!["aa".into(), "bb".into()];
    let name_col = FieldColumn::new(schema.get_field("name").unwrap(), new_names);

    let upsert_options = milvus::mutate::UpsertOptions::new().partial_update(true);
    client
        .upsert(
            &collection_name,
            vec![id_col, name_col],
            Some(upsert_options),
        )
        .await?;

    client.drop_collection(&collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn index_type_inverted() -> Result<()> {
    let collection_name = format!("test_inverted_{}", gen_random_name());
    let client = Client::new(URL).await?;

    let schema = CollectionSchemaBuilder::new(&collection_name, "inverted index test")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .add_field(FieldSchema::new_varchar("category", "", 128))
        .build()?;

    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Create INVERTED index on varchar field
    let index_params = IndexParams::new(
        "category_idx".to_owned(),
        IndexType::Inverted,
        MetricType::L2, // metric type not used for scalar index
        HashMap::new(),
    );
    client
        .create_index(&collection_name, "category", index_params)
        .await?;

    client.drop_collection(&collection_name).await?;
    Ok(())
}
