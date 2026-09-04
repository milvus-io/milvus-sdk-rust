# AGENTS.md

Guidance for AI coding agents working in this repository. Setup, prerequisites, and build tooling
are in [DEVELOPMENT.md](DEVELOPMENT.md); this file focuses on conventions for writing and
reviewing code.

## Repository orientation

- The **V2 API** (`src/v2`, surfaced as `milvus::v2`) is the active SDK surface. Put all new
  feature work here. Active feature areas include collection, alias, partition, index, DML, DQL,
  iterators, database, RBAC, resource group, snapshot, CDC, bulk import, session, telemetry,
  and global-cluster routing.
- The **V1 API** (crate-root `milvus::client`, `schema`, `query`, `mutate`, `index`) is deprecated
  compatibility. Limit V1 edits to compatibility preservation, build fixes, security fixes, and
  critical correctness fixes.
- Generated protobuf bindings live in `OUT_DIR` (see `src/proto/mod.rs`). Never patch generated
  build output; change proto sources / `build.rs` instead.

## Universal V2 code style

- Every maintained Rust source file starts with the repository's canonical LF AI & Data foundation
  Apache-2.0 license header.
- Public V2 modules use `//!` module docs; public APIs and types use `///` docs.
- Put this exact separator immediately before every V2 struct/enum (name only; prose goes in
  `///` rustdoc below it):

  ```rust
  ///////////////////////////////////////////////////////////////////////////////
  // TypeName
  ///////////////////////////////////////////////////////////////////////////////
  ```

- Mark public request/response DTOs and extensible public enums `#[non_exhaustive]`; use wildcard
  match arms for extensible enums.
- Keep each type and its implementations together.

## Value types (`src/v2/types`)

- **Every public struct uses a zero-argument `new()`** that establishes SDK defaults; configure
  required values with fluent methods. Types produced only by decoding (e.g. response-like value
  types constructed via `from_proto`) may omit `new()`.
- **Do not add `Default` to structs** merely as an alias for `new()` (keep it only where an
  enum/sentinel genuinely needs it).
- **For each member, keep the method family adjacent and ordered**:
  1. `field(value) -> Self` — consuming fluent construction.
  2. `set_field(&mut self, value) -> &mut Self` — in-place mutation.
  3. `get_field(&self)` or a natural boolean `is_field(&self)` — reading.

  When a fluent `field(value)` setter occupies the plain name, use `get_field()` for the getter to
  avoid a Rust method collision (e.g. `get_size()`, `get_params()`, `get_last_element_offset()`).
  For `Copy` fields return by value; return `&str`/slices/references for owned data.
- **List/array/vector members** additionally provide a singular consuming `add_item(value) -> Self`
  after the member family.
- Keep enum/protobuf conversions `pub(crate)` unless intentionally public.
- Do not expose generated protobuf messages in SDK-owned public types.

## Request DTOs (`src/v2/request`)

- Users construct requests via `Request::builder()...build()?`; request fields are private.
- No `Default` on request DTOs; non-unit requests have a private `empty()` holding SDK defaults.
- Expose a public explicit `RequestBuilder`; builder setters are consuming and chainable
  (`field(value) -> Self`); `build(self) -> Result<Request>` validates required fields/ranges/
  mutual exclusions.
- Provide `into_builder(self)` on every request and idiomatic read-only `field()` accessors.
- Keep request→protobuf conversion `pub(crate)` and adjacent to the owning request; never expose
  raw `as_proto`/`proto_mut`.

## Response DTOs (`src/v2/response`)

- Responses are RPC outputs obtained from `ClientV2` methods; users do not construct them.
- No `Default`, no public constructors/setters/builders; expose read-only `field()` accessors.
- Keep protobuf decoding in `pub(crate) fn from_proto(...)`; propagate malformed payloads as typed
  V2 errors — never fabricate defaults or silently drop malformed fields.

## ClientV2

- Construct with `ClientV2::new(&ConnectConfig)`.
- Public feature methods accept an owned validated request DTO and return
  `crate::v2::error::Result<...>` (or a stream/task/iterator as appropriate).
- Route RPCs through the centralized retry helpers with explicit `RetrySemantics`; do not replay
  non-idempotent mutations (insert/upsert/delete/truncate/credential/resource-transfer/snapshot
  mutations) after ambiguous transport failures.

## Build, format, lint, test

- `cargo test --lib`, `cargo test --test v2_ut` (unit; no server), `cargo test --doc`.
- `cargo check --all-targets` (also compiles examples and server-backed system tests).
- `cargo fmt --all -- --check` and `git diff --check` before handoff.
- Server-backed system tests are under `tests/v2/st`; ask before starting Docker or running
  destructive tests.
- V2 examples live under `examples/v2` (V1 under `examples/v1`) and import
  `milvus::v2::prelude::*`; use uppercase collection names `RUST_V2_<EXAMPLE_NAME>`.

## Commits and PRs

- Commit with `git commit -sm '<message>'` so every commit carries a Signed-off-by trailer.
- Each PR must contain exactly **one** commit (squash before merging).
- Examples and system tests must clean up resources they create; prefer unique names in concurrent
  tests.
