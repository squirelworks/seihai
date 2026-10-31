#!/bin/bash
# The fuzz lab: game gserver on 14803 and a Login lobby on 14804 that sends
# 5.x/6.x clients to the fault proxy on 15803. Loopback only.
R=${SEIHAI_LAB:?set SEIHAI_LAB to the private lab folder}/run
case $1 in
 up)
  (cd $R/fz && GS_LAB_NOLIST=1 setsid ./gs2emu > $R/fz.log 2>&1 &)
  (cd $R/fzlogin && GS_LAB_NOLIST=1 GS_LAB_SERVERWARP="0,Lab Game,127.0.0.1,15803" setsid ./gs2emu > $R/fzlogin.log 2>&1 &)
  sleep 2;;
 down) $(dirname "$0")/killgs.sh fz fzlogin;;
esac
