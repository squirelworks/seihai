# Extra player-world archives: derived-facts inventory (2026-10-25)

Derived counts only: no level, script or account contents. Source: the user's shared Drive folder, 92 `.7z` archives (357 MB packed, 1.6 GB of file data, 2.1 GB on disk) in `/workspace/graal/lab/playerworlds_extra/<archive-stem>/` (mode 700, box-only, not for any repo or upload). Per-world numbers: `playerworlds-extra-inventory-2026-10-25.tsv`. Magic = first bytes of every file, whatever its extension.

## Totals

- Worlds: 92; files: 214,814; unpacked: 1.63 GB.

| Magic | files | worlds |
|--|--:|--:|
| `GLEVNW01` | 70,659 | 76 |
| `GR-V1.03` | 27,189 | 57 |
| `GR-V1.02` | 781 | 6 |
| `GR-V1.01` | 457 | 4 |
| `GR-V1.00` | 21 | 4 |
| `GR-V1.04` | 0 | 0 |
| `Z3-V1.04` | 34 | 4 |
| `Z3-V1.03` | 26 | 4 |
| `Z3-V1.02` | 0 | 0 |
| `GRMAP001` | 75 | 32 |
| `GRMAP002` | 1 | 1 |
| `GRNPC001 (NPC export .txt)` | 15,075 | 20 |
| `GRAWP001 (weapon export .txt)` | 2,720 | 22 |
| `GRACC001 (account export .txt)` | 993 | 1 |
| `GRPKG001 (.gupd)` | 5 | 2 |
| `GANI0001` | 6,682 | 45 |
| `GANI0002` | 142 | 16 |

By name: `.nw` 70,645, `.graal` 27,240, `.zelda` 1,040, `.gmap` 72, `.gani` 10,167, images 74,704, audio 2910, `.txt` 26,816, `.gs/.gs2` 0.

Compared with the existing corpus (`lab/playerworld`: GR-V1.03 + GLEVNW01 only, no `.zelda`, no Z3, no GR-V1.00 to .02): this set adds GR-V1.00/.01/.02, Z3-V1.03/.04, GRMAP002 and GRPKG001, and many gmaps and ganis.

## Good test candidates

**Z3-V1.0x levels (`.zelda` / `.graal` / `.editor` names), counts of Z3 files:** Graal-1.1.9-LevelPack (27), Graal-1.3-LevelPack (14), GraalClassicJanuary2000 (14), kimjonghackedyou (3), Termina-Classic-LevelPack-1 (2).

**GR-V1.00 to .02 variants (old levels; count of those files):** Graal-1.1.9-LevelPack (505), Graal-1.3-LevelPack (282), GraalClassicJanuary2000 (282), Termina-Classic-LevelPack-1 (168), mantara (21), valikoria-1 (1).

**Many files named `.zelda` (GR-V1.0x or Z3 inside):** Graal-1.1.9-LevelPack (390), Graal-1.3-LevelPack (303), GraalClassicJanuary2000 (303), kimjonghackedyou (34), Saber-Of-Power-Gold (6), jedi (3), mantara (1).

**GR-V1.03 `.graal` heavy (loader volume):** Enigma-12-21-06 (3123), Graal-1.3-LevelPack (3095), Termina-Classic-LevelPack-1 (2989), Delteria-12-6-02 (1697), Elven-Lands-2001 (1523), GraalClassicJanuary2000 (1507), Doomsday-Nov-2002 (1244), Enigma-Graal-2004-02-28 (1207).

**GLEVNW01 `.nw` heavy:** Final-Doomsday (9917), Doomsday-2003 (4648), Atrius-10-26-06-1 (4449), Enigma-12-21-06 (3365), GraalClassic-12-9-06 (3162), Maloria-12-21-06 (3078), Era-Full-Backup-Oct-30-2006 (2695), Enigma-Graal-2004-02-28 (2670).

**Mixed formats in one world (both GR-V1.03 and GLEVNW01 in volume):** Enigma-12-21-06 (3123), Enigma-Graal-2004-02-28 (1207), Delteria-12-6-02 (916), Cora (571), Doomsday-Nov-2002 (559), Rudora (482), bravo-31-December-2002 (348), Maloria-03-04 (312).

**Gmaps (GRMAP001 `.gmap`):** elemental_kingdoms (11), Final-Doomsday (8), Dragandia (4), Hyroko (4), Stonehenge (4), bravo-evolved-12-11-04-1 (4), Atrius-The-Final-Legacy-1 (3), Doomsday-2003 (3), Elven-Lands-Joey-Revision (3), Shangri-La-10-02-06 (3). Hyroko holds the only GRMAP002 file.

