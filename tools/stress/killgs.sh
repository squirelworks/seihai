#!/bin/bash
# Kill the lab gservers whose working directory is one of the named run dirs.
for p in $(pgrep -x gs2emu); do
  d=$(basename "$(readlink /proc/$p/cwd)")
  for want in "$@"; do [ "$d" = "$want" ] && kill $p; done
done
