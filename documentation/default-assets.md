# Default asset set (item 19)

Seihai ships a small, self-contained asset folder, `assets/default/`, so a fresh download can open a level, walk, swim and hear footsteps without a server and without any file from the original clients. This note says what is in it, how it is made, which layout contracts it meets, and how item 20 will wire it in.

Dated 2026-10-25. Base: `main` at `d5389c2`. Item 20 (wiring) is described under "How it is wired in" below. This branch also carries the commits of #49 (inventory) and #51 (reference sheets), because #51 was still open when it was written; if #51 has been merged the merge is empty for those files.

## Originality and licence

**Every file is original.** It was drawn or synthesised by `tools/gen_default_assets.py`: images from rectangles, ellipses, polygons and hand-written palettes (Pillow `ImageDraw` on palette images); sounds from sine tones and filtered noise; the GANIs and levels are text written by the script. Nothing was read from, traced from, recoloured from, resampled from or otherwise derived from any pixel or audio sample of the playerworld archives or the Graal clients, and the images of those archives were not opened for this item. The only things taken from them are the numbers already published in `asset-reference.md` (sheet sizes, grid layouts, frame and sprite counts, sound formats), which describe a file format, not content.

**Licence:** the same as the Seihai repository (`MIT OR Apache-2.0`, see the workspace `Cargo.toml`), for the generated files and for the generator. Confirmed by the maintainer (2026-10-25): the generated files and the generator use the same licence as the Seihai repository (`MIT OR Apache-2.0`).

## Contents (54.7 KB in 21 files)

```
assets/default/
  pics1.png                  2048x512 tile sheet, 8-bit palette PNG, opaque        5.1 KB
  tiledefs.txt               tile types, `index,x,y,type,name` rows                 2.2 KB
  state.png                  224x145 HUD sheet, 4-bit palette PNG (our own layout)  0.6 KB
  sprites.png                32x338, shadow and ripples, 4-bit palette PNG          0.4 KB
  bodies/body.png            128x720 body sheet, 4-bit palette PNG                  1.5 KB
  heads/head0.png            32x560 head sheet, 4-bit palette PNG                   0.6 KB
  swords/sword1.png          128x96 sword sheet, 4-bit palette PNG                  0.5 KB
  shields/shield1.png        38x20 shield sheet, 4-bit palette PNG                  0.2 KB
  gani/idle.gani             1 step                                                  0.7 KB
  gani/walk.gani             8 steps, footsteps after steps 0 and 4                  2.9 KB
  gani/swim.gani             8 steps, head only, splash after step 0                 1.3 KB
  gani/sword.gani            5 steps, sword.wav after step 0, SETBACKTO idle         2.3 KB
  gani/sit.gani              1 step                                                  0.7 KB
  gani/hurt.gani             1 step                                                  0.9 KB
  sounds/steps.wav           22,050 Hz, 8-bit, mono, 150 ms                          3.3 KB
  sounds/steps2.wav          22,050 Hz, 8-bit, mono, 150 ms (different pitch)        3.3 KB
  sounds/water.wav           8,000 Hz, 16-bit, mono, 450 ms                          7.2 KB
  sounds/sword.wav           8,000 Hz, 8-bit, mono, 260 ms                           2.1 KB
  levels/new1.nw             starter level, 64x64, GLEVNW01 text                     9.6 KB
  levels/new2.nw             second tiny level (a room), GLEVNW01 text               9.4 KB
  levels/tiledefs/tiledefsOffline.txt   one line, `pics1.png,,0,0,0`
```

The folder names are the ones `AssetResolver` already searches (`bodies/`, `heads/`, `gani/`, `sounds/`, `levels/`, `levels/tiledefs/`, root for `pics1.png`, `sprites.png`, `state.png`, `tiledefs.txt`), so nothing in the resolver has to change to find them. `seihai --assets assets/default --check-assets` reports every required file for 2.05, 2.17, 2.22 and 2.31 as found and read; the optional ones it still lists as missing are `offlinetutorial.gmap` (2.05), music (`intro.mid`; the set has no music) and most of the 2.05 sound-effect list.

### Tile sheet and ids

