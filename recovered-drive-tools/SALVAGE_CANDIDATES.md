# Salvage candidates → seihai-servertools (clean-room only)

Short list after the 2026-10-31 Ghidra follow-up. **Do not** commit recovered binaries, decompiled C, or GPL/Delphi text into the MIT clone.

## Clear candidates

**None** for a new `setup-core` / `lat-core` feature in this batch.

| Tool | Why not a clear rewrite |
| --- | --- |
| `gserverstat` / `gserverstat2` | Public HTML account leaderboards; gserver-era account format. Belongs with seihai-gserver / analysis, not lab wizard cores. |
| GCD / GChar | Client character designers → `seihai` client if anything. |
| Graal Auto Key / Graal Digger | Client trainers — out of scope. |
| DClient `setup.exe` | Incomplete VB6 client installer; bad archive. |
| ListServer / classic gserver ELFs | Already marked `analysis-only`; prefer existing C# listserver notes / gserver source trees over RE paste. |

## Optional (weak) behaviour note only

**Lab status files** (from `gserverstats.pl`, not from reimplementing the HTML ELFs):

- Classic ops wrote `playercount.txt`, `accountcount.txt`, `serveruptime.txt` under the gserver path and checked whether a `gserver` process was alive.
- `setup-core` **already** implements wizard lights from **console lines** (`Listening on`, listserver connect). A future optional reader for those three text files could enrich a lab status page — only if a lab profile actually emits them.
- If pursued: invent loopback fixtures; do **not** copy the Perl or embed real host paths from the dump (`/home/gentoox/gserver`, server display names, etc.).

## Process reminder

Promote only when behaviour is understood from notes/tests, target crate is clear, and no raw/GPL text would land in the clone (`SALVAGE_PLACEMENT.md` §C).
