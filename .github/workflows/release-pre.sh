#!/bin/sh

# Pre-build hook for ./.github/workflows/release.yaml.
# Runs inside the rust-build.action Alpine Docker container before `cargo build`,
# not on the ubuntu-latest runner itself. Install any system dependencies that
# cargo build requires but that are not present in the container image.
# This file needs to be executable (chmod +x) so that it can be called by release.yaml.

echo "Installing build dependencies"

# protoc         — the protobuf compiler used by prost-build to generate
#                  Rust source from .proto files during cargo build.
# protobuf-dev   — C headers and libraries required to link against
#                  the protobuf runtime.
apk add --no-cache protoc protobuf-dev
