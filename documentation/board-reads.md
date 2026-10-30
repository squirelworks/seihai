# `board[]` and `tiles[x,y]` reads (laundry item 11)

Scripts read the level's tiles with `board[i]` (every era) and, from 2.1, with
`tiles[x,y]`. Seihai used to return 0 for both. The code is
`scripting/engine/interp.rs` (`board_get`), filled from the session's current
level in `host.rs` (`sync_player`, before every event); the tests are
`crates/seihai-core/tests/board_reads.rs`.

## What the real client does

Read from the expression evaluator of the 2.05 rev 2 client (`board`, expression
100) and of the 2.1 test 3 client (`board` 100, and `tiles` 131 with the
signature `[R,R]`). Only the rule is written here, no code.

| read | rule | Seihai |
|---|---|---|
| `board[i]` | `i` becomes `floor(i + 0.0001)`. With a level loaded and 0 <= i < 4096, the value is the level's 16-bit tile word at `i`, so `i = x + 64*y`, layer 0. Otherwise 0. | same |
| `tiles[x,y]` | `x` and `y` each become `floor(v + 0.0001)`. With a level loaded and both in 0..63, the tile at `(x, y)`. Off the level, tile 632. With no level, 0. | same |
| `tiles` before 2.1 | not in the 2.05 expression table, so it is an ordinary variable | an ordinary variable for 2.05 and earlier |

The value is the tile id the level file holds (the same id `Level::tile_id_at`
gives and the renderer draws), so scripts that split it into a column and a
row with `b % 16` and `int(b / 16)` see the real numbers.

The board the scripts read is the board as it is now: a cut bush, a lifted
object or a server board change (`PLO_BOARDMODIFY`) shows in the next read.

## Tests

- `new3.nw` (fixture): `board[21+64*10]` is 158, `board[23+64*10]` 159,
  `board[20+64*10]` 0; `tiles[21,10]` the same under 2.22.
- Rounding: `board[660.99995]` reads tile 661, `board[660.9]` tile 660.
- `board[-1]`, `board[4096]` and both reads with no level are 0;
  `tiles[64,10]` and `tiles[21,-1]` are 632.
- Under 2.05, `tiles[21,10]` is a plain variable (0, then what was stored).
- **A real script, unchanged:** the Shovel weapon from the NPC library
  (`npc-1823`). It reads the 2x2 tiles in front of the player with `board[]`
  and puts a hole only when all four are column 15 of rows 31, 63 or 127. On
  `new3.nw` grass it does nothing; after a board change makes the four tiles
  511 it emits `putobject Hole,32,30`; with one tile back to grass it does
  nothing again.

## Not done

- **On a gmap,** the 2.1 client subtracts the level's map origin (times 64)
  from the `tiles` coordinates and, when the point is on another level of the
  map, reads that level's tile (632 if none is loaded there). Seihai reads the
  current level only, so `tiles[]` past its edge gives 632 on a gmap too.
- A second condition on the level (a flag the 2.1 client checks before the
  0..63 test) was not identified; Seihai does not model it.
- **Writes** (`board[i] = t`, then `updateboard`) are still ignored; this item
  covers reads only.
- 2.22 and 2.31 were not read; Seihai uses the 2.1 test 3 rule for every 2.1+
  build.
- Not run against a real client in the lab.
