---
title: Context
description: Domain glossary for BitNode Console.
---

## Glossary

### Logs

- **Log**: the Bitcoin daemon's record of what it is doing, as viewed in the
  console.
- **Log File**: the Bitcoin daemon's own log file (`debug.log`). It is the only
  place the console reads the Log from; the systemd journal is not used.
- **Log Entry**: one record from the Log File.
- **Level**: how serious a Log Entry is, using bitcoind's own levels:
  `trace`, `debug`, `info`, `warning`, `error`. A Log Entry without a level
  tag (older daemons) is `info`.
  _Avoid_: Priority, Severity.
- **Category**: the bitcoind subsystem that wrote a Log Entry (for example
  `net`, `mempool`, `validation`), shown in the line as `[net]`.
- **Log Unavailable**: the console cannot show the Log, for one of four
  reasons:
  - **Disabled**: bitcoind is not writing a Log File.
  - **Not Found**: the Log File's location is known but no file is there.
  - **Unreadable**: the Log File exists but the console may not read it.
  - **Unlocatable**: the console cannot work out where the Log File is.
