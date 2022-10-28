fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "src/protos/ring.proto";
    tonic_build::configure()
        .out_dir("src")
        .compile(&[proto_file], &["."])?;
    Ok(())
}
