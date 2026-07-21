## Plan: `lib_bitcoind` client + `lib_rpc` daemon service (dashboard scope)

### 0. Important context up front

Bitcoin Core/Knots does **not** expose a gRPC/protobuf interface, so there are
**no `.proto` definitions to fetch or compile** for the daemon. `bitcoind`
speaks **JSON-RPC over HTTP** (HTTP `POST` to `/`, body is a JSON-RPC envelope,
auth via HTTP Basic using `rpcuser`/`rpcpassword`, or via the `.cookie` file).

The protobuf/tonic stack already in this repo (`lib_rpc`, the `.proto` files
under `lib_rpc/protos/`) is the **frontend ↔ backend** transport only. It is
unrelated to how the backend talks to the daemon.

So there are two pieces to build:

1. **`lib_bitcoind`** — a small, typed JSON-RPC *client* library for the daemon.
2. **A `daemon` gRPC *service* in `lib_rpc`** — surfaces that client to the
   React frontend, mirroring the existing journals service.

```
React frontend ──gRPC/gRPC-Web──▶ lib_rpc DaemonService ──JSON-RPC/HTTP──▶ bitcoind
                                        │
                                        └── uses lib_bitcoind (this plan)
```

### 0a. Client approach (decided): `corepc-types` + our own async transport

We do **not** use `corepc-client` or `bitcoincore-rpc` directly:

- Both are **blocking/synchronous**; using them inside this async/tonic backend
  would force `tokio::task::spawn_blocking` around every call.
- The corepc maintainers **explicitly advise against `corepc-client` in
  production**: *"If you require a JSON RPC client in production software it is
  expected you write your own and only use the `corepc-types` crate in your
  dependency graph."* `bitcoincore-rpc` is deprecated and points to corepc.

Instead we take the **maintainer-endorsed hybrid**, which is what actually
future-proofs us:

- **Own thin async transport** (reqwest) — small, async-native, no
  `spawn_blocking`.
- **`corepc-types` for the response types** — `corepc-types` (v0.15.0, June
  2026) ships serde-`Deserialize` structs for **every Core release, modules
  `v17`–`v31`** (e.g. `corepc_types::v29::GetBlockchainInfo`), each tested
  against that version, plus `.into_model()` conversions to concrete
  `rust-bitcoin` types. The response-type definitions are the part that rots as
  Core evolves, and this crate maintains them for us.

Facts verified against corepc 0.15.0:
- Version modules are **always compiled — not feature-gated**; just
  `use corepc_types::vNN`.
- Default features (`std`) are fine. **Do not** enable
  `serde-deny-unknown-fields`, so unknown/newer daemon fields are ignored
  (forward-compatible across daemon versions).
- Transitively pulls `bitcoin = 0.32` (rust-bitcoin) — the accepted cost of
  future-proofing the types.

```
reqwest POST ──▶ corepc_types::v29::GetBlockchainInfo   (serde Deserialize, maintained per Core version)
                 └─ optional .into_model()? -> rust-bitcoin model types
```

---

### 1. Scope (locked): dashboard only, built to expand

The feasibility-stage dashboard (`product-requirements.md` §3.1) needs exactly:

- Status of services (daemon up/down, plus Tor / I2P / CJDNS reachability).
- Number of peers / connections.
- Blockchain verification progress (for the sync graph).

Two daemon RPC calls cover all of that, so **that is all we implement now**:

| Dashboard need                | Daemon RPC          | corepc-types response type |
|-------------------------------|---------------------|----------------------------|
| Sync graph, block height, IBD | `getblockchaininfo` | `corepc_types::v29::GetBlockchainInfo` |
| Peer count + network status   | `getnetworkinfo`    | `corepc_types::v29::GetNetworkInfo` |

Daemon **up/down** status is derived from whether these calls succeed (a
transport error ⇒ daemon down/unreachable). VPN status is not daemon-knowable
and is out of scope.

**Future expansion is designed in, not built:** adding `getpeerinfo`,
`uptime`, `getmempoolinfo`, etc. is a small additive change at each layer — a
new client wrapper (the corepc-types struct already exists), a new proto rpc,
a new handler + `_from`. Each layer carries an explicit `// Future:` comment
marking the extension point (§3, §4).

---

### 2. Deliverables

1. New crate `backend/libs/lib_bitcoind`: `Client` + typed
   `get_blockchain_info()` / `get_network_info()` returning `corepc-types`
   response structs, one `thiserror` enum, Basic + cookie auth, unit-tested
   against a mock HTTP server.
