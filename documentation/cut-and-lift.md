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
  entries, no glove) and write the entry's "left" tiles over the four.
* **Lift.** The A key tries this first (before a sign or a chest). Glove
  power `g` allows the first `g + 2` entries: with no glove the bush and the
  512 block, power 1 adds the 684 block, power 2 the rock and the 990 block (the
  table order is the client's). The four tiles
  are replaced by the entry's "left" tiles and the player carries the object.

## What Seihai does

`ObjectTables::parse` reads arrays 9 to 12 (the nested layout above; it is
checked against the 2.22 file's values); `OfflineSession::try_cut` runs in
`start_swing` and `try_lift` runs first in `interact`; the board is edited
in place (`Level::find_object`, `put_object`). Pressing A with an object in
the hands puts it down. Only a client with an `arrays.dat` (the user's own
folder or a server's cache) has the tables: the built-in default set has none,
so nothing is cuttable or liftable there.

## Open (not read, not invented)

* **Throwing** is implemented (see `throwing.md`); what a thrown object
  does to NPCs, players and baddies it hits was not read.
* **What a cut or a lift drops** (rupees, hearts, bombs): the spawn routine
  takes the entry index and was not read, so nothing drops.
* **The carrying gani** (the walk-while-carrying pose) was not read. The
  carried object is drawn as its own sprite (not the board tiles) above the
  head; see `throwing.md`.
* **Telling the server.** The client writes the changed tiles into the level
  for everyone (a board-modify the client sends). Seihai edits only its own
  copy, so other players do not see the bush vanish and a re-entry restores
  it.
* The glove power values come from items (`glove1` = 1, `glove2` = 2 in
  Seihai); how the server's player props carry it online is not wired.
* Pulling and pushing blocks (the grab action, the second thing A does in the
  client) is not implemented.
