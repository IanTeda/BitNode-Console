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
bitcoin-knots     # provides bitcoind binary (or bitcoinknots if available in nixpkgs)
systemd           # provides journalctl + systemd-cat
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

## Part 3: lib_settings — DaemonSettings

**New file:** `backend/libs/lib_settings/src/bitcoind.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinDaemonSettings {
    #[serde(serialize_with = "serialize_secret_string")]
    pub unit_name: SecretString,
}
fn default_unit_name() -> String { "bitcoind.service".to_string() }
impl Default for BitcoinDaemonSettings { ... }
```

Follow the exact pattern of `RpcSettings` / `WebSettings` (accessor methods, `Default`
impl, unit tests for parse + default).

**Update `lib_settings/src/settings.rs`:** add `pub daemon: DaemonSettings` to
`Settings`.

**Update `bitnode_console.conf`:**
```ini
[bitcoind]
unit_name = bitcoind.service
```


---

## Part 4: lib_journald implementation

**Approach:** `systemd` crate (v0.10) — C FFI bindings to libsystemd. More complete
than `journald-query`: exposes all journal fields (including `PRIORITY`), supports
exact "last N lines" via `seek_tail` + `previous_skip`, and `match_or` allows
filtering by both `_SYSTEMD_UNIT` and `SYSLOG_IDENTIFIER` in one query. `systemd`
is already in `shell.nix` (added in Part 1) and provides the `libsystemd` shared
library the crate links against.

**File structure:**

```
backend/libs/lib_journald/
├── Cargo.toml          — add systemd, tokio (rt+sync), async-stream dependencies
└── src/
    ├── lib.rs          — public re-exports: fetch, stream, JournalEntry, Error, Result
    ├── error.rs        — Error enum (exists — extend with systemd + tokio variants)
    ├── domains.rs        — JournalEntry struct, from_record(), priority_name() mapping
    ├── fetch.rs        — fetch() async fn — spawn_blocking wrapper for GetLogs RPC
    └── stream.rs       — stream() fn — OS thread + mpsc channel bridge for StreamLogs RPC
```

**Key constraints:**

- `Journal` is explicitly `!Send + !Sync` (documented in the crate). It must be
  created and used on a single thread. Creating it *inside* a `spawn_blocking`
  closure is fine — the closure itself captures only `Send` data (e.g. `String`).
- Streaming uses a blocking `await_next_entry()` loop, so it needs a dedicated OS
  thread with an `mpsc` channel bridge (same as `journald-query`).

**Add to `lib_journald/Cargo.toml`:**
```toml
systemd = "0.10"
tokio = { workspace = true }   # needs rt + sync features for spawn_blocking + mpsc
async-stream = "0.3"           # stream! macro for the async wrapper
```

**Key types (`lib_journald/src/entry.rs`):**
```rust
pub struct JournalEntry {
    pub message: String,
    pub timestamp_us: i64,   // from journal.timestamp_usec() cast to i64
    pub priority: String,    // mapped from PRIORITY field ("0"–"7" → name)
    pub unit: String,        // from _SYSTEMD_UNIT or SYSLOG_IDENTIFIER field
}
```

Map `PRIORITY` values: `"0"` → `"emerg"`, `"1"` → `"alert"`, `"2"` → `"crit"`,
`"3"` → `"err"`, `"4"` → `"warning"`, `"5"` → `"notice"`, `"6"` → `"info"`,
`"7"` → `"debug"`. Anything else → `"info"`.

`next_entry()` / `next_entry_field()` returns `JournalRecord` (`BTreeMap<String, String>`)
containing all fields for the current entry. Read `MESSAGE`, `PRIORITY`,
`_SYSTEMD_UNIT` directly from the map.

**Opening the journal**

Use `OpenOptions` (no path required — libsystemd locates journal files itself):
```rust
// System journal (production: bitcoind.service)
let mut j = OpenOptions::default().system(true).open()?;

// Or user journal (if bitcoind runs as a user service)
let mut j = OpenOptions::default().current_user(true).open()?;
```

**Filtering**

Add a unit match and, in dev, an OR with the syslog identifier so both sources
are covered with one `Journal` handle:
```rust
j.match_add("_SYSTEMD_UNIT", unit)?;
j.match_or()?;
j.match_add("SYSLOG_IDENTIFIER", unit_without_service_suffix)?;
```

**Public API:**

- `pub async fn fetch(unit: &str, lines: u32) -> Result<Vec<JournalEntry>, Error>`

  Creates `Journal` inside `spawn_blocking` so it stays on one thread:
  ```rust
  tokio::task::spawn_blocking(move || {
      let mut j = OpenOptions::default().system(true).open()?;
      j.match_add("_SYSTEMD_UNIT", &unit)?;
      // Exact "last N lines": seek to tail, walk back N, collect forward
      j.seek(JournalSeek::Tail)?;
      j.previous_skip(lines as u64)?;
      let mut entries = Vec::new();
      while let Some(record) = j.next_entry()? {
          let ts = j.timestamp_usec()? as i64;
          entries.push(JournalEntry::from_record(&record, ts));
      }
      Ok(entries)
  }).await?
  ```

- `pub fn stream(unit: &str, tail_lines: u32) -> impl Stream<Item = Result<JournalEntry, Error>>`

  Spawns a plain OS thread so `Journal` never crosses a thread boundary. The thread
  emits `tail_lines` historical entries then follows new ones, sending all over an
  `mpsc` channel:
  ```rust
  let (tx, mut rx) = tokio::sync::mpsc::channel(64);
  std::thread::spawn(move || {
      let mut j = OpenOptions::default().system(true).open()?;
      j.match_add("_SYSTEMD_UNIT", &unit)?;
      // Historical tail
      j.seek(JournalSeek::Tail)?;
      j.previous_skip(tail_lines as u64)?;
      while let Some(record) = j.next_entry()? {
          let ts = j.timestamp_usec()? as i64;
          if tx.blocking_send(Ok(JournalEntry::from_record(&record, ts))).is_err() { return; }
      }
      // Follow new entries (blocks via sd_journal_wait internally)
      loop {
          match j.await_next_entry(None) {
              Ok(Some(record)) => {
                  let ts = j.timestamp_usec()? as i64;
                  if tx.blocking_send(Ok(JournalEntry::from_record(&record, ts))).is_err() { break; }
              }
              Ok(None) => {}
              Err(e) => { let _ = tx.blocking_send(Err(e)); break; }
          }
      }
  });
  async_stream::stream! {
      while let Some(item) = rx.recv().await { yield item; }
  }
  ```

**Dev note — `_SYSLOG_IDENTIFIER` vs `_SYSTEMD_UNIT`**

`systemd-cat -t bitcoind` writes `SYSLOG_IDENTIFIER=bitcoind`, not
`_SYSTEMD_UNIT`. Use `match_or()` to match both in one journal handle — no code
path duplication needed:
```rust
j.match_add("_SYSTEMD_UNIT", "bitcoind.service")?;
j.match_or()?;
j.match_add("SYSLOG_IDENTIFIER", "bitcoind")?;
```
This means `daemon.unit_name = "bitcoind.service"` in settings, and lib_journald
derives the bare identifier (`"bitcoind"`) by stripping the `.service` suffix for
the `SYSLOG_IDENTIFIER` match.

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
