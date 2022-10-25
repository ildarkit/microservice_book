use protoc_rust_grpc::Codegen;

fn main() {
    Codegen::new()
        .includes(&["src/protos"])
        .input("src/protos/ring.proto")
        .rust_protobuf(true)
        .out_dir("src")
        .run()
        .expect("protoc-rust-grpc");
}
