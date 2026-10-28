# Legacy Classic Scripting Corpus

A prior analysis project produced a cleaned, catalogued, and documented extract of Classic Graal NPC scripts. The full archive was packaged on decommissioning (2026-10-28) as **Graal Scripting.zip** and placed on Google Drive under `/[Grok/[Projects/`.

Direct link: https://drive.google.com/file/d/1WDbffUEHO-rwa6FcVSXYG7KfSr1SKxVu/view

## What the archive contains

- **Grokd extract** — 1,520 unique script bodies from 6,275 placements, body-hash catalog (`CATALOG.tsv`), packs per system.
- **Phoenix documented copies** — same scripts with headers covering classification, events, functions, `toweapons` names, `client.*` / `save[]` usage, timeout rates, and gotchas.
- **Ash architecture docs** — Classic client NPC runtime model (events, `timeout` scheduler, weapon-bar persistence via `toweapons`, image ID bands, authority notes) plus per-pack blueprints, flowcharts, algorithms, and pseudocode.
- Original pack folders (larency HUD/inventory/shop, Phoenix quests/farm/baddies, Meh weapons/smartbaddy, Ultimate NPC Pack minigames, standalones).

Classification highlights (~1,519 scripts):

| Class | Count | Typical home |
|-------|-------|--------------|
| quest / dialogue | 516 | Phoenix |
| level prop / trigger | 390 | UNP + Phoenix |
| farm / planting | 136 | Phoenix |
| player weapon / projectile | 135 | Meh + UNP |
| shop / merchant | 111 | UNP + larency |
| minigame | 59 | UNP + Phoenix |
| enemy / baddy AI | 44 | Meh + Phoenix |
| inventory / equipment UI | 37 | larency |
| HUD / stats overlay | 20 | larency |

Dominant tags: tricxta (HUD/inventory), Ace2896 (weapons + smartbaddy), Agret (UNP).

## Relevance to Seihaiscript

The current engine in `client/src/scripting/engine/` already implements a practical GS1 subset with version gating, timeout scheduling, and host effects. The archive supplies concrete real-world patterns for the open items listed in `documentation/seihaiscript.md` and the roadmap (layer 4):

- **Image ID management** — Classic HUDs and shops fight over the ~300+ band; the Ash runtime notes document the bands and collision patterns.
- **`toweapons` / weapon-bar persistence** and `weaponfired` input — larency inventory and shop scripts are the clearest examples.
- **`board[]` / `tiles[]` reads** — many props and triggers depend on them; level data is already loaded.
- **Timeout-driven AI and growth** — smartbaddy, farm scripts, and minigames show typical rates (0.05 for AI/projectiles, 0.1–0.2 for menus, 0.5–1.0 for idle).
- **Client-authority patterns** — `client.*`, `save[]`, and local-only state are heavily used; useful for deciding what the offline host should simulate.

Suggested next steps for the engine (for review by the Seihai development project):

1. Wire `board`/`tiles` reads to the loaded level.
2. Improve `showimg`/`hideimg`/`setimg` with reserved ID bands or a simple manager.
3. Add a basic weapon-bar / hotbar host path so `toweapons` and `weaponfired` can drive inventory UIs offline.
4. Use the documented larency and Phoenix scripts as a regression corpus.

See also the parallel notes in the Graal_analysis repository (`npc-library/archival-scripting-notes.md`).

## Boundary

Clean-room. No decompiled client code. The archive is reference material for expanding the host effects and command surface; it is not intended to be dropped in as runnable code.
