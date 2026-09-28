#!/bin/sh
set -eu

# The public issuer uses .localhost, which HTTP clients may force to loopback.
# Resolve Caddy on the Compose network while retaining the public issuer URL.
issuer_address="$(getent ahostsv4 caddy | awk 'NR == 1 { print $1 }')"
if [ -z "$issuer_address" ]; then
    echo "Could not resolve the Compose Caddy service for OIDC" >&2
    exit 1
fi
export MAHFEL_OIDC__ISSUER_RESOLVE_IP="$issuer_address"

exec /usr/local/bin/mahfel-api "$@"
