#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/docker-compose.yml"
HEALTH_URL="http://localhost:9091/healthz"
TIMEOUT_SECONDS=300
POLL_INTERVAL=1
CODE_COV="${CODE_COV:-false}"

cleanup() {
  docker compose -f "$COMPOSE_FILE" down || true
}

wait_for_milvus() {
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while (( SECONDS < deadline )); do
    if curl -fsS --max-time 2 "$HEALTH_URL" >/dev/null; then
      return 0
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
  RUST_BACKTRACE="${RUST_BACKTRACE:-1}" cargo llvm-cov --workspace --lcov --output-path lcov.info --ignore-filename-regex 'src/proto/.*' "$@"
elif [[ "$#" -eq 0 ]]; then
  cargo test
else
  "$@"
fi
