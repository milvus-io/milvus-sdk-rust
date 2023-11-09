# Milvus Rust SDK
Rust SDK for Milvus.

**This is still in progress, be careful to use it in your production**

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

Pre-requisites:
- cargo
- protocol-compiler
- docker (for testing)

### How to test
Many tests require the Milvus server, the project provide a docker-compose file to setup a Milvus cluster:
```
docker-compose -f ./docker-compose.yml up -d
```
You may need to wait for seconds until the system ready

Run all tests:
```
cargo test
```

Enable the full backtrace for debugging:
```
RUST_BACKTRACE=1 cargo test
```

test