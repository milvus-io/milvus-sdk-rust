# Tutorial 5: Insert, upsert, and delete data

This standalone project demonstrates the V2 data-manipulation interfaces:

1. Insert rows with JSON objects using `row(...)`.
2. Insert a batch using column-oriented `FieldData` values.
3. Replace an existing entity with a full upsert.
4. Change selected fields with a partial upsert.
5. Delete entities by primary-key IDs.
6. Delete entities with a filter expression.
7. Query the remaining entities to verify the mutations.

An insert request must use either rows or columns, not both. All columns in one request must have
the same non-zero row count. Upserts require primary keys; partial upserts may omit unchanged
non-primary fields.

## Prerequisites and run

Milvus 2.6 or later must be accessible. Connection settings come from `MILVUS_URI` and
`MILVUS_TOKEN`, defaulting to `http://localhost:19530` and `root:Milvus`.

```bash
cargo run --manifest-path tutorial/5_dml/Cargo.toml
```

The program creates, loads, mutates, verifies, and drops a uniquely named collection.
