# Claude Laundry List — 2026-10-28

Seihai is moving on a live server. These are the gaps seen in play.

## Tile types

Deep water is still walkable. Type 11 is shallow water (swim). Deep water is a different type in arrays.dat and should block, or at least not be treated as type 11. The loaded arrays.dat from the server is the source. Check which type numbers the server's arrays.dat assigns to deep water tiles, and make those types block.

## Signs

A sign should open when the player walks onto it, not when A is pressed. The current check is on the A key. The real client opens the panel on the foot test point entering the sign's tile.

## Ledges

Running against a ledge jumps the player down. The real client has a ledge tile type that, when approached from the high side, moves the player down one tile and plays a jump. Seihai treats every solid tile as a wall. The ledge type is in arrays.dat. Add the jump when the player walks into a ledge from the high side.

## Script parser

The stress test parsed 3038 scripts. 498 have diagnostics. The common failures are:

- `if(expr)` with no space after `if`
- `a&&b` with no spaces
- `elseif` as a single word
- `in |lo,hi>` closed with `>`
- `players[i].x` and `npcs[i].save[n]` chained indexes
- `in {list}` inside a larger expression

These are in client/tests/fixtures/npc-scripts/. The test is client/tests/script_stress.rs.

## Visual effects

The script log is full of unapplied visual commands: setzoomeffect, setcoloreffect, drawaslight, drawoverplayer, drawunderplayer. These are the light and colour effects on the map. They are recognized and ignored. Drawing them is a later item.

## From the decompiles

These were in the cleaned notes and are not in Seihai yet:

- The full tile type table from arrays.dat, including deep water, lava, chairs, and ledges
- Sign open on touch
- Ledge jump
- The hurt and death animations
- Other players' weapons and shields drawn
- The chat bubble above the player's head fading after a few seconds
