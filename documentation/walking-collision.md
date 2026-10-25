# Walking collision: the real client's rule

Reported twice on public servers:

* The player stood on rocks and bushes (fixed in #76: a body resting against a
  wall by its right or bottom edge was counted as "inside" it).
* On the 2.22 profile: "stuck in a spot that should have let me through; the
  client reads too far ahead and blocks me from walking onto grass next to an
  object" (user report, 2026-10-25).

The first report came from a one-point wall test. The second came from copying
only half of the real client's walking routine. This note is the whole of it, as
read from the 2.05, 2.17.1, 2.22 and 2.31 clients. It is the same routine in
all four (same tables, same branches, same constants). Derived facts only; no
addresses (those are kept off the public repos).

## The rule in six lines

1. Each arrow key is its own step, in the order up, left, down, right: half a
   tile (times a speed factor, 1.0 on foot), tested at the position the earlier
   keys left.
2. Six probe points per facing sit on the leading edge of the body box
   (`x + 0.5 .. x + 2.5`, `y + 1 .. y + 3`), each read as the tile
   `floor(coordinate + 0.0001)`. The step is kept if the first four are free.
3. If it is refused, a corner nudge may move the player 0.1 tile along the edge
   (see below), then the step is taken back.
4. Then the position along the facing is pulled flush to the tile line (up to
   `floor(y)`, left to `floor(x + 0.5) - 0.5`, down to `floor(y + 1)`, right to
   `floor(x + 1.5) - 0.5`), if it is not on it already.
5. After every step, accepted or not, the position snaps to a tile line it is
   within 0.0625 of (y on whole numbers, x + 0.5 on whole numbers).
6. A point is blocked by a chest record (2 x 2 tiles, edges included), a
   blocking NPC or other sprite, or a tile that is not in the passable lists.

## Probe points

Offsets from the sprite's top-left, taken at the position the step would
reach. The table is the same in all four clients.

| facing | the six points |
|--------|----------------|
| up    | y + 1.0; x + 0.525, 1.475, 1.525, 2.475, 2.525, 0.475 |
| left  | x + 0.5; y + 1.025, 1.975, 2.025, 2.975, 3.025, 0.975 |
| down  | y + 3.0; x + 0.525, 1.475, 1.525, 2.475, 2.525, 0.475 |
| right | x + 2.5; y + 1.025, 1.975, 2.025, 2.975, 3.025, 0.975 |

The points are numbered 0 to 5 in that order. Points 0 to 3 decide the step.
Points 4 and 5 only steer the nudge. A coordinate is floored as
`floor(v + 0.0001)`, so a point exactly on a tile line reads the tile that
starts there, and one a hair under a line (rounding) reads the tile above it.

Note where the body is: `x + 2.5` and `y + 3` are the right and bottom edges of
the body, but the probe sits exactly on that line, which reads the tile one
beyond the body. The body therefore cannot step so that its right or bottom edge
touches the next tile line while that tile is a wall. The snap in rule 4 closes
that last half tile.

## The corner nudge

Only on a refused step, using the probes of the step just tried:

* Point 0 blocked and points 2, 3 and 4 free: the position is moved +0.1 across
  the edge (x for up and down, y for left and right).
* Otherwise, points 0 and 1 free, point 3 blocked, point 5 free: moved -0.1.

The nudged position is then used for the rest of the step (the step itself is
still taken back). Pressing against a corner repeats this on every key tick, so
the body slides round it at 0.1 tile per tick (50 ms) until the blocked probe
clears, instead of stopping on the corner. Only a block at one end of the
edge nudges: a rock straight ahead (point 1 or 2 blocked, or point 0 and 3
blocked together) does not, and the body stops.

## The snaps

After a refused step (nudge applied, step taken back), only along the facing:

| facing | when | new position |
|--------|------|--------------|
| up    | fraction of y > 0          | y = floor(y) |
| left  | fraction of x + 0.5 > 0    | x = floor(x + 0.5) - 0.5 |
| down  | fraction of y > 0          | y = floor(y + 1) |
| right | fraction of x + 0.5 > 0    | x = floor(x + 1.5) - 0.5 |