2. New `daemon` gRPC service in `lib_rpc`: `daemon.proto`, generated types,
   `services/daemon/` handlers, `From`-conversions corepc-types→proto,
   registered in `server.rs` behind the existing access-token interceptor.
3. Green `cargo fmt` / `cargo clippy` / `cargo test`.

---

### 3. `lib_bitcoind` — architecture & conventions

#### 3a. Crate layout (mirrors `lib_journals`)

```
backend/libs/lib_bitcoind/
├── Cargo.toml
└── src/
    ├── lib.rs            # module decls + flat re-exports + `Result` alias + version pin
    ├── error.rs          # thiserror Error enum + `pub type Result<T>`
    ├── client.rs         # Client: reqwest client, base URL, credential, generic `call`
    ├── auth.rs           # Credential: resolve Basic vs cookie-file
    ├── envelope.rs        # private JSON-RPC Request/Response<R> envelope
    └── methods/
        ├── mod.rs               # re-exports; `// Future: add modules here`
        ├── blockchain_info.rs   # get_blockchain_info() -> types::GetBlockchainInfo
        └── network_info.rs      # get_network_info() -> types::GetNetworkInfo
```

#### 3b. Version pin lives in one place

`lib.rs` centralises the Core-version choice so bumping it is a one-line change:

```rust
/// Bitcoin Core/Knots RPC version whose response types this client targets.
/// Bump this alias to track a newer daemon; unknown/new fields are ignored,
/// so reading the stable dashboard subset tolerates daemon-version drift.
pub use corepc_types::v29 as types;   // <- single future-proofing knob
```

Public methods return `types::GetBlockchainInfo` / `types::GetNetworkInfo`
(re-exported), so callers name them via `lib_bitcoind::types::…`.

#### 3c. Public API shape

One `Client`, one async method per RPC call, returning the corepc-types struct:

```rust
let client = lib_bitcoind::Client::new(&settings.bitcoind)?;  // holds one pooled reqwest::Client
let chain  = client.get_blockchain_info().await?;              // -> types::GetBlockchainInfo
let net     = client.get_network_info().await?;                // -> types::GetNetworkInfo
// Future: client.get_peer_info(), client.uptime(), client.get_mempool_info(), …
```

Conventions (consistent with the workspace):

- **Method-per-call, snake_case** mirroring the RPC name. Each method is a
  ~3-line wrapper over one private generic
  `Client::call<P: Serialize, R: DeserializeOwned>(&self, method, params) -> Result<R>`
  that owns the envelope + auth + HTTP + error mapping. Adding a method = one
  wrapper (the corepc-types struct already exists).
- **`Client` is cheap to clone** — holds a single connection-pooled
  `reqwest::Client`. Construct once, share.
- **Return corepc-types response structs directly** (they derive
  `Debug, Clone, Deserialize`). Callers that want strongly-typed rust-bitcoin
  values can call `.into_model()`; the dashboard only needs scalar fields, so
  the raw structs suffice and no rust-bitcoin types leak into our conversions.
- **One `thiserror` enum + `pub type Result<T>`** (like `lib_journals`).
- **`///` rustdoc on every public item, Australian English** (CLAUDE.md),
  noting which daemon RPC each method wraps.
- **No `unwrap`/`expect` outside tests**; `?` + `Result` throughout.
- **`SecretString`** preserved end-to-end; auth built at call time via reqwest's
  `.basic_auth()` and never logged.

#### 3d. Error enum

```rust
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bitcoind HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),            // daemon down / connection refused

    #[error("Bitcoind RPC error {code}: {message}")]
    Rpc { code: i32, message: String },           // JSON-RPC `error` object

    #[error("Bitcoind authentication failed (HTTP 401)")]
    Unauthorized,

    #[error("Bitcoind response decode error: {0}")]
    Decode(String),

    #[error("Bitcoind cookie file error: {0}")]
    Cookie(#[source] std::io::Error),

    #[error("Bitcoind client configuration error: {0}")]
    Config(String),
}
```

#### 3e. JSON-RPC transport details

- **Request:** `POST http://<rpc_host>:<rpc_port>/`, body
  `{"jsonrpc":"1.0","id":"bitnode-console","method":"getblockchaininfo","params":[]}`.
  Use `"1.0"` for widest Core/Knots compatibility.
- **Response:** `{"result":<value|null>,"error":<null|{code,message}>,"id":…}`.
  Our private `Response<R>` envelope deserialises `result` into the corepc-types
  `R`. Non-null `error` ⇒ `Error::Rpc`.
- **Auth:** cookie file takes precedence when
  `settings.bitcoind.cookie_file()` is `Some` (read file; contents are the
  `user:pass` pair, split on the first `:`); otherwise Basic from
  `rpc_user()`/`rpc_password()`. Applied with reqwest
  `.basic_auth(user, Some(pass))` — **no `base64` dependency needed**.
