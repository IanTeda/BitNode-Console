# Docker

## Overview

BitNode Console ships four container images, all published to
`ghcr.io/ianteda/…` from the release workflow. Each image targets a different
deployment shape — pick the one that matches how you want to run the console.

| Image | Dockerfile | What it contains |
|---|---|---|
| `bitnode-console` | `docker/Dockerfile.console` | `bin_console` — HTTP + gRPC server with the React frontend embedded in the binary |
| `bitnode-backend` | `docker/Dockerfile.backend` | `bin_backend` — gRPC-only server, no frontend, for headless/API-only deployments |
| `bitnode-frontend` | `docker/Dockerfile.frontend` | Standalone React SPA served by nginx, for splitting the frontend onto a separate host or CDN |
| `bitnode-node` | `docker/Dockerfile.node` | `bin_console` plus a Bitcoin Knots daemon (`bitcoind` + `bitcoin-cli`) supervised by `supervisord` — a single all-in-one node + web console image |

All Dockerfiles support both `linux/amd64` and `linux/arm64` and are built
with `docker buildx` in CI. `Dockerfile.node` selects the correct Bitcoin
Knots tarball via the `TARGETARCH` build argument that buildx injects.

## Dockerfiles

### `Dockerfile.console`

Three-stage build:

1. `node-builder` — installs frontend dependencies with pnpm and runs
   `pnpm build` to produce `frontend/dist`.
2. `rust-builder` — a `rust:1-bookworm` image with `protobuf-compiler` and
   Node 22 installed. The pre-built `frontend/dist` and `node_modules` are
   copied in from stage 1 before `cargo build --release -p bin_console`
   runs. Node has to be present at this stage because
   `backend/libs/lib_web/build.rs` shells out to `pnpm run build`
   during every Rust build.
3. Runtime — `debian:bookworm-slim` with just `ca-certificates` and
   `libssl3`. The `bin_console` binary is copied in and set as the entry
   point. Ports `3000` (HTTP) and `50051` (gRPC) are exposed.

### `Dockerfile.backend`

Identical layout to `Dockerfile.console` but builds the `bin_backend` crate in
stage 2 and only exposes the gRPC port (`50051`). The frontend build is
still run because `lib_web/build.rs` is invoked for the whole workspace;
the resulting assets are simply not shipped in the runtime image.

### `Dockerfile.frontend`

Two-stage build. Stage 1 installs frontend dependencies and runs
`pnpm build`. Stage 2 copies the resulting `dist/` into `nginx:alpine` and
drops in `docker/nginx.conf`, which serves the SPA with a `try_files`
fallback to `index.html` so client-side routing works. Only port `80` is
exposed.

### `Dockerfile.node`

An all-in-one image intended for single-host deployments where the console
and the Bitcoin daemon run together. Stages 1 and 2 mirror
`Dockerfile.console`. Stage 3 layers on top of `debian:bookworm-slim`:

- Downloads the Bitcoin Knots tarball for the target architecture using
  the `BITCOIN_KNOTS_VERSION` and `TARGETARCH` build arguments, then
  installs `bitcoind` and `bitcoin-cli` into `/usr/local/bin`.
- Copies in the `bin_console` binary from stage 2 and the
  `docker/supervisord.conf` configuration.
- Creates a `bitcoin` system user and the `/home/bitcoin/.bitcoin` and
  `/etc/bitnode` directories, which are declared as volumes so state
  survives container restarts.
- Exposes `3000` (HTTP), `50051` (gRPC), `8333` (Bitcoin P2P), and `8332`
  (Bitcoin RPC).
- Runs `supervisord` in the foreground, which starts and supervises both
  `bitcoind` (as the `bitcoin` user) and `bin_console`.

## Supporting files

- `docker/nginx.conf` — nginx server block for `Dockerfile.frontend`.
  Serves `index.html` for any unmatched path so the React router handles
  the URL.
- `docker/supervisord.conf` — the `supervisord` program list used by
  `Dockerfile.node`. Both programs log to stdout/stderr so
  `docker logs` shows their output.

## Building locally

All Dockerfiles expect the workspace root as the build context — they
reference paths like `frontend/`, `backend/`, and `docker/`. Run
`docker build` from the repository root, not from inside `docker/`.

### `bitnode-console`

```bash
docker build \
  -f docker/Dockerfile.console \
  -t bitnode-console:local \
  .
```

Run it, mapping the HTTP and gRPC ports:

```bash
docker run --rm -p 3000:3000 -p 50051:50051 bitnode-console:local
```

The console is then reachable at <http://localhost:3000>.

### `bitnode-backend`

```bash
docker build \
  -f docker/Dockerfile.backend \
  -t bitnode-backend:local \
  .

docker run --rm -p 50051:50051 bitnode-backend:local
```

Point a gRPC client (for example `grpcurl`) at `localhost:50051` to test.

### `bitnode-frontend`

```bash
docker build \
  -f docker/Dockerfile.frontend \
  -t bitnode-frontend:local \
  .

docker run --rm -p 8080:80 bitnode-frontend:local
```

The SPA is then reachable at <http://localhost:8080>. Note that the
frontend expects an API to talk to — run `bitnode-backend` or
`bitnode-console` alongside it and configure the frontend's API base URL
accordingly.

### `bitnode-node`

The all-in-one image needs volumes for the Bitcoin data directory and the
BitNode config directory, and a longer set of ports:

```bash
docker build \
  -f docker/Dockerfile.node \
  --build-arg BITCOIN_KNOTS_VERSION=27.1.knots20240801 \
  -t bitnode-node:local \
  .

docker run --rm \
  -p 3000:3000 -p 50051:50051 -p 8333:8333 \
  -v bitnode-data:/home/bitcoin/.bitcoin \
  -v bitnode-config:/etc/bitnode \
  bitnode-node:local
```

The container needs a `bitcoin.conf` in the mounted data volume before
`bitcoind` will start successfully — on the first run, `supervisord` will
keep restarting `bitcoind` until one is present. Port `8332` (Bitcoin RPC)
is intentionally not published to the host by default; only expose it if
you understand the security implications.

### Cross-architecture builds

To build for `linux/arm64` from an `amd64` host (for example, to test the
image locally before pushing), use `docker buildx` with QEMU emulation:

```bash
docker buildx build \
  --platform linux/arm64 \
  -f docker/Dockerfile.node \
  -t bitnode-node:local-arm64 \
  --load \
  .
```

Emulated builds are significantly slower than native ones; expect the Rust
compilation step in particular to take considerably longer under QEMU.
