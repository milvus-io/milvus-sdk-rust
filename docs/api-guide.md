## Milvus Rust SDK Function Usage Manual (v0.1.0 | Compatible with Milvus 2.5.15)

This document lists each function's capabilities, runnable usage examples (with detailed explanations and expected outputs) by functional modules to help you get started quickly. Examples default to using local Milvus - please start the service first when necessary and ensure the port and authentication configuration match.

---

### Quick start

```rust
use milvus::{
    client::{Client, ClientBuilder, ConsistencyLevel},
    data::FieldColumn,
    error::Result,
    index::{IndexParams, IndexType, MetricType},
    options::LoadOptions,
    query::{QueryOptions, SearchOptions},
    schema::{CollectionSchemaBuilder, FieldSchema},
    value::Value,
};
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const COLLECTION: &str = "hello_milvus";
const VEC_FIELD: &str = "embeddings";
const DIM: i64 = 128;

#[tokio::main]
async fn main() -> Result<()> {
    // connect with milvus
    let client = ClientBuilder::new(URL)
        .timeout(std::time::Duration::from_secs(30))
        // .username("root").password("Milvus") // if use authentication
        .build()
        .await?;

    // 1) define schema and create collection
    let schema = CollectionSchemaBuilder::new(COLLECTION, "Quick start collection")
        .add_field(FieldSchema::new_primary_int64("id", "primary key", true))
        .add_field(FieldSchema::new_float_vector(VEC_FIELD, "vector", DIM))
        .build()?;

    if client.has_collection(COLLECTION).await? {
        client.drop_collection(COLLECTION).await?;
    }
    client.create_collection(schema.clone(), None).await?;

    // 2) insert data and flush
    let mut rng = rand::thread_rng();
    let vectors: Vec<f32> = (0..(DIM * 1000)).map(|_| rand::random::<f32>()).collect();
    let vec_col = FieldColumn::new(schema.get_field(VEC_FIELD).unwrap(), vectors);
    client.insert(COLLECTION, vec![vec_col], None).await?;
    client.flush(COLLECTION).await?;

    // 3) create index and load
    let index_params = IndexParams::new(
        "vec_index".to_string(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from([("nlist".to_string(), "32".to_string())]),
    );
    client.create_index(COLLECTION, VEC_FIELD, index_params).await?;
    client.load_collection(COLLECTION, Some(LoadOptions::default())).await?;

    // 4) search and query
    let search_vec = Value::from((0..DIM).map(|_| rand::random::<f32>()).collect::<Vec<f32>>());
    let sopt = SearchOptions::new()
        .limit(10)
        .add_param("anns_field", VEC_FIELD)
        .add_param("metric_type", "L2")
        .add_param("nprobe", "16");
    let search_res = client.search(COLLECTION, vec![search_vec], Some(sopt)).await?;
    println!("Top-10 search results: {:?}", search_res[0].score);

    let qopt = QueryOptions::default();
    let query_res = client.query(COLLECTION, "id > 0", &qopt).await?;
    println!("Query returned columns: {}", query_res.len());

    // 5) clean up
    client.drop_collection(COLLECTION).await?;
    Ok(())
}
```

---

### Client and Connection (Client / ClientBuilder)

#### Function: `Client::new(dst)`
- Functionality: Creates a connection to Milvus using default timeout (`RPC_TIMEOUT`).
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    println!("Client connected.");
    Ok(())
}
```
- Explanation: Quickly establishes a connection to Milvus using default timeout.
- Expected Output: Prints "Client connected."; if Milvus is not started or address is incorrect, will return a connection error.

#### Function: `Client::with_timeout(dst, timeout, username, password)`
- Functionality: Creates a connection with custom timeout and authentication (Basic Auth).
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::with_timeout(
        "http://localhost:19530",
        Duration::from_secs(20),
        Some("root".to_string()),
        Some("Milvus".to_string()),
    ).await?;
    println!("Client connected with auth and timeout.");
    Ok(())
}
```
- Explanation: Sets 20s timeout and uses username/password.
- Expected Output: Prints "Client connected with auth and timeout."; incorrect credentials will return authentication error.

