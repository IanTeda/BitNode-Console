---
title: CI/CD Implementation Plan
description: GitHub Actions workflows and Docker images for BitNode Console
status: draft
---

## Overview

Add a full CI/CD pipeline using GitHub Actions. Three workflows cover continuous
integration, binary releases, and Docker image publishing. All images push to
`ghcr.io` using `GITHUB_TOKEN` — no extra secrets required.

**Key constraint:** `backend/libs/lib_web/build.rs` calls `pnpm run build` unconditionally
during every `cargo build`. Every Rust build step (CI, release, Docker) must have
Node 22 + pnpm + frontend `node_modules` available.

---

## Stage 1 — CI Workflow

**File:** `.github/workflows/ci.yml`  
**Trigger:** push to `main` or `develop`; pull requests targeting `main`

### Job: `frontend`

```yaml
runs-on: ubuntu-latest
steps:
  - uses: actions/checkout@v4
  - uses: actions/setup-node@v4
    with: { node-version: "22" }
  - uses: pnpm/action-setup@v4
    with: { version: "10" }
  - run: pnpm install --frozen-lockfile
    working-directory: frontend
  - run: pnpm run lint
    working-directory: frontend
  - run: pnpm exec tsc --noEmit
    working-directory: frontend
```

### Job: `backend`

```yaml
runs-on: ubuntu-latest
steps:
  - uses: actions/checkout@v4
  - run: sudo apt-get update && sudo apt-get install -y protobuf-compiler
  - uses: actions/setup-node@v4
    with: { node-version: "22" }
  - uses: pnpm/action-setup@v4
    with: { version: "10" }
  - run: pnpm install --frozen-lockfile
    working-directory: frontend          # required by lib_web/build.rs
  - uses: dtolnay/rust-toolchain@stable
    with: { components: "rustfmt,clippy" }
  - uses: Swatinem/rust-cache@v2
  - run: cargo fmt --all -- --check
  - run: cargo clippy --all-targets --all-features -- -D warnings
  - run: cargo test --all
```

---

## Stage 2 — Release Workflow

**File:** `.github/workflows/release.yml`  
**Trigger:** push of tag matching `v*`

Builds `bin_console` and `bin_rpc` for both targets using `cross` (which handles
the aarch64 linker/sysroot via Docker internally).

### Job: `build` (matrix)

```yaml
strategy:
  matrix:
    target:
      - x86_64-unknown-linux-gnu
      - aarch64-unknown-linux-gnu

runs-on: ubuntu-latest
permissions:
  contents: write

steps:
  - uses: actions/checkout@v4
  - run: sudo apt-get update && sudo apt-get install -y protobuf-compiler
  - uses: actions/setup-node@v4
    with: { node-version: "22" }
  - uses: pnpm/action-setup@v4
    with: { version: "10" }
  - run: pnpm install --frozen-lockfile
    working-directory: frontend
  - uses: dtolnay/rust-toolchain@stable
  - uses: Swatinem/rust-cache@v2
  - run: cargo install cross --git https://github.com/cross-rs/cross
  - run: cross build --release --target ${{ matrix.target }} -p bin_console
  - run: cross build --release --target ${{ matrix.target }} -p bin_rpc
  - name: Package binaries
    run: |
      TARGET=${{ matrix.target }}
      mkdir -p dist
      cp target/$TARGET/release/bin_console dist/
      cp target/$TARGET/release/bin_rpc dist/
      tar -czf bin_console-$TARGET.tar.gz -C dist bin_console
      tar -czf bin_rpc-$TARGET.tar.gz -C dist bin_rpc
  - uses: actions/upload-artifact@v4
    with:
      name: binaries-${{ matrix.target }}
      path: "*.tar.gz"
```

### Job: `release` (needs: build)

```yaml
runs-on: ubuntu-latest
permissions:
  contents: write

steps:
  - uses: actions/download-artifact@v4
    with: { path: artifacts, merge-multiple: true }
  - uses: softprops/action-gh-release@v2
    with:
      files: artifacts/*.tar.gz
      generate_release_notes: true
```

---

## Stage 3 — Docker Workflow

**File:** `.github/workflows/docker.yml`  
**Trigger:** push of tag `v*`; push to `main` (publishes `:latest`)

### Images published

