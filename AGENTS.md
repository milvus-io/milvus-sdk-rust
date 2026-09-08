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

## Build, format, lint

- `cargo check --all-targets` (also compiles examples and server-backed system tests).
- `cargo fmt --all -- --check` and `git diff --check` before handoff.
- Use Clippy when requested or proportionate to the change.

## Examples

- V2 examples live under `examples/v2` (V1 under `examples/v1`).
- V2 examples import `milvus::v2::prelude::*` and use the V2 request/type APIs.
- Use uppercase collection names `RUST_V2_<EXAMPLE_NAME>`.
- Compile without running them when no live Milvus mutation is requested:
  `cargo build --examples`.
- Examples connect to Milvus and may create, modify, or delete resources; review connection
  settings before running and clean up resources they create.

## Tutorials

- Standalone application crates under `tutorial/` (`1_quickstart` through `8_rbac`), one per
  path in `tutorial/README.md`. Each pins the published `milvus-sdk-rust` version in its
  `Cargo.toml`; keep tutorial code and the pinned version mutually compatible in the same commit.
- Tutorial code changes (including README output and version references) land in lockstep with a
  release-prep PR that bumps the pin, as done for previous releases; do not update tutorial code
  to an unpublished API surface without also bumping its pinned dependency.
- Keep tutorial READMEs beginner-oriented: first-run command, default endpoint/credential
  assumptions, representative output, and concise troubleshooting.
- `scripts/run_tests.sh` compile-checks tutorials against the current checkout with a temporary
  Cargo patch; `scripts/run_tutorials.sh` runs them against one standalone Milvus container using
  the pinned crates.io version.

## Tests

The test suite has three tiers. Run the tier that matches the change; the fast tiers require no
Milvus server.

### Unit tests (`cargo test --lib`)

- In-crate `#[cfg(test)]` modules co-located with the code (e.g. `src/v2/request/*.rs`,
  `src/v2/client/*.rs`, `src/v2/types/*.rs`). No server, no network.
- Cover builder validation, protobuf encoding/decoding, value-type invariants, cache/retry logic,
  and internal helpers.
- Run with `cargo test --lib`. Prefer this tier for request/response/type logic that needs no RPC.

### Integration tests (`cargo test --test v2_ut`)

- `tests/v2/ut/` targets the `v2_ut` test binary. It exercises `ClientV2` methods against the
  local in-process `MockServer` (`tests/v2/ut/common.rs`), which records RPCs, stores server-side
  state, and can inject transport failures (`fail_next_transport`) or per-RPC error responses.
- No Milvus server or Docker required. Run with `cargo test --test v2_ut`.
- Add a mock handler in `tests/v2/ut/common.rs` when a feature dispatches a new RPC, and assert
  wire behavior (which RPC fired, request contents, fallback paths, retry/no-retry) here rather
  than in system tests.

### System tests (`cargo test --test v2_st` / `--test v1_st`)

- `tests/v2/st/` (target `v2_st`) and `tests/v1/` (target `v1_st`) run against a real Milvus
  server. They validate end-to-end behavior the mock cannot: real schema evolution, RBAC
  enforcement, server-side validation, and cross-component DML/DQL flows.
- They require Milvus at `http://localhost:19530` (override with `MILVUS_URI`) and may create,
  modify, and delete resources. Ask before starting Docker or running destructive tests.
- On Linux, `./scripts/run_tests.sh` starts a managed standalone Milvus container, runs the
  non-server checks and `v1_st`/`v2_st`, then removes the container.
- Server-backed tests must clean up resources they create; prefer unique names in concurrent tests.

### Doctests (`cargo test --doc`)

- `cargo test --doc` compiles and runs `///` examples. Keep them current with API changes.

## Commits and PRs

- Commit with `git commit -sm '<message>'` so every commit carries a Signed-off-by trailer.
- Each PR must contain exactly **one** commit (squash before merging).
- Examples and system tests must clean up resources they create; prefer unique names in concurrent
  tests.