#### Function: `ClientBuilder::new(dst)`, `username(..)`, `password(..)`, `timeout(..)`, `build()`
- Functionality: Configures connection parameters using builder pattern and generates `Client`. Recommended approach.
- Usage Example:
```rust
use milvus::client::ClientBuilder;
use milvus::error::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientBuilder::new("http://localhost:19530")
        .username("root")
        .password("Milvus")
        .timeout(Duration::from_secs(60))
        .build()
        .await?;
    println!("Builder client ready.");
    Ok(())
}
```
- Explanation: Chain-configure authentication and timeout, then build the client.
- Expected Output: Prints "Builder client ready."; throws error if parameters are invalid or connection fails.

#### Function: `Client::flush_collections(collections: Vec<&str>)`
- Functionality: Batch flush multiple collections, persisting memory data.
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let res = client.flush_collections(vec!["col_a", "col_b"]).await;
    println!("Flush collections ok? {}", res.is_ok());
    Ok(())
}
```
- Explanation: Executes flush on multiple collections.
- Expected Output: Prints whether successful; may error if collections don't exist.

---

### Collections (Collection) and Data Full Workflow (Comprehensive Coverage of Most Functions)

This example covers: create/check/list/describe/stats/insert/flush/create_index/load/query/search/release/compaction/drop_index/drop_collection.

- Functions involved:
  - `has_collection`, `create_collection`, `list_collections`, `describe_collection`, `get_collection_stats`
  - `insert`, `flush`
  - `create_index`, `describe_index`, `list_indexes`, `drop_index`
  - `load_collection`, `get_load_state`, `release_collection`
  - `manual_compaction`, `get_compaction_state`
  - `drop_collection`
- Usage Example:
```rust
use milvus::{
    client::{Client, ConsistencyLevel},
    data::FieldColumn,
    error::Result,
    index::{IndexParams, IndexType, MetricType},
    options::{CreateCollectionOptions, GetLoadStateOptions, LoadOptions},
    query::{QueryOptions, SearchOptions},
    schema::{CollectionSchemaBuilder, FieldSchema},
    value::Value,
};
use std::collections::HashMap;