**Gani-heavy (client animation loader):** Lumina-1 (2467), Final-Doomsday (1131), Doomsday-2003 (1101), elemental_kingdoms (1027), Atrius-10-26-06-1 (686), Maloria-03-04 (492), Maloria-2002 (479), graal2001 (431).

**Export-only text dumps, no levels (GRNPC001/GRAWP001 parsers):** Zodiac-06-17-07-1 (no level file at all), Joey-Era-Edit (1 `.nw`).

**Tiny / smoke worlds:** Playerworld130-1, Dargaro-Space-World-1, Kuburiki-1 (15 to 32 files), alkarenhyralt_levels (3 files, no levels).

**Old-client era by name (2000 to 2002):** GraalClassicJanuary2000, graal2001, Doomsday-2001, Elven-Lands-2001 (1523 GR-V1.03, no `.nw`), Doomsday-Nov-2002, Delteria-12-6-02, bravo-31-December-2002, Maloria-2002, Graal-1.1.9-LevelPack, Graal-1.3-LevelPack. Graal-1.3-LevelPack and GraalClassicJanuary2000 share an identical GR-V1.00..02/Z3 subset (same per-magic counts), so they likely overlap.

**Wrong-extension / cross-named files** (loader robustness): `GR-V1.03` inside `.nw` (Enigma-12-21-06 9, Enigma-Graal-2004-02-28 9, Doomsday-2003, Lumina-1, Providence, silverdreams); `GLEVNW01` inside `.graal` (Delteria, Doomsday-Nov-2002, bravo-31-December-2002, Enigma-12-21-06, Server-Interface-Test 7, silverdreams 7); GR-V1.03 under `.event` (Server-Interface-Test 8), `.bak`, `.vip`, `.un`, `.sil`, `.lost`, `.forest`, `.gone`; `GLEVNW01` under `.copy` (graal2001 11), `.png` (Enigma-12-21-06 14), `.gif` (Millenium 2004/2006, 1 each), `.temp`; `GRMAP001` under `.txt` or `.graal`.

## New magic / format variants (not in the existing notes)

Already known: GR-V1.00 to .04, GLEVNW01, Z3-V1.02/.03/.04, GRMAP001, GANI0001/2, GRNPC001. **Not seen in the notes before:**

- `GRMAP002` in `.gmap`: 1 file (Hyroko). A second gmap revision.
- `GRAWP001` (weapon export text, `.txt`): 2,720 files in 22 worlds.
- `GRACC001` (account export `.txt`): 993 files, all in Lumina-1. These are personal account data. I did not open any; treat that world as sensitive and do not use it server-side.
- `GRPKG001` in `.gupd`: 5 files (Stonehenge 3, Thamhic_Release 2).
- `GRNPC001` and `GANI0001/0002` appear widely; `GANI0001` without extension in 3 worlds (Cythus-Med-1, Lumina-1, Shangri-La-10-02-06).
- No `GR-V1.04` and no `GLEVNW02` or other `GLEVNW` revisions; no `Z3-V1.02`; no unknown `GR-V1.xx` or `Z3-V1.xx` revisions.
- Level-extension files with no recognised magic (all single or few): ZIP data named `.graal` (Cora 1), PNG named `.graal` (Delteria 1), GIF named `.graal` (Graal-1.3-LevelPack 1, Termina-Classic-LevelPack-1 1), PNG named `.nw` (dark_nations 2), RAR named `.nw` (Enigma-12-21-06 1, Enigma-Graal-2004-02-28 1), 4 `.nw` files that are headerless ASCII text (Doomsday-2003, Elven-Lands-Joey-Revision), 1 `.nw` with binary header `4c00000001140200` (bravo-31-December-2002).

## Not done
- No Seihai `--level-info` / `--check-assets` pass (no release binary built); parsing is untested on these worlds. Counts come from file headers only.
- Zero-byte level files: Doomsday-Nov-2002 (1), Elven-Lands-2001 (2), Enigma-12-21-06 (27), Enigma-Graal-2004-02-28 (1), Faharia-1 (1), Final-Doomsday (5), GraalClassic-12-9-06 (1), Maloria-12-21-06 (1), dark_nations (4).

## Extraction notes
All 92 archives downloaded (0 failed) and extracted with no path escapes or symlinks. 87 via py7zr, 5 (Cora, Final-Doomsday, Hyroko, Lumina-1, S.A.I) via the official 7-Zip 25.01 Linux binary, saved at `/workspace/tools/7zz/7zz`, because py7zr cannot do BCJ2.
