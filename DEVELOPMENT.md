# Developing the Milvus Rust SDK

New SDK functionality belongs in the V2 API. The original V1 API is maintained for compatibility
and should receive only compatibility, correctness, or security fixes.

## Prerequisites

- Rust toolchain with Cargo
- Git submodules initialized
- Linux server-backed tests: Python 3 and access to a running Docker daemon
- macOS compile and non-server tests: Homebrew; Docker is not required

The protobuf compiler is provided through Cargo, so a system-installed `protoc` is not required.

Initialize the repository submodules:

```shell
git submodule update --init --recursive
```

On supported Linux distributions and macOS, install the development dependencies with:

```shell
./scripts/install_deps.sh
```

The script installs the Rust toolchain, formatting and linting components, and platform build
dependencies. It attempts to install the optional coverage tools `cargo-llvm-cov` and the `lcov`
package that provides `genhtml`; failed optional installations produce warnings rather than failing
the complete setup. Set `SKIP_COVERAGE_TOOLS=true` to omit them.

On Linux, the script also installs Python 3 and Docker when needed, starts Docker when possible,
and verifies daemon access. On macOS, it uses Homebrew and prepares the compile and non-server test
environment.

## Build

Build the SDK or compile all targets:

```shell
cargo build
cargo check --all-targets
```

Build an optimized artifact with:

```shell
cargo build --release
```

Generated protobuf bindings are written to Cargo's `OUT_DIR`. Do not edit generated files under
`target/`.

## Tests without Milvus

The V2 test target uses local mock servers and does not require Milvus:

```shell
cargo test --test v2_ut
```

Run the repository's complete non-server validation path with:

```shell
./scripts/run_tests.sh --no-server
```

This compiles all targets, checks every standalone tutorial against the current checkout, runs
library tests and V2 mock-server tests, and compiles the doctests.

## Server-backed tests

When invoked directly, the V1 compatibility and V2 system targets require Milvus at
`http://localhost:19530`:

```shell
cargo test --test v1_st
cargo test --test v2_st
```

On Linux, the recommended command manages a standalone Milvus container automatically:

```shell
./scripts/run_tests.sh
```

The script runs non-server checks first, starts Milvus with embedded etcd and local storage, waits
for readiness, runs `v1_st` and `v2_st`, and removes the container afterward. It uses two system-test
threads by default. Increase or reduce this only when appropriate for the test host:

```shell
SYSTEM_TEST_THREADS=4 ./scripts/run_tests.sh
SYSTEM_TEST_THREADS=1 ./scripts/run_tests.sh
```

If direct server-backed tests report gRPC timeouts, reduce Cargo's test concurrency:

```shell
cargo test --test v2_st -- --test-threads=2
```

## Tutorials and examples

Examples under `examples/v2` and tutorials under `tutorial/` may create, modify, and delete Milvus
resources. Use a disposable or explicitly approved server.

Compile tutorials against the current checkout through the non-server test script. To run every
tutorial against the exact published crates.io version on Linux, start the repository-managed
standalone server with:

```shell
./scripts/run_tutorials.sh
```

The RBAC tutorial can exercise management calls on the default authorization-disabled server, but
permission enforcement requires a separately configured authorization-enabled Milvus instance.

## Formatting and linting

Check root-crate formatting before submitting changes:

```shell
cargo fmt --all -- --check
```

Tutorials are standalone Cargo projects rather than root-workspace members. If tutorial Rust files
changed, check them separately:

```shell
for manifest in tutorial/*/Cargo.toml; do
  cargo fmt --manifest-path "$manifest" -- --check
done
```

Run Clippy when appropriate for the change:

```shell
cargo clippy --all-targets --all-features
```

## Coverage

Coverage requires `cargo-llvm-cov` and `genhtml` from the `lcov` package:

```shell
CODE_COV=true ./scripts/run_tests.sh
```

The command writes the raw report to `code_coverage/lcov.info` and the HTML report to
`code_coverage/index.html`. Coverage is not supported with `run_tests.sh --no-server`.

## Protobuf regeneration and clean builds

The SDK compiles protobuf sources from the `milvus-proto` submodule into Cargo's build output. To
force regeneration after changing protobuf or build inputs:

```shell
cargo clean
cargo build
```

Do not commit files generated under `target/`.

## Debugging

Enable backtraces when investigating a Rust test failure:

```shell
RUST_BACKTRACE=1 cargo test
```

For a failing managed server run, the test script prints recent Milvus container logs before
cleanup.
