#!/bin/sh
# Entrypoint for the BitNode all-in-one node container.
#
# Starts both bitcoind and bin_console under supervisord. bitcoind runs as the
# `bitcoin` user; bin_console runs as the `bitnode` user. supervisord itself
# runs as root so it can switch to those users when spawning processes.
#
# Bitcoin daemon configuration is read from:
#   /home/bitcoin/.bitcoin/bitcoin.conf  (mount a volume at /home/bitcoin/.bitcoin)
#
# BitNode Console configuration is read from environment variables (prefix: BITNODE_)
# or a bitnode_console.conf file. See lib_settings for all available settings.
#
# Key environment variables for bin_console:
#   BITNODE_WEB_HOST          — HTTP bind address    (default: 127.0.0.1)
#   BITNODE_WEB_PORT          — HTTP port            (default: 8090)
#   BITNODE_RPC_HOST          — gRPC bind address    (default: 127.0.0.1)
#   BITNODE_RPC_PORT          — gRPC port            (default: 50051)
#   BITNODE_RPC_PASSWORD_HASH — Argon2id PHC hash    (default: empty — login will fail)
#   BITNODE_RPC_TOKEN_SECRET  — JWT signing key      (default: empty — signing will fail)
#   BITNODE_RPC_ALLOWED_IPS   — permitted CIDRs      (default: 127.0.0.1/32)

set -e

# Warn loudly if secrets are absent — bin_console will start but auth will fail
# at runtime, which is harder to diagnose than a clear startup warning.
if [ -z "${BITNODE_RPC_PASSWORD_HASH:-}" ]; then
    echo "WARNING: BITNODE_RPC_PASSWORD_HASH is not set — all login attempts will fail." >&2
fi

if [ -z "${BITNODE_RPC_TOKEN_SECRET:-}" ]; then
    echo "WARNING: BITNODE_RPC_TOKEN_SECRET is not set — JWT signing will fail." >&2
fi

# Ensure the bitcoin data directory exists and is owned correctly in case the
# mounted volume was created by a different user or process.
mkdir -p /home/bitcoin/.bitcoin
chown bitcoin:bitcoin /home/bitcoin/.bitcoin

# Replace the shell process with supervisord so it becomes PID 1 and receives
# container signals (SIGTERM, SIGINT) directly for clean shutdown of both
# bitcoind and bin_console.
exec supervisord -n -c /etc/supervisor/supervisord.conf
