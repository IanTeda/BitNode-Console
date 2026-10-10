---
title: How bitcoind resolves its Log File location
description: >-
  Research notes on how Bitcoin Core and Bitcoin Knots derive the debug.log
  path from configuration, how file logging is disabled, and what RPC can
  report about it.
issue: https://github.com/IanTeda/BitNode-Console/issues/35
date: 2026-10-10
sources:
  - bitcoin/bitcoin master @ 228877d7836a55c1d6fde90b0b7ba2d4c90248ab
  - bitcoinknots/bitcoin tag v29.4.2.knots20260508
status: research
---

## Answer in brief

The Log File path is computed once at startup, in `SetLoggingOptions()`:

```cpp
// src/init/common.cpp (Core master; identical in Knots 29.x)
LogInstance().m_print_to_file = !args.IsArgNegated("-debuglogfile");
LogInstance().m_file_path = AbsPathForConfigVal(
    args, args.GetPathArg("-debuglogfile", DEFAULT_DEBUGLOGFILE));
```

So, in words:

1. Take the effective `debuglogfile` setting (default `debug.log`,
   `DEFAULT_DEBUGLOGFILE` in `src/logging.cpp`).
2. If it is absolute, use it as is.
3. If it is relative, join it to the **network-specific** data directory
   (`<datadir>/<chain subdir>/`), where `<datadir>` is the final datadir
   after the config file has been read.
4. File logging is off only when the setting is **negated**
   (`-nodebuglogfile` / `nodebuglogfile=1`). `debuglogfile=0` does **not**
   disable it — it writes to a file literally named `0`.

A running daemon reports the resolved path via `getrpcinfo` → `logpath`
(both Core and Knots). When file logging is disabled, `logpath` is an empty
string. Knots additionally exposes the network datadir via
`getgeneralinfo` → `datadir`. Core has no RPC returning the datadir.

## Step 1: the base datadir

`ArgsManager::GetDataDir()` (`src/common/args.cpp`) uses `-datadir` if set
(made absolute with `fs::absolute`, so relative to the daemon's working
directory; it must already exist), otherwise `GetDefaultDataDir()`:

| OS | Default datadir |
| --- | --- |
| Linux / Unix-like | `$HOME/.bitcoin` (`/.bitcoin` if `HOME` unset/empty) |
| macOS | `$HOME/Library/Application Support/Bitcoin` |
| Windows | `%LOCALAPPDATA%\Bitcoin`, unless legacy `%APPDATA%\Bitcoin` exists |

Source: `GetDefaultDataDir()` in `src/common/args.cpp`; table in
`doc/files.md`. Knots 29.x has the same function verbatim.

`datadir=` may also appear in `bitcoin.conf`. `ReadConfigFiles()`
(`src/common/config.cpp`) calls `ClearPathCache()` after parsing so the
config-file datadir takes effect; `src/common/init.cpp` (`InitConfig`)
documents this and notes a `bitcoin.conf` inside the new datadir is **not**
then read (it errors unless `allowignoredconf=1`).

Packaging matters for us: Core's `contrib/init/bitcoind.service` runs
`bitcoind -conf=/etc/bitcoin/bitcoin.conf -datadir=/var/lib/bitcoind`, so on
a systemd host the log is typically `/var/lib/bitcoind/debug.log` and the
datadir comes from the **command line**, not the conf file.

## Step 2: the chain subdirectory

`GetDataDir(net_specific=true)` appends `BaseParams().DataDir()`
(`src/chainparamsbase.cpp`):

| Chain | Selected by | Subdir | Conf section |
| --- | --- | --- | --- |
| main | default, `chain=main` | *(none)* | `[main]` |
| testnet3 | `-testnet`, `chain=test` | `testnet3/` | `[test]` |
| testnet4 | `-testnet4`, `chain=testnet4` | `testnet4/` | `[testnet4]` |
| signet | `-signet`, `chain=signet` | `signet/` | `[signet]` |
| regtest | `-regtest`, `chain=regtest` | `regtest/` | `[regtest]` |

Section names come from `ChainTypeToString()` (`src/util/chaintype.cpp`).
Note the testnet3 **section** is `[test]` but the **directory** is
`testnet3/`.

Core master difference: a custom signet (`-signetchallenge` other than the
default) uses `signet_<message-start-hex>/` (`GetSignetDataDir()`,
`doc/files.md`). Knots 29.x always uses `signet/`.

