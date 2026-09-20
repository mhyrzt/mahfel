fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let mut prost = tonic_prost_build::Config::new();
    prost.protoc_executable(protoc);
    tonic_prost_build::configure().compile_with_config(
        prost,
        &[
            "../../../proto/mahfel/media/v1/node.proto",
            "../../../proto/mahfel/media/v1/room.proto",
            "../../../proto/mahfel/media/v1/transport.proto",
            "../../../proto/mahfel/media/v1/track.proto",
            "../../../proto/mahfel/media/v1/media_control.proto",
        ],
        &["../../../proto"],
    )?;
    println!("cargo:rerun-if-changed=../../../proto");
    Ok(())
}
