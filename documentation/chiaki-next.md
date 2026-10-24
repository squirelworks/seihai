# Next work for Chiaki

Current queue after items 1–11 and 16. Open the PR for one item, then start the next. Do not wait for merge.

## Immediate

### Item 12 — Remaining fields (ids 24, 46, 178)

Seihai #39 holds the implementation (rebased). Once it is mergeable, it lands. Notes are already in Graal_analysis batch g.

### Item 15 — Differential-testing divergences

Running in the private lab. One PR per divergence:
- Rectangle level links (Seihai walks through them; real client sends the warp).
- Login player props, file update (34), process list (44).
- 2.22 position flags (`gr.x` / `gr.y`).
- Early id 46 on 2.31.
- Asset file requests, walk update fields, walk speed (~10 vs 7.8 tiles/s), frame bundling.

Each fix is checked against a lab capture first. If the capture does not show the exact bytes, skip with a note.

## After item 15

### Item 13 — Early builds

Skipped: no real 2.00-era listserver reply has turned up. Leave open.

### Item 14 — Public-server checks

Blocked from the build box (ports 14921–14923 time out). Checklist is in Seihai #38. Needs a network that can reach those ports. Password stays in the environment.

### Level editor

Notes are in Graal_analysis `tools/level-editor/` (item 16). Seihai tracking checklist is ready. No implementation started. When ready, open a Seihai PR that copies the checklist into `documentation/` and sketches the tool under `tools/level-editor`.

## Order

Open the PR for one item, then start the next. Do not wait for merge. The reviewer will catch up.