so the body ends with its edge exactly on the wall's tile line (body edge `x +
2.5` or `y + 3` on a whole number for right and down; `x + 0.5` or `y + 1` on a
whole number for left and up). Half a tile short of a wall to the right or
below, a press into the wall moves the body that half tile.

Then, for every step: `y` snaps to `floor(y + 0.0625)` if its fraction is under
0.0625 or over 0.9375, and `x` snaps to `floor(x + 0.5 + 0.0625) - 0.5` if the
fraction of `x + 0.5` is. That ends a run of 0.1 nudges on the tile line and
removes rounding.

Neither snap is tested against the walls. They do not move a body into a wall:
they only move it up to the tile line the refused probe was on.

## What was wrong in Seihai

Before this change Seihai tested the four decisive points and stopped. So:

* **Half a tile short of a wall.** The right and bottom probes sit on the tile
  line beyond the body. A body whose right edge was half a tile from a wall
  could not take the step that would put the edge on the wall's line, and stayed
  half a tile short. The real client takes that step because it snaps there.
  With a rock (or a tree, or a sign) at the right or below, the body could never
  reach the last half tile, which blocked walking onto the grass beside it: the
  reported "blocks too far ahead". Left and up were flush already because the
  probe is inside the body there.
* **No corner nudge.** A body whose edge clipped the end of a wall by less than
  a tile stopped dead; the real one slides round it. This is the "stuck where it
  should let me through" part.
* **Chest edges.** A chest covers its edges (`x .. x + 2`, `y .. y + 2`
  inclusive); Seihai used half-open intervals, so a body could stand half a tile
  into a chest from below or the right.

## What Seihai does now

* `WALK_PROBES` holds all six points per facing and `WALK_NUDGE` the nudges.
  `OfflineSession::walk_step` is one arrow key of the routine above;
  `try_move` applies the keys in order and keeps positions to 0.001 tile (so
  repeated 0.1 nudges do not drift off a half tile).
* The tile of a point is `floor(coordinate + 0.0001)`; off the board is a wall
  (or the neighbouring level in a gmap).
* Chests block on a closed 2 x 2 box. NPC blocking and the other sprite lists
  are not part of the walking test yet (see below).
* The #76 rule is kept: a body that a warp put inside a wall may still step to
  a place that is less inside it, with no nudge or snap. The real client has no
  such rule: it tests the leading edge only, so it lets a body in a wall step
  wherever the leading edge is free. Seihai keeps the stricter #76 rule on top.
* A change of position by nudge or snap counts as a move (a position update goes
  out, the walk GANI steps). The real client sets its position the same way
  after a refused step.
* The feet point (`foot()`) stays for links, water and scripts. `facing_point()`
  is as before.

## Positions on the wire

The routine works in doubles. A nudged position is not a half tile. The
classic frame carries x * 2 as a byte, which truncates (10.1 goes out as 10.0);
the gen-5 X2/Y2 carries sixteenths (10.1 as 10.0625). Seihai keeps the exact
value locally and sends what the encoders give.

## Confidence

| part | confidence |
|------|------------|
| six probe points, step order, keep-if-first-four-free | high: same table and branch in four clients |
| tile read is `floor(v + 0.0001)` | high |
| nudge conditions and amounts | high |
| flush snap per facing | high for the arithmetic; medium that it is meant as "close to the wall" (it only fires on a refused step) |
| 0.0625 snap | high |
| chest box (2 x 2, edges included) | high |
| the sprite and NPC lists in the blocking test | medium: found, not decoded |

Read from the compiled clients (no lab run). Checks against a running 2.22
client of the stopping positions and the slide would still be welcome.

## Not done / unverified

* The blocking test asks four lists in the level before it reads the tile: a
  list of objects with two flags and a body box like the player's (checked only
  when a client option is on), the level's NPCs (those with a blocking shape
  or image), a list of 1.5 x 2 boxes with an enabled flag, and the chests. Only
  the chests are in Seihai's walk. What the first and third lists are (other
  players? signs? baddies?) was not identified.
* NPC `setshape` blocking is not applied to walking.
* Swimming, the fast speed (1.2), the slow speed (0.4 and 0.2) and carrying
  change the step length. Seihai keeps its own step sizes and timing.
* A nudge that lands off a half tile makes the position non-half-tile for a
  few ticks; the classic wire cannot show that.
* What the real game does at a gmap seam when the neighbour is missing: a
  missing level reads as a wall here, as before.
* Pushing blocks (the state after a refused step) has its own probes; not
  implemented.
