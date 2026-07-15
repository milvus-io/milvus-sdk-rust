// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::PathBuf;

const PROTO_DIR: &str = "milvus-proto/proto";
const PROTO_FILES: &[&str] = &[
    "common.proto",
    "feder.proto",
    "milvus.proto",
    "msg.proto",
    "rg.proto",
    "schema.proto",
];
const PROTO_ENTRY_FILES: &[&str] = &["common.proto", "milvus.proto", "schema.proto"];

fn proto_path(file: &str) -> PathBuf {
    PathBuf::from(PROTO_DIR).join(file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for proto_file in PROTO_FILES {
        println!(
            "cargo:rerun-if-changed={}",
            proto_path(proto_file).display()
        );
    }

    if std::env::var_os("PROTOC").is_none() {
        let protoc = protoc_bin_vendored::protoc_bin_path()?;
        std::env::set_var("PROTOC", protoc);
    }

    let proto_entry_files: Vec<_> = PROTO_ENTRY_FILES
        .iter()
        .map(|proto_file| proto_path(proto_file))
        .collect();

    tonic_build::configure()
        .build_server(true)
        .generate_default_stubs(true)
        .compile_protos(&proto_entry_files, &[PROTO_DIR])?;
    Ok(())
}
