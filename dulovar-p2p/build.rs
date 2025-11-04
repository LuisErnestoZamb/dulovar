fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .out_dir("src/grpc_daemon")
        .compile_protos(&["src/proto/alert.proto"], &["proto"])
        .unwrap();
    Ok(())
}
