#!/bin/bash
# Start/stop the two private lab gservers (loopback only).
R=${SEIHAI_LAB:?set SEIHAI_LAB to the private lab folder}/run
case $1 in
 up)
  (cd $R/gs && GS_LAB_NOLIST=1 GS_LAB_512_ROUNDDOWN=1 setsid ./gs2emu > $R/gs.log 2>&1 &)
  (cd $R/login && GS_LAB_NOLIST=1 GS_LAB_512_ROUNDDOWN=1 GS_LAB_SERVERWARP="0,Lab Game,127.0.0.1,14802" setsid ./gs2emu > $R/login.log 2>&1 &)
  sleep 2; pgrep -ax gs2emu;;
 down) $(dirname "$0")/killgs.sh gs login;;
esac
