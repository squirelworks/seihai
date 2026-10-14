# Asset Formats

Seihai needs loaders for the common Graal / Graal Reborn file formats so levels and characters can be displayed without the original client.

## Core Formats

| Format          | Purpose                          | Notes                                      |
|-----------------|----------------------------------|--------------------------------------------|
| Level (`.nw`, `.graal`, etc.) | Tile map + links + NPCs     | Several historical variants exist          |
| Tileset / pics  | Tile graphics                    | `pics1.png` style and later packed formats |
| Sprite sheets   | Player bodies, heads, swords, shields | Indexed or PNG                       |
| Gani            | Character and object animations  | Frame sequences + offsets                  |
| Sounds          | WAV / other                      | Simple playback                            |
| Server files    | `servers.txt`, account files, NPC scripts | Text-based, easy to parse             |

## Level Structure (conceptual)

A level typically contains:

- Dimensions and tile data (often a flat array or run-length encoded)
- Links to neighboring levels
- NPC definitions (image, position, script)
- Optional signs, chests, baddies
- Background / overlay layers in later formats

Exact binary layouts are documented from community reverse-engineering and the open server loaders; detailed specs go in `../shworkingpapers/assets/`.

## Sprite & Gani

- Bodies, heads, swords, and shields are usually separate image sets that are composited at runtime.
- Gani files describe animation sequences (idle, walk, attack, etc.) with frame indices and timing.
- The client must support the classic 8-direction walking animations used by 2.x clients.

## Implementation Plan

1. Text-based level and NPC loaders (easiest, covers many Reborn servers).
2. Binary level loader matching the formats used by GServer-v2.
3. PNG/sprite sheet loader + simple atlas.
4. Gani parser and animator.
5. Sound loader.

All loaders should be usable by both the client and the offline tools (editors, generators).
