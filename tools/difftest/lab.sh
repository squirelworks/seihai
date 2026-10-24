#!/usr/bin/env bash
# Differential-testing lab runner: the same scripted session against a real
# (Wine) Graal client and against Seihai, in a PRIVATE network namespace with
# its own displays, then a frame-level diff of the two captures.
#
#   lab.sh up                          create the namespace and the two displays
#   lab.sh run  <ver> <scenario> <real|seihai|seihai-parity>   one capture (pcap + trace)
#       seihai         Seihai with its default options
#       seihai-parity  Seihai with --send-tampercheck --send-showimg (the
#                      opt-in senders), to see what is left once those are on
#   lab.sh pair <ver> <scenario> [seihai|seihai-parity]   real, Seihai, then the diff
#   lab.sh diff <ver> <scenario> [seihai|seihai-parity]   only the diff step
#   lab.sh all  "<ver ...>" "<scenario ...>"   every pair for each version and scenario
#   lab.sh retrace                     re-decode every pcap in cap/ and re-diff every pair
#                                      (no lab needed: use it after changing the tools)
#   lab.sh down                        stop everything this script started
#
# <ver> is 2.17, 2.22 or 2.31. Scenarios are the files in scenarios/.
#
# Safety rules (the script enforces the first three):
#   * the namespace name must start with "graallabd"; the shared lab
#     namespaces (graallab, graallabb, graallabc, graallabg) are refused;
#   * the displays are never :3 and never one that is already in use;
#   * /etc is never edited: the lab's hosts file is bind-mounted over
#     /etc/hosts inside a private mount namespace of the lab processes;
#   * "down" kills only processes inside the private namespace, only the
#     Xvfb processes this script started, and `wineserver -k` only on the
#     lab's own prefix copy.
# Everything it needs lives under $DIFFTEST_ROOT (mode 700, outside the repo;
# see tools/difftest/README.md). No public server is contacted: every socket
# is loopback inside the namespace. Accounts and the password are
# placeholders; the password is never printed.
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
ROOT=${DIFFTEST_ROOT:?set DIFFTEST_ROOT to the private lab directory (mode 700)}
NS=${DIFFTEST_NS:-graallabd}
DR=${DIFFTEST_DISPLAY_REAL:-31}
DS=${DIFFTEST_DISPLAY_SEIHAI:-32}
ACCT=${DIFFTEST_ACCOUNT:-dtacct}
export DIFFTEST_PW=${DIFFTEST_PASSWORD:-labpass-placeholder}
SEIHAI=${DIFFTEST_SEIHAI_BIN:-$REPO/target/release/seihai}
DIFFT=${DIFFTEST_BIN:-$REPO/target/release/difftest}
LUSER=$(id -un)
export PATH=/usr/bin:$PATH

case $NS in
  graallab|graallabb|graallabc|graallabg) echo "refusing shared lab namespace $NS" >&2; exit 2;;
  graallabd*) ;;
  *) echo "namespace must start with graallabd (got $NS)" >&2; exit 2;;
esac
for d in $DR $DS; do
  [ "$d" = 3 ] && { echo "refusing display :3" >&2; exit 2; }
done
[ "$DR" = "$DS" ] && { echo "the two displays must differ" >&2; exit 2; }

mkdir -p "$ROOT/cap" "$ROOT/run" "$ROOT/out"; chmod 700 "$ROOT" "$ROOT/cap" 2>/dev/null

# Run a command inside the namespace, as the normal user, with the lab's
# hosts file over /etc/hosts (private mount namespace; /etc is untouched).
nsx() {
  sudo ip netns exec "$NS" unshare -m bash -c \
    'h=$1; u=$2; shift 2; mount --bind "$h" /etc/hosts && exec sudo -u "$u" env HOME="$HOME" "$@"' _ \
    "$ROOT/hosts" "$LUSER" "$@"
}
ns_pids() { sudo ip netns pids "$NS" 2>/dev/null; }
disp_busy() { [ -e "/tmp/.X$1-lock" ]; }

