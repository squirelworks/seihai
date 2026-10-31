# Default asset set (item 19)

Seihai ships a small, self-contained asset folder, `assets/default/`, so a fresh download can open a level, walk, swim and hear footsteps without a server and without any file from the original clients. This note says what is in it, how it is made, which layout contracts it meets, and how item 20 will wire it in.

Dated 2026-10-25. Base: `main` at `d5389c2`. Item 20 (wiring) is described under "How it is wired in" below. This branch also carries the commits of #49 (inventory) and #51 (reference sheets), because #51 was still open when it was written; if #51 has been merged the merge is empty for those files.

## Originality and licence

**Every image and sound is original.** It was drawn or synthesised by `tools/gen_default_assets.py`: images from rectangles, ellipses, polygons and hand-written palettes (Pillow `ImageDraw` on palette images); sounds from sine tones and filtered noise; the GANIs and levels are text written by the script. Nothing was read from, traced from, recoloured from, resampled from or otherwise derived from any pixel or audio sample of the playerworld archives or the Graal clients, and the images of those archives were not opened for this item. Two kinds of numbers are taken from them: those already published in `asset-reference.md` (sheet sizes, grid layouts, frame and sprite counts, sound formats), and the **tile type lists of the standard Graal tile layout** (arrays 0 to 6 of the 2.22 `arrays.dat`: which tile ids are passable, liquid, lava, thrown over, chairs and beds), kept as id ranges in `tools/graal_tile_lists.txt`. Those say which tile id is a wall or water, so that the built-in sheet and table use the same ids as a Graal `pics1.png`; they are not art.

**Licence:** the same as the Seihai repository (`MIT OR Apache-2.0`, see the workspace `Cargo.toml`), for the generated files and for the generator. Confirmed by the maintainer (2026-10-25): the generated files and the generator use the same licence as the Seihai repository (`MIT OR Apache-2.0`).

## Contents (105.7 KB in 28 files)

```
assets/default/
  pics1.png                  2048x512 tile sheet on the Graal tile layout, 8-bit palette PNG, opaque  24.1 KB
  tiledefs.txt               the 4096 tile types of the Graal layout, flat list     11.8 KB
  state.png                  224x145 HUD sheet, 4-bit palette PNG (2.05 layout)     1.6 KB
  sprites.png                128x402, shadow, ripples, carried objects, break pieces  1.5 KB
  bodies/body.png            128x720 body sheet, 4-bit palette PNG                  1.5 KB
  heads/head0.png            32x560 head sheet, 4-bit palette PNG                   0.6 KB
  swords/sword1.png          128x96 sword sheet, 4-bit palette PNG                  0.5 KB
  shields/shield1.png        38x20 shield sheet, 4-bit palette PNG                  0.2 KB
  */*.gif (4)                GIF copies of the four player sheets, for 2.05 builds  10 KB
  gani/idle.gani             1 step                                                  0.7 KB
  gani/walk.gani             8 steps, footsteps after steps 0 and 4                  2.9 KB
  gani/swim.gani             8 steps, head only, splash after step 0                 1.3 KB
  gani/sword.gani            5 steps, sword.wav after step 0, SETBACKTO idle         2.3 KB
  gani/sit.gani              1 step                                                  0.7 KB
  gani/hurt.gani             1 step                                                  0.9 KB
  gani/dead.gani             15 steps, fall and lie down, held on the last step      2.7 KB
  gani/carrystill.gani       1 step, arms raised (an object is held)                0.7 KB
  gani/carry.gani            8 steps, arms raised, footsteps after steps 0 and 4    2.9 KB
  sounds/steps.wav           22,050 Hz, 8-bit, mono, 150 ms                          3.3 KB
  sounds/steps2.wav          22,050 Hz, 8-bit, mono, 150 ms (different pitch)        3.3 KB
  sounds/water.wav           8,000 Hz, 16-bit, mono, 450 ms                          7.2 KB
  sounds/sword.wav           8,000 Hz, 8-bit, mono, 260 ms                           2.1 KB
  levels/new1.nw             starter level, 64x64, GLEVNW01 text                     9.3 KB
  levels/new2.nw             second tiny level (a fenced yard), GLEVNW01 text        9.2 KB
  levels/tiledefs/tiledefsOffline.txt   one line, `pics1.png,,0,0,0`
```

