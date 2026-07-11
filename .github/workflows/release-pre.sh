#!/bin/sh

# Pre-build hook for ./.github/workflows/release.yml.
# Runs inside the ubuntu-latest build container before `cargo build`.
# Install any system dependencies that cargo build requires but that are not
# present in the default runner image.

echo "Installing build dependencies"

# protobuf-compiler  — the `protoc` binary used by prost-build to compile
#                      .proto files into Rust source during cargo build.
# libprotobuf-dev    — C headers and libraries required to link against
#                      the protobuf runtime.
apt-get update && apt-get install -y protobuf-compiler libprotobuf-dev
