# Next work for Chiaki — GANI expansion and follow-on

Asset track (items 17–20) is merged. Default set is in `assets/default/` and wired as the built-in fallback. Licence confirmed as MIT OR Apache-2.0.

Open the PR for one item, then start the next. Do not wait for merge.

## Immediate

### Item 22 — Expand the default GANIs

The default set currently has only `idle.gani`, `walk.gani`, and `swim.gani`. The reference notes in `documentation/asset-reference.md` already measured the stock layouts.

- Add `sword.gani` (5 frames, sound cue after frame 0, `SETBACKTO idle`).
- Add `sit.gani` (1 frame).
- Add a simple hurt pose (or a short `hurt.gani`) that the hit path can play.
- Wire the S key to play the sword animation and a hit to play the hurt pose.
- Keep the existing sound cues (`steps2.wav`, `water.wav`); add a short `sword.wav` if the generator can synthesise one.
- Output: the new GANI files under `assets/default/gani/`, an update to `tools/gen_default_assets.py` so they regenerate, and a short note in `documentation/default-assets.md`. Tests that the parser reads them and the offline session steps them.

### Item 23 — Script-to-GANI

The script engine has stubs for `setimg`, `showimg`, and `hideimg`. The offline session already has the hooks.

- Make `setimg` change the drawn sprite (or the NPC's GANI) so a script can switch an NPC between idle and walk.
- Make `showimg` / `hideimg` toggle the NPC's visible flag and have the renderer honour it.
- Cover at least one real NPC script from the library that uses these commands (smoke test, ignored if the script is not in the tree).
- Output: the wiring in the offline session and a test that a scripted `setimg` changes what is drawn.

### Item 24 — Level editor (first slice)

Notes are in Graal_analysis `tools/level-editor/` (2.22 reference, version deltas, tracking checklist).

- Copy the checklist into Seihai `documentation/level-editor.md`.
- Start a CLI under `tools/level-editor/` that can open a `.nw`, list tiles, paint a rectangle of one tile id, and save.
- Keep the version selector in mind (2.22 first). No GUI yet.
- Output: the CLI plus a note. The client parsers are already there.

## After that (pick up when ready)

- Combat leftovers: knockback on a hit, the packet-40 field, bomb/arrow/baddy damage. Needs a lab capture or a careful reading of the existing notes.
- GIF/MNG decoder (optional). The corpora have ~30k GIFs and ~2k MNGs; Seihai has no decoder. A simple GIF reader would let the client load heads and swords that ship as `.gif` in older worlds.
- Item 13 — Early builds: a real 2.00-era listserver reply (still unobserved).
- Item 14 — Public-server checks: blocked from the build box. Checklist is in `documentation/public-checks.md`. Needs a network that can reach listserver.graal.in:14921–14923. Port 14923 is the legacy port for pre-2.17 clients.

## Order

Open the PR for one item, then start the next. Do not wait for merge. The reviewer will catch up.
