# GServer-v2 beta4 notes

Source: https://github.com/xtjoeytx/GServer-v2/tree/beta4

## Appearance

- `setbody filename;` and `sethead filename;` are GS1 commands. They set `PlayerProp::BODYIMAGE` and `PlayerProp::HEADIMAGE` on the player.
- Chat commands `sethead <name>` and `setbody <name>` exist. They look up the file in the filesystem and send the prop. If the file is missing they request it from the listserver.
- `setheadallowed` and `setbodyallowed` are server settings (default true).

## Warps

- `Player::warp` calls `enterLevel`. If already on the level it sends X2/Y2 and the gmap cell. Otherwise it sets the position before the level is entered.
- A link request (`PLI_LEVELWARP`, id 0, or `PLI_LEVELWARPMOD`, id 30) to a plain level (not part of a gmap) goes to `enterLevel` with the requested position, not to `warp`. The server keeps the position and sends the level (`PLO_LEVELNAME`, board, links, signs, chests, NPCs, `PLO_SETACTIVELEVEL`), but no `PLO_PLAYERWARP`. A link into a gmap part does go through `warp` and gets `PLO_PLAYERWARP2`.
- So the client must put the player at the link's arrival itself, cached copy or not. Seihai only did that for cached links, so a link to a level it did not hold left the player where they stood in the old level ("first-load warps", 2026-10-29). It now holds the requested arrival for every link it starts and applies it when the server names the new level; a position the server does send still wins.
- A level the client does not hold can take longer than the 4 s warp timeout to arrive (the whole board, links and NPCs are sent). Giving up the warp used to drop the held arrival too, so the late level left the player at their old `playerx`/`playery` (laundry 2026-10-30, item 6). The arrival is now held by the destination's name (`net::LinkArrival`), not by the pending warp: the timeout only lets the player walk again, and when the destination is named, however late, the player is put at the link's arrival. A different level named first (a server-side warp) drops it.

## File transfer

- The server requests missing files from the listserver via `SVO_GETFILE3`.
