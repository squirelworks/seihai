# Level editor (item 24)

Seihai will get a level editor that follows the classic one by version, the way the client follows its version chain: **2.22 first**, other versions as small differences on top. This page is the tracking list (copied from the analysis notes in Graal_analysis `tools/level-editor/`, in Seihai's own words and order) and the notes for the part that exists: a command-line tool in `tools/level-editor/` that opens a `.nw` text level, lists its tiles, paints a rectangle, and saves, and a window on top of it (`level-editor gui`): a tile palette, painting on layer 0 or 1, stamps from an object set, link / sign / chest / NPC markers, a `.gmap` grid that opens its levels, and save.

The notes in Graal_analysis are derived facts about the editor's behaviour (no code); nothing from a decompile is repeated here. Tags in the status column: **done** in this slice, **next** is the natural following step, **later** is not started.

## The tool

```
level-editor [--version V] list  FILE.nw [--rows]
level-editor [--version V] paint FILE.nw X Y W H TILE [--layer L] (--out FILE | --in-place)
level-editor [--version V] save  FILE.nw (--out FILE | --in-place)
level-editor [--version V] [--sheet PNG] gui [FILE.nw | FILE.gmap]
level-editor versions
```

- `list` prints the summary (rows placed and skipped, counts of links, signs, chests, baddies and NPCs, whether a `HEIGHTS` block is there, how many layer-1 cells are filled and how many rows of other layers are kept, the ten most used tile ids). `--rows` also dumps the board, one line per row, each tile as three hex digits. `level-editor FILE.nw` alone is `list`.
- `paint` fills a rectangle (top-left corner `X Y`, size `W H`, in tiles) with one tile id, on layer 0, or on layer 1 with `--layer 1`. The rectangle must lie inside the 64 x 64 board and the id must be 0 to 4095 (decimal or `0x` hex). A bad rectangle or id is an error and nothing is written. Painting the tile that is already there changes nothing and the file is written back unchanged.
- `save` opens a level and writes it again. With no change the bytes are the same.
- Writing needs an explicit `--out FILE` or `--in-place`, so a paint never overwrites a level by accident. Files are written through a temporary file in the same folder and a rename, so an interrupted save cannot leave half a level. (2.22 deleted the old file first; the notes say 2.3 changed this.)
- `--version` defaults to 2.22. 2.17.1 is accepted as the same profile (the notes found no difference). Every other version stops with "not yet supported" and says why: 2.05 has no `HEIGHTS` block, no predefined NPCs and a different open filter; builds before 2.05 have no text level format at all; 2.31 has no built-in editor. Binary levels (`GR-V1`, `Z3-V1`) say "not yet" too.

The old tool printed an ASCII preview and had `--export`; its export rebuilt the file from Seihai's parsed level and lost everything the parser drops (comments, `HEIGHTS`, line endings), so it was replaced by a save that keeps the file's own lines.

### The window

`level-editor gui [FILE.nw | FILE.gmap]` opens a window (egui, the toolkit the client's launcher already uses; no new dependency for the build). Everything it writes goes through the same `Document` as the command line, and every record it reads goes through the client's own parsers (`Level` for links, signs, chests and NPCs, `ObjectSet` for `GOBJSET01`, `Gmap` for `GRMAP001`); there is no new format work.

- **Board.** The 64 x 64 board drawn with the tile sheet, 16 px per tile, scrollable, zoom 1x or 2x, an optional grid. The status bar shows the cell under the mouse with its layer-0 and layer-1 tiles.
- **Tile palette.** The whole sheet (`pics1.png` layout, 2048 x 512) below the board; a left click selects the tile under the mouse and a yellow box marks the selection. The tile id can also be typed in the status bar (0 to 4095). The sheet is the built-in `pics1.png` unless `--sheet PNG` names another.
- **Layers.** *Layer 0 / 1* picks the active layer: painting, erasing and stamps go there, and the right button picks from it. *Show 0 / 1* hides or shows each layer (a hidden layer 0 is drawn dark grey). Layer 1 is drawn over layer 0 where it has a tile.
- **Tools.** *Paint*: the left button paints the selected tile on one cell; dragging paints a stroke (cells skipped between two frames are filled in along a straight line). *Erase* (layer 1 only): empties cells the same way. *Stamp*: see below. *Objects*: see below. In every tool the right button picks the tile under the mouse on the active layer.
- **Stamps.** With the Stamp tool a side panel loads a `GOBJSET01` object set from a typed path and lists its stamps with their sizes. The picked stamp follows the mouse, see-through, and a left click drops it with its top-left corner on the cell, on the active layer. Cells that fall off the board are left out.
- **Objects.** Links (cyan, their own rectangle), signs (yellow, 2 x 1), chests (orange, 2 x 2), NPCs (magenta, 2 x 2) and baddies (red, 2 x 2) are drawn as markers on the board in every tool. With the Objects tool a click on a marker selects it and dragging moves it in whole-tile steps (an NPC at 12.5 stays on the half tile); a click on an empty cell deselects and sets where the *+ link / + sign / + chest / + NPC* buttons put a new one. The side panel lists every object and edits the selected one: x and y (*Move*), a link's size, destination and arrival (`30`, `playerx`, `playery+10`), the first line of an NPC script or of a sign's text, and *Delete*. Baddies are shown but not edited. A record the client's parser cannot read is shown and left alone.
- **NPC script pane.** With the Objects tool, a double-click on an NPC marker opens a window with the first 20 lines of its script (the lines between the `NPC` line and `NPCEND`). *Apply* puts the text back into the level in place of the lines it showed; *Apply and save level* also writes the file. Lines after the 20th are kept as they are, so a longer script is never cut. More than 20 lines, or a line that is `NPCEND`, is refused and nothing changes. Each new line keeps the line ending of the line it replaces. This is not a GS1 editor: no highlighting, test, or find.
- **Gmaps.** Opening a `.gmap` shows its grid of level names; a cell whose file is not next to the `.gmap` is greyed out and a hole says so. A click opens that level (asking first about unsaved changes) and the *Gmap* button in the toolbar goes back to the grid. Only one level is open at a time.
- **Save.** *Save* (or Ctrl+S) writes to the file that was opened; *Save as* writes to the path in the box next to it, which then becomes the Save target. Both write through a temporary file and a rename and keep the round-trip rules below (an unchanged level is written back byte for byte). *Open* takes a path typed in the box (there is no file dialog). Opening another level or closing the window with unsaved changes asks Save / Discard / Cancel; the title starts with `*` while there are unsaved changes.
- **Version selector.** Defaults to 2.22 (or the `--version` given); it lists the versions the command line accepts, 2.22 and 2.17.1, which are one profile. Other versions are not offered (they stop with "not yet" on the command line).

Not in the window (later items of the list below): undo, selection and copy/paste, rectangle and flood fill, blocking or tile-type overlays, layers above 1, editing baddies and chest items, the full NPC script and sign text, NPC images on the board, saving an object set, tabs, New, binary levels, editing several gmap levels at once.

### Gonstruct as a reference

The Gonstruct 0.2.3 source (December 2014, in the project files under `graal-decompile/unpacked-installers/gonstruct/`) was read to see which features a later editor has and how they are arranged. No code was copied.

| Gonstruct behaviour | In Seihai |
|---|---|
| Layers: an active layer and a visibility toggle per layer; layer 0 starts as tile 0, higher layers start transparent | **matched** for layers 0 and 1 (Gonstruct has a spin box up to 50 and add / delete layer buttons: **skipped**) |
| Saving a layer above 0 as one `BOARD x y w layer` row per run of non-transparent tiles | **matched** for layer 1; layer 0 stays the 64 canonical rows |
| Rows of layers it does not show | Gonstruct keeps every layer; Seihai keeps rows of layers 2 and up verbatim but does not show or edit them |
| Tile objects (stamps) kept in groups in its preferences, with new / delete / save | **different source**: Seihai loads a `GOBJSET01` file with the client's parser; creating and saving stamps **skipped** |
| NPCs: click selects, drag moves in whole tiles, double click opens an image and script dialog, right-drag clones | select and drag **matched**; double click opens a script pane for the first 20 lines (no image field, no full script dialog), no clone, no NPC image on the board |
| Links and signs: drawn on the board, edited in list dialogs (destination, new x / y; sign text) | **matched** as markers plus a side-panel list; links also move by drag; sign text is first line only |
| Chests and baddies | Gonstruct does not read them and drops them on save (found by the archive thread); Seihai shows both, moves and adds chests, and keeps baddies as they are |
| A `.gmap` opened as one big editable map of all its levels | **skipped** by request: Seihai shows the grid and opens one level per click |
| Undo for tiles and NPCs, selection copy / paste, flood fill | **skipped** (later items below) |

### How the text file is read and written

Rules taken from the 2.22 notes:

- The first line is `GLEVNW01`. The board is one 64 x 64 grid of 12-bit ids (two base64 characters each, high character first); index = y x 64 + x.
- `BOARD x y w layer data`: `x` and `y` 0 to 63, `w` at least 1, `x + w` at most 64, and `data` at least `2 x w` characters. **The layer field is not read**: a row with layer 1 is placed on the same board. A row that does not fit is skipped silently, an invalid character ends a row (the tiles before it stay), rows may come in any order and later rows overwrite earlier ones. The board starts as all zeros.
- `NPC ... NPCEND`, `SIGN ... SIGNEND`, `BADDY ... BADDYEND` and `HEIGHTS ... HEIGHTSEND` swallow every line up to their end line, so a script line that looks like a `BOARD` row is not one.

**Layers (Seihai's choice, as Gonstruct).** Besides the 2.22 view (every row on one board, which `list`, `tile` and `row` show), the editor keeps layer 0 (rows with layer 0; starts as tile 0) and layer 1 (rows with layer 1; starts empty) apart, and rows of any other layer as their own lines. A real 2.22 client still does not read the layer, so in 2.22 a layer-1 tile simply replaces the layer-0 tile under it.

**Round trip (Seihai's choice).** The editor keeps the file as its original lines (with their line endings). A level that was not changed is written back **byte for byte**. After a paint, the `BOARD` rows are replaced by 64 canonical rows of layer 0 (`BOARD 0 y 64 0 <128 characters>`, the shape the 2.22 writer emits), then one `BOARD x y w 1 data` row per run of filled layer-1 cells, then the rows of other layers as they were, all put where the first old `BOARD` row was (right after the header when there was none), and every other line is written back as it was: comments, links, signs, chests, baddies, NPC scripts, `HEIGHTS`, CRLF endings, a missing final newline. The two starter levels are already in the canonical shape, so painting a tile and then painting the old tile back gives the starter's bytes again (tested).

Not what the 2.22 writer does: it rebuilds the whole file from its own model (one board, so layer-1 rows end up flattened into layer 0) and writes `HEIGHTS` only when the height overlay was edited. Keeping layer 1 and the other lines is a deliberate difference, so that a first edit of a hand-written or server-made level shows only the board in a diff.

**Object edits.** Moving or editing a link, sign, chest or NPC rewrites only that record's lines (the header line with single spaces, or the one text line), keeping the file's line ending; the board rows are not rewritten for it. New objects are added at the end of the file: `LINK level.nw x y 2 2 playerx playery` (clipped to the board), `SIGN x y` with an empty line, `CHEST x y greenrupee -1`, `NPC - x y` with an empty line. A changed link is read back with the client's parser before it is written.

### Tests

`tools/level-editor/tests/editor.rs` (18 tests): the starter levels `new1.nw` and `new2.nw` and a synthetic level (comments, a `BOARD`-looking script line, `HEIGHTS`, partial, short and out-of-range rows, a layer-1 row, links, signs, chests, baddies) round trip byte for byte in LF, CRLF and no-final-newline forms; the row rules; a paint changes only the rectangle and keeps every other line; Seihai's own `Level` parser reads what the editor saved; every id from 0 to 4095 survives a save and read; bounds and id range; version selector; and the command line (list, row dump, paint to a new file or in place with no temp file left, save, error messages and exit codes). No corpus level is used.

`tools/level-editor/tests/gui.rs` (9 tests): the sheet mapping covers every id 0 to 4095 on its own cell and inverts; drag strokes have no gaps; the editor state paints, picks, refuses bad cells and ids, saves and saves as (painting the old tile back gives the starter's bytes); a failed open keeps the current level; the board image matches the sheet. The real window code is also driven headless (an egui context with synthetic mouse and key events, no display): a palette click then a board click paints, Ctrl+S saves to disk, a drag paints a stroke, a right click picks, a close with unsaved changes is held for the prompt, and no board is drawn without a level.

`tools/level-editor/tests/layers_objects.rs` (15 tests): a level with layer-1 runs, a layer-2 row and every kind of object reads its layers apart and keeps the 2.22 view, round trips byte for byte untouched, and after painting and erasing saves both layers (layer 1 in runs, empty cells left out, the layer-2 row as it was) so that the editor and the client's `Level` read them back; objects match the client's counts and fields; a move, a destination, arrival and size change, and a sign's first line each change only their own line; NPCs keep a half-tile position and get their first script line replaced; CRLF stays CRLF; objects of each kind are added (and read by the client) and deleted; a paint followed by an object edit keeps both; stamps load, drop on the active layer and are clipped at the board edge; erase and pick follow the active layer; gmap cells name the files next to the `.gmap`; and, headless, a click paints layer 1 and saves it, the Stamp tool drops a stamp, the Objects tool drags a link and selects and deselects, and a gmap cell click opens its level while a missing cell does nothing. The CLI's `--layer 1` paints and `list` reports it.

`tools/level-editor/tests/script_pane.rs` (6 tests): the pane reads the first 20 lines of a 25-line script and its total; links and signs have no script; a save replaces the shown lines and keeps lines 21 to 25 with nothing else in the file changed; an empty script gets lines before `NPCEND` and can be emptied back to the original bytes; 21 lines, a line break or an `NPCEND` line is refused with the file unchanged; CRLF stays CRLF; and, headless, one click on an NPC only selects, a double-click opens the pane without moving it, a double-click on a link opens nothing, and *Apply* then save writes the script.

## Tracking list: the 2.22 feature list in implementation order

| # | Feature (what the 2.22 editor does) | Status |
|---|---|---|
| 1 | **Level model and tile sheet.** A 64 x 64 board of tile ids, plus links, signs, chests, NPCs and baddies. Id to sheet: `col = (id % 16) + (id / 512) * 16`, `row = (id / 16) % 32` | board **done** (read, paint, write, layers 0 and 1); links, signs, chests and NPCs read with the client parser, moved and partly edited (**done** in the window); baddies kept as text |
| 2 | **Read and write GLEVNW01 text**, including `LINK`, `SIGN`, `CHEST`, `NPC`, `BADDY` and the 9 x 9 `HEIGHTS` block, with a round-trip test | **done** for the board and layers 0 and 1; other records round trip verbatim and are rewritten line by line when edited; `HEIGHTS` kept as it is |
| 3 | **Read the binary levels** (`GR-V1.00` to `1.03`, `Z3-V1.03/04`) and **write `GR-V1.03`**, picking the format by file extension (`.nw` text, anything else binary) | **later** (the client has a binary reader; the editor says "not yet") |
| 4 | **Board view**: scrolling, 16 px tiles, overlays for blocking, foreground and terrain tiles, a status bar | scrolling, 16 px tiles, zoom, grid, status bar with cell and id **done** in the window; overlays **later** |
| 5 | **Selection, copy, cut, paste, delete, floating paste** | **later** (`paint` is the single-tile fill) |
| 6 | **Undo** of board writes: per tab, 128 records, rectangle snapshots | **later** |
| 7 | **Tile palette**: one tile or a block, the background tile, draw mode, flood fill on the right button | one-tile palette, click and drag to paint, right-click pick **done** in the window; block selection, background tile, flood fill **later** |
| 8 | **Stamps**: load `GOBJSET01` sets, place, add the selection to a set, save the set | load and place (on the active layer, with a preview) **done** in the window; adding to and saving a set **later** |
| 9 | **Link tools**: list and dialog, *turn selection into a warp field* (`playerx` / `playery` defaults, the edge rules), open the destination in a new tab | list, place, drag, size, destination and arrival **done** in the window; warp field from a selection and opening the destination **later** |
| 10 | **Sign tools**: list, x/y, text in the sign alphabet, signs at (0, 0) | list, place, drag, first text line **done** in the window; full text **later** |
| 11 | **Chest tool**: the 25 items, sign index with -1 for none | place (as `greenrupee`, -1), drag, delete **done** in the window; item and sign index **later** |
| 12 | **Baddy tool**: ten types, three verses, types 4 to 6 without verses | shown as markers; editing **later** |
| 13 | **NPC tool**: image, script with test and find, clone, predefined NPCs (`defnpc*.txt`) | place, drag (keeps half tiles), first script line, 20-line script pane on double click, delete **done** in the window; image, full script, clone, predefined NPCs **later** |
| 14 | **Tabs, Save (save-all), Save as, Close with a prompt, New.** Write to a temporary file and rename, not delete-then-write | save through temp file and rename, Save and Save as, a Save / Discard / Cancel prompt on open and close **done** (one level, no tabs); the rest **later** (a new level can be made from `assets/default/levels/new1.nw` by `save --out`) |
| 15 | **Keys**: the key map of the editor, `F10` border toggle, optional `Ctrl+Q` batch re-save | **later** |
| 16 | **Maps**: several levels at once, the level cache, gmap generation | `.gmap` grid that opens one level per click **done** in the window; several levels at once, the cache and generation **later** |
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
