# Ledge jumps, pushing and pulling

Derived from the 2.05 rev 2 client and the 2.22 `arrays.dat`. No addresses or code are quoted. Units are tiles; a tick is the 50 ms game tick, and a walk step is one arrow key in one tick. Test level: `client/tests/fixtures/levels/new3.nw`. Code: `client/src/offline/pushpull.rs`.

## What the player sees

1. **Running into a wall** shows the push pose once the player has been stopped for 18 steps in a row (0.9 s). A free step, a corner nudge or letting go of the arrow ends it.
2. **Running into a ledge** (the cliff edges of `new3.nw`) jumps the player off it after the same 18 steps, with `jump.wav`. The jump takes 8 ticks along a fixed curve: 5 tiles up, 7 tiles down, or 6.5 tiles to the side and 3 down. The arrows do nothing in the air. No special gani plays: the walk pose stays.
3. **Running into a stone block** pushes it one tile after 29 blocked steps (about 1.5 s), and again every 20 steps after that. The player follows one tile and `stonemove.wav` plays.
4. **A on a wall** (nothing to lift, no chest or sign in front) grabs it. The grab holds while A is held. Holding the arrow away from the wall plays the pull pose. Every 18 ticks of pulling the client tries to pull what it holds: a pullable stone block comes one tile and the player steps back with it (`lift.wav`); anything else stays where it is and the pull pose simply goes on. Letting go of the arrow goes back to the grab pose; letting go of A ends the grab.

The old Seihai behaviour, for the record: every solid tile was a plain wall, and A on a rock the glove could not lift showed the first sword step for 400 ms. That was the "sword swing that sticks" in the laundry list.

## The ledge test

The client counts blocked steps. A step counts when it is refused and no corner nudge was applied; a free step or a nudge sets the count back to 0, and so does the first refused step after a free one.

* At count 18 the ledge test runs. At 38 the count goes back to 18, so it runs again every 20 steps.
* Test point, from the player's position, by facing (up, left, down, right): `(1.5, 0.5)`, `(0, 2)`, `(1.5, 3.5)`, `(3, 2)`. The tile there must be a ledge tile **for that facing**. In the usual tile mode those are the four lists of `arrays.dat` array 8 (2.05 calls the class tile type 21, "jumping stone"):

| facing | ledge tiles (2.22 `arrays.dat`) |
|---|---|
| up | 190, 191, 4045, 4046 |
| left | 212, 228, 563, 579, 4028, 4044 |
| down | 185, 186, 4061, 4062 |
| right | 273, 289, 4031, 4047 |

* Landing point, from `(x + 1.5, y + 2)`: `(0, -5)`, `(-6.5, 3)`, `(0, 7)`, `(6.5, 3)`. It must pass the ordinary walking test (not a wall, not a chest). If it does not, the player just keeps pushing.
* The jump starts from the position after the refused step. Per tick `i = 0..7` (the first in the same tick as the start):

| facing | position |
|---|---|
| up | `y0 - [1.5, 2.7, 3.8, 4.5, 5.0, 5.2, 5.2, 5.0][i]` |
| down | `y0 + [-0.2, 0.2, 0.5, 1.5, 2.8, 4.0, 5.4, 7.0][i]` |
| left / right | `x0 -/+ [1, 2, 3, 3.8, 4.5, 5.2, 5.8, 6.5][i]`, `y0 + [-0.2, -0.5, 0, 0.5, 1.1, 1.7, 2.3, 3.0][i]` |

* `sounds/jump.wav` plays at `(x + 1.5, y + 2)` when it starts. Keys are not read until it lands.

In `new3.nw` (decoded with the 2.22 lists): the up ledges are the cliff edge at y = 14 and 15 (x 18 to 46); the left ledges are the two slanted cliff sides, x 14 to 16 and x 48 to 56, between y = 27 and 50. There are no down or right ledges. The push block is at 34,19, the pull block at 31,24, and 42,26 to 43,28 holds a mix of push and pull pieces that neither moves cleanly.

## Pushing

At count 28 (checked before the step is counted, so on the 29th blocked step), and every 20 after, the client tries a push:

