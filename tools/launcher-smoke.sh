#!/usr/bin/env bash
# The one-program smoke (#469, #471) for CI on a desktop with no one at it:
# for each VERSION, `seihai --launcher-smoke` draws the start window, joins
# a stand-in game server on loopback as if Join were pressed, runs that
# line's client in the same process, and comes back to the start window
# when the stand-in drops the connection.
#
# Usage: tools/launcher-smoke.sh /path/to/seihai OUTDIR VERSION...
#
# The stand-in takes the client's login, holds the connection for HOLD
# seconds (default 8) so the client's window draws, then closes it. A
# version passes when the start window drew, the client connected and sent
# its login, and the start window came back with a reason. Each run's
# output is OUTDIR/VERSION.log; with SCREENSHOT set (a command taking a
# file name, e.g. `screencapture -x`), the screen is saved while the
# client is up.
set -u
SEIHAI=$1
OUT=$2
shift 2
HOLD=${HOLD:-8}
mkdir -p "$OUT"
export SEIHAI_FRAME_LOG=1
# 6.x will not log in without a password; the stand-in takes any.
export SEIHAI_PASSWORD=smoke
# The smoke saves nothing, but a client run on its own might: keep it out
# of the runner's home.
export HOME="$OUT/home"
mkdir -p "$HOME"
fail=0

for V in "$@"; do
  safe=$(echo "$V" | tr -c 'A-Za-z0-9.\n' '_')
  log="$OUT/$safe.log"
  got="$OUT/$safe.login.bin"
  ready="$OUT/$safe.ready"
  rm -f "$got" "$ready"
  python3 -I - "$ready" "$got" "$HOLD" <<'PY' &
import socket, sys, time
ready, got, hold = sys.argv[1], sys.argv[2], float(sys.argv[3])
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", 0))
s.listen(1)
open(ready, "w").write(str(s.getsockname()[1]))
s.settimeout(120)
c, _ = s.accept()
c.settimeout(0.5)
data = b""
end = time.time() + hold
while time.time() < end:
    try:
        b = c.recv(65536)
        if not b:
            break
        data += b
    except socket.timeout:
        pass
open(got, "wb").write(data)
c.close()
s.close()
PY
  stand_in=$!
  for _ in $(seq 50); do [ -s "$ready" ] && break; sleep 0.1; done
  port=$(cat "$ready")
  echo "=== $V -> 127.0.0.1:$port"
  "$SEIHAI" --launcher-smoke "$V@127.0.0.1:$port" > "$log" 2>&1 &
  pid=$!
  if [ -n "${SCREENSHOT:-}" ]; then
    sleep $((HOLD / 2 + 2))
    $SCREENSHOT "$OUT/$safe.png" || true
  fi
  # The whole run: start window, client, start window again.
  for _ in $(seq 120); do kill -0 "$pid" 2>/dev/null || break; sleep 1; done
  if kill -0 "$pid" 2>/dev/null; then
    echo "$V: still running after 120 s" >> "$log"
    kill "$pid" 2>/dev/null
  fi
  wait "$pid"
  rc=$?
  kill "$stand_in" 2>/dev/null
  wait "$stand_in" 2>/dev/null
  cat "$log"
  bytes=0; [ -f "$got" ] && bytes=$(wc -c < "$got")
  bytes=$(echo $bytes)
  echo "$V: exit $rc, the client sent $bytes bytes"
  ok=1
  grep -q "smoke: start window drew .* joining" "$log" || { echo "::error::$V: the start window did not join"; ok=0; }
  [ "$bytes" -gt 0 ] || { echo "::error::$V: the client sent no login"; ok=0; }
  grep -q "smoke: back at the start window" "$log" || { echo "::error::$V: did not come back to the start window"; ok=0; }
  grep -q "smoke: start window drew .* again" "$log" || { echo "::error::$V: the start window did not draw again"; ok=0; }
  [ "$rc" -eq 0 ] || { echo "::error::$V: seihai exited $rc"; ok=0; }
  [ "$ok" -eq 1 ] && echo "$V: ok" || fail=1
done
exit $fail