The folder names are the ones `AssetResolver` already searches (`bodies/`, `heads/`, `gani/`, `sounds/`, `levels/`, `levels/tiledefs/`, root for `pics1.png`, `sprites.png`, `state.png`, `tiledefs.txt`), so nothing in the resolver has to change to find them. `seihai --assets assets/default --check-assets` reports every required file for 2.05, 2.17, 2.22 and 2.31 as found and read; the optional ones it still lists as missing are `offlinetutorial.gmap` (2.05), music (`intro.mid`; the set has no music) and most of the 2.05 sound-effect list.

### Tile sheet and ids: the standard Graal layout

One `pics1.png` and one `tiledefs.txt` serve both this set and a Graal folder. Before, the built-in sheet had its own tile ids (water at 32, a bush at 16, ...), so its `tiledefs.txt` described only that sheet: a Graal level drawn on it showed the purple checker, and the table could not be used with a Graal `pics1.png`. Now both follow the Graal layout:

- **`tiledefs.txt`** holds all 4096 types as a flat list, 16 per line (line `n` is ids `16n` to `16n + 15`, `#` lines are comments; `TileTypes::parse` reads it). The types are those of the Graal 2.22 `arrays.dat`, by the rules of `TileTypes::parse_arrays` (`tile-types.md`): 0 open, 3 chair, 4 bed top, 5 bed bottom, 11 water, 12 lava, 13 lava floor, 20 a wall thrown objects fly over, 22 a wall. Loading it gives exactly the table that loading the real 2.22 `arrays.dat` gives (checked for all 4096 ids). The 2.05 rev 2 file differs only in 20 ids (2650 to 2717, passable in 2.22), so the table fits 2.05 too except there.
- **`pics1.png`** is 2048x512 with Seihai's id mapping (`column = id % 16 + (id / 512) * 16`, `row = (id / 16) % 32`), which is the real sheets' layout. Every id is drawn with the art of its type: grass for open floor, a grey stone block for walls, a lower tan block for type 20, water, lava, a cracked crust for lava floor, a chair, a bed. So any Graal level shows where its walls, water and floors are, though not what the real tiles depict. The ids below are documented and get art of their own.

| ids (2x2 objects: `id, id+1, id+16, id+17`) | what | type | source |
|--:|--|--:|--|
| 0 | grass | 0 | `binary-level-tiles.md` (tile 0 is grass on the 1.x and 2.x sheets) |
| 2047 | grass (left under chests) | 0 | lab note `lab-304-linux-2026-10-30.md` (`labbush.nw`), `chest-drawing.md` |
| 2 | bush | 20 | `arrays.dat` arrays 9 and 11 |
| 34 | stone | 20 | array 11 |
| 512 | sign | 20 | array 11, `board-respawn.md` |
| 684 | pot | 20 | array 11, `board-respawn.md` |
| 990 | black stone | 20 | array 11 |
| 420 | tall grass | 0 | array 9 |
| 677, 679 | left by a cut bush, cut tall grass | 0 | arrays 10 and 12 |
| 1802, 1770, 1834 | left by a lifted sign, pot, stone | 0 | array 12 |
| 1784 | closed chest | 20 | array 13, `chest-drawing.md` |
| 829 | open chest | 20 | array 14 |
| 322 | deep water (swum in) | 11 | Graal_analysis `protocol/v5v6-lab-play.md` |
| 398 | shallow water (walked on) | 0 | `tile-types.md` (`new4.nw`) |
| 3644 | deep water in no list (a wall) | 22 | `tile-types.md` (`new4.nw`) |

No other id was given a meaning, and no type was made up: every type comes from the lists. Ids whose art is not known (trees, houses, roofs, paths, the many floor variants) look like their type.

The foot point (links and water; walls are tested on the body box, `walking-collision.md`): `FOOT_DX = 2.0`, `FOOT_DY = 2.75`, one point per player, tile under `floor(x + 2.0), floor(y + 2.75)`.

To refresh the lists from an `arrays.dat`: `python3 tools/gen_default_assets.py --import-arrays path/to/arrays.dat`, then regenerate. The generator checks the list lengths of the 2.22 file (934, 116, 133, 76, 40, 4, 4).

### Player sheets