cmd_up() {
  if ! ip netns list | grep -qw "$NS"; then
    sudo ip netns add "$NS" && echo "$NS" > "$ROOT/.created_ns"
  fi
  sudo ip netns exec "$NS" ip link set lo up
  for d in $DR $DS; do
    if [ -e "$ROOT/.xvfb$d.pid" ] && kill -0 "$(cat "$ROOT/.xvfb$d.pid")" 2>/dev/null; then continue; fi
    if disp_busy "$d"; then echo "display :$d is in use by someone else; pick another" >&2; exit 2; fi
    Xvfb ":$d" -screen 0 1024x768x24 -ac +extension GLX +render -noreset >"$ROOT/xvfb$d.log" 2>&1 &
    echo $! > "$ROOT/.xvfb$d.pid"
  done
  sleep 1
  echo "up: namespace $NS, displays :$DR (real client) and :$DS (Seihai)"
}

cmd_down() {
  # Only our namespace, only our Xvfb pids, only our prefix.
  ns_pids | xargs -r sudo kill 2>/dev/null; sleep 1; ns_pids | xargs -r sudo kill -9 2>/dev/null
  WINEPREFIX="$ROOT/wp" WINEDEBUG=-all wineserver -k 2>/dev/null
  for d in $DR $DS; do
    [ -e "$ROOT/.xvfb$d.pid" ] && kill "$(cat "$ROOT/.xvfb$d.pid")" 2>/dev/null; rm -f "$ROOT/.xvfb$d.pid"
  done
  if [ -e "$ROOT/.created_ns" ]; then sudo ip netns del "$NS" 2>/dev/null; rm -f "$ROOT/.created_ns"; fi
  chmod 600 "$ROOT"/cap/* 2>/dev/null
  echo "down: done"
}

xd()  { DISPLAY=":$1" xdotool "${@:2}"; }
snap() { python3 -c "from PIL import ImageGrab; ImageGrab.grab(xdisplay=':$1').save('$2')" 2>/dev/null; }

# Real client exe and listserver row per version.
exe_for() { case $1 in 2.17) echo Graal_2171.exe;; 2.22) echo Graal_222.exe;; 2.31) echo Graal_231.exe;; esac; }
pristine_for() { case $1 in 2.17) echo pristine217;; 2.22) echo pristine222;; 2.31) echo pristine231;; esac; }

# Account file for the run: start level and position come from the scenario.
make_account() {  # <server dir> <level> <x> <y>
  local f="$1/servers/lab/accounts/$ACCT.txt"
  sed -e "s/labb31/$ACCT/g" -e "s/^LEVEL .*/LEVEL $2/" -e "s/^X .*/X $3/" -e "s/^Y .*/Y $4/" \
    "$1/servers/lab/accounts/labb31.txt" > "$f"
}

start_services() {  # <run dir> <tag> <level> <x> <y>
  local run=$1 tag=$2
  rm -rf "$run/server" "$run/ls"; cp -a "$ROOT/server" "$run/server"; cp -a "$ROOT/ls" "$run/ls"
  : > "$run/ls/serverlog.txt"; : > "$run/ls/clientlog.txt"
  make_account "$run/server" "$3" "$4" "$5"
  ( umask 077; sudo ip netns exec "$NS" tcpdump -i lo -s 0 -U -n -Z "$LUSER" \
      -w "$ROOT/cap/$tag.pcap" 'tcp and (port 14922 or port 14802 or port 14803)' \
      >"$run/tcpdump.log" 2>&1 < /dev/null & )
  sleep 1
  ( cd "$run/ls" && nsx ./listserver >"$run/listserver.out" 2>&1 < /dev/null & )
  sleep 1
  ( cd "$run/server" && nsx ./gs2emu >"$run/gserver.out" 2>&1 < /dev/null & )
  for _ in $(seq 1 40); do
    sudo ip netns exec "$NS" ss -ltn | grep -q '127.0.0.1:14802' && break; sleep 0.5
  done
  sudo ip netns exec "$NS" ss -ltn | grep -q '127.0.0.1:14802' || { echo "gserver did not start" >&2; return 1; }
  # The listserver must know the gserver before a client asks for the list.
  for _ in $(seq 1 40); do grep -q 'New Server' "$run/ls/serverlog.txt" 2>/dev/null && break; sleep 0.5; done
  grep -q 'New Server' "$run/ls/serverlog.txt" 2>/dev/null || { echo "gserver did not register with the listserver" >&2; return 1; }
  sleep 1
}

