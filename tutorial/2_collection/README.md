# Tutorial 2: Manage collections

This tutorial is a separate Cargo project that downloads `milvus-sdk-rust` version `2.6.0`
from crates.io. It demonstrates how to:

1. Connect with `ClientV2` and list collections.
2. Define a schema and a vector index, then create a collection.
3. Check for and describe a collection.
4. Add and remove a collection property.
5. Load a collection and inspect its load state.
6. Read collection statistics.
7. Release, rename, and truncate a collection.
8. Drop the collection safely.

## Prerequisites

- Rust and Cargo are installed.
- Milvus is running and accessible.
- `milvus-sdk-rust` version `2.6.0` has been published to crates.io.

The tutorial uses these environment variables:

| Variable | Default |
|---|---|
| `MILVUS_URI` | `http://localhost:19530` |
| `MILVUS_TOKEN` | `root:Milvus` |

## Run

From the SDK repository root:

```bash
cargo run --manifest-path tutorial/2_collection/Cargo.toml
```

Or from this directory:

```bash
cargo run
```

The program creates a uniquely named collection and drops it before exiting normally. The
collection is created with an index on its vector field, so it can be loaded immediately.

## Use another Milvus server

```bash
MILVUS_URI="https://your-milvus-endpoint" \
MILVUS_TOKEN="your-token" \
cargo run --manifest-path tutorial/2_collection/Cargo.toml
```

Each RPC accepts a validated request object constructed with `Request::builder()...build()?`.
The example uses synchronous loading with a bounded timeout so it waits until the collection is
ready without waiting indefinitely.
