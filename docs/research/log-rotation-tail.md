---
title: Log File rotation, truncation and live tail mechanics
description: >-
  How bitcoind changes debug.log under a reader (SIGHUP reopen, logrotate,
  -shrinkdebugfile), and which cursor and watch mechanism survives it.
issue: https://github.com/IanTeda/BitNode-Console/issues/34
parent: https://github.com/IanTeda/BitNode-Console/issues/32
status: research
date: 2026-10-10
sources:
  - bitcoin/bitcoin master @ 228877d7836a55c1d6fde90b0b7ba2d4c90248ab
  - bitcoinknots/bitcoin 29.x-knots @ 58398baf33e588779685ead478e6397bb28ed3d6
  - notify-rs/notify main @ e455a21ce5cab30902f203241e0f8de70ccd163a
  - logrotate/logrotate logrotate.8.in (default branch)
---

## Question

The console will read bitcoind's `debug.log` directly and serve paged reads
plus a live tail on Linux, macOS, BSD and Windows. How does bitcoind change
that file underneath a reader, and what cursor and watch mechanism survives
those changes?

## Short answer

- bitcoind only ever **appends** (`fopen(path, "a")`, unbuffered). It changes
  the file in exactly two ways: it **reopens by path** after `SIGHUP`
  (non-Windows only), and it **truncates in place** at startup when
  `-shrinkdebugfile` is on, keeping the last 10,000,000 bytes once the file
  exceeds 11,000,000 bytes. Everything else (move/create, copytruncate) is
  done by an external tool such as logrotate.
- There is no maximum file size. The only built-in limit is a disk-filling
  **rate limiter**: 1 MiB per source location per hour (`-logratelimit`,
  default on).
- A byte offset alone is not a safe cursor. Make the cursor
  `(file identity, offset, fingerprint of the bytes just before offset)`.
  File identity catches move/create; `size < offset` catches most truncation;
  the fingerprint catches truncation followed by regrowth past the offset,
  including startup shrink (same inode, new content).
- For the live tail, a **periodic `stat` poll is sufficient and fully
  portable**; `notify` is an optional latency optimisation that should only
  ever trigger the same re-check, never be trusted on its own. Watch the
  **parent directory**, not the file, if `notify` is used.

## How bitcoind writes the Log File

### Opening and appending

`BCLog::Logger::StartLogging` opens the file in append mode and disables
stdio buffering, then writes five newlines to separate this run from the
previous one (`src/logging.cpp`, `StartLogging`):

```cpp
m_fileout = fsbridge::fopen(m_file_path, "a");
...
setbuf(m_fileout, nullptr); // unbuffered
// Add newlines to the logfile to distinguish this execution from the
// last one.
FileWriteStr("\n\n\n\n\n", m_fileout);
```

Consequences for a reader:

- `"a"` means `O_APPEND` on POSIX, so every write lands at the current end of
  file even if someone else truncated it. This is why logrotate
  `copytruncate` works cleanly with bitcoind: after truncation, writes resume
  at offset 0 rather than leaving a sparse hole.
- Unbuffered means each log line is written promptly, but a reader can still
  observe a partially written line. Only consume up to the last `\n`.
- The `"\n\n\n\n\n"` separator is a cheap, reliable marker of a daemon
  restart inside the file.
- `fsbridge::fopen` is a thin wrapper over `::fopen` on all platforms
  (`src/util/fs.cpp`), so there are no special share or lock flags.

The path comes from `-debuglogfile` (default `debug.log`, relative paths are
resolved against the network-specific datadir); `-nodebuglogfile` disables
file logging entirely (`src/init/common.cpp`, `AddLoggingArgs` /
`SetLoggingOptions`). The console must read the same effective path; see the
configuration research for resolving it.

Before `StartLogging`, up to `DEFAULT_MAX_LOG_BUFFER` (1,000,000 bytes) of
early messages are held in memory and flushed afterwards
(`src/logging.h`).

### SIGHUP: reopen by path

`init.cpp` registers `HandleSIGHUP` only under `#ifndef WIN32`; the handler
just sets a flag:

```cpp
static void HandleSIGHUP(int)
{
    LogInstance().m_reopen_file = true;
}
```

