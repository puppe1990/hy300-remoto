#!/system/bin/sh
# WHY: init owns this process; exec so the service PID is dnsmasq, not a leftover sh.
DIR=/data/local/dnssink
IFACE=wlan0
TIMEOUT=60

wlan_ipv4() {
  ip -4 addr show "$IFACE" 2>/dev/null |
    busybox awk '/inet / { split($2, a, "/"); print a[1]; exit }'
}

wait_wlan_ip() {
  i=0
  while [ "$i" -lt "$TIMEOUT" ]; do
    ipaddr=$(wlan_ipv4)
    if [ -n "$ipaddr" ]; then
      echo "$ipaddr"
      return 0
    fi
    sleep 1
    i=$((i + 1))
  done
  return 1
}

write_conf() {
  ipaddr=$1
  cat >"$DIR/dnsmasq.conf" <<EOF
domain-needed
bogus-priv
no-resolv
server=1.1.1.1
server=8.8.8.8
strict-order
no-hosts
addn-hosts=$DIR/hosts
listen-address=127.0.0.1
listen-address=$ipaddr
bind-interfaces
interface=$IFACE
no-dhcp-interface=$IFACE
cache-size=10000
pid-file=$DIR/dnsmasq.pid
user=root
EOF
}

stop_old() {
  if [ ! -f "$DIR/dnsmasq.pid" ]; then
    return 0
  fi
  pid=$(cat "$DIR/dnsmasq.pid")
  if [ -n "$pid" ]; then
    kill "$pid" 2>/dev/null
    sleep 1
  fi
}

ipaddr=$(wait_wlan_ip) || {
  echo "wlan0 sem IPv4 em ${TIMEOUT}s, esperado endereço IPv4 na interface wlan0" >&2
  exit 1
}

stop_old
write_conf "$ipaddr"
echo dnssink >/sys/power/wake_lock
exec /system/bin/dnsmasq -C "$DIR/dnsmasq.conf" -k -x "$DIR/dnsmasq.pid"