const URL: &str = "http://localhost:19530";
const C: &str = "collection_full_demo";
const VEC: &str = "feature";
const DIM: i64 = 128;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new(URL).await?;

    // 1) Schema & Create
    let schema = CollectionSchemaBuilder::new(C, "demo collection")
        .add_field(FieldSchema::new_primary_int64("id", "pk", true))
        .add_field(FieldSchema::new_float_vector(VEC, "vec", DIM))
        .build()?;

    if client.has_collection(C).await? {
        client.drop_collection(C).await?;
    }
    client.create_collection(
        schema.clone(),
        Some(CreateCollectionOptions::with_consistency_level(
            ConsistencyLevel::Session,
        )),
    ).await?;
    println!("Created collection: {}", C);

    // 2) Insert & flush
    let vectors: Vec<f32> = (0..(DIM * 2000)).map(|_| rand::random::<f32>()).collect();
    let vcol = FieldColumn::new(schema.get_field(VEC).unwrap(), vectors);
    client.insert(C, vec![vcol], None).await?;
    client.flush(C).await?;
    println!("Inserted & flushed 2000 entities.");

    // 3) Index & Load
    let index_params = IndexParams::new(
        "vec_idx".to_string(),
        IndexType::IvfFlat,
        MetricType::L2,
        HashMap::from([("nlist".to_string(), "32".to_string())]),
    );
    client.create_index(C, VEC, index_params).await?;
    client.load_collection(C, Some(LoadOptions::default())).await?;
    println!("Index created & collection loaded.");

    // 4) Query & Search
    let qopt = QueryOptions::new().limit(10).output_fields(vec!["id".to_string()]);
    let qres = client.query(C, "id >= 0", &qopt).await?;
    println!("Query got {} column(s).", qres.len());

    let sopt = SearchOptions::new()
        .limit(5)
        .add_param("anns_field", VEC)
        .add_param("metric_type", "L2")
        .add_param("nprobe", "16");
    let qv = Value::from(vec![0.0f32; DIM as usize]);
    let sres = client.search(C, vec![qv], Some(sopt)).await?;
    println!("Top-5 search results count: {}", sres[0].size);

    // 5) Stats & Index info
    let stats = client.get_collection_stats(C).await?;
    println!("Row count: {}", stats.get("row_count").unwrap_or(&"0".to_string()));
    let idx_names = client.list_indexes(C, None).await?;
    println!("Indexes: {:?}", idx_names);
    let first_idx_info = client.describe_index(C, &idx_names[0]).await?;
    println!("Index info entries: {}", first_idx_info.len());

    // 6) Release / Compaction / Drop index / Drop collection
    client.release_collection(C).await?;
    let comp = client.manual_compaction(C, None).await?;
    let comp_state = client.get_compaction_state(comp.id).await?;
    println!("Compaction plan_count={}, state={:?}", comp.plan_count, comp_state);

    client.drop_index(C, &idx_names[0]).await?;
    client.drop_collection(C).await?;
    println!("Collection dropped.");
    Ok(())
}
```
- Explanation:
  - First creates a collection (with primary key and vector fields), writes 2000 vectors, `flush` to persist.
  - Creates IVF_FLAT index and loads the collection, then executes simple `query` and `search`.
  - Gets collection statistics and index information, releases collection then performs manual compaction, finally deletes index and collection.
- Expected Output:
  - Prints in sequence: creation success, insert and flush success, index creation and loading success, query/search result counts, row_count, index names, index description entry count, compaction state, collection deletion success.

---

### Partitions (Partition) Management

#### Function: `create_partition(collection, partition)`
- Functionality: Creates a partition within a collection.
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let (c, p) = ("partition_col", "pA");
    if !client.has_collection(c).await? {
        panic!("Please create collection {} first", c);
    }
    client.create_partition(c.to_string(), p.to_string()).await?;
    println!("Partition created: {}", p);
    Ok(())
}
```
- Explanation: Creates new partition `pA` in existing collection `partition_col`.
- Expected Output: Prints "Partition created: pA".

#### Function: `list_partitions(collection)`, `has_partition(collection, partition)`
- Functionality: Lists partitions, checks if partition exists.
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let parts = client.list_partitions("partition_col".to_string()).await?;
    println!("Partitions: {:?}", parts);
    println!("Has pA? {}", client.has_partition("partition_col".to_string(), "pA".to_string()).await?);
    Ok(())
}
```
- Explanation: Prints collection's partition list and checks if `pA` exists.
- Expected Output: Shows partition array, `Has pA? true/false`.

#### Function: `get_partition_stats(collection, partition)`
- Functionality: Gets partition statistics (such as row count).
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let stats = client.get_partition_stats("partition_col".to_string(), "pA".to_string()).await?;
    println!("Partition stats: {:?}", stats);
    Ok(())
}
```
- Explanation: Returns statistics like `{"row_count": "...", ...}`.
- Expected Output: Prints partition statistics.

#### Function: `load_partitions`, `release_partitions`, `get_load_state(.. with_partition_names(..))`
- Functionality: Partition-level load/release and state query.
- Usage Example:
```rust
use milvus::{client::Client, error::Result, options::GetLoadStateOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let c = "partition_col";
    client.load_partitions(c, vec!["pA"], 0, None).await?;
    let st = client.get_load_state(
        c,
        Some(GetLoadStateOptions::with_partition_names(vec!["pA".to_string()])),
    ).await?;
    println!("Partition load state: {:?}", st);
    client.release_partitions(c, vec!["pA"]).await?;
    Ok(())
}
```
- Explanation: Loads partition `pA`, queries state as `Loaded`, then releases.
- Expected Output: Prints load state (Loaded/NotLoad).

#### Function: `drop_partition(collection, partition)`
- Functionality: Deletes a partition.
- Usage Example:
```rust
use milvus::client::Client;
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    client.drop_partition("partition_col".to_string(), "pA".to_string()).await?;
    println!("Partition pA dropped.");
    Ok(())
}
```
- Explanation: Deletes partition `pA`.
- Expected Output: Prints "Partition pA dropped.".

---

### Aliases (Alias)

