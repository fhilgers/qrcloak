fn main() -> miette::Result<()> {
    let file_descriptors =
        protox::compile(["./protobuf/wire_format/v1/payload.proto"], ["protobuf"])?;
    prost_build::compile_fds(file_descriptors).unwrap();
    Ok(())
}
