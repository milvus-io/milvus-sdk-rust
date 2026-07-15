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
OUTPUT_DIR="$ROOT_DIR/doc"
TEMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TEMP_DIR"
}

trap cleanup EXIT INT TERM

cd "$ROOT_DIR"

RUSTDOCFLAGS="${RUSTDOCFLAGS:+$RUSTDOCFLAGS }-A rustdoc::bare-urls" \
  cargo doc --no-deps --target-dir "$TEMP_DIR"

mkdir -p "$OUTPUT_DIR"
cp -a "$TEMP_DIR/doc/." "$OUTPUT_DIR/"
rm -rf "$OUTPUT_DIR/milvus/proto"

printf 'Documentation generated at %s\n' "$OUTPUT_DIR/milvus/index.html"