#### Function: `create_alias(collection, alias)`, `alter_alias(collection, alias)`, `drop_alias(alias)`
- Functionality: Creates/modifies/deletes collection aliases.
- Usage Example:
```rust
use milvus::{client::Client, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let (c, a) = ("alias_col", "alias_a");
    if !client.has_collection(c).await? { panic!("Please create collection {} first", c); }
    client.create_alias(c, a).await?;
    println!("Alias created: {}", a);
    client.alter_alias(c, a).await?;
    println!("Alias altered to point {}", c);
    client.drop_alias(a).await?;
    println!("Alias dropped.");
    Ok(())
}
```
- Explanation: Points alias `alias_a` to collection `alias_col`, then deletes the alias.
- Expected Output: Prints creation/modification/deletion confirmation messages.

#### Function: `describe_alias(alias)`, `list_aliases(collection)`
- Functionality: Queries alias details, lists collection's aliases.
- Usage Example:
```rust
use milvus::{client::Client, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let (alias, col, db) = client.describe_alias("alias_a").await?;
    println!("alias={} collection={} db={}", alias, col, db);

    let (_db, _col, aliases) = client.list_aliases(col.as_str()).await?;
    println!("Aliases: {:?}", aliases);
    Ok(())
}
```
- Explanation: Outputs alias's corresponding collection and database name; lists all aliases under the collection.
- Expected Output: Prints alias information and array list.

---

### Data Writing and Modification (Mutate)

#### Function: `insert(collection, fields, Option<InsertOptions>)`
- Functionality: Inserts data columns (`FieldColumn`).
- Usage Example:
```rust
use milvus::{client::Client, data::FieldColumn, error::Result, schema::{CollectionSchemaBuilder, FieldSchema}};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let c = "mutate_col";
    if client.has_collection(c).await? { client.drop_collection(c).await?; }
    let schema = CollectionSchemaBuilder::new(c, "")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("vec", "", 64)).build()?;
    client.create_collection(schema.clone(), None).await?;

    let ids: Vec<i64> = (0..100).collect();
    let vecs: Vec<f32> = (0..(64 * 100)).map(|_| rand::random()).collect();
    let cols = vec![
        FieldColumn::new(schema.get_field("id").unwrap(), ids),
        FieldColumn::new(schema.get_field("vec").unwrap(), vecs),
    ];
    client.insert(c, cols, None).await?;
    println!("Inserted 100 rows.");
    Ok(())
}
```
- Explanation: Creates table and inserts 100 rows (with integer primary key and 64-dimensional vectors).
- Expected Output: Prints "Inserted 100 rows.".

#### Function: `upsert(collection, fields, None)`
- Functionality: Updates if exists, inserts if not exists (same input as `FieldColumn`).
- Usage Example (similar to insert):
```rust
use milvus::{client::Client, data::FieldColumn, error::Result, schema::{CollectionSchemaBuilder, FieldSchema}};
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let c = "upsert_col";
    if client.has_collection(c).await? { client.drop_collection(c).await?; }
    let schema = CollectionSchemaBuilder::new(c, "")
        .add_field(FieldSchema::new_primary_int64("id", "", false))
        .add_field(FieldSchema::new_float_vector("vec", "", 64)).build()?;
    client.create_collection(schema.clone(), None).await?;
    let ids: Vec<i64> = (0..10).collect();
    let vecs: Vec<f32> = (0..(64*10)).map(|_| rand::random()).collect();
    let cols = vec![
         FieldColumn::new(schema.get_field("id").unwrap(), ids),
        FieldColumn::new(schema.get_field("vec").unwrap(), vecs),
    ];
    client.upsert(c, cols, None).await?;
    println!("Upsert done.");
    Ok(())
}
```
- Explanation: Updates corresponding row if primary key already exists.
- Expected Output: Prints "Upsert done.".

