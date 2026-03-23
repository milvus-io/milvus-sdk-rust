fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto files are pre-generated in src/proto/
    // To regenerate, install protoc and uncomment below:
    // tonic_build::configure()
    //     .build_server(false)
    //     .out_dir("src/proto")
    //     .compile_protos(
    //         &[
    //             "milvus-proto/proto/common.proto",
    //             "milvus-proto/proto/milvus.proto",
    //             "milvus-proto/proto/schema.proto",
    //         ],
    //         &["milvus-proto/proto"],
    //     )?;
    Ok(())
}
