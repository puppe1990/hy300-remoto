# shellcheck shell=bash
# Sourced by the Mac-side dnssink scripts. Override with HY300_ADB_HOST.
HY300_ADB_HOST="${HY300_ADB_HOST:-192.168.15.4:3268}"
REMOTE=/data/local/dnssink
STEVENBLACK_URL="https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts"

adb_hy() {
  command adb -s "$HY300_ADB_HOST" "$@"
}
