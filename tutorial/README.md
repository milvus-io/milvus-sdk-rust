# Milvus Rust SDK tutorials

This directory contains beginner-oriented tutorials and advanced administration guides for the
Milvus Rust SDK.

Each tutorial should be an independent Cargo project that depends on the published
`milvus-sdk-rust` crate rather than the SDK source tree. This keeps the tutorials close to the
experience of an application developer installing the SDK from crates.io.

Run a tutorial from the repository root with:

```bash
cargo run --manifest-path tutorial/<tutorial-name>/Cargo.toml
```

## Start here

If this is your first Milvus Rust SDK program, run the quick start first:

```bash
cargo run --manifest-path tutorial/1_quickstart/Cargo.toml
```

It expects Milvus at `http://localhost:19530` with the default `root:Milvus` credentials. For a
different server, set `MILVUS_URI` and `MILVUS_TOKEN`:

```bash
MILVUS_URI="https://your-milvus-endpoint" \
MILVUS_TOKEN="your-token" \
cargo run --manifest-path tutorial/1_quickstart/Cargo.toml
```

To start a disposable standalone Milvus container and run every tutorial, use
`scripts/run_tutorials.sh`. It is intended for Linux and requires Docker.

The source-linked examples used for SDK development and testing remain under `examples/`.

## Beginner tutorials

- [`1_quickstart`](1_quickstart/): connect, create a collection, insert data, search, and clean up.
- [`2_collection`](2_collection/): manage the collection lifecycle.
- [`3_schema`](3_schema/): define collection fields for the supported V2 data types.
- [`4_index`](4_index/): create and inspect vector and scalar indexes.
- [`5_dml`](5_dml/): insert, upsert, and delete collection data.
- [`6_dql`](6_dql/): query, search, hybrid search, and iterate through results.

## Advanced tutorials

- [`7_database`](7_database/): create, configure, select, and remove databases.
- [`8_rbac`](8_rbac/): manage users, roles, privilege groups, and grants.

The advanced tutorials change server-level resources. Run them only with an administrative
credential and on a disposable or explicitly approved Milvus instance.

## Common troubleshooting

- `connection refused`: Milvus is not running at `MILVUS_URI`, or the endpoint is not reachable.
- `unauthenticated` or `permission denied`: `MILVUS_TOKEN` is missing, invalid, or lacks permission
  for the requested resource.
- load/index timeout: verify that Milvus is healthy and has enough CPU and memory, then retry.
- `port is already allocated` from `scripts/run_tutorials.sh`: another container or tutorial run is
  using ports `29530` or `29091`; stop it or override `MILVUS_GRPC_PORT` and
  `MILVUS_HEALTH_PORT`.
- Docker daemon errors: start Docker and ensure the current user can run `docker info`.