| Image name | Dockerfile | Purpose |
|---|---|---|
| `ghcr.io/<owner>/bitnode-console` | `docker/Dockerfile.console` | bin_console (HTTP + gRPC, embeds frontend) |
| `ghcr.io/<owner>/bitnode-rpc` | `docker/Dockerfile.rpc` | bin_rpc (gRPC-only, headless) |
| `ghcr.io/<owner>/bitnode-frontend` | `docker/Dockerfile.frontend` | React SPA on nginx |
| `ghcr.io/<owner>/bitnode-node` | `docker/Dockerfile.node` | Console + Bitcoin Knots via supervisord |

### Job: `build-and-push` (matrix over the four images above)

```yaml
runs-on: ubuntu-latest
permissions:
  contents: read
  packages: write

steps:
  - uses: actions/checkout@v4
  - uses: docker/setup-qemu-action@v3          # linux/arm64 emulation
  - uses: docker/setup-buildx-action@v3
  - uses: docker/login-action@v3
    with:
      registry: ghcr.io
      username: ${{ github.actor }}
      password: ${{ secrets.GITHUB_TOKEN }}
  - uses: docker/metadata-action@v5
    id: meta
    with:
      images: ghcr.io/${{ github.repository_owner }}/${{ matrix.image }}
      tags: |
        type=semver,pattern={{version}}
        type=semver,pattern={{major}}.{{minor}}
        type=raw,value=latest,enable={{is_default_branch}}
  - uses: docker/build-push-action@v6
    with:
      context: .
      file: docker/Dockerfile.${{ matrix.image }}
      platforms: linux/amd64,linux/arm64
      push: true
      tags: ${{ steps.meta.outputs.tags }}
      labels: ${{ steps.meta.outputs.labels }}
      cache-from: type=gha
      cache-to: type=gha,mode=max
      build-args: |
        BITCOIN_KNOTS_VERSION=27.1.knots20240801
```

---

## Stage 4 — Dockerfiles

### `docker/Dockerfile.console`

Multi-stage build. The pre-built `frontend/dist` is copied into the Rust stage so
`lib_web/build.rs`'s `npm run build` finds the assets and exits fast.

```dockerfile
# --- Stage 1: Build frontend ---
FROM node:22-bookworm-slim AS node-builder
WORKDIR /workspace/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

# --- Stage 2: Build Rust binary ---
FROM rust:1-bookworm AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler curl ca-certificates && rm -rf /var/lib/apt/lists/*
# Install Node 22 (needed by lib_web/build.rs which calls npm run build)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs && rm -rf /var/lib/apt/lists/*
RUN corepack enable

WORKDIR /workspace
# Pre-populate node_modules so npm run build is a cache hit
COPY --from=node-builder /workspace/frontend/node_modules frontend/node_modules
COPY --from=node-builder /workspace/frontend/dist frontend/dist
COPY . .
RUN cargo build --release -p bin_console

# --- Stage 3: Minimal runtime ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /workspace/target/release/bin_console /usr/local/bin/
EXPOSE 3000 50051
ENTRYPOINT ["bin_console"]
```

### `docker/Dockerfile.rpc`

Identical to `Dockerfile.console` but builds `bin_rpc` in stage 2 and exposes only
the gRPC port.

```dockerfile
FROM node:22-bookworm-slim AS node-builder
WORKDIR /workspace/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

FROM rust:1-bookworm AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs && rm -rf /var/lib/apt/lists/*
RUN corepack enable
WORKDIR /workspace
COPY --from=node-builder /workspace/frontend/node_modules frontend/node_modules
COPY --from=node-builder /workspace/frontend/dist frontend/dist
COPY . .
RUN cargo build --release -p bin_rpc

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /workspace/target/release/bin_rpc /usr/local/bin/
EXPOSE 50051
ENTRYPOINT ["bin_rpc"]
```

### `docker/Dockerfile.frontend`

```dockerfile
FROM node:22-bookworm-slim AS builder
WORKDIR /app
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
```

### `docker/nginx.conf`

```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

### `docker/Dockerfile.node`

Uses `TARGETARCH` (injected by buildx) to select the correct Bitcoin Knots tarball URL.

```dockerfile
# --- Stage 1: Build frontend ---
FROM node:22-bookworm-slim AS node-builder
WORKDIR /workspace/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

