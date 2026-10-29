# NPC draw order

An NPC has a drawing height. The 2.05 client keeps it as one number per NPC, and four script commands set it:

| Command | Height | Seihai `DrawLayer` |
|---|---|---|
| `drawunderplayer` | -1 | `Under` |
| (default) | 0 | `Player` |
| `drawoverplayer` | 1 | `Over` |
| `drawaslight` | 9 | `Light` |

`blockagain` and `blockagainlocal` set the height back to 0, whatever it was, and also undo `dontblock`.

The NPC props the 2.05 client sends carry the same thing in the visibility flags (prop 13): bit 2 for `drawoverplayer`, bit 4 for `drawunderplayer`. Seihai does not read that prop yet. An online NPC gets its height from its script, which Seihai runs.

## The order Seihai draws in

1. Tiles, chests and the rest of the board.
2. `drawunderplayer` NPCs, images below height 0, NPCs with the default height, images at height 0 (`NpcPass::BelowPlayers`).
3. Other players, then the local player, then what is carried or thrown.
4. `drawoverplayer` NPCs, images at heights 1 to 8, the screen shading, `drawaslight` NPCs, images at height 9 (`NpcPass::AbovePlayers`).
5. Names, the chat bubble, the status bar, then the screen-layer images (`NpcPass::Screen`).

## Image heights (`showimg`, `changeimgvis`)

From the 2.05 code: each `showimg` image keeps its own height on the same scale as the NPCs.

* `showimg` sets it to the NPC's height + 1 (the player's, 0, for a weapon), every time, also on an index already in use. It also clears the image's part; its colour and zoom stay.
* `changeimgvis index,v`: 0, 1, 2 give the NPC's height - 1, + 0, + 1; 3 gives 9 (the light height) and 4 gives 10, whatever the NPC's height.
* Height 10 and up is the screen layer: the image's x and y are screen pixels, not level tiles.

Seihai draws each image after the NPCs of the same height. Which of the two the 2.05 client draws first was not found. Weapon images are not drawn (Seihai keeps images on level NPCs only). The range check on `v` was read only in part; Seihai ignores values past 4, which is inferred.

The `showimg` images go over the player because the 2001 new-features notes give their default height as "like drawoverplayer". The same notes put `drawaslight` NPCs "between drawoverplayers and the stats", above the day and night shading. Seihai has no day and night shading yet.

## What is confirmed and what is not

- Confirmed from the 2.05 code: the four heights and their values, and that `blockagain` and `blockagainlocal` reset the height to 0.
- From the 2001 new-features notes: under, like the player, over; the `showimg` default; where `drawaslight` goes.
- Not tested: whether NPCs with the default height and the players are sorted by their y position in the real client. The notes say "like the player". Seihai draws them all before the players. A tree without `drawoverplayer` therefore still draws under the player.
- `changeimgvis` is done (above). An NPC under `drawaslight` that calls `showimg` gets a height 10 image, so it lands on the screen layer; that follows the 2.05 code.
