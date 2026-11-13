#!/bin/bash
# run.sh <tag> [extra ato args]: Ato 3.153 on Enigma, record lab466null, walk the earlier water route.
T=$1; shift; O=~/lab466/out/$T; mkdir -p $O; export DISPLAY=:86 LIBGL_ALWAYS_SOFTWARE=1
[ -S /tmp/.X11-unix/X86 ] || { setsid nohup Xvfb :86 -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 < /dev/null & for i in $(seq 1 30); do [ -S /tmp/.X11-unix/X86 ] && break; sleep 1; done; sleep 2; }
ev(){ echo "$(date +%s.%N) $*" >> $O/events.txt; }
pacat --playback --raw --format=s16le --rate=44100 --channels=2 --device=lab466null < /dev/zero & ZP=$!; sleep 1; parec --device=lab466null.monitor --file-format=wav $O/rec.wav & RP=$!; sleep 2; ev beep; paplay --device=lab466null ~/lab466/beep.wav; ev rec-start
GRAAL_PW=$(head -c 9 /dev/urandom | base64 | tr -d /+=) PULSE_SINK=lab466null setsid nohup ~/lab400/E ~/lab466/bin/seihai-ato --version 3.153 --online 127.0.0.1:14806 --account lab466$T --password-env GRAAL_PW --assets ~/seihai-lab/client-304 "$@" 2>&1 < /dev/null | python3 -u ~/lab466/stamp.py $O/ato-stdout-timed.txt &
AP=$!
for i in $(seq 1 60); do W=$(xdotool search --onlyvisible --name "Seihai Ato" 2>/dev/null | head -1); [ -n "$W" ] && grep -q " level " $O/ato-stdout-timed.txt && break; sleep 1; done
sleep 3; xdotool windowactivate --sync $W 2>/dev/null; xdotool windowfocus $W
k(){ ev down $1; xdotool keydown $1; sleep $2; xdotool keyup $1; ev up $1; sleep 0.3; }
shot(){ ev shot $1; scrot -o $O/$1.png; }
shot 0-start
ev mark land-walk-start; k Down 0.2; k Down 3.3; sleep 3; shot 1-egk10
k Left 0.2; k Left 2.5; ev mark land-idle-start; sleep 3; shot 2-land-idle
k Left 1.6; ev mark float-start; sleep 1; shot 3-in-water; sleep 4; ev mark float-end
k Left 3.0; shot 4-swim-left; k Right 4.6; ev mark back-on-land; sleep 1; shot 5-back-on-land
k Right 1.5; sleep 2; ev mark stop-rec; shot 6-end
kill $RP $ZP; kill $(pgrep -P $AP 2>/dev/null) $AP 2>/dev/null; for p in $(pgrep -x seihai-ato); do tr "\0" " " < /proc/$p/cmdline | grep -q "lab466$T" && kill $p; done
ev done
