#!/bin/bash
# lab2-ato.sh (traced gserver build) TAG: #458 pass 2, Ato 5.12 --graal3d windowed on Xvfb :71, gserver-v2 lab3d (TCP 14830, UDP 14831) in the lab netns.
L=$HOME/Claude/labs/lab458-pass2-2026-11-14; R=$L/run; O=$L/out/ato-$1; mkdir -p $O
G=$L/gs-trace; N=$R/ns.sh; D=:71; ATO=$L/target/release/seihai-ato
[ -S /tmp/.X11-unix/X71 ] || { setsid nohup Xvfb $D -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 < /dev/null & sleep 2; }
ev(){ echo "$(date +%s.%3N) $*" >> $O/events.txt; }
mkdir -p $G/servers/lab3d/logs.old; mv $G/servers/lab3d/logs/* $G/servers/lab3d/logs.old/ 2>/dev/null
(cd $G && SEIHAI_G3D_TRACE=1 setsid $N ./gs2emu lab3d > $O/gserver-stdout.txt 2>&1 < /dev/null &)
for i in $(seq 30); do $N ss -lun | grep -q ':14831 ' && break; sleep 1; done; ev gserver-up
setsid $N tcpdump -i lo -U -w $O/graal3d-udp-14831.pcap -G 150 -W 1 udp port 14831 > $O/tcpdump.txt 2>&1 < /dev/null &
sleep 1; ev ato-start
( cd $O && env -u WAYLAND_DISPLAY DISPLAY=$D LIBGL_ALWAYS_SOFTWARE=1 GRAAL_PW=lab458pw setsid $N $ATO --version 5.12 --online 127.0.0.1:14830 --account labato --password-env GRAAL_PW --graal3d 2> $O/ato-stderr.txt < /dev/null | tee $O/ato-stdout.txt | python3 -u $R/stamp.py $O/ato-stdout-timed.txt ) &
for i in $(seq 60); do grep -q 'login step 6' $O/ato-stdout.txt 2>/dev/null && break; sleep 1; done; ev step6-seen; sleep 3
export DISPLAY=$D
python3 $R/xt.py "ev $O/events.txt" "focus ${WIN:-Ato}" "shot $O/0-start.png" \
   "hold Up 10" "wait 1" "shot $O/1-walked.png" \
   "hold Left 1.5" "wait 0.5" "hold Up 2" "wait 0.5" "hold Right 3" "wait 1" "shot $O/2-turned.png" \
   "tap Tab" "wait 0.3" "type hi e:smile" "tap Return" "wait 3" "shot $O/3-said.png"
sleep 3; ev stop
kill -INT $($R/mypids.sh $ATO) 2>/dev/null; sleep 2
for i in $(seq 90); do pgrep -x tcpdump -U $(id -u) >/dev/null || break; $N python3 -c "import socket;socket.socket(2,2).sendto(b'end',('127.0.0.1',14831))"; sleep 1; done
kill -INT $($R/mypids.sh $G/gs2emu) 2>/dev/null; sleep 2; kill $($R/mypids.sh $G/gs2emu) 2>/dev/null
cp -r $G/servers/lab3d/logs $O/gserver-logs 2>/dev/null; ev done
