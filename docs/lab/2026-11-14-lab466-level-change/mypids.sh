#!/bin/bash
# mypids.sh EXE-PATH : pids whose /proc exe is exactly this file
for p in /proc/[0-9]*; do [ "$(readlink $p/exe 2>/dev/null)" = "$1" ] && echo ${p#/proc/}; done
