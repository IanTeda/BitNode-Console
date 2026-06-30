# Journald Log Viewer — Implementation Plan

## Context

BitNode Console needs to surface Bitcoin Knots/Core daemon logs in the browser without
requiring SSH access. The backend already has a gRPC-Web stack (tonic + tonic-web) that
the React frontend consumes via `@protobuf-ts`. A `lib_journald` skeleton crate exists
but is empty. This plan fills it out end-to-end: new proto service → backend
implementation → frontend streaming log viewer.

---

## Part 1: Dev environment for bitcoind-knots

**Approach:** shell.nix extension + `systemd-cat` wrapper script.

Add to `shell.nix` packages:
```nix
bitcoin    # provides bitcoind binary (or bitcoinknots if available in nixpkgs)
systemd    # provides journalctl + systemd-cat
```

Create `dev/` directory (data/ gitignored, scripts committed):

**`dev/bitcoin.conf`:**
```ini
regtest=1
server=1
rpcuser=bitcoinrpc
rpcpassword=devpassword
```

**`dev/run-bitcoind.sh`:**
```sh
#!/usr/bin/env bash
mkdir -p "$(dirname "$0")/data"
bitcoind \
  -conf="$(pwd)/dev/bitcoin.conf" \
  -datadir="$(pwd)/dev/data" \
  2>&1 | systemd-cat -t bitcoind
```

Piping through `systemd-cat -t bitcoind` sends stdout/stderr into the user journal
under the identifier `bitcoind`. Verify with: `journalctl -t bitcoind -f`.

For development, override `daemon.unit_name` in a local config override or via
environment variable (`BITNODE_CONSOLE_DAEMON_UNIT_NAME=bitcoind`) since
`systemd-cat` produces `_SYSLOG_IDENTIFIER=bitcoind` rather than a `.service` unit.
Alternatively, create a proper systemd user unit at
`~/.config/systemd/user/bitcoind.service` so the service name matches production.


---

## Part 2: Protobuf definition

**New file:** `backend/libs/lib_rpc/protos/bitnode_console/v1/journald/journald.proto`
Package: `bitnode_console.v1.journald.v1`

```protobuf
syntax = "proto3";
package bitnode_console.v1.journald.v1;

service JournaldService {
  // Fetch the last N log lines (unary, for initial page load).
  rpc GetLogs(GetLogsRequest) returns (GetLogsResponse);
  // Stream new log entries in real time (server-streaming, for live tail).
  rpc StreamLogs(StreamLogsRequest) returns (stream LogEntry);
}

message GetLogsRequest  { uint32 lines = 1; }
message GetLogsResponse { repeated LogEntry entries = 1; }
message StreamLogsRequest { uint32 tail_lines = 1; }

message LogEntry {
  string message      = 1;
  int64  timestamp_us = 2;  // microseconds since Unix epoch
  string priority     = 3;  // "debug"|"info"|"warning"|"err"|"crit"
  string unit         = 4;
}
```

Note: unit name is NOT a request field — the server uses the configured
`daemon.unit_name` from settings, preventing authenticated users from
reading arbitrary systemd units.

**Mirror proto** at `frontend/protos/bitnode_console/v1/journald/journald.proto`
(identical file — frontend and backend keep proto sources in sync manually).

**Update `lib_rpc/build.rs`:** add journald.proto to `compile_protos` list alongside
utilities and authentication.

**Update `lib_rpc/src/generated_protos/mod.rs`:** add:
```rust
#[path = "bitnode_console.v1.journald.v1.rs"]
pub mod journald;
```

---

## Part 3: lib_journald implementation

**Approach:** subprocess `journalctl --output json` — no libsystemd C bindings,
works anywhere journalctl is in PATH, avoids adding systemd to shell.nix for the
library itself.

**Add to `lib_journald/Cargo.toml`:**
```toml
tokio = { workspace = true }        # needs process + io features
serde = { workspace = true }
serde_json = { workspace = true }
async-stream = "0.3"                # stream! macro for streaming RPCs
```

**Key types (`lib_journald/src/lib.rs`):**
```rust
pub struct JournalEntry {
    pub message: String,
    pub timestamp_us: i64,
    pub priority: String,
    pub unit: String,
}
```

Map `journalctl` JSON fields: `MESSAGE` → message, `__REALTIME_TIMESTAMP` → timestamp_us
(string µs since epoch), `PRIORITY` → "0"–"7" mapped to name, `_SYSTEMD_UNIT` → unit.

