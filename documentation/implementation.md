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
- F4 toggles the player list.
- F12 saves a PNG to `screenshots/`.
- Chests open with A or Space when facing them. The terminal prints the item.
- Signs open a reading panel.
- Nickname tags and chat bubbles draw over other players.

## Protocol

| Piece | Status |
|-------|--------|
| `write_login` | versioned login line, GCHAR type and key |
| `ClientVersion` | 1.39–2.31 toggle with token and encryption gen |
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