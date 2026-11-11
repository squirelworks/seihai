# Release points

When each repo is ready for a release, and how a release happens. Rows that say
"rechecked 2026-11-11" were checked that day against each repo's main; the
others are from 2026-11-04. Update a row in the same
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
| `seihai` (client and `seihai-core`) | **v0.0.6** | Everything on main as of the rebuilt draft: `seihai-draw` (#350), 3-OS CI with fmt and strict clippy (#354, #381, #382), the GS1→GS2 front end (#376, #383, #384), the Torque engine scaffold and `seihai-issho` (#378, #379), listserver.graal.in as the only host (#389), `playerleaves` for server scripts (#390). | **Draft rebuilt on current main 2026-11-11; waiting on Squirrel's Publish.** Publishing it unblocks the `seihai-core` pin moves below. |
| `seihai` | v0.1.0 | Ato: server list and offline mode, and the post-login frame; Graal3D M1 (G3D-1, 2, 4, 6, 7, with #357). | Not reached (rechecked 2026-11-11): Ato still lists the server list, offline mode and the post-login frame under "Not in yet"; G3D-1 is open. |
| `seihai-listserver` | v0.2.0 (proposed; Squirrel has not set its version or scope) | From its `docs/NEXT.md`: the RC list (id 5) over the secure handshake (SEI00001), which `seihai-rc` has sent since 2026-11-01 while the listserver answers SEI00001 only on id 7; NEXT 7, a `seihai-rc` test that logs in through the real listserver; a lab run where a real RC lists a hidden (HQ level 0) server. | Not started (rechecked 2026-11-11). v0.1.0 was published 2026-11-04 with all four archives; since then only docs and workflow changes (#14 `skip_macos`, #15, #16). CI green on all three platforms; pins `seihai-core` v0.0.5; no issues. The RC-list item is unblocked; the other two live mostly in `seihai-rc`. |
| `seihai-npcserver` | **v0.1.1** | What main gained since v0.1.0: `playerswordpower` and `playershieldpower` (SET_PLAYER_PROPS props 8 and 9, keeping the image), `playerhp` (prop 2), the release workflow's `skip_macos` switch, docs and `PROTOCOL.md` synced with `seihai-gserver` (#18, #19). No protocol change; link version 1. | **Ready to cut** (rechecked 2026-11-11): CI green, no open PRs or issues. v0.1.0 was published 2026-11-04 for Linux, Windows and macOS; `seihai-gserver` pins it and moves to v0.1.1 afterwards. Next, v0.2.0 = milestone 3 (a sample world with real clients), blocked on `seihai-gserver` stage 5. `playerleaves` needs `seihai-core` with #390 (v0.0.6). |
| `seihai-gserver` | **v0.1.0** | Stage 2, the world (2.1 to 2.10). | **Reached 2026-11-05** (rechecked 2026-11-11 at `0272f65`): 2.1 #13, 2.2 #14, 2.3 #15, 2.4 #18, 2.5 #22, 2.6 #29 #30, 2.7 #24, 2.8 #23 #25, 2.9 #31, 2.10 lab #34; NPC-server pinned by tag v0.1.0 (#32). Not tagged: the release workflow (#28) makes a draft once the tag is pushed on Squirrel's go-ahead. Stage 3, `.gpak` serving (#47) and packet-id names (#46) are on main too, so the tag would include them. |
| `seihai-rc` | v0.1.0 | Stage 2, players, on the command line (#9–#13, #17). Props set (76) is done (#17); only **list RCs (65)** is missing. Also: bump `Cargo.toml` from 0.0.1 and write `docs/releases/v0.1.0.md`. | Not reached, one item (rechecked 2026-11-11). The 2.22 RC never sends 65, so it needs a capture from a newer RC; whether to drop it from v0.1.0 is Squirrel's call. Stage 3 accounts (#18), the file-link widths (#20) and the 162 keepalive (#21) are in too. |
| `seihai-servertools` | v0.1.0 | Setup steps 1–6, checks and `check --json`, "Who may register", listserver limits and player capacity (#41), the Lab page and window polish; **the real-fork lab run** of the whole setup (NEXT, Step 4 lab run). | Not reached (rechecked 2026-11-11 at servertools `2730ed5`): everything but the real-fork lab run is in. It pins `seihai-core`/`seihai-draw` to rev `5195744`; moving to `v0.0.6` is wanted but not required. |
| `seihai-gserver-v2` (GPL fork) | **v4.0.0-seihai.1** | Already merged: 1.33–6.x login fixes, socket/disconnect/RC-hang/level-unload crash fixes, RC folder rights, account fields and flags, `.gpak`/`.gpake` serving (#20), gs2lib packet names (#28). Still to merge: the upstream beta4 `25396a1` sync (#30) and the lab world's modern folder for `.gpak` (#31). | **Draft stale** (rechecked 2026-11-11): built 2026-11-04 from `da09752`. When #30 and #31 merge, the release workflow is re-run, then Squirrel presses Publish. Linux and Windows required, macOS best effort. Version name confirmed by Squirrel, 2026-11-04. |
| `seihai-gs2lib` (GPL fork) | none | It has no binary; `seihai-gserver-v2` pins it by commit and ships it inside its release. | No release planned (rechecked 2026-11-11). Branch `seihai` is at `417ef96`: upstream gs2lib `35056f8` (#7), upstream packet names primary with the old names as aliases (#8), README and change-log refreshes (#6, #9). No open issues or PRs; CI green on all three platforms. `seihai-gserver-v2` #30 pins `417ef96`. |
| `Graal_analysis` | **TRM v0.0.5** | `tools/manual/coverage.py` as a CI report, plus one prose-coverage PR per Part I–XI (#191). | Not started (rechecked 2026-11-11 at `adc128e`). The deliverable is the Technical Reference Manual (`manual/`): v0.0.3 (2026-11-04) and v0.0.4 (2026-11-05) are published; tags v0.0.1 to v0.0.4 exist. The PDF builder is `tools/manual/build_manual.py` (#230); CI builds it on Linux and Windows. No tag without the owner's go-ahead. |
| `graal-client-repro` (formerly `graal-delphi`) | **v0.0.4.1**, a development checkpoint for all 74 clients | Linux and Windows zips for every client, plus the C++ host and DSO/VM checkpoint work merged since v0.0.4 (#167–171, #181–185). macOS deferred. | **Blocked** (rechecked 2026-11-11): release-all failed three times on the Worlds player's NuGet restore (the release VERSION env var leaked into MSBuild). The fix is #189; then Squirrel re-runs release-all. v0.0.4 (2026-11-05, 71 clients) is the last published. Next: 0.0.5 (per-client fidelity backlog, #107 stress gate). |

There is no separate `seihai-core` repo: `seihai-core` is the crate in this
repo, and the `v0.0.x` tags here are its versions.

## After a Publish

- `seihai` v0.0.6 published: move `seihai-servertools` (rev `5195744`) and the
  server repos (`v0.0.5`) to tag `v0.0.6`, one PR each.
- `seihai-npcserver` v0.1.0 published: pinned by tag in `seihai-gserver` (seihai-gserver#32). After v0.1.1, move that pin too.
- `seihai-rc` v0.1.0 published: pin it by tag in `seihai-servertools`.
- Tick the row here and the roadmap item in #353.