The reopen happens lazily on the **next log write**
(`src/logging.cpp`, `LogPrintStr_`):

```cpp
if (m_reopen_file) {
    m_reopen_file = false;
    FILE* new_fileout = fsbridge::fopen(m_file_path, "a");
    if (new_fileout) {
        setbuf(new_fileout, nullptr); // unbuffered
        fclose(m_fileout);
        m_fileout = new_fileout;
    }
}
```

Points to note:

- The reopen is by **path**, so after a move/rename rotation bitcoind starts
  writing to whatever is now at `debug.log` (creating it if absent, because
  of `"a"`).
- Between the rename and the next log line, bitcoind keeps writing to the
  **old inode** (now e.g. `debug.log.1`). A reader that switches files as
  soon as it sees the rename can miss those trailing lines; it must drain the
  old handle first.
- If the reopen fails, bitcoind silently keeps the old handle (fixed in
  0.18, #13159, "Don't close old debug log file handle prematurely").
- SIGHUP reopen has existed since 0.7.0 (release notes: "Reopen debug.log
  upon SIGHUP").
- **Windows has no SIGHUP path**, so the only in-daemon change on Windows is
  the startup shrink.

Knots `29.x-knots` has the identical open, reopen and shrink code.

### -shrinkdebugfile at startup

`StartLogging(const ArgsManager&)` in `src/init/common.cpp` calls
`ShrinkDebugFile()` **before** opening the file for appending, when
`-shrinkdebugfile` is true. Its default comes from
`DefaultShrinkDebugFile()`, which is `m_categories == BCLog::NONE`, i.e.
**on unless any `-debug=<category>` is set** (help text: "default: 1 when
no -debug").

`ShrinkDebugFile` (`src/logging.cpp`):

```cpp
constexpr size_t RECENT_DEBUG_HISTORY_SIZE = 10 * 1000000;
...
if (file && log_size > 11 * (RECENT_DEBUG_HISTORY_SIZE / 10))
{
    std::vector<char> vch(RECENT_DEBUG_HISTORY_SIZE, 0);
    fseek(file, -((long)vch.size()), SEEK_END) ...
    int nBytes = fread(vch.data(), 1, vch.size(), file);
    fclose(file);
    file = fsbridge::fopen(m_file_path, "w");
    if (file) { fwrite(vch.data(), 1, nBytes, file); fclose(file); }
}
```

So:

- Trigger: file size **> 11,000,000 bytes** at daemon start.
- Result: the last **10,000,000 bytes** are kept (decimal MB, not MiB).
- Mechanism: `fopen("w")` is truncate-in-place, so the **inode / file
  identity does not change**. Only size and content do.
- The cut is at an arbitrary byte, so the first line of the shrunk file is
  usually a partial line. Skip to the first `\n` when reading from offset 0.
- It only runs at startup. A long-running daemon's file grows without bound
  unless something external rotates it.

### Size limits and rate limiting

There is no maximum log file size in bitcoind. The only limit on growth is
the rate limiter (`src/logging.h`):

```cpp
inline constexpr uint64_t RATELIMIT_MAX_BYTES{1_MiB};
inline constexpr auto RATELIMIT_WINDOW{1h};
inline constexpr bool DEFAULT_LOGRATELIMIT{true};
```

It is per **source location**, applies to unconditional logging only, and
when tripped bitcoind writes an "Excessive logging detected from ..."
message, then suppresses that location's **file** output until the window
resets (console output is not suppressed). It is controlled by
`-logratelimit` (`src/init/common.cpp`, `src/init.cpp`). Knots `29.x-knots`
also has `LogRateLimiter`. Practical bound: debug categories can still
produce large volumes, since they are not rate limited.

## How external rotation changes the file

bitcoind ships no logrotate configuration (no `logrotate` hits in
bitcoin/bitcoin outside `init.cpp`'s SIGHUP comment), so operators bring
their own. From the logrotate man page (`logrotate.8.in`):

| Mode | What happens | Needs SIGHUP? | Reader sees |
|------|--------------|---------------|-------------|
| default / `create` | `debug.log` renamed to `debug.log.1`; with `create`, a new empty `debug.log` is created "immediately after rotation (before the postrotate script is run)" | Yes, in `postrotate` | Path now resolves to a **different file identity**; old inode keeps receiving lines until bitcoind's next write after SIGHUP |
| `copytruncate` | "Truncate the original log file to zero size in place after creating a copy"; "a very small time slice between copying the file and truncating it, so some logging data might be lost" | No | **Same identity**, size drops to 0 then regrows; lines in the copy/truncate gap are lost to everyone |
| `copy` | Copy only, original untouched | No | No change |
| `delaycompress` | Defers compression of the previous file one cycle, for programs that "might continue writing to the previous log file for some time" | n/a | Old file stays plain text one cycle longer, so it can still be drained |

Without SIGHUP and without `create`, a rename leaves bitcoind writing to
`debug.log.1` forever and `debug.log` absent. The reader must cope with the
path being missing.

## Detecting rotation and truncation

Four signals, cheapest first:

1. **Path missing** (`ENOENT`): rotated without `create`, or not yet
   recreated. Keep draining the open handle; retry the path.
2. **Identity changed**: the path now names a different file than the open
   handle. Unix: `std::os::unix::fs::MetadataExt::{dev, ino}` on
   `fs::metadata(path)` vs `file.metadata()`. Windows: volume serial number
   plus file index from `GetFileInformationByHandle` (the `std` accessors for
   these are unstable; the `same-file` crate wraps both platforms).
   Indicates move/create rotation.
3. **Size went backwards**: `metadata.len() < offset`. Indicates
   copytruncate or startup shrink, as long as the file has not regrown past
   the offset before the check.
4. **Fingerprint mismatch**: the bytes ending at `offset` no longer match
   what was recorded. This is the only signal that catches:
   - startup shrink, where the file is 10,000,000 bytes afterwards and any
     cursor below that is still "in range" but points at different content;
   - copytruncate followed by regrowth past the old offset between two
     checks (busy node, slow poll).

`mtime` is not useful as a discriminator: every append changes it.

## Cursor design

Recommendation: an opaque, serialised cursor containing

- `file_id`: `(dev, ino)` on Unix, `(volume_serial, file_index)` on Windows;
- `offset`: byte offset of the next unread byte, always at a line boundary
  (just after a `\n`);
- `fingerprint`: a hash of the up to N bytes (for example 64 to 256)
  immediately before `offset`, or the line ending at `offset`.

Validation on use: open the path, compare identity, check
`len >= offset`, read the N bytes before `offset` and compare the hash. Any
mismatch means the cursor is **stale**; return a distinct error (for
example `FAILED_PRECONDITION` / "cursor stale") so the frontend reloads
from the tail instead of showing spliced or duplicated lines.

Paged reads backwards ("older") work by seeking back from a cursor in fixed
chunks and splitting on `\n`; because the file is append-only between
rotations, any offset before a valid cursor stays valid while the identity
and fingerprint hold. Rotated files (`debug.log.1`, compressed archives) are
out of scope for paging unless explicitly added.

What the cursor cannot promise:

- Lines lost in logrotate's copy/truncate gap are unrecoverable.
- After a move/create rotation, older pages are in `debug.log.1`, not the
  live file.

## Live tail: notify vs polling

### notify crate

Latest stable is `notify` 8.2.0 (2025-08-03); 9.0 is in release candidates
(`9.0.0-rc.5`, 2026-08-30). Backends (`notify/src/lib.rs`):

| Platform | Backend |
|----------|---------|
| Linux, Android | inotify |
| macOS | FSEvents (default) or kqueue (`macos_kqueue` feature) |
| FreeBSD | inotify on 14.5+ native builds (9.x), kqueue otherwise |
| Other BSDs | kqueue |
| Windows | ReadDirectoryChangesW |
| Anywhere | `PollWatcher` |

"Known Problems" in the crate docs that matter here:

- **Network filesystems** "like NFS may not emit any events"; workaround is
  `PollWatcher`.
- **Docker on macOS (M1)** fails with "Function not implemented"; use
  `PollWatcher`. Relevant to the Docker images in `docker/`.
- **macOS FSEvents and unowned files**: "some events cannot be observed
  easily when trying to follow files that do not belong to you". The console
  will commonly run as a different user from bitcoind, so this bites
  directly.
- **inotify limits** (`fs.inotify.max_user_watches`/`max_user_instances`);
  `PollWatcher` is not subject to them.
- **Renamed or removed watched paths**: "behaviour may be unexpected ... one
  may non-recursively watch the _parent_ directory as well and manage
  related events" (`Watcher::watch` docs). For logrotate this is the normal
  case, so watch the directory containing `debug.log`.

`PollWatcher` defaults to a **30 s** interval (`Config::default`), compares
metadata only unless `compare_contents` is set (which hashes every file
every poll). For a single file, a hand-rolled `stat` loop is simpler and
gives the same result.

### Recommendation

1. Build the tail around one function, `check()`, that re-stats the path and
   the open handle, applies the four detection signals above, drains any
   remaining bytes from the old handle, switches files if needed, and emits
   complete new lines.
2. Drive `check()` from a `tokio::time::interval` (for example 250 ms to
   1 s). One `stat` per interval per tail stream (or one shared reader fanned
   out to subscribers) is negligible cost and works identically on every
   platform, in Docker, and on network mounts.
3. Optionally add `notify` (`RecommendedWatcher`, non-recursive, on the
   parent directory) purely as a wake-up hint that calls `check()` early.
   Never infer rotation from event kinds alone; events are lossy and differ
   per backend. Keep the interval poll as the safety net.
4. Treat `PollWatcher` as unnecessary given (2).

This keeps one code path for correctness and makes `notify` a removable
latency improvement rather than a portability risk.

## Platform notes

- **Linux / BSD / macOS**: an open handle survives rename and unlink, so
  "drain old handle, then switch" works for move/create rotation.
- **Windows**: no SIGHUP, so no reopen; only startup shrink changes the
  file (same file, truncate-in-place). Rust's `File::open` on Windows uses a
  default share mode of read, write and delete, so the console's open handle
  does not block bitcoind's `fopen("w")` during shrink. Whether an external
  tool can rename `debug.log` while bitcoind holds it open depends on the C
  runtime's share mode for `fopen` and was **not verified** here.

## Open questions

- Should paging extend into rotated files (`debug.log.1`, `.gz`)? Needs a
  product decision; it complicates the cursor (needs a file generation, not
  just identity).
- The effective log path (`-debuglogfile`, datadir, network subdirectory,
  `-nodebuglogfile`) must be resolved consistently with the configuration
  research ticket.
- Should the console detect daemon restarts (the `\n\n\n\n\n` separator, or
  RPC `uptime`) to show a visual break in the tail?

## Sources

- bitcoin/bitcoin `src/logging.cpp` (`StartLogging`, `LogPrintStr_`,
  `ShrinkDebugFile`, `LogRateLimiter`), `src/logging.h`
  (`DEFAULT_MAX_LOG_BUFFER`, `RATELIMIT_*`, `m_reopen_file`), `src/init.cpp`
  (`HandleSIGHUP`, signal registration under `#ifndef WIN32`, rate limiter
  setup), `src/init/common.cpp` (`-debuglogfile`, `-shrinkdebugfile`,
  `-logratelimit`, `StartLogging`), `src/util/fs.cpp` (`fsbridge::fopen`),
  `doc/release-notes/release-notes-0.7.0.md`,
  `doc/release-notes/release-notes-0.18.0.md` (#13159). Commit
  `228877d7836a55c1d6fde90b0b7ba2d4c90248ab`.
- bitcoinknots/bitcoin `src/logging.cpp` on `29.x-knots`, commit
  `58398baf33e588779685ead478e6397bb28ed3d6`.
- notify-rs/notify `notify/src/lib.rs` (backends, Known Problems,
  `Watcher::watch`), `notify/src/config.rs` (`Config::default`,
  `with_poll_interval`, `with_compare_contents`), GitHub releases list.
  Commit `e455a21ce5cab30902f203241e0f8de70ccd163a`.
- logrotate/logrotate `logrotate.8.in` (`create`, `copy`, `copytruncate`,
  `delaycompress`).
- Rust `std` docs: `std::os::unix::fs::MetadataExt`,
  `std::os::windows::fs::OpenOptionsExt::share_mode` (cited from knowledge,
  not re-fetched in this session).
