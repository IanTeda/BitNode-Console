#!/bin/sh
# Runs before nginx starts. Replaces placeholder strings baked into the build
# with real values from environment variables, then hands off to nginx.
set -e

# Apply defaults for any variables not set by the caller.
: "${BITNODE_PORT:=8080}"
: "${BITNODE_RPC_BASE_URL:=http://127.0.0.1:50051}"
: "${BITNODE_RPC_DEADLINE_MS:=30000}"

# Inject the port into the nginx config. The placeholder is replaced here
# rather than at build time so a single image can serve on any port.
sed -i "s|__BITNODE_PORT__|${BITNODE_PORT}|g" \
  /etc/nginx/conf.d/default.conf

# Inject RPC settings into the built JS assets. Vite bakes these as literal
# placeholder strings at build time; we replace them here so one image works
# across environments without a rebuild.
find /usr/share/nginx/html/assets -name '*.js' | while read -r f; do
  sed -i \
    -e "s|__BITNODE_RPC_BASE_URL__|${BITNODE_RPC_BASE_URL}|g" \
    -e "s|__BITNODE_RPC_DEADLINE_MS__|${BITNODE_RPC_DEADLINE_MS}|g" \
    "$f"
done

# Replace the shell process with nginx so the container's PID 1 is nginx
# itself — signals (SIGTERM, SIGHUP) reach it directly.
exec nginx -g 'daemon off;'
