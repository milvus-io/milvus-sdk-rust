/// Milvus 2.6 Features Example
///
/// Demonstrates new capabilities introduced in Milvus 2.6:
/// - COSINE metric with HNSW index
/// - Truncate collection
/// - Batch describe collections
/// - Schema evolution (add_collection_field)
/// - Partial upsert
/// - BM25 full-text search function
/// - Int8 vector field
/// - Timestamptz field
///
/// Requires: Milvus 2.6+ running on localhost:19530.
/// Start with: docker-compose up -d

use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::{CreateCollectionOptions, LoadOptions};
use milvus::query::{QueryOptions, SearchOptions};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use rand::Rng;
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const DIM: i64 = 16;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new(URL).await?;

    println!("=== Milvus 2.6 Features Demo ===\n");

    cosine_search_example(&client).await?;
    truncate_collection_example(&client).await?;
    batch_describe_example(&client).await?;
    partial_upsert_example(&client).await?;
    int8_vector_example(&client).await?;
    timestamptz_example(&client).await?;
    bm25_function_example(&client).await?;

    println!("\n=== All Milvus 2.6 demos completed successfully! ===");
    Ok(())
}

/// COSINE metric search with HNSW index
async fn cosine_search_example(client: &Client) -> Result<(), Error> {
    println!("--- 1. COSINE Metric Search with HNSW ---");
    let name = "demo_cosine_search";
    cleanup(client, name).await;

    let schema = CollectionSchemaBuilder::new(name, "COSINE metric demo")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("embedding", "", DIM))
        .add_field(FieldSchema::new_varchar("category", "", 128))
        .build()?;
    client.create_collection(schema.clone(), None).await?;

    // Insert data
    let num = 500;
    let mut rng = rand::thread_rng();
    let embeddings: Vec<f32> = (0..num * DIM).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let categories: Vec<String> = (0..num)
        .map(|i| format!("cat_{}", i % 5))
        .collect();

    let embed_col = FieldColumn::new(schema.get_field("embedding").unwrap(), embeddings);
    let cat_col = FieldColumn::new(schema.get_field("category").unwrap(), categories);
    client.insert(name, vec![embed_col, cat_col], None).await?;
    client.flush(name).await?;

    // Create HNSW index with COSINE
    let index_params = IndexParams::new(
        "cosine_hnsw".to_owned(),
        IndexType::HNSW,
        MetricType::COSINE,
        HashMap::from([
            ("M".to_owned(), "16".to_owned()),
            ("efConstruction".to_owned(), "64".to_owned()),
        ]),
    );
    client.create_index(name, "embedding", index_params).await?;

    // Create INVERTED index on category for scalar filtering
    let scalar_idx = IndexParams::new(
        "cat_inverted".to_owned(),
        IndexType::Inverted,
        MetricType::L2,
        HashMap::new(),
    );
    client.create_index(name, "category", scalar_idx).await?;

    client
        .load_collection(name, Some(LoadOptions::default()))
        .await?;

    // Search
    let query_vec: Vec<f32> = (0..DIM).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let options = SearchOptions::with_limit(5)
        .output_fields(vec!["id".to_string(), "category".to_string()])
        .add_param("ef", "64")
        .filter("category == \"cat_2\"".to_string());

    let results = client
        .search(name, vec![Value::from(query_vec)], Some(options))
        .await?;

    for r in &results {
        println!(
            "  Found {} results, top score: {:.4}",
            r.size,
            r.score.first().unwrap_or(&0.0)
        );
    }

    client.drop_collection(name).await?;
    println!("  OK\n");
    Ok(())
}

