# Changelog

## milvus-sdk-rust 2.6.0 (2026-08-07)

### Feature

- Introduce `ClientV2`, a validated request/response-style API aligned with Milvus 2.6, while
  retaining the original `Client` API for compatibility in maintenance mode
- Connection: support database selection, connect timeout, RPC deadlines, keepalive, retries, TLS
  server-name override, custom CA certificates, and client certificates/private keys
- Collection and schema: support full and simple creation, schema and index creation, description,
  listing, statistics, replicas, rename, truncate, properties, fields, and collection functions
- Loading: support load, refresh, release, load-state progress, synchronous waiting, and bounded
  operation timeouts
- Partition and alias: support complete partition lifecycle and alias creation, retargeting,
  description, listing, and removal
- Database: support selection and complete database lifecycle, description, and property management
- Index: support creation with synchronous waiting, description, listing, removal, and property
  management for scalar and vector indexes
- DML: support row- and column-based insert/upsert, partial upsert, per-field update operations,
  delete, nullable/default values, dynamic fields, and advanced field types
- DQL: support get, query, search, hybrid search, primary-key ID queries/searches, grouping,
  full-text and text match, reranking, highlighting, and analyzer execution
- Iterators: support query and search iterators, including legacy and server-token search pagination
- Utility: support health and server-version inspection, flush/flush-all, segment listing,
  compaction, optimization, compaction state/plans, and asynchronous optimize tasks
- CDC and replication: support replication configuration, replication information, WAL message
  streaming, checkpoints, and salvage positions
- Resource groups: support lifecycle, configuration, description, listing, node transfer, and replica
  transfer operations
- RBAC: support users, roles, role membership, privileges, privilege groups, password updates, and
  user/role alteration
- Support nullable and default-valued fields, dynamic fields, struct arrays, geometry,
  timestamps with time zones, arrays, JSON, and dense, binary, half-precision, sparse, and Int8
  vector types
- Add borrowing `ResultRow`/`ResultRowIter` and type-preserving `ResultValue` APIs for efficient
  query and search result processing
- Add the Milvus 2.6 bulk-import REST interfaces: `bulk_import()`, `get_import_progress()`, and
  `list_import_jobs()`

### Improvement

- Align schema-cache and session-timestamp-cache behavior with the Milvus C++ and Java SDKs,
  including alias normalization, collection rename handling, lifecycle invalidation, per-client
  in-flight loads, cancellation safety, and effective TLS endpoint isolation
- Distinguish idempotent and non-idempotent retries so ambiguous transport failures do not replay
  committed DML, truncate, credential, or resource-group delta operations
- Avoid payload-sized request cloning on the normal insert and upsert paths and validate nested
  vector dimensions and narrowing numeric conversions before sending data
- Bound connection establishment, load, index, flush, and optimize polling by their configured
  deadlines
- Add typed V2 validation, conversion, server, timeout, cancellation, malformed-response, and
  retry-exhaustion errors
- Expand V2 examples, README usage guidance, crate publishing automation, and mock/system coverage
- Add standalone tutorials for databases, collections, schemas, indexes, DML, DQL, and bulk import


## milvus-sdk-rust 0.1.0 (2023-05-23)

### Feature

- Publish the initial Milvus Rust SDK to crates.io
- Provide the original asynchronous `Client` API for connecting to Milvus and performing core
  collection, partition, index, mutation, query, and search operations
- Support authenticated and TLS-enabled connections
