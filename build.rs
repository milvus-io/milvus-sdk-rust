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
        .build_server(false)
        .compile_protos(&proto_entry_files, &[PROTO_DIR])?;
    Ok(())
}
