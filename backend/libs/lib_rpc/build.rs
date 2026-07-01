//-- ./backend/libs/lib_rpc/build.rs

// Build script for BitNode-Console RPC library
// Compiles protobuf files using tonic_prost_build to generate Rust gRPC code
//
// Note: Comment `out_dir` and `.file_descriptor` if you want tonic_prost_build
// to build the code in the OUT_DIR (i.e. /target) instead of directly in src/rpc.
// This will also require adjusting the module paths in src/rpc/mod.rs accordingly.

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("protos");

    // Re-run the build script if any of the files below change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos/bitnode_console/");

    tonic_build::configure()
        .out_dir(manifest_dir.join("src/generated_protos"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_client(true)
        .build_server(true)
        .build_transport(true)
        .compile_well_known_types(false)
        .file_descriptor_set_path(out_dir.join("bitnode_console_v1_descriptor.bin"))
        .compile_protos(
            &[
                proto_dir.join("bitnode_console/authentication/authentication.proto"),
                proto_dir.join("bitnode_console/common/pagination.proto"),
                proto_dir.join("bitnode_console/journald/journald.proto"),
                proto_dir.join("bitnode_console/utilities/utilities.proto"),
            ],
            &[&proto_dir],
        )?;
    Ok(())
}