- **`body.png` 128x720**: four 32 px direction columns (`x = 0, 32, 64, 96` = up, left, down, right) and 22 rows of 32 px (22 x 32 = 704, plus 16 spare). Row 0 is the idle pose (the cell `draw_player_sprites` reads at `(dir * 32, 0)`); rows 1 to 8 are the eight walk poses (foot-falls at poses 1 and 5); rows 9 to 12 are the item 22 poses (9 sword wind-up, 10 sword swing, 11 sit, 12 hurt, our own numbering). Row 13 is the carry standing pose and rows 14 to 21 the eight carry-walk poses (both arms raised, our own art; `carrystill.gani` and `carry.gani` place them). The 16 spare rows at `y = 704` hold the lying body of `dead.gani`, a 32x16 cell per direction (all four the same). Body sprite id = `200 + 4 * pose + dir`. The "right" column is the "left" column mirrored.
- **`head0.png` 32x560**: one 32 px column, rows `y = 32 * dir` for the four directions (read at `(0, dir * 32)` by `draw_player_sprites`; head sprite id `100 + dir`). Rows from y = 128 down are empty; the extra head poses of other sets (hurt, sword) are not drawn.
- **`sword1.png` 128x96**: four direction columns by three rows of 32x32 (held, mid-swing with an arc, thrust). `sword.gani` places these (symbolic sheet `SWORD`, sprite ids `300 + 4 * row + dir`).
- **`shield1.png` 38x20**: four sprites at `x = 0, 6, 14, 30` of 6x18, 8x20, 16x20 and 8x20 (up, left, down, right). The core GANIs draw no shield (a player without one draws none); the sheet is there for the shield layer.
- **`sprites.png` 128x402**: the shadow is 24x12 at `(0, 0)`; the two swim rings are 32x16 at `(0, 306)` and `(0, 322)`; the five carried objects (bush, pot-like block, two stones, a boulder) are 32x32 cut-outs with transparent corners at `(0, 338)`, `(32, 338)`, `(64, 338)`, `(96, 338)`, `(96, 370)`; the break pieces are leaf bits at `(44, 0)`/`(60, 0)`, rock chunks 16x16 at `(24, 82)` .. `(40, 98)`, pot chunks 16x14 at `(56, 82)` .. `(72, 96)`, and two 30x30 splash frames at `(24, 114)`/`(54, 114)`. All of it is drawn by `build_sprites` from polygons and ellipses; the rectangles are where `documentation/throwing.md`'s sprite table points. Everything else is transparent. The renderer skips only fully transparent pixels (`blit_sprite`), so the shadow is a checker-dithered ellipse rather than a soft alpha, and the rings carry an opaque water-coloured ellipse that hides the lower half of the head.
- **`state.png` 224x145**: our own art at the rectangles the 2.05 status routine reads (`hud.md`, `render/hud.rs`), so the built-in sheet fills the same HUD layout as a real client's. Big hearts 40x44, full at `(0, 0)` and half at `(40, 0)`; small hearts 31x33, full at `(80, 0)` and half at `(111, 0)` (a half heart is a whole heart with its right side empty; empty containers are not drawn, so there is no empty heart). The icon strip `(80, 33, 120, 16)` holds a rupee, a bomb and an arrow centred at strip x 6, 65 and 113, so each sits over its count (x 280, 338 and 386 on screen). The button the key names go on is 22x33 at `(202, 0)`; `(169, 15)` is left transparent so `three_buttons` does not take the sheet for 1.38 to 1.41 art. Ten digits in 14x16 cells at `y = 81` (the 5x7 glyphs doubled, with a one-pixel dark rim). The MP frame is 130x22 at `(0, 49)` with its fill row (100x10) at `y = 71`; the AP frame is at `(0, 97)` with its fill at `y = 119`. A frame has an emblem in its left 22 px and a dark well the fill covers from `+22, +6`. Before this the sheet was a 16 px placeholder in a layout of its own, which the HUD read with the 2.05 rectangles: cropped hearts and a stray digit row in the top right.

Palettes: sword, shield, sprites and state are 4-bit palette PNGs with index 0 transparent (`tRNS`). The body is a 4-bit palette PNG in the 2.05 body layout, so it can be drawn in the player's colours (`appearance.md`): 0 outline, 1 belt, 2 transparent, 3 shoes, 4 sleeves, 5 skin, 6 coat, the rest after. The light tunic, dark skin and dark boot shades are folded into their parts; the head is also a 16-colour palette PNG (written 4-bit; the 8-bit form the 2.22 sheets use reads the same); `pics1.png` is an 8-bit palette PNG with no transparency. All are non-interlaced and read by `read_png_rgba` / `read_png_rgb`.

### GANIs

Our own text files, in the format `Gani::parse` reads: `SPRITE id SHEET x y w h description` lines (`SHEET` is the symbolic `BODY`, `HEAD` or `SPRITES`), `DEFAULTHEAD` / `DEFAULTBODY`, `LOOP`, `CONTINUOUS`, then `ANI` ... `ANIEND` with four direction rows per step (up, left, down, right), rows of `sprite x y` placements separated by commas, a blank line between steps, and `PLAYSOUND name x y` after the step's rows.