#### Function: `delete(collection, &DeleteOptions::with_ids(..) | with_filter(..))`
- Functionality: Deletes by ID collection or filter expression.
- Usage Example:
```rust
use milvus::{client::Client, error::Result, mutate::DeleteOptions, value::ValueVec};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let c = "mutate_col";
    // by ids
    let ids = vec![1i64, 2, 3];
    client.delete(c, &DeleteOptions::with_ids(ValueVec::Long(ids))).await?;
    println!("Deleted by ids.");
    // by filter
    client.delete(c, &DeleteOptions::with_filter("id < 5".to_string())).await?;
    println!("Deleted by filter.");
    Ok(())
}
```
- Explanation: Deletes by primary key first, then by expression.
- Expected Output: Prints two deletion completion messages.

#### Function: `flush(collection)`
- Functionality: Persists collection's pending write data.
- Usage Example:
```rust
use milvus::{client::Client, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    client.flush("mutate_col").await?;
    println!("Flushed mutate_col.");
    Ok(())
}
```
- Explanation: Ensures subsequent index creation/search visibility.
- Expected Output: Prints "Flushed mutate_col.".

---

### Indexes (Index)

#### Function: `IndexParams::new(name, IndexType, MetricType, params)`
- Functionality: Constructs index parameters; common keys in `params` include `nlist`, `M` (HNSW), etc.
- Usage Example (create/list/describe/delete index):
```rust
use milvus::{client::Client, error::Result, index::{IndexParams, IndexType, MetricType}};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let c = "index_col";
    let client = Client::new("http://localhost:19530").await?;
    // Assume collection and data already exist
    let idx = IndexParams::new("vec_idx".to_string(), IndexType::IvfFlat, MetricType::L2,
        HashMap::from([("nlist".to_string(), "32".to_string())]));
    client.create_index(c, "vec", idx).await?;
    let names = client.list_indexes(c, None).await?;
    println!("Index names: {:?}", names);
    let info = client.describe_index(c, &names[0]).await?;
    println!("Index info len: {}", info.len());
    client.drop_index(c, &names[0]).await?;
    println!("Index dropped.");
    Ok(())
}
```
- Explanation: Creates IVF_FLAT index and views/deletes it.
- Expected Output: Prints index name array, index description count, and deletion confirmation.

---

### Loading and Releasing (Load / Release)

#### Function: `load_collection(name, Some(LoadOptions::default()))`, `release_collection(name)`
- Functionality: Loads collection for search/query; releases memory usage.
- Usage Example:
```rust
use milvus::{client::Client, error::Result, options::LoadOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    client.load_collection("collection_full_demo", Some(LoadOptions::default())).await?;
    println!("Loaded collection_full_demo.");
    client.release_collection("collection_full_demo").await?;
    println!("Released collection_full_demo.");
    Ok(())
}
```
- Explanation: Collection can be searched/queried after loading; releases resources after use.
- Expected Output: Prints loading and releasing success.

#### Function: `get_load_state(name, Option<GetLoadStateOptions>)`
- Functionality: Queries collection or partition load state (`Loaded`/`NotLoad`).
- Usage Example (see "Partition Management" or "Collection Comprehensive Example").
- Expected Output: Prints enum state.

---

### Query and Search (Query / Search / Get)

#### Function: `Client::query(collection, expr, &QueryOptions)`
- Functionality: Queries scalar fields by expression/return fields.
- Usage Example:
```rust
use milvus::{client::Client, error::Result, query::QueryOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let c = "collection_full_demo";
    let client = Client::new("http://localhost:19530").await?;
    let opt = QueryOptions::new().limit(10).output_fields(vec!["id".to_string()]);
    let res = client.query(c, "id >= 0", &opt).await?;
    println!("Query columns: {}", res.len());
    Ok(())
}
```
- Explanation: Returns `Vec<FieldColumn>` where each element represents a column.
- Expected Output: Prints column count (e.g., 1).

#### Function: `Client::search(collection, data, Some(SearchOptions))`
- Functionality: Vector Top-K / Range search (requires index creation and loading).
- Usage Example:
```rust
use milvus::{client::Client, error::Result, query::SearchOptions, value::Value};

#[tokio::main]
async fn main() -> Result<()> {
    let c = "collection_full_demo";
    let client = Client::new("http://localhost:19530").await?;
    let query_vec = Value::from(vec![0.0f32; 128]);
    let sopt = SearchOptions::new()
        .limit(5)
        .add_param("anns_field", "feature")
        .add_param("metric_type", "L2")
        .add_param("nprobe", "16");
    let res = client.search(c, vec![query_vec], Some(sopt)).await?;
    println!("Top-5 size: {}", res[0].size);
    Ok(())
}
```
- Explanation: Top-K=5, requires setting search field and metric type.
- Expected Output: Prints first result group's size (5).

