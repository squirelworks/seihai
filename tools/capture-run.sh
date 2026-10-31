#!/usr/bin/env bash
# Seihai: one online run with a packet capture. Linux.
# The Windows equivalent is tools/capture-run.ps1; keep the two in step.
#
# Usage: tools/capture-run.sh [options] [-- extra seihai args]
#   --target HOST:PORT    listserver or gserver (default listserver.graal.in:14922)
#   --version V           client version (default 2.22)
#   --account A           account (default guest)
#   --password-file F     read the password from F (first line); otherwise
#                         GRAAL_PW from the environment, otherwise a hidden prompt
#   --seihai PATH         the binary (default target/release/seihai)
#   --out DIR             parent folder for the output (default _debug)
#
# Example: tools/capture-run.sh --target listserver.graal.in:14922 --version 2.22
#
# Run it from the repo root. Needs tcpdump; sudo is used only when tcpdump
# lacks capture rights. Seihai's debug folder (packets.log, script.log) is
# written when the settings file has debug=1, and is copied into the output.
# Keep a public run short: one login, a few chat lines, a warp or two.
set -u
umask 077

TARGET="listserver.graal.in:14922"
VERSION="2.22"
ACCOUNT="guest"
PWFILE=""
SEIHAI="./target/release/seihai"
OUTROOT="_debug"
# Listserver, gserver and serverwarp ports seen so far.
PORTS="14921 14922 14923 14801 14802 14842 14900 14912 14913"

while [ $# -gt 0 ]; do
  case "$1" in
    --target) TARGET="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --account) ACCOUNT="$2"; shift 2 ;;
    --password-file) PWFILE="$2"; shift 2 ;;
    --seihai) SEIHAI="$2"; shift 2 ;;
    --out) OUTROOT="$2"; shift 2 ;;
    --) shift; break ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "unknown option: $1 (try --help)"; exit 1 ;;
  esac
done

HOST="${TARGET%%:*}"
PORT="${TARGET##*:}"
case " $PORTS " in *" $PORT "*) ;; *) PORTS="$PORTS $PORT" ;; esac
FILTER="tcp and ($(echo "$PORTS" | sed 's/ / or /g; s/\([0-9][0-9]*\)/port \1/g'))"

[ -x "$SEIHAI" ] || { echo "Seihai not found at $SEIHAI (cargo build --release, or --seihai PATH)"; exit 1; }
command -v tcpdump >/dev/null || { echo "tcpdump not found (sudo apt install tcpdump)"; exit 1; }

OUT="$OUTROOT/run-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUTROOT" && mkdir -m 700 "$OUT" || exit 1
echo "Output folder: $OUT"
echo "Target: $HOST:$PORT  version: $VERSION  account: $ACCOUNT"

{
  echo "date: $(date -Is)"
  echo "seihai: $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
  echo "target: $HOST:$PORT  version: $VERSION  account: $ACCOUNT"
  if timeout 6 bash -c "exec 3<>/dev/tcp/$HOST/$PORT" 2>/dev/null; then
    echo "port $PORT: connect OK"
  else
    echo "port $PORT: no connection"
  fi
} | tee "$OUT/00_run_info.txt"

# The password never goes on the command line or into the output.
if [ -n "$PWFILE" ]; then
  IFS= read -r GRAAL_PW < "$PWFILE" || { echo "cannot read $PWFILE"; exit 1; }
elif [ -z "${GRAAL_PW:-}" ]; then
  read -r -s -p "Password for account $ACCOUNT (hidden): " GRAAL_PW; echo
fi
export GRAAL_PW

# tcpdump with file capabilities (setcap cap_net_raw,cap_net_admin) runs as is.
SUDO=""
if ! getcap "$(command -v tcpdump)" 2>/dev/null | grep -q cap_net_raw && [ "$(id -u)" != 0 ]; then
  SUDO="sudo"
  sudo -v || { echo "sudo is needed for tcpdump"; exit 1; }
fi
$SUDO tcpdump -i any -s 0 -U -w "$OUT/capture.pcap" "$FILTER" >"$OUT/tcpdump.log" 2>&1 &
TCPDUMP_PID=$!
sleep 1
kill -0 "$TCPDUMP_PID" 2>/dev/null || { echo "tcpdump did not start:"; cat "$OUT/tcpdump.log"; exit 1; }

MARK="$OUT/.start"; touch "$MARK"
echo
echo "Starting Seihai. Close its window when done; Ctrl+C here if it hangs."
echo
"$SEIHAI" --online "$HOST:$PORT" --account "$ACCOUNT" --password-env GRAAL_PW \
  --version "$VERSION" "$@" 2>&1 | tee "$OUT/seihai_output.txt"
unset GRAAL_PW

sleep 1
$SUDO kill -INT "$TCPDUMP_PID" 2>/dev/null
wait "$TCPDUMP_PID" 2>/dev/null
[ -n "$SUDO" ] && sudo chown -R "$(id -u):$(id -g)" "$OUT"

# Seihai's own debug folders from this run (packets.log, script.log).
for d in _debug/debug-*; do
  [ -d "$d" ] && [ "$d" -nt "$MARK" ] && cp -r "$d" "$OUT/"
done
rm -f "$MARK"
chmod -R go-rwx "$OUT"

( cd "$(dirname "$OUT")" && tar czf "$(basename "$OUT").tar.gz" "$(basename "$OUT")" ) && chmod 600 "$OUT.tar.gz"
echo
echo "Done: $OUT.tar.gz ($(wc -c < "$OUT/capture.pcap") bytes of capture)"
echo "Decode with: $SEIHAI capture show $OUT/capture.pcap"
echo "(Contains login frames. Keep it private.)"
