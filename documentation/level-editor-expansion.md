# Level editor expansion notes

Seihai already has a first CLI slice of a level editor (open, list, paint, save) and tracking notes that live in the Graal_analysis repository under `tools/level-editor/`. This file records concrete next steps and the archival test assets that are now available.

## Existing foundation

- CLI editor under `tools/level-editor/` (open / list / paint / save), and a window on it (`level-editor gui`: tile palette, layers 0 and 1, `GOBJSET01` stamps, link / sign / chest / NPC markers, a `.gmap` grid, save).
- Tracking checklist and 2.22 reference: Graal_analysis `tools/level-editor/seihai-tracking.md`, `222-reference.md`, `version-deltas.md`.
- Binary and text level loaders already in `client/src/assets/level.rs` (GLEVNW01, GR-V1, Z3-V1).
- Tile-type tables (text tiledefs and `arrays.dat`).

## Archival test data

A full experimental walker project (Ember / Avelan / tileset variants) was decommissioned 2026-10-28 and archived:

- Google Drive: `/[Grok/[Projects/Graal Enigma Playerworld.zip`
- File ID: `1Zqyp3ECN8dP-dk6jBjqfRQ728_PZzbgY`
- Link: https://drive.google.com/file/d/1Zqyp3ECN8dP-dk6jBjqfRQ728_PZzbgY/view

Useful pieces for editor work:

- **Avelan** — clean 10×10 GLEVNW01 world, 100 levels, start `avg-05.nw` at 10.5, 56.5. Ideal fixture for round-trip open/edit/save tests.
- **types.bin** — 4096-byte pics1 type-0 mask (body blocked by 20 and 22; throw blocked only by 22). Drop-in validation for tiledef loaders.
- **Emberhold tilesets** (`em_tiles` … `em_tiles4.png`) — 2048×512 sheets whose art already follows the mask. Useful for visual editor previews.
- Stitched overworld maps and the original Enigma start-area renders.

A short archival write-up also lives in Graal_analysis (PR adding `archive/ember-avelan-experimental.md`).

## Suggested next slices

1. **Round-trip fixtures**  
   Load `avg-05.nw`, paint a few tiles, save, reload, and assert tile identity. Repeat for a small GR-V1 sample. Catches encoder/decoder drift early.

2. **Mask-aware painting**  
   When the user paints a tile, optionally show or enforce the type-0 mask (wall vs floor). The Emberhold sheets are a ready visual reference.

3. **Link and sign editing**  
   The CLI already lists records. Add edit/create for `LINK` rectangles and `SIGN` blocks, using the same arrival rules (`-1` on the free axis, `playerx`/`playery` offsets) already documented in level.rs.

4. **Stamp / object support**  
   The 2×2 object tables already parsed from `arrays.dat` (cut/lift) are the natural next paint mode after single tiles.

5. **Visual preview**  
   Re-use the existing software renderer (or a thin export to PPM/PNG) so the CLI can dump a board image after edits. The archived stitched maps show the expected look.

6. **Version-gated defaults**  
   Warp-field defaults, save-format-by-extension, and the 9×9 `HEIGHTS` block are already catalogued in Graal_analysis `tools/level-editor/`. Expose the ones the current target version actually writes.

## Boundary

These notes are implementation suggestions only. The Seihai development project decides priority and scope. No new protocol or decompile material is introduced here.

## Related

- Graal_analysis `tools/level-editor/`
- `documentation/level-editor.md` (existing Seihai notes)
- `documentation/assets.md` (link offsets, tile id mapping)
