# Milvus Rust SDK tutorials

This directory contains beginner-oriented tutorials for the Milvus Rust SDK.

Each tutorial should be an independent Cargo project that depends on the published
`milvus-sdk-rust` crate rather than the SDK source tree. This keeps the tutorials close to the
experience of an application developer installing the SDK from crates.io.

Run a tutorial from the repository root with:

```bash
cargo run --manifest-path tutorial/<tutorial-name>/Cargo.toml
```

The source-linked examples used for SDK development and testing remain under `examples/`.

## Tutorials

- [`1_database`](1_database/): connect to Milvus and use the database-management interfaces.
- [`2_collection`](2_collection/): define a schema and use the collection lifecycle interfaces.
- [`3_schema`](3_schema/): define collection fields for every supported V2 data type.
- [`4_index`](4_index/): create and inspect indexes for vector and scalar fields.
- [`5_dml`](5_dml/): insert, upsert, and delete collection data.
- [`6_dql`](6_dql/): query, search, hybrid search, and iterate through results.
- [`7_import_data`](7_import_data/): submit bulk-import jobs, monitor progress, and list jobs.
