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
| Level, gmap | `levels/` (downloads), root, `maps/` | A name with an extension must be one the profile opens (`levels.nw`, `.graal`, `.zelda`, `.gmap`). A bare name takes the profile's extensions in turn (`.nw`, `.graal`, `.zelda`, `.gmap`, those the build opens). A missing `.zelda` name is not swapped for a `.graal` one. |
| GANI | `gani/`, `levels/ganis/`, `levels/bomys/`, `levels/`, root | `.gani` added to a bare name. None at all where the profile's `ui.ganis` is false (1.39 is one). |
| Image | the folder hint (`bodies`, `heads`), `levels/`, root, every `levels/<category>/`, the same categories under root, `maps/` | Bare name: `files.image_ext` first (`.gif` below 2.1, `.png` after), then `.png`, `.gif`, `.mng`. A `.gif` or `.png` name falls back to the other extension (the folder ships PNG only). |
| Sound | `sounds/`, `levels/sounds/`, `levels/`, root | `.wav` added to a bare name; a leading `+` (loop) is dropped. |
| Music | `levels/midis/`, `midis/`, `levels/`, `sounds/`, root | `.mid` added to a bare name. |
| Tile types | root, `levels/` | `tiledefs.txt`, else `arrays.dat`. |
| Tiledefs | `levels/tiledefs/`, `tiledefs/` | `image,prefix,type[,x,y]` lines. |

### Layers (item 20)

The table above is the folder order *inside one data folder*. The resolver
searches up to three folders, one after the other, and the first that has the
name wins:

1. the **server's cache folder** (`<cache root>/<host>_<port>[~name]/`, what
   the server sent; the only place a download is written),
2. the user's **asset folder** (`--assets`, optional),
3. the **built-in default set** (`assets/default/`, compiled in, read-only).

So `--assets` may be missing, empty or partial: what it lacks comes from the
built-in set, file by file. Built-in levels (`new1.nw`, `new2.nw`) are found
offline only, and the built-in tile type table is used only with the built-in
sheet. Layout, naming, what a download does and how `WANTFILE` / `UPDATEFILE`
treat built-in files: `documentation/file-cache.md`. The built-in set:
`documentation/default-assets.md`. `AssetResolver::new(root, profile)` is the
one-folder resolver as before; `AssetResolver::empty(profile).with_assets(..)
.with_cache(..).with_builtin(..)` builds the layered one. `seihai
--check-assets` checks the `--assets` folder alone, or the built-in set when
there is none.

All names ignore case (the folder has `TORERO-2.MID`; Linux file systems do not
ignore case). A name with `..`, a leading `/` or a colon is refused: names come
from the server. A miss is never cached, so a file that arrives from the server
a moment later is found at once.

The client binary uses it for every level (`--level`, level changes,
gmaps and their neighbours), GANI, NPC image, body/head art and the tile type
table. Sound effects keep their own case-insensitive index inside `SoundEngine`.

### `.graal` levels

`Level::load` now reads the sections after the board of `GR-V1.00` to
`GR-V1.04` as Graal_analysis `formats/level-binary.md` and
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

#### What real files showed (two supplied levels and a 3113-level player world)

Two real `GR-V1.03` levels and an old player world (3141 files named
`.graal`) were run through the reader. Nothing from them is stored in the
repository; the `#[ignore]` tests read them from a folder you point at.

