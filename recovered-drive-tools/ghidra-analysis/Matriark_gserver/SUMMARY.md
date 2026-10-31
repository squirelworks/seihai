# Salvage analysis: Matriark's GServer (1.39)

- **Binary (ELF):** `Developer Tools/# Graal Client Tools And Sources/GServer/GServer 1.39.rar.extracted/GServer 1.39/Matriark's GServer/gserver`
- **Also in pack:** Win32 `GServer.exe`, `dbsetup.exe`, `GStats.exe`, `gserverstats.pl` / `gserverstats2.pl`, `wingserver.zip`, short bugs note
- **Analyzed (UTC):** 20261031T212013Z · Ghidra 12.2 · static only
- **Format:** ELF 32-bit LSB (x86), **statically linked, stripped**, ~1.4 MB (hash differs from unstripped `GServersource/gserver`)

## Purpose (plain behaviour)

Community **Matriark** build of classic **gserver** (pack labelled 1.39). Same product niche as `gserver-linux` / `gserver13` / Win PE twins: world daemon with MySQL account/profile tables, guilds path, RC/NPC session surfaces, and HTML-ish leaderboard queries (kills/deaths/onlinetime/alignment). Static link + strip hide symbols; strings show account load/save failures, adminlevel lookups, decompress errors, and `gserver.db` naming.

Pack-local bugs note (behaviour only): head/shield/sword path checks disagree with send paths under `/levels`; guild nickname clash needs a leading `*`; possible server-flag/account overflow (unspecified). Prefer **`GServersource` C++ trees** over deep decompile of this stripped image.

**Do not run** against public networks.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `analysis-only` |
| --- | --- |
| Why | Alternate classic gserver binary; clean-room protocol/ops notes only. Prefer paired sources under `_inbox/extracted/GServersource/` / `gserver_source/`. |
| Related | `gserver-linux/`, `gserver13/`, `gserver-win/`, `gserver_pe_GServersource/` |
| Not | `setup-core` / `lat-core` / `seihai` / `seihai-rc` as a drop-in binary |
| Licence | Legacy third-party — clean-room only |

## Ghidra stats (machine)

| Field | Value |
| --- | --- |
| Functions | ~3658 (internal ~3658) |
| Imports | 0 (static) |
| Exports / entries | ~2 |
| Interesting strings | ~407 |
| Decompiled (local only) | 20 — **not committed** |

## Prefer source / unstripped twins

- `_inbox/extracted/GServersource/` (`gserver.cpp`, `npcs.cpp`, `levels.cpp`, …)
- `_inbox/extracted/gserver_source/` (`gserver13`, stats tools)
- Do not deep-decompile this stripped static ELF further for salvage docs.

## Artifacts (local)

`prelim/`, `functions.*`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
