fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");

    tonic_build::configure()
        .build_server(false)
        .out_dir("src/proto")
        .compile_with_config(
            config,
            &[
                "milvus-proto/proto/common.proto",
                "milvus-proto/proto/milvus.proto",
                "milvus-proto/proto/schema.proto",
            ],
            &["milvus-proto/proto"],
        )?;

    Ok(())
}
