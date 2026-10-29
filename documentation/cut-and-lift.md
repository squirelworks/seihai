# Cutting and lifting

What the client does is in `arrays.dat`, not in the level: the same file that
lists the passable tiles. Its arrays are, in file order (each is a big-endian
`u32` header, low 16 bits the count, high 16 bits 0 for a flat list of ints
and non-zero for a list of nested arrays):

| array | holds |
|-------|-------|
| 0 | passable tiles (`nowall`) |
| 1 | water and lava |
| 3 | lava |
| 9 | cuttable 2 x 2 objects (bush types) |
| 10 | the 2 x 2 tiles a cut leaves |
| 11 | liftable 2 x 2 objects: bush, a 512 block, a 684 block, the rock, a 990 block |
| 12 | the tiles a lift leaves |
| 13, 14 | closed and opened chest tiles |

An object is `[column][row]`. The 2.22 file's tables: 2 cuttable types (the
bush `2 3 / 18 19` and `420 421 / 436 437`) and 5 liftable ones (the rock is
`34 35 / 50 51`, entry 3).

## The rules read from the 2.05 client

* **Probe point.** For both, the point is one tile beyond the middle of the
  body box's leading edge: north `(x + 1.5, y)`, west `(x - 0.5, y + 2)`,
  south `(x + 1.5, y + 4)`, east `(x + 3.5, y + 2)`.
* **Match.** Take the tile under the point. For each table entry in order,
  for each of its four tiles that equals it, the object's top-left is the
  point minus that tile's column and row; all four board tiles must equal
  the entry's four, on the board. The first full match wins.
* **Cut.** As the sword swing starts, match against the cut table (all
  entries, no glove) and write the entry's "left" tiles over the four, then a
  spawn routine takes the entry index (debris or items; not read).
* **Lift.** The A key tries this first (before a chest). Glove
  power `g` allows the first `g + 2` entries of arrays.dat array 11: power 0
  the bush and the sign (512), power 1 adds the pot (684), power 2 the light
  stone (34), power 3 the black stone (990) (the table order is the client's). The four tiles
  are replaced by the entry's "left" tiles and the player carries the object.

## What Seihai does

`ObjectTables::parse` reads arrays 9 to 12 (the nested layout above; it is
checked against the 2.22 file's values); `OfflineSession::try_cut` runs in
`start_swing` and `try_lift` runs first in `interact`; the board is edited
in place (`Level::find_object`, `put_object`). Pressing A with an object in
the hands puts it down. Only a client with an `arrays.dat` (the user's own
folder or a server's cache) has the tables: the built-in default set has none,
so nothing is cuttable or liftable there.

## The cut effect

A cut plays a break effect (the same rows as a landed throw, `throwing.md`) at
the object's top-left tile: `try_cut` pushes a `Leap` anchored there, drawn one
frame per 50 ms tick and gone after the last. Entry 0, the bush, plays row 0 of
the break table (seven frames of four leaf pieces, sprites 25 and 26, the
bush's leaves a thrown bush makes); entry 1 (the `420 421 / 436 437` object) plays row 1 (four
frames of three pieces of sprite 24). The row values are the 2.22 file's;
**that the cut entry number picks the break row is an inference** (the
client's spawn routine takes the entry index and was not read; the rows line up:
two cut entries, and rows 0 and 1 are the two that no throw uses). Where the
effect is anchored (the object's top-left versus its centre) is also inferred
from the piece offsets, which stay inside the 32 x 32 object for the bush.
The built-in default art has no sprite 24, so entry 1's effect is empty without
a real `sprites.png`.

## Sword while carrying

Read from the 2.05 client's key handler (derived, no lab capture yet). The
game keys are handled by index: weapon (D), sword (S) and action (A).

* **Sword key, carrying:** no swing. The carried object is thrown, exactly
  as the action key throws it (same flight, the carry sprite goes back to
  none, the idle gani is set). If the player was walking (or in the
  lay-down mode below), the player's mode goes back to idle. None of the
  swing's preconditions (sword power and so on) are checked on this path.
* **Sword key, empty hands:** the normal swing.
* **Action key, carrying:** throws (unless in the lay-down mode, which has
  its own action).
* **Weapon key, carrying:** does not throw. It only acts when the carried
  sprite is one of three (ids 239, 261, 263) and then starts a separate
  lay-down mode. Seihai does not implement that mode; D does nothing while
  carrying.

Seihai: `start_swing` refuses while something is carried, and the same S
press goes on to the interact, which throws. Online the throw sends
PLI_THROWCARRIED as for A. Not done: the mode reset to idle while walking
(Seihai's pose follows the arrow keys each frame, so a held arrow key keeps
the walk pose after the throw; the real client may show one idle frame
first).

Checked only in 2.05. 2.22 and 2.31 were not read and not captured; Seihai
uses the 2.05 rule for every build until they are.

## Open (not read, not invented)

* Whether the cut also makes a sound, and the pieces' exact timing against the
  real client; a lab capture of a cut would settle both. Other cut-table
  effects (drops) are below.

* **Throwing** is implemented (see `throwing.md`); what a thrown object
  does to NPCs, players and baddies it hits was not read.
* **What a cut or a lift drops** (rupees, hearts, bombs): the spawn routine
  takes the entry index and was not read, so nothing drops.
* **Carrying pose and what others see**: see `throwing.md`, "Carry pose
  and other players".
* **Telling the server.** The client writes the changed tiles into the level
  for everyone (a board-modify the client sends). Seihai edits only its own
  copy, so other players do not see the bush vanish and a re-entry restores
  it.
* **Glove power.** 2.05 starts every player at 1, so pots lift without a
  glove item; `glove1` raises it to 2 and `glove2` to 3 (2.05's item pickup,
  and GServer's `GLOVEP 1` default and item code). Online Seihai takes the
  server's GLOVEPOWER (player prop 6). Seihai used to start at 0, which is
  why the pots in houses would not lift.
* Grabbing, pulling and pushing follow the 2.05 client: A on any wall
  (a rock the glove cannot lift is one) holds it while A is down, the arrow
  away from it pulls, and stone blocks move. See `ledges-push-pull.md`.
