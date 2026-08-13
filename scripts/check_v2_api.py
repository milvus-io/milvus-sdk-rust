#!/usr/bin/env python3
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

"""Check mechanical conventions for public ClientV2 request/response DTOs."""

from __future__ import print_function

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REQUEST_DIR = ROOT / "src" / "v2" / "request"
RESPONSE_DIR = ROOT / "src" / "v2" / "response"

TYPE_PATTERN = re.compile(r"(?m)^pub struct (\w+(?:Request|Response))\b")
ACCESSOR_ALIASES = {
    "async_mode": ("is_async",),
    "detail": ("is_detail_enabled",),
    "enable_dynamic_field": ("is_dynamic_field_enabled",),
    "with_detail": ("should_include_detail",),
    "with_hash": ("should_include_hash",),
    "with_shard_nodes": ("should_include_shard_nodes",),
}
SETTER_ALIASES = {
    "rows": ("row",),
}


def matching_brace(source, opening):
    depth = 0
    for index in range(opening, len(source)):
        char = source[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    raise ValueError("unmatched brace at byte {}".format(opening))


def block_after(source, pattern, start=0):
    match = re.search(pattern, source[start:], re.MULTILINE)
    if match is None:
        return None
    absolute_start = start + match.start()
    opening = source.find("{", absolute_start)
    if opening < 0:
        return None
    closing = matching_brace(source, opening)
    return source[opening + 1 : closing]


def impl_blocks(source, name, start=0):
    blocks = []
    for match in re.finditer(r"(?m)^impl {}\s*".format(re.escape(name)), source[start:]):
        absolute = start + match.start()
        opening = source.find("{", absolute)
        if opening >= 0:
            blocks.append(source[opening + 1 : matching_brace(source, opening)])
    return "\n".join(blocks)


def preceding_attributes(source, offset):
    prefix = source[:offset]
    separator = max(prefix.rfind("\n\n"), prefix.rfind("///////////////////////////////////////////////////////////////////////////////"))
    return prefix[separator:]


def has_non_exhaustive_attribute(source, offset):
    attributes = preceding_attributes(source, offset)
    return re.search(r"(?m)^\s*#\[non_exhaustive\]\s*$", attributes) is not None


def check_request(path, source, name, offset, failures):
    location = "{}:{}".format(path.relative_to(ROOT), source.count("\n", 0, offset) + 1)
    if not has_non_exhaustive_attribute(source, offset):
        failures.append("{}: {} must be #[non_exhaustive]".format(location, name))

    builder = name + "Builder"
    builder_impl = None
    if re.search(r"(?m)^pub struct {}\b".format(re.escape(builder)), source) is not None:
        builder_impl = impl_blocks(source, builder)
        if builder_impl is None or re.search(
            r"pub fn build\s*\(\s*self\s*\)\s*->\s*Result\s*<\s*{}\s*>".format(name),
            builder_impl,
        ) is None:
            failures.append("{}: {} is missing build() -> Result<{}>".format(location, builder, name))

    request_block = block_after(source, r"^pub struct {}\s*".format(re.escape(name)), offset)
    if request_block is None:
        return
    fields = re.findall(r"(?m)^\s*pub\(crate\)\s+(\w+)\s*:", request_block)
    request_impl = impl_blocks(source, name, offset)
    if request_impl is None:
        failures.append("{}: {} is missing an implementation block".format(location, name))
        return
    if re.search(r"pub fn into_builder\s*\(\s*(?:mut\s+)?self\s*\)\s*->", request_impl) is None:
        failures.append("{}: {} is missing into_builder()".format(location, name))
    for field in fields:
        accessor_names = [field]
        if field.startswith(("is_", "has_", "should_")):
            accessor_names.append(field)
        else:
            accessor_names.extend(["is_" + field, "has_" + field, "should_" + field])
        accessor_names.extend(ACCESSOR_ALIASES.get(field, ()))
        if not any(re.search(r"pub fn {}\s*\(\s*&self".format(re.escape(candidate)), request_impl) for candidate in accessor_names):
            failures.append("{}: {} is missing read-only accessor for field `{}`".format(location, name, field))
        setter_names = (field,) + SETTER_ALIASES.get(field, ())
        if builder_impl is None or not any(re.search(r"pub fn {}\s*\(".format(re.escape(candidate)), builder_impl) for candidate in setter_names):
            failures.append("{}: {}Builder is missing setter for field `{}`".format(location, name, field))


def check_response(path, source, name, offset, failures):
    location = "{}:{}".format(path.relative_to(ROOT), source.count("\n", 0, offset) + 1)
    if not has_non_exhaustive_attribute(source, offset):
        failures.append("{}: {} must be #[non_exhaustive]".format(location, name))
    response_block = block_after(source, r"^pub struct {}\s*".format(re.escape(name)), offset)
    if response_block is None:
        return
    fields = re.findall(r"(?m)^\s*pub\(crate\)\s+(\w+)\s*:", response_block)
    response_impl = impl_blocks(source, name, offset)
    if response_impl is None:
        failures.append("{}: {} is missing an implementation block".format(location, name))
        return
    for field in fields:
        accessor_names = [field, "is_" + field, "has_" + field, "should_" + field]
        accessor_names.extend(ACCESSOR_ALIASES.get(field, ()))
        if not any(re.search(r"pub fn {}\s*\(\s*&self".format(re.escape(candidate)), response_impl) for candidate in accessor_names):
            failures.append("{}: {} is missing read-only accessor for field `{}`".format(location, name, field))

def check_public_proto_references(path, source, failures):
    for number, line in enumerate(source.splitlines(), 1):
        stripped = line.strip()
        if not stripped.startswith("pub "):
            continue
        if "crate::proto" in stripped or re.search(r"\bproto::", stripped):
            failures.append(
                "{}:{}: public API references a generated protobuf type".format(
                    path.relative_to(ROOT), number
                )
            )


def main():
    failures = []
    counts = {"request": 0, "response": 0}
    for kind, directory in (("request", REQUEST_DIR), ("response", RESPONSE_DIR)):
        for path in sorted(directory.glob("*.rs")):
            source = path.read_text(encoding="utf-8")
            check_public_proto_references(path, source, failures)
            for match in TYPE_PATTERN.finditer(source):
                name = match.group(1)
                if name.endswith("Request") and kind == "request":
                    counts["request"] += 1
                    check_request(path, source, name, match.start(), failures)
                elif name.endswith("Response") and kind == "response":
                    counts["response"] += 1
                    check_response(path, source, name, match.start(), failures)

    if failures:
        print("V2 API convention checks failed:", file=sys.stderr)
        for failure in failures:
            print("- " + failure, file=sys.stderr)
        return 1

    print(
        "V2 API convention checks passed for {} requests and {} responses.".format(
            counts["request"], counts["response"]
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
