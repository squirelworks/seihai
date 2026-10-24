# Where a level link fires

Queue item 15(a), from the differential testing (`documentation/difftest.md`,
divergence 1: "Seihai never sends the level warp for a rectangle link, the real
client does").

## The old rule and why it missed

Seihai tested one point, the feet (`x + 2.0`, `y + 2.75`), against the link's tile
rectangle. The lab's rectangle link (`LINK lv_c.nw 10 28 2 5 ...`: columns 10 and 11, rows
28 to 32) is walked into from the side at `y` 30.5 in the difftest scenarios. The feet
point is then at row 33 (`30.5 + 2.75`), under the link, so Seihai never
entered it, while the real client's feet strip (`y + 2` to `y + 3`) still overlaps
the bottom row. The edge links (strips along the level border) were fine because they
run the whole level.

## What the real clients do

Method: scenarios `linkgeom` and `linkgeom2` (`tools/difftest/scenarios/`) in the
private lab, 2.22 and 2.31 real clients. A server chat command puts the player at
a half-tile position, the scenario holds an arrow key for 1.2 s (24 steps of 0.5
tile), and the trace shows the last position the client reported and the
`PLI_LEVELWARP`/`PLI_LEVELWARPMOD` that follows. Server warps only land on half
tiles, so nothing finer than half a tile can be measured.

The warp goes out at the step after the last reported position. Results (both
versions agree on every probe except one in 2.31, below):

| walking | start | last reported before the warp | link |
|---|---|---|---|
| down | x 8.5, 9.0, 9.5, 10.0, 10.5, y 20 | y 24.5 | rectangle, rows 28 to 32 |
| down | x 7.0, 7.5, 8.0, 11.0, 11.5, 12.0, y 20 | no warp | |
| up | x 8.5, 9.5, y 40 | y 32.5 | rectangle |
| up | x 11.0, y 40 | no warp | |
| left | x 14, y 26 to 27.5 | x 12 | rectangle, columns 10 to 12 |
| left | x 14, y 32 to 33.5 | no warp | |
| right | x 6, y 26.5 to 27.5 | x 7 | rectangle |
| right | x 6, y 32.5 and 33 | no warp | |
| down | x 30, from y 56 | y 58.5 | south edge strip (rows 62, 63) |
| up | x 30, from y 6 | y 1.5 | north edge strip (rows 0, 1) |
| right | y 30.5, from x 56 | x 59 | east edge strip (columns 62, 63) |

One 2.31 probe differs: up at x 10.5, y 40 warped after y 33, one step earlier
than at x 8.5 and 9.5 (the 2.22 client did not). Seihai does not copy it; the
cause is not known (a missed tick in that run is possible).

## The model

The real client behaves as if each walking direction tests a small box against
the link rectangle (boxes that only touch a link edge do not fire). Offsets are from the
player's position, in tiles:

| direction | x range | y range | what the captures fix |
|---|---|---|---|
| north | 1.0 to 2.0 | 0.75 to 1.25 | the top edge is above `y + 0.5` and at or below `y + 1.0` |
| west | 0.25 to 0.75 | 2.0 to 3.0 | the left edge is right of `x + 0.0` and at or left of `x + 0.5`; the box reaches `y + 2` (warp at y 26) and not `y + 1` (no warp at y 32) |
| south | 1.0 to 2.0 | 2.75 to 3.25 | the bottom edge is below `y + 3.0` and at or above `y + 3.5` |
| east | 2.25 to 2.75 | 2.0 to 3.0 | the right edge is at least `x + 2.5` and below `x + 3.0` |

The numbers inside those ranges are chosen, not measured: only where the half-tile
step first overlaps is known. The one tile wide north and south box is what the
down probes at x 8.0 (no warp) and 10.5 (warp) require: a point cannot do both.
`offline::LINK_BOX` holds the table and `offline` has tests with every probe above.

## Scope

Only builds with `levels.link_boxes` use the boxes: 2.22 and 2.31, plus the builds
between and after them in the version chain, taken as the same (inferred). Earlier
builds keep the foot-point rule; nobody measured them.

## Before and after (lab, real client against Seihai)

`linkgeom2` (22 probes per version, 2.22 and 2.31): a probe counts as agreeing when the real
client and Seihai either both send the level warp for `lv_c.nw` or both do not.

| Seihai | probes that agree with the real client | probes where Seihai warps |
|---|---|---|
| before (foot point) | 16 of 22 | 11 (real client: 15) |
| after (direction boxes) | 22 of 22 | 15 |

Same result on 2.22 and 2.31.

`door_warp` on 2.22: the real client sends `LEVELWARP lv_c.nw 60 30`, then `LEVELWARPMOD lv_a.nw 14 30` on the
way back, then `LEVELWARP lv_w.nw` at the edge. Before, Seihai sent only the edge warp (the diff listed the
`LEVELWARP` to `lv_c.nw` and the `LEVELWARPMOD` as missing, 3 high divergences in all). After, Seihai sends the
`lv_c.nw` warp with the same `x=60`, `y=30`, the `LEVELWARPMOD` back at `x=14`, `y=30`, and the edge warp, and
the report has no missing level warp. The edge warp's `x` differs (real 17.5, Seihai 20.5) because the two clients
walk at different speeds (divergence 9).