**Public API:**
- `pub async fn fetch(unit: &str, lines: u32) -> Result<Vec<JournalEntry>, Error>`
  Spawns: `journalctl -u {unit} -n {lines} --output json --no-pager`
  Reads stdout to completion, parses each line as JSON.

- `pub fn stream(unit: &str, tail: u32) -> impl Stream<Item = Result<JournalEntry, Error>>`
  Spawns: `journalctl -u {unit} -n {tail} -f --output json`
  Yields entries via `async_stream::stream!` reading `tokio::io::BufReader` line-by-line.

---

## Part 4: lib_settings — DaemonSettings

**New file:** `backend/libs/lib_settings/src/daemon.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    #[serde(default = "default_unit_name")]
    pub unit_name: String,
}
fn default_unit_name() -> String { "bitcoind.service".to_string() }
impl Default for DaemonSettings { ... }
```

Follow the exact pattern of `RpcSettings` / `WebSettings` (accessor methods, `Default`
impl, unit tests for parse + default).

**Update `lib_settings/src/settings.rs`:** add `pub daemon: DaemonSettings` to
`Settings`.

**Update `bitnode_console.conf`:**
```ini
[daemon]
unit_name = bitcoind.service
```

---

## Part 5: lib_rpc — JournaldService

### Service impl files (follow utilities pattern exactly)

```
lib_rpc/src/services/journald/
├── mod.rs         — JournaldServiceImpl { unit: String } + trait impl
├── get_logs.rs    — calls lib_journald::fetch(), maps to proto GetLogsResponse
└── stream_logs.rs — calls lib_journald::stream(), wraps in tonic Streaming
```

`JournaldServiceImpl::new(unit: String)` stores the unit name.
Both RPCs delegate to handler functions in their own module (same pattern as
`services/utilities/ping.rs`).

**Update `services/mod.rs`:** re-export `JournaldServiceImpl` and
`JournaldServiceServer` (from generated_protos).

### Wire into server.rs

`Server` currently holds only `RpcSettings`. Add `daemon: lib_settings::DaemonSettings`
as a stored field alongside `settings`. Update `Server::new()` signature:

```rust
pub async fn new(
    rpc: lib_settings::RpcSettings,
    daemon: lib_settings::DaemonSettings,
) -> crate::Result<Self>
```

In `run()`, register with the access-token interceptor (same as utilities):
```rust
let journald_service = JournaldServiceServer::with_interceptor(
    JournaldServiceImpl::new(self.daemon.unit_name.clone()),
    access_token_interceptor,
);
```
Add `.add_service(journald_service)` to the server builder chain.

**Update callers** (`bin_console` and `bin_rpc` main.rs) to pass `settings.daemon`
to `Server::new()`.

---

## Part 5: Frontend log viewer

### Proto generation

Update `frontend/package.json` `proto:gen` script to include journald.proto in the
`protoc` invocation alongside utilities.proto.

### gRPC-Web client

**New file:** `frontend/src/lib/rpc/journald.ts`

Follow `utilities.ts` pattern: create `JournaldServiceClient` with
`GrpcWebFetchTransport` and `accessTokenInterceptor`. Export `journaldClient()`
singleton factory.

### Log viewer page

**New route:** `frontend/src/routes/_restricted/dashboard/logs.tsx`

- On mount: call `GetLogs` for the last 200 lines, render history.
- Then open `StreamLogs` server-streaming call; append each `LogEntry` to state.
- Auto-scroll to bottom (unless user has scrolled up — pause scroll lock).
- Display: monospace list, timestamp + priority badge (colour-coded) + message.
- Use existing shadcn `Badge` and `ScrollArea` components.

Add a nav link to the logs page in the dashboard sidebar/nav.


---

## Verification

1. `cargo test -p lib_journald` — unit tests for `fetch`/`stream` (mock journalctl
   script in PATH or test against a known-present unit like `dbus.service`)
2. `cargo test -p lib_settings` — confirm `DaemonSettings` parses `unit_name` from INI
3. `cargo build` — confirms proto compilation and all crates compile clean
4. `npm run proto:gen` in `frontend/` — regenerates TypeScript stubs including journald
5. `cargo run -p bin_console` — start full server
6. Open browser → `/dashboard/logs` → observe initial history then live streaming entries
7. In a second terminal: `journalctl -u bitcoind.service -f` — same entries as browser
