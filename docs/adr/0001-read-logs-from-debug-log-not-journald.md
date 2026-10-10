---
title: Read logs from debug.log, not journald
status: accepted
date: 2026-10-10
---

## Context

The feasibility stage proved log viewing against the systemd journal
(`lib_journals`, linking libsystemd). That ties the console to systemd hosts:
the crate does not even build where libsystemd is absent (macOS, BSD,
Alpine/musl, Windows). Bitcoin Knots/Core always writes its own log file,
`debug.log`, on every platform, and the product requirements (FR.7–FR.10)
already describe a `debug.log` viewer.

## Decision

Replace `lib_journals` with `lib_logs`, which reads only bitcoind's Log File.
Journald support is removed entirely, not kept as an optional source or cargo
feature. The gRPC contract is replaced with a log-file-shaped `LogsService`
rather than adapted from the journal one.

## Consequences

- The console builds and reads logs on any OS where it can see the Log File.
- Daemons configured with `-nodebuglogfile` (journal or console output only)
  cannot be viewed; the console must detect and report this.
- The console must work out where the Log File is from bitcoind's own
  configuration, and handle rotation and startup truncation itself.
