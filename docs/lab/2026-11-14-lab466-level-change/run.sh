#!/bin/bash
# run.sh TAG FLOAT_SEC: #466 level change. Ato 3.153 on the Enigma copy (egk-10 has a lab LINK at x=4 y=40-49 in deep water
# to the dry start spot onlinestartlocal.graal 23.5,30.5). Gserver-v2 + Ato in a loopback-only netns, Xvfb :87, PipeWire null sink lab466null.
L=$HOME/Claude/labs/lab466-levelchange-2026-11-14; R=$L/run; O=$L/out/$1; F=$2; mkdir -p $O; rm -f $O/*
G=$L/gs; N=$R/ns.sh; D=:87; ATO=$L/target/release/seihai-ato; A=$HOME/Claude/cadence-lab-2026-11-13/c304/tree
[ -S /tmp/.X11-unix/X87 ] || { setsid nohup Xvfb $D -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 < /dev/null & sleep 2; }
ev(){ echo "$(date +%s.%N) $*" >> $O/events.txt; }
mkdir -p $G/servers/enigma/logs
(cd $G && setsid $N ./gs2emu enigma > $O/gserver-stdout.txt 2>&1 < /dev/null &)
for i in $(seq 60); do $N ss -ltn | grep -q ':14806 ' && break; sleep 1; done; ev gserver-up
pw-cat --playback --raw --format s16 --rate 44100 --channels 2 --target lab466null - < /dev/zero & ZP=$!; sleep 1
pw-record --target lab466null -P '{ stream.capture.sink=true }' --rate 44100 --channels 2 --format s16 $O/rec.wav & RP=$!; sleep 2
ev beep; pw-play --target lab466null $R/beep.wav; ev rec-start
( cd $O && env -u WAYLAND_DISPLAY DISPLAY=$D LIBGL_ALWAYS_SOFTWARE=1 PIPEWIRE_NODE=lab466null GRAAL_PW=lab466pw setsid $N $ATO --version 3.153 --online 127.0.0.1:14806 --account lab466$1 --password-env GRAAL_PW --assets $A 2> $O/ato-stderr.txt < /dev/null | python3 -u $R/stamp.py $O/ato-stdout-timed.txt ) &
for i in $(seq 90); do grep -q " level onlinestartlocal" $O/ato-stdout-timed.txt 2>/dev/null && break; sleep 1; done; sleep 3
export DISPLAY=$D
python3 $R/xt.py "ev $O/events.txt" "focus Seihai Ato" "shot $O/0-start.png" \
  "hold Down 0.2" "wait 0.3" "hold Down 3.3" "wait 3" "shot $O/1-egk10.png" \
  "hold Left 0.2" "wait 0.3" "hold Left 2.5" "wait 3" \
  "hold Left 1.6" "wait 1" "shot $O/2-in-water.png" "wait $F" \
  "hold Left 1.5" "wait 0.3" "shot $O/3-arrived.png" "wait 3" \
  "hold Down 1.0" "wait 2" "shot $O/4-end.png" > $O/xt.txt 2>&1
ev stop; kill -INT $($R/mypids.sh $ATO) 2>/dev/null; sleep 1; kill $RP $ZP 2>/dev/null; sleep 1
kill -INT $($R/mypids.sh $G/gs2emu) 2>/dev/null; sleep 2; kill $($R/mypids.sh $G/gs2emu) 2>/dev/null
cp $G/servers/enigma/logs/*.txt $O/ 2>/dev/null; ev done
