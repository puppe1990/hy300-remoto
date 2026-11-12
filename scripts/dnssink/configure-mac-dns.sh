#!/usr/bin/env bash
# Point this Mac's Wi-Fi at the HY300 sinkhole, with 1.1.1.1 as DNS 2.
set -euo pipefail
networksetup -setdnsservers Wi-Fi 192.168.15.4 1.1.1.1
echo "Wi-Fi DNS: 192.168.15.4 1.1.1.1"