/// Truncate collection: remove all data without dropping
async fn truncate_collection_example(client: &Client) -> Result<(), Error> {
    println!("--- 2. Truncate Collection ---");
    let name = "demo_truncate";
    cleanup(client, name).await;

    let schema = CollectionSchemaBuilder::new(name, "truncate demo")
        .add_field(FieldSchema::new_primary_int64("id", "", true))
        .add_field(FieldSchema::new_float_vector("v", "", 4))
        .build()?;
    client.create_collection(schema.clone(), None).await?;

    // Insert some data
    let vecs: Vec<f32> = (0..100 * 4).map(|i| (i as f32) * 0.01).collect();
    let vec_col = FieldColumn::new(schema.get_field("v").unwrap(), vecs);
    client.insert(name, vec![vec_col], None).await?;
    client.flush(name).await?;
    println!("  Inserted 100 rows");

    // Truncate
    client.truncate_collection(name).await?;
    println!("  Collection truncated (data removed, schema preserved)");

    // Collection still exists
    assert!(client.has_collection(name).await?);
    println!("  Collection still exists: true");

    client.drop_collection(name).await?;
    println!("  OK\n");
    Ok(())
}

/// Batch describe: describe multiple collections in one RPC
async fn batch_describe_example(client: &Client) -> Result<(), Error> {
    println!("--- 3. Batch Describe Collections ---");
    let names = ["demo_batch_a", "demo_batch_b", "demo_batch_c"];
    for n in names {
        cleanup(client, n).await;
        let schema = CollectionSchemaBuilder::new(n, "batch demo")
            .add_field(FieldSchema::new_primary_int64("id", "", true))
            .add_field(FieldSchema::new_float_vector("v", "", 4))
            .build()?;
        client.create_collection(schema, None).await?;
    }

    let collections = client
        .batch_describe_collections(names.iter().map(|s| s.to_string()).collect())
        .await?;

    println!("  Described {} collections in one call:", collections.len());
    for c in &collections {
        println!("    - {} (id={})", c.name, c.id);
    }

    for n in names {
        client.drop_collection(n).await?;
    }
    println!("  OK\n");
    Ok(())
}

/// Partial upsert: update only specified fields
async fn partial_upsert_example(client: &Client) -> Result<(), Error> {
    println!("--- 4. Partial Upsert ---");
    let name = "demo_partial_upsert";
    cleanup(client, name).await;

    let schema = CollectionSchemaBuilder::new(name, "partial upsert demo")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .add_field(FieldSchema::new_varchar("label", "", 256))
        .build()?;
    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                milvus::client::ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Initial insert
    let ids: Vec<i64> = vec![1, 2, 3];
    let vecs: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
    let labels: Vec<String> = vec!["apple".into(), "banana".into(), "cherry".into()];

    client
        .insert(
            name,
            vec![
                FieldColumn::new(schema.get_field("id").unwrap(), ids),
                FieldColumn::new(schema.get_field("embedding").unwrap(), vecs),
                FieldColumn::new(schema.get_field("label").unwrap(), labels),
            ],
            None,
        )
        .await?;
    client.flush(name).await?;
    println!("  Inserted 3 rows: apple, banana, cherry");

    // Partial upsert: only update label for id=1,2 (keep embedding unchanged)
    let upsert_ids: Vec<i64> = vec![1, 2];
    let new_labels: Vec<String> = vec!["APPLE".into(), "BANANA".into()];

    let upsert_opts = milvus::mutate::UpsertOptions::new().partial_update(true);
    client
        .upsert(
            name,
            vec![
                FieldColumn::new(schema.get_field("id").unwrap(), upsert_ids),
                FieldColumn::new(schema.get_field("label").unwrap(), new_labels),
            ],
            Some(upsert_opts),
        )
        .await?;
    println!("  Partial upsert: updated labels to APPLE, BANANA (embedding unchanged)");

    client.drop_collection(name).await?;
    println!("  OK\n");
    Ok(())
}

/// Int8 vector field
async fn int8_vector_example(client: &Client) -> Result<(), Error> {
    println!("--- 5. Int8 Vector Field ---");
    let name = "demo_int8_vector";
    cleanup(client, name).await;

    let dim: i64 = 8;
    let schema = CollectionSchemaBuilder::new(name, "int8 vector demo")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_int8_vector("embedding", "", dim))
        .build()?;
    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                milvus::client::ConsistencyLevel::Strong,
            )),
        )
        .await?;

    // Int8 vectors are stored as bytes (1 byte per dimension)
    let num_rows = 100;
    let ids: Vec<i64> = (0..num_rows).collect();
    let mut rng = rand::thread_rng();
    let vectors: Vec<u8> = (0..num_rows * dim as i64)
        .map(|_| rng.gen_range(0..128) as u8)
        .collect();

    let id_col = FieldColumn::new(schema.get_field("id").unwrap(), ids);
    let vec_col = FieldColumn::new(schema.get_field("embedding").unwrap(), vectors);

    client.insert(name, vec![id_col, vec_col], None).await?;
    client.flush(name).await?;
    println!("  Inserted {} rows with Int8Vector (dim={})", num_rows, dim);

    client.drop_collection(name).await?;
    println!("  OK\n");
    Ok(())
}

