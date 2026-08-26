#!/usr/bin/env bash
# Licensed to the LF AI & Data foundation under one
# or more contributor license agreements. See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership. The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MILVUS_CONTAINER_SCRIPT="$ROOT_DIR/tests/v2/st/milvus_container.py"
COVERAGE_DIR="$ROOT_DIR/code_coverage"
LCOV_FILE="$COVERAGE_DIR/lcov.info"
CODE_COV="${CODE_COV:-false}"
MILVUS_CONTAINER_ID=""
RUN_SERVER_TESTS=true
SYSTEM_TEST_THREADS="${SYSTEM_TEST_THREADS:-2}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

if [[ "${1:-}" == "--no-server" ]]; then
  RUN_SERVER_TESTS=false
  shift
fi

if [[ "$RUN_SERVER_TESTS" == "false" && "$CODE_COV" == "true" ]]; then
  echo "CODE_COV=true is not supported with --no-server" >&2
  exit 1
fi

if [[ "$CODE_COV" == "true" ]]; then
  if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "cargo-llvm-cov is required when CODE_COV=true; install it with 'cargo install cargo-llvm-cov --locked'" >&2
    exit 1
  fi
  if ! command -v genhtml >/dev/null 2>&1; then
    echo "genhtml is required to generate the HTML coverage report; install the lcov package" >&2
    exit 1
  fi
fi

cleanup_resources() {
  local status="$1"
  if [[ -n "$MILVUS_CONTAINER_ID" ]]; then
    if [[ "$status" -ne 0 ]]; then
      python3 "$MILVUS_CONTAINER_SCRIPT" logs "$MILVUS_CONTAINER_ID" || true
    fi
    python3 "$MILVUS_CONTAINER_SCRIPT" stop "$MILVUS_CONTAINER_ID" || true
    MILVUS_CONTAINER_ID=""
  fi
}

handle_exit() {
  local status=$?
  trap - EXIT INT TERM
  cleanup_resources "$status"
  exit "$status"
}

handle_signal() {
  local status="$1"
  trap - EXIT INT TERM
  cleanup_resources "$status"
  exit "$status"
}

check_tutorials() {
  local manifest
  echo "==> Checking tutorial crates"
  for manifest in "$ROOT_DIR"/tutorial/*/Cargo.toml; do
    CARGO_TARGET_DIR="$ROOT_DIR/target/tutorials" \
      cargo check --manifest-path "$manifest" --all-targets \
        --config "patch.crates-io.milvus-sdk-rust.path='$ROOT_DIR'"
  done
}

run_non_server_tests() {
  echo "==> Running compile checks and non-server tests"
  cd "$ROOT_DIR"
  cargo check --all-targets
  check_tutorials
  cargo test --lib -- --test-threads=4
  cargo test --test v2_ut -- --test-threads=4
  cargo test --doc
}

run_non_server_coverage_tests() {
  echo "==> Running compile checks and non-server coverage tests"
  cd "$ROOT_DIR"
  cargo check --all-targets
  check_tutorials
  cargo llvm-cov clean --workspace
  cargo llvm-cov --workspace --no-report --lib -- --test-threads=4
  cargo llvm-cov --workspace --no-report --test v2_ut -- --test-threads=4
  cargo test --doc
}

require_linux_server_support() {
  if [[ "$(uname -s)" != "Linux" ]]; then
    echo "Server-backed tests are supported by this script on Linux only; use --no-server on this platform" >&2
    exit 1
  fi

  command -v python3 >/dev/null 2>&1 || {
    echo "python3 is required to launch the Milvus test container" >&2
    exit 1
  }
}

start_milvus() {
  echo "==> Starting Milvus for server-backed tests"
  trap handle_exit EXIT
  trap 'handle_signal 130' INT
  trap 'handle_signal 143' TERM
  MILVUS_CONTAINER_ID="$(
    python3 "$MILVUS_CONTAINER_SCRIPT" start
  )"
}

run_server_tests() {
  echo "==> Running V1 and V2 server-backed tests"
  cd "$ROOT_DIR"
  cargo test --test v1_st -- --test-threads="$SYSTEM_TEST_THREADS"
  cargo test --test v2_st -- --test-threads="$SYSTEM_TEST_THREADS"
}

run_server_coverage_tests() {
  echo "==> Running V1 and V2 server-backed coverage tests"
  cd "$ROOT_DIR"
  cargo llvm-cov --workspace --no-report --test v1_st -- --test-threads="$SYSTEM_TEST_THREADS"
  cargo llvm-cov --workspace --no-report --test v2_st -- --test-threads="$SYSTEM_TEST_THREADS"
  cargo llvm-cov report --lcov --output-path "$LCOV_FILE" --ignore-filename-regex 'src/proto/.*'
  genhtml "$LCOV_FILE" --output-directory "$COVERAGE_DIR"
}

if [[ "$RUN_SERVER_TESTS" == "false" ]]; then
  if [[ "$#" -ne 0 ]]; then
    echo "--no-server does not accept an additional command" >&2
    exit 1
  fi
  run_non_server_tests
  exit 0
fi

if [[ "$#" -eq 0 && "$CODE_COV" == "true" ]]; then
  mkdir -p "$COVERAGE_DIR"
  run_non_server_coverage_tests
  require_linux_server_support
  start_milvus
  run_server_coverage_tests
elif [[ "$#" -eq 0 ]]; then
  run_non_server_tests
  require_linux_server_support
  start_milvus
  run_server_tests
else
  require_linux_server_support
  start_milvus
  cd "$ROOT_DIR"
  if [[ "$CODE_COV" == "true" ]]; then
    mkdir -p "$COVERAGE_DIR"
    cargo llvm-cov --workspace --lcov --output-path "$LCOV_FILE" --ignore-filename-regex 'src/proto/.*' "$@" -- --test-threads="$SYSTEM_TEST_THREADS"
    genhtml "$LCOV_FILE" --output-directory "$COVERAGE_DIR"
  else
    "$@"
  fi
fi