The chain is chosen by `GetChainArg()` from command line and config (only
one of `-regtest/-signet/-testnet/-testnet4/-chain` may be set), and
`ReadConfigFiles()` reads `[chain]` sections and chain-specific
`includeconf` before the chain is final.

## Step 3: relative vs absolute `debuglogfile`

`AbsPathForConfigVal(args, path, net_specific = true)`
(`src/common/config.cpp`, default arg in `src/common/args.h`):

- absolute or empty path → returned unchanged;
- relative path → `AbsPathJoin(GetDataDirNet(), path)`.

`GetPathArg()` also `lexically_normal()`s the value and strips a trailing
slash. Examples, regtest, datadir `/var/lib/bitcoind`:

| Setting | Resulting Log File |
| --- | --- |
| *(unset)* | `/var/lib/bitcoind/regtest/debug.log` |
| `debuglogfile=` (empty) | `/var/lib/bitcoind/regtest/debug.log` |
| `debuglogfile=logs/node.log` | `/var/lib/bitcoind/regtest/logs/node.log` |
| `debuglogfile=/var/log/bitcoin/debug.log` | `/var/log/bitcoin/debug.log` |
| `debuglogfile=0` | `/var/lib/bitcoind/regtest/0` (**not** disabled) |
| `nodebuglogfile=1` / `-nodebuglogfile` | disabled; path empty |

The help text agrees: "Relative paths will be prefixed by a net-specific
datadir location. Pass -nodebuglogfile to disable writing the log to a
file." (`src/init/common.cpp`, Core and Knots).

## Step 4: detecting that logging to file is disabled

`InterpretKey()` (`src/common/args.cpp`) treats a `no` prefix as negation
and `InterpretValue()` stores a negated key as boolean `false`.
`IsArgNegated()` is `GetSetting(arg).isFalse()`. Consequences:

- `-nodebuglogfile`, `-nodebuglogfile=1`, `nodebuglogfile=1` (conf) →
  disabled.
- `debuglogfile=0` → value is the string `"0"`, not `false`; file named `0`.
  The source comment in `InterpretKey()` uses `-nodebuglogfile` as the
  canonical example of negating a non-boolean option.
- `nodebuglogfile=0` is a double negative → `true` (with a warning) →
  `SettingToString(true)` is `"1"`, so it logs to a file named `1`. Edge
  case, but a config parser that treats it as "enabled, debug.log" is wrong.
- Disabled also forces `GetPathArg()` to return an empty path (it returns
  `{}` when the setting `isFalse()`), so `m_file_path` is empty.

When enabled, `StartLogging()` opens the file in append mode and fails init
with "Could not open debug log file …" if it cannot. Separately,
`-printtoconsole` defaults to on unless `-daemon`, which is why a
foreground `bitcoind` under systemd also lands in the journal.

## Step 5: precedence between sources

`MergeSettings()` in `src/common/settings.cpp` (Core):

> Forced config > command line > read-write settings file > config file
> network-specific section > config file default section

Rules that affect `debuglogfile`:

- **Command line beats conf.** `-nodebuglogfile` on the command line
  disables file logging even if the conf sets `debuglogfile=…`, and vice
  versa.
- **Within the conf file the first assignment wins**, not the last
  (backward-compatible "weird behaviour" comment in `GetSetting()`).
- **Network section beats default section.** `debuglogfile` is not a
  network-only option, so a top-level `debuglogfile=` applies to every
  chain unless overridden by `[regtest]` etc.
- **`settings.json`** (`-settings`, default `settings.json` in the
  network datadir; `-nosettings` disables) is the "read-write settings
  file". It is loaded in `InitConfig()` before `SetLoggingOptions()`, so a
  `debuglogfile` key there would apply, but bitcoind itself never writes
  one (it is used for GUI/wallet settings). Reading it is optional for us.
- **`includeconf`** is only allowed in the main config (command line may
  only pass `-noincludeconf`); paths are relative to the **base** datadir
  (`net_specific=false`); includes from included files are ignored with a
  warning; `[chain]` section includes are read before default-section
  includes. `conf=` inside a config file is a parse error.
- **`-conf`** relative paths are resolved against the base datadir taken
  from the command line (`ReadConfigFiles()`); `-noconf` reads no file.

## Bitcoin Knots differences (29.x)

