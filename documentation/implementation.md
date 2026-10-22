# Seihai — current implementation

Date: 2026-10-14

## Offline slice

`OfflineSession` can:
1. Load a `GLEVNW01` `.nw` via `Level::load`.
2. Decode tile ids from the 2-char base64 board encoding.
3. Apply `BoardModify` updates.
4. Reject moves that land on wall tiles (`collision >= 20`).
5. Track water (`== 11`) in `in_water` without blocking.
6. Parse `LINK`, `SIGN`, `BADDY`, and `CHEST` records.
7. Decode binary sign text (x+0x20, alphabet byte-89, space token).
8. Step an optional walk GANI on every successful move.
9. Apply `hide`/`show` (toggles `Npc.visible`), `warp` (sets player position), `hurt` (health), `say`/`message` (last message), `set`/`unset` (flags), `setstring`/`addstring` (string variables), and `setplayerprop` (props) via `handle_command`.
10. Use the documented single-point foot test (`+32` / `+44` pixel offsets) in `try_move`.
11. Load type-0 collision from `arrays.dat` via `load_tiletypes_arrays` (first array = nowall list).
12. Detect nearby NPCs with `check_touch` and fire simple `hide`/`warp` patterns from their scripts.
13. Place stamps onto the board via `Level::place_stamp` or `place_named_stamp` (from an `ObjectSet`).
14. Detect when the player enters a `LINK` rectangle and record the destination in `pending_link`.

Directions are fixed, in Graal order: 0 up, 1 left, 2 down, 3 right.

## Render

SDL2 is not wired. The `render` module provides:
- `ascii_board` — full board, direction character for the player
- `ascii_camera` — window centered on the player
- `write_ppm` — 64×64 PPM (walls dark, water blue, player red or blue)

## Protocol (provisional)

| Type | Notes |
|------|-------|
| `LoginRequest` | classic line: prefix + `GNW#####` + length-biased account/password |
| `LevelRequest` | level name |
| `PlayerUpdate` | classic (×2), precise (×16), provisional walk frame |
| `BoardModify` | x, y, width, layer, tile data |

Packet ids remain provisional. The classic login line and the walk frame do not use the provisional ids.

## Assets

| Module | Status |
|--------|--------|
| `crc32` | ported from client export routines |
| `png` | minimal writer |
| `level` | GLEVNW01 parser, TileTypes (text or arrays.dat), records |
| `gani` | SPRITE/ANI/ANIEND/PLAYSOUND/DEFAULTATTR parser + stepper |

## Scripting

`Bytecode::execute` runs a minimal stack VM covering the OpenGS1 arithmetic, comparison, and jump opcodes. `Commands` holds offline stubs. `Call` and array ops are no-ops.

## Tools

`seihai-level-editor` reads a `.nw` and prints an ASCII preview or re-exports it.

## Clean-room boundary

Implementation is driven by public server sources, observed packet shapes, and format documentation. Raw decompiles are not imported.