`pics1.png` is exactly 2048x512: 128 columns by 32 rows of 16 px cells, so every id from 0 to 4095 has a cell with Seihai's mapping `column = id % 16 + (id / 512) * 16`, `row = (id / 16) % 32`. Only cells of ids that the set uses are drawn; all other cells are an 8 px purple/black checker, so a wrong id is visible at once instead of looking like grass.

All used ids are below 512, i.e. in block 0 (columns 0 to 15). That makes them valid on every sheet at least 256 px wide, including a narrower 1.33 to 1.39 sheet.

| id | name | type | art |
|--:|--|--:|--|
| 0, 1, 2 | grass, grass2, flowers | 0 | three grass variants (0 is also the id an empty board cell means) |
| 3 | path | 0 | dirt |
| 16 | bush | 22 | bush on grass |
| 17 | rock | 22 | boulder on grass |
| 32, 33 | water, water2 | **11** | two water frames |
| 34, 35 | sand, sand2 | 0 | pond edge |
| 48, 49 | wall, wallcap | 22 | brick face and capped brick |
| 50 | roof | 22 | red roof tiles |
| 51 | doorway | 0 | walkable opening; the door links sit on it |
| 52 | woodfloor | 0 | planks, for the room |
| 54 | void | 22 | near-black, outside the room |
| 64..65, 80..81 | chest0..3 | 22 | closed chest on grass, 2x2 (ids `id, id+1, id+16, id+17`) |
| 66..67, 82..83 | chestopen0..3 | 22 | open chest on grass |
| 68..69, 84..85 | chestw0..3 | 22 | closed chest on a wood floor |
| 70..71, 86..87 | chestwopen0..3 | 22 | open chest on a wood floor |
| 96, 97 | sign0, sign1 | 22 | sign, 2x1, on grass |
| 98, 99 | signw0, signw1 | 22 | sign on a wood floor |
| 112..113, 128..129 | tree0..3 | 22 | tree, 2x2 |

Types follow the rules in `assets/level.rs`: `>= 20` blocks (`TYPE_WALL_MIN`), `11` is water (`TYPE_WATER`), everything else is open floor. Wall, water and object art fills its whole 16x16 cell (whole tiles decide blocking). The chest and sign records of a level (`CHEST x y ...`, `SIGN x y`) are triggers; the art is the board tiles at that position, which the starter levels place under them. Chests are solid as a 2x2 box and the sign as 2x1, as in `offline/mod.rs`; their tiles are type 22 too.

