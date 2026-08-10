# Tutorial 7: Import prepared data

This tutorial is a separate Cargo project that downloads `milvus-sdk-rust` version `2.6.0`
from crates.io. It demonstrates all three Milvus 2.6 bulk-import REST interfaces:

1. `bulk_import()` creates an import job.
2. `list_import_jobs()` lists recent jobs for the collection.
3. `get_import_progress()` monitors a job until it completes or fails.

Bulk import is asynchronous. Creating a job only means that Milvus accepted the request; the
new data is not ready until the job reaches `Completed`.

## Prerequisites

- Rust and Cargo are installed.
- Milvus 2.6 is running and accessible.
- `milvus-sdk-rust` version `2.6.0` has been published to crates.io.
- The target collection already exists.
- Prepared JSON or Parquet files have already been uploaded to the S3-compatible bucket and path
  configured for the Milvus server.

The values passed to `files()` must be object keys in the storage bucket used by Milvus. They are
relative to that bucket's configured root path. They cannot be local paths on the machine running
this Rust program, arbitrary HTTP URLs, or objects in a different bucket that Milvus cannot
access. Do not include an `s3://bucket-name/` prefix when using `files()`.

This Rust SDK tutorial covers the REST import job APIs. It does not generate or upload prepared
files. Use a compatible BulkWriter or your existing data pipeline for that preparation step, then
pass the resulting object keys from the Milvus storage bucket to this program.

## Prepare the source data

The prepared data must match the target collection schema:

- Field names and data types must match the collection fields.
- If the primary field uses AutoID, omit that field from the source rows.
- Extra source fields require dynamic fields to be enabled; Milvus stores them in `$meta`.
- For JSON and Parquet, put one bucket-relative object key in each inner file group.
- Prefer submitting multiple prepared files in one job for better import throughput.

For example, two Parquet files are represented as:

```text
folder/1.parquet;folder/2.parquet
```

The tutorial converts that value to:

```rust
vec![
    vec!["folder/1.parquet"],
    vec!["folder/2.parquet"],
]
```

Use a comma only when several files belong to one logical group, and a semicolon to start the
next group.

## Configuration

| Variable | Required | Default | Meaning |
|---|---:|---|---|
| `MILVUS_COLLECTION` | yes | — | Existing target collection |
| `MILVUS_IMPORT_FILES` | yes | — | Object keys in the Milvus S3 bucket; separate groups with semicolons and grouped keys with commas |
| `MILVUS_URI` | no | `http://localhost:19530` | Milvus REST endpoint |
| `MILVUS_TOKEN` | no | `root:Milvus` | API key or `username:password` token |
| `MILVUS_DATABASE` | no | empty/default database | Target database |
| `MILVUS_PARTITION` | no | default partition | Target partition |

Do not set `MILVUS_PARTITION` when the collection uses a partition key.

## Run

```bash
MILVUS_COLLECTION="quick_setup" \
MILVUS_IMPORT_FILES="a1e18323/1.parquet;a1e18323/2.parquet" \
cargo run --manifest-path tutorial/7_import_data/Cargo.toml
```

In this example, Milvus reads `a1e18323/1.parquet` and `a1e18323/2.parquet` from the
S3-compatible bucket configured for that Milvus deployment.

The program prints the created job ID, lists recent jobs, and checks progress every five seconds.
It stops after ten minutes rather than polling forever.

## Cloud object URLs and volumes

`BulkImportRequest` also supports the Milvus 2.6 cloud inputs:

```rust
let object_storage_request = BulkImportRequest::builder()
    .collection_name("books")
    .cluster_id("cluster-id")
    .object_urls([["s3://bucket/path/books.parquet"]])
    .access_key("access-key")
    .secret_key("secret-key")
    .token("temporary-session-token")
    .build()?;

let volume_request = BulkImportRequest::builder()
    .collection_name("books")
    .cluster_id("cluster-id")
    .volume_name("my-volume")
    .data_paths([["prepared/books.parquet"]])
    .build()?;
```

For a project database, use `project_id()` and `region_id()` together instead of `cluster_id()`.
The deprecated singular `object_url()` is available for compatibility, but new code should use
`object_urls()`.

## After completion

- Imported data is not guaranteed to be visible before the state becomes `Completed`.
- If the collection was not loaded, load it after completion.
- If it was already loaded, refresh the load so the imported segments become queryable.
- Avoid relying on deletes against the imported data before the job completes.

Milvus 2.6 limits each file to 16 GB and each request to at most 1,024 files. The official docs
also state a maximum of 1,024 concurrent import requests.

See the official Milvus 2.6 guides:

- [Prepare Source Data](https://milvus.io/docs/v2.6.x/prepare-source-data.md)
- [Import Data](https://milvus.io/docs/v2.6.x/import-data.md)
