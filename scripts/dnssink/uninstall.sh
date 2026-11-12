#!/usr/bin/env bash
# Stop the sinkhole, drop payload + overlay rc, restore Mac Wi-Fi DNS to DHCP.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=host.sh
source "$ROOT/host.sh"

adb_hy root >/dev/null
adb_hy wait-for-device
adb_hy remount >/dev/null

pid="$(adb_hy shell "cat $REMOTE/dnsmasq.pid 2>/dev/null" | tr -d '\r' || true)"
if [[ -n "${pid:-}" ]]; then
  adb_hy shell "kill $pid" >/dev/null || true
fi
adb_hy shell "echo dnssink >/sys/power/wake_unlock" >/dev/null || true
adb_hy shell "rm -rf $REMOTE" >/dev/null
adb_hy shell "rm -f /system/etc/init/dnssink.rc" >/dev/null
networksetup -setdnsservers Wi-Fi Empty
echo "dnssink removido; DNS do Mac voltou ao DHCP"
