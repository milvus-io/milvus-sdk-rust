use milvus::client::{Client, ConsistencyLevel};
use milvus::data::FieldColumn;
use milvus::database::CreateDbOptions;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::proto::common::KeyValuePair;
use milvus::options::CreateCollectionOptions;
use milvus::query::{
    AnnSearchRequest, GetOptions, HybridSearchOptions, IdType, QueryOptions, RrfRanker,
    SearchOptions, WeightedRanker,
};
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use std::collections::HashMap;

mod common;
use common::*;

const DIM: i64 = 8;
const ROW_COUNT: i64 = 80;

#[tokio::test]
async fn query_search_hybrid_search_and_get() -> Result<()> {
    let client = Client::new(URL).await?;
    let collection_name = format!("test_query_{}", gen_random_name());

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let schema = CollectionSchemaBuilder::new(&collection_name, "query test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(FieldSchema::new_int64("age", ""))
            .add_field(FieldSchema::new_double("deposit", ""))
            .add_field(FieldSchema::new_float_vector("picture", "", DIM))
            .add_field(FieldSchema::new_float_vector("face", "", DIM))
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let ids: Vec<i64> = (0..ROW_COUNT).collect();
        let ages: Vec<i64> = (0..ROW_COUNT).map(|i| i % 100).collect();
        let deposits: Vec<f64> = (0..ROW_COUNT).map(|i| i as f64).collect();
        let pictures = gen_random_f32_vector_custom(ROW_COUNT, DIM);
        let faces = gen_random_f32_vector_custom(ROW_COUNT, DIM);

        client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("id").unwrap(), ids),
                    FieldColumn::new(schema.get_field("age").unwrap(), ages),
                    FieldColumn::new(schema.get_field("deposit").unwrap(), deposits),
                    FieldColumn::new(schema.get_field("picture").unwrap(), pictures),
                    FieldColumn::new(schema.get_field("face").unwrap(), faces),
                ],
                None,
            )
            .await?;
        client.flush(&collection_name).await?;

        for (field, index_name) in [("picture", "picture_index"), ("face", "face_index")] {
            let index_params = IndexParams::new(
                index_name.to_string(),
                IndexType::IvfFlat,
                MetricType::L2,
                HashMap::from([("nlist".to_string(), "32".to_string())]),
            );
            client
                .create_index(&collection_name, field, index_params)
                .await?;
        }
        client.load_collection(&collection_name, None).await?;

        let query_options = QueryOptions::new()
            .limit(10)
            .output_fields(vec!["id".to_string(), "age".to_string()]);
        let query_result = client
            .query(&collection_name, "age >= 10 and age < 20", &query_options)
            .await?;
        assert!(query_result.iter().any(|field| field.name == "id"));
        assert!(query_result.iter().any(|field| field.name == "age"));
        assert!(query_result.first().is_some_and(|field| field.len() > 0));

        let get_options =
            GetOptions::new().output_fields(vec!["id".to_string(), "deposit".to_string()]);
        let get_result = client
            .get(
                &collection_name,
                IdType::Int64(vec![1, 2, 3]),
                Some(get_options),
            )
            .await?;
        assert!(get_result.iter().any(|field| field.name == "id"));
        assert!(get_result.first().is_some_and(|field| field.len() == 3));

        let search_vector = gen_random_f32_vector_custom(1, DIM);
        let search_options = SearchOptions::with_limit(5)
            .output_fields(vec!["id".to_string(), "age".to_string()])
            .anns_field(vec!["picture".to_string()])
            .add_param("metric_type", "L2")
            .add_param("nprobe", "10");
        let search_result = client
            .search(
                &collection_name,
                vec![Value::FloatArray(search_vector.clone().into())],
                Some(search_options),
            )
            .await?;
        assert_eq!(search_result.len(), 1);
        assert!(search_result[0].size > 0);

        let picture_req = AnnSearchRequest::new(
            vec![Value::FloatArray(search_vector.into())],
            "picture".to_string(),
            vec![
                KeyValuePair {
                    key: "metric_type".to_string(),
                    value: "L2".to_string(),
                },
                KeyValuePair {
                    key: "nprobe".to_string(),
                    value: "10".to_string(),
                },
            ],
            5,
        );
        let face_req = AnnSearchRequest::new(
            vec![Value::FloatArray(
                gen_random_f32_vector_custom(1, DIM).into(),
            )],
            "face".to_string(),
            vec![
                KeyValuePair {
                    key: "metric_type".to_string(),
                    value: "L2".to_string(),
                },
                KeyValuePair {
                    key: "nprobe".to_string(),
                    value: "10".to_string(),
                },
            ],
            5,
        );
        let hybrid_options = SearchOptions::with_limit(5).output_fields(vec!["id".to_string()]);
        let hybrid_result = client
            .hybrid_search(
                &collection_name,
                vec![picture_req, face_req],
                Box::new(WeightedRanker::new(vec![0.5, 0.5])),
                Some(hybrid_options),
            )
            .await?;
        assert_eq!(hybrid_result.len(), 1);
        assert!(hybrid_result[0].size > 0);

        Ok(())
    })
    .await
}