stop_services() {  # <run dir> <tag>
  ns_pids | xargs -r sudo kill 2>/dev/null; sleep 2; ns_pids | xargs -r sudo kill -9 2>/dev/null
  WINEPREFIX="$ROOT/wp" WINEDEBUG=-all wineserver -k 2>/dev/null
  chmod 600 "$ROOT/cap/$2.pcap" 2>/dev/null
}

# ---- scenario steps (shared by both clients) -------------------------------
# start <level> <x> <y>   where the account starts (read by the runner)
# wait <s>                sleep
# hold <Key> <s>          hold an arrow key
# chat <text>             say it in the local chat bubble
# mark <name>             a label in the run's marks file (wall clock)
# click <x> <y>           (real client only) a mouse click, e.g. to focus
scenario_start() { awk '$1=="start"{print $2, $3, $4}' "$HERE/scenarios/$1.steps"; }

run_steps() {  # <kind> <scenario> <marks file>
  local kind=$1 file="$HERE/scenarios/$2.steps" marks=$3 disp
  [ "$kind" = real ] && disp=$DR || { kind=seihai; disp=$DS; }
  : > "$marks"
  while IFS= read -r line; do
    set -- $line
    case ${1:-} in
      ''|'#'*|start) ;;
      wait) sleep "$2";;
      hold) echo "$(date +%s.%N) hold $2 $3" >> "$marks"; xd $disp keydown "$2"; sleep "$3"; xd $disp keyup "$2"; sleep 0.5;;
      chat) shift; echo "$(date +%s.%N) chat" >> "$marks"
            if [ "$kind" = real ]; then
              xd $disp mousemove 500 625 click 1; sleep 0.6; xd $disp type --delay 60 "$*"; xd $disp key Return
              sleep 1.5; xd $disp mousemove 500 300
            else
              # Seihai polls keys once per frame, so a bare `xdotool key` (press and
              # release 12 ms apart) can fall between two polls: hold Return.
              xd $disp keydown Return; sleep 0.15; xd $disp keyup Return; sleep 0.5
              xd $disp type --delay 60 "$*"; sleep 0.3
              xd $disp keydown Return; sleep 0.15; xd $disp keyup Return; sleep 1.5
            fi;;
      mark) echo "$(date +%s.%N) mark $2" >> "$marks";;
      click) [ "$kind" = real ] && xd $disp mousemove "$2" "$3" click 1;;
    esac
  done < "$file"
}

real_login() {  # <ver> <run dir> <exe>
  local ver=$1 run=$2 exe=$3 d=$DR
  rm -rf "$run/client"; cp -a "$ROOT/$(pristine_for "$ver")" "$run/client"
  ( cd "$run/client" && nsx env DISPLAY=:$d WINEPREFIX="$ROOT/wp" WINEDEBUG=-all wine reg delete 'HKCU\Software\Graal\Client' /f >/dev/null 2>&1 )
  ( cd "$run/client" && nsx env DISPLAY=:$d WINEPREFIX="$ROOT/wp" WINEDEBUG=-all wine "$exe" >"$run/client.log" 2>&1 < /dev/null & )
  sleep 15
  xd $d mousemove 511 441 click 1; xd $d type --delay 80 "$ACCT"
  xd $d mousemove 500 530 click 1; xd $d type --delay 80 "$ACCT"
  xd $d mousemove 511 561 click 1; xd $d type --delay 80 "$DIFFTEST_PW"
  sleep 1; snap $d "$run/login1.png"
  xd $d mousemove 696 545 click 1; sleep 8
  snap $d "$run/login2.png"
  # first row of the server list, then Connect
  xd $d mousemove 255 389 click 1; sleep 1; xd $d mousemove 711 575 click 1
  sleep 14
  xd $d mousemove 500 300 click 1
  snap $d "$run/ingame.png"
}

