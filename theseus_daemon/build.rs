fn main() {
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile(&["src/protos/theseus.proto"], &["src/protos"])
        .unwrap();
}