1. It fires the trigger action `pushed` at the test point above, with the facing as its parameter. NPCs there get `actionpushed`. Online the 2.05 client also sends it to the server and to its peers.
2. It reads the tile under that point. If it is one of the four tiles of the pushable block (`arrays.dat` array 16, entry 0: 1764, 1765 / 1780, 1781), that tile's place in the block gives the block's top-left. Only that one tile is compared, as in 2.05.
3. The two cells the block moves into must not be a wall, a chest, water, a chair (or the lower half of a bed, which Seihai's tile table does not load). From the block's top-left: up `(0,-1) (1,-1)`, left `(-1,0) (-1,1)`, down `(0,2) (1,2)`, right `(2,0) (2,1)`.
4. The block moves one tile. Each of its old cells first takes the tile two cells ahead, then the block is written one cell ahead. So the row the block leaves gets the floor that was in front of it.
5. The 2.05 client sends the 4 x 4 area around the old spot as a board modify, moves the player one tile in the facing, and plays `sounds/stonemove.wav`.

<<<<<<< HEAD
Seihai does steps 1 to 5 locally, but does not send the board modify or the trigger action online yet (as with lifting, see `cut-and-lift.md`). ## Pushed and pulled NPCs

When no block is there, the push routine looks for an NPC with `canbepushed` at the same probe point, and the pull routine for one with `canbepulled` at the grab point. Seihai does the same:

* `actionpushed` / `actionpulled` always fires first.
* **Edge.** Every point along the edge the NPC moves into, one per tile from its left or top, must be free of walls, chests and other NPCs (water does not stop it). Pushing up that is `(t, y - 1)` for `t` from `x` while `t < x + w`; left `(x - 1, t)`, down `(t, y + h)`, right `(x + w, t)`. A pull checks the side facing the player, and first the two cells behind the player: from the grab cell, up `(0,3) (1,3)`, left `(3,0) (3,1)`, down `(0,-3) (1,-3)`, right `(-3,0) (-3,1)`.
* **Size.** A character NPC counts as 3 x 3 tiles for the edge and is hit in `x + 0.5 .. x + 2.5`, `y + 1 .. y + 3`; otherwise its `setshape` size, else the image part it shows. Seihai counts an NPC whose image size it does not know as 2 x 2.
* **Move.** The NPC moves one tile (towards the player for a pull), the player one tile the same way, the block's sound plays, and the NPC's script gets `waspushed` / `waspulled`. 2.05 sends nothing to the server for the NPC.
* **Blocking.** 2.05 stops the player at every NPC that blocks. Seihai stops the player only at pushable and pullable NPCs (they are what a push needs), and lets A grab them. Other NPCs are still walked through.
* 2.22 is the same, except that the NPC lookup also searches the neighbouring map parts on a gmap.
=======
Seihai does steps 1 to 5. Online it sends the 4 x 4 area as `PLI_BOARDMODIFY` (1) and each push or pull try as `PLI_TRIGGERACTION` (38): no NPC (GINT 0), the point in half tiles, then `pushed,<dir>` or `pulled,<dir>`, the message the 2.05 client builds. 2.05 also sends the trigger to UDP peers (id 26); Seihai does not. NPCs with `canbepushed` are moved by the same routine in 2.05 and get `waspushed`; Seihai's scripts do not move NPCs, so only `actionpushed` reaches them.
>>>>>>> origin/main

## Grabbing and pulling

* **Order on A**: throw what is carried; lift; (Seihai: open a chest, read a sign); grab. 2.05 tries the grab straight after the lift; Seihai keeps its chest and sign checks first so they still work.
* **Grab test** at `(1.5, 0.9)`, `(0, 2)`, `(1.5, 3.1)`, `(3, 2)` by facing: a wall tile there. 2.05 also grabs an NPC that blocks and refuses when another player stands there; Seihai tests tiles only.
* The grab lasts while A is held. Arrow keys do not move the player or turn them.
* **The pull cycle.** Holding the arrow away from the wall sets a counter to 2 and starts the pull pose. Each tick the counter goes 2, 3, ... 19, then back to 2 (18 ticks). Whenever the arrow is held at 19 the client tries a pull. Letting go of any arrow puts the counter back to 1 (just holding).
* **A pull try** fires `pulled` one tile out from the body (`(1.5, 0)`, `(-0.5, 2)`, `(1.5, 4)`, `(3.5, 2)`), with the opposite of the facing as its parameter, then reads the tile at the grab point. If it is a tile of the pullable block (array 16, entry 1: 1998, 1999 / 2014, 2015), the two cells behind the player must be free (from the block's top-left: up `(0,4) (1,4)`, left `(4,0) (4,1)`, down `(0,-3) (1,-3)`, right `(-3,0) (-3,1)`). Then the block moves one tile towards the player (each old cell first takes the tile two cells behind), the player steps back one tile, and `lift.wav` plays (or `lift2.wav`, by a player flag that was not traced).
* **The futile pull.** On anything else (a wall, a rock too heavy for the glove, a push block, the mixed block pieces at 42,26 in `new3.nw`) nothing moves. The `pulled` trigger still fires every 18 ticks, and the pull pose keeps playing.

## Poses and sprites

| state | gani | sprite number S on the wire |
|---|---|---|
| pushing (18+ blocked steps) | `push` (one step per blocked step) | 11, 12, 13, 11, 12 by frame |
| grabbing | `grab` | 14 |
| pulling | `pull` (one step per tick, looping) | 15, 16, 17 by frame |
| jumping | walk (unchanged) | as walking |

Seihai draws these only when the gani is loaded; the default asset set has none of the three, so without a data folder that has them the player stands. The sprite numbers come from the 2.05 animation step table (push steps 14 to 18, grab 19, pull 20 to 22).

## 2.22 and 2.31

The same in both, with the same numbers, except: any blocking NPC can be grabbed (2.05 skips some, most likely character NPCs); the two server full-stop flags stop walking, ledges and the A key; and the jump, stone and pull sounds play only while a built-in gani is showing. Seihai follows 2.05, which matches all of this for tiles.

## Open

* What the 2.05 client draws right after A is let go while the pull gani is on was not settled; Seihai goes back to idle.
* Online sends: the board modify for a moved block and the `pushed` / `pulled` trigger action.
* Blocking on every NPC that blocks, which needs the NPC image sizes in the session.
* Moving `canbepushed` / `canbepulled` NPCs.
* A jump across a gmap edge: 2.05 shifts the jump's start point when the level changes in the air; Seihai moves into the next cell when it lands.
