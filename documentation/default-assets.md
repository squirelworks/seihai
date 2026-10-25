# Default asset set (item 19)

Seihai ships a small, self-contained asset folder, `assets/default/`, so a fresh download can open a level, walk, swim and hear footsteps without a server and without any file from the original clients. This note says what is in it, how it is made, which layout contracts it meets, and how item 20 will wire it in.

Dated 2026-10-25. Base: `main` at `d5389c2`. This branch also carries the commits of #49 (inventory) and #51 (reference sheets), because #51 was still open when it was written; if #51 has been merged the merge is empty for those files.

## Originality and licence

**Every file is original.** It was drawn or synthesised by `tools/gen_default_assets.py`: images from rectangles, ellipses, polygons and hand-written palettes (Pillow `ImageDraw` on palette images); sounds from sine tones and filtered noise; the GANIs and levels are text written by the script. Nothing was read from, traced from, recoloured from, resampled from or otherwise derived from any pixel or audio sample of the playerworld archives or the Graal clients, and the images of those archives were not opened for this item. The only things taken from them are the numbers already published in `asset-reference.md` (sheet sizes, grid layouts, frame and sprite counts, sound formats), which describe a file format, not content.

**Licence:** the same as the Seihai repository (`MIT OR Apache-2.0`, see the workspace `Cargo.toml`), for the generated files and for the generator. *The maintainer is asked to confirm this in the PR*; if a different licence is wanted for art and sound (CC0, for example), that is one line in this note and a `LICENSE` file in `assets/default/`.

## Contents (47.8 KB in 17 files)

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
  sounds/steps.wav           22,050 Hz, 8-bit, mono, 150 ms                          3.3 KB
  sounds/steps2.wav          22,050 Hz, 8-bit, mono, 150 ms (different pitch)        3.3 KB
  sounds/water.wav           8,000 Hz, 16-bit, mono, 450 ms                          7.2 KB
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

The foot test the sheet was drawn for: `FOOT_DX = 2.0`, `FOOT_DY = 2.75`, one point per player, tile under `floor(x + 2.0), floor(y + 2.75)`.

### Player sheets

- **`body.png` 128x720**: four 32 px direction columns (`x = 0, 32, 64, 96` = up, left, down, right) and 22 rows of 32 px (22 x 32 = 704, plus 16 spare). Row 0 is the idle pose (the cell `draw_player_sprites` reads at `(dir * 32, 0)`); rows 1 to 8 are the eight walk poses (foot-falls at poses 1 and 5). Rows 9 to 21 are empty and reserved. Body sprite id = `200 + 4 * pose + dir`. The "right" column is the "left" column mirrored.
- **`head0.png` 32x560**: one 32 px column, rows `y = 32 * dir` for the four directions (read at `(0, dir * 32)` by `draw_player_sprites`; head sprite id `100 + dir`). Rows from y = 128 down are empty; the extra head poses of other sets (hurt, sword) are not drawn.
- **`sword1.png` 128x96**: four direction columns by three rows of 32x32 (held, mid-swing with an arc, thrust). No `sword.gani` ships yet, so nothing refers to it; the size matches the contract.
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

Offsets are the idle offsets already in `draw_player_sprites` (`OFFSETS`: shadow, body, head per direction inside the 48x48 player box). The swim head is lowered by 14 px, the number `draw_player_sprites` uses; the 2.22 animation uses 16 or 20 (asset-reference, "not settled"), and this set follows the code so that the GANI and the renderer agree until that is settled.

### Sounds

Mono PCM WAV files in the stock formats: footsteps at 22,050 Hz 8-bit (`steps.wav` for builds without GANIs, `steps2.wav` for the walk cues), the water sound at 8,000 Hz 16-bit. Synthesised from sine tones with a falling pitch and low-pass filtered noise (footsteps: a thud with a decaying envelope; splash: band-passed noise with four short rising bubble chirps). `decode_wav` reads all three. No sword, bomb or item sound is provided (all are optional in the checks).

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

Python 3.9 or later and Pillow; no network. The seed is `SEED = 20261025` in the script and every random choice (grass tufts, noise, scatter on the levels) comes from `random.Random(SEED + n)`, so two runs give identical files on the same machine; `--check` compares decoded pixels for the PNGs (not bytes, because zlib output can differ between builds) and bytes for everything else. The tree is small (47.8 KB), so it is committed; the generator is the source of truth, and a change to the art is a change to the script plus a regenerated tree.

The contract test is `client/tests/default_assets.rs` (`cargo test --test default_assets`): it loads each file with Seihai's own readers and checks sizes, bit depths, the 128x32 grid, that every named tile id lands on its own cell and `blit_tile` draws that cell, the type table (water 11, walls 20 or more, ids below 4096), the GANI structure (1, 8 and 8 steps, four rows each, the cues, sprites inside their sheets), WAV formats and lengths, both levels (every id named, links lead to existing levels and arrive on open floor, chests and signs sit on their art), the offline session (default spawn open, pond swims, a bush blocks) and `AssetResolver` / `check_folder` for 2.05, 2.17, 2.22 and 2.31.

## How item 20 will wire it in

Not done in this PR (no resolver or client changes). The plan, with the maintainer's decision of 2026-10-25 (t119u):

1. **Fallback.** `AssetResolver` gets the default folder as its last search root: a file that is not found in the chosen assets folder (or the folder is missing or empty) is looked up in the built-in set. The default set is **read-only**: it is never a download target and nothing ever overwrites it. How it is built in (copied next to the binary or embedded) is item 20's choice; `--assets` keeps pointing at a user folder when given.
2. **Per-server download cache, searched first.** What a server sends goes to a cache folder of its own, not into the assets folder, as the real clients' cache did (this is how defunct playerworlds were kept): `<cache root>/<server key>/...` with the same sub-folders the clients use (`levels`, `levels/ganis`, `sounds`, `maps`, and so on, as in `documentation/file-cache.md`). The server key is the game server's host and port made file-safe, for example `play.example.org_14900`. The search order for a name is: that server's cache, then the user's assets folder, then the built-in defaults. `save_downloads` (the real `saveimages`) still decides whether anything is written; with it off, downloads stay in memory for the session and the cache is not touched.
3. **Requests for files Seihai already has.** Per the answer of 2026-10-25 (t122u), `walk.gani` and the step sounds are requested only when Seihai has no copy: a cached copy or a built-in default counts as having it. Whether built-in files are offered with `UPDATEFILE` or treated as missing for `WANTFILE` is also settled in item 20.
4. **Acceptance.** `--check-assets` succeeding on the built-in set for 2.05, 2.17, 2.22 and 2.31 (it already does when pointed at this folder), the starter level opening and the player walking and swimming offline, and chests and signs drawn from tiles instead of the placeholder boxes in `render_view`.

## Not settled

- The licence of the art and sound, until the maintainer confirms (see above).
- The real `state.png` layout; ours is a placeholder.
- The swim head offset (14 px in code and here; 16 and 20 in the 2.22 animation).
- Whether a 1.33 to 1.39 client reads ids on a 1856-wide sheet the same way.
- The art is functional and small, not final: no hats, no sword animation, no tile variety beyond what the two levels need. Item 21 (generator tool) can grow it.
