# BitNode Console development shell
#
# This Nix shell defines a reproducible development environment for the
# repository. It provides the required toolchains (Rust, docs, frontend),
# build dependencies, and default environment variables used during local
# development and by VS Code extensions.
#
# Learn more about Nix development shells:
# - https://nix.dev/tutorials/first-steps/declarative-and-reproducible-developer-environments
# - https://nixos.org/manual/nixpkgs/stable/#sec-pkgs-mkShell

{ pkgs ? import <nixpkgs> {} }:

let
  # Unified Python environment for MkDocs and all required plugins/extensions.
  # Bundling them with withPackages ensures they share one interpreter and
  # avoids PYTHONPATH conflicts between individually-installed packages.
  mkdocsPython = pkgs.python3.withPackages (ps: with ps; [
    mkdocs
    mkdocs-material
    pymdown-extensions
  ]);

in

# Define the interactive development shell environment.
pkgs.mkShell {
  # Tooling available in the shell PATH.
  packages = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rust-analyzer
    taplo

    # Docs and task runner
    mdbook
    cargo-make
    mkdocsPython  # mkdocs + material + pymdown-extensions

    # Code Linting and Coverage
    cargo-tarpaulin
    clippy
    rustfmt
    cargo-audit

    # Frontend tooling
    nodejs_22
    pnpm

    # Build dependencies
    pkg-config
    openssl
    protobuf  # protoc compiler required by tonic-build/prost-build
    buf       # protobuf linter and breaking-change detector

    # direnv Nix shell caching (needed for VS Code extension host)
    nix-direnv

    # Development tools
    grpcurl

    # Password hashing
    libargon2
  ];

  # Environment defaults applied whenever entering the shell.
  # These variables improve diagnostics/logging and let rust-analyzer resolve
  # the standard library source via RUST_SRC_PATH.
  shellHook = ''
    export BITNODE_ENV="development"
    export CARGO_TERM_COLOR="always"
    export RUST_BACKTRACE="1"
    export RUST_LOG="info"
    export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
    export PROTOC="${pkgs.protobuf}/bin/protoc"
  '';
}
