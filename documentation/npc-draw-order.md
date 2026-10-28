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
2. `drawunderplayer` NPCs, then NPCs with the default height (`NpcPass::BelowPlayers`).
3. Other players, then the local player, then what is carried or thrown.
4. `drawoverplayer` NPCs, then the `showimg` images, then `drawaslight` NPCs (`NpcPass::AbovePlayers`).
5. Names, the chat bubble, then the status bar.

The `showimg` images go over the player because the 2001 new-features notes give their default height as "like drawoverplayer". The same notes put `drawaslight` NPCs "between drawoverplayers and the stats", above the day and night shading. Seihai has no day and night shading yet.

## What is confirmed and what is not

- Confirmed from the 2.05 code: the four heights and their values, and that `blockagain` and `blockagainlocal` reset the height to 0.
- From the 2001 new-features notes: under, like the player, over; the `showimg` default; where `drawaslight` goes.
- Not tested: whether NPCs with the default height and the players are sorted by their y position in the real client. The notes say "like the player". Seihai draws them all before the players. A tree without `drawoverplayer` therefore still draws under the player.
- Not done: `changeimgvis`, which sets a `showimg` image's own height. All images use the default height for now.
