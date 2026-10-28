# Carrying, throwing and breaking objects

Derived from the 2.05 client and the 2.22 `arrays.dat` (the 2.05 sprite table was not compared; the 2.22 one is used). No addresses or code are quoted. Units are tiles (1 tile = 16 px) unless a value says pixels.

## What the player sees

1. The player lifts a bush, block or rock (`cut-and-lift.md`). The object is drawn **as its own sprite**, a 32x32 cut-out of `sprites.png` with transparent corners, above the head. The board tiles around the object (the grass behind a bush) are not part of it. The first version of Seihai blitted the four board tiles, which is why the grass showed.
2. A, Space (Seihai) or S **while carrying** throws it. In the 2.05 client both the sword key and the action key (A) throw what is carried; the sword key does not swing then. The weapon key (D) does not throw. There is no separate "put down". See `cut-and-lift.md`, "Sword while carrying".
3. The object flies for up to 9 ticks in the facing direction with a shadow on the ground under it.
4. It lands when the ticks run out, or earlier on a wall. A bush, block or rock then **breaks into pieces that fly apart and vanish**; over water it makes a splash. A bomb stays on the ground.

stderr prints `thrown object N (facing D, flies up to 9 ticks)` when the key is accepted (N is the lift-table entry, the same N as `lifted object N`), `thrown object N: PLI_THROWCARRIED sent` online, and `thrown object N landed at x,y: break effect K` (or `stays on the ground`) when it comes down. The same code path runs offline and online; online adds the one send.

## Numbers

* **Carried sprite by lift entry** (bush, block, three heavier): sprite ids 261, 267, 265, 263, 239.
* **Where the carried object is drawn, and where a throw starts**: sprite top-left `(player x + 0.5, player y - 1.375)`. The ground point under it starts at `y = player y + 1.0` (the shadow's reference).
* **Per tick** `(dx, dy, dz)`, `dy` moves the drawn object and `dz` the ground point: north `(0, -0.8, -1.0)`, west `(-1, 0.2, 0)`, south `(0, 1.2, 1.0)`, east `(1, 0.2, 0)`. The object falls 0.2 tile per tick relative to its ground point in every direction. 9 ticks: 9 tiles along the throw.
* **Shadow**: sprite 0 (24x12) at `(x + 0.25, ground + 1.25)`.
* **Wall test** each tick at `(x + 1, ground + 1)`: a wall tile (type >= 20) or the board edge ends the flight, except tile type 20, which a thrown object flies through. **Water test** at landing at `(x + 1, y + 1)` (type 11).
* **Landing effect** (a "leap" in the client): anchored at `(object x, ground y)`; kind 5 (splash) on water, kind 0 (leaves) for sprite 261, kind 3 (pot pieces) for sprite 267, kind 2 (rock pieces) otherwise. One frame per tick, removed after the last: kind 0 has 7 frames, kinds 2 and 3 have 4, kind 5 has 8. Each frame is a list of pieces `(sprite, dx px, dy px)` from the anchor (array 19 of `arrays.dat`; the values for kinds 0, 2, 3, 5 are built in).
* **Bomb-like carried sprites** (ids 65, 66, 321, 347, 348) leave the object on the ground instead of an effect. Seihai has no bomb item yet, so this path is exercised only by the tests.
* **Sprite table** (array 24 of `arrays.dat`, 356 entries): each entry is one packed `u32`, `x = (v >> 24) * 2`, `y = ((v >> 12) & 0xfff) * 2`, `w = ((v >> 6) & 0x3f) * 2`, `h = (v & 0x3f) * 2`. Break-table pieces (array 19): sprite id in the high 16 bits, then two signed bytes, x and y offsets in pixels. The sheet rectangles Seihai uses without an `arrays.dat` are built in (`throwing::builtin_sprite_rect`).

| id | rect (x, y, w, h) | what |
|---|---|---|
| 0 | 0 0 24 12 | shadow |
| 25, 26 | 44 0 / 60 0, 16x16 | leaf bits |
| 96..99 | (24, 82) (40, 82) (24, 98) (40, 98), 16x16 | rock chunks |
| 100..103 | (56, 82) (72, 82) (56, 96) (72, 96), 16x14 | pot chunks |
| 326, 327 | (24, 114) (54, 114), 30x30 | splash |
| 261, 267, 265, 263 | (0, 338) (32, 338) (64, 338) (96, 338), 32x32 | carried: bush, block, stone, stone |
| 239 | (96, 370) 32x32 | carried boulder |

## Sound

The 2.05 client plays a crush sound on a break and a water sound on a splash. Seihai's default sounds have a `water.wav` but no crush sound, and no cue is wired for either; this is open.

## Open (not read, not invented)

* **Tick length.** The thrown object's own interval was not read. Seihai runs it on the 50 ms game clock, so a full flight is 450 ms.
* **Hits.** What a thrown object does to NPCs, players and baddies in flight (damage, stopping, the NPC's "touch" events) and the extra object checks in the client's wall test were not read; nothing is hit.
* **Order inside a tick** (step, then test) is inferred; a throw into a wall one tile away may land a tick earlier or later than the real one.
* **Carrying pose.** Whether the player has a different animation while carrying, and whether the carried object bobs with the walk, was not read; the object is drawn at its start position.
* **Telling other players.** Online, Seihai sends `PLI_THROWCARRIED` (11, empty, as the 2.05 client does). The lift itself (the carried-sprite player prop) is not sent, so other clients may not show the carry; and the server's reply `PLO_THROWCARRIED` for other players' throws is not drawn.
* **Board edit on break.** The real client may write tiles on a break (it does for a lift); none was seen.
* **The 2.05 sprite table** was not compared with the 2.22 one.
* The default art is original (`tools/gen_default_assets.py`), at the rectangles above; a user's own `sprites.png` of the real layout works because the same rectangles are used.

## Carry pose and other players

Read from the 2.05 client (derived).

* **Pose.** Lifting plays the `lift` gani; while an object is held the
  player shows `carrystill` standing and `carry` walking (built-in gani
  names 10, 11 and 13). On the wire (SPRITE, 17) these are animation steps
  23 (lift), 24 (carrystill) and 25 + frame (carry), which the 2.05 step
  table sends as S = 23, 18, and 19, 20, 19, 18, 21, 22, 21, 18. A throw
  sets the idle gani again.
* **CARRYSPRITE (19).** Sent as a GCHAR: -1 (byte 31) with empty hands; for
  a held object, 0 for sprite 65, 200 for 66, 201 for 239, otherwise the
  sprite id minus 260 (bush 261 is 1, stone 263 is 3, vase 265 is 5, sign
  267 is 7). The receiver maps it back the same way; a value whose sprite is
  not in the carry table (261, 267, 265, 263, 239, 65, 66, 321, 347, 348)
  becomes the bush. When a player's own CARRYSPRITE comes back as -1, the
  player stops carrying.

Seihai:

* Lift sends CARRYSPRITE with the carried sprite's code, then a walk update
  with S = 18. A throw sends `PLI_THROWCARRIED`, CARRYSPRITE -1 and a walk
  update with S = 0. Every walk update while carrying sends S = 18 (Seihai
  sends S = 0 for a plain walk too; the real client's walk and carry-walk
  steps are not sent). The lift pose (S = 23) is not shown.
* Another player's CARRYSPRITE is kept (`OtherPlayer::carry`) and its
  object drawn above that player at the same place as our own. Other
  players' bodies are still drawn in the idle pose.
* Our own player draws `carrystill.gani` standing and `carry.gani` walking
  (the walk's step) while carrying. The default set's art for them is our
  own (arms raised; `default-assets.md`).