seihai_start() {  # <ver> <run dir> <kind>
  local ver=$1 run=$2 d=$DS SEIHAI_EXTRA=${DIFFTEST_SEIHAI_ARGS:-}
  [ "$3" = seihai-parity ] && SEIHAI_EXTRA="$SEIHAI_EXTRA --send-tampercheck --send-showimg"
  # The real client logs in ~40 s after the services start (its login screens
  # take that long). A gserver that has just started still sends a default-level
  # warp first; give Seihai the same warm-up so that is not mistaken for a
  # client difference.
  sleep "${DIFFTEST_SEIHAI_SETTLE:-30}"
  rm -rf "$run/assets" "$run/home"; mkdir -p "$run/home"
  cp -a "$ROOT/assets" "$run/assets"; rm -rf "$run/assets/levels"
  ( printf '0\n' | nsx env DISPLAY=:$d HOME="$run/home" DIFFTEST_PW="$DIFFTEST_PW" "$SEIHAI" --assets "$run/assets" \
      --online 127.0.0.1:14922 --account "$ACCT" --password-env DIFFTEST_PW --version "$ver" $SEIHAI_EXTRA \
      >"$run/seihai.log" 2>&1 & )
  sleep 12
  xd $d mousemove 300 300
  snap $d "$run/ingame.png"
}

cmd_run() {  # <ver> <scenario> <real|seihai>
  local ver=$1 sc=$2 kind=$3 tag="$1-$2-$3"
  local run="$ROOT/run/$tag"; mkdir -p "$run"
  read -r lvl x y < <(scenario_start "$sc")
  cmd_up >/dev/null
  start_services "$run" "$tag" "$lvl" "$x" "$y" || return 1
  echo "run $tag: services up"
  if [ "$kind" = real ]; then real_login "$ver" "$run" "$(exe_for "$ver")"; else seihai_start "$ver" "$run" "$kind"; fi
  run_steps "$kind" "$sc" "$run/marks.txt"
  sleep 2
  stop_services "$run" "$tag"
  "$DIFFT" trace "$ROOT/cap/$tag.pcap" --version "$ver" --scenario "$sc" --client "${kind%-parity}" \
      --out "$ROOT/out/$tag.trace.tsv" 2>&1 | sed 's/^/  /'
}

cmd_diff() {  # <ver> <scenario> [seihai|seihai-parity]
  local ver=$1 sc=$2 k=${3:-seihai}
  "$DIFFT" diff "$ROOT/out/$ver-$sc-real.trace.tsv" "$ROOT/out/$ver-$sc-$k.trace.tsv" \
      --out "$ROOT/out/$ver-$sc-$k.diff" | grep '^Divergences' | sed "s/^/$ver $sc $k: /"
}

cmd_pair() { cmd_run "$1" "$2" real && cmd_run "$1" "$2" "${3:-seihai}" && cmd_diff "$1" "$2" "${3:-seihai}"; }

# Every scenario for every version: one real capture, then Seihai with its
# defaults and with the opt-in senders on, and both diffs.
cmd_all() {
  for v in $1; do for sc in $2; do
    cmd_run "$v" "$sc" real || continue
    for k in seihai seihai-parity; do cmd_run "$v" "$sc" "$k" && cmd_diff "$v" "$sc" "$k"; done
  done; done
}

cmd_retrace() {
  local p b ver rest sc k
  for p in "$ROOT"/cap/*.pcap; do
    b=$(basename "$p" .pcap); ver=${b%%-*}; rest=${b#*-}
    k=${rest##*-}; case $rest in *-seihai-parity) k=seihai-parity; sc=${rest%-seihai-parity};; *) sc=${rest%-*};; esac
    "$DIFFT" trace "$p" --version "$ver" --scenario "$sc" --client "${k%-parity}" --out "$ROOT/out/$b.trace.tsv" >/dev/null
  done
  for p in "$ROOT"/out/*-real.trace.tsv; do
    b=$(basename "$p" -real.trace.tsv); ver=${b%%-*}; sc=${b#*-}
    for k in seihai seihai-parity; do
      [ -e "$ROOT/out/$b-$k.trace.tsv" ] && cmd_diff "$ver" "$sc" "$k"
    done
  done
}

case ${1:-} in
  retrace) cmd_retrace;;
  up) cmd_up;;
  down) cmd_down;;
  run) cmd_run "$2" "$3" "$4";;
  pair) cmd_pair "$2" "$3" "${4:-seihai}";;
  diff) cmd_diff "$2" "$3" "${4:-seihai}";;
  all) cmd_all "$2" "$3";;
  *) sed -n '2,12p' "$0"; exit 2;;
esac
