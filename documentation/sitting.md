# Sitting on chairs and lying in bed

User reports: benches have a special GANI in which the character sits
(2026-10-25); the sit pose only worked on the top half of a chair, and beds
had no pose (2026-10-30).

## What the 2.05 rev 2 client does

Read from the 2.05 rev 2 executable (`Graal_v2.05_rev2.exe`, the Ghidra export
in the project's `graal-decompile/v205-delphi/` plus the disassembly of the
routine, since the decompiler lost its floating-point arguments). The tile
type lists are the ones in Graal_analysis
`client-internals/decomp-findings-2026-10-28-b/01-tile-type-tests.md`
(type 3 = `arrays.dat` array 4, type 4 = array 5, type 5 = array 6).

One routine decides both poses. Points are from the sprite's top-left (x, y),
in tiles:

1. **Chair.** If `(x + 1.5, y + 2)` is type 3, the client sets the `sit` gani
   and stops there. That point is the middle of the walking body box
   (x + 0.5 to x + 2.5, y + 1 to y + 3), the same point the lava test uses.
2. **Bed.** Otherwise, the left side `(x + 1, y)` or `(x + 1, y + 0.5)` must be
   type 4, and the right side `(x + 1.8, y)` or `(x + 1.8, y + 0.5)` too. Then
   the client sets the `sleep` gani and moves the head onto the bed's top row:
   when neither `(x + 1, y)` nor `(x + 1.8, y)` is type 4, y goes down by 0.5;
   then y becomes `floor(y + 0.0001)`. x is not changed.
3. Both poses are set only when the player is in the normal or walking state
   (state 0 or 1, so not swimming), not carrying anything and not on a horse.
4. Type 5 (bed bottom) is never asked here.

The routine runs on level entry, when an arrow key is let go, and in every
player tick where the player walked or the X or Y property was set (a script
or the server moved them). In the walking tick it runs after the walk code has
set the `walk` gani, so on a chair the sit pose replaces the walk while
walking.

The gani names are entries 20 (`sit`) and 21 (`sleep`) of the client's table of
built-in animation names (Graal_analysis `water-and-ganis.md`).

In the 2.22 `arrays.dat` the chair tiles (array 4) are all passable, the bed
tops (array 5) are not and the bed bottoms (array 6) are. The head row (y to
y + 1) is above the walking body box, so the head can lie on a bed top that is
a wall.

## What Seihai does

* `OfflineSession::refresh_rest` is the routine above. It runs at the end of
  `refresh_water`, which runs after every step, warp, arrival, landing, push,
  knockback and tile-type load. It sets `sitting` or `sleeping`. Swimming or
  carrying clears both. Seihai has no horse.
* `TileTypes` keeps the bed tops of `arrays.dat` in their own sorted list
  (`bed_tops`), because those tiles keep their wall type byte. A text tiledefs
  file lists them on a `bedtop:` line (the built-in `tiledefs.txt` has
  `bedtop: 708-711`), or marks an open bed top with type byte 4, as a tileset
  of the new kind does.
* `pose::current_pose` draws `sit.gani` or `sleep.gani` step 0 while the flag
  is set, walking or not, after the sword, hurt, grab, pull and carry poses.
* The default set has an original `sleep.gani` (from
  `tools/gen_default_assets.py`): only the head and hat, at the idle head
  offsets, since the bed tile is the blanket. A server's `sleep.gani` replaces
  it, as with every pose.
* Another player whose gani is `sleep` is drawn with the same file.

Consequences of the read code (not checked live):

* In a bed, a step up or down is snapped straight back to the top row, so the
  way out is to the side, until one of the two head probes is off the bed top.
* Lying down only needs the head; walking up into a bed from below, the body
  box stops below the bed top and the head lands on it.

## Open

* What other players are sent while sitting or sleeping (the gani property, or
  the 2.05 sprite values 37 and 38). Seihai draws the poses locally and sends
  nothing new.
* The real client sets the gani once and keeps it until something else sets
  one (a swing ends with `idle`, so a player who swings on a chair stands until
  they move). Seihai derives the pose from the flags, so it sits again when the
  swing ends.
* The client's tile test reads a point as the tile it falls in; whether it adds
  0.0001 first, as the wall test does, was not read. Seihai truncates.
