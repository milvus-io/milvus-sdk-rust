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

# Bump the crate version in Cargo.toml and sync the pinned milvus-sdk-rust
# version across every standalone tutorial manifest and tutorial README.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT_DIR/Cargo.toml"

if [[ "$#" -ne 1 ]]; then
  echo "Usage: $0 X.Y.Z" >&2
  exit 1
fi

version="$1"
if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid version: ${version}. Expected X.Y.Z." >&2
  exit 1
fi

grep -q '^name = "milvus-sdk-rust"' "$MANIFEST" || {
  echo "No milvus-sdk-rust manifest found at ${MANIFEST}." >&2
  exit 1
}

# Portable in-place editing for both GNU and BSD sed: write to a temporary
# file and move it over the original so `-i` flag differences never matter.
# Exits 0 when the file actually changed, 1 when it was already up to date.
sed_in_place() {
  local file="$1"
  shift
  if sed "$@" "$file" > "${file}.tmp"; then
    if ! cmp -s "$file" "${file}.tmp"; then
      mv "${file}.tmp" "$file"
      return 0
    fi
    rm -f "${file}.tmp"
  fi
  return 1
}

# README references look like `milvus-sdk-rust` version `X.Y.Z`.
version_ref_pattern='`milvus-sdk-rust` version `[0-9]+\.[0-9]+\.[0-9]+`'
target_ref="\`milvus-sdk-rust\` version \`${version}\`"

updated=0

if ! grep -Fxq "version = \"${version}\"" "$MANIFEST"; then
  sed_in_place "$MANIFEST" "s/^version = \".*\"$/version = \"${version}\"/"
  echo "Updated ${MANIFEST#${ROOT_DIR}/} to version ${version}"
  updated=$((updated + 1))
fi

for manifest in "$ROOT_DIR"/tutorial/*/Cargo.toml; do
  if ! grep -Fxq "milvus-sdk-rust = \"=${version}\"" "$manifest"; then
    sed_in_place "$manifest" "s/^milvus-sdk-rust = \"=.*\"$/milvus-sdk-rust = \"=${version}\"/"
    echo "Updated ${manifest#${ROOT_DIR}/} to milvus-sdk-rust ${version}"
    updated=$((updated + 1))
  fi
done

for readme in "$ROOT_DIR"/tutorial/*/README.md; do
  if grep -Eq "$version_ref_pattern" "$readme"; then
    if sed_in_place "$readme" "s/\`milvus-sdk-rust\` version \`[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\`/\`milvus-sdk-rust\` version \`${version}\`/g"; then
      echo "Updated ${readme#${ROOT_DIR}/} to milvus-sdk-rust ${version}"
      updated=$((updated + 1))
    fi
  fi
done

if ! grep -Fxq "version = \"${version}\"" "$MANIFEST"; then
  echo "Mismatch: root Cargo.toml is not set to version ${version}." >&2
  exit 1
fi
for manifest in "$ROOT_DIR"/tutorial/*/Cargo.toml; do
  if ! grep -Fxq "milvus-sdk-rust = \"=${version}\"" "$manifest"; then
    echo "Mismatch: ${manifest#${ROOT_DIR}/} is not pinned to ${version}." >&2
    exit 1
  fi
done
for readme in "$ROOT_DIR"/tutorial/*/README.md; do
  if grep -Eq "$version_ref_pattern" "$readme" && ! grep -Fq "$target_ref" "$readme"; then
    echo "Mismatch: ${readme#${ROOT_DIR}/} still has a non-target milvus-sdk-rust version reference." >&2
    exit 1
  fi
done

if [[ "$updated" -eq 0 ]]; then
  echo "Version ${version} is already set in all versioned files."
else
  echo "Version bumped to ${version} across ${updated} file(s)."
fi
