# Changelog

## milvus-sdk-rust 3.0.0 (2026-09-03)

### Feature

- Introduce a session client (`ClientV2::session`) that routes DQL operations (query, search,
  hybrid search, get, and iterators) to a target global-cluster member, and recognize
  `*-global-cluster-*` endpoints with topology discovery, primary selection, background refresh,
  and proactive/reactive failover across generations
- Add default-on client telemetry to `ClientV2`, shared by clones and cluster-scoped sessions,
  reporting the seven Go SDK operation families, with heartbeats and the five server commands
  (`push_config`, `collection_metrics`, `show_errors`, `show_latency_history`, `get_config`)
- Support snapshots: create/drop/list/describe, restore with state and job listing, and pin/unpin
  snapshot data
- Support external collections: refresh, progress, and job listing for file-resource-backed
  external tables
- Support struct-field and function-field schema evolution: `add_collection_struct_field`,
  `add_function_field`, `drop_function_field`, and `drop_collection_field`
- Support search-time function chains (rerank/refine) and hierarchical bucket search aggregation
  with typed `FunctionChain`, `SearchAggregation`, `MetricSpec`, `SortSpec`, `TopHitsSpec`, and
  nested sub-aggregations
- Support resumable query iterator cursors, so a query can continue from a previously captured
  cursor after a disconnect or process restart
- Support nullable struct sub-fields with wire-level `nullable` propagation on insert and query
- `list_indexes`: add a `field_name` filter; returned indexes are filtered by field name when set
- `compact`: add `l0_compaction` and `target_size_unit` (`TargetSizeUnit`: B/KB/MB/GB/TB/PB),
  normalized at `build()` time so `into_builder().build()` round-trips idempotently
- `grant_privilege` / `revoke_privilege`: add the V1 object-scoped form via `object_type` and
  `object_name`, routed to the legacy `OperatePrivilege` RPC when set

### Improvement

- Align validation with pymilvus: enforce `round_decimal` in `-1..=6` on `search`,
  `hybrid_search`, and `search_iterator`, reject `compact.target_size <= 0`, and treat empty
  insert/upsert data and empty get ids as short-circuited empty results instead of build errors
- Keep the deprecated V1 surface compiling through the Milvus 3.x proto upgrade

### Breaking change

- This is the first major release targeting Milvus `3.x`; the `2.6.x` line remains supported for
  existing deployments
- `compact`: a `target_size` of `0` is now rejected; callers that passed `0` to mean "server
  default" must omit the field so the server picks its default target size
- `insert`/`upsert`/`get`: empty input now builds and returns an empty result without issuing an
  RPC instead of failing at request construction

## milvus-sdk-rust 2.6.0 (2026-08-14)

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
- Add a beginner quick-start tutorial, focused collection/schema/index/DML/DQL tutorials, and
  advanced database and RBAC administration tutorials
- Improve V1 rustdoc examples so authentication, database, query, index, and resource-group
  examples compile with the published crate paths and valid Milvus URLs
- Replace Docker Compose test setup with a signal-safe standalone Milvus launcher, add macOS
  non-server validation, and verify published-crate tutorials after release
- Add complete V2 Rustdoc coverage, clearer V1/V2 navigation, standalone development guidance, and
  optional `tracing` diagnostics for retries, schema/session caches, iterator paging, and polling
- Define and validate Rust 1.86 as the MSRV, complete the crates.io package metadata, and restrict
  published package contents to the SDK sources, examples, tests, protobuf inputs, and release
  documents
- Add Windows CI, strict Rustdoc/package validation, Clippy correctness/suspicious enforcement, and
  mechanical checks for V2 DTO extensibility, builders, accessors, setters, and protobuf boundaries
- Harden the release workflow by validating tag ancestry and successful post-merge checks, running
  tutorials against the tagged local SDK before publication, and validating the crates.io artifact
  before creating the GitHub release


## milvus-sdk-rust 0.1.0 (2023-05-23)

### Feature

- Publish the initial Milvus Rust SDK to crates.io
- Provide the original asynchronous `Client` API for connecting to Milvus and performing core
  collection, partition, index, mutation, query, and search operations
- Support authenticated and TLS-enabled connections
