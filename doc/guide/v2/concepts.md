# V2 Concepts and Data Model

This page explains the core ideas behind the Rust SDK V2 API: request validation, consistency,
indexes and metrics, row access, and error handling.

## Requests are validated builders

Every operation is configured with a typed request built through `RequestType::builder()`.
Builders validate required fields, value ranges, and mutually exclusive inputs in `build()`
before any RPC is sent:

```rust,no_run
use milvus::v2::prelude::*;

fn build() -> Result<()> {
    // Both an insert request needs a collection name...
    let ok = InsertRequest::builder().collection_name("books").build()?;
    assert_eq!(ok.collection_name(), "books");

    // ...and a delete request needs exactly one selection form.
    let _ = DeleteRequest::builder()
        .collection_name("books")
        .filter("id in [1, 2, 3]")
        .build()?;
    Ok(())
}
```

A built request is immutable; call `into_builder()` to copy it back into a builder for edits.

## Field types and vectors

`DataType` describes the server-side schema type; `FieldData` holds column-oriented values in
memory (used by DML requests and returned by reads). Vector types include dense `FloatVector`,
`BinaryVector`, `Float16Vector`, `BFloat16Vector`, `Int8Vector`, and `SparseFloatVector`.

- [`DataType`](crate::v2::types::DataType)
- [`FieldData`](crate::v2::types::FieldData)

## Consistency levels

Reads (query, get, search, hybrid search) can carry a `ConsistencyLevel`:

- `Strong` reads see all data committed before the request started.
- `Session` reads see data written earlier by the same session (backed by a per-collection
  timestamp the SDK records after successful DML).
- `Bounded` tolerates a bounded staleness window for lower latency.
- `Eventually` has the weakest guarantee and lowest latency.
- `Customized` (the SDK default) sends no explicit level, letting the server apply its own
  configured policy.

The SDK maintains an endpoint/database/collection timestamp cache that powers Session
consistency automatically.

## Indexes and metrics

`IndexParam` selects an `IndexType` and a `MetricType` for a field. The metric determines how
similarity is scored (`L2`, `IP`, `Cosine`, ...). Index creation is asynchronous on the server;
`sync(true)` (the default) waits for the index to finish building, bounded by `timeout_ms`.

- [`IndexType`](crate::v2::types::IndexType)
- [`MetricType`](crate::v2::types::MetricType)

## Reading results

`QueryResponse` and `SearchResponse` own decoded column data. Use the borrowing
`ResultRowIter` returned by `rows()` for typed per-row access without allocating JSON maps, or
`get_output_rows()` when owned JSON objects are needed (for example, to serialize rows).

Search responses expose one `SingleResult` per query vector, each carrying `get_ids()`,
`get_scores()`, and requested output fields.

## Error handling

All fallible client methods return `milvus::v2::error::Result<T>` (= `std::result::Result<T, Error>`).
The `Error` enum distinguishes:

- `Server` — the server rejected the request (code, legacy code, reason).
- `Validation` — local request validation failed before the RPC.
- `Conversion` — SDK/JSON/protobuf value conversion failed.
- `Timeout` — the operation exceeded its configured deadline.
- `Grpc` — a transport-level gRPC failure.
- `RetryExhausted` — all retry attempts were consumed.

```rust,no_run
use milvus::v2::prelude::*;

async fn handle(client: &ClientV2) {
    let result = client
        .query(QueryRequest::builder().collection_name("books").filter("id > 0").build().unwrap())
        .await;
    match result {
        Ok(_) => {}
        Err(Error::Validation(error)) => eprintln!("invalid request: {}", error.reason()),
        Err(Error::Server(error)) => eprintln!("server error code {}: {}", error.code(), error.reason()),
        Err(other) => eprintln!("operation failed: {other}"),
    }
}
```

## Retries

Retry configuration lives in `RetryConfig` (attempts, backoff, timeout, and whether rate-limit
responses may be retried). Non-idempotent mutations (insert, upsert, delete, truncate) are never
replayed after ambiguous transport failures; idempotent reads and lifecycle operations are retried
according to `RetryConfig`.

## See also

- [Quick start](#milvus-rust-sdk-v2-quick-start)
- [`v2`](crate::v2) API reference