async fn cleanup_consistency_case(
    query_client: &mut Client,
    db_name: &str,
    collection_name: &str,
) -> Result<()> {
    if db_name != "default" {
        query_client.using_database(db_name).await?;
    }
    if query_client.has_collection(collection_name).await.unwrap_or(false) {
        query_client.drop_collection(collection_name).await?;
    }
    if db_name != "default" {
        query_client.using_database("default").await?;
        let databases = query_client.list_databases().await?;
        if databases.iter().any(|db| db == db_name) {
            query_client.drop_database(db_name).await?;
        }
    }
    Ok(())
}

async fn create_consistency_collection(
    query_client: &mut Client,
    db_name: &str,
    collection_name: &str,
    consistency_level: ConsistencyLevel,
) -> Result<CollectionSchema> {
    if db_name != "default" {
        let databases = query_client.list_databases().await?;
        if !databases.iter().any(|db| db == db_name) {
            query_client
                .create_database(db_name, Some(CreateDbOptions::new()))
                .await?;
        }
        query_client.using_database(db_name).await?;
    }

    let schema = CollectionSchemaBuilder::new(collection_name, "consistency test")
        .add_field(FieldSchema::new_primary_int64("pk", "", false))
        .add_field(FieldSchema::new_float_vector("vector", "", 4))
        .build()?;

    query_client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(consistency_level)),
        )
        .await?;
    query_client
        .create_index(
            collection_name,
            "vector",
            IndexParams::new(
                "vector_idx".to_string(),
                IndexType::Flat,
                MetricType::L2,
                HashMap::new(),
            ),
        )
        .await?;
    query_client.load_collection(collection_name, None).await?;

    Ok(schema)
}

async fn run_consistency_visibility_case(
    query_client: &mut Client,
    db_name: &str,
    consistency_level: ConsistencyLevel,
) -> Result<()> {
    let collection_name = format!("test_consistency_{}", gen_random_name());
    let schema =
        create_consistency_collection(query_client, db_name, &collection_name, consistency_level)
            .await?;

    let mut insert_client = query_client.clone();
    if db_name != "default" {
        insert_client.using_database(db_name).await?;
    }
    if db_name == "default" {
        insert_client.using_database("default").await?;
    }

    for i in 0..20_i64 {
        let row_vector = vec![i as f32, i as f32 + 0.1, i as f32 + 0.2, i as f32 + 0.3];
        insert_client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("pk").unwrap(), vec![i]),
                    FieldColumn::new(schema.get_field("vector").unwrap(), row_vector.clone()),
                ],
                None,
            )
            .await?;

        let filter = format!("pk == {}", i);
        if i % 3 == 0 {
            let query_resp = query_client
                .query(
                    &collection_name,
                    &filter,
                    &QueryOptions::new().output_fields(vec!["pk".to_string()]),
                )
                .await?;
            let pk_column = query_resp.iter().find(|field| field.name == "pk").unwrap();
            assert_eq!(pk_column.len(), 1);
        } else if i % 2 == 0 {
            let search_resp = query_client
                .search(
                    &collection_name,
                    vec![Value::FloatArray(row_vector.clone().into())],
                    Some(
                        SearchOptions::with_limit(10)
                            .anns_field(vec!["vector".to_string()])
                            .filter(filter.clone())
                            .add_param("metric_type", "L2"),
                    ),
                )
                .await?;
            assert_eq!(search_resp.len(), 1);
            assert_eq!(search_resp[0].size, 1);
        } else {
            let sub_req = AnnSearchRequest::new(
                vec![Value::FloatArray(row_vector.into())],
                "vector".to_string(),
                vec![KeyValuePair {
                    key: "metric_type".to_string(),
                    value: "L2".to_string(),
                }],
                7,
            )
            .with_expr(filter.clone());
            let search_resp = query_client
                .hybrid_search(
                    &collection_name,
                    vec![sub_req],
                    Box::new(RrfRanker::new(20.0)),
                    Some(HybridSearchOptions::with_limit(5)),
                )
                .await?;
            assert_eq!(search_resp.len(), 1);
            assert_eq!(search_resp[0].size, 1);
        }
    }

    cleanup_consistency_case(query_client, db_name, &collection_name).await?;
    Ok(())
}

#[tokio::test]
async fn consistency_level_visibility_parity() -> Result<()> {
    let mut client = Client::new(URL).await?;
    let temp_db_name = format!("test_level_db_{}", gen_random_name());

    let databases = client.list_databases().await?;
    if !databases.iter().any(|db| db == &temp_db_name) {
        client
            .create_database(&temp_db_name, Some(CreateDbOptions::new()))
            .await?;
    }

    let test_result: Result<()> = async {
        run_consistency_visibility_case(&mut client, "default", ConsistencyLevel::Session).await?;
        run_consistency_visibility_case(&mut client, &temp_db_name, ConsistencyLevel::Session)
            .await?;
        run_consistency_visibility_case(&mut client, "default", ConsistencyLevel::Strong).await?;
        run_consistency_visibility_case(&mut client, &temp_db_name, ConsistencyLevel::Strong)
            .await?;
        Ok(())
    }
    .await;

    client.using_database("default").await?;
    let databases = client.list_databases().await?;
    if databases.iter().any(|db| db == &temp_db_name) {
        client.drop_database(&temp_db_name).await?;
    }

    test_result
}
