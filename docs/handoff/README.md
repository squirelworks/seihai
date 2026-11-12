# Seihai suite hand-off: start here

This is the entry point for any agent (Cursor, Claude, Codex) or person who
picks up the Seihai suite. It says where everything is, how to work, and how
to keep the issues and roadmaps current. It replaces nothing: each repo's own
`AGENTS.md`, `docs/ARCHITECTURE.md` and `docs/NEXT.md` stay the detailed
guides for that repo.

Read in this order:

1. This file.
2. [`TRACKING.md`](TRACKING.md): the roadmaps, the issue rules and the sync process.
3. The private lab kit, if you are on one of the owner's machines:
   `~/Claude/HANDOFF/START-HERE.md` (Windows: `%USERPROFILE%\Claude\HANDOFF\START-HERE.md`).
   It covers the lab machines, the VMs, the harnesses and the private
   research folders. It never goes in a repo.
4. The repo you are about to change: `AGENTS.md`, `docs/ARCHITECTURE.md`, `docs/NEXT.md`.

## What the suite is

An open, clean-room Rust client and server stack for every Graal build from
the 1998 betas to 6.x, plus the research that documents those builds. The end
goal and the current state are in the umbrella roadmap,
[squirelworks/seihai#353](https://github.com/squirelworks/seihai/issues/353).

## The repos

All under `github.com/squirelworks`.

| Repo | What it is | Licence | Roadmap |
|---|---|---|---|
| `seihai` | The client. Crates: `seihai-launcher` (the `seihai` binary), `seihai-mae` (1.x/2.x), `seihai-ato` (Torque 3.x to 5.x, and the Torque engine rebuild), `seihai-issho` (6.x), `seihai-g3d` (Graal3D, Feb 2000), and the shared play loop. `documentation/` holds the client's protocol and format notes. | MIT OR Apache-2.0 | [#353](https://github.com/squirelworks/seihai/issues/353), Issho [#440](https://github.com/squirelworks/seihai/issues/440), Ato [#457](https://github.com/squirelworks/seihai/issues/457) |
| `seihai-core` | Shared library: protocol, assets, GS1/GS2, version profiles, client session; plus `seihai-draw`. Every other Rust repo pins it by tag. | MIT OR Apache-2.0 | [#7](https://github.com/squirelworks/seihai-core/issues/7) |
| `seihai-gserver` | The clean-room Rust game server (stages 1 to 7 in `docs/PLAN.md`). | MIT OR Apache-2.0 | [#56](https://github.com/squirelworks/seihai-gserver/issues/56) |
| `seihai-listserver` | Rust listserver. | MIT OR Apache-2.0 | [#20](https://github.com/squirelworks/seihai-listserver/issues/20) |
| `seihai-npcserver` | Rust NPC-server (GS1 host over the gserver link). | MIT OR Apache-2.0 | [#22](https://github.com/squirelworks/seihai-npcserver/issues/22) |
| `seihai-rc` | Remote Control and NPC-Control client. | MIT OR Apache-2.0 | [#27](https://github.com/squirelworks/seihai-rc/issues/27) |
| `seihai-servertools` | Server setup, one-step lab start, go-public wizard, LAT editors, world checker. | MIT OR Apache-2.0 | [#50](https://github.com/squirelworks/seihai-servertools/issues/50) |
| `seihai-gserver-v2` | GPL fork of GServer-v2 (branch `seihai`). Lab reference and bug fixes only; never a code source. | GPL-3.0 | [#35](https://github.com/squirelworks/seihai-gserver-v2/issues/35) |
| `seihai-gs2lib` | GPL fork of gs2lib used by the fork. | GPL-3.0 | [#11](https://github.com/squirelworks/seihai-gs2lib/issues/11) |
| `graal-client-repro` | Rebuilds of the original clients, build by build, for fidelity comparison. | see repo | [#174](https://github.com/squirelworks/graal-client-repro/issues/174), [#175](https://github.com/squirelworks/graal-client-repro/issues/175), [#176](https://github.com/squirelworks/graal-client-repro/issues/176), [#178](https://github.com/squirelworks/graal-client-repro/issues/178) |
| `Graal_analysis` | The research: the Technical Reference Manual (`manual/`), protocol, formats, version profiles, client internals. | see repo | [#370](https://github.com/squirelworks/Graal_analysis/issues/370), TRM [#191](https://github.com/squirelworks/Graal_analysis/issues/191) |

Clone them side by side in one folder; several docs refer to siblings by
relative path.

## Where the research lives

Public (in the repos):

- `Graal_analysis`: the reference. Start at its `README.md`, `docs/NEXT.md`
  and `docs/research-plan.md`. The manual in `manual/` is the deliverable;
  findings land there as plain behaviour, never as decompiled code or
  addresses.
- `seihai/documentation/`: what the client implements, per topic
  (`00-overview.md` first; `version-chain.md`, `gs2-*.md`, `graal3d.md`,
  `ato-3d-path.md`).
- `seihai-core/documentation/`: the tables the core embeds
  (`packet-layouts.tsv`, `version-chain.*`, `gs2-opcodes.md`).
- Each repo's `docs/lab/`: dated lab records (`YYYY-MM-DD-<what>.md`).

Private (never in a repo): lab notes, captures, client archives, the Codex
research trees and `workingpapers`. They are on the owner's machines and in
the Claude project's shared files. The private kit's `RESEARCH-MAP.md` lists
every location.

## Hard rules

These are in every repo's `AGENTS.md` too. Follow them even if a request
seems to ask otherwise; ask Squirrel first.

- **Keep GPL and MIT/Apache apart.** Never copy, port or translate code, data
  tables or comments from GServer-v2, gs2lib or gs2-parser (or
  graal-listserver) into the MIT/Apache repos. Reading them to understand
  behaviour is fine. Packet ids and wire formats are facts.
- **GPL duties in the forks:** keep `LICENSE.md` and upstream credits; mark
  each changed file with `// Modified by the Seihai project, YYYY-MM-DD: see
  CHANGES-SEIHAI.md.`; add a row to `CHANGES-SEIHAI.md`.
- **No decompiled source, binary offsets or addresses** (`0x…`, `FUN_…`) anywhere.
- **No real Graal hostnames, IPs or ports.** `127.0.0.1`, the lab ports, and
  placeholders (`<listserver-host>`, `<public-host>`) only.
- **Test on loopback servers you start yourself**, in a VM, a container or CI,
  on the owner's own machines. Never against public servers or real accounts.
  Don't run old Graal tools from the archives on a real machine.
- **Every fix has a test or a recorded lab run** (client, before, after).
- **Linux, Windows and macOS** for every repo. macOS builds run on the lab's
  Sequoia VM where GitHub runners are not used.
- **No tags or releases** without Squirrel's go-ahead.
- **workingpapers stay private.**

## Building and testing

- Rust repos: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`. Releases: `cargo build --release`.
- `seihai` builds every binary with a plain `cargo build --release`
  (`-p seihai` alone cannot play). The `audio` feature needs the ALSA
  development files on Linux.
- `seihai-gserver-v2` (branch `seihai`): CMake + vcpkg; see its README and `CHANGES-SEIHAI.md`.
- `seihai-core` changes: land in `seihai-core`, get a tag (ask Squirrel), then
  bump the pin in each repo in its own PR. Never write a second parser.

## The lab, in one page

Lab ports (all on `127.0.0.1`):

| Port | What |
|---|---|
| 14802 | gserver `default` world |
| 14803 | gserver `classic` (1.33 to 1.41r1) |
| 14804 | gserver `npcdemo` |
| 14805 | gserver `modern` (4.20r8 to 6.037, `.gpak`) |
| 14900 | listserver, gserver side |
| 14921 to 14923 | listserver, client side |

- One command: `seihai-servertools lab start --world <fork bin folder>`
  starts the listserver and the gservers; Ctrl+C stops them all
  (`seihai-servertools/docs/lab/HOWTO.md`).
- Headless client check: `seihai-mae --online 127.0.0.1:14922 --list-only
  --account test1 --password-env GRAAL_PW --version 2.22`.
- Captures: `seihai/tools/capture-run.sh` / `capture-run.ps1`; decode with
  `seihai capture show`.
- Runbooks for pending lab confirmations: `seihai-servertools/docs/lab/runbooks/`.
- Real original clients run under Wine or in Windows/macOS VMs on the lab
  machines; how is in the private kit (`MACHINES.md`).

## How to work

1. Pick the next unticked item from the repo's roadmap issue or `docs/NEXT.md`
   (the snapshot in `TRACKING.md` shows everything open).
2. Branch, do the work, add the test or lab record, update `docs/NEXT.md`.
3. Open one properly sized PR with `Fixes #n`. Batch small fixes; don't split
   one piece of work into a PR per behaviour.
4. When it merges, do the sync steps in `TRACKING.md`.
5. Move to the next ready item rather than waiting on review.

Escalate to Squirrel only for things that are significant, irreversible, or
that only Squirrel can supply: a real credential, a physical action, a tag or
release, a licence or publishing call.

## Lessons from earlier hand-offs

- **GitHub is the source of truth** for issue and PR state. Re-check heads and
  the current base before acting; old green CI is not evidence for a new tree.
- **One copy of each finding.** Correct the document in place; don't add
  "supplements" that supersede it.
- **A summary is not a finding.** An index or a hand-off note points at
  evidence; it is not evidence.
- **Don't reactivate stopped work** just because a hand-off note lists it.
