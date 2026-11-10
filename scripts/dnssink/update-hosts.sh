#!/usr/bin/env bash
# Download StevenBlack, filter, push to the HY300, SIGHUP dnsmasq if it is up.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=host.sh
source "$ROOT/host.sh"

RAW="$(mktemp)"
OUT="$(mktemp)"
trap 'rm -f "$RAW" "$OUT"' EXIT

curl -fsSL "$STEVENBLACK_URL" -o "$RAW"
node "$ROOT/build-hosts.mjs" "$RAW" "$ROOT/whitelist.txt" >"$OUT"

adb_hy root >/dev/null
adb_hy wait-for-device
adb_hy shell mkdir -p "$REMOTE"
adb_hy push "$OUT" "$REMOTE/hosts" >/dev/null
adb_hy push "$ROOT/whitelist.txt" "$REMOTE/whitelist.txt" >/dev/null

pid="$(adb_hy shell "cat $REMOTE/dnsmasq.pid 2>/dev/null" | tr -d '\r' || true)"
if [[ -n "${pid:-}" ]]; then
  adb_hy shell "kill -HUP $pid" >/dev/null || true
fi

bytes="$(wc -c <"$OUT" | tr -d ' ')"
echo "hosts $bytes bytes em $REMOTE/hosts"