#### Function: `Client::get(collection, IdType, Some(GetOptions))`
- Functionality: Batch retrieves records by primary key.
- Usage Example:
```rust
use milvus::{client::Client, error::Result, query::{GetOptions, IdType}};

#[tokio::main]
async fn main() -> Result<()> {
    let c = "collection_full_demo";
    let client = Client::new("http://localhost:19530").await?;
    let opt = GetOptions::new().output_fields(vec!["id".to_string(), "feature".to_string()]);
    let res = client.get(c, IdType::Int64(vec![1,2,3,4,5]), Some(opt)).await?;
    println!("Get columns: {}", res.len());
    Ok(())
}
```
- Explanation: Returns field columns, length equals output field count.
- Expected Output: Prints column count (e.g., 2).

---

### Iterators (Large Result Streaming)

#### Function: `query_iterator(collection, QueryIteratorOptions)`
- Functionality: Paginated query results by filter expression, avoiding returning too much at once.
- Usage Example:
```rust
use milvus::{client::Client, error::Error, iterator::QueryIteratorOptions};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new("http://localhost:19530").await?;
    let mut it = client.query_iterator(
        "collection_full_demo",
        QueryIteratorOptions::new()
            .batch_size(50)
            .filter("id >= 0".to_string())
            .output_fields(vec!["id".to_string()])
            .limit(200),
    ).await?;
    let mut total = 0usize;
    while let Some(batch) = it.next().await? {
        if batch.is_empty() { it.close(); break; }
        // Each batch is Vec<FieldColumn>, process your data
        total += batch[0].len();
    }
    println!("Iterator total rows: {}", total);
    Ok(())
}
```
- Explanation: Returns at most `batch_size` items each time; `limit` controls total return upper bound; `close()` terminates.
- Expected Output: Prints cumulative row count.

#### Function: `search_iterator(collection, data, SearchIteratorOptions)`
- Functionality: Paginated version of vector search.
- Usage Example:
```rust
use milvus::{client::Client, error::Error, iterator::SearchIteratorOptions, value::Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new("http://localhost:19530").await?;
    let qv = Value::from(vec![0.0f32; 128]);
    let mut it = client.search_iterator(
        "collection_full_demo",
        vec![qv],
        SearchIteratorOptions::new()
            .batch_size(50)
            .anns_field("feature".to_string())
            .add_search_param("metric_type".to_string(), "L2".to_string())
            .add_search_param("params".to_string(), "{\"nprobe\": 10}".to_string())
            .reduce_stop_for_best(true)
            .limit(10),
    ).await?;
    let mut total_pages = 0;
    while let Some(results) = it.next().await? {
        if results.is_empty() { it.close(); break; }
        total_pages += 1;
    }
    println!("Search iterator pages: {}", total_pages);
    Ok(())
}
```
- Explanation: Returns search result collections in batches by page, easy to handle large-scale retrieval.
- Expected Output: Prints iteration page count (depends on data/parameters).

---

### Databases (Database)

#### Function: `create_database(name, Some(CreateDbOptions))`
- Functionality: Creates database and configures properties (replica count, max collections, etc.).
- Usage Example:
```rust
use milvus::{client::Client, database::CreateDbOptions, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("http://localhost:19530").await?;
    client.create_database("db_A", Some(CreateDbOptions::new().replica_number(1).max_collections(3))).await?;
    println!("Database db_A created.");
    Ok(())
}
```
- Explanation: Creates `db_A`, limits replicas/collections.
- Expected Output: Prints creation success.

#### Function: `list_databases()`, `describe_database(name)`
- Functionality: Lists/describes databases.
- Usage Example:
```rust
use milvus::{client::Client, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("http://localhost:19530").await?;
    let list = client.list_databases().await?;
    println!("Databases: {:?}", list);
    let desc = client.describe_database("db_A").await?;
    println!("db_A: {:?}", desc);
    Ok(())
}
```
- Explanation: Prints all databases and `db_A` details.
- Expected Output: Database array and detail structure.

