# Release points

When each repo is ready for a release, and how a release happens. Checked on
2026-11-04 against every repo's main (`seihai` branch for the GPL forks), its
open PRs and the roadmap (#353); updated at 14:30 UTC the same day. Update a row in the same PR that changes it.

## How a release happens: drafts, then the owner presses Publish

1. When a repo reaches its point, someone bumps the version in `Cargo.toml` and
   writes the release notes in `docs/releases/v<version>.md`, in one normal PR:
   what is new, what it is for, what is known not to work. Generated lists of PR
   titles are not notes. Then they run that repo's **release** workflow by hand
   (Actions, release, Run workflow) with the version, e.g. `0.1.0`.
2. The workflow builds every target (Linux, Windows, macOS x86_64 and arm64),
   and creates a **draft** GitHub release `v<version>` pointing at that commit,
   with those notes and the archives attached. It stops with an error if the
   notes file is missing.
3. **Squirrel opens the repo's Releases page and presses Publish.** Only then is
   the tag made. Until then nothing is tagged and nothing is public.

Agents may run step 1 and 2. Nobody but the owner presses Publish, and nobody
pushes a release tag by hand. A pushed `v*` tag still works (it builds and
makes a draft), but the draft route is the normal one. Publishing makes the tag,
which runs the workflow once more. For a release that is already published, the
workflow only replaces its notes from `docs/releases/` and leaves the tag, its
commit and the files alone, so a notes fix is a PR and then one run.

**A release waiting on Squirrel never blocks other work.** PRs keep merging on
main while a draft waits. A draft points at a fixed commit, so later merges do not
change it. If a draft goes stale, delete it and run the workflow again.

## Where each repo stands

| Repo | Next release | What must be in | Status |
|---|---|---|---|
| `seihai` (client and `seihai-core`) | **v0.0.6** | `seihai-draw` on main (#350); 3-OS PR CI (#354, #358); Phase 0 doc fixes (#355, #356). | **Reached; draft being made** (release run started 2026-11-04 after #359 merged). Publishing it unblocks the `seihai-core` pin moves below. |
| `seihai` | v0.1.0 | Ato: server list and offline mode, and the post-login frame; Graal3D M1 (G3D-1, 2, 4, 6, 7, with #357). | Not reached. |
| `seihai-listserver` | **v0.1.0** | Roadmap Phase 4: lab run with the fork (#10), `seihai-gserver` registers (#10), real 2.x clients under Wine (#11). | **Reached**, except NEXT 7 (a `seihai-rc` test that logs in through the listserver). That test lives in `seihai-rc` and does not change this binary. Squirrel, 2026-11-04: the listserver releases on its own, without waiting for NEXT 7. **Published 2026-11-04** (seihai-listserver#12); hand-written notes from seihai-listserver#13. |
| `seihai-npcserver` | **v0.1.0** | Milestone 2 (#1–#5); `seihai-core` v0.0.5 (#7); link messages for sounds, `setshape`, `triggeraction` and rupees/darts/bombs (#8–#12). | **Published 2026-11-04** (seihai-npcserver#13; hand-written notes from #14). `seihai-gserver` moves its pin from rev `340bfa7` to the tag. The rest of the logged-only effects and milestone 3 go to v0.2.0. |
| `seihai-gserver` | v0.1.0 | Stage 2, the world, all of it: 2.1 online (#13), 2.2 levels (#14), 2.3 other players (#15), 2.4 warps (#18), 2.5 gmaps (#22), 2.7 files (#24), plus **2.6** flags/chests/items, **2.8** the 50 ms tick (#23, #25 open), **2.9** anti-cheat, **2.10** the real-client lab run. NPC-server pinned by tag. | Not reached: 2.6, 2.8, 2.9, 2.10 missing. |
| `seihai-rc` | v0.1.0 | Stage 2, players, on the command line: done (#9–#13) except **props set (76)** and **list RCs (65)**. | Not reached: two items. |
| `seihai-servertools` | v0.1.0 | Setup steps 1–6 (#2–#7), checks and `check --json` (#12, #17, #20), "Who may register" (#15), #21 (listserver limits); **the real-fork lab run** of the whole setup (NEXT, Step 4 lab run). | Not reached: #21 open, the lab run not done. Moving `seihai-core`/`seihai-draw` off rev `72bb8d2` to `v0.0.6` is wanted but not required, so an unpublished v0.0.6 never holds this up. |
| `seihai-gserver-v2` (GPL fork) | **v4.0.0-seihai.1** | The fork's fixes (#1–#10) and the two RC fixes (#11, #12); 4-target CI green. | **Reached; draft being made** (#11, #12 and the draft workflow #13 merged; release run started 2026-11-04), with the GPL source archive. Version name confirmed by Squirrel, 2026-11-04. Phase 5 needs this release for its "real `gs2emu` from a fork release" lab run. |
| `seihai-gs2lib` (GPL fork) | none | It has no binary; `seihai-gserver-v2` pins it by commit and ships it inside its release. | No release planned. |
| `Graal_analysis` | **v0.0.1** and **v0.0.2** | The book PDFs, already in the tree: `release/0.0.1/` (#187, #188, commit `c1dd727`) and `book/0.0.2/` (#189, commit `aaccd17`). | **Both published 2026-11-04** (v0.0.1 at `c1dd727`, v0.0.2 at `aaccd17`; Graal_analysis#226), with hand-written notes from Graal_analysis#229. Next: Manual 1.0 when the TRM is done (Graal_analysis #219 and #222 merged); its PDF is built in CI by `tools/manual/buildpdf.sh` (Graal_analysis #227). |
| `graal-delphi` | one release per version of a rebuilt build, tag = folder and version (e.g. `graal-2.05r2-GNW03081-v0.0.1`) | The build's PR merged and CI green on Linux, Windows and macOS, with hand-written notes. Workflow from graal-delphi #1, versioned tags and macOS from #6. | 2.05r2 v0.0.1 reached (#6 merged). Same draft route: a manual run with the folder and version (or a pushed `graal-*-v*` tag) makes a draft; the owner presses Publish. 2.22 (#8) and 2.17.1 (#7) follow when they merge. |

There is no separate `seihai-core` repo: `seihai-core` is the crate in this
repo, and the `v0.0.x` tags here are its versions.

## After a Publish

- `seihai` v0.0.6 published: move `seihai-servertools` (rev `72bb8d2`) and the
  server repos (`v0.0.5`) to tag `v0.0.6`, one PR each.
- `seihai-npcserver` v0.1.0 published: pin it by tag in `seihai-gserver` (PR open).
- `seihai-rc` v0.1.0 published: pin it by tag in `seihai-servertools`.
- Tick the row here and the roadmap item in #353.
