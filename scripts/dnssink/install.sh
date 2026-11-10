#!/usr/bin/env bash
# Push payload + init overlay, start dnsmasq now (init picks it up on next boot).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=host.sh
source "$ROOT/host.sh"

adb_hy root >/dev/null
adb_hy wait-for-device
adb_hy remount >/dev/null
adb_hy shell mkdir -p "$REMOTE"

"$ROOT/update-hosts.sh"

adb_hy push "$ROOT/start.sh" "$REMOTE/start.sh" >/dev/null
adb_hy shell "chmod 755 $REMOTE/start.sh"
adb_hy push "$ROOT/dnssink.rc" /system/etc/init/dnssink.rc >/dev/null
adb_hy shell "chmod 644 /system/etc/init/dnssink.rc"

# Init only parses new .rc at boot. Background setsid so this adb shell can exit.
adb_hy shell "busybox setsid /system/bin/sh $REMOTE/start.sh </dev/null >$REMOTE/start.log 2>&1 &"
echo "dnssink instalado em $HY300_ADB_HOST (rc no overlay; reboot registra o serviço init)"