#### Function: `alter_database_properties(name, CreateDbOptions)`, `using_database(name)`, `drop_database(name)`
- Functionality: Modifies database properties, switches current database, deletes database.
- Usage Example:
```rust
use milvus::{client::Client, database::CreateDbOptions, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("http://localhost:19530").await?;
    client.alter_database_properties("db_A", CreateDbOptions::new().replica_number(0).max_collections(2)).await?;
    client.using_database("db_A").await?;
    println!("Using db_A.");
    client.drop_database("db_A").await?;
    println!("db_A dropped.");
    Ok(())
}
```
- Explanation: Updates properties then switches to `db_A`, finally deletes.
- Expected Output: Prints switch and deletion success.

---

### Resource Groups (Resource Group)

#### Function: `list_resource_groups()`, `create_resource_group(name, Some(CreateRgOptions))`
- Functionality: Lists resource groups, creates resource group (sets limits/requests).
- Usage Example:
```rust
use milvus::{client::Client, error::Result, resource_group::CreateRgOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    println!("Before: {:?}", client.list_resource_groups().await?);
    client.create_resource_group("rg_test", Some(CreateRgOptions::new().limits(10).requests(2))).await?;
    println!("After: {:?}", client.list_resource_groups().await?);
    Ok(())
}
```
- Explanation: Creates resource group named `rg_test`, prints before/after lists.
- Expected Output: Resource group list contains `rg_test`.

#### Function: `describe_resource_group(name)`, `update_resource_groups(HashMap<name, UpdateRgOptions>)`, `drop_resource_group(name)`
- Functionality: Describes/updates/deletes resource group.
- Usage Example:
```rust
use milvus::{client::Client, error::Result, resource_group::{CreateRgOptions, UpdateRgOptions}};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("http://localhost:19530").await?;
    let rg = "rg_test";
    if !client.list_resource_groups().await?.iter().any(|x| x == rg) {
        client.create_resource_group(rg, Some(CreateRgOptions::new().limits(10).requests(2))).await?;
    }
    if let Some(desc) = client.describe_resource_group(rg).await? {
        println!("Desc: {:#?}", desc);
    }
    let cfg = HashMap::from([(rg.to_string(), UpdateRgOptions::new().limits(0).requests(0))]);
    client.update_resource_groups(cfg).await?;
    client.drop_resource_group(rg).await?;
    println!("Resource group dropped.");
    Ok(())
}
```
- Explanation: Views `rg_test` configuration, updates then deletes.
- Expected Output: Prints description information and deletion confirmation.

---

### Authentication and RBAC (Authentication)

#### Function (Users): `create_user`, `list_users`, `describe_user`, `update_password`, `drop_user`
- Functionality: Creates/lists/describes (with roles)/changes password/deletes users.
- Usage Example:
```rust
use milvus::{client::ClientBuilder, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientBuilder::new("http://localhost:19530")
        .username("root").password("Milvus").build().await?;
    // Users
    client.create_user("user_a", "pwd_a").await?;
    println!("Users: {:?}", client.list_users().await?);
    println!("User desc: {:?}", client.describe_user("user_a").await?);
    client.update_password("user_a", "pwd_a", "pwd_b").await?;
    client.drop_user("user_a").await?;
    println!("User user_a dropped.");
    Ok(())
}
```
- Explanation: Demonstrates creation, query, password change and deletion.
- Expected Output: User list contains `user_a`, description information contains associated roles (if any).

