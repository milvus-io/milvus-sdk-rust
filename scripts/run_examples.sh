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

cd "$ROOT_DIR"

examples_run=0

for example_path in examples/v2/*.rs; do
  example_name="$(basename "$example_path" .rs)"

  case "$example_name" in
    utils | cdc | optimize | external_table)
      continue
      ;;
  esac

  example_target="v2_${example_name}"
  printf '==> Running %s\n' "$example_target"
  cargo run --example "$example_target"
  examples_run=$((examples_run + 1))
done

printf '\nAll examples passed\n'
printf '%d examples ran\n' "$examples_run"
