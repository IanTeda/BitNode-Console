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

# Define the interactive development shell environment.
pkgs.mkShell {
  # Tooling available in the shell PATH.
  packages = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    taplo

    # Docs and task runner
    mdbook
    cargo-make

    # Frontend tooling
    nodejs_22
    nodePackages.pnpm

    # Build dependencies
    pkg-config
    openssl

    # direnv Nix shell caching (needed for VS Code extension host)
    nix-direnv
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
  '';
}