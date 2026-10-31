#!/bin/bash
# linktest.sh VERSION ACCOUNT DISPLAY BIN: log in through the lab Login lobby or straight in, walk east into the hub's east-edge link.
V=$1; A=$2; D=$3; BIN=${4:-$(cd "$(dirname "$0")/../.." && pwd)/target/release/seihai}
O=${SEIHAI_LAB:?set SEIHAI_LAB to the private lab folder}/out/linktest/$(echo "$V" | tr ' ' _)-$(basename $(dirname $(dirname $BIN))); rm -rf $O; mkdir -p $O/.config/seihai; [ -n "${DEBUG:-}" ] && echo debug=1 > $O/.config/seihai/seihai.conf
(Xvfb :$D -screen 0 1024x768x24 -ac -nolisten tcp >/dev/null 2>&1 &); sleep 1
case $V in 5.*|6.*) T=127.0.0.1:14801;; *) T=127.0.0.1:14802;; esac
(cd $O && DISPLAY=:$D HOME=$O LAB_PW=x timeout 30 $BIN --online $T --account $A --password-env LAB_PW --version "$V" --no-sound > run.log 2>&1 &)
sleep 8; DISPLAY=:$D xdotool mousemove 400 300; for w in $(DISPLAY=:$D xdotool search --name "."); do DISPLAY=:$D xdotool windowfocus $w 2>/dev/null; done; sleep 1
DISPLAY=:$D xdotool keydown Right; sleep 9; DISPLAY=:$D xdotool keyup Right
sleep 14
pkill -f "Xvfb :$D " ; echo "$V: $(grep -c 'warp not answered' $O/run.log) not answered; $(grep -c 'level from' $O/run.log) levels; $(grep -m2 'link to' $O/run.log | tr '\n' ' ')"
