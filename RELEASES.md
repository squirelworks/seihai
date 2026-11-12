# Release points

When each repo is ready for a release, and how a release happens. Rows that say
"rechecked 2026-11-12" were checked that day against each repo's main and
releases page; "rechecked 2026-11-11" rows are from the day before. Update a row in the same
PR that changes it.

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
| `seihai` (client and `seihai-core`) | **v0.0.7** | `playerlogin`/`playerlogout` for the NPC-server's level-less NPCs (#416), the GS1→GS2 element and `in` forms (#414, #422), Ato's NPCs, offline levels and gmaps, and online gmaps (#417 to #420). | **Draft to be made from main after this PR** (notes: `docs/releases/v0.0.7.md`), waiting on Publish. Then seihai-npcserver moves to tag `v0.0.7` (seihai-npcserver#25). |
| `seihai` (client and `seihai-core`) | **v0.0.6** | Everything on main as of the rebuilt draft: `seihai-draw` (#350), 3-OS CI with fmt and strict clippy (#354, #381, #382), the GS1→GS2 front end (#376, #383, #384), the Torque engine scaffold and `seihai-issho` (#378, #379), listserver.graal.in as the only host (#389), `playerleaves` for server scripts (#390). | **Published 2026-11-11** (`35fb728`). The listserver, NPC-server, RC, gserver and servertools now pin `seihai-core` by tag `v0.0.6` (rechecked 2026-11-12). |
| `seihai` | v0.1.0 | Ato: server list and offline mode, and the post-login frame; Graal3D M1 (G3D-1, 2, 4, 6, 7, with #357). | Not reached (rechecked 2026-11-12): the server list (#393) and the post-login frame (#395) are in, and Ato now runs 4.x and 5.x (#394), walks online (#396), shows other players (#398, #405), follows level links (#399, #406) and plays sounds (#408 to #410). Offline mode is still under "Not in yet"; G3D-1 is open. |
| `seihai-listserver` | v0.2.0 (proposed; Squirrel has not set its version or scope) | From its `docs/NEXT.md`: the RC list (id 5) over the secure handshake (SEI00001), which `seihai-rc` has sent since 2026-11-01 while the listserver answers SEI00001 only on id 7; NEXT 7, a `seihai-rc` test that logs in through the real listserver; a lab run where a real RC lists a hidden (HQ level 0) server. | Not started (rechecked 2026-11-11). v0.1.0 was published 2026-11-04 with all four archives; since then only docs and workflow changes (#14 `skip_macos`, #15, #16). CI green on all three platforms; pins `seihai-core` v0.0.5; no issues. The RC-list item is unblocked; the other two live mostly in `seihai-rc`. |
| `seihai-npcserver` | v0.2.0 | Milestone 3: a sample world tested with real clients. | Blocked on `seihai-gserver` stage 5. **v0.1.1 published 2026-11-12** (`846f30b`, four archives): `playerswordpower`, `playershieldpower`, `playerhp`, `playerleaves` on `seihai-core` v0.0.6. `seihai-gserver` pins it (seihai-gserver#61). |
| `seihai-gserver` | v0.2.0 | Stage 4 (RC and NC endpoints for `seihai-rc`) and the rest of stage 5 (the NPC-server's messages back). | Not started. **v0.1.0 published 2026-11-12** (`5be1708`, four archives): stages 1 to 3, `.gpak` serving, anti-cheat, the listserver and NPC-server links (notes: seihai-gserver#60). Open from the stage 3 lab: real clients' hits, 0 hearts on login. |
| `seihai-rc` | **v0.1.0** | Stage 2, players, on the command line (#9–#13, #17), with stage 3 accounts (#18), the file link (#20) and the 162 keepalive (#21). | **Draft ready, waiting on Publish** (2026-11-12; seihai-rc#29). Squirrel moved list RCs to v0.2.0. Id 65 is never sent by any RC, so v0.2.0 lists RCs from the player list (an RC is a player with an empty level; seihai-rc#30). |
| `seihai-servertools` | v0.1.0 | Setup steps 1–6, checks and `check --json`, "Who may register", listserver limits and player capacity (#41), the Lab page and window polish; **the real-fork lab run** of the whole setup (NEXT, Step 4 lab run). | Not reached (rechecked 2026-11-12): everything but the real-fork lab run is in. It pins `seihai-core` and `seihai-draw` to tag `v0.0.6`. |
| `seihai-gserver-v2` (GPL fork) | **v4.0.0-seihai.1** | Already merged: 1.33–6.x login fixes, socket/disconnect/RC-hang/level-unload crash fixes, RC folder rights, account fields and flags, `.gpak`/`.gpake` serving (#20), gs2lib packet names (#28). Still to merge: the upstream beta4 `25396a1` sync (#30) and the lab world's modern folder for `.gpak` (#31). | **Published 2026-11-11** (`576c954`). Since then on branch `seihai`: the Graal 3D Torque UDP side (#39, #41), the modern lab world letting in 4.04r2 (#38) and a macOS CMake preset (#40) (rechecked 2026-11-12). |
| `seihai-gs2lib` (GPL fork) | none | It has no binary; `seihai-gserver-v2` pins it by commit and ships it inside its release. | No release planned (rechecked 2026-11-11). Branch `seihai` is at `417ef96`: upstream gs2lib `35056f8` (#7), upstream packet names primary with the old names as aliases (#8), README and change-log refreshes (#6, #9). No open issues or PRs; CI green on all three platforms. `seihai-gserver-v2` #30 pins `417ef96`. |
| `Graal_analysis` | **TRM v0.0.5** | `tools/manual/coverage.py` as a CI report, plus one prose-coverage PR per Part I–XI (#191). | **Published 2026-11-11** (`94623be`, rechecked 2026-11-12). The deliverable is the Technical Reference Manual (`manual/`); v0.0.3, v0.0.4 and v0.0.5 are published. The PDF builder is `tools/manual/build_manual.py` (#230); CI builds it on Linux and Windows. No tag without the owner's go-ahead. |
| `graal-client-repro` (formerly `graal-delphi`) | **v0.0.4.1**, a development checkpoint for all 74 clients | Linux and Windows zips for every client, plus the C++ host and DSO/VM checkpoint work merged since v0.0.4 (#167–171, #181–185). macOS deferred. | **Published 2026-11-11** (`b808d91`, rechecked 2026-11-12). Next: 0.0.5 (per-client fidelity backlog, #107 stress gate). Since v0.0.4.1, main has gained the 6.1.1.3 connector login scaffold (#200), the 6.11r1 Mac folder (#201), one folder per Worlds build (#199), the shared Worlds connector scaffold (#207) and the 1.35 one-byte player id (#205). |

There is no separate `seihai-core` repo: `seihai-core` is the crate in this
repo, and the `v0.0.x` tags here are its versions.

## After a Publish

- `seihai` v0.0.6 published: done, every server repo and `seihai-servertools`
  pins tag `v0.0.6`.
- `seihai` v0.0.7 published: `seihai-npcserver` moves to tag `v0.0.7` and adds the `playerlogin` test; the other repos move when they need it.
- `seihai-npcserver` v0.1.1 published: `seihai-gserver` moves its pin to v0.1.1 (seihai-gserver#61).
- `seihai-rc` v0.1.0 published: pin it by tag in `seihai-servertools`.
- Tick the row here and the roadmap item in #353.
