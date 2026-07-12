#!/bin/sh
# Entrypoint for the bin_backend container.
#
# bin_backend reads all configuration at startup via lib_settings, which merges
# sources in priority order (lowest to highest):
#   1. Built-in defaults
#   2. System config file  (/etc/bitnode_console/bitnode_console.conf)
#   3. User config file    (~/.config/bitnode_console/bitnode_console.conf)
#   4. Executable dir      (<exe-dir>/bitnode_console.conf)
#   5. Working dir         (./bitnode_console.conf)
#   6. Explicit config     (--config <path>)
#   7. Environment vars    (prefix: BITNODE_)
#
# Key environment variables (see lib_settings/src/rpc.rs for all fields):
#   BITNODE_RPC_HOST          — bind address       (default: 127.0.0.1)
#   BITNODE_RPC_PORT          — gRPC port          (default: 50051)
#   BITNODE_RPC_PASSWORD_HASH — Argon2id PHC hash  (default: empty — login will fail)
#   BITNODE_RPC_TOKEN_SECRET  — JWT signing key    (default: empty — signing will fail)
#   BITNODE_RPC_ALLOWED_IPS   — permitted CIDRs    (default: 127.0.0.1/32)
#
# In Docker, BITNODE_RPC_HOST should be set to 0.0.0.0 so the server
# is reachable from outside the container.

set -e

# Warn loudly if secrets are absent — the binary will start but auth will fail
# at runtime, which is harder to diagnose than a clear startup warning.
if [ -z "${BITNODE_RPC_PASSWORD_HASH:-}" ]; then
    echo "WARNING: BITNODE_RPC_PASSWORD_HASH is not set — all login attempts will fail." >&2
fi

if [ -z "${BITNODE_RPC_TOKEN_SECRET:-}" ]; then
    echo "WARNING: BITNODE_RPC_TOKEN_SECRET is not set — JWT signing will fail." >&2
fi

# Replace the shell process with bin_backend so it becomes PID 1 and receives
# container signals (SIGTERM, SIGINT) directly for clean shutdown.
exec bin_backend
