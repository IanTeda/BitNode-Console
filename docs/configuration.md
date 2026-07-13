---
title: Configuration
description: Layered configuration reference for the BitNode Console backend.
---

## Overview

BitNode Console uses a layered configuration approach. Settings are loaded from
multiple sources and merged in precedence order — a value from a higher-ranked
source always wins over the same key from a lower-ranked source.

## Configuration sources

Sources are applied from lowest to highest precedence:

| # | Source | Example location |
|---|--------|-----------------|
| 1 | Built-in defaults | Hard-coded in source |
| 2 | System config file | `/etc/bitnode_console/bitnode_console.conf` |
| 3 | User config file | `~/.config/bitnode_console/bitnode_console.conf` |
| 4 | Executable directory | `<binary_dir>/bitnode_console.conf` |
| 5 | Working directory | `./bitnode_console.conf` |
| 6 | Explicit config file | `--config / -c` CLI flag |
| 7 | Environment variables | `BITNODE_FRONTEND_PORT=9100` |
| 8 | CLI flags (highest) | `--frontend-port 9100` |

A source is silently skipped if the file does not exist. An error is returned if
a file exists but cannot be parsed.

## Settings

Settings are grouped into sections by area of concern:

- **Application** — application-level authentication settings
- **Tracing** — controls logging verbosity and startup output
- **Backend** — gRPC backend server settings (host, port, auth, IP allowlist)
- **Bitcoin Daemon** — JSON-RPC credentials and systemd unit for the Bitcoin node
- **Frontend** — HTTP server that serves the React web frontend

### Config file format

Config files use INI format with one section per settings group:

```ini
[tracing]
enabled = true
level = info
show_settings_startup = false

[backend]
host = 127.0.0.1
port = 50051
password_hash = ""
token_secret = ""
allowed_ips = 127.0.0.1/32

[bitcoind]
unit_name = bitcoind.service
rpc_host = 127.0.0.1
rpc_port = 8332
rpc_user = ""
rpc_password = ""

[frontend]
host = 127.0.0.1
port = 8090
```

All sections and their keys are optional; any omitted value falls back to the
built-in default for that field.

### Settings reference

#### `[tracing]`

Controls the tracing / logging subsystem.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | `bool` | `true` | Enable or disable tracing output entirely |
| `level` | `string` | `info` | Minimum log level: `off`, `error`, `warn`, `info`, `debug`, `trace` |
| `show_settings_startup` | `bool` | `false` | Print the active settings via `tracing::info!` at startup |

#### `[backend]`

Controls the gRPC backend server.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | `string` | `127.0.0.1` | Interface to bind the gRPC listener to |
| `port` | `u16` | `50051` | Port to bind the gRPC listener to |
| `password_hash` | `string` | `""` | Argon2id PHC hash of the login password; all logins fail until set |
| `token_secret` | `string` | `""` | Secret used to sign JWT tokens; token signing fails until set |
| `allowed_ips` | `string list` | `127.0.0.1/32` | IP addresses or CIDR subnets permitted to connect |

To generate a `password_hash` value:

```sh
sudo apt install libargon2-0 -y && \
  echo -n "yourpassword" | argon2 "$(openssl rand -hex 16)" -id -e
```

Paste the resulting `$argon2id$…` PHC string into `password_hash`.

#### `[bitcoind]`

Credentials and service settings for the Bitcoin Knots/Core daemon.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `unit_name` | `string` | `bitcoind.service` | systemd unit name used to filter journal log entries |
| `rpc_host` | `string` | `127.0.0.1` | JSON-RPC host of the Bitcoin daemon |
| `rpc_port` | `u16` | `8332` | JSON-RPC port (mainnet `8332`, testnet `18332`, regtest `18443`) |
| `rpc_user` | `string` | `""` | JSON-RPC username; must match `rpcuser` in `bitcoin.conf` |
| `rpc_password` | `string` | `""` | JSON-RPC password; must match `rpcpassword` in `bitcoin.conf` |
| `cookie_file` | `path` | _(none)_ | Path to the daemon's `.cookie` file for cookie-based auth |

Either `rpc_user`/`rpc_password` or `cookie_file` must be configured for RPC
calls to succeed.

#### `[frontend]`

Controls the HTTP server that serves the React web frontend.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | `string` | `127.0.0.1` | Interface to bind the HTTP listener to |
| `port` | `u16` | `8090` | Port to bind the HTTP listener to |

### Environment variables

Any setting can be overridden with an environment variable following the pattern
`BITNODE_<SECTION>_<KEY>` (all uppercase):

```sh
# Override frontend port
BITNODE_FRONTEND_PORT=9100

# Override tracing level
BITNODE_TRACING_LEVEL=debug

# Disable tracing
BITNODE_TRACING_ENABLED=false

# Set backend allowed IPs
BITNODE_BACKEND_ALLOWED_IPS=192.168.1.0/24
```

Environment variables take precedence over all config files.

### CLI flags

CLI flags override every other source. Run `--help` on any binary for the full
list. Common flags:

```sh
bitnode-console \
  --config /etc/myapp/custom.conf \
  --frontend-port 9100 \
  --frontend-host 0.0.0.0 \
  --backend-port 50051 \
  --tracing-level debug
```