#### Function (Roles): `create_role`, `list_roles`, `describe_role`, `grant_role`, `revoke_role`, `drop_role`
- Functionality: Creates/lists/describes roles; grants/revokes roles to/from users; deletes roles.
- Usage Example:
```rust
use milvus::{client::ClientBuilder, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientBuilder::new("http://localhost:19530")
        .username("root").password("Milvus").build().await?;
    client.create_role("role_a").await?;
    client.create_user("user_b", "pwd").await?;
    client.grant_role("user_b", "role_a").await?;
    println!("Role desc: {:?}", client.describe_role("role_a").await?);
    client.revoke_role("user_b", "role_a").await?;
    client.drop_role("role_a", true).await?;
    client.drop_user("user_b").await?;
    println!("Role workflow done.");
    Ok(())
}
```
- Explanation: Grants role to user, views role, revokes and deletes.
- Expected Output: Prints role information (with binding information), no residue after completion.

#### Function (Privileges): `grant_privilege`, `revoke_privilege`; Privilege Groups: `create_privilege_group`, `list_privilege_groups`, `add_privilege_to_group`, `revoke_privilege_from_group`, `drop_privilege_group`
- Functionality: Grants/revokes privileges to/from roles; manages privilege groups and adds/removes privileges within groups.
- Usage Example:
```rust
use milvus::{client::ClientBuilder, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientBuilder::new("http://localhost:19530")
        .username("root").password("Milvus").build().await?;
    client.create_role("r_priv").await?;
    client.grant_privilege("r_priv", "ShowCollections", "Global", "*", None).await?;
    println!("Role desc: {:?}", client.describe_role("r_priv").await?);

    if client.list_privilege_groups().await?.contains_key(&"pg1".to_string()) {
        client.drop_privilege_group("pg1").await?;
    }
    client.create_privilege_group("pg1").await?;
    client.add_privilege_to_group("pg1", vec!["ShowCollections".to_string()]).await?;
    client.revoke_privilege_from_group("pg1", vec!["ShowCollections".to_string()]).await?;
    client.drop_privilege_group("pg1").await?;

    client.revoke_privilege("r_priv", "Global", "ShowCollections", "*", None).await?;
    client.drop_role("r_priv", true).await?;
    println!("Privilege workflow done.");
    Ok(())
}
```
- Explanation: Demonstrates role privilege granting/revoking, and privilege group addition/deletion.
- Expected Output: Prints role description, privilege group list changes and workflow completion.

---

### Option Builders and Common Method List (Quick Reference)

The following builder methods are typically used in chain within the same code segment, examples already demonstrated in multiple cases above. They don't directly produce output, their purpose is to construct request parameters.

- `QueryOptions`
  - `new()`, `limit(n)`, `output_fields(Vec<String>)`, `partition_names(Vec<String>)`
  - Purpose: Controls query return count, return fields, specifies partitions.
- `SearchOptions`
  - `new()`, `limit(n)`, `expr(expr)`, `with_expr(expr)`, `output_fields(..)`, `add_param(key, value)`, `metric_type(..)`, `with_metric_type(..)`, `radius(f32)`, `range_filter(f32)`, `partitions(..)`, `with_partitions(..)`
  - Purpose: Sets search Top-K, filter expressions, search field (via `add_param("anns_field", vec_field)`), metric type (like "L2"/"IP"), index/search parameters (like `"nprobe"`).
- `InsertOptions`
  - `new()`, `partition_name(name)`, `with_partition_name(name)`
  - Purpose: Selects partition when writing.
- `DeleteOptions`
  - `with_ids(ValueVec::Long/Str(..))`, `with_filter(expr)`, `partition_name(name)`
  - Purpose: Deletes by primary key or expression, can limit partitions.
- `IndexParams`
  - `new(name, IndexType, MetricType, HashMap)`; common `IndexType::IvfFlat/HNSW/Trie..`, `MetricType::L2/IP..`
  - Purpose: Defines index type and parameters.
- `IndexInfo` (returned by `describe_index`)
  - Common getters: `field_name()`, `id()`, `params()`, `state()`
  - Purpose: Views field name, ID, parameters and build state of an index.

---

For more context or end-to-end examples, please refer to the repository's built-in examples and tests:
- Examples: `examples/collection.rs`, `examples/index_example.rs`, `examples/query_search.rs`, `examples/iterator.rs`, `examples/authentication.rs`, `examples/database.rs`, `examples/resource_groups.rs`
- Tests: `tests/collection.rs`, `tests/partition.rs`, `tests/client.rs`, `tests/aggressivehpctesting/*` 