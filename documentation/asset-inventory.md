# Playerworld asset inventory (item 17)

Derived counts only, from file headers and from running Seihai's own readers. The two corpora are private, box-only archives of old player worlds. No file, level, script, image or sound from them is in this repository, and none should be added. The only things committed for this item are this note and `asset-inventory.tsv`.

Dated 2026-10-25. Seihai main at `79489c1` (after #48).

## Corpora

| Corpus | Where (box only) | Worlds | Files | On disk |
|--|--|--:|--:|--:|
| `first` | the first playerworld (`Enigma - 12-21-06`) | 1 | 13,934 | 80.7 MB |
| `extra` | 92 archives from the shared Drive folder, one directory each | 92 | 214,814 | 1,626.1 MB |

`asset-inventory.tsv` has one row per world (93 rows: `corpus`, `world`, then every column below) and three total rows: `TOTAL_first`, `TOTAL_extra`, `TOTAL_all`. It extends the header-only table of the earlier extra-worlds note with image, audio, GANI and parse columns.

**Overlap, so totals are an upper bound.** Comparing (file name, size) pairs: `first` is the same data as `Enigma-12-21-06` (100% of the smaller set). `Enigma-Graal-2004-02-28` is a 100% subset of `Enigma-12-21-06`. `Maloria-2002` is a 100% subset of `Maloria-03-04`. `GraalClassicJanuary2000` is a 100% subset of `Graal-1.3-LevelPack`, which overlaps `Termina-Classic-LevelPack-1` by 84%. The two Millenium worlds overlap by 91%. `TOTAL_extra` is therefore the better "distinct-ish" figure and `TOTAL_all` counts the Enigma world twice.

## How it was counted

- **By magic, not by extension.** A level is classified by its first 8 bytes. The extension counts (`ext_*`) are separate columns, so wrongly named files show up as a difference between the two.
- **Images** are classified by signature (PNG, GIF, MNG) and sized from the header only (PNG `IHDR`, GIF logical screen, MNG `MHDR`). PNG is "idx4" for colour type 3 at bit depth 4, "idx_other" for other palette depths (all but a handful are 8-bit), "truecolor" for types 2 and 6 and "gray" for 0 and 4. A GIF is always palettised; it is split by global colour table size (16 or fewer = 4-bit, more = 8-bit; `gif_no_gct` has no global table).
- **Dimension buckets** use the larger side of the image: `dim_le32`, `dim_33_64`, `dim_65_128`, `dim_129_256`, `dim_257_512`, `dim_513_1024`, `dim_1025_2048`, `dim_gt2048`. `sheet_2048x512` and `sheet_1856x512` count the exact tile-sheet sizes Seihai already knows.
- **GANI** is counted by magic (`GANI0001`, `GANI0002`) and, separately, by a `.gani` extension with no magic (`gani_ext_nomagic`). The stock animations (`idle.gani`, `walk.gani`, `swim.gani`, `sword.gani`, `sit.gani`) start with `SPRITE` and have no magic, so most real GANIs are in the third column.
- **Audio** is `.wav` by `RIFF/WAVE` (split by format tag into plain PCM, IMA ADPCM and other), MP3 by `ID3` or an MPEG frame header, MIDI by `MThd`, OGG by `OggS`.
- **Tiledefs** are files with `tiledef` in the name. Of the 8, one is a GIF; the other 7 are text.
- **NPC and weapon exports** are `GRNPC001` and `GRAWP001` text files, counted only. `GRPKG001` is the package/update format.
- **Account exports** (`GRACC001`, 993 files, all in `Lumina-1`) were not opened. In that world no `.txt` file was opened at all, so its `GRNPC001`, `GRAWP001` and `GRPKG001` counts are the earlier header-only counts carried over, and `txt_unopened` (1,283) records how many files were skipped. That world should not be used against any server.

## Headline counts

| Type | `first` | `extra` | Notes |
|--|--:|--:|--|
| `GLEVNW01` (`.nw`) levels | 3,365 | 70,659 | 77 of the 93 rows; extension `.nw` on 74,002 files in total |
| `GR-V1.03` (`.graal`) | 3,123 | 27,189 | 58 worlds have any `GR-V1.0x` |
| `GR-V1.00` / `.01` / `.02` | 0 | 21 / 457 / 781 | only the old level packs and a few small worlds |
| `GR-V1.04` | 0 | 0 | none |
| `Z3-V1.03` / `.04` (`.zelda`) | 0 | 26 / 34 | 5 worlds; 1,040 files are named `.zelda`, most hold `GR-V1.0x` inside |
| `Z3-V1.02` | 0 | 0 | none |
| `GRMAP001` / `GRMAP002` (`.gmap`) | 2 | 75 / 1 | 33 worlds; the one `GRMAP002` is in Hyroko |
| PNG | 3,734 | 44,308 | |
| GIF | 2,717 | 28,200 | |
| MNG | 150 | 1,836 | |
| GANI, magic `GANI0001` / `GANI0002` | 159 / 4 | 6,682 / 142 | 53 worlds have any GANI |
| GANI, `.gani` with no magic | 115 | 3,346 | stock animations are here |
| WAV | 50 | 1,880 | 31 worlds; 1,846 PCM, 8 IMA ADPCM, 26 other format |
| MP3 / OGG | 0 / 0 | 35 / 25 | MP3 in 5 worlds; OGG only in Lumina-1 |
| MIDI | 113 | 970 | |
| Tiledefs (name) | 0 | 8 | Cora, Doomsday-2003, Final-Doomsday, Thamhic_Release, revenge-online-12-2-04 |
| `GRNPC001` NPC exports | 32 | 15,075 | counted only |
| `GRAWP001` weapon exports | 88 | 2,720 | counted only |
| `GRPKG001` | 0 | 5 | Stonehenge 3, Thamhic_Release 2 |
| `GRACC001` account exports | 0 | 993 | Lumina-1 only; not opened |

Largest worlds by type (all in the tsv):

- **Levels, `.nw`:** Final-Doomsday 9,917, Doomsday-2003 4,648, Atrius-10-26-06-1 4,449.
- **Levels, `.graal`:** Enigma-12-21-06 3,123, Graal-1.3-LevelPack 3,095, Termina-Classic-LevelPack-1 2,989.
- **GANI:** Lumina-1 2,468, Final-Doomsday 1,131, Doomsday-2003 1,101, elemental_kingdoms 1,027.
- **WAV:** Lumina-1 1,180, Final-Doomsday 129, graal2001 126, Shangri-La-10-02-06 124.
- **NPC exports:** Era-Full-Backup-Oct-30-2006 7,867, GraalClassic-12-9-06 2,875, Zerati-10-26-06-1 1,979.
- **Weapon exports:** Zodiac-06-17-07-1 554, Era-Full-Backup-Oct-30-2006 321.

### Images: palette and size

| | `first` | `extra` |
|--|--:|--:|
| PNG 4-bit palette | 729 | 6,024 |
| PNG other palette depth (nearly all 8-bit) | 2,848 | 36,439 |
| PNG truecolor (RGB or RGBA) | 156 | 1,826 |
| PNG gray | 1 | 19 |
| GIF with 16 or fewer colours | 867 | 8,425 |
| GIF with more than 16 colours | 1,849 | 19,751 |
| GIF with no global table | 1 | 24 |

| Largest side | `first` | `extra` |
|--|--:|--:|
| up to 32 px | 1,271 | 11,777 |
| 33 to 64 | 1,166 | 12,170 |
| 65 to 128 | 1,164 | 10,497 |
| 129 to 256 | 1,017 | 10,017 |
| 257 to 512 | 200 | 2,260 |
| 513 to 1,024 | 1,776 | 27,444 |
| 1,025 to 2,048 | 6 | 162 |
| over 2,048 | 1 | 17 |

Exact sizes seen most often across all images (PNG, GIF, MNG), which line up with the standard Graal player art: `32x560` (21,504, head sheets), `32x32` (6,158), `192x144` (5,022), `128x720` (2,664, body sheets), `128x96` (2,158, sword sheets), `76x40` (1,837) and `38x20` (1,708, shield sheets), `16x16` (1,141). A `2048x512` sheet, the tile-sheet size Seihai assumes, appears in 70 files across 28 worlds (2 in `first`, 68 in `extra`); one `1856x512` sheet exists.

**Palette facts for the starter files.** In the candidate worlds, `pics1.png` is an 8-bit palette PNG, while `body.png` (128x720), `sword1.png` (128x96) and `shield1.png` (38x20) are 4-bit palette PNGs, and heads are 32x560 GIFs with a 16-colour table (or 4-bit PNGs in Lumina-1). So Seihai's palette reader has to handle both 4-bit and 8-bit palette PNGs, and GIF (no decoder yet) for heads.

## Starter-set candidates

Names are file names only. The scoring is simple: one point each for a `pics1.png`-style sheet, a `body` sheet, a `head` sheet, a `sword` sheet, a `shield` sheet, `idle.gani`, `walk.gani`, `swim.gani`, a `steps`/`steps2` WAV, a water-style WAV, and at least one small level (a non-empty `GLEVNW01` or `GR-V1.03` file of 12,000 bytes or less). The per-world points are the `starter_*` columns of the tsv. The maximum is 11.

| Score | World | Has |
|--:|--|--|
| 10 | Final-Doomsday | `body.png` (128x720, 4-bit), `head01.png` and others (32x560), `sword1.gif`, `sword2.gif`, `sword3.gif` (24x24), `shield1.png` (96x20), `idle.gani`, `walk.gani`, `swim.gani`, `sword.gani`, `sit.gani`, `steps.wav`, `steps2.wav` (22,050 Hz, 8-bit), `water.wav`, `waterfall.wav`; no `pics1` sheet |
| 10 | Lumina-1 (do not use server-side) | `body.png`, `head0.png`, `sword1.png`, `shield1.png`, the same five GANIs, `steps.wav`, `steps2.wav`, `water.wav`; no `pics1` sheet; holds the account exports |
| 9 | Maloria-03-04 | `pics1.png` (2048x512, 8-bit palette), `body.png` (128x720, 4-bit), `head118.gif` and others (32x560), `sword1.png` (128x96, 4-bit), `shield1.png` (38x20, 4-bit), `idle.gani`, `walk.gani`, `swim.gani`, `sword.gani`, `sit.gani`; no footstep WAV |
| 8 | Maloria-2002 | Same art as Maloria-03-04 without `pics1.png` (an exact subset of it) |

Reading the table: **Maloria-03-04 is the best single source for the sheets** (it has the only complete set of `pics1.png`, body, head, sword, shield and the three core GANIs together), and **Final-Doomsday supplies the sounds** (`steps.wav`, `steps2.wav`, `water.wav`; `Doomsday-Nov-2002` also has `steps2.wav` and `water.wav`). `pics1.png` itself also exists in `Coronia-1` and `Hyroko` (2048x512, 8-bit palette; the two files have the same size, 210,232 bytes, which suggests the same sheet, and Maloria's is 261,789 bytes). Other worlds with a `2048x512` tile sheet under other names include BlackPoint-1, Mag-07-1 (`pics1d-2007.png`), Renegade (`rene_pics1.png`), FastLane (`pics1server.png`) and Shangri-La-10-02-06.

Worlds with all three core GANIs: Doomsday-2003, Elven-Lands-2001, Final-Doomsday, Lumina-1, Maloria-03-04, Maloria-12-21-06, Maloria-2002, graal2001. Worlds with a `steps` WAV: Doomsday-Nov-2002, Final-Doomsday, Lumina-1. There is no swim sound by name other than `water.wav` (a splash/loop, 8 kHz 16-bit) and Lumina's `mc_link_swim.wav` and `la_link_swim.wav`.

**Small levels** (full 64x64 boards of 8 to 11 kB, with a few links, an NPC, a chest or a sign): from Final-Doomsday, `heartquestfour21.nw` (9,585 bytes: 2 links, 1 NPC, 1 chest, 1 sign) and `craig_questwater6.nw` (9,795 bytes: 2 links, 2 NPCs, 3 chests, 1 sign); from Maloria-03-04, `insidefireblast.nw` (9,412 bytes). Almost every `GLEVNW01` level is about 9 kB, so size does not separate them; a link, chest and sign count does. For the old binary format, any `GR-V1.03` file of a few kB works (for example the 3,478-byte `maloria_i-03.graal`).

**Provenance.** This is all player-made or community-made art and audio inside private archives. It is reference only: item 19 draws its own sheets, GANIs and sounds, and uses the files above only for sizes, layout and tile positions.

## Parse pass rates (Seihai readers, current main)

Run with a release build of main and a small throwaway program (not committed) that calls the same readers the client does, on every non-empty file of each kind. Numbers only. "Pass" means the reader returned `Ok` (`Some` for gmaps); for `GR-V1` and `Z3-V1` levels it also means no unread tail or layout anomaly.

| Format | Reader | Tried | Pass | Rate |
|--|--|--:|--:|--:|
| `GLEVNW01` (`.nw`) | `Level::parse` | 74,024 | 74,023 | 99.999% |
| `GR-V1.00` to `.03` | `Level::parse_binary_with_layout` (`--level-info`) | 31,571 | 31,571 | 100% |
| `Z3-V1.03` / `.04` | `ZeldaLevel::parse` | 60 | 60 | 100% |
| `GRMAP001` | `Gmap::parse` | 74 | 74 | 100% |
| `GRMAP002` | `Gmap::parse` | 1 | 0 | 0% (not supported; the one file) |
| GANI (magic and no-magic) | `Gani::parse` | 10,447 | 10,447 | 100% |
| PNG | `read_png_rgba` | 48,045 | 47,946 | 99.79% |
| WAV | `decode_wav` | 1,914 | 1,904 | 99.48% |
| Tiledefs (text) | `parse_tiledefs` | 7 | 6 | 85.7% (see below) |

(The GANI row covers the 6,987 files with magic plus the 3,461 `.gani` files without, minus one zero-byte file.)

By palette type, PNG pass is 100% for 4-bit palettes (6,753 of 6,753), 99.89% for other palettes (39,244 of 39,287) and 97.2% for truecolor and gray (1,949 of 2,005).

**Failure causes**, from the reader's own error text:

- **PNG (99 files):** 77 are interlaced (Adam7) 8-bit PNGs (the reader says "unsupported PNG bit depth or interlace"), 18 are truncated files ("truncated chunk"), 2 are files named `.png` that are not PNGs, 1 is a 1-bit gray PNG and 1 is too short to hold a header.
- **WAV (10 files):** 8 are format tag 0x55 (MP3-in-WAV) and 2 are tag 2 (MS ADPCM). Both are unsupported formats, not corruption.
- **GRMAP002** is unsupported (one file in Hyroko).
- **Tiledefs:** one of the 7 text files does not parse; the other 6 do.
- **`.nw` panic (a real reader bug).** One level, `s_b-skate-park.nw` in Delteria-12-6-02, makes `Level::parse` **panic** (`slice index starts at 1 but ends at 0`, `client/src/assets/level.rs`, `parse_npc`). It has an `NPC` header line with no body and no `NPCEND`; `lines[1..lines.len().saturating_sub(1)]` then slices `1..0` when the block is a single line. The fix is to return `None` (or an empty script) when there are fewer than two lines. Not fixed in this PR (docs only); a one-line change plus a test.

A second thing the scan showed: `--level-info` is only right for binary levels. It does not reject a text `GLEVNW01` or `GRMAP001` file; it prints a nonsense layout instead (counts that do not mean anything) and exits 0. A text `.nw` has to go through `Level::parse`, and a `.gmap` through `Gmap::parse`. The numbers above use the right reader for each magic.

**`--check-assets` on world roots.** `seihai --assets <world root> --check-assets --version 2.22` finds only 1 of 29 listed files in Maloria-03-04 (the root `pics1.png`, read as a 2048x512 PNG), 0 of 29 in Final-Doomsday, and 1 of 49 in the first playerworld (`idle.gani`). That is expected, not a parse failure: the resolver searches fixed folders (`levels/`, the root, and the like) and the worlds keep their files in their own sub-folders. The same run's level scan reads every `.graal`/`.zelda` by magic: in the first playerworld 3,141 files, of which 3,113 are `GR-V1` and 28 are not (flagged `BAD`: not `GR-V1` or `Z3-V1` by magic, so `GLEVNW01` or other data under a level extension); in Graal-1.3-LevelPack 3,312 files, 5 `Z3-V1.03`, 8 `Z3-V1.04`, 3,298 `GR-V1` and 1 unrecognised. Item 18 should stage a clean folder from these files rather than point the resolver at a world root.

## Other findings

- **Wrong-extension and cross-named levels** (detailed in the earlier extra-worlds note): `ext_nw` and `ext_graal` differ from the magic counts (for example `ext_graal` is 30,381 and `GR-V1.0x` by magic is 31,571, because `.zelda`, `.bak`, `.event` and other names hold `GR-V1.03`). A loader must go by magic.
- **Several stock GANIs have no magic** (start with `SPRITE`), as the table shows. The GANI reader accepts them.
- **No `GR-V1.04` or `Z3-V1.02` files** exist in the corpora, so those two layouts stay untested on real data.
- **Zero-byte level files** exist (70 across both corpora, by extension). They have no magic and were skipped by the parse run.
- **Audio** is almost all PCM WAV (8-bit 22,050 Hz and 16-bit 8,000 Hz are the common stock formats) plus MIDI; MP3 and OGG are rare (35 and 25 files).

## Caveats

- Counts are from headers, and parse numbers are from this build of Seihai. Seihai has no GIF or MNG decoder, so GIF and MNG files are counted and sized but not decoded (about 30,900 GIF and 1,986 MNG files).
- `TOTAL_all` double counts the Enigma world; see Overlap.
- In Lumina-1 no `.txt` file was opened, so text-export counts there are carried over from an earlier header-only scan (not re-measured) and tiledefs there would have been missed (none are named).
- A "starter candidate" is judged by file names and header sizes only. Nothing was viewed, and a file with the right name may still be the wrong art (there are over 2,000 head sheets in Lumina-1 alone).
- Name matching is on the basename only, so the same name in two folders of one world counts once in the "has" columns but twice in the totals.
- The probe program, the scan scripts and all intermediate data live on the box outside the repository. The tsv reproduces the counts; the corpora themselves are not redistributable.
