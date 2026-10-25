# Sitting on benches

User report (2026-10-25): benches have a special GANI in which the character
sits; standing on a bench-like tile should draw it.

## What is known

* The scripting reference lists tile types 3 (chair), 4 (bed top) and 5 (bed
  bottom). The built-in animation names include `sit` and `sleep`; the sprite
  numbers are 37 (sit) and 38 (sleep) (Graal_analysis `11-emotes-ganis.md`).
* In the 2.22 `arrays.dat`, array 4 holds 40 tile ids in 2 x 2 and 2 x 3
  groups (the wooden plank seat, benches, chairs, couches); every one of them is
  also in the passable list (array 0). Array 5 (4 ids) and array 6 (4 ids) are the
  two halves of a bed: the top is not passable, the bottom is. The mapping of
  arrays 4 to 6 onto types 3 to 5 is an inference from that order and those
  passability patterns; it was checked against the tile art (plank seats, bed
  pillows).

## What Seihai does

* `TileTypes::parse_arrays` now reads seven arrays instead of four and gives
  every passable tile of array 4 the type byte 3 (`TYPE_CHAIR`). Walking is
  unchanged (type 3 is open). A text tiledefs file with type 3 does the same.
* `OfflineSession::refresh_water` (run after every step, warp and tile-type
  load) also sets `sitting` when the feet tile (the foot point, x + 2.0,
  y + 2.75) is a chair tile.
* `pose::current_pose` draws `sit.gani` step 0 while `sitting` and the player is
  not walking (the walk animation wins while an arrow key is held), after the
  sword and hurt poses and never while swimming.
* The GANI is the normal `sit.gani` from the asset resolver (a server's, if it
  has one, reloaded when it downloads, or the built-in original in `assets/default/gani/sit.gani`).

## Open (not read from the real client)

* The exact trigger. Everything above is the likeliest reading (feet on a type-3
  tile); whether the real client sits only after a pause, faces the player a
  particular way, or snaps the player to the seat is not known. Lab capture or a
  decompile read of the sit path would settle it.
* What other players are sent while sitting (the sprite value 37 or a GANI
  property). Seihai draws the pose locally and sends nothing new.
* Beds: arrays 5 and 6 and the `sleep` pose are not implemented (the default
  assets have no `sleep.gani`).
