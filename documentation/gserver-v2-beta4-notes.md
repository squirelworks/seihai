# GServer-v2 beta4 notes

Source: https://github.com/xtjoeytx/GServer-v2/tree/beta4

## Appearance

- `setbody filename;` and `sethead filename;` are GS1 commands. They set `PlayerProp::BODYIMAGE` and `PlayerProp::HEADIMAGE` on the player.
- Chat commands `sethead <name>` and `setbody <name>` exist. They look up the file in the filesystem and send the prop. If the file is missing they request it from the listserver.
- `setheadallowed` and `setbodyallowed` are server settings (default true).

## Warps

- `Player::warp` calls `enterLevel`. If already on the level it sends X2/Y2 and the gmap cell. Otherwise it sets the position before the level is entered.
- The first-load arrival problem is on the client side: the client applies the link arrival before the board arrives, then the board overwrites it.

## File transfer

- The server requests missing files from the listserver via `SVO_GETFILE3`.
