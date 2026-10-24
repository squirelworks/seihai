# Next work for Chiaki — asset generation track

Current queue after items 1–15 and 16. Open the PR for one item, then start the next. Do not wait for merge.

## Immediate (still running)

### Item 15 — Differential-testing divergences

Running in the private lab. One PR per divergence (rectangle links, login props, 2.22 position flags, early id 46, asset requests, walk speed, etc.). Check each fix against a lab capture first; skip with a note if the bytes are not shown.

## After item 15 — original assets for Seihai

The playerworld archives supply reference material. Goal: a clean, self-contained default asset set that ships with Seihai so a fresh download can open a level and walk without relying on a server or the original client files.

### Item 17 — Inventory the playerworld archives
The extra archives are already inventoried. See `documentation/inventories/playerworlds-extra-inventory-2026-10-25.md` and the matching `.tsv`. Use that as the starting point; do not re-scan the whole set unless a specific world needs a deeper look. Note the sensitive Lumina-1 GRACC001 account exports — do not open or use them.



- Scan the archives that were handed over.
- Produce a short inventory: counts of .nw / .graal / .zelda / .gmap, PNG/GIF/MNG sheets, GANI files, WAV/MP3, tiledefs, NPC scripts.
- Flag anything that looks like a complete starter set (pics1-style sheet, body/head sheets, idle/walk GANIs, basic sounds, one or two small levels).
- Output: a note in `documentation/asset-inventory.md` plus a tsv of file counts by type. No raw files committed.

### Item 18 — Extract and clean reference sheets

- Pull the most usable tile sheets, body/head/sword/shield sheets, and core GANIs into a private working folder.
- Document the tile id mapping that Seihai already uses (column `(t % 16) + (t / 512) * 16`, row `(t / 16) % 32`) and confirm it matches the sheets.
- Note which sheets are 4-bit palette PNGs vs true-colour, and which need the existing palette reader.
- Output: a short `documentation/asset-reference.md` describing the best source files and any clean-up steps. Do not commit the original sheets.

### Item 19 — Generate a default asset root

- Create a new `assets/default/` tree under the repo (or a sibling folder that the build can copy) containing:
  - A minimal pics1-style tile sheet (grass, bush, rock, water, wall, chest, sign) that matches Seihai's 128-column assumption and the foot-test offsets.
  - Body and head sheets sufficient for the four walk directions plus idle and swim (head-only in water).
  - The core GANIs (idle, walk, swim) with the sound cues already fixed in the parser.
  - A handful of short WAV footsteps and a swim sound.
  - One small starter level (text GLEVNW01 or .nw) with a few links, a sign, and a chest so offline play has something to open immediately.
- Gate by version profile where it matters (older builds may need different sheet widths).
- Output: the asset tree plus a note in `documentation/default-assets.md`. The assets themselves stay out of the main source tree if they are large; document how to fetch or generate them.

### Item 20 — Wire the default assets into the client

- Update `AssetResolver` and the `--assets` default so a missing or empty assets folder falls back to the built-in set.
- Make `seihai --check-assets` succeed against the default set for the profiles that matter (2.05, 2.17, 2.22, 2.31).
- Confirm the offline starter level loads, the player walks and swims, and the chest/sign render without green or yellow placeholder boxes.
- Output: a Seihai PR that adds the fallback path and a test that the default set loads.

### Item 21 — Simple asset generator tool (optional)

- A small CLI under `tools/` that can re-emit the default tile sheet, body sheet, and starter level from parameters (seed, tile size, colour palette). Useful for regenerating if the sheets ever need to change.
- Output: the tool plus a one-page note. Skip if the hand-crafted set is already solid.

## Still open (do not drop)

- Item 13 — Early builds: a real 2.00-era listserver reply (still unobserved).
- Item 14 — Public-server checks: blocked from the build box; checklist is in `documentation/public-checks.md`. Needs a network that can reach 14921–14923.
- Level editor implementation: notes are in Graal_analysis `tools/level-editor/`. Copy the checklist into Seihai `documentation/` and start the tool when the asset track is under way.
- Damage/respawn leftovers (knockback, packet-40 field, bombs/arrows/baddies) and file-transfer leftovers (`GWEBL001`, login UPDATEFILE set) remain documented; pick them up after the asset track or interleave if a lab capture makes one of them easy.

## Order

Open the PR for one item, then start the next. Do not wait for merge. The reviewer will catch up. Item 15 stays first until its PRs land.
