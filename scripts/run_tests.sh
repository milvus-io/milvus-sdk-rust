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
COMPOSE_FILE="$ROOT_DIR/docker-compose.yml"
COVERAGE_DIR="$ROOT_DIR/code_coverage"
LCOV_FILE="$COVERAGE_DIR/lcov.info"
HEALTH_URL="http://localhost:9091/healthz"
TIMEOUT_SECONDS=300
POLL_INTERVAL=1
CODE_COV="${CODE_COV:-false}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

if [[ "$CODE_COV" == "true" ]] && ! command -v genhtml >/dev/null 2>&1; then
  echo "genhtml is required to generate the HTML coverage report; install the lcov package" >&2
  exit 1
fi

cleanup() {
  docker compose -f "$COMPOSE_FILE" down || true
}

wait_for_milvus() {
  local started=$SECONDS
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  echo "Waiting for Milvus to become healthy at $HEALTH_URL..."
  while (( SECONDS < deadline )); do
    if curl -fsS --max-time 2 "$HEALTH_URL" >/dev/null 2>&1; then
      echo "Milvus is healthy."
      return 0
    fi
    local elapsed=$((SECONDS - started))
    if (( elapsed > 0 && elapsed % 10 == 0 )); then
      echo "Milvus is still starting (${elapsed}s elapsed)..."
    fi
    sleep "$POLL_INTERVAL"
  done

  echo "Timed out waiting for Milvus health endpoint: $HEALTH_URL" >&2
  return 1
}

trap cleanup EXIT INT TERM

docker compose -f "$COMPOSE_FILE" up -d
wait_for_milvus

cd "$ROOT_DIR"

if [[ "$CODE_COV" == "true" ]]; then
  mkdir -p "$COVERAGE_DIR"
  cargo llvm-cov --workspace --lcov --output-path "$LCOV_FILE" --ignore-filename-regex 'src/proto/.*' "$@" -- --test-threads=4
  genhtml "$LCOV_FILE" --output-directory "$COVERAGE_DIR"
elif [[ "$#" -eq 0 ]]; then
  cargo test -- --test-threads=4
else
  "$@"
fi
