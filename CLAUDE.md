# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BitNode Console is a web-based interface for managing a Bitcoin Knots/Core daemon
node — configuration (`bitcoin.conf`), status monitoring, and log viewing without
requiring CLI/SSH access. The project is currently in its **feasibility stage**
(see `docs/product-requirements.md` and `README.md` roadmap): demonstrating
auth via RPC credentials, daemon status, journalctl log output, and parsing
`bitcoin.conf` settings. Most functionality does not exist yet — `backend/server` is
currently a minimal axum "Hello, World!" server.

## Repository Structure

- Cargo workspace (`Cargo.toml`) with members under `backend/*`.
- `backend/server` — axum-based web server (binary crate, `src/main.rs`).
- `docs/` — mdBook documentation source (`book.toml`, built to `docs/.book/`).
- `shell.nix` / `.envrc` — Nix + direnv development environment.

## Development Environment

This repo uses `direnv` + `nix-shell` (`shell.nix`) to provide the toolchain:
`cargo`, `rustc`, `rustfmt`, `clippy`, `rust-analyzer`, `mdbook`, `cargo-make`,
`nodejs_22`, `pnpm`. Run `direnv allow` once after cloning; entering the
directory then loads the environment automatically. If direnv isn't set up,
run `nix-shell` manually from the repo root.

## Common Commands

- Build: `cargo build`
- Run the web server: `cargo run -p server` (serves on `0.0.0.0:3000`)
- Test (all): `cargo test`
- Test (single, by name): `cargo test <test_name>`
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Build docs (mdBook): `cargo make docs-build` or `mdbook build`
- Serve docs locally on port 8001: `cargo make docs-serve`

## Workspace Lint/Style Configuration

Lint rules are centralised in the root `Cargo.toml` under `[workspace.lints]`
and apply to all crates unless overridden locally:

- `unsafe_code = "forbid"` — no unsafe code anywhere in the workspace.
- `warnings = "deny"` — all compiler warnings are treated as errors; code must
  compile cleanly.
- `unused = "allow"` (low priority) — unused code/variables are permitted
  during exploratory development, but should be cleaned up before committing
  where practical.
- `clippy::pedantic` and `clippy::nursery` are enabled at `warn` level.

Shared dependency versions are intended to live in `[workspace.dependencies]`
in the root `Cargo.toml` (currently mostly commented out as placeholders) —
add dependencies there and reference with `{ workspace = true }` from member
crates when a dependency is shared across multiple crates.

## Rust Coding Conventions

These conventions (from `.ai/instructions/rust.instructions.md`) apply to
all `.rs` files:

- Prefer `Result<T, E>` and the `?` operator over `unwrap()`/`expect()`;
  avoid panics outside of truly unrecoverable situations.
- Use `thiserror` (or `anyhow`) for custom error types.
- Prefer borrowing (`&T`, `&str`) over cloning/owned types unless ownership
  transfer is required.
- Prefer iterators over index-based loops; avoid premature `collect()`.
- Split logic into modules; keep `main.rs`/`lib.rs` minimal.
- Document all public items with `///` rustdoc comments, written in
  Australian English.
- Use the `fake` crate to generate test data; write unit tests in
  `#[cfg(test)] mod tests` alongside the code they test, and integration
  tests under `tests/`.
- Code must pass `cargo fmt`, `cargo clippy`, and `cargo test` before
  committing.

## Markdown Conventions

For `.md` files (from `.ai/instructions/markdown.instructions.md`):

- Do not use an H1 (`#`) heading — titles are generated automatically; start
  at H2 (`##`).
- Include YAML front matter with required metadata where applicable.
- Use fenced code blocks with a language identifier.
- Wrap prose at roughly 80 characters.
