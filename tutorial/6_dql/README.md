# Tutorial 6: Query and search data

This standalone project demonstrates the main V2 data-query interfaces:

1. `query` for scalar filtering and selected output fields.
2. `search` for nearest-neighbor search on one vector field.
3. `hybrid_search` for combining dense and sparse searches with weighted reranking.
4. `query_iterator` for reading filtered entities in bounded batches.
5. `search_iterator` for iterating a larger nearest-neighbor result set.

The program creates a collection with dense and sparse vector indexes, inserts a small data set,
loads the collection, runs every operation, and drops the collection afterward.

Results are read with `ResultRowIter` and `ResultRow`. Typed getters such as `get_i64`, `get_str`,
and `get_f32` avoid materializing each row as an owned JSON map. `ResultRow::get` returns a
generic `ResultValue` when the field type is not known by the application.

## Prerequisites and run

Milvus 2.6 or later must be accessible. Connection settings come from `MILVUS_URI` and
`MILVUS_TOKEN`, defaulting to `http://localhost:19530` and `root:Milvus`.

```bash
cargo run --manifest-path tutorial/6_dql/Cargo.toml
```

The iterator examples deliberately use small batch and total limits so their pagination is easy
to observe.
