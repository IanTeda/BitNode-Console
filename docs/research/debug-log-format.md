---
title: debug.log line format across Bitcoin Core and Knots versions
issue: 33
map: 32
status: research
date: 2026-10-10
sources:
  bitcoin-core:
    repo: bitcoin/bitcoin
    tags: [v22.0, v23.0, v24.0.1, v26.0, v27.0, v28.0, v29.0, v30.0, v31.0, v32.0rc3]
    master: 228877d7836a55c1d6fde90b0b7ba2d4c90248ab  # 2026-10-10
  bitcoin-knots:
    repo: bitcoinknots/bitcoin
    tag: v29.4.2.knots20260508
    commit: 58398baf33e588779685ead478e6397bb28ed3d6
files:
  - src/logging.cpp
  - src/logging.h
  - src/init/common.cpp
  - src/rpc/node.cpp
  - src/rpc/server.cpp
  - src/rest.cpp
  - src/zmq/zmqnotificationinterface.cpp
  - doc/release-notes/release-notes-{23..31}.0.md
---

## Summary

- A `debug.log` record is a fixed sequence of optional prefixes followed by
  the message: `[*] ` rate-limit marker, timestamp, `[thread]`,
  `[file:line] [function]`, a `[category]`/`[category:level]`/`[level]` tag,
  then the message text. Every prefix is optional and depends on version and
  flags.
- Up to and including Core v32 and Knots 29.x, a message **can** contain
  embedded `\n`, which is written raw. The physical lines that follow have
  **no prefix at all**. Core master (merged 2026-09-30 as #35833, so expected
  in v33) escapes them as `\x0a`, giving exactly one physical line per
  record.
