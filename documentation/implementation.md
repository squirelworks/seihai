# Seihai — current implementation

Date: 2026-10-22

## Offline slice

`OfflineSession` can:
1. Load a `GLEVNW01` `.nw`, a binary `GR-V1` `.graal`, or a `Z3-V1` `.zelda` via `Level::load`.
2. Decode tile ids from the 2-char base64 board encoding (text) or the RLE bitstream (binary).
3. Apply `BoardModify` updates.
4. Reject moves that land on wall tiles (`collision >= 20`).
5. Track water (`== 11`) in `in_water` without blocking.
6. Parse `LINK`, `SIGN`, `BADDY`, and `CHEST` records.
7. Decode binary sign text (x+0x20, alphabet byte-89, space token).
8. Step an optional walk GANI on every successful move.
9. Apply `hide`/`show`, `warp`, `hurt`, `say`/`message`, `set`/`unset`, `setstring`/`addstring`, `setplayerprop`, `setimg`/`setshape`, and `move` via `handle_command`.
10. Use the documented single-point foot test (`+32` / `+44` pixel offsets) in `try_move`.
11. Load type-0 collision from `arrays.dat` via `load_tiletypes_arrays`.
12. Detect nearby NPCs with `check_touch`.
13. Place stamps via `Level::place_stamp` or `place_named_stamp`.
14. Detect `LINK` entry and record the destination in `pending_link`.

Directions follow Graal order: **0 up, 1 left, 2 down, 3 right**.

## Render

No system SDL2. The `render` module provides:
- `ascii_board` / `ascii_camera` — text preview
- `write_ppm` — 64×64 PPM
- `render_view` — software tile compositor from a 2048×512 sheet
- `blit_sprite` — RGB/RGBA blit, skips alpha 0 or magenta
- `draw_player_sprites` — body and head by direction
- `draw_hud` — health and rupee bars
- permanent chat bar at the bottom
- chest/sign/NPC markers
- windowed binary via `minifb` (`cargo run -p seihai`)
- F5 opens a file picker for `.nw` / `.graal` / `.zelda`
- Play offline button in the GUI skips the connection

The player sprite is shifted down 12 px so the feet sit on the collision tile.

`pics1.png` tile mapping: eight 16-column blocks of 512 tiles. Tile `n` is at column `n % 16 + (n / 512) * 16`, row `(n / 16) % 32`.

## Chat and overlays

From the merged chat/HUD PRs:
- Permanent chat bar at the bottom. Enter opens it, Enter again sends.
- F2 opens a PM to the nearest player.
- F3 toggles the debug overlay (fps, position, level, frames, players).
- Tab toggles the player list. (F4 is the level editor, not built yet.)
- F1 or ? shows the key help; a hint says so at start. The list is `client/src/help.rs`, and a test keeps it in step with `bin/main.rs`.
- F12 saves a PNG to `screenshots/`.
- F9 is developer mode (wallhack). It is off at every start, never saved, shows `DEV MODE` while on, and turns itself off on a level change, warp or disconnect. Build with `--no-default-features` to compile the wallhack out.
- Chests open with A or Space when facing them. The terminal prints the item.
- Signs open a reading panel.
- Nickname tags and chat bubbles draw over other players.

## Tiles, swimming and links

- Tile types come from `arrays.dat` (the real client's data): array 0 is the no-wall list, array 1 the liquid list, array 3 the lava list. Water and lava are not walls. Anything off the board is a wall.
- Walls are tested at the four leading-edge points of the 2 x 2 body box (`WALK_PROBES`, see `walking-collision.md`), each truncated to a tile. The foot point `(x + 2.0, y + 2.75)` is still used for links, water and scripts.
- Swimming steps take twice as long as walking. On 2.1+ the client sends gani `swim`/`idle` (prop 10) when it enters or leaves water. Nothing is sent before 2.1: prop 10 is the bow slot there, and the 2.05 swim sprite number is not verified.
- After a warp the client does not check links until a warp reply arrives, the player has moved, and the foot has left any link that contains the arrival tile.

## Protocol

| Piece | Status |
|-------|--------|
| `write_login` | versioned login line, GCHAR type and key |
| `ClientVersion` | 1.33–2.31 version switch; everything per version is in its `VersionProfile` |
| `VersionProfile` | per-version table: login, packet and prop sets, files, levels, UI switches (see `version-profiles.md`) |
| `Gen3` | LCG insertion, verified 0 mismatches on 704 lab messages |
| `Gen5` | partial-XOR + zlib/bz2, verified on 2.22 lab sessions |
| `read_zlib_frame` / `write_zlib_frame` | u16be + zlib |
| `SessionState` | level-entry dispatcher, player props 15/16/17 |
| `parse_list_reply` | SVRLIST GSTRING layout (stub-verified) |
| UDP | `seq` + GCHAR id, XOR with `recipientId ^ 1` |
| `PacketId` | deprecated; real ids in Graal_analysis `packet-ids.md` |

## Assets

`AssetRoot` expects:
- `pics1.png`
- `levels/bodies/body.png`, `levels/heads/head.png` (4-bit palette with tRNS)
- `levels/*.nw`, `levels/*.graal`, `levels/*.zelda`
- `gani/` or `levels/ganis/`
- `tiledefs.txt` or `arrays.dat`

## Open

- Real chest art and full HUD (hearts, rupees, bombs, sword)
- Other players and minimap
- Full GANI frame stepping for walk
- Movement speed and edge clamp
- Client→server bz2 frame and real listserver reply