| file | steps | placements per row | cues |
|--|--:|--|--|
| `idle.gani` | 1 | shadow (id 0), body (200 + dir), head (100 + dir) | none |
| `walk.gani` | 8 | the same, body pose 1 to 8; body and head bob 0 to 2 px | `steps2.wav 1.5 2` after steps 0 and 4 |
| `swim.gani` | 8 | head (100 + dir), ring (id 20 or 21, alternating); no body | `water.wav 1.5 2` after step 0; `WAIT 1` after every step |
| `sword.gani` | 5 | shadow, body (pose 9 or 10), head, sword cell (`SWORD` ids 300..311) | `sword.wav 1.5 2` after step 0; `SETBACKTO idle` |
| `sit.gani` | 1 | shadow, body (pose 11), head 6 px lower | none |
| `hurt.gani` | 1 | shadow, body (pose 12, leaning away), head shifted away | none |
| `dead.gani` | 15 | 6 steps as `hurt.gani`, 8 kneeling (sit pose, head 10 px lower), then lying: the 32x16 lying body (ids 288..291) with the down-facing head at its left end, the same for every direction | none; held on the last step |

Offsets are the idle offsets already in `draw_player_sprites` (`OFFSETS`: shadow, body, head per direction inside the 48x48 player box). The swim head is lowered by 14 px, the number `draw_player_sprites` uses; the 2.22 animation uses 16 or 20 (asset-reference, "not settled"), and this set follows the code so that the GANI and the renderer agree until that is settled.

### Sounds

Mono PCM WAV files in the stock formats: footsteps at 22,050 Hz 8-bit (`steps.wav` for builds without GANIs, `steps2.wav` for the walk cues), the water sound at 8,000 Hz 16-bit. Synthesised from sine tones with a falling pitch and low-pass filtered noise (footsteps: a thud with a decaying envelope; splash: band-passed noise with four short rising bubble chirps). `sword.wav` (item 22) is a 260 ms whoosh at 8,000 Hz 8-bit, the stock sword format: noise through a low-pass whose cut-off sweeps up, under a rising sine chirp. `decode_wav` reads all four. No bomb or item sound is provided (all are optional in the checks).

### Levels

Both use only the documented ids of the table above, so they read the same on this sheet and on a Graal `pics1.png`.

- **`levels/new1.nw`** (the name `--check-assets` and the notes use for the sample level): 64x64 grass with a border of 2x2 bushes on the north, west and east edges, a pond of deep water (322, swum in) inside a ring of shallow water (398), bushes, stones, pots and tall grass, a sign (2x2 at (26, 27); its record is the lower row, (26, 28)) with a welcome text and a chest record at (34, 26) holding `greenrupee` on 2047 grass, the way servers leave the board under a chest. A bare patch (1834) in a ring of black stones is the way to the second level. The default spawn point (30, 30.5) is open grass. Links: `LINK new2.nw 41 20 2 2 31 47` (the patch) and `LINK new1.nw 2 63 60 1 playerx 3` (the open south edge wraps to the north: a link to itself).
- **`levels/new2.nw`**: a yard fenced with black stones and surrounded by stones, a chest at (24, 40) holding `bluerupee`, a sign at (36, 39) (record (36, 40)), a pot and tall grass, and a gap in the south fence with `LINK new1.nw 30 55 4 1 40 23` back outside.

Link arrivals land the foot point on open floor, away from the link rectangles, so a link does not fire again at once.

## Version-profile gating

Only what the existing resolver already does is used; nothing was added to it.

- **Builds without GANIs** (`ui.ganis` false in the profile): `AssetResolver::gani` finds none, so the three GANIs are ignored and the footstep and water sounds are read by name (`steps.wav`, `water.wav` ship for that reason).
- **Level extensions:** the starter levels are `.nw`; for a profile with `levels.nw = false` (the early `.graal`-only builds) they are not opened. A `.graal` or `.zelda` starter level is not generated.
- **Image extension:** a bare image name gets the profile's `image_ext` first and then `.png`; pre-2.1 profiles ask for `.gif`, and the resolver already falls back from `.gif` to `.png`. The set is PNG only (there is no GIF or MNG decoder).
- **Sheet width:** the sheet is the 2048-wide layout of 1.41 and later. Whether a real 1.33 to 1.39 client maps ids the same way on its 1856-wide sheet is not settled (asset-reference, "Other widths"), and the 1.x `arrays.dat` files have shorter lists than 2.22, so the table is the 2.x one.
- **HUD:** `state.png` is 224x145; a profile that wants 224x97 (1.38+ layouts, see `documentation/assets.md`) is not given a separate file.

