#!/bin/bash
# ns.sh up | down | CMD... : lo-only network namespace for lab 458 pass 2 (bwrap --unshare-net holder)
PF=$HOME/Claude/labs/lab458-pass2-2026-11-14/run/ns.pid
if [ "$1" = up ]; then
  setsid nohup bwrap --dev-bind / / --unshare-net sleep 2147483 >/dev/null 2>&1 &
  sleep 1; ps -eo pid,args | awk '$2=="sleep" && $3=="2147483"{print $1}' | tail -1 > $PF; cat $PF; exit
fi
if [ "$1" = down ]; then kill $(cat $PF); exit; fi
exec nsenter --preserve-credentials -U -n -t $(cat $PF) "$@"