- `SEIHAI_GRAAL_LEVELS=<dir>` (`real_graal_levels_match_the_format_notes`):
  every `.graal` in the folder must parse and tile up, and the two supplied
  levels are checked against counts, section offsets and a few tile ids
  (keyed by the file's CRC-32).
- `SEIHAI_PLAYERWORLD=<dir>` (`playerworld_graal_corpus_parses`): walks the
  tree. 3113 files are `GR-V1.03`, 27 are empty, 1 is a text level
  (`GLEVNW01`) under a `.graal` name. All 3113 binary levels now parse with
  the board complete, every link line read, and the sections ending exactly at
  the end of the file. `SEIHAI_CORPUS_TSV=<file>` also writes one derived row
  per file.
- `seihai --level-info <file>` prints sizes, offsets and record counts of a
  binary level (no content).

What this settled:

- **13-bit codes are right for `GR-V1.03`.** 1998 of the 3113 files hold a tile
  id of 2048 or more, which only a 13-bit code can carry. The notes' board
  rules (flag bit, count in the low byte, `0x100` for a double run, runs up
  to 64) decode all 3113 boards to exactly 4096 tiles, with the section
  that follows starting on the next byte.
- **Section shape.** Links end with `#`; the baddy section ends with
  `FF FF FF` plus a line end; NPCs end with `#`; chests end with `#`. The
  real files stop right after the chest `#`: when a level has no signs there
  is no sign section and no empty line, so the sign reader runs into the end
  of the file. Links, chests and NPC coordinates are whole tiles (a link's
  two tiles and a chest's own tile carry matching board ids).
- **A baddy at x = 0 is a baddy.** Only a first byte of `0x80` or more ends the
  baddy section (as `level-binary-client.md` says). Two levels have baddies at
  x = 0; the reference `grv1.py` stops there and then misreads the NPCs.
- **Sign index `0x1F` (-1) in a chest means "no sign".** 343 of 375 chests.
- **Link arrivals can be expressions.** Besides numbers, `playerx`,
  `playery`, 14 levels write `playery+10`, `playerx-15`, `playery-1.5`, and
  one writes a stray word. Before, such a link was dropped. Now it is kept;
  the signed number is stored as `Link::x_offset` / `y_offset`, and a stray
  word is read as "keep the player's coordinate". The offline engine adds the
  offset. Whether a real client evaluates `playery+10` is open: no capture
  shows it.
- **The sign decoder was wrong.** Seihai decoded signs with the retracted
  `char + 89` rule, which loses punctuation and line breaks. It now uses the
  one packed alphabet (`byte - 32` index, glyph codes, `#K(n)` escapes) from
  `formats/sign-text.md`, checked against the real-sign vectors in that note.
  The two supplied levels have no signs; the corpus has 2381 (808 files) whose
  text uses `#K(n)` for characters the table lacks (`_ = * $ @ ^ |` and
  Latin-1 letters).
- **No `HEIGHTS` block** appears in any file (all are `GR-V1.03`; there is no
  `GR-V1.04` sample), so the block is still read past and not stored.
- NPC and baddy positions are single bytes and may lie outside the 64 x 64
  level (NPC x from -32 to 223); the reader keeps them as given.

### `.zelda` (Z3-V1) levels

`Z3-V1.03` and `Z3-V1.04` files have their own reader, `client/src/assets/zelda.rs`
(`ZeldaLevel::parse`, `to_level`, `to_bytes`). `Level::load` and
`Level::parse_binary_with_layout` hand every `Z3-V1` magic to it; the `.graal`
section reader no longer sees them. The layout, settled against the Java Zelda
Online source, the 1st Graal and 1.39 clients (decompiles, derived facts only)
and 139 distinct real files (663 copies) from the old client builds:

```text
char[8]  magic      Z3-V1.03 | Z3-V1.04      (Z3-V1.02: Java client only, 11-bit codes)
bits     board      4096 tiles, 12-bit codes, the GR-V1 run rules
text     links      `dest x y w h newx newy` lines up to `#`
binary   computers  3 raw bytes x y type, the rest of the line, up to a first byte >= 0x80
text     signs      `x+32 y+32 text` lines up to an empty line or the end
```

- **No NPC and no chest section.** "Computer" is the Zelda Online word for a
  baddy. A computer record is `x y type` and a line end. In `Z3-V1.03` the line
  is empty (this is what the notes call "no verse line", and also the 4-byte
  record of the Java source); in `Z3-V1.04` it is the verse text
  `sight\hurt\attack`. The terminator is `FF FF FF` and a line end. `x` and `y`
  are raw bytes (tiles), not the +32 of signs. A `0x0A` among the three bytes is
  data (2 real records have one), so the reader takes three bytes before it looks
  for the line end, as the 1.39 loader does and as the Java source's fixed four
  bytes do. Types are 0 to 6 in every real file; the Java source names them
  (soldier green, blue, red, shot soldier, swamp soldier, jumping hare, octopus);
  later builds' names for the same numbers are not claimed.
- **Links use decimals.** `Z3-V1.03` links are written `31.0 42.0 2.0 1.0 43.5 41.0`
  (124 of 207 distinct `1.03` links have a decimal point in x, y, width or
  height; all are whole tiles). Both Delphi clients and the Java source read
  six floating-point numbers. A **negative arrival coordinate keeps the player's
  own** (`-1` in 151 links of the distinct files): 1st Graal and the Java source
  both set a "constant" flag from `< 0`.
- **The reader follows the clients' loop, not only the notes.** The
  double-run flag stays set until a double run completes (the `.graal` reader
  clears it on the next control code; no real file shows the difference, 0 of
  663). A run that overruns the board is an error. The first `FF` record ends
  the computers; the sign loop ends at an empty line.
- **Strict, and honest about unknowns.** `ZeldaLayout` records offsets, counts
  and every `anomalies` line (a section without its `#` or terminator, text
  after a `1.03` record, a bad link line, bytes after the signs). A clean file
  reaches the end of the file exactly: all 663 real copies do. Computer text and
  sign bytes are kept as stored (`Computer::extra`, `ZeldaSign::raw`).
- **The extension decides nothing.** The clients read the magic. Of the 139
  distinct `Z3-V1` files, 31 `Z3-V1.04` files sit under a `.graal` name (all of
  1st Graal's levels) and 1 `Z3-V1.03` file under `.editor`. In the other
  direction 694 distinct `.zelda`-named files are `GR-V1.00` to `GR-V1.03`
  (2nd Graal to 15th Graal ship 68 to 368 of them per build). `Level::load`
  dispatches on the magic for all of them.
- **Profile gate.** `levels.zelda` is true for every row. Every client binary
  in the Drive set (1st Graal, 1.28b2 to 1.41, 2.0 to 2.31, the NewMain and NW
  betas) compares the level magic with `Z3-V1.03` and `Z3-V1.04` and with no
  other `Z3-V1` revision. `Z3-V1.02` is read by no Graal client and is in no real
  file; Seihai still decodes it (11 bits) for the Java client's files and says so
  in the layout report. The resolver opens a `.zelda` name for every profile and
  tries `.zelda` for a bare name after `.graal`; that order is Seihai's. No
  binary says whether a build tries `.zelda` for a bare name.
- **`seihai --level-info FILE`** prints the Z3-V1 layout (sections with offsets,
  record counts, computer types, negative arrivals, anomalies) and exits 1 if the
  file has an anomaly. **`--check-assets`** now also reads every `.zelda` and
  `.graal` file under the folder by magic, lists up to 20 that do not read
  clean, and prints counts.
- **Engine.** The converted `Level` has the board, links, signs and the computers
  as baddies, so the offline engine's link rules and the F5 picker work on
  `.zelda` files (`zelda_level_links_work_in_the_engine`). Baddies are still not
  simulated.

Two things changed for `.graal` and `.nw` levels on the way:

- **Negative link arrivals.** `link_coord` read `-1` as the arrival
  coordinate `-1`. It now means "keep the player's coordinate", like `playerx`.
  805 of 5381 distinct `GR-V1` links in the Drive set use it. This rests on the
  1st Graal and Java rule above; whether 2.x clients treat `-1` the same is not
  checked.
- **The old reader's Z3-V1 faults**, found by replaying the old section reader
  over the real files: it dropped every link with a decimal x, y, width or
  height (124 links, 47 distinct `1.03` files) and misread the computers of all
  31 distinct `1.03` files that have any (it consumed 3 bytes per record and
  left the line end to start the next record).

Real files, aggregates only:

- `SEIHAI_DRIVE_CLIENTS=<folder holding the build folders>`:
  `real_z3_files_match_the_format_notes` (every `Z3-V1` file under it reads
  clean, with the derived facts above; the structure survives `to_bytes`) and
  `real_drive_builds_level_scan_is_clean` (`--check-assets`' scan on every build
  folder that has levels).
- The player world has no `.zelda` file and no `Z3-V1` file: 3113 `GR-V1.03` under
  `.graal`, 9 `GR-V1.03` under `.nw`, 3347 `GLEVNW01` `.nw`.

Not settled:

- **Which build tries `.zelda` for a bare name**, and whether a missing `.zelda`
  link falls back to the `.graal` of the same stem (1 to 3 links per build would
  need it; no binary says).
- **Sign glyph codes 82 to 88.** The Java source reads codes 82 to 88 as `#1` to
  `#4` glyph pairs; the shared alphabet has other characters there. 5 of the 120
  distinct real `Z3-V1` signs contain code 82 or 83. Their text under the shared
  alphabet was not compared with the Java reading. Seihai uses the shared
  alphabet.
- **1st Graal reads computers a line at a time**, so a record with `0x0A` as x or y
  would end its section early; no 1st Graal file has one (the 2 real cases are in
  `Z3-V1.03` files). Not run.
- **What `Z3-V1.04` verse text means for types other than the Java ones**, and
  the fields a `Z3-V1.04` computer may carry beyond three verses (none seen).
- **`Z3-V1.02`** has no sample; its 11-bit rule is the Java source's.

### Checking a folder

`seihai --assets <dir> --check-assets [--version V]` resolves and loads the
files a version uses (tile sheet, tile types, HUD art, player art, walk/swim
GANIs, the named sound effects, a MIDI file, a sample level and gmap, the
offline tiledefs) and prints them. The list per build is
`expected_files(profile)` in `client/src/assets/coverage.rs`; it follows the
profile (no GANIs for 1.39, `.nw` only where `levels.nw`, the 24-name 1.39
sound list below the GANI builds and the 15-name 2.05 list from there, the
offline tiledefs from 2.17). It then reads every `.zelda` and `.graal` file under
the folder by magic (a `Z3-V1` file must parse to its end with no anomaly), lists
up to 20 failures and prints counts in a note.

Two `#[ignore]` tests use the real folder (`SEIHAI_DATA_222=<folder>`):

- `real_222_folder_serves_every_main_version`: 1.39, 2.05, 2.17, 2.22 and
  2.31 each find and read every file they use (29 or 31 per build, all found).
- `real_222_folder_every_file_of_each_kind_loads`: every `.png` (307),
  `.gani` (139), `.wav` (104) and `.nw` (1) in the folder loads, and each is
  found by name through the resolver.

### Not settled or not done

- **Which copy wins.** The resolver prefers a downloaded copy in `levels/`
  over a shipped one at the root and over `maps/`. A real client looks for
  the name as a path from the game folder first, then in its file index, and
  replaces the file it found in place; `DownloadStore::local_path` follows
  that for download checks and saves. The scan order of the index when two
  folders hold the same name is open (`documentation/file-cache.md`).
- **Tile sheets of other widths.** `pics1.png` in the 2.22 folder is 2048×512
  (128 columns). The notes give 1856×512 for 1.33 to 1.39. How tile ids map on
  that sheet is not in the notes, so the renderer still assumes 128 columns,
  and a 1.39 profile run from the 2.22 folder draws the 2.22 sheet. The
  HUD art differs too: `state.png` here is 224×145, the 1.38+ layout in the
  profile is 224×97.
- **No `.graal` level ships in the 2.22 folder** (`maps/bigmap.txt` names some).
  Real files came later (see "What real files showed"). The `HEIGHTS` block of
  `GR-V1.04` is skipped, not stored: no real file has one. The two supplied
  levels' NPCs name `.gif` art; GIF is not decoded.
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