- The level/category tag only exists from v24. Its rules were reworked in v27
  (#28318). v28 removed the `error()` helper, so `ERROR: ...` lines became
  `[error] ...` (#29236). v31 dropped the category from categorised
  warnings and errors (#34033).
- Startup writes `"\n\n\n\n\n"` (five newlines) to the file before the
  buffered early messages, in every version checked.
- **RPC/REST/ZMQ: no.** Neither Core nor Knots exposes log *contents* by any
  interface. The `logging` RPC only reads and sets which debug categories are
  enabled. `getrpcinfo` returns the log file's path (`logpath`), so a client
  can find it, but it still has to read the file.

## Record grammar

ABNF-ish. `SP` is one space, `LF` is `\n`. Order is fixed in all versions
(see `FormatLogStrInPlace` / `Format` in `src/logging.cpp`).

```text
record      = [rl-mark] [timestamp SP] [thread SP] [srcloc SP func SP]
              [tag SP] message LF
rl-mark     = "[*] "                       ; Core v30+, Knots 29.x only
timestamp   = date "T" hms [ "." 6DIGIT ] "Z" [ mock ]
date        = 4DIGIT "-" 2DIGIT "-" 2DIGIT
hms         = 2DIGIT ":" 2DIGIT ":" 2DIGIT
mock        = " (mocktime: " date "T" hms "Z)"   ; only when setmocktime > 0
thread      = "[" thread-name "]"           ; -logthreadnames; "unknown" if unnamed (v26+), "[]" in v22
srcloc      = "[" path ":" 1*DIGIT "]"      ; -logsourcelocations; leading "./" stripped
func        = "[" function-name "]"         ; see "Source locations" for shape
tag         = "[" category "]"              ; debug level with a category
            / "[" category ":" level "]"    ; any other level with a category
            / "[" level "]"                 ; uncategorised, level != info
message     = *( %x20-7E / %x80-FF / ctl-escape / raw-LF )
ctl-escape  = "\x" 2HEXDIG                  ; any byte < 0x20 (bar LF pre-master) or 0x7F
raw-LF      = LF                            ; up to v32/Knots 29.x only, starts an
                                            ; unprefixed continuation line
level       = "trace" / "debug" / "info" / "warning" / "error"
category    = "all" / <see Categories table>
```

Notes that matter for a parser:

- **Timestamps are always UTC** and always end in `Z`. They come from
  `FormatISO8601DateTime` (`%Y-%m-%dT%H:%M:%SZ`). With `-logtimemicros`, the
  `Z` is removed and `.%06dZ` is appended. With `-logtimestamps=0` there is no
  timestamp and no separating space.
- **The `[*] ` marker comes before the timestamp.** It is inserted at index 0
  after formatting whenever any source location is currently suppressed by the
  rate limiter. Core v30+ and Knots 29.x have this; Core v29 does not.
- **Bracket groups are ambiguous without knowing the flags.**
  - Thread names (`net`, `http`, `scheduler`, ...) collide with category names.
  - Message bodies can begin with brackets. For example, wallet messages
    start with the wallet display name, such as `[default wallet] ...`
    (`WalletLogPrintf` → `LogInfo("%s %s", GetDisplayName(), ...)`).

  BitNode Console manages `bitcoin.conf`, so it should read
  `logtimestamps`, `logtimemicros`, `logthreadnames`, `logsourcelocations` and
  `loglevelalways` from there and parse by position. It should accept a tag
  only when its contents match the category/level vocabulary. If something
  fails to parse, keep it as message text rather than dropping it.
- The file is opened in append mode and written **unbuffered**, one `fwrite`
  per record. In v29+ a record is always LF-terminated, because `\n` is
  appended if missing.
- `-shrinkdebugfile` (default on when no `-debug`) truncates an over-11 MB file
  to its last 10 MB at startup. **The first line of the file may therefore be
  a partial line.**
- Log reopen (SIGHUP, for logrotate) reopens the same path in append mode.
- Console output (`-printtoconsole`, e.g. under systemd/journald) uses the
  same formatted string, `[*] ` included. Rate-limited records are dropped
  from the **file only**; the console still gets them.

## Flags and defaults

From `src/init/common.cpp` and `src/logging.h` (identical in Core master and
Knots 29.x):

| Flag | Default | Effect on the line |
| --- | --- | --- |
| `-logtimestamps` | 1 | ISO 8601 UTC timestamp prefix |
| `-logtimemicros` | 0 | `.ffffff` microseconds (debug-only option) |
| `-logthreadnames` | 0 | `[thread] ` prefix |
| `-logsourcelocations` | 0 | `[file:line] [function] ` prefix |
| `-loglevelalways` | 0 | always `[category:level]`, `all` for uncategorised (v27+) |
| `-logips` | 0 | content only: peer IPs included in messages; no format change |
| `-logratelimit` | 1 | v30+/Knots: per-source 1 MiB/hour cap on unconditional logs, `[*] ` marker |
| `-debug=<cat>` / `-debugexclude` | off | enables debug/trace records for categories |
| `-loglevel=<lvl>\|<cat>:<lvl>` | debug | v24+: which of info/debug/trace pass for enabled categories |
| `-shrinkdebugfile` | 1 if no `-debug` | startup truncation (see above) |

## Tag rules by version

### v22, v23: no tag

`LogPrintf` and `LogPrint(category, ...)` both write the bare message. The
category is never printed and levels do not exist. Errors were logged by the
`error()` helper as the text `ERROR: ...`.

### v24 to v26: `Level::None` era

`BCLog::Level` gains `Trace, Debug, Info, Warning, Error, None`. The prefix is
written whenever `category != NONE || level != None`:

| Call | Prefix |
| --- | --- |
| `LogPrintf` | none |
| `LogPrint(cat, ...)` / `LogPrintfCategory(cat, ...)` | `[cat] ` |
| `LogPrintLevel(cat, lvl, ...)` | `[cat:lvl] ` |
| `error(...)` | none, text `ERROR: ...` |

### v27: #28318 "Simplify API for level based logging"

`Level::None` is removed. `GetLogPrefix()` applies these rules, and they are
unchanged through master:

- Category `NONE` is treated as `ALL` (uncategorised).
- Uncategorised + `info` → **no tag**.
- Uncategorised + other level → `[warning] `, `[error] ` (`[debug]`/`[trace]`
  are not emitted in practice).
- Categorised + `debug` → `[cat] ` (debug is implied).
- Categorised + other level → `[cat:trace] `, `[cat:info] `, `[cat:warning] `,
  `[cat:error] `.
- `-loglevelalways=1` → always `[cat:level] `, uncategorised as `all`, for
  example `[all:info] `, `[net:debug] `.

New macros: `LogInfo`, `LogWarning`, `LogError`, `LogDebug`, `LogTrace`.
`LogPrintf` becomes an alias for `LogInfo`. The `error()` helper still exists
in v27 (`logging.h` writes `LogPrintf("ERROR: %s\n", ...)`), so `ERROR: `
lines remain.

### v28: #29236 "Nuke error(...)"

`error()` is removed and its call sites use `LogError`, so the old `ERROR: `
text becomes the `[error] ` tag. This is the "v28 level change" visible in
logs. `util` is removed as a category.

### v31: #34033 "Unify error and warning log formatting"

Categorised `[net:error]` and `[net:warning]` call sites are rewritten to
`LogError`/`LogWarning`. They now print `[error] ` / `[warning] `, or
`[all:error]` with `-loglevelalways`. The prefix rules themselves are
unchanged.

### Knots 29.x

Knots 29.x uses the Core v29 tag rules, so `[net:warning]`-style categorised
warnings still occur.

## Continuation and multi-line behaviour

| Version | Message without trailing `\n` | Embedded `\n` |
| --- | --- | --- |
| v22 to v28 | next call is appended to the same line **without prefix** (`m_started_new_line`) | written raw; next physical line has no prefix |
| v29 to v32, Knots 29.x | each call is its own record; `\n` appended if missing | written raw; next physical line has no prefix |
| master (v33) | each call is its own record; one trailing `\n` stripped, then re-added | escaped as `\x0a` |

`LogEscapeMessage` escapes every byte `< 0x20` and `0x7F` as `\xNN` in all
versions. Before master it exempted `\n`. Master's #35833 (merge
`d4b0e1e4b078dbb4ab57886d288470684dec5718`, 2026-09-30) removed the
exemption, specifically to stop RPC users from forging log lines. It was
merged after the v32 branch, so v32.0rc3 still writes raw newlines.

**Parser rule for v32 and earlier and Knots:** a physical line that does not
start with a valid timestamp (when `-logtimestamps=1`), or with `[*] ` and
then a timestamp, belongs to the previous record. With `-logtimestamps=0`,
continuation lines cannot be told apart from records.

## Source locations

`-logsourcelocations` adds `[path:line] [function] `. The function part
varies by version:

- v22 to v29: `__func__`, the plain name, e.g. `[ProcessMessage]`.
- v30 and Knots 29.x: `std::source_location::function_name()`, the **full
  signature** (v30 release notes, #32604). It can contain spaces, parentheses,
  commas and, with GCC templates, a nested `[with T = ...]`. **Match brackets
  with balancing, not `[^\]]*`.**
- v31+: `function_name_short()`, back to the plain name (v31 release notes,
  #34088).

## Startup marker

`StartLogging()` writes `"\n\n\n\n\n"` to the file when it opens it. This is
the same in v22, v26, v30, v31, master and Knots 29.x. Because the previous
run ended in `\n`, a restart shows up as **four empty lines** followed by the
buffered early records. The first record is the version banner,
`LogPrintf(CLIENT_NAME " version %s\n", ...)`, for example
`Bitcoin Knots version v29.4.2.knots20260508 (release build)`.

From v28, early records are buffered with their original timestamp and
formatted on flush. If the buffer overflows, a record
`Early logging buffer overflowed, N log lines discarded.` precedes them.

## Categories

From `LogCategories` / `LOG_CATEGORIES_BY_STR` in `src/logging.cpp`. The
special values `0`/`none` and `1`/`all` are accepted by `-debug`; `all`
appears in tags only for uncategorised records under `-loglevelalways`.

| Category | Core versions | Knots 29.x |
| --- | --- | --- |
| net, tor, mempool, http, bench, zmq, walletdb, rpc, estimatefee, addrman, selectcoins, reindex, cmpctblock, rand, prune, proxy, mempoolrej, coindb, qt, leveldb, validation, i2p, ipc | v22+ | yes |
| libevent | v22 to v31 (v32+: accepted, warns, no-op) | yes |
| lock (only with `DEBUG_LOCKCONTENTION`), blockstorage | v23+ | yes |
| util | v23 to v27 | no |
| txreconciliation, scan, txpackages | by v26 (absent in v23) | yes |
| kernel, privatebroadcast | v31+ | no |
| mining | v32+ | no |

Levels: `trace`, `debug`, `info`, `warning`, `error` (v24+). Users can only
select `info`, `debug` and `trace` via `-loglevel`. `info` and above always
log regardless of `-debug`.

## Sample lines (test fixtures)

Message bodies are illustrative. Prefixes are exact for the version and flags
shown.

```text
# Core v22/v23, defaults
2021-09-14T03:02:11Z Bitcoin Core version v22.0.0 (release build)
2021-09-14T03:05:43Z received: verack (0 bytes) peer=0
2021-09-14T03:05:44Z ERROR: AcceptBlockHeader: block 00000000... is marked invalid

# Core v24-v26, -debug=net
2023-05-01T10:00:00Z Loaded best chain: hashBestChain=00000000... height=790000
2023-05-01T10:00:01Z [net] received: verack (0 bytes) peer=0
2023-05-01T10:00:01Z [net:debug] example LogPrintLevel debug record
2023-05-01T10:00:02Z [net:warning] example LogPrintLevel warning record

# Core v27, -debug=net (error() helper still present)
2024-04-16T09:00:00Z Bitcoin Core version v27.0.0 (release build)
2024-04-16T09:00:01Z [warning] example LogWarning record
2024-04-16T09:00:01Z [error] example LogError record
2024-04-16T09:00:01Z ERROR: example error() helper record
2024-04-16T09:00:02Z [net] received: verack (0 bytes) peer=0
2024-04-16T09:00:02Z [net:trace] example trace record

# Core v27+, -loglevelalways=1 -debug=net
2024-04-16T09:00:00Z [all:info] Bitcoin Core version v27.0.0 (release build)
2024-04-16T09:00:02Z [net:debug] received: verack (0 bytes) peer=0
2024-04-16T09:00:03Z [all:warning] example LogWarning record

# Core v29, -logtimemicros -logthreadnames -logsourcelocations -debug=net
2025-04-14T08:15:30.123456Z [msghand] [net_processing.cpp:3520] [ProcessMessage] [net] received: verack (0 bytes) peer=3
2025-04-14T08:15:30.200000Z [init] [init.cpp:1100] [AppInitMain] Using 16 MiB out of 16 MiB requested for script execution cache

# Core v30 / Knots 29.x, -logsourcelocations (full signature)
2025-10-12T01:00:00Z [net_processing.cpp:3520] [void {anonymous}::PeerManagerImpl::ProcessMessage(CNode&, const std::string&, DataStream&, std::chrono::microseconds, const std::atomic<bool>&)] [net] received: verack (0 bytes) peer=3

# Core v30+ / Knots 29.x, rate limiter engaged (marker precedes timestamp)
[*] 2025-10-12T01:00:00Z [warning] Excessive logging detected from net.cpp:123 (Foo): >1048576 bytes logged during the last time window of 3600s. Suppressing logging to disk from this source location until time window resets. Console logging unaffected. Last log entry.
[*] 2025-10-12T01:00:05Z Some other unconditional record
2025-10-12T02:00:00Z [warning] Restarting logging from net.cpp:123 (Foo): 52000 bytes were dropped during the last 3600s.

# Wallet record (no tag, bracketed message prefix)
2025-10-12T01:00:00Z [default wallet] Wallet completed loading in 12ms

# mocktime (regtest/functional tests only)
2025-10-12T01:00:00Z (mocktime: 2020-01-01T00:00:00Z) [net] received: verack (0 bytes) peer=0

# -logtimestamps=0
[net] received: verack (0 bytes) peer=0

# Up to v32 / Knots 29.x: embedded newline -> unprefixed continuation line
2025-10-12T01:00:00Z [warning] RPC User __cookie__ not allowed to call method getblock
2025-10-12T00:59:59Z ERROR: forged line injected via the method name

# Up to v28: partial-line continuation (two calls, one physical line)
2024-01-01T00:00:00Z Verifying blocks... done

# master (v33): embedded newline escaped, one record per line
2026-10-01T00:00:00Z [warning] RPC User __cookie__ not allowed to call method getblock\x0a2026-...

# Restart boundary (all versions): the previous record ends in LF, then 5 LF
2025-10-12T01:00:00Z Shutdown: done




2025-10-12T01:05:00Z Bitcoin Knots version v29.4.2.knots20260508 (release build)
```

## Can logs be read without debug.log? (RPC/REST/ZMQ)

**No**, in both Core (through master 228877d) and Knots 29.4.2.

- **`logging` RPC** (`src/rpc/node.cpp`, same in Knots apart from a libevent
  hook). It takes optional `include` and `exclude` arrays of category names,
  enables or disables debug logging for them, and returns
  `{ "<category>": bool, ... }`. It is configuration only and returns no log
  content. Knots additionally toggles libevent's own logging when `libevent`
  changes.
- **`getrpcinfo`** (`src/rpc/server.cpp`, Core and Knots) returns `logpath`,
  "The complete file path to the debug log". This is useful for locating the
  file, for example when `-debuglogfile` is overridden. Knots's
  `getgeneralinfo` additionally returns `datadir`.
- **Knots-specific RPCs** in `src/rpc/node.cpp` are `getgeneralinfo` and
  `getmemoryinfo`, `logging`, `getindexinfo`, and hidden test helpers. None
  of them return log lines.
- **REST** (`src/rest.cpp`, Knots 29.4.2) serves tx, block,
  block/notxdetails, blockfilter, blockfilterheaders, chaininfo, mempool,
  headers, getutxos, spenttxouts, deploymentinfo, blockhashbyheight and fee.
  There is no logs endpoint.
- **ZMQ** (`src/zmq/zmqnotificationinterface.cpp`, Knots) publishes
  `hashblock`, `hashtx`, `rawblock`, `rawtx` and `sequence`, plus Knots's
  `hashwallettx` and `rawwallettx`. There is no log topic.
- **In-process only:** `Logger::m_print_callbacks` receives every formatted
  line. It is used by the GUI, tests and libbitcoinkernel, not exposed over
  any network interface.
- **Proposals:**
  - #34038 (open) extends the `logging` RPC with trace-level *configuration*,
    not log content.
  - #30342 and #33847 (open) and #36191 and #36197 (closed, unmerged) are
    about libbitcoinkernel per-context log delivery, which is a C API rather
    than RPC.
  - Issue #7199 ("Separate RPC events from catch-all debug.log") is closed.
  - The searches found no PR that adds a log-reading RPC, REST endpoint or
    ZMQ topic.

So the options are to read and tail `debug.log` (path via `getrpcinfo`), or
to capture stdout with `-printtoconsole` (e.g. journald). Both carry the same
line format.

## Upcoming changes to watch

- #35833 (merged, v33): newline escaping. Continuation lines disappear.
- #34730 (open): would merge `warning` and `error` into a single `alert`
  level, which is a new tag token.
- #34038 (open): replaces `-loglevel` with `-trace`. Config only; the format
  is unaffected.
