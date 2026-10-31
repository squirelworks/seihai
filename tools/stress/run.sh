#!/bin/bash
# One stress run: fresh accounts, both lab servers, loopback pcap, harness.
# Usage: run.sh NAME [stress.py args...]
set -u
NAME=$1; shift
T=$(cd "$(dirname "$0")" && pwd); L=${SEIHAI_LAB:?set SEIHAI_LAB to the private lab folder}; O=$L/out/$NAME
rm -rf "$O"; mkdir -p "$O"; chmod 700 "$O"
$T/servers.sh down >/dev/null 2>&1; for p in $(pgrep -x Xvfb); do n=$(tr "\0" " " < /proc/$p/cmdline | awk "{print \$2}" | tr -d :); [ -n "$n" ] && [ "$n" -ge 50 ] && [ "$n" -lt 90 ] && kill $p; done; sleep 1
for s in gs login; do
  d=$L/run/$s/servers/default
  rm -f $d/accounts/stress*.txt
  python3 $T/genworld.py $d/world $d/accounts 64 >/dev/null
  : > $d/logs/serverlog.txt 2>/dev/null
done
rm -rf "$(cd "$(dirname "$0")/../.." && pwd)/target/release/_cache"
$T/servers.sh up > "$O/servers.txt"
tcpdump -i lo -s 0 -U -w "$O/lo.pcap" 'tcp port 14801 or tcp port 14802' > "$O/tcpdump.log" 2>&1 &
TD=$!
sleep 1
python3 $T/stress.py --out "$O" "$@" > "$O/harness.log" 2>&1
kill $TD; sleep 1
pgrep -x gs2emu > /dev/null || echo "GSERVER DIED" >> "$O/harness.log"
cp $L/run/gs.log $L/run/login.log "$O/" 2>/dev/null
cp $L/run/gs/servers/default/logs/*.txt "$O/" 2>/dev/null
$T/servers.sh down
echo "run $NAME done"