/// Timestamptz field
async fn timestamptz_example(client: &Client) -> Result<(), Error> {
    println!("--- 6. Timestamptz Field ---");
    let name = "demo_timestamptz";
    cleanup(client, name).await;

    let schema = CollectionSchemaBuilder::new(name, "timestamptz demo")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("embedding", "", 4))
        .add_field(FieldSchema::new_timestamptz("created_at", "creation time"))
        .build()?;
    client
        .create_collection(
            schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                milvus::client::ConsistencyLevel::Strong,
            )),
        )
        .await?;

    let ids: Vec<i64> = vec![1, 2, 3];
    let vecs: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
    // Timestamps in microseconds since epoch
    let timestamps: Vec<i64> = vec![1700000000_000_000, 1700000001_000_000, 1700000002_000_000];

    // Use ValueVec::Timestamptz for correct wire type serialization
    let ts_value = milvus::value::ValueVec::Timestamptz(timestamps);
    let ts_col = FieldColumn {
        name: "created_at".to_string(),
        dtype: milvus::proto::schema::DataType::Timestamptz,
        value: ts_value,
        dim: 1,
        max_length: 0,
        is_dynamic: false,
    };

    client
        .insert(
            name,
            vec![
                FieldColumn::new(schema.get_field("id").unwrap(), ids),
                FieldColumn::new(schema.get_field("embedding").unwrap(), vecs),
                ts_col,
            ],
            None,
        )
        .await?;
    client.flush(name).await?;
    println!("  Inserted 3 rows with Timestamptz field");

    // Query
    let index_params = IndexParams::new(
        "emb_idx".to_owned(),
        IndexType::Flat,
        MetricType::L2,
        HashMap::new(),
    );
    client.create_index(name, "embedding", index_params).await?;
    client
        .load_collection(name, Some(LoadOptions::default()))
        .await?;

    let options = QueryOptions::new()
        .output_fields(vec!["id".to_string(), "created_at".to_string()])
        .limit(10);
    let result = client.query(name, "id > 0", &options).await?;
    println!(
        "  Query returned {} columns, {} rows",
        result.len(),
        result.first().map(|c| c.len()).unwrap_or(0)
    );

    client.drop_collection(name).await?;
    println!("  OK\n");
    Ok(())
}

