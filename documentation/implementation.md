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

Directions are fixed: 0 south, 1 west, 2 east, 3 north.

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
