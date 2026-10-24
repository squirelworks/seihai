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

## Status: loaders and the 2.22 data folder

The 2.22 folder ("Graal Reborn 2.22.0") is the reference layout. One resolver,
`AssetResolver` (`client/src/assets/resolve.rs`), is the only code that maps a
name to a file. It takes the data folder and a version profile.

### Layout it reads

| Kind | Folders tried, in order | Name rules |
|------|-------------------------|-----------|
| Level, gmap | `levels/` (downloads), root, `maps/` | A name with an extension must be one the profile opens (`levels.nw`, `.graal`, `.gmap`). A bare name takes the profile's extensions in turn. |
| GANI | `gani/`, `levels/ganis/`, `levels/bomys/`, `levels/`, root | `.gani` added to a bare name. None at all where the profile's `ui.ganis` is false (1.39 is one). |
| Image | the folder hint (`bodies`, `heads`), `levels/`, root, every `levels/<category>/`, the same categories under root, `maps/` | Bare name: `files.image_ext` first (`.gif` below 2.1, `.png` after), then `.png`, `.gif`, `.mng`. A `.gif` or `.png` name falls back to the other extension (the folder ships PNG only). |
| Sound | `sounds/`, `levels/sounds/`, `levels/`, root | `.wav` added to a bare name; a leading `+` (loop) is dropped. |
| Music | `levels/midis/`, `midis/`, `levels/`, `sounds/`, root | `.mid` added to a bare name. |
| Tile types | root, `levels/` | `tiledefs.txt`, else `arrays.dat`. |
| Tiledefs | `levels/tiledefs/`, `tiledefs/` | `image,prefix,type[,x,y]` lines. |

All names ignore case (the folder has `TORERO-2.MID`; Linux file systems do not
ignore case). A name with `..`, a leading `/` or a colon is refused: names come
from the server. A miss is never cached, so a file that arrives from the server
a moment later is found at once.

The client binary uses it for every level (`--level`, level changes,
gmaps and their neighbours), GANI, NPC image, body/head art and the tile type
table. Sound effects keep their own case-insensitive index inside `SoundEngine`.

### `.graal` levels

`Level::load` now reads the sections after the board of `GR-V1.00` to
`GR-V1.04` (and `Z3-V1.03/04`) as Graal_analysis `formats/level-binary.md` and
`level-binary-client.md` describe them: links (the name may contain spaces),
baddies (three raw bytes and a verse line, ended by `FF FF FF`), NPCs
(`x+32, y+32, image#script`, line breaks as `0xA7`), chests from `GR-V1.01`
(four GCHARs, the item index named from the 25-name table) and signs
(the wire sign alphabet). Before, only the board was read, so every NPC, link
and sign in a 1.39-era level was lost. `GR-V1.04` now uses 13-bit codes like
`.02` and `.03`; before it fell through to 12.

Tests build `.graal` files from a board writer that follows the 2.05 writer's
rules (single runs, double runs, 13-bit packing), so no game level is
embedded.

### Checking a folder

`seihai --assets <dir> --check-assets [--version V]` resolves and loads the
files a version uses (tile sheet, tile types, HUD art, player art, walk/swim
GANIs, the named sound effects, a MIDI file, a sample level and gmap, the
offline tiledefs) and prints them. The list per build is
`expected_files(profile)` in `client/src/assets/coverage.rs`; it follows the
profile (no GANIs for 1.39, `.nw` only where `levels.nw`, the 24-name 1.39
sound list below the GANI builds and the 15-name 2.05 list from there, the
offline tiledefs from 2.17).

Two `#[ignore]` tests use the real folder (`SEIHAI_DATA_222=<folder>`):

- `real_222_folder_serves_every_main_version`: 1.39, 2.05, 2.17, 2.22 and
  2.31 each find and read every file they use (29 or 31 per build, all found).
- `real_222_folder_every_file_of_each_kind_loads`: every `.png` (307),
  `.gani` (139), `.wav` (104) and `.nw` (1) in the folder loads, and each is
  found by name through the resolver.

### Not settled or not done

- **Which copy wins.** The resolver prefers a downloaded copy in `levels/`
  over a shipped one at the root and over `maps/`. What a real client does
  when both exist is the file-transfer caching question (queue item 9).
- **Tile sheets of other widths.** `pics1.png` in the 2.22 folder is 2048×512
  (128 columns). The notes give 1856×512 for 1.33 to 1.39. How tile ids map on
  that sheet is not in the notes, so the renderer still assumes 128 columns,
  and a 1.39 profile run from the 2.22 folder draws the 2.22 sheet. The
  HUD art differs too: `state.png` here is 224×145, the 1.38+ layout in the
  profile is 224×97.
- **No `.graal` level ships in the folder** (`maps/bigmap.txt` names some). The
  `.graal` reader is tested with synthetic files only. The `HEIGHTS` block of
  `GR-V1.04` is skipped, not stored.
- **GIF and MNG.** Found, not decoded. The folder holds 28 `.mng` and no GIF.
- **Tiledef matching.** The files parse; how a client matches `prefix` against
  a level name and what types other than 0 and 1 mean (the folder uses 4) is
  not in the notes, so no sheet is chosen from them.
- **GANI files that name files the folder lacks.** Across the 139 GANIs, 25
  sprite sheet names and 3 sound names are not in the folder (they are fetched by
  the server file request); `attr1` to `attr6`, `param1`, `horse`, `shield`
  and `sword` in the sheet column are script-bound names, not files.
  37 `PLAYSOUND name n` lines in the bomy GANIs carry one number after the
  name. The notes give `name x y` only, so the parser keeps the number in the
  name and the file is not found. The meaning of the single number is open.
- **The picker.** The GUI's level picker lists `levels/` only; the folder's
  one sample level is at the root.
- **Music, GIF/MNG art and HUD layout per version** are listed above but not
  rendered by these loaders.
