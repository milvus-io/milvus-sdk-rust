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

# Verify that Zilliz Cloud only APIs carry the "Zilliz Cloud only." rustdoc
# marker, so generated docs distinguish the cloud-only surface from the
# general-purpose API (mirrors the Java SDK's @zillizCloudOnly javadoc tag).

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MARKER='Zilliz Cloud only.'
# Ring size for the line history used to scan each item's doc block.
RING=1000

# Entire modules that are Zilliz Cloud only: their module-level doc must carry
# the marker so generated docs identify the whole module at a glance.
CLOUD_ONLY_MODULES=(
  src/v2/client/global_cluster.rs
  src/v2/types/global_cluster.rs
  src/v2/client/session.rs
)

# Cloud-only public API living in otherwise general-purpose files. Each entry is
# "path:fn_name"; every `pub fn <fn_name>` in the file must be documented with
# the marker. Substring matching means one entry covers related names, e.g.
# "object_url" also covers object_urls and object_url_group.
CLOUD_ONLY_ITEMS=(
  src/v2/client.rs:session
  src/v2/bulk_import.rs:object_url
  src/v2/bulk_import.rs:cluster_id
  src/v2/bulk_import.rs:project_id
  src/v2/bulk_import.rs:region_id
  src/v2/bulk_import.rs:access_key
  src/v2/bulk_import.rs:secret_key
  src/v2/bulk_import.rs:token
  src/v2/bulk_import.rs:volume_name
  src/v2/bulk_import.rs:data_path
)

fail() {
  echo "check-zilliz-cloud-tags: $1" >&2
  exit 1
}

for module in "${CLOUD_ONLY_MODULES[@]}"; do
  grep -q -- "$MARKER" "$module" \
    || fail "$module must carry the '$MARKER' marker in its module doc"
done

for entry in "${CLOUD_ONLY_ITEMS[@]}"; do
  file="${entry%%:*}"
  symbol="${entry#*:}"
  awk -v sym="$symbol" -v marker="$MARKER" -v ring="$RING" '
    index($0, "pub fn " sym) {
      marked = 0
      for (i = 1; i <= ring; i++) {
        prev = lines[(NR - i) % ring]
        if (prev !~ /^[ \t]*\/\/\//) break
        if (index(prev, marker)) { marked = 1; break }
      }
      if (!marked) {
        print FILENAME ":" FNR ": pub fn " sym " is Zilliz Cloud only but lacks the " marker " marker"
        bad = 1
      }
    }
    { lines[NR % ring] = $0 }
    END { exit bad }
  ' "$file" || fail "see above"
done

echo "check-zilliz-cloud-tags: OK"