# --- Stage 2: Build bin_console ---
FROM rust:1-bookworm AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs && rm -rf /var/lib/apt/lists/*
RUN corepack enable
WORKDIR /workspace
COPY --from=node-builder /workspace/frontend/node_modules frontend/node_modules
COPY --from=node-builder /workspace/frontend/dist frontend/dist
COPY . .
RUN cargo build --release -p bin_console

# --- Stage 3: Runtime with Bitcoin Knots ---
FROM debian:bookworm-slim
ARG BITCOIN_KNOTS_VERSION=27.1.knots20240801
ARG TARGETARCH

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 wget supervisor && rm -rf /var/lib/apt/lists/*

# Download the correct Bitcoin Knots tarball for the build architecture
RUN BK_MAJOR=$(echo "$BITCOIN_KNOTS_VERSION" | cut -d. -f1) && \
    case "$TARGETARCH" in \
      amd64) BK_ARCH="x86_64-linux-gnu" ;; \
      arm64) BK_ARCH="aarch64-linux-gnu" ;; \
      *) echo "Unsupported arch: $TARGETARCH" && exit 1 ;; \
    esac && \
    wget -q "https://bitcoinknots.org/files/${BK_MAJOR}.x/${BITCOIN_KNOTS_VERSION}/bitcoin-${BITCOIN_KNOTS_VERSION}-${BK_ARCH}.tar.gz" \
         -O /tmp/bitcoin-knots.tar.gz && \
    tar -xzf /tmp/bitcoin-knots.tar.gz -C /tmp && \
    install -m 0755 /tmp/bitcoin-${BITCOIN_KNOTS_VERSION}/bin/bitcoind /usr/local/bin/ && \
    install -m 0755 /tmp/bitcoin-${BITCOIN_KNOTS_VERSION}/bin/bitcoin-cli /usr/local/bin/ && \
    rm -rf /tmp/bitcoin*

COPY --from=rust-builder /workspace/target/release/bin_console /usr/local/bin/
COPY docker/supervisord.conf /etc/supervisor/conf.d/bitnode.conf

RUN useradd -r -s /bin/false bitcoin && \
    mkdir -p /home/bitcoin/.bitcoin /etc/bitnode && \
    chown bitcoin:bitcoin /home/bitcoin/.bitcoin

VOLUME ["/home/bitcoin/.bitcoin", "/etc/bitnode"]
EXPOSE 3000 50051 8333 8332

CMD ["supervisord", "-n", "-c", "/etc/supervisor/supervisord.conf"]
```

### `docker/supervisord.conf`

```ini
[supervisord]
nodaemon=true
logfile=/dev/null
logfile_maxbytes=0

[program:bitcoind]
command=bitcoind -conf=/home/bitcoin/.bitcoin/bitcoin.conf -datadir=/home/bitcoin/.bitcoin
user=bitcoin
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0

[program:bitnode-console]
command=bin_console
autostart=true
autorestart=true
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
```

---

## Implementation Order

1. **Stage 1** — `.github/workflows/ci.yml` (unblocks PR feedback loop)
2. **Stage 2** — `.github/workflows/release.yml` (binary artifacts on tags)
3. **Stage 3** — `.github/workflows/docker.yml`
4. **Stage 4** — `docker/Dockerfile.*` + `docker/nginx.conf` + `docker/supervisord.conf`

---

## Open Questions / Notes for Review

- **Bitcoin Knots version** `27.1.knots20240801` is used as a default. Update the
  `BITCOIN_KNOTS_VERSION` build-arg in `docker.yml` and `Dockerfile.node` when a
  newer release is available.
- **Ports** for bin_console (`3000`, `50051`) are placeholders — confirm against
  actual `bitnode_console.conf` defaults.
- **`cross` build time** — installing `cross` from git on every release run adds
  ~3–5 min. Consider caching the `cross` binary or pinning to a released version
  via `cargo-binstall`.
- **Frontend tsc check** — `pnpm exec tsc --noEmit` requires `typescript` in
  `devDependencies`; verify it is present in `frontend/package.json`.
- ~~**`lib_web/build.rs`** calls `npm` (not `pnpm`). The Node 22 install provides
  `npm` as a shim; verify `npm run build` resolves to pnpm's script runner, or
  change the build.rs to call `pnpm` directly.~~ **Done** — `build.rs` already
  spawns `pnpm run build`; stale `npm` strings in the panic messages have been
  updated to match.
