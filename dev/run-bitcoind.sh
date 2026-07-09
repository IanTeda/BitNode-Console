#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$SCRIPT_DIR/data"

bitcoind \
  -conf="$SCRIPT_DIR/bitcoin.conf" \
  -datadir="$SCRIPT_DIR/data" \
  2>&1 | systemd-cat -t bitcoind
