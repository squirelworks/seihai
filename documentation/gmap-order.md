# Gmap message order

Queue item 15, from the differential testing (`documentation/difftest.md`, divergences
11 and 12): the order of `PLI_ADJACENTLEVEL` (35) and the prop order of the 2.22 crossing
message.

## `PLI_ADJACENTLEVEL` order

Real 2.22 and 2.31 clients in the lab (`gmap_cross`, a 2x2 gmap `gm_nw gm_ne / gm_sw gm_se`, player
in the north-west cell) send the three neighbours as `gm_sw`, `gm_ne`, `gm_se`, in every run of
both versions. That is columns first, then rows: (0,1), (1,0), (1,1). Seihai went rows first (`gm_ne`,
`gm_sw`, `gm_se`). `Gmap::neighbours` now loops columns outside, rows inside; the server answers in
the order it was asked, so its `LEVELNAME` replies follow.

Only one cell was observed (the starting cell), so "columns first" is the simplest rule that fits it,
not a proven one. The gmap file order (`gm_ne` before `gm_sw`) does not fit.

## 2.22 crossing props

The real 2.22 client's crossing message writes the coordinate before the cell prop: `X, GMAPLEVELX`
(ids 15, 43) going east or west, `Y, GMAPLEVELY` (16, 44) going south or north. The 2.31 message is
`GMAPLEVELX, X2` (43, 78). Both are ascending prop ids, which is also how the walk props are written
(`GANI, X, SPRITE`; `GANI, SPRITE, X2`). `crossing_props_message` now writes its props in ascending id
order. The old 2.22 test had the cell prop first; it was written before the 2.22 capture was decoded.

A step that changes both axes at once was never captured. Ascending id order is applied to it too
(inferred).

Not changed: the value. The real 2.22 client reports `X=0.5` after crossing east where Seihai reports
`X=0`; the real client's props go out every second step (100 ms), so it is one step further when it
sends. That belongs to the walk cadence (see `documentation/difftest-leftovers.md`).

## Before and after (lab, `gmap_cross`, real client against Seihai)

| | real | before | after |
|---|---|---|---|
| ADJACENTLEVEL order (2.22 and 2.31) | sw, ne, se | ne, sw, se | sw, ne, se |
| 2.22 east crossing props | `X=0.5, GMAPLEVELX=1` | `GMAPLEVELX=1, X=0` | `X=0, GMAPLEVELX=1` |
| 2.22 west, south, north crossing props | `X=63.5, GMAPLEVELX=0` and the same for Y | cell prop first | coordinate first, same values |
| server `LEVELNAME` order (low divergence "field level") | gm_sw first | gm_ne first | gm_sw first (divergence gone) |

The ranked report's "props values differ" row for 2.22 no longer lists the crossing props.

## Crossing props on the 100 ms walk cadence (laundry item 8, 2026-10-31)

Seihai sent the whole crossing frame (39, flags, cell prop and coordinate) at the step that crossed,
even on builds that send their own props only every 100 ms (2.17, 2.22, 3.04,
`protocol/walkcadence.rs`). That gave a second props message inside one 100 ms slot. The real 2.22
captures fit one rule instead:

* the step that crosses sends the tamper check (39) and the `gr` flag in its own frame;
* the cell prop (43 or 44) goes with the next props message the cadence sends, next to the wrapped
  coordinate of its axis, ids ascending (`X, GMAPLEVELX`).

| real 2.22 capture | where the crossing step fell | props message |
|---|---|---|
| lab 2026-10-24, east | on a send slot | `X=0, 43=1`, same frame as the 39 |
| lab 2026-10-24, south | between slots | `Y=0.5, 44=1`, the frame after |
| lab 2026-10-24, west | between slots (39 at 63.5) | `X=63, 43=0`, the next one |
| `gmap_cross` difftest, east | between slots | `X=0.5, 43=1` |

Seihai now does the same: on a cadence build the main loop holds `gmap::crossing_lead` (the 39) for the
step's tick frame and calls `WalkCadence::cross`, which puts the cell prop and its axis coordinate in the
next message it sends. 2.31 (no cadence) is unchanged: 46 and the props in one frame at the step.

Confirmed: the four 2.22 rows above (tests in `protocol/walkcadence.rs` and `net/mod.rs`). Inferred: 2.17
and 3.04 cross the same way (their cadence matches 2.22's, but no crossing was captured). Not run yet: a lab walk of the 2x2 gmap with
Seihai 2.22 against the real client after this change.

## Walking back into a gmap level (lab item 6, 2026-10-30)

Lab, 2.22 walk around a 2x2 gmap: Seihai sent `PLI_NPCPROPS` again on every crossing back into a level
(`GANI idle` for a shopkeeper, `GANI idle` and `BODY body.png` for a warp NPC), because the level's
scripts were loaded afresh and `created` re-ran `setcharani`. The real 2.22 sent no NPC props on the
same walk. Now a level that is still loaded as a neighbour keeps its scripts: leaving it parks its NPC
objects and `level.` variables (`ScriptWorld::park_level`), and walking back in resumes them without
`created`. A level dropped from the neighbourhood, or any level after leaving the gmap, starts afresh.
`playerenters` still fires on each entry (not captured either way). Timeouts due while parked fire once
on return. Test: `re_entering_a_loaded_gmap_level_resumes_its_scripts`.

The NPC props side does the same (live rerun, 2026-10-30: with the scripts kept, `GANI idle` and `BODY`
still went out on each crossing back, because the sync forgot what it had sent and replayed the
server's older stored records). `ServerNpcSync::switch_level` parks what the server has for a gmap
level's NPCs and brings it back on re-entry, and the stored records are not replayed into a level whose
NPCs are still linked. Test: `walking_back_into_a_gmap_level_sends_nothing_again`.

The second live rerun still showed `GANI idle` (and `BODY`) for the two character NPCs of `minipw_1_1`
before every crossing, in every direction. Their records come with that level's data when it is sent
as a neighbour, but `SessionState` filed them under the player's level and the game loop added them to
it, running their `created`. A record now belongs to the level being sent (the last `PLO_LEVELNAME`),
and only the player's level takes updates as they come; a neighbour's wait in `server_npcs` until it is
entered. Test: `a_neighbours_npcs_are_kept_for_the_neighbour`.

Still open: on the **first** entry of a level the 2.22 build sends `VISFLAGS` for each NPC and the props
its `created` sets (`BODY`), the 2.05 rule (`documentation/ledges-push-pull.md`). The real 2.22 sent no NPC
props at all on the same walk. Which props 2.22 sends, if any, is not captured, so this is not changed here.
