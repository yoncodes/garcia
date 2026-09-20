use std::path::Path;

fn main() {
    let proto_files = [
        "cg.proto",
        "err.proto",
        "pbcommon_main.proto",
        "pbcommon.proto",
        "proids.proto",
        "cs_main.proto",
        "cs.proto",
    ];

    for proto in &proto_files {
        println!("cargo::rerun-if-changed={proto}");
    }

    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc is unavailable");
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc);
    config.out_dir("include");
    config.include_file("_.rs");
    config
        .compile_protos(&proto_files, &[Path::new(".")])
        .expect("failed to compile protobuf schemas");
}
