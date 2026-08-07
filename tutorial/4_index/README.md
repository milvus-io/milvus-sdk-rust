# Tutorial 4: Create and manage indexes

This standalone Cargo project uses `milvus-sdk-rust` version `2.6.0` to demonstrate indexes for
both vector and scalar fields.

The tutorial:

1. Creates a collection without indexes.
2. Creates an HNSW index for a float-vector field.
3. Creates an inverted index for a `VarChar` field.
4. Creates an STL_SORT index for a numeric scalar field.
5. Lists and describes the resulting indexes.
6. Drops one index and lists the remaining indexes.
7. Drops the tutorial collection.

## Index choices

Vector indexes require a compatible `MetricType`. This tutorial uses HNSW with cosine distance
and supplies its `M` and `efConstruction` build parameters. `AutoIndex` is a simpler alternative
when you want Milvus to choose the vector index implementation.

Common scalar choices include:

- `Inverted` for general scalar and text filtering.
- `StlSort` for ordered numeric values.
- `Trie` for prefix-oriented string lookup.
- `Bitmap` for low-cardinality scalar fields.
- `Rtree` for geometry fields.

## Prerequisites and run

Milvus 2.6 or later must be accessible. Connection settings come from `MILVUS_URI` and
`MILVUS_TOKEN`, defaulting to `http://localhost:19530` and `root:Milvus`.

```bash
cargo run --manifest-path tutorial/4_index/Cargo.toml
```

The program uses synchronous index creation with a 60-second overall wait limit and cleans up
the collection before exiting normally.
