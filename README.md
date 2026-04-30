# Milvus Rust SDK
Rust SDK for Milvus.

**This is still in progress, but should be already to run in your production environemnt, we are actively looking for maintainers of this repo**

## Get Started
Add the SDK into your project:
```
cargo add milvus-sdk-rust
```

Connect to milvus service and create collection:
```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    const URL: &str = "http://localhost:19530";

    let client = Client::new(URL).await?;

    let schema =
        CollectionSchemaBuilder::new("hello_milvus", "a guide example for milvus rust SDK")
            .add_field(FieldSchema::new_primary_int64(
                "id",
                "primary key field",
                true,
            ))
            .add_field(FieldSchema::new_float_vector(
                DEFAULT_VEC_FIELD,
                "feature field",
                256,
            ))
            .build()?;
    let collection = client.create_collection(schema.clone(), None).await?;
    Ok(())
}
```

## Development

### Prerequisites

- Rust toolchain with Cargo
- initialized `milvus-proto` submodule
- Docker or Docker Compose for integration tests

The protobuf compiler is provided through Cargo during build, so a system-installed `protoc` is not required.

Initialize submodules if needed:

```
git submodule update --init --recursive
```

### Build

This repository uses Cargo directly; there is no Makefile.

Build the SDK:

```
cargo build
```

Build an optimized release artifact:

```
cargo build --release
```

### Test

Many tests require a Milvus server at `localhost:19530`. Start one with the provided Docker Compose file:

```
docker-compose -f ./docker-compose.yml up -d
```

On systems using Docker Compose v2, use:

```
docker compose -f ./docker-compose.yml up -d
```

Wait until Milvus is ready, then run all tests:

```
cargo test
```

Run a single test target:

```
cargo test --test client_flush_collections -- --nocapture
```

Enable the full backtrace for debugging:

```
RUST_BACKTRACE=1 cargo test
```

### Clean

Remove build artifacts:

```
cargo clean
```

### Rebuild

Force a clean rebuild, including regenerated protobuf bindings:

```
cargo clean
cargo build
```
