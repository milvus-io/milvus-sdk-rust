# Milvus Rust SDK

The official Rust SDK for [Milvus](https://milvus.io/).

## Compatibility

The current `3.x` SDK release line targets Milvus `3.x`. See [CHANGELOG.md](CHANGELOG.md) for
release-specific features, fixes, and compatibility notes.

The minimum supported Rust version (MSRV) is Rust `1.86`.

## Use the SDK in your project

### Choose the client API

New applications should use `ClientV2`, which provides the current request/response-style API.
The original `Client` API is retained for compatibility with existing applications and is in
maintenance mode.

### Add dependencies

Create a Rust application and add the SDK and Tokio runtime:

```shell
cargo new milvus-quickstart
cd milvus-quickstart
cargo add milvus-sdk-rust
cargo add tokio --features macros,rt-multi-thread
```

### Minimal ClientV2 program

The following program connects to Milvus and checks server health:

```rust
use milvus::v2::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI")
        .unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN")
        .unwrap_or_else(|_| "root:Milvus".to_owned());

    let client = ClientV2::new(&ConnectConfig::new().uri(uri).token(token)).await?;
    let health = client
        .check_health(CheckHealthRequest::builder().build()?)
        .await?;

    println!("Milvus is healthy: {}", health.is_healthy());
    Ok(())
}
```

Run it with:

```shell
cargo run
```

This program expects Milvus at `http://localhost:19530` and uses `root:Milvus` by default. For
another server, set `MILVUS_URI` and `MILVUS_TOKEN`. See the
[Milvus installation guide](https://milvus.io/docs/install-overview.md) for deployment options.

For an independently buildable application that covers collection creation, insertion, loading,
search, result handling, and cleanup, follow [Tutorial 1: Quick start](tutorial/1_quickstart/).

## Tutorials

Standalone Cargo projects are available under [`tutorial`](tutorial/README.md). They consume the
published SDK as an application dependency and provide a beginner-to-advanced sequence:

1. [`Quick start`](tutorial/1_quickstart/)
2. [`Collection`](tutorial/2_collection/)
3. [`Schema`](tutorial/3_schema/)
4. [`Index`](tutorial/4_index/)
5. [`DML`](tutorial/5_dml/)
6. [`DQL`](tutorial/6_dql/)
7. [`Database (advanced)`](tutorial/7_database/)
8. [`RBAC (advanced)`](tutorial/8_rbac/)

Each tutorial has its own prerequisites, configuration, run command, expected output, and
troubleshooting guidance. Start with the [`tutorial index`](tutorial/README.md) for shared
connection settings and the recommended learning path.

On Linux, repository maintainers can start a disposable standalone Milvus server and run every
tutorial with:

```shell
./scripts/run_tutorials.sh
```

## More examples

Additional V1 and V2 examples are available under [`examples`](examples). New applications should
start with the V2 examples in [`examples/v2`](examples/v2).

Compile all examples without running them:

```shell
cargo build --examples
```

Run an example by its Cargo target name:

```shell
cargo run --example v2_simple
```

Examples connect to Milvus and may create, modify, or delete resources. Review the selected example
and its connection settings before running it.

## Troubleshooting

- `connection refused`: start Milvus or set `MILVUS_URI` to a reachable endpoint.
- `unauthenticated` or `permission denied`: set `MILVUS_TOKEN` to a valid API key or
  `username:password` credential with permission for the requested operation.
- load or index timeout: verify that Milvus is healthy and has enough CPU and memory, then retry.
- Docker or port errors from repository scripts: ensure `docker info` succeeds and the required
  ports are available.

## Optional SDK diagnostics

Enable the SDK's `tracing` feature and add a subscriber to your application:

```toml
[dependencies]
milvus-sdk-rust = { version = "3", features = ["tracing"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Initialize the subscriber before creating `ClientV2`:

```rust
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
```

Call `init_tracing()` at the beginning of `main`, then select the events to display with
`RUST_LOG`:

```shell
RUST_LOG=milvus_sdk=debug cargo run
```

Use a narrower target when diagnosing one subsystem:

```shell
RUST_LOG=milvus_sdk::retry=debug cargo run
RUST_LOG=milvus_sdk::schema_cache=debug cargo run
RUST_LOG=milvus_sdk::polling=debug cargo run
```

The tracing feature is disabled by default. The SDK does not log credentials, request payloads,
filters, or vector data.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for development setup, builds, formatting, mock and
server-backed tests, Docker automation, coverage, and protobuf regeneration.

## License

[Apache License 2.0](LICENSE)
