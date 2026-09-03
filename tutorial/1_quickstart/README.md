# Tutorial 1: Quick start

This is the smallest complete Milvus application in the Rust SDK. It connects to Milvus,
creates a collection, inserts a few rows, searches a vector, prints the results, and removes the
collection before exiting.

## Prerequisites

- Rust and Cargo are installed.
- Milvus 2.6 or later is running and accessible.
- `milvus-sdk-rust` version `2.6.1` has been published to crates.io.

Connection settings use `MILVUS_URI` and `MILVUS_TOKEN`, defaulting to
`http://localhost:19530` and `root:Milvus`.

## Run

```bash
cargo run --manifest-path tutorial/1_quickstart/Cargo.toml
```

The collection name is unique for each run. The program cleans it up on normal completion.

## Expected output

The IDs and scores vary, but the run should include messages like:

```text
Calling create_collection: create "RUST_V2_QUICKSTART_..."
create_collection completed
Calling search: find the two nearest embedding matches
search completed
Calling drop_collection: remove "RUST_V2_QUICKSTART_..."
drop_collection completed
```

## Troubleshooting

- `connection refused`: start Milvus or set `MILVUS_URI` to the reachable endpoint.
- `unauthenticated` or `permission denied`: set a valid `MILVUS_TOKEN`.
- `timed out waiting for collection`: check that the Milvus server is healthy and has enough
  resources to build the vector index.
