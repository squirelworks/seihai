# Level editor (item 24)

Seihai will get a level editor that follows the classic one by version, the way the client follows its version chain: **2.22 first**, other versions as small differences on top. This page is the tracking list (copied from the analysis notes in Graal_analysis `tools/level-editor/`, in Seihai's own words and order) and the notes for the part that exists: a command-line tool in `tools/level-editor/` that opens a `.nw` text level, lists its tiles, paints a rectangle, and saves. There is no GUI.

The notes in Graal_analysis are derived facts about the editor's behaviour (no code); nothing from a decompile is repeated here. Tags in the status column: **done** in this slice, **next** is the natural following step, **later** is not started.

## The tool

```
level-editor [--version V] list  FILE.nw [--rows]
level-editor [--version V] paint FILE.nw X Y W H TILE (--out FILE | --in-place)
level-editor [--version V] save  FILE.nw (--out FILE | --in-place)
level-editor versions
```

- `list` prints the summary (rows placed and skipped, counts of links, signs, chests, baddies and NPCs, whether a `HEIGHTS` block is there, the ten most used tile ids). `--rows` also dumps the board, one line per row, each tile as three hex digits. `level-editor FILE.nw` alone is `list`.
- `paint` fills a rectangle (top-left corner `X Y`, size `W H`, in tiles) with one tile id. The rectangle must lie inside the 64 x 64 board and the id must be 0 to 4095 (decimal or `0x` hex). A bad rectangle or id is an error and nothing is written. Painting the tile that is already there changes nothing and the file is written back unchanged.
- `save` opens a level and writes it again. With no change the bytes are the same.
- Writing needs an explicit `--out FILE` or `--in-place`, so a paint never overwrites a level by accident. Files are written through a temporary file in the same folder and a rename, so an interrupted save cannot leave half a level. (2.22 deleted the old file first; the notes say 2.3 changed this.)
- `--version` defaults to 2.22. 2.17.1 is accepted as the same profile (the notes found no difference). Every other version stops with "not yet supported" and says why: 2.05 has no `HEIGHTS` block, no predefined NPCs and a different open filter; builds before 2.05 have no text level format at all; 2.31 has no built-in editor. Binary levels (`GR-V1`, `Z3-V1`) say "not yet" too.

The old tool printed an ASCII preview and had `--export`; its export rebuilt the file from Seihai's parsed level and lost everything the parser drops (comments, `HEIGHTS`, line endings), so it was replaced by a save that keeps the file's own lines.

### How the text file is read and written

Rules taken from the 2.22 notes:

- The first line is `GLEVNW01`. The board is one 64 x 64 grid of 12-bit ids (two base64 characters each, high character first); index = y x 64 + x.
- `BOARD x y w layer data`: `x` and `y` 0 to 63, `w` at least 1, `x + w` at most 64, and `data` at least `2 x w` characters. **The layer field is not read**: a row with layer 1 is placed on the same board. A row that does not fit is skipped silently, an invalid character ends a row (the tiles before it stay), rows may come in any order and later rows overwrite earlier ones. The board starts as all zeros.
- `NPC ... NPCEND`, `SIGN ... SIGNEND`, `BADDY ... BADDYEND` and `HEIGHTS ... HEIGHTSEND` swallow every line up to their end line, so a script line that looks like a `BOARD` row is not one.

**Round trip (Seihai's choice).** The editor keeps the file as its original lines (with their line endings). A level that was not changed is written back **byte for byte**. After a paint, the `BOARD` rows are replaced by 64 canonical rows (`BOARD 0 y 64 0 <128 characters>`, the shape the 2.22 writer emits) put where the first old `BOARD` row was (right after the header when there was none), and every other line is written back as it was: comments, links, signs, chests, baddies, NPC scripts, `HEIGHTS`, CRLF endings, a missing final newline. The two starter levels are already in the canonical shape, so painting a tile and then painting the old tile back gives the starter's bytes again (tested).

Not what the 2.22 writer does: it rebuilds the whole file from its own model and writes `HEIGHTS` only when the height overlay was edited. Keeping the other lines verbatim is a deliberate difference, so that a first edit of a hand-written or server-made level shows only the board in a diff.

### Tests

`tools/level-editor/tests/editor.rs` (18 tests): the starter levels `new1.nw` and `new2.nw` and a synthetic level (comments, a `BOARD`-looking script line, `HEIGHTS`, partial, short and out-of-range rows, a layer-1 row, links, signs, chests, baddies) round trip byte for byte in LF, CRLF and no-final-newline forms; the row rules; a paint changes only the rectangle and keeps every other line; Seihai's own `Level` parser reads what the editor saved; every id from 0 to 4095 survives a save and read; bounds and id range; version selector; and the command line (list, row dump, paint to a new file or in place with no temp file left, save, error messages and exit codes). No corpus level is used.

## Tracking list: the 2.22 feature list in implementation order

| # | Feature (what the 2.22 editor does) | Status |
|---|---|---|
| 1 | **Level model and tile sheet.** A 64 x 64 board of tile ids, plus links, signs, chests, NPCs and baddies. Id to sheet: `col = (id % 16) + (id / 512) * 16`, `row = (id / 16) % 32` | board **done** (read, paint, write); the other lists are only counted and kept as text |
| 2 | **Read and write GLEVNW01 text**, including `LINK`, `SIGN`, `CHEST`, `NPC`, `BADDY` and the 9 x 9 `HEIGHTS` block, with a round-trip test | **done** for the board; other records round trip verbatim; editing them is **next** |
| 3 | **Read the binary levels** (`GR-V1.00` to `1.03`, `Z3-V1.03/04`) and **write `GR-V1.03`**, picking the format by file extension (`.nw` text, anything else binary) | **later** (the client has a binary reader; the editor says "not yet") |
| 4 | **Board view**: scrolling, 16 px tiles, overlays for blocking, foreground and terrain tiles, a status bar | **later** (`list --rows` is the text stand-in) |
| 5 | **Selection, copy, cut, paste, delete, floating paste** | **later** (`paint` is the single-tile fill) |
| 6 | **Undo** of board writes: per tab, 128 records, rectangle snapshots | **later** |
| 7 | **Tile palette**: one tile or a block, the background tile, draw mode, flood fill on the right button | **later** (`paint` is a rectangle of one tile) |
| 8 | **Stamps**: load `GOBJSET01` sets, place, add the selection to a set, save the set | **later** |
| 9 | **Link tools**: list and dialog, *turn selection into a warp field* (`playerx` / `playery` defaults, the edge rules), open the destination in a new tab | **later** |
| 10 | **Sign tools**: list, x/y, text in the sign alphabet, signs at (0, 0) | **later** |
| 11 | **Chest tool**: the 25 items, sign index with -1 for none | **later** |
| 12 | **Baddy tool**: ten types, three verses, types 4 to 6 without verses | **later** |
| 13 | **NPC tool**: image, script with test and find, clone, predefined NPCs (`defnpc*.txt`) | **later** |
| 14 | **Tabs, Save (save-all), Save as, Close with a prompt, New.** Write to a temporary file and rename, not delete-then-write | save through temp file and rename **done**; the rest **later** (a new level can be made from `assets/default/levels/new1.nw` by `save --out`) |
| 15 | **Keys**: the key map of the editor, `F10` border toggle, optional `Ctrl+Q` batch re-save | **later** |
| 16 | **Maps**: several levels at once, the level cache, gmap generation | **later** |
| 17 | **Tile definitions**: `tiledefs` lines, types 0 and 1; type 4 once its meaning is traced | **later** (the client already reads `tiledefs`) |
| 18 | **Play** the level; needs the client | out of scope for the editor |
| 19 | **Editor control NPCs** (`editnpc*.txt` with mouse variables); needs the script engine | **later**, last |

## Per-version differences (profile switches, none applied yet)

The suggested profile fields: formats read, formats written by extension, has chests, has baddies, has heights, has predefined NPCs, has level cache, tile definition support, open filters, load clamps, dialogs. Today there is one profile; `Version` in `tools/level-editor/src/lib.rs` is where they go.

| Version | Difference from 2.22 | In the tool |
|---|---|---|
| 2.17.1 | none found | same profile, accepted |
| 2.05 (rev 1 and 2) | no heights, no level cache, no tile-array or log windows, no predefined NPCs, open filter `*.graal` only, image and script in separate dialogs, the text save has no `HEIGHTS` | "not yet" |
| 2.04 | as 2.05 and **no text format at all**; chest load clamp 0 to 62, NPC 0 to 63; object sets and tile definitions present | "not yet" |
| 2.00 to 2.0 rev 3 | as 2.04 without object sets and tile definitions; revisions 0 to 2 also open `*.zelda` | "not yet" |
| 1.34 to 1.41 | as 2.00; the baddy class is called baddy; draw mode and the map key arrive in 1.41 | "not yet" |
| 1.28b2 to 1.33 | baddy class named opponent | "not yet" |
| 1st to 2nd Graal and earlier | no level class, no character dialog before 1.28b2; 1st Graal has no chests or NPC dialog | "not yet" |
| New World betas | no chests, no baddies; heights and a map in beta 1; the text format is described by the beta's own format file | "not yet" |
| 2.1 test builds | merged NPC dialog, predefined NPCs, no sign-code window | "not yet" |
| 2.31 and GraalEditor 2.3.1.1 | the editor is a separate program (the 2.31 client has no editor forms; its start screen's *Editor* button launches `GraalEditor.exe`, seen live); writes text with `HEIGHTS`; an older note claims the board there is decimal (unchecked) | "not yet" |

2.05 is the first one worth doing: the board rules are the same, so the difference is mostly what the editor must refuse to write (a `HEIGHTS` block).

## Open points (from the notes, still open)

- The mouse state machine (drag, drop, double-click by kind): known only by shape.
- What the two session flags that gate chest and baddy load and save mean.
- Tile definition type 4 and its x/y values.
- Whether arrival offsets such as `playery+10` in a link are evaluated at warp time (the editor only stores the text).
- The GraalEditor board format (decimal or base64) against a real file.

Closed since the first slice (live checks in Graal_analysis `client-internals/live-checks-editor-offline-sound.md`, 2026-10-30): pressing *Play* on a named, modified level saves it first, as the 2.22 notes say; *Generate map* walks the loaded big map's level list and does not follow links; *Edit flags* edits the in-memory flag list only and is never written to disk. None of these changes the CLI.

## Notes found while doing this

- The tile-to-sheet formula in the checklist is the one the renderer uses (`render/tiles.rs`). The older `tile_src_rect` (`id % 128` / `id / 128`) has been deleted.
- The text format holds ids 0 to 4095 only. Binary levels from player worlds use higher ids; editing those needs the binary reader and writer (item 3 above).
