# Tutorial 3: Define collection schemas

A schema defines the fields that each entity in a collection can contain. This standalone Cargo
project downloads `milvus-sdk-rust` version `2.6.0` from crates.io and demonstrates every usable
V2 `DataType`.

The program creates three collections so the examples remain within common Milvus vector-field
limits:

1. Scalar and container fields.
2. Dense and binary vector fields.
3. Sparse and `Int8Vector` fields plus a struct-array field with scalar and vector sub-fields.

It describes the collections to show the schemas returned by Milvus, then drops all tutorial
collections.

## Data types

| Category | `DataType` | Important schema settings |
|---|---|---|
| Boolean | `Bool` | Optional `nullable` or `default_value` |
| Integer | `Int8`, `Int16`, `Int32`, `Int64` | Primary keys may use `Int64` |
| Floating point | `Float`, `Double` | `Float` stores 32-bit values; `Double` stores 64-bit values |
| Text | `VarChar` | Set `max_length`; a primary key may use `VarChar` instead of `Int64` |
| Document | `Json` | Stores structured JSON values |
| Spatial | `Geometry` | Stores geometry in Well-Known Text form |
| Time | `Timestamptz` | Stores timestamp-with-time-zone values |
| Array | `Array` | Set `element_type` and `max_capacity`; string arrays also need `max_length` |
| Struct array | `Struct` | Construct with `StructFieldSchema`, a positive `max_capacity`, and sub-fields |
| Dense vector | `FloatVector`, `Float16Vector`, `BFloat16Vector`, `Int8Vector` | Set a positive `dimension` |
| Binary vector | `BinaryVector` | Set a positive bit `dimension`, normally divisible by 8 |
| Sparse vector | `SparseFloatVector` | No fixed dimension is declared |

`DataType::Unknown` is the SDK's unset sentinel. It is not valid for a field in a built
`CreateCollectionRequest`.

An array element can be `Bool`, `Int8`, `Int16`, `Int32`, `Int64`, `Float`, `Double`, or
`VarChar`. A struct-array field is represented by `StructFieldSchema` rather than by constructing
a top-level `FieldSchema` with `DataType::Struct` directly.

## Schema rules demonstrated

- A collection has exactly one primary key, using `Int64` or `VarChar`.
- Dense and binary vectors require `dimension(...)`; sparse vectors do not.
- `VarChar` uses `max_length(...)`.
- `Array` uses `element_type(...)` and `max_capacity(...)`.
- `nullable(true)` permits a missing or null value.
- `default_value(...)` supplies a value when a field is omitted.
- `StructFieldSchema` defines an array of structured elements and may contain vector sub-fields.
- `enable_dynamic_field(false)` rejects fields that are not declared by the schema.

## Prerequisites

- Rust and Cargo are installed.
- Milvus 2.6 or later is running and accessible.
- `milvus-sdk-rust` version `2.6.0` has been published to crates.io.

The tutorial uses these environment variables:

| Variable | Default |
|---|---|
| `MILVUS_URI` | `http://localhost:19530` |
| `MILVUS_TOKEN` | `root:Milvus` |

## Run

From the SDK repository root:

```bash
cargo run --manifest-path tutorial/3_schema/Cargo.toml
```

Or from this directory:

```bash
cargo run
```

To connect to another Milvus server:

```bash
MILVUS_URI="https://your-milvus-endpoint" \
MILVUS_TOKEN="your-token" \
cargo run --manifest-path tutorial/3_schema/Cargo.toml
```