- **HTTP status:** `401` ⇒ `Error::Unauthorized`; other non-2xx surfaced
  appropriately. `BitcoinDaemonSettings` already provides `rpc_url()`,
  `rpc_user()`, `rpc_password()` (`SecretString`), `cookie_file()`.

#### 3f. `Cargo.toml`

```toml
[package]
name = "lib_bitcoind"
description.workspace = true
repository.workspace = true
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[lints]
workspace = true

[dependencies]
lib_settings = { path = "../../libs/lib_settings" }

secrecy = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }

corepc-types = "0.15"        # version modules v17..v31; default `std`; do NOT enable serde-deny-unknown-fields
reqwest = { version = "0.12", default-features = false, features = ["json"] }
serde_json = { version = "1.0" }

[dev-dependencies]
wiremock = "0.6"
tokio = { workspace = true }
```

Auto-included by the existing `members = ["backend/libs/*"]` glob; no `build.rs`.
Note: pulls `bitcoin = 0.32` transitively via corepc-types — expected. Watch for
`clippy::pedantic`/`nursery` friction from the new dep tree; keep our own code
clean (third-party code is not linted, only compiled).

---

### 4. `daemon` gRPC service in `lib_rpc`

Follows the journals service pattern exactly (unary calls, like utilities).

#### 4a. Proto — `protos/bitnode_console/daemon/daemon.proto`

```proto
syntax = "proto3";
package bitnode_console.daemon.v1;   // matches bitnode_console.journals.v1 convention

// DaemonService exposes read-only Bitcoin daemon status for the dashboard.
// The daemon connection is fixed by server configuration; clients cannot
// target an arbitrary node.
service DaemonService {
  rpc GetBlockchainInfo (GetBlockchainInfoRequest) returns (BlockchainInfo);
  rpc GetNetworkInfo    (GetNetworkInfoRequest)    returns (NetworkInfo);
  // Future: rpc GetPeerInfo, rpc GetUptime, rpc GetMempoolInfo, rpc GetNetTotals …
}

message GetBlockchainInfoRequest {}
message GetNetworkInfoRequest {}

message BlockchainInfo {
  string chain = 1;
  uint64 blocks = 2;
  uint64 headers = 3;
  double verification_progress = 4;
  bool   initial_block_download = 5;
  uint64 size_on_disk = 6;
  bool   pruned = 7;
  // Future: append fields as the dashboard grows.
}

message NetworkInfo {
  uint64 version = 1;
  string subversion = 2;          // identifies Core vs Knots
  uint32 connections = 3;
  uint32 connections_in = 4;
  uint32 connections_out = 5;
  repeated NetworkReachability networks = 6;
  // Future: relay_fee, warnings, …
}

message NetworkReachability {
  string name = 1;      // ipv4 | ipv6 | onion | i2p | cjdns
  bool   reachable = 2;
}
```

The proto stays our own hand-written contract (decoupled from corepc-types), so
the frontend API is stable even if the corepc-types version pin bumps.

#### 4b. Register the proto in the build (2 edits)

- `build.rs`: add `proto_dir.join("bitnode_console/daemon/daemon.proto")` to the
  `compile_protos` list.
- `src/generated_protos/mod.rs`: add
  ```rust
  pub mod daemon {
      include!(concat!(env!("OUT_DIR"), "/bitnode_console.daemon.v1.rs"));
  }
  ```
  The combined `FILE_DESCRIPTOR_SET` (reflection) picks it up automatically.

#### 4c. Service modules — `src/services/daemon/`

```
services/daemon/
├── mod.rs                  # re-exports client/server/messages + Impl
├── service_impl.rs         # DaemonServiceImpl { client: lib_bitcoind::Client }
├── get_blockchain_info.rs  # handle() -> convert corepc-types -> proto
├── get_network_info.rs     # handle() -> convert corepc-types -> proto
├── blockchain_info_from.rs # From<lib_bitcoind::types::GetBlockchainInfo> for proto BlockchainInfo
└── network_info_from.rs    # From<lib_bitcoind::types::GetNetworkInfo>    for proto NetworkInfo
                            # // Future: one handler + one _from module per new rpc
```

- `DaemonServiceImpl::new(&BitcoinDaemonSettings) -> crate::Result<Self>` builds
  and stores a cloneable `lib_bitcoind::Client`.
