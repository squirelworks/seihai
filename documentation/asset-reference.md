# Reference sheet measurements (item 18)

What the real sheets, animations and sounds look like **as numbers**, and what a Seihai default asset set therefore has to match. Names, sizes, formats and counts only: no pixel data, no screenshots or renderings, no dumps of sheet contents, and no text from any GANI or level is in this note.

Dated 2026-10-25. Based on `chiaki/asset-inventory` (#49, still open when this was written, so this branch carries its commit). Seihai code quoted here is from `main` at `d5389c2`.

**Design note.** Seihai's shipped default set will be **original art drawn by us** (item 19). The playerworld files are player-made or community-made and stay in private archives. They are used here only to learn the constraints a default set must satisfy: sheet sizes, grid positions, palette types, GANI structure and sound formats. Nothing from them goes into the repository, and nothing from this note should be used to copy a picture.

## What was staged

27 files were copied into the private folder `/workspace/graal/lab/asset_ref/` on the build box (mode 700, files mode 600, never in a repo, never uploaded): three `pics1.png` sheets, two body sheets, two head sheets, two sword sheets, three shield sheets, ten GANIs (the five stock ones from two worlds) and five WAVs. The 1856x512 sheet from Lumina-1 was read in place for its size and tile mapping only. Not opened: the `GRACC001` account exports and every `.txt` file of the Lumina-1 world. `asset-reference.tsv` has one row per measured file (`world`, path inside the world, kind, bytes, size, format, details).

## 1. Tile sheet and id mapping

### The mapping

Seihai (`client/src/render/tiles.rs`, `Framebuffer::blit_tile`) draws tile id `t` (0 to 4095) from this cell of a 16 px grid:

```
column = t % 16 + (t / 512) * 16
row    = (t / 16) % 32
```

Arithmetic against the real sheet size, 2048 x 512:

- 2048 / 16 = **128 columns**, 512 / 16 = **32 rows**, so 128 x 32 = **4096 cells**, one per tile id (ids are 12 bits in `.nw`, `0` to `4095`).
- The sheet is **eight blocks of 16 columns**. Each block holds 16 x 32 = 512 tiles, and there are 8 x 512 = 4096 ids.
- Block `b = t / 512` (0 to 7) starts at column `16 * b`. Inside a block the id runs left to right, then down: `t % 512 = row * 16 + col_in_block`.
- Extremes: `t = 0` is column 0, row 0. `t = 511` is column 15, row 31. `t = 512` is column 16, row 0. `t = 4095` is column 127, row 31, the last cell. So the largest id lands exactly on the last cell of the sheet and no id can fall outside.

The mapping is therefore a bijection from 0..4095 onto the 128 x 32 grid, and it needs exactly 2048 x 512 pixels.

### Checked with Seihai's own reader and tile code

A throwaway integration test (not committed, to keep this PR to the note; the source is on the box) read each sheet with `read_png_rgba` to get its size, then built a synthetic RGB sheet of that size in which the cell at (column, row) carries its own column and row, and called the real `blit_tile` for every id 0 to 4095. Results:

| Sheet (read by `read_png_rgba`) | Size | Ids drawn | Ids skipped (outside sheet) | Distinct cells hit | Max column | Max row | Cells on sheet |
|--|--|--:|--:|--:|--:|--:|--:|
| Maloria-03-04 `pics1.png` | 2048x512 | 4096 | 0 | 4096 | 127 | 31 | 4096 |
| Coronia-1 `images/pics1.png` | 2048x512 | 4096 | 0 | 4096 | 127 | 31 | 4096 |
| Hyroko `temp/pics1.png` | 2048x512 | 4096 | 0 | 4096 | 127 | 31 | 4096 |
| Lumina-1 `drak_images/pics1c.png` | 1856x512 | 3712 | 384 | 3712 | 115 | 31 | 3712 |

Every drawn id also matched the formula above (asserted in the test). The existing unit test at `client/src/render/tiles.rs` (flat-colour sheet, tile `n` in the red byte) covers the same mapping on a synthetic sheet.

### Checked against real levels

The cell count shows the mapping is self-consistent, not that it is the one real levels use. For that, 60 text levels of Maloria-03-04 (shuffled with a fixed seed, 241,920 horizontally adjacent layer-0 tile pairs) were looked up on Maloria's `pics1.png` three ways. The number is the mean absolute difference, per colour channel (0 to 255), between the right edge column of the left tile and the left edge column of the right tile. Tiles that belong next to each other in a drawn level should match along their edges.

| Mapping | Mean edge difference |
|--|--:|
| Seihai: `col = t % 16 + (t / 512) * 16`, `row = (t / 16) % 32` | **9.93** |
| Column-major (`col = t / 32`, `row = t % 32`) | 42.32 |
| Same Seihai mapping, tile ids randomly shuffled | 54.08 |
| Linear (`col = t % 128`, `row = t / 128`) | 74.92 |

Seihai's mapping is about five times better than chance and the alternatives are worse than chance. The levels of this world use 3,262 distinct layer-0 ids out of 4096 (881 text levels), with ids up to 4095, so all eight blocks are in use.

### Surprise: `assets::tile_src_rect` uses the other mapping

`client/src/assets/level.rs` has `pub fn tile_src_rect(id)` (re-exported from `assets`) that returns the **linear** cell (`col = id % 128`, `row = id / 128`) and its comment says "on a 2048x512 sheet (128x32 of 16 px)". That is the 74.92 row above, not the mapping the renderer uses. Nothing in the client calls it (only the re-export names it), so nothing draws wrongly today, but any caller that picked it up would. It has since been deleted; the renderer's mapping in `render/tiles.rs` is the only one.

### Other widths

- **1856 x 512** (116 columns): seven full blocks plus a partial eighth. Ids 0 to 3583 (blocks 0 to 6) keep the same cells as on the 2048 sheet. In block 7 only columns `112..115` exist, so only ids with `t % 16 < 4` (128 of the 512) are on the sheet; the other **384 ids** are skipped by `blit_tile`, which silently draws nothing for a tile that does not fit wholly inside the sheet. The notes give 1856 wide for 1.33 to 1.39 (`documentation/assets.md`); how a real 1.39 client maps ids on it is still not settled, and the one 1856 sheet in the corpora (`pics1c.png`, a 2.3.1-era server image folder) is not evidence for it. Only the arithmetic is shown.
- A sheet **narrower than 2048** works for the ids whose cell is on it and draws nothing for the rest. A sheet of just block 0 (16 columns x 32 rows = 256 x 512) is enough for ids 0 to 511.
- **Tile sheets in the corpora:** 70 files of 2048x512 across 28 worlds (item 17), one 1856x512.

### How full the sheet is

In the two sheet files read pixel by pixel, 4,049 (Maloria) and 4,038 (Coronia) of the 4,096 cells hold more than one colour, and the per-block counts are 492 to 511 of 512 in every block. A real full sheet fills the whole grid. A default set does **not** need to: it only needs the cells its starter level and tile-type table use, and everything else may be one flat colour.

### Tiledefs

The 8 tiledef files in the corpora are 59 to 730 bytes. Their lines are `image,prefix,type,x,y` (parsed by `parse_tiledefs`). The `x` values seen in the Final-Doomsday file are `0, 288, 640, 1024, 1280, 1728`, all multiples of 16 and below 2048 (columns `0, 18, 40, 64, 80, 108`). Whether `x,y` is a pixel position on the base sheet is not settled (`documentation/assets.md`, "Tiledef matching"), so this note does not rely on it.

## 2. Palette types and the readers

Every PNG below was measured from its header and chunk list (colour type, bit depth, `PLTE` entry count, `tRNS` length, interlace). Seihai's `read_png_rgba` (`client/src/assets/png.rs`) reads non-interlaced gray, RGB, RGBA, gray+alpha and **palette PNGs at 1, 2, 4 and 8 bits with `tRNS`** into RGBA, so both palette depths below already go through the existing palette reader. There is no GIF or MNG decoder.

| File (world) | Size | Type | `PLTE` | `tRNS` | Reader |
|--|--|--|--:|--:|--|
| `pics1.png` (Maloria-03-04) | 2048x512 | palette, 8-bit | 256 | none (opaque) | `read_png_rgba` |
| `pics1.png` (Coronia-1, Hyroko; identical bytes) | 2048x512 | palette, 8-bit | 256 | 225 | `read_png_rgba` |
| `body.png` (Maloria-03-04, Final-Doomsday; identical bytes) | 128x720 | palette, **4-bit** | 16 | 3 | `read_png_rgba` |
| `sword1.png` (Maloria-03-04) | 128x96 | palette, 4-bit | 16 | 5 | `read_png_rgba` |
| `shield1.png` (Maloria-03-04) | 38x20 | palette, 4-bit | 16 | 3 | `read_png_rgba` |
| `shield1.png` (Final-Doomsday, custom) | 96x20 | palette, 8-bit | 32 | 1 | `read_png_rgba` |
| `head103.png` (Final-Doomsday) | 32x560 | palette, 8-bit (16 entries used) | 16 | 10 | `read_png_rgba` |
| `head118.gif` (Maloria-03-04) | 32x560 | GIF89a, 16-colour global table | n/a | transparent index | **no decoder** |
| `sword1.gif`, `shield1.gif` (Maloria-03-04) | 128x96, 38x20 | GIF89a, 16 colours | n/a | transparent index | **no decoder** |

Read: the stock sheets are **4-bit palette PNGs** (body, sword, shield) and an **8-bit 256-colour palette PNG** (`pics1.png`). Heads exist as 8-bit palette PNGs with 16 entries and as 16-colour GIFs (item 17 found the candidate worlds' heads as 32x560 GIFs, and 4-bit PNGs in Lumina-1). Truecolor is rare for these kinds (1,826 of 44,308 PNGs in the second corpus, none of the stock sheets). Transparency is carried by `tRNS` on palette entries, which `read_png_rgba` turns into alpha. `blit_tile` copies RGB only, so the tile sheet is read with `read_png_rgb` and tile transparency does not apply; the body, head and shield blits use RGBA and keep alpha.

### The PNG failure cases from item 17

Item 17 counted 99 PNG read failures out of 48,045: **77 interlaced** (Adam7), **18 truncated** ("truncated chunk"), plus 4 others (2 not PNGs, 1 1-bit gray, 1 too short). A header-only recount for this item, by PNG signature across all file names (so it also sees `.PNG` and `.png.tmp`), found 79 interlaced files and 27 whose chunk list is cut off before `IEND`. The difference to 77 and 18 is the name filter and was not reconciled further.

- **Interlaced (79):** all 8-bit; 32 RGBA, 31 palette, 16 RGB. 44 are in Lumina-1 and 17 in Final-Doomsday. None is a 2048x512 sheet. Five are `128x720` body sheets and several are 32x560 heads and shield or sword sheets. The reader refuses them ("unsupported PNG bit depth or interlace"). Fixing that needs an Adam7 pass in `read_png_rgba`; none of the stock sheets needs it.
- **Truncated (27, 17 of them in Lumina-1):** the chunk list ends inside a chunk. They include 3 body sheets (128x720), 5 heads (32x560) and about ten tile sheets of 288x288 up to 2048x512 (exactly one, `dreamscapetiles.png`, is 2048x512). Nothing to repair; they are unusable as sources and are skipped.
- The default set will be written non-interlaced, so neither case applies to it.

## 3. Best sources per kind, and what a default set must match

Where each kind is best read from, what clean-up the real file would need, and the layout a drawn replacement has to follow. All measurements are in `asset-reference.tsv`.

| Kind | Best source (world, file) | Size and format | Clean-up the source would need |
|--|--|--|--|
| Tile sheet | Maloria-03-04 `pics1.png` (261,789 bytes); also Coronia-1 and Hyroko `pics1.png` (210,232 bytes) | 2048x512, 8-bit palette, 256 entries | none to read it; Coronia's/Hyroko's `tRNS` of 225 entries means palette-transparent tiles |
| Body sheet | Maloria-03-04 `Levels/Bodies/body.png`; Final-Doomsday `levels/bodies_dd/body.png` (same 9,169 bytes, identical) | 128x720, 4-bit palette (16 entries, 3 `tRNS` bytes) | none |
| Head sheet | Final-Doomsday `levels/heads_dd/head103.png` (PNG); Maloria-03-04 `Levels/Heads/head118.gif` (GIF) | 32x560; 8-bit palette (16 entries) or 16-colour GIF | the GIF needs a decoder or a conversion to PNG |
| Sword sheet | Maloria-03-04 `Levels/Swords/sword1.png` (and `.gif`) | 128x96, 4-bit palette | none |
| Shield sheet | Maloria-03-04 `Levels/Shields/shield1.png` (and `.gif`) | 38x20, 4-bit palette | none (Final-Doomsday's 96x20 `shield1.png` is a custom sheet, not the stock layout) |
| GANIs | Maloria-03-04 `Levels/Ganis/` and Final-Doomsday `levels/ganis_dd/`: `idle`, `walk`, `swim`, `sword`, `sit` (every one byte-identical between the two worlds: 7,988 / 10,031 / 9,288 / 10,377 / 7,871 bytes) | text, no magic line | none; `Gani::parse` reads all ten (frame counts below) |
| Footsteps | Final-Doomsday `levels/sounds_dd/sfx/steps.wav` and `steps2.wav`; Doomsday-Nov-2002 `levels/steps2.wav` | 22,050 Hz, mono, 8-bit PCM, 4,045 samples (183 ms), 4,089-byte files | none; `decode_wav` reads them |
| Water | Final-Doomsday `levels/sounds_dd/sfx/water.wav` (7,052 bytes); Doomsday-Nov-2002 `levels/water.wav` (7,072) | 8,000 Hz, mono, 16-bit PCM, 3,504 samples (438 ms) | none |
| Sword swing | Doomsday-Nov-2002 `levels/sword.wav`, also in Coronia-1, Dark-Diamond-1, Lumina-1 (4,712 or 4,713 bytes) | 8,000 Hz, mono, 8-bit PCM | header measured only; not staged |

### Sheet sizes and grids a default set must match

- **`pics1.png`**: 2048x512, 128 columns x 32 rows of 16x16 tiles, the id mapping of section 1. Palette PNG is fine, 4-bit or 8-bit, non-interlaced.
- **`body.png`**: **128x720**. Frames are 32x32 cells in **four columns, one per direction** (`x = 0, 32, 64, 96` = up, left, down, right, Graal order 0 to 3), stacked in **rows of 32 px**. The stock animations use 22 distinct row offsets, `y = 0, 32, ... , 672` (22 x 32 = 704, and the last sprites are 48 px tall, so 672 + 48 = 720). Sprite sizes used on the sheet are 32x32, 32x48 and 48x32. In the GANIs, body sprite id = `200 + 4 * pose + dir`, with the rectangle `(32 * dir, 32 * pose, 32, 32)`; pose 0 is the idle pose (ids 200 to 203), and the walk uses poses 1 to 5 at `y = 32 ... 160` (ids 206, 210, 214, 218, 222 for facing down). A default body sheet needs at least the idle row and the walk poses; `draw_player_sprites` itself reads only row 0 (`x = dir * 32`, `y = 0`).
- **Head sheets**: **32x560**, one 32-px column. Head sprite id = `100 + dir` at `(0, 32 * dir, 32, 32)` for the four directions (rows `y = 0, 32, 64, 96`); the stock animations use 17 head sprites in all, of sizes 32x32 and 32x48 (the gap from `y = 128` to `y = 176` implies one 48-px sprite there, then every 32 px up to `y = 528`, ending at 560). `draw_player_sprites` reads `(0, dir * 32, 32, 32)`.
- **Sword sheet**: 128x96, 17 sprites in the sword animation, sizes between 10x22 and 34x12. Max extent 128x96.
- **Shield sheet**: 38x20, four sprites at `x = 0, 6, 14, 30` of sizes 6x18, 8x20, 16x20 and 8x20.
- **Hat/attr1 sheet** (`DEFAULTATTR1`): 192x144 in 48x48 cells (4 x 3), 9 sprites used per animation (`x = 0, 48, 96, 144`, `y = 0, 48, 96`). Optional for a first default set; an animation naming a sheet it cannot find draws nothing.
- **`sprites.png`**: the shadow is 24x12 at `(0, 0)`; the swim ripple is 32x16 at `(0, 306)` and `(0, 322)`, so the sheet is **at least 32 x 338**; the carried objects and break pieces of `throwing.md` need **128 x 402** (the 2.22 table's extent is 1016 rows, those are only the rectangles Seihai draws). `AssetResolver`/`--check-assets` list `sprites.png`, `state.png` (HUD), `pics1.png`, `body.png`, `head0.png` as required for every version.
- **Symbolic sheet names in GANIs:** `BODY`, `HEAD`, `SHIELD`, `SWORD`, `ATTR1`, `SPRITES` stand for the sheet files; the `DEFAULT*` header lines supply their file names (the stock ones name a body, a head and a hat file and a sword file).

### GANI frame structure

Measured with `Gani::parse` and a separate header count, identical for Maloria-03-04 and Final-Doomsday.

| GANI | `SPRITE` lines | Frames (4 direction rows each) | Sound cues | Other |
|--|--:|--:|--|--|
| `idle` | 117 | 1 | none | five placements per row |
| `walk` | 117 | 8 | `steps2.wav` after frames 0 and 4 | `LOOP`, `CONTINUOUS` |
| `swim` | 119 | 8 | `water.wav` after frame 0 | `LOOP`, `CONTINUOUS`, `WAIT 1` after every frame |
| `sword` | 134 | 5 | `sword.wav` after frame 0 | `SETBACKTO idle`, `DEFAULTPARAM1` names a sword file |
| `sit` | 117 | 1 | none | |

- A frame is **four consecutive rows**, in the order up, left, down, right; a row is a comma-separated list of placements `sprite_id x y`. A frame's `PLAYSOUND name x y` lines follow its four rows. `x y` is `1.5 2` in every stock cue, the player's own listening point (`LISTENER_DX`, `LISTENER_DY`).
- The idle row has five placements: shadow (`SPRITES` id 0), shield (ids 10 to 13), body (ids 200 to 203), head (ids 100 to 103) and hat (ids 40 to 43). The shadow, body and head offsets are `(12, 34) (8, 16) (8, 2)` facing up, `(12, 36) (12, 16) (12, 2)` left, `(12, 34) (8, 16) (8, 2)` down and `(12, 36) (4, 16) (4, 2)` right. These are exactly the `OFFSETS` table in `draw_player_sprites`, so the measured file and the code agree. The sprite box is 48x48 (3x3 tiles), and its top-left is the player position.
- Walk: facing down, the body y offset over the eight frames takes the values 14, 16 and 20, and the head y offset the values 0, 2 and 4 (the body and head bob together by a few pixels).
- Swim: no body is placed. The row is the ripple (id 20 on `sprites.png`), the head and the hat. The head y offset is 22 facing up and 18 facing left, down and right, against 2 in idle, so the head sits 20 px (up) or 16 px (other directions) lower. Seihai's `draw_player_sprites` lowers the head by a flat **14 px** in water (`h.1 + 14`). That is a 2 to 6 px difference from this animation. Which build's `swim.gani` the 14 came from, and whether a lab capture settled it, is not recorded here; flagged for item 20, not changed.
- Loading all ten files through `Gani::parse` gives 117, 117, 119, 134, 117 sprites and 1, 8, 8, 5, 1 frames for `idle`, `walk`, `swim`, `sword`, `sit`, each with four rows per frame and `single_direction` false.

### Sounds

- The stock effects are **plain PCM WAV, mono**: footsteps 22,050 Hz 8-bit, water 8,000 Hz 16-bit, sword 8,000 Hz 8-bit. `decode_wav` reads all of these.
- **IMA ADPCM** (format tag `0x11`, 4 bits per sample, block alignment 256 at 8,000 Hz and 512 at 22,050 Hz) is rare: 8 files in the corpora (Lumina-1 6, Galactic-Nations 1, Outworld 1). The reader supports it, but a default set has no need for it.
- Not read: MP3-in-WAV (tag `0x55`, 8 files) and MS ADPCM (tag 2, 2 files) are unsupported (item 17).
- A default set of short mono PCM files at 22,050 Hz 8-bit (steps) and 8,000 Hz 16-bit (water) matches the stock formats. Names to provide are those `walk.gani`, `swim.gani` and `sword.gani` name: `steps2.wav`, `water.wav`, `sword.wav` (plus `steps.wav` for the builds without GANIs; the 2.05 list in `coverage.rs` is `arrow axe bomb crush extra fire horse2 item lift lift2 put steps sword thunder water`, all optional).

## 4. What Seihai's own code fixes for the default tile sheet

From the client code, not from the corpora.

- **Grid:** `TILE_PX = 16`, `SHEET_COLS = 128`, `SHEET_ROWS = 32` (`render/tiles.rs`). The 128-column assumption is in `blit_tile` (`col = t % 16 + (t / 512) * 16`, `row = (t / 16) % 32`), and `render_view` is handed the sheet size; a tile that does not fit wholly inside the loaded sheet is skipped without a message. So a default `pics1.png` is **exactly 2048 x 512** for every id to resolve, or at least covers the cells of the ids used.
- **Ids:** a level board is 64 x 64 tiles. In `.nw` an id is two base64 characters (`0` to `4095`; `AA` is 0). Binary `.graal` levels use 11, 12 or 13 bits per code (`GR-V1.02` to `.04` use 13), and from 2048 up the file magic matters; a starter level in text form keeps ids below 4096 and, to be safe for any narrower sheet, below 512.
- **Collision table:** one type byte per id (4096 entries), from `tiledefs.txt` (flat list or `index,x,y,type,name` rows) or `arrays.dat`. Type `>= 20` is a wall (`TYPE_WALL_MIN`), `11` is water (`TYPE_WATER`), `12` is lava, everything else is open floor. An id that `arrays.dat` does not list as passable gets type 22. The default set therefore needs a table whose bush, rock and wall ids are `>= 20` and whose water ids are `11`.
- **Foot point (links, water; walls use the body box, see `walking-collision.md`):** `FOOT_DX = 2.0`, `FOOT_DY = 2.75` tiles from the sprite's top-left (`offline/mod.rs`), i.e. pixel (32, 44) inside the 48x48 sprite box. The tile under the feet is `floor(x + 2.0)`, `floor(y + 2.75)`; the original client truncates the point to a tile, so tile 63 runs from 63.0 to 64.0 and a point off the 64x64 board is a wall. The wall, water and link tests all use this one point. Whole tiles decide blocking, so wall and water art must fill their 16x16 cell and a wall tile has to be a wall in the type table, not just look like one. Chests block as a 2x2 tile box tested with the same point.
- **Swim:** a foot point on a type-11 tile starts swimming (half walking speed, `SWIM_STEP_MS = 100` against `WALK_STEP_MS = 50`), and `draw_player_sprites` then draws the head only.
- **Player art read by the code:** `body.png` frame `(dir * 32, 0, 32, 32)`; head frame `(0, dir * 32, 32, 32)`; shadow `sprites.png` `(0, 0, 24, 12)`; offsets as in section 3. Sheets are read as RGBA with palette transparency.
- **Files the resolver looks for:** `pics1.png`, a tile-type table, `state.png`, `sprites.png`, `bodies/body.png` (or `levels/bodies/`), `heads/head0.png`, `walk.gani`, `swim.gani` (required where the version has GANIs), `idle.gani` and `sword.gani` (optional), and `levels/new1.nw` as a sample level (`assets/coverage.rs`). `state.png` is 224x145 in the 2.22 folder (`documentation/assets.md`) and 224x97 for 1.38+ profile layouts, so the HUD sheet is version-dependent.

## 5. Not settled

- How a real 1.33 to 1.39 client maps ids on the 1856-wide sheet (only the arithmetic above is shown).
- Which of the stock sheets are "clean" in terms of licence: all are player-made or community-made; that is why the default set is redrawn.
- The swim head offset (14 px in code against 16 and 20 in this animation).
- The tile-id positions of any particular terrain (grass, bush and so on) are not recorded here; item 19 chooses its own ids and writes its own tile-type table.
- GIF and MNG: a decoder is not planned for the default set (it will be PNG).
- The `tile_src_rect` mismatch in section 1 (since fixed: the function was deleted).