## Regenerating

```
python3 tools/gen_default_assets.py                 # writes assets/default/
python3 tools/gen_default_assets.py --check         # exit 1 if the tree differs from the generator
python3 tools/gen_default_assets.py --preview /tmp/preview.png   # contact sheet of this art
```

Python 3.9 or later and Pillow; no network. The seed is `SEED = 20261025` in the script and every random choice (grass tufts, noise, scatter on the levels) comes from `random.Random(SEED + n)`, so two runs give identical files on the same machine; `--check` compares decoded pixels for the PNGs (not bytes, because zlib output can differ between builds) and bytes for everything else. The tree is small (105.7 KB), so it is committed; the generator is the source of truth, and a change to the art is a change to the script plus a regenerated tree.

The contract test is `client/tests/default_assets.rs` (`cargo test --test default_assets`): it loads each file with Seihai's own readers and checks sizes, bit depths, the 128x32 grid, that `blit_tile` draws each id's cell, that every id has the art of its type and the documented ids their own, the type table (the `new4.nw` tiles of `tile-types.md`, the objects of `arrays.dat`, 116 liquid ids), the GANI structure (1, 8 and 8 steps, four rows each, the cues, sprites inside their sheets), WAV formats and lengths, both levels (only documented ids, links lead to existing levels and arrive on open floor, chests and signs sit on their art), the offline session (default spawn open, pond swims, a bush blocks) and `AssetResolver` / `check_folder` for 2.05, 2.17, 2.22 and 2.31.

## How it is wired in (item 20)

The set is compiled into the binary and is the last layer of every lookup. The full rules are in `documentation/file-cache.md` (cache layout, naming, search order, what a download does, how `WANTFILE` / `UPDATEFILE` treat built-in files); this is the short version.

1. **Embedded, then written once.** `client/src/assets/builtin.rs` holds every file of `assets/default/` with `include_bytes!` (105.7 KB), and a test fails if that list and the folder differ. The client finds files by path (resolver, sound engine, readers), so at start-up the set is written to `<settings dir>/builtin/<id>/` (`<id>` = CRC-32 of the whole set) and that folder is the lowest layer. The files are written 0444 and a missing or changed one is rewritten from the binary, so the folder is a copy of what is in the executable and nothing else; a new build with new art uses a new `<id>` and leaves the old folder alone. If the settings folder cannot be written the set goes to the system temp folder; if that fails, the client runs without the layer and says so. No network, no shipped data folder next to the exe.
2. **Fallback.** `--assets` is optional. With no folder, a missing folder or an empty one, every lookup falls through to the built-in set; a folder with only some files uses its own where it has them and ours for the rest (file by file). An offline run with no `--assets` and no `--level` starts in `new1.nw` (the launcher's "Play offline" does the same); with `--assets`, behaviour is unchanged. `seihai --check-assets` with no `--assets` checks the built-in set (exit 0 for 1.39, 2.05, 2.17, 2.22 and 2.31, no required file missing, nothing unreadable); with `--assets DIR` it still checks exactly that folder.
3. **Per-server cache, searched first.** Search order is: the server's cache folder, the user's asset folder, the built-in set. Downloads are written to the cache only; the built-in set is read-only and never a download target (a target under it is refused). `save_downloads` / `--no-save-downloads` still gate writing. Layout and naming: `<cache root>/<host>_<port>[~<server name>]/` with the real clients' sub-folders; the root is `--cache-dir`, then `cache_dir=` in the settings file, then `<settings dir>/cache`.
4. **Built-in files and the server.** A file only the built-in set has is asked for with `WANTFILE`, with no mod time (never `UPDATEFILE`), so the server's copy can be saved in the cache and win; if the server has none, ours stays. `walk.gani`, `steps.wav` and `steps2.wav` (the real client's first-step requests) are asked for only if there is no copy anywhere, built-in included, which with this set means never in a normal run.
5. **Built-in levels are offline only.** Online, the resolver does not find `new1.nw` / `new2.nw` in the built-in set, so a server's level of the same name is requested from the server.
6. **Drawing.** `render_view` no longer draws the brown chest box and the tan sign board over the board. A sign is a trigger; its art is the board tiles under it. A chest record is drawn from the record with tiles 1784 (closed) or 829 (open), as the client does (see `chest-drawing.md`); the built-in sheet has its chest art at those ids, so there is no special case for it. NPC placeholders are unchanged.
7. **One tile table.** The built-in `tiledefs.txt` is the Graal layout's table, so it is used whenever no layer above has a `tiledefs.txt` or `arrays.dat`, also with a user's own `pics1.png`. A user folder with its own `arrays.dat` still wins.

