# Tutorial 7: Manage databases (advanced)

This tutorial is a separate Cargo project that downloads `milvus-sdk-rust` version `3.0.0`
from crates.io. It demonstrates how to:

1. Connect with `ClientV2`.
2. List databases.
3. Create and describe a database.
4. Add and remove a database property.
5. Select a database with `use_database`.
6. Return to the default database and clean up.

## Prerequisites

- Rust and Cargo are installed.
- Milvus is running and accessible.
- `milvus-sdk-rust` version `3.0.0` has been published to crates.io.

The tutorial uses these environment variables:

| Variable | Default |
|---|---|
| `MILVUS_URI` | `http://localhost:19530` |
| `MILVUS_TOKEN` | `root:Milvus` |

## Run

From the SDK repository root:

```bash
cargo run --manifest-path tutorial/7_database/Cargo.toml
```

Or from this directory:

```bash
cargo run
```

The program creates a uniquely named database and drops it before exiting normally.

## Use another Milvus server

```bash
MILVUS_URI="https://your-milvus-endpoint" \
MILVUS_TOKEN="your-token" \
cargo run --manifest-path tutorial/7_database/Cargo.toml
```

Each RPC accepts a validated request object constructed with `Request::builder()...build()?`.
Responses expose read-only accessors such as `database_names()`, `database_name()`, and
`properties()`.

## Expected output

```text
Calling create_database: create "RUST_V2_DATABASE_..."
create_database completed
Calling alter_database_properties: set database.replica.number=1
alter_database_properties completed
Calling use_database: select "RUST_V2_DATABASE_..."
use_database completed
Calling drop_database: remove "RUST_V2_DATABASE_..."
drop_database completed
```
