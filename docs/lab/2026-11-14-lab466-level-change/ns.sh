#!/bin/bash
# ns.sh up | down | CMD... : lo-only network namespace for lab 466 level change (bwrap --unshare-net holder)
PF=$HOME/Claude/labs/lab466-levelchange-2026-11-14/run/ns.pid
if [ "$1" = up ]; then
  setsid nohup bwrap --dev-bind / / --unshare-net sleep 2147466 >/dev/null 2>&1 &
  sleep 1; ps -eo pid,args | awk '$2=="sleep" && $3=="2147466"{print $1}' | tail -1 > $PF; cat $PF; exit
fi
if [ "$1" = down ]; then kill $(cat $PF); exit; fi
exec nsenter --preserve-credentials -U -n -t $(cat $PF) "$@"