What was checked, offline, on Xvfb with this set only (before the move to the Graal layout) (screenshots of our own art, not committed; `/workspace/graal/seihai_notes/default-assets-offline-*.png` on the author's box): the starter level loads, the player walks (several directions, pose changes), chest and sign show their tiles, an opened chest shows the open art, the pond swims (head only, ring), and a run with an empty `--assets` folder looks the same. The tests are `client/tests/builtin_wiring.rs` (14) and the unit tests of `builtin.rs`, `servercache.rs`, `settings.rs`, the resolver and the file cache.

The generator `tools/gen_default_assets.py` already re-emits the whole set from a seed (`--check`, `--preview`), so the optional item 21 tool is mostly done by it; a parameterised front end (tile size, palette) is not part of this PR.

## Poses: sword, sit, hurt (item 22)

`sword.gani`, `sit.gani`, `hurt.gani` and `sounds/sword.wav` are generated by the same script (body rows 9 to 12 and the sword cells come from `body_pose_cell` and `build_sword`; the GANI text from `sword_gani`, `sit_gani` and `hurt_gani`; `--preview` draws every step). Like the rest of the set they are drawn from rectangles, ellipses and rotated copies of our own cells, and the sword sound is synthesised; no corpus image or audio was opened for them. What comes from `asset-reference.md` is only structure: the sword animation has five steps with a sound cue after step 0 and `SETBACKTO idle`, the sit animation has one step, the sword sheet is 128x96.

**How the client uses them** (`client/src/pose.rs`, `render::draw_gani_frame`):

- **S key.** The swing already ran in `combat.rs` (sprite value S = 6 to 10, one step per two 50 ms ticks). Step `i` of the swing now draws step `i` of `sword.gani` over the player's idle figure, and entering a step queues the `PLAYSOUND` cues that follow it, so `sword.wav` plays once when the swing starts. When the swing ends the pose is gone and the idle figure is drawn again; that is what `SETBACKTO idle` says, so the parser keeps the name (`Gani::setback_to`) and the test checks it. The swing is refused in water and while dead (unchanged).
- **A hit.** A hit that the judge accepts starts hurt mode (`take_hit`, unchanged: 40 ticks in which nothing can hurt the player, sprite S = 32 sent). `hurt.gani` is drawn by its waits (a step lasts `1 + WAIT` ticks of 50 ms): a looped GANI repeats for the whole of hurt mode, any other plays once (see `health-death.md`). The default file holds its one step for 31 ticks (`WAIT 30`), the length of the stock file and of the hurt pose read live in 2.05 rev 2. This works in the offline session too, because the pose follows the combat state, not the network.
- **Sit.** `sit.gani` is drawn while `OfflineSession::sitting` is set. Nothing sets it yet, since no key is bound.
- Priority: swing, then hurt, then sit. No pose while swimming or dead, and none when its GANI is not loaded (a build without GANIs, or a data folder that lacks the file), so the old idle drawing is the fallback.

**Seihai's own conventions, because the real clients' behaviour is not documented here:** the sprite others see lasts the whole of hurt mode, and is not changed; the body rows 9 to 12, the sword sprite ids 300 to 311, and the hand positions the sword cells are placed at; sit being a flag with no key; one sword sheet (`sword1.png`) whatever the sword power. The walk and swim animations still only supply sounds, as before: the local player is not drawn with the walk poses (that is outside this item).

No file request is added for these (the first-step list is unchanged); a copy of one of them that arrives from a server is loaded in its place.

## Not settled

- The swim head offset (14 px in code and here; 16 and 20 in the 2.22 animation).
- Whether a 1.33 to 1.39 client reads ids on a 1856-wide sheet the same way.
- The art is functional and small, not final: no hats, and one look per tile type outside the documented ids. Naming more ids (trees, houses, paths) needs a documented source for each; then the generator can draw them.
- The built-in set has no cut and lift tables (`arrays.dat` arrays 9 to 12), so offline with the built-in set alone bushes are not cut or lifted. Shipping those tables is a separate change.
