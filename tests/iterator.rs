use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::iterator::{QueryIteratorOptions, SearchIteratorOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use std::collections::HashMap;

mod common;
use common::*;

const DIM: i64 = 8;
const ROW_COUNT: i64 = 120;

#[tokio::test]
async fn query_and_search_iterators_return_pages() -> Result<()> {
    let client = Client::new(URL).await?;
    let collection_name = format!("test_iterator_{}", gen_random_name());

    run_with_collection_cleanup(&client, vec![collection_name.clone()], || async {
        let schema = CollectionSchemaBuilder::new(&collection_name, "iterator test")
            .add_field(FieldSchema::new_primary_int64("id", "", false))
            .add_field(FieldSchema::new_int64("age", ""))
            .add_field(FieldSchema::new_double("deposit", ""))
            .add_field(FieldSchema::new_float_vector("picture", "", DIM))
            .build()?;
        client.create_collection(schema.clone(), None).await?;

        let ids: Vec<i64> = (0..ROW_COUNT).collect();
        let ages: Vec<i64> = (0..ROW_COUNT).map(|i| i % 100).collect();
        let deposits: Vec<f64> = (0..ROW_COUNT).map(|i| i as f64).collect();
        let pictures = gen_random_f32_vector_custom(ROW_COUNT, DIM);

        client
            .insert(
                &collection_name,
                vec![
                    FieldColumn::new(schema.get_field("id").unwrap(), ids),
                    FieldColumn::new(schema.get_field("age").unwrap(), ages),
                    FieldColumn::new(schema.get_field("deposit").unwrap(), deposits),
                    FieldColumn::new(schema.get_field("picture").unwrap(), pictures),
                ],
                None,
            )
            .await?;
        client.flush(&collection_name).await?;

        let index_params = IndexParams::new(
            "picture_index".to_owned(),
            IndexType::IvfFlat,
            MetricType::L2,
            HashMap::from([("nlist".to_owned(), "32".to_owned())]),
        );
        client
            .create_index(&collection_name, "picture", index_params)
            .await?;
        client.load_collection(&collection_name, None).await?;

        let query_options = QueryIteratorOptions::new()
            .batch_size(10)
            .limit(25)
            .filter("age >= 10".to_string())
            .output_fields(vec!["id".to_string(), "age".to_string()]);
        let mut query_iterator = client
            .query_iterator(&collection_name, query_options)
            .await?;
        let query_page = query_iterator.next().await?.expect("query page");
        assert!(!query_page.is_empty());
        assert!(query_page.iter().all(|field| field.len() <= 10));
        query_iterator.close();

        let search_vector = gen_random_f32_vector_custom(1, DIM);
        let search_options = SearchIteratorOptions::new()
            .batch_size(5)
            .limit(10)
            .anns_field("picture".to_string())
            .output_fields(vec!["id".to_string(), "age".to_string()])
            .add_search_param("metric_type".to_string(), "L2".to_string())
            .add_search_param("params".to_string(), "{\"nprobe\": 10}".to_string());
        let mut search_iterator = client
            .search_iterator(
                &collection_name,
                vec![Value::FloatArray(search_vector.into())],
                search_options,
            )
            .await?;
        let search_page = search_iterator.next().await?.expect("search page");
        assert!(!search_page.is_empty());
        assert!(search_page.iter().all(|result| result.size > 0));
        search_iterator.close();

        Ok(())
    })
    .await
}
