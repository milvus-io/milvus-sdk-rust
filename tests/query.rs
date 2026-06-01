use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::proto::common::KeyValuePair;
use milvus::query::{
    AnnSearchRequest, GetOptions, IdType, QueryOptions, SearchOptions, WeightedRanker,
};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
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