- **`bitcoin_rw.conf`**: Knots adds `-confrw=<file>` (default
  `bitcoin_rw.conf`, relative to the **network** datadir) read after the
  main config (`src/common/config.cpp`). Its precedence sits between the
  command line and `settings.json`:
  command line > `bitcoin_rw.conf` > `settings.json` > `[chain]` > default
  (`src/common/settings.cpp`, `Source::CONFIG_FILE_RW`). If we model
  effective settings for Knots, this file must be included.
- **`getgeneralinfo` RPC** returns `datadir` (network-specific, i.e.
  `GetDataDirNet()`), `blocksdir`, `clientversion`, `useragent`,
  `startuptime` (`src/rpc/node.cpp`). Core has no equivalent.
- Custom signets keep `signet/` (no `signet_<hex>/` suffix).
- `SetLoggingOptions()`, `AbsPathForConfigVal()`, `GetDefaultDataDir()` and
  `getrpcinfo.logpath` are otherwise identical.

## What a running daemon can tell us over RPC

| RPC | Core | Knots | Useful field |
| --- | --- | --- | --- |
| `getrpcinfo` | yes | yes | `logpath`: "The complete file path to the debug log" |
| `getgeneralinfo` | no | yes | `datadir`, `blocksdir` |
| `logging` | yes | yes | only category on/off map; no path |
| `getblockchaininfo` | yes | yes | `chain` (gives the subdir); no path |

`getrpcinfo` reads `LogInstance().m_file_path` directly
(`src/rpc/server.cpp`), so it is the already-resolved absolute path, after
all precedence rules. Because a negated `debuglogfile` yields an empty
`m_file_path`, **`logpath == ""` means logging to file is disabled**.
This is the authoritative answer whenever the daemon is up and RPC is
reachable; config parsing is only needed when it is not.

Caveat: `logpath` is a path on the **daemon's** host/filesystem namespace
(e.g. inside a container), which may differ from what BitNode Console sees.

## Implications for BitNode Console

- Prefer `getrpcinfo.logpath` when RPC is available; empty string → show
  "file logging disabled".
- Fallback resolver order: CLI flags of the running process (e.g. from
  the systemd unit / `/proc/<pid>/cmdline`) → [Knots: `bitcoin_rw.conf`]
  → `settings.json` → `[chain]` section → default section, with
  first-assignment-wins inside the conf and `includeconf` expanded.
- Treat only `no`-prefixed keys as disabling; `debuglogfile=0` is a path.
- Join relative paths onto `<datadir>/<chain subdir>/`, using the
  post-conf datadir.

## Sources

- `src/init/common.cpp` — `-debuglogfile` help, `SetLoggingOptions()`,
  `StartLogging()`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/init/common.cpp>
- `src/logging.cpp` — `DEFAULT_DEBUGLOGFILE = "debug.log"`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/logging.cpp>
- `src/common/args.cpp` — `InterpretKey`, `InterpretValue`, `GetPathArg_`,
  `GetDataDir`, `GetDefaultDataDir`, `IsArgNegated`, `GetChainArg`,
  `SettingToString`, `GetSettingsPath`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/common/args.cpp>
- `src/common/config.cpp` — `ReadConfigFiles`, `IsConfSupported`,
  `AbsPathForConfigVal`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/common/config.cpp>
- `src/common/init.cpp` — `InitConfig`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/common/init.cpp>
- `src/common/settings.cpp` — `MergeSettings`, `GetSetting`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/common/settings.cpp>
- `src/chainparamsbase.cpp`, `src/util/chaintype.cpp` — chain subdirs and
  section names.
- `src/rpc/server.cpp` — `getrpcinfo` / `logpath`:
  <https://github.com/bitcoin/bitcoin/blob/master/src/rpc/server.cpp>
- `doc/files.md` — datadir and file locations:
  <https://github.com/bitcoin/bitcoin/blob/master/doc/files.md>
- `contrib/init/bitcoind.service` — packaged systemd unit.
- Knots (tag `v29.4.2.knots20260508`): `src/common/config.cpp`
  (`-confrw`), `src/common/settings.cpp` (`CONFIG_FILE_RW`),
  `src/rpc/node.cpp` (`getgeneralinfo`), `src/init.cpp` (`-confrw` help),
  `doc/files.md`:
  <https://github.com/bitcoinknots/bitcoin/tree/v29.4.2.knots20260508>
