# Milvus Rust SDK V2 Quick Start

The V2 API is the active SDK surface. Import it through the prelude, create a [`ClientV2`],
build requests with validated builders, and read responses through SDK-owned accessors.

The examples below assume a local Milvus at `http://localhost:19530` with the default
`root:Milvus` credentials.

## 1. Connect

```rust,no_run
use milvus::v2::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientV2::new(
        &ConnectConfig::new()
            .uri("http://localhost:19530")
            .token("root:Milvus"),
    )
    .await?;

    let health = client
        .check_health(CheckHealthRequest::builder().build()?)
        .await?;
    println!("healthy: {}", health.is_healthy());
    Ok(())
}
```

## 2. Define a schema and create a collection

```rust,no_run
use milvus::v2::prelude::*;

async fn create(client: &ClientV2) -> Result<()> {
    let schema = CollectionSchema::new()
        .add_field(FieldSchema::new().name("id").data_type(DataType::Int64).primary_key(true))
        .add_field(FieldSchema::new().name("title").data_type(DataType::VarChar).max_length(256))
        .add_field(FieldSchema::new().name("embedding").data_type(DataType::FloatVector).dimension(4));

    let request = CreateCollectionRequest::builder()
        .collection_name("books")
        .schema(schema)
        .build()?;
    client.create_collection(request).await?;
    Ok(())
}
```

## 3. Create an index and load the collection

```rust,no_run
use milvus::v2::prelude::*;
use std::collections::HashMap;

async fn index_and_load(client: &ClientV2) -> Result<()> {
    let index = CreateIndexRequest::builder()
        .collection_name("books")
        .index_params(vec![IndexParam::new()
            .field_name("embedding")
            .index_type(IndexType::Hnsw)
            .metric_type(MetricType::Cosine)
            .extra_params(HashMap::from([("M".to_owned(), "16".to_owned())]))])
        .build()?;
    client.create_index(index).await?;

    client
        .load_collection(LoadCollectionRequest::builder().collection_name("books").build()?)
        .await?;
    Ok(())
}
```

## 4. Insert data

```rust,no_run
use milvus::v2::prelude::*;

async fn insert(client: &ClientV2) -> Result<()> {
    let request = InsertRequest::builder()
        .collection_name("books")
        .columns(vec![
            FieldData::Int64 { name: "id".into(), values: vec![1, 2] },
            FieldData::VarChar { name: "title".into(), values: vec!["A".into(), "B".into()] },
            FieldData::FloatVector {
                name: "embedding".into(),
                values: vec![vec![0.1, 0.2, 0.3, 0.4], vec![0.5, 0.6, 0.7, 0.8]],
            },
        ])
        .build()?;
    client.insert(request).await?;
    Ok(())
}
```

## 5. Query and search

```rust,no_run
use milvus::v2::prelude::*;

async fn query(client: &ClientV2) -> Result<()> {
    let request = QueryRequest::builder()
        .collection_name("books")
        .filter("id > 0")
        .output_fields(["id", "title"])
        .limit(10)
        .build()?;
    let response = client.query(request).await?;
    for row in response.results().rows()? {
        println!("id = {}", row.get_i64("id")?);
    }
    Ok(())
}

async fn search(client: &ClientV2) -> Result<()> {
    let request = SearchRequest::builder()
        .collection_name("books")
        .vectors(SearchVectors::Float(vec![vec![0.1, 0.2, 0.3, 0.4]]))
        .output_fields(["id", "title"])
        .limit(5)
        .build()?;
    let response = client.search(request).await?;
    for query in response.results().iter() {
        for (index, row) in query.rows()?.enumerate() {
            println!("hit score = {}", query.get_scores()[index]);
        }
    }
    Ok(())
}
```

## 6. Clean up

```rust,no_run
use milvus::v2::prelude::*;

async fn cleanup(client: &ClientV2) -> Result<()> {
    client
        .drop_collection(DropCollectionRequest::builder().collection_name("books").build()?)
        .await?;
    Ok(())
}
```

## Next steps

- [Concepts and data model](#v2-concepts-and-data-model) for consistency, indexes, and error handling.
- The `examples/v2/` directory contains runnable programs for each feature area.
- The `tutorial/` directory contains independently buildable applications.