- Handlers call the client, map `lib_bitcoind::Error` → `tonic::Status` via
  `crate::Error`, and convert corepc-types → proto via `From` (same pattern as
  `journals/entry_from.rs`). Note: some corepc-types fields may be typed
  differently from the proto (e.g. numeric width, `networks` element shape);
  the `_from` conversions handle the mapping and any needed
  `try_into`/`unwrap_or_default`.

#### 4d. Error plumbing — `src/error.rs` (2 edits)

- Add `#[error("Daemon error: {0}")] Daemon(#[from] lib_bitcoind::Error)`.
- Extend `From<Error> for tonic::Status`: map daemon transport failures to
  `tonic::Status::unavailable(..)` (so the dashboard can show "daemon
  unreachable" distinctly), others to `internal`.

#### 4e. Register in `src/server.rs` (mirrors the journals block)

```rust
let daemon_access_token_interceptor =
    crate::interceptors::AccessTokenInterceptor::new(self.settings.backend.token_secret().clone());
let daemon_service = DaemonServiceServer::with_interceptor(
    DaemonServiceImpl::new(&self.settings.bitcoind)?,
    daemon_access_token_interceptor,
);
// … .add_service(daemon_service)
```

#### 4f. `lib_rpc/Cargo.toml`

Add `lib_bitcoind = { path = "../../libs/lib_bitcoind" }`.

---

### 5. Testing strategy

**`lib_bitcoind` (unit, default `cargo test`):** `wiremock` mock server; assert
- happy path deserialises each method into the corepc-types struct from
  captured fixture JSON (a real `getblockchaininfo` / `getnetworkinfo` body);
- JSON-RPC `error` object ⇒ `Error::Rpc { code, message }`;
- `401` ⇒ `Error::Unauthorized`; malformed body ⇒ `Error::Decode`;
- Basic vs cookie auth produces the expected `Authorization` header
  (wiremock request matcher);
- credentials never appear in `Debug`/`tracing` output.
- Opt-in `tests/regtest.rs` gated on an env var (e.g. `BITNODE_REGTEST_URL`)
  for a live `bitcoind -regtest`; skipped in CI without a daemon.

**`lib_rpc` daemon service:** unit-test the `From` conversions
(corepc-types→proto field mapping); integration-test the service end-to-end
against a `wiremock` daemon behind a real `Server`, following the existing
ping/journals test style (incl. access-token enforcement).

Use `fake` for generated test data (CLAUDE.md). Gate on
`cargo fmt` / `cargo clippy` / `cargo test`.

---

### 6. Step-by-step task list

**`lib_bitcoind`**
1. Scaffold crate (`Cargo.toml` + `lib.rs` with the `corepc_types::v29` version
   alias); `cargo build -p lib_bitcoind`.
2. `error.rs` — `Error` + `Result<T>` + tests.
3. `envelope.rs` — private `Request`/`Response<R>` + `error` object + tests.
4. `auth.rs` — `Credential` (Basic vs cookie file) + tests
   (incl. secret-not-leaked).
5. `client.rs` — `Client::new(&BitcoinDaemonSettings)` + generic `call<P,R>`
   (reqwest `.basic_auth`); wiremock tests for transport/error/auth-header.
6. `methods/blockchain_info.rs`, `methods/network_info.rs` — thin wrappers
   returning corepc-types structs + fixture tests.
7. `lib.rs` — flat re-exports (`Client`, `Error`, `Result`, `types`).

**`lib_rpc` daemon service**
8. Add `daemon.proto`; wire into `build.rs` + `generated_protos/mod.rs`.
9. Add `lib_bitcoind` dep; add `Daemon` error variant + `tonic::Status` mapping.
10. `services/daemon/*` — `service_impl`, two handlers, two `_from` conversions,
    `mod.rs` re-exports.
11. Register `DaemonServiceServer` in `server.rs` behind the access-token
    interceptor; add integration tests.

**Finish**
12. `cargo fmt` / `cargo clippy` / `cargo test`; resolve lint friction.
13. Optional: mdBook note on the daemon RPC layer; promote
    `serde_json`/`reqwest` to `[workspace.dependencies]` (tidy-up).

---

### 7. Notes / assumptions / open decisions

- **Version pin:** default `corepc_types::v29`. Confirm which Core/Knots version
  the target daemon runs; the pin is one line in `lib.rs` and, because we read a
  stable field subset and don't deny unknown fields, tolerates drift.
- The proto is our own stable contract; corepc-types is an internal
  implementation detail of `lib_bitcoind` (not leaked to the frontend).
- Frontend TS/gRPC-Web client generation for `DaemonService` is a separate
  frontend task, not covered here.
- VPN status (§3.1 dashboard) is not daemon-derived and is out of scope.
</content>
