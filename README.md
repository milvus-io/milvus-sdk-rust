# Milvus Rust SDK

Rust SDK for [Milvus](https://milvus.io/).

New applications should use `ClientV2`, which provides the current request/response-style API.
The original `Client` API is retained for compatibility with existing applications and is in
maintenance mode.

## Quick start

Add the SDK to your project:

```shell
cargo add milvus-sdk-rust
```

Connect to Milvus with `ClientV2` and check the server health:

```rust
use milvus::v2::error::Result;
use milvus::v2::request::utility::CheckHealthRequest;
use milvus::v2::{ClientV2, ConnectConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let config = ConnectConfig::new().uri("http://localhost:19530");
    let client = ClientV2::new(&config).await?;
    let health = client
        .check_health(CheckHealthRequest::builder().build()?)
        .await?;

    println!("Milvus is healthy: {}", health.is_healthy());
    Ok(())
}
```

### Basic V2 workflow

The following snippets continue with the connected `client` above and use a four-dimensional
example collection:

```rust
use milvus::v2::prelude::*;
use serde_json::json;

const COLLECTION: &str = "RUST_V2_README";
const DIMENSION: u32 = 4;
```

1. Create a collection with an integer primary key, a float-vector field, and an index for the
   vector field. A vector index is required before the collection can be loaded.

   ```rust
   let schema = CollectionSchema::new()
       .enable_dynamic_field(true)
       .add_field(
           FieldSchema::new()
               .name("id")
               .data_type(DataType::Int64)
               .primary_key(true),
       )
       .add_field(
           FieldSchema::new()
               .name("vector")
               .data_type(DataType::FloatVector)
               .dimension(DIMENSION),
       );

   client
       .create_collection(
           CreateCollectionRequest::builder()
               .collection_name(COLLECTION)
               .schema(schema)
               .index_param(
                   IndexParam::new()
                       .field_name("vector")
                       .index_type(IndexType::AutoIndex)
                       .metric_type(MetricType::Cosine),
               )
               .build()?,
       )
       .await?;

   client
       .load_collection(
           LoadCollectionRequest::builder()
               .collection_name(COLLECTION)
               .timeout_ms(60_000)
               .build()?,
       )
       .await?;
   ```

2. Insert rows one at a time with `row()`. The schema enables dynamic fields, so `title` does not
   need to be declared explicitly.

   ```rust
   let inserted = client
       .insert(
           InsertRequest::builder()
               .collection_name(COLLECTION)
               .row(json!({
                   "id": 1,
                   "title": "Rust",
                   "vector": [0.1, 0.2, 0.3, 0.4]
               }))
               .row(json!({
                   "id": 2,
                   "title": "Milvus",
                   "vector": [0.4, 0.3, 0.2, 0.1]
               }))
               .build()?,
       )
       .await?;
   println!("Inserted {} rows", inserted.insert_count());
   ```

3. Insert a prepared batch with `rows()`.

   ```rust
   let rows: Vec<EntityRow> = serde_json::from_value(json!([
       {
           "id": 3,
           "title": "Vector database",
           "vector": [0.2, 0.3, 0.4, 0.5]
       },
       {
           "id": 4,
           "title": "Similarity search",
           "vector": [0.5, 0.4, 0.3, 0.2]
       }
   ]))?;

   let inserted = client
       .insert(
           InsertRequest::builder()
               .collection_name(COLLECTION)
               .rows(rows)
               .build()?,
       )
       .await?;
   println!("Inserted {} batched rows", inserted.insert_count());
   ```

4. Insert by columns. Undeclared dynamic fields are supplied as JSON objects in the `$meta`
   column.

   ```rust
   let inserted = client
       .insert(
           InsertRequest::builder()
               .collection_name(COLLECTION)
               .columns(vec![
                   FieldData::int64("id", vec![5, 6]),
                   FieldData::float_vector(
                       "vector",
                       vec![vec![0.2, 0.3, 0.4, 0.5], vec![0.5, 0.4, 0.3, 0.2]],
                   ),
                   FieldData::json(
                       "$meta",
                       vec![json!({"title": "Embeddings"}), json!({"title": "Search"})],
                   ),
               ])
               .build()?,
       )
       .await?;
   println!("Inserted {} column-based rows", inserted.insert_count());
   ```

5. Search with one query vector and request the dynamic `title` field.

   ```rust
   let search = client
       .search(
           SearchRequest::builder()
               .collection_name(COLLECTION)
               .vector_field("vector")
               .vectors(SearchVectors::Float(vec![vec![0.1, 0.2, 0.3, 0.4]]))
               .output_fields(["title"])
               .limit(4)
               .consistency_level(ConsistencyLevel::Strong)
               .build()?,
       )
       .await?;
   ```

6. Interpret results through borrowing rows or column-oriented fields. `rows()` yields
   `ResultRow` values that borrow the decoded columns, avoiding the JSON allocation performed by
   `get_output_row()`, `get_output_rows()`, or `to_entity_row()`. Query rows expose their requested
   output fields. Search rows additionally expose the primary key, score, and, for element-level
   searches over struct arrays or arrays of vectors, an optional element offset.

   Use typed getters such as `get_i64()`, `get_str()`, and `get_float_vector()` when the schema is
   known. Use `get()` when writing generic result-processing code; it returns a borrowed
   `ResultValue` that preserves the field's actual scalar, array, struct, or vector type. Both forms
   report missing fields and incompatible types through `Result`.

   ```rust
   for result in search.results() {
       // Borrowing row-oriented access with typed values.
       for row in result.rows()? {
           let id = row.get_i64("id")?;
           let title = row.get_str("title")?;
           let score = row.get_f32("score")?;
           println!("id={id}, title={title}, score={score}");

           if let Some(offset) = row.element_offset() {
               println!("matched element offset={offset}");
           }

           // Type-preserving access when the field type is determined at runtime.
           match row.get("title")? {
               ResultValue::String(value) => println!("title={value}"),
               ResultValue::Null => println!("title=null"),
               value => println!("title has another type: {value:?}"),
           }
       }

       // Column-oriented access.
       println!("ids: {:?}", result.get_ids());
       println!("scores: {:?}", result.get_scores());
       if let Some(titles) = result
           .get_output_field("title")
           .and_then(|field| field.as_varchar())
       {
           println!("titles: {titles:?}");
       }
   }
   ```

7. Clean up the collection when finished.

   ```rust
   client
       .drop_collection(
           DropCollectionRequest::builder()
               .collection_name(COLLECTION)
               .build()?,
       )
       .await?;
   ```

More V2 examples are available under [`examples/v2`](examples/v2). For example:

```shell
cargo run --example v2_simple
```

The examples create or modify Milvus resources and expect a server at
`http://localhost:19530` unless otherwise documented.

## Tutorials

Beginner-oriented, standalone Cargo projects are available under [`tutorial`](tutorial/README.md).
They use the published SDK as an application dependency and provide this learning sequence:

1. [`Database`](tutorial/1_database/)
2. [`Collection`](tutorial/2_collection/)
3. [`Schema`](tutorial/3_schema/)
4. [`Index`](tutorial/4_index/)
5. [`DML`](tutorial/5_dml/)
6. [`DQL`](tutorial/6_dql/)
7. [`Import data`](tutorial/7_import_data/)

Each tutorial contains its own prerequisites, configuration, and run instructions. Start with the
[`tutorial index`](tutorial/README.md) for an overview.

## Development

### Prerequisites

- Rust toolchain with Cargo
- Initialized `milvus-proto` submodule
- Linux server-backed tests: Python 3 and access to a running Docker daemon
- macOS compile and non-server tests: Homebrew; Docker is not required

The protobuf compiler is provided through Cargo, so a system-installed `protoc` is not
required.

Initialize the submodules if needed:

```shell
git submodule update --init --recursive
```

On supported Linux distributions and macOS, install the build and coverage tools with:

```shell
./scripts/install_deps.sh
```

The script installs the Rust toolchain and formatting/linting components, and attempts to install
the optional coverage tools `cargo-llvm-cov` and the `lcov` package that provides `genhtml`. If an
optional coverage installation fails, the script warns and continues; coverage commands will then
report the missing tool. Set `SKIP_COVERAGE_TOOLS=true` to omit these optional tools. On Linux the
script also installs Python 3 and Docker when needed, starts the Docker service when possible, and
verifies that the current user can access the daemon. On macOS it uses Homebrew and prepares the
compile and non-server test environment.

### Build

Build the SDK:

```shell
cargo build
```

Build an optimized release artifact or compile all targets:

```shell
cargo build --release
cargo check --all-targets
```

### Test

Run V2 unit tests, which use a mock server and do not require a Milvus instance:

```shell
cargo test --test v2_ut
```

The V1 compatibility tests and V2 system tests require Milvus at `localhost:19530`:

```shell
cargo test --test v1_st
cargo test --test v2_st
```

When running Milvus-backed tests, `cargo test` may report gRPC `Timeout expired` errors if too
many tests access the server concurrently. Limit the number of test threads to reduce contention:

```shell
cargo test -- --test-threads=4
```

If timeouts still occur, use a lower value such as `--test-threads=2`.

`scripts/run_tests.sh` uses two threads for system tests by default. Override this only when the
Milvus test host has enough resources:

```shell
SYSTEM_TEST_THREADS=4 ./scripts/run_tests.sh
```

On Linux, the repository test script first runs the library, V2 unit, and documentation tests.
After those pass, `tests/v2/st/milvus_container.py` starts one standalone Milvus container with
embedded etcd, local storage, and the default standalone WAL. It waits for Milvus to become ready,
runs the V1 and V2 system tests, and removes the container afterward:

```shell
./scripts/run_tests.sh
```

To compile all targets and run only tests that do not require Milvus, use:

```shell
./scripts/run_tests.sh --no-server
```

To generate LCOV and HTML coverage reports, install
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) and the `lcov` package, which
provides `genhtml`, then run:

```shell
CODE_COV=true ./scripts/run_tests.sh
```

The raw coverage report is written to `code_coverage/lcov.info`. The human-readable report is
written to `code_coverage/index.html`.

Enable full backtraces when debugging a test directly:

```shell
RUST_BACKTRACE=1 cargo test
```

### Format

Check formatting before submitting changes:

```shell
cargo fmt --check
```

### Clean and rebuild

Remove build artifacts and force regeneration of protobuf bindings:

```shell
cargo clean
cargo build
```
