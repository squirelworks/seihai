#!/usr/bin/env bash
# Seihai: one game-server login with a packet capture. Linux.
# Usage:  ./run_gserver_check.sh /path/to/seihai host:port [version]
# host:port is required. Default version 2.22, default account guest (override with ACCOUNT=).
# Needs tcpdump and sudo. Password is read from a hidden prompt.
# Keep it to one login of a few minutes: about five ordinary chat lines,
# one or two level warps through normal links. Then close Seihai.
set -u
umask 077

SEIHAI="${1:-}"
TARGET="${2:-}"
VERSION="${3:-2.22}"
ACCOUNT="${ACCOUNT:-guest}"
HOST="${TARGET%%:*}"
PORT="${TARGET##*:}"

if [ -z "$SEIHAI" ] || [ ! -x "$SEIHAI" ] || [ -z "$TARGET" ]; then
  echo "Usage: $0 /path/to/seihai host:port [version]"; exit 1
fi
command -v tcpdump >/dev/null || { echo "tcpdump not found (sudo apt install tcpdump)"; exit 1; }

OUT="$(pwd)/seihai_gserver_check_$(date +%Y%m%d_%H%M%S)"
mkdir -m 700 "$OUT"
echo "Output folder: $OUT"
echo "Target: $HOST:$PORT  version: $VERSION"

{
  echo "date: $(date -Is)"
  if timeout 6 bash -c "exec 3<>/dev/tcp/$HOST/$PORT" 2>/dev/null; then
    echo "port $PORT: connect OK"
  else
    echo "port $PORT: no connection"
  fi
} | tee "$OUT/00_port_check.txt"

read -r -s -p "Password for account $ACCOUNT (hidden): " GRAAL_PW; echo
export GRAAL_PW

sudo -v || { echo "sudo is needed for tcpdump"; exit 1; }
sudo tcpdump -i any -s 0 -U -w "$OUT/capture.pcap" "host $HOST and tcp port $PORT" \
  >"$OUT/tcpdump.log" 2>&1 &
TCPDUMP_PID=$!
sleep 2

echo
echo "Starting Seihai. Say hi once in chat, walk through one door, then close the window."
echo "Press Ctrl+C here if it hangs."
echo
"$SEIHAI" --online "$HOST:$PORT" --account "$ACCOUNT" --password-env GRAAL_PW \
  --version "$VERSION" 2>&1 | tee "$OUT/seihai_output.txt"

sleep 2
sudo kill -TERM "$TCPDUMP_PID" 2>/dev/null
wait "$TCPDUMP_PID" 2>/dev/null
unset GRAAL_PW
sudo chown -R "$(id -u):$(id -g)" "$OUT"; chmod 600 "$OUT"/* 2>/dev/null

( cd "$(dirname "$OUT")" && tar czf "$OUT.tar.gz" "$(basename "$OUT")" ) && chmod 600 "$OUT.tar.gz"
echo
echo "Done. Send me this file:  $OUT.tar.gz"
echo "(Contains the pcap with login frames and Seihai's output. Keep it private.)"
