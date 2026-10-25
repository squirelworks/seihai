# Walking collision: the body box

Reported on a public server: the player stood on rocks and bushes. The tile
types were right (rocks and bushes are not in the passable list of
`arrays.dat`, so they are walls); the test was the problem. Seihai tested one
point, the feet at `(x + 2.0, y + 2.75)`. The sprite is about three tiles wide
with its middle at `x + 1.5`, so everything left of that point could overlap a
wall.

## What the 2.05 client does

Read from the client's walking routine (derived facts only).

* Each arrow key is handled on its own, in the order up, left, down, right.
  A step is half a tile (times a speed factor, 1.0 on foot).
* For the step, the client works out the new position and reads **six**
  points for the facing, offsets from the sprite's top-left:

  | facing | points |
  |--------|--------|
  | up    | y + 1.0; x + 0.525, 1.475, 1.525, 2.475 (then 2.525 and 0.475) |
  | left  | x + 0.5; y + 1.025, 1.975, 2.025, 2.975 (then 3.025 and 0.975) |
  | down  | y + 3.0; x + 0.525, 1.475, 1.525, 2.475 (then 2.525 and 0.475) |
  | right | x + 2.5; y + 1.025, 1.975, 2.025, 2.975 (then 3.025 and 0.975) |

  So the body is a box from `x + 0.5` to `x + 2.5` and from `y + 1` to
  `y + 3`, and the points lie on its leading edge.
* A point is blocked if a chest record covers it (2 x 2 tiles, edges
  included, opened or not), a solid object or another blocking sprite is
  there, or its tile is not in the passable lists (`arrays.dat` arrays 0 and
  1, or the type byte is 20 or more).
* The step is kept if the **first four** points are free. Otherwise the
  position is put back and the player is shown pushing against the wall.
* When the step is refused, two more rules nudge the player 0.1 tile along
  the edge so a corner does not catch: if only the first point is blocked and
  the third, fourth and fifth are free the player moves +0.1 across the edge
  (right, or down for a horizontal move); if the first and second are free,
  the fourth is blocked and the sixth is free it moves -0.1.

## What Seihai does now

* `WALK_PROBES` holds the four points per facing; a step is refused if any is
  a wall, off the board or inside a chest.
* The keys are applied one at a time in the client's order. A diagonal is two
  tested steps, so it slides along a wall. (The old rule that refused a
  diagonal between two touching walls is gone; the box cannot pass such a
  gap sideways anyway.)
* A player whose body box already overlaps a wall (put there by a warp) may
  walk out, as before.
* The feet point is kept for links, water and scripts (`touch`).
* `facing_point()` (what the A key probes) now uses the client's
  "one tile beyond the middle of the leading edge" table: up `(x + 1.5, y)`,
  left `(x - 0.5, y + 2)`, down `(x + 1.5, y + 4)`, right `(x + 3.5, y + 2)`.
  The client reads this table for lifting and for the sword's cut; chests and
  signs use it here by assumption. A player stopped by a chest or sign has
  the point inside it, as the tests check.

## Open

* The 0.1-tile corner nudge is not implemented. Positions on the wire are in
  half tiles, and a nudged position is not one. Nudging is also why the
  client slides around corners without stopping.
* The client snaps the position after a step when its fractional part is
  within 0.0625 of a tile edge; not read in detail, not done.
* Swimming and the faster speed (1.2) change the step length in the client;
  Seihai keeps its own timing.
* A wall under the left half of the body when walking up is now caught; what
  the real game does at the very top or left of a level (the body box leaves
  the board by up to a tile) is the same rule as inside it: a point off the
  board is a wall.
