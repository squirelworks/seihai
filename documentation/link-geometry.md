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

## The model: one probe point per facing (2026-10-31)

Every build tests one probe point against each link: the player's position
plus an offset that depends on the facing. The point comes from a table of
four `(x, y)` doubles in the 2.05 client's link routine (derived):

| facing | probe |
|---|---|
| north | `x + 1.5`, `y + 1.0` |
| west | `x + 0.5`, `y + 2.0` |
| south | `x + 1.5`, `y + 3.0` |
| east | `x + 2.5`, `y + 2.0` |

For a link that is not a big-map edge link, the probe fires when it is in the
link's rectangle, with the far edges included (`x <= probe x <= x + width`,
the same for y). A probe within one tile of a level edge (x or y at most 1, or at
least 63) counts only while the player faces out across that edge.

The same 64 bytes of the table are in the Graal.exe of 9th and 15th Graal,
1.1, 1.1.9, 1.3b, 1.21, 1.38, 1.39, 1.41.1, 2.00, 2.04, 2.05, 2.17, 2.22 and
2.31. Only 2.05's link routine was read. The other builds are backed by
captures, not by reading their routines:

* Every 2.22 and 2.31 probe in the tables above, with the far edge included.
  That covers the down probes at x 8.0 (probe x 9.5, no warp) and 10.5
  (probe x 12, on the link's far edge, warp). The one exception is the
  2.31 run listed above.
* The lab `new1.nw` west link (`LINK testworld.nw 0 28 1 6`), walked west
  from x 6 or 8 (lab, 2026-10-31):

  | client | y 25.5 | y 26 to 32 | y 32.5 | y 33 |
  |---|---|---|---|---|
  | real 2.00 | stops at x -0.5 | warps after x 1 | stops | |
  | real 2.22 | | warps (31.5, 32) after x 1, at 0.5 | stops | stops |
  | Seihai 2.00 (#273) | stops | warps at x 0.5 | stops | |

  Until 2026-10-31, Seihai gave 2.22 and 2.31 boxes fitted to the 2026-10-25
  captures (`LINK_BOX`). The west box's top edge was at `y + 2`, so at y 32
  it only touched the link's bottom edge (34) and missed the warp that real
  2.22 takes there. The probe point fits every capture, so the boxes are gone.

Builds before that used one point, the feet (`x + 2.0`, `y + 2.75`), in the
link's tile. That point can never reach column 0: the edge wall stops the
player at x -0.5, where the feet are in column 1. So a 2.00 player walked into
the west link above and stopped (#273).

`offline::LINK_PROBE` and `offline::probe_in_link` implement this, and the
tests in `offline` cover every probe above.

## Before and after (lab, real client against Seihai)

`linkgeom2` (22 probes per version, 2.22 and 2.31): a probe counts as agreeing when the real
client and Seihai either both send the level warp for `lv_c.nw` or both do not.

| Seihai | probes that agree with the real client | probes where Seihai warps |
|---|---|---|
| before (foot point) | 16 of 22 | 11 (real client: 15) |
| after (direction boxes) | 22 of 22 | 15 |

Same result on 2.22 and 2.31.

`door_warp` on 2.22: the real client sends `LEVELWARP lv_c.nw 60 30`, then `LEVELWARPMOD lv_a.nw 14 30` on the
way back, then `LEVELWARP lv_w.nw` at the edge. Before, Seihai sent only the edge warp (the diff listed the `LEVELWARPMOD`
as a missing id, a high divergence, and the `lv_c.nw` warp as a field mismatch against the edge warp). After, Seihai sends the
`lv_c.nw` warp with the same `x=60`, `y=30`, the `LEVELWARPMOD` back at `x=14`, `y=30`, and the edge warp, and
the report has no missing level warp. The edge warp's `x` differs (real 17.5, Seihai 20.5) because the two clients
walk at different speeds (divergence 9).

## Edge links to the level next door (big map)

Read from the 2.05 client (derived; Graal_analysis
`client-internals/decomp-findings-2026-10-28/03-edge-links.md`). Not run
against a real client.

* On entering a level the client looks it up in the big map (the map
  screen's grid of quoted level names, `bigmap.txt`) and keeps the eight
  levels around it.
* A link that is one tile wide on the west (x 0) or east (x 63) edge, or
  one tile high on the north (y 0) or south (y 63) edge, and whose
  destination is the big-map neighbour on that side, arrives at the
  player's position moved by 64 tiles across the seam; the other
  coordinate is kept, and the link's own new-x and new-y are not used.
  East off x 63.75 lands at x -0.25.
* Every other link, and every link when there is no big map, arrives at
  its new-x and new-y (`playerx` / `playery` keep the coordinate). So an
  east-edge link written `playerx` puts the player on the east edge of the
  destination, in the real client too, unless the two levels are next to
  each other on the big map.

Seihai (`offline::edge_link_arrival`, `OfflineSession::big_map`,
`gmap::Gmap::parse_grid`):

* Online the big map is the file the server names in `PLO_BIGMAP` (171);
  Seihai asks for it once and reads it when it arrives. Offline,
  `--bigmap FILE` gives it. Without one, nothing changes.
* The real client fires a neighbour edge link once the player has crossed
  the edge (when the neighbour level is loaded); Seihai still fires it on
  the probe point above, a little earlier, so the arrival is held to
  where the real crossing would land: at least -0.5 going east, at most
  61.5 going west, at most 61 going north, at least -1 going south (a
  Seihai choice).
* Level names are compared without case; the real client's compare of the
  link destination with the neighbour name is exact (not changed here, open
  whether it matters).
* **Asked for ahead (laundry list P7, 2026-10-29).** Online, when the
  player's body (sprite position plus 1.5, 2) is within 10 tiles of an edge,
  Seihai asks the server for the big-map neighbour on that side (and the
  corner one when two sides are near), once per name per connection, as a
  level file (`files.need`, the same request as for a level it does not
  have). The file lands in the cache, so the link loads it at once instead of
  drawing the server's board first. Before, the neighbour was asked for only
  after the link was walked. The real 2.05 client keeps the eight levels
  around it (above); how and when it fetches them online is not traced, so
  the 10-tile distance is Seihai's.
  Lab check (GServer-v2 at 127.0.0.1:14802, 2.22, `--bigmap` with
  `giftest.nw` west of `new1.nw`, player at x 56): `new1.nw` arrived and was
  saved without the player moving.
* Open: walking past the edge into a loaded neighbour, the probe offsets,
  and whether 2.22 and 2.31 use the `PLO_BIGMAP` grid the same way.
* `new3.nw` (the laundry list's fixture) has no links, so it cannot show
  this on its own; it needs a big map naming it and links to its
  neighbours.

## Off the board without tile types (2026-10-30)

A Windows lab run with `--no-builtin` (no tile types, nothing cached) walked
Seihai off the server's `new1.nw` to y 75. Seihai only treated off the board
as a wall through the tile-type test, so without tile types the edge was open.
The 2.22 client's wall test checks the bounds before the tile and calls off
the board a wall (Graal_analysis
`client-internals/decomp-findings-2026-10-28-b/01-tile-type-tests.md`), so the
edge stops the player whatever the tiles are. Seihai now does the same when it
has no tile types; on a gmap a loaded neighbour is still open ground.

The edge links were not at fault. The lab level has one link, on the west
edge; a test with that link as the server sends it (`PLO_LEVELLINK`), no tile
types and the link probe shows it fires and arrives at the link's own x.
The other edges have no links, so walking off them should not warp. Whether
the run itself crossed the west edge is not in its log.