`tiledefs.txt` is a sparse `index,x,y,type,name` list (`x,y` is the pixel position of the id's cell, derived from the mapping above). `TileTypes::parse` needs the table to reach id 4095, so the last row names 4095 explicitly (type 0). Ids that no row names are type 0.

The foot point (links and water; walls are tested on the body box, `walking-collision.md`) the sheet was drawn for: `FOOT_DX = 2.0`, `FOOT_DY = 2.75`, one point per player, tile under `floor(x + 2.0), floor(y + 2.75)`.

### Player sheets

- **`body.png` 128x720**: four 32 px direction columns (`x = 0, 32, 64, 96` = up, left, down, right) and 22 rows of 32 px (22 x 32 = 704, plus 16 spare). Row 0 is the idle pose (the cell `draw_player_sprites` reads at `(dir * 32, 0)`); rows 1 to 8 are the eight walk poses (foot-falls at poses 1 and 5); rows 9 to 12 are the item 22 poses (9 sword wind-up, 10 sword swing, 11 sit, 12 hurt, our own numbering). Rows 13 to 21 are empty and reserved. Body sprite id = `200 + 4 * pose + dir`. The "right" column is the "left" column mirrored.
- **`head0.png` 32x560**: one 32 px column, rows `y = 32 * dir` for the four directions (read at `(0, dir * 32)` by `draw_player_sprites`; head sprite id `100 + dir`). Rows from y = 128 down are empty; the extra head poses of other sets (hurt, sword) are not drawn.
- **`sword1.png` 128x96**: four direction columns by three rows of 32x32 (held, mid-swing with an arc, thrust). `sword.gani` places these (symbolic sheet `SWORD`, sprite ids `300 + 4 * row + dir`).
- **`shield1.png` 38x20**: four sprites at `x = 0, 6, 14, 30` of 6x18, 8x20, 16x20 and 8x20 (up, left, down, right). The core GANIs draw no shield (a player without one draws none); the sheet is there for the shield layer.
- **`sprites.png` 32x338**: the shadow is 24x12 at `(0, 0)`; the two swim rings are 32x16 at `(0, 306)` and `(0, 322)`; everything else is transparent. The renderer skips only fully transparent pixels (`blit_sprite`), so the shadow is a checker-dithered ellipse rather than a soft alpha, and the rings carry an opaque water-coloured ellipse that hides the lower half of the head.
- **`state.png` 224x145**: the HUD sheet is the one file whose real layout is not documented, and nothing in Seihai reads it yet (`--check-assets` only needs it to load). Ours holds a full, half and empty heart (16x16 at `x = 0, 16, 32`), a rupee, a bomb and an arrow (about 16x16 at `x = 50`, `66`, `82`) and ten 5x7 digits in 8x12 cells at `y = 16`. It is a placeholder with our own layout, not the real `state.png` layout; the 1.38+ layouts use 224x97, and a smaller sheet works for them too (the 145 row count is a superset).

Palettes: body, sword, shield, sprites and state are 4-bit palette PNGs with index 0 transparent (`tRNS`); the head is also a 16-colour palette PNG (written 4-bit; the 8-bit form the 2.22 sheets use reads the same); `pics1.png` is an 8-bit palette PNG with no transparency. All are non-interlaced and read by `read_png_rgba` / `read_png_rgb`.

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

Offsets are the idle offsets already in `draw_player_sprites` (`OFFSETS`: shadow, body, head per direction inside the 48x48 player box). The swim head is lowered by 14 px, the number `draw_player_sprites` uses; the 2.22 animation uses 16 or 20 (asset-reference, "not settled"), and this set follows the code so that the GANI and the renderer agree until that is settled.

### Sounds

Mono PCM WAV files in the stock formats: footsteps at 22,050 Hz 8-bit (`steps.wav` for builds without GANIs, `steps2.wav` for the walk cues), the water sound at 8,000 Hz 16-bit. Synthesised from sine tones with a falling pitch and low-pass filtered noise (footsteps: a thud with a decaying envelope; splash: band-passed noise with four short rising bubble chirps). `sword.wav` (item 22) is a 260 ms whoosh at 8,000 Hz 8-bit, the stock sword format: noise through a low-pass whose cut-off sweeps up, under a rising sine chirp. `decode_wav` reads all four. No bomb or item sound is provided (all are optional in the checks).

### Levels

- **`levels/new1.nw`** (the name `--check-assets` and the notes use for the sample level): 64x64, grass with a bush border on the north, west and east edges, a pond (water ids 32/33, sand ring) that is deep enough to swim in, a brick house with a roof and a doorway, a dirt path from the door to the default spawn point (30, 30.5, which is open grass), four trees, bushes and rocks, a sign at (26, 28) with a welcome text and a chest at (34, 26) holding `greenrupee`. Links: `LINK new2.nw 41 21 2 1 30 47` (the doorway, to the room) and `LINK new1.nw 2 63 60 1 playerx 3` (the open south edge wraps to the north: a link to itself).
- **`levels/new2.nw`**: a 20x15 wood-floor room in a void of type-22 tiles, a chest at (24, 40) holding `bluerupee`, a sign at (36, 40), and a doorway at (31..32, 53) with `LINK new1.nw 31 53 2 1 40 22` back outside.

Link arrivals land the foot point on open floor, away from the link rectangles, so a link does not fire again at once.

## Version-profile gating

Only what the existing resolver already does is used; nothing was added to it.

- **Builds without GANIs** (`ui.ganis` false in the profile): `AssetResolver::gani` finds none, so the three GANIs are ignored and the footstep and water sounds are read by name (`steps.wav`, `water.wav` ship for that reason).
- **Level extensions:** the starter levels are `.nw`; for a profile with `levels.nw = false` (the early `.graal`-only builds) they are not opened. A `.graal` or `.zelda` starter level is not generated.
- **Image extension:** a bare image name gets the profile's `image_ext` first and then `.png`; pre-2.1 profiles ask for `.gif`, and the resolver already falls back from `.gif` to `.png`. The set is PNG only (there is no GIF or MNG decoder).
- **Sheet width:** every id of the set is below 512 and so lies in block 0, which is at the same position for any sheet that has the first 16 columns. Whether a real 1.33 to 1.39 client maps ids the same way on its 1856-wide sheet is not settled (asset-reference, "Other widths"). A 1856 variant could be made by cropping `pics1.png`; nothing here needs it.
- **HUD:** `state.png` is 224x145; a profile that wants 224x97 (1.38+ layouts, see `documentation/assets.md`) is not given a separate file.

## Regenerating

```
python3 tools/gen_default_assets.py                 # writes assets/default/
python3 tools/gen_default_assets.py --check         # exit 1 if the tree differs from the generator
python3 tools/gen_default_assets.py --preview /tmp/preview.png   # contact sheet of this art
```

Python 3.9 or later and Pillow; no network. The seed is `SEED = 20261025` in the script and every random choice (grass tufts, noise, scatter on the levels) comes from `random.Random(SEED + n)`, so two runs give identical files on the same machine; `--check` compares decoded pixels for the PNGs (not bytes, because zlib output can differ between builds) and bytes for everything else. The tree is small (54.7 KB), so it is committed; the generator is the source of truth, and a change to the art is a change to the script plus a regenerated tree.

The contract test is `client/tests/default_assets.rs` (`cargo test --test default_assets`): it loads each file with Seihai's own readers and checks sizes, bit depths, the 128x32 grid, that every named tile id lands on its own cell and `blit_tile` draws that cell, the type table (water 11, walls 20 or more, ids below 4096), the GANI structure (1, 8 and 8 steps, four rows each, the cues, sprites inside their sheets), WAV formats and lengths, both levels (every id named, links lead to existing levels and arrive on open floor, chests and signs sit on their art), the offline session (default spawn open, pond swims, a bush blocks) and `AssetResolver` / `check_folder` for 2.05, 2.17, 2.22 and 2.31.

## How it is wired in (item 20)

The set is compiled into the binary and is the last layer of every lookup. The full rules are in `documentation/file-cache.md` (cache layout, naming, search order, what a download does, how `WANTFILE` / `UPDATEFILE` treat built-in files); this is the short version.

1. **Embedded, then written once.** `client/src/assets/builtin.rs` holds every file of `assets/default/` with `include_bytes!` (54.7 KB), and a test fails if that list and the folder differ. The client finds files by path (resolver, sound engine, readers), so at start-up the set is written to `<settings dir>/builtin/<id>/` (`<id>` = CRC-32 of the whole set) and that folder is the lowest layer. The files are written 0444 and a missing or changed one is rewritten from the binary, so the folder is a copy of what is in the executable and nothing else; a new build with new art uses a new `<id>` and leaves the old folder alone. If the settings folder cannot be written the set goes to the system temp folder; if that fails, the client runs without the layer and says so. No network, no shipped data folder next to the exe.
2. **Fallback.** `--assets` is optional. With no folder, a missing folder or an empty one, every lookup falls through to the built-in set; a folder with only some files uses its own where it has them and ours for the rest (file by file). An offline run with no `--assets` and no `--level` starts in `new1.nw` (the launcher's "Play offline" does the same); with `--assets`, behaviour is unchanged. `seihai --check-assets` with no `--assets` checks the built-in set (exit 0 for 1.39, 2.05, 2.17, 2.22 and 2.31, no required file missing, nothing unreadable); with `--assets DIR` it still checks exactly that folder.
3. **Per-server cache, searched first.** Search order is: the server's cache folder, the user's asset folder, the built-in set. Downloads are written to the cache only; the built-in set is read-only and never a download target (a target under it is refused). `save_downloads` / `--no-save-downloads` still gate writing. Layout and naming: `<cache root>/<host>_<port>[~<server name>]/` with the real clients' sub-folders; the root is `--cache-dir`, then `cache_dir=` in the settings file, then `<settings dir>/cache`.
4. **Built-in files and the server.** A file only the built-in set has is asked for with `WANTFILE`, with no mod time (never `UPDATEFILE`), so the server's copy can be saved in the cache and win; if the server has none, ours stays. `walk.gani`, `steps.wav` and `steps2.wav` (the real client's first-step requests) are asked for only if there is no copy anywhere, built-in included, which with this set means never in a normal run.
5. **Built-in levels are offline only.** Online, the resolver does not find `new1.nw` / `new2.nw` in the built-in set, so a server's level of the same name is requested from the server.
6. **Drawing.** `render_view` no longer draws the brown chest box and the tan sign board over the board. A sign is a trigger; its art is the board tiles under it. A chest record is drawn by the client itself (see `chest-drawing.md`); with the built-in sheet a chest whose board tiles already are a built-in chest keeps them and an opened one shows the open tiles (closed id + 2, see the tile table above). NPC placeholders are unchanged.

What was checked, offline, on Xvfb with this set only (screenshots of our own art, not committed; `/workspace/graal/seihai_notes/default-assets-offline-*.png` on the author's box): the starter level loads, the player walks (several directions, pose changes), chest and sign show their tiles, an opened chest shows the open art, the pond swims (head only, ring), and a run with an empty `--assets` folder looks the same. The tests are `client/tests/builtin_wiring.rs` (14) and the unit tests of `builtin.rs`, `servercache.rs`, `settings.rs`, the resolver and the file cache.

The generator `tools/gen_default_assets.py` already re-emits the whole set from a seed (`--check`, `--preview`), so the optional item 21 tool is mostly done by it; a parameterised front end (tile size, palette) is not part of this PR.

## Poses: sword, sit, hurt (item 22)

`sword.gani`, `sit.gani`, `hurt.gani` and `sounds/sword.wav` are generated by the same script (body rows 9 to 12 and the sword cells come from `body_pose_cell` and `build_sword`; the GANI text from `sword_gani`, `sit_gani` and `hurt_gani`; `--preview` draws every step). Like the rest of the set they are drawn from rectangles, ellipses and rotated copies of our own cells, and the sword sound is synthesised; no corpus image or audio was opened for them. What comes from `asset-reference.md` is only structure: the sword animation has five steps with a sound cue after step 0 and `SETBACKTO idle`, the sit animation has one step, the sword sheet is 128x96.

**How the client uses them** (`client/src/pose.rs`, `render::draw_gani_frame`):

- **S key.** The swing already ran in `combat.rs` (sprite value S = 6 to 10, one step per two 50 ms ticks). Step `i` of the swing now draws step `i` of `sword.gani` over the player's idle figure, and entering a step queues the `PLAYSOUND` cues that follow it, so `sword.wav` plays once when the swing starts. When the swing ends the pose is gone and the idle figure is drawn again; that is what `SETBACKTO idle` says, so the parser keeps the name (`Gani::setback_to`) and the test checks it. The swing is refused in water and while dead (unchanged).
- **A hit.** A hit that the judge accepts starts hurt mode (`take_hit`, unchanged: 40 ticks in which nothing can hurt the player, sprite S = 32 sent). While the first 500 ms of it run, `hurt.gani` is drawn. This works in the offline session too, because the pose follows the combat state, not the network.
- **Sit.** `sit.gani` is drawn while `OfflineSession::sitting` is set. Nothing sets it yet, since no key is bound.
- Priority: swing, then hurt, then sit. No pose while swimming or dead, and none when its GANI is not loaded (a build without GANIs, or a data folder that lacks the file), so the old idle drawing is the fallback.

**Seihai's own conventions, because the real clients' behaviour is not documented here:** the name `hurt.gani` and the idea of playing it on a hit; showing it for 500 ms rather than the whole of hurt mode (the sprite others see lasts the whole of it, and is not changed); the body rows 9 to 12, the sword sprite ids 300 to 311, and the hand positions the sword cells are placed at; sit being a flag with no key; one sword sheet (`sword1.png`) whatever the sword power. The walk and swim animations still only supply sounds, as before: the local player is not drawn with the walk poses (that is outside this item).

No file request is added for these (the first-step list is unchanged); a copy of one of them that arrives from a server is loaded in its place.

## Not settled

- The real `state.png` layout; ours is a placeholder.
- The swim head offset (14 px in code and here; 16 and 20 in the 2.22 animation).
- Whether a 1.33 to 1.39 client reads ids on a 1856-wide sheet the same way.
- The art is functional and small, not final: no hats, no tile variety beyond what the two levels need. Item 21 (generator tool) can grow it.
