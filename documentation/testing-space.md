# Testing space

Date: 2026-10-28

Seihai is expanding into systematic differential testing against real clients. This note records the current state and the natural next increments so the testing work has a single place to grow.

## Current differential-testing surface

- Capture-to-frame-sequence decoder (`difftest trace`)
- Differ with ranked report (`difftest diff`)
- Private-namespace scenario runner (`tools/difftest/lab.sh`)
- First results: 2.22 and 2.31, login, gmap crossing, chat, door/warp
- Ranked divergences documented in `documentation/difftest.md`

## Natural next increments

1. **CI check** — Turn the existing public frames into a CI job that fails on unexpected drift. Even a subset of the sixty-odd real frames would lock the parity already achieved.
2. **2.17 client** — The real 2.17 launcher needs its own UI steps. Document the exact sequence and add it to the lab runner.
3. **Server-side warp scenario** — The roadmap lists it as not yet covered. A short scripted scenario that forces a `PLO_SERVERWARP` and checks the reconnect would close the gap.
4. **Gmap remaining fields** — 185 board heights, neighbour NPCs/links/chests drawn across the seam, and a `35` with a warm cache. (Cached-level crossings are settled: `protocol.md` "Gmaps". `gr.x`/`gr.y`: `gr-flags.md`.) The analysis notes already describe the expected behaviour; the testing space needs the corresponding lab cases.
5. **Public-server game checks** — List checks are done. The remaining read-only game-server checks are listed in `documentation/public-checks.md`. Adding them as tracked scenarios keeps the approval gate visible.

## Relationship to the analysis archive

The Graal_analysis repository and the Drive zip (`Graal Client Analysis.zip`) hold the cleaned observations and the original passes that produced the expected behaviours. Differential testing is the place those expectations are checked against live clients. New divergences found here should be recorded in `documentation/difftest.md` and, when they reveal a missing rule, a short cleaned note can be added to Graal_analysis.

## Boundary

Lab captures stay private. Nothing in this note or the testing tools commits raw pcaps or credentials.
