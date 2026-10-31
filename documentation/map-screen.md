# The M map

User report (2026-10-31): pressing M in the real client opens the map; in Seihai it only closed a sign.
Code: `crates/seihai-mae/src/mapscreen.rs`, wired in `crates/seihai-mae/src/bin/main.rs`; `setmap` in
`seihai-core` `scripting/engine/host.rs`.

## Which builds have it

Every 1.x and 2.x build. The 1.21 readme lists "Show Map or Cancel Reading: 'M'", 1.37 and earlier have it
on a fixed key, 1.38 to 2.31 make it the configurable action `map` (action 7). `enablefeatures` (2.1+
scripts) can turn the M key off. Seihai shows it for every `seihai-mae` build.

## What the real client does

Read from the 2.05 rev 2 executable (the Ghidra export in the project's `graal-decompile/v205-delphi/`):
the game key handler, the map draw routine, the scroll clamp, the map-click routine called from the game
window's mouse-down handler, the map loader and the training-mode defaults. Release notes from the 2.22
folder (`newfeatures141.txt`, `newfeatures2001.txt`, `newfeatures2002.txt`).

* **M** opens the map (overlay state 1, mode 1) or closes it when it is open. M while the item list is
  open swaps the list for the map.
* **Opening** centres the view on the player's spot on the map, then clamps it. A player whose level is not
  on the map gets the middle of the map.
* **The four movement keys** scroll the view 8 pixels a key press (key events, so OS key repeat applies).
* **The clamp**: the scroll stays in `0 ..= picture - view` on each axis; a picture smaller than the view is
  centred (1.38 notes: "the M map is displayed in the vertical middle of the screen when it is smaller").
* **Q** swaps the map for the item list. **P** closes it. A, S and D do nothing while it is open. Tab still
  reaches the chat field (handled before the overlay).
* **Drawing**: black, then the picture at view origin minus scroll. Then every player whose map cell is
  known gets a head, 16 pixels (12 under one flag not identified), centred on its spot. The names of the
  players picked in the player list are written under their heads; with none picked the own player is
  the list, so the own name shows (1.41 notes: "when you select a player in the player list then you see
  his nickname displayed on the M map"). "Map" is written at (20, 20) with a blue copy at (22, 22)
  under it; online, "Others: n" at (485, 25). Both texts come from the language file (`MapString1`,
  `MapString2`).
* **Ctrl+click** (2.05): the game window's mouse-down handler calls the map-click routine only while the
  map is open and Ctrl is held (Delphi's `ssCtrl` shift bit); a plain click does nothing. Offline it loads
  the clicked level with the player at the clicked spot (`pixel_in_cell * 64 / 56`, whole tiles); online it
  says `warpto x y level` as chat, a staff command the server ignores from a player without the right.
  Seihai does the same (`mapscreen::click_target`). The map stays open.
* **Animation** (2.01 and later): opening the map while standing still or sitting sets the `maps1` gani
  (which goes on to the looping `maps2` by its `SETBACKTO`), so others see the player reading; closing sets
  `maps3`, which goes back to `idle`. In 2.05 the open sets it only from idle, walk, walkslow or sit, and the
  close only from `maps1` or `maps2`. 2.17 removed it on Graal Kingdoms only. Seihai: `pose::MapPose`, for
  2.01 and later, drawn when `maps1.gani` (and `maps2`/`maps3`) is in the assets or downloaded; the built-in
  set has none, so offline without a Graal folder nothing changes. The GANI prop carries `maps1` and then
  `maps3` on the builds with a walk cadence sender (2.17, 2.22); 2.01 to 2.05 do not send pose ganis in
  Seihai yet. Walking, carrying, dying or being hurt ends it.
* **`enablefeatures bits`** (2.1+ vocabulary): bit 1 off makes M do nothing (an open map still closes).
  Seihai honours bit 1 (`OfflineSession::features`); the other bits (P, Q, R, S+A, S+D, Tab) are not
  applied yet.

### Picture and grid

* The grid is a `bigmap.txt`-style text: rows of quoted level names (`Gmap::parse_grid`).
* 2.05 draws 56 pixels a level: `map.png` (1120×1176) goes with the 20×21 `bigmap.txt`, the tutorial's
  `tutorialmap.png` (168×224) with its 3×4 grid, and the editor's "Generate map" (M in the editor) draws 56
  pixels a level. The 2.22 folder's `bigmap.png` (1024×371) goes with the 58×22 `bigmap2.txt`, about 17.6
  pixels a level, so 2.1+ must scale to the picture. Seihai takes the cell size as picture size over grid
  size, which gives 56 for every 2.05 pair.
* A player's spot: `cell * cell_size + position * cell_size / 64` (the inverse of the click-to-warp
  routine, which turns a clicked pixel into `cell` and `pixel_in_cell * 64 / 56`).

### Which map

| source | builds | Seihai |
|---|---|---|
| `bigmap.txt` + `map.png` from the client's `maps/` folder | all (2.05's built-in default) | used when nothing else names a map |
| the server-list entry's fields 8 to 11 | 1.x to 2.05 | not done |
| `setmap image,levelnames,x,y` | 1.x and later | done (`OfflineSession::script_map`) |
| `PLO_BIGMAP` (171) `txt,png,x,y` from the gserver's `bigmap=` option | 2.1 and later | done; the files are asked for with `WANTFILE` when missing |
| a gmap's `MAPIMG` | 2.1 and later (inferred) | done when the gmap names one |
| `--bigmap FILE` | Seihai | its grid with `map.png` |
| training mode: `tutorialmap.txt` + `tutorialmap.png` | 2.05 | not done (Seihai's tutorial is its own) |

The last two fields of `setmap` and `bigmap=` (`x,y`, e.g. `1.5,3.8` in the 2.05 tutorial, `12.2,8` in the
2.1 notes) are, by GServer-v2's `serveroptions.txt` notes, "the position where the heads of players not on
the map will be drawn". Seihai reads them in levels (they fit the tutorial's 3×4 grid only that way) and
draws an off-map player there. This is inferred; 2.05's own draw routine skips a player whose cell is
negative, and where the defaults land in the player object was not traced.

`maps/bigmap.txt` and other map texts are now found in the `maps/` folder of the assets and the download
cache, where the real clients keep and save them (`filecache::find_in_root`).

## Checked

* Unit tests: `mapscreen::tests` (cell size, spots, centring, the clamp, the 8-pixel scroll, drawing, file
  loading, the off-map point, the Ctrl+click target), `host::tests::setmap_names_the_m_maps_picture_and_grid`,
  `host::tests::enablefeatures_without_bit_1_turns_the_map_key_off`,
  `offline::tests::the_map_reading_pose_runs_maps1_maps2_then_maps3`,
  `filecache::tests::map_level_lists_are_found_in_the_maps_folder`.
* Offline, 2.05 under Xvfb: an assets folder with the 2.22 folder's `maps/map.png` and a 20×21 `bigmap.txt`
  with `new1.nw` at cell (5, 8). M opened the map centred on the player's head, Right scrolled it, M closed
  it.
* Loopback lab, 2.22 against the lab GServer-v2 (`seihai-lab-servers.tar.gz`) with `maps = labmap.txt` and
  `bigmap = labmap.txt,labmap.png,0.5,0.5` (a 2×1 grid and a 240×120 picture, 120 pixels a level): the
  server sent 171, Seihai read the grid, asked for `labmap.png` on the first M, drew it centred (smaller
  than the view), with the own head and name and a second logged-in player's head, and "Others: 1". Q
  swapped the map for the item list, M swapped back, P closed it without pausing.
* Not checked against a real client's screen: the exact head size and the text font and placement are
  from the decompile, not from a screenshot.
