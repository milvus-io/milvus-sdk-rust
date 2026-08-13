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
MILVUS_GRPC_PORT="${MILVUS_GRPC_PORT:-29530}"
MILVUS_HEALTH_PORT="${MILVUS_HEALTH_PORT:-29091}"
TUTORIAL_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target/tutorials}"
MILVUS_CONTAINER_ID=""
USE_LOCAL_SDK=false
export MILVUS_URI="${MILVUS_URI:-http://localhost:$MILVUS_GRPC_PORT}"

if [[ "${1:-}" == "--local-sdk" ]]; then
  USE_LOCAL_SDK=true
  shift
fi
if [[ "$#" -ne 0 ]]; then
  echo "Usage: $0 [--local-sdk]" >&2
  exit 1
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

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "Tutorial server automation is supported on Linux only." >&2
  exit 1
fi

for tool in cargo docker python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool is required to run the tutorials." >&2
    exit 1
  fi
done

trap handle_exit EXIT
trap 'handle_signal 130' INT
trap 'handle_signal 143' TERM

echo "==> Starting Milvus for tutorials"
MILVUS_CONTAINER_ID="$(
  python3 "$MILVUS_CONTAINER_SCRIPT" start \
    --grpc-port "$MILVUS_GRPC_PORT" \
    --health-port "$MILVUS_HEALTH_PORT"
)"

cd "$ROOT_DIR"
echo "==> Running beginner tutorials (1_quickstart through 6_dql)"
for manifest in tutorial/*/Cargo.toml; do
  tutorial_name="${manifest#tutorial/}"
  tutorial_name="${tutorial_name%/Cargo.toml}"
  if [[ "$tutorial_name" == 7_database ]]; then
    echo "==> Running advanced tutorials (database administration and RBAC)"
  fi
  if [[ "$tutorial_name" == 8_rbac ]]; then
    echo "==> RBAC note: the default tutorial server does not enable authorization; RBAC calls may run, but permission-denial behavior is not validated."
  fi
  echo "==> Running ${manifest%/Cargo.toml}"
  if [[ "$USE_LOCAL_SDK" == "true" ]]; then
    CARGO_TARGET_DIR="$TUTORIAL_TARGET_DIR" \
      cargo run --manifest-path "$manifest" \
        --config "patch.crates-io.milvus-sdk-rust.path='$ROOT_DIR'"
  else
    CARGO_TARGET_DIR="$TUTORIAL_TARGET_DIR" cargo run --manifest-path "$manifest"
  fi
done