/// BM25 function for full-text search
///
/// BM25 functions must be defined at schema creation time using the proto
/// CollectionSchema's `functions` field. The `add_collection_function` RPC
/// is reserved for future function types.
async fn bm25_function_example(client: &Client) -> Result<(), Error> {
    println!("--- 7. BM25 Full-Text Search (RunAnalyzer + Schema Function) ---");
    let name = "demo_bm25";
    cleanup(client, name).await;

    // 1. Test the analyzer API
    println!("  Testing analyzer...");
    let results = client
        .run_analyzer(
            vec![
                "Milvus is a vector database".to_string(),
                "Built for AI applications".to_string(),
            ],
            "{\"type\": \"standard\"}",
        )
        .await?;
    for (i, r) in results.iter().enumerate() {
        let tokens: Vec<&str> = r.tokens.iter().map(|t| t.token.as_str()).collect();
        println!("  Text {}: tokens = {:?}", i, tokens);
    }

    // 2. Create collection with BM25 function defined in the schema.
    //    BM25 maps a VarChar input field to a SparseFloatVector output field.
    //    The function must be set at schema creation time via add_function().
    let bm25_function = milvus::proto::schema::FunctionSchema {
        name: "bm25_fn".to_string(),
        id: 0,
        description: "BM25 text to sparse".to_string(),
        r#type: milvus::proto::schema::FunctionType::Bm25 as i32,
        input_field_names: vec!["text".to_string()],
        input_field_ids: vec![],
        output_field_names: vec!["sparse_vector".to_string()],
        output_field_ids: vec![],
        params: vec![],
    };

    let sdk_schema = CollectionSchemaBuilder::new(name, "BM25 full-text search demo")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(
            FieldSchema::new_varchar("text", "document text", 4096)
                .add_type_param("enable_analyzer", "true")
                .add_type_param("enable_match", "true")
                .add_type_param("analyzer_params", "{\"type\": \"standard\"}"),
        )
        .add_field(FieldSchema::new_sparse_float_vector(
            "sparse_vector",
            "BM25 output",
        ))
        .add_field(FieldSchema::new_float_vector("dense_vector", "", 4))
        .add_function(bm25_function)
        .build()?;

    client
        .create_collection(
            sdk_schema.clone(),
            Some(CreateCollectionOptions::with_consistency_level(
                milvus::client::ConsistencyLevel::Strong,
            )),
        )
        .await?;
    println!("  Created collection with BM25 function in schema");

    // 3. Insert documents. Only provide text + dense_vector.
    //    The BM25 function auto-generates sparse_vector from text.
    let ids: Vec<i64> = vec![1, 2, 3, 4, 5];
    let texts: Vec<String> = vec![
        "Milvus is a high-performance vector database".into(),
        "Vector search enables AI applications".into(),
        "Rust provides memory safety without garbage collection".into(),
        "Full-text search with BM25 ranking algorithm".into(),
        "Milvus supports hybrid search combining vectors and text".into(),
    ];
    let dense_vecs: Vec<f32> = vec![
        0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7,
        1.8, 1.9, 2.0,
    ];

    client
        .insert(
            name,
            vec![
                FieldColumn::new(sdk_schema.get_field("id").unwrap(), ids),
                FieldColumn::new(sdk_schema.get_field("text").unwrap(), texts),
                FieldColumn::new(sdk_schema.get_field("dense_vector").unwrap(), dense_vecs),
            ],
            None,
        )
        .await?;
    client.flush(name).await?;
    println!("  Inserted 5 documents (sparse_vector auto-generated by BM25)");

    // 4. Create indexes and load
    let sparse_idx = IndexParams::new(
        "sparse_idx".to_owned(),
        IndexType::SparseInvertedIndex,
        MetricType::BM25,
        HashMap::from([("bm25_k1".to_owned(), "1.2".to_owned()), ("bm25_b".to_owned(), "0.75".to_owned())]),
    );
    client.create_index(name, "sparse_vector", sparse_idx).await?;

    let dense_idx = IndexParams::new(
        "dense_idx".to_owned(),
        IndexType::Flat,
        MetricType::L2,
        HashMap::new(),
    );
    client.create_index(name, "dense_vector", dense_idx).await?;
    client
        .load_collection(name, Some(LoadOptions::default()))
        .await?;
    println!("  Indexed (SPARSE_INVERTED_INDEX + FLAT) and loaded");

    // 5. Full-text search via query filter on BM25 field
    //    Note: Direct BM25 vector search via search() requires text placeholder
    //    support in get_place_holder_group(), which is a follow-up enhancement.
    //    For now, use query() with TextMatch expressions for full-text search.
    let query_options = QueryOptions::new()
        .output_fields(vec!["id".to_string(), "text".to_string()])
        .limit(3);

    let results = client
        .query(name, "text_match(text, 'vector database')", &query_options)
        .await?;
    if let Some(id_col) = results.iter().find(|c| c.name == "id") {
        println!("  Full-text query (text_match) found {} hits", id_col.len());
    }

    client.drop_collection(name).await?;
    println!("  OK\n");
    Ok(())
}

async fn cleanup(client: &Client, name: &str) {
    if client.has_collection(name).await.unwrap_or(false) {
        let _ = client.drop_collection(name).await;
    }
}
