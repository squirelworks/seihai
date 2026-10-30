# Tile types

How the 2.05 client decides what a tile is, and what Seihai does with it.
From the 2.05 decompile (the `arrays.dat` loader, the per-type test, the
wall test and the script functions `tiletype`, `onwall` and `onwater`).

## Two kinds of tileset

The client picks a tileset per level from its tile definitions
(`addtiledef`). Each definition has a type:

- **Type 0, the default** (`pics1.png`). Tile types come from the lists in
  `arrays.dat`. This is the only kind Seihai draws today.
- **Types 1 and 2, the new kind.** Tile types follow from where the tile
  sits on the sheet, with no file. `new_world_tile_type` in
  `client/src/assets/level.rs` is that rule. Seihai does not yet switch to
  this rule when a level uses such a tileset, because `addtiledef` is not
  wired.
- **Type 5.** It has no tile lists at all. Only NPC shapes have types there.

## The default tileset: one list per type

`arrays.dat` is read as 25 arrays (see the Graal_analysis note on the
format). 2.05 never builds a 4096-entry type table for the default
tileset. Each question asks one list:

| question | array | type number |
|---|---|---|
| wall (`onwall`, walking) | not in 0 and not in 1 | 22 |
| chair | 4 | 3 |
| bed top | 5 | 4 |
| bed bottom | 6 | 5 |
| water (`onwater`, swimming) | 1, the liquid list | 11 |
| lava | 3 | 12 |
| thrown objects fly over | 2 | 20 |
| ledge or jumping stone | 8 (one list per direction) | 21 |

Any other type number (0, 2, 6 to 10) is only answered by NPC shapes
(`setshape2`), never by a board tile. Array 7 is not a type list: the
position of a tile in it (0–11, 12–19, 20–62, 63 and up) gives one of four
ground classes. The horse uses these to pick the sprite drawn under it.

A tile off the board reads as tile 632, which is in no list. So off the
board `onwall` is true (that test bounds-checks first) and `onwater` is
false.

Two things follow from the lists:

- **Lava is water to the water test.** The liquid list holds both water and
  lava, so `onwater` and the swim test are true on lava too. A separate
  lava test counts the time spent there. When the count passes 100 ticks,
  the client calls the routine that appears to hurt the player (with what
  looks like -0.5), and the count starts again. This reading is inferred.
  One effect that splashes on liquid uses a different sprite on lava.
- **A tile can be in two lists.** In the 2.22 file, 6 of the 133
  throw-through tiles are also passable, and 9 lava tiles are not in the
  liquid list. All 9 are passable, so they are walked on and still burn.
  (An earlier version of this page called them walls.)

## What Seihai does

`TileTypes::parse_arrays` still keeps one type byte per tile. Walking comes
first, so the byte holds every case above that changes play:

- passable (array 0) gives 0, and liquid (array 1) gives 11, or 12 if the
  tile is also in array 3. Every other tile is 22. This is the same as
  before.
- **New:** a wall that is in array 2 becomes 20. Thrown objects already
  flew over type 20, but `arrays.dat` never produced it, so until now they
  stopped at fences and low walls.
- chairs (array 4) give 3, as before. **New:** bed tops (array 5) give 4
  and bed bottoms (array 6) give 5. These apply only to passable tiles,
  and nothing reads the bed types yet.
- arrays 7 and 8 are not read here. Ledges belong to the cliff work.
- **New:** a passable tile in the lava list (array 3) but not in the
  liquid list gives 13, Seihai's own number for lava floor: walked on, and
  it burns. `TileTypes::is_lava` is true for 12 and 13.

## Deep water and lava (`new4.nw`)

Squirrel's test level `client/tests/fixtures/levels/new4.nw` has each kind.
With the lists of the 2.22 `arrays.dat`:

| tile | lists | Seihai type | what happens |
|---|---|---|---|
| 398, light blue | passable | 0 | shallow water: walked on |
| 461, 472, water edge | passable, liquid | 11 | swum in |
| 3644, dark blue | none | 22 | deep water: a wall |
| 2882, bright red | liquid, lava | 12 | swum in, burns |
| 2958, red | passable, lava | 13 | walked on, burns |

So deep water needs no rule of its own: it is a wall because the 2.22 file
lists it nowhere. The built-in `tiledefs.txt` is this same 2.22 table, so these
tiles behave the same with the built-in set (`default-assets.md`).

- **Swimming:** the swim test is true on water and lava (the liquid list).
  `refresh_water` now swims on `TileKind::Lava` too.
- **Burning:** `OfflineSession::tick_lava`, run from `tick_life`, keeps the
  client's count on 50 ms ticks. Each tick it tests the point (x + 1.5,
  y + 2) against the lava list. On lava the count goes up, and the 101st
  tick in a row takes half a heart and sets the count to 0. A tick off lava
  sets it to 0. That is a little over 5 seconds a half heart.

Script functions:

- `onwater(x, y)`: true on any liquid tile, lava included, and false off
  the board.
- `tiletype(x, y)`: -1, because 2.05 returns -1 on the default tileset. It
  answers only for tilesets of the new kind.

## Not done

- On lava the client shows its `lava` gani rather than `swim`; Seihai
  draws the swim gani. The burn takes the hearts but plays no hurt effect.
  The client's 50 ms player tick for the burn is assumed. Online, the burn
  changes Seihai's hearts as any local hit does.
- `addtiledef` and the new-kind rule are not wired.
- The bed and ground-class lists have no consumer.
- 2.00 to 2.04 were not checked. All of them do have the strings
  `tiletype`, `onwater` and `onwall`.
