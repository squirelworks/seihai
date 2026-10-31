# Salvage analysis: Matriark's GServer (1.39)

- **Binary:** `Developer Tools/# Graal Client Tools And Sources/GServer/GServer 1.39.rar.extracted/GServer 1.39/Matriark's GServer/gserver`
- **Analyzed (UTC):** 20261031T212013Z · Ghidra 12.2 · static only
- **Format:** ELF 32-bit LSB (x86), statically linked, stripped, ~1.4 MB

## Purpose (plain behaviour)

Community **Matriark** build of classic **gserver** (pack labelled GServer 1.39). Same product niche as `gserver-linux` / `gserver13` / `gserver_pe_GServersource`: game-world server process (accounts, levels, RC/NPC session paths). Static link + strip make symbols sparse; prefer strings and comparative notes against unstripped `GServersource` trees.

**Do not run** against public networks.

## Recommended placement

| Tag | `analysis-only` (ops notes) / compare with `setup-core` rewrite targets |
| --- | --- |
| Why | Alternate classic gserver binary; behaviour for clean-room servertools, not binary reuse. |
| Related | `gserver-linux/`, `gserver13/`, `gserver-win/`, `gserver_pe_GServersource/` |
| Not | Drop-in production binary; trainer/client scope |
| Licence | Legacy third-party — clean-room only |

## Ghidra stats (machine)

| Field | Value |
| --- | --- |
| Functions | ~3658 (internal ~3658) |
| Imports | 0 (static) |
| Exports / entries | ~2 |
| Interesting strings | ~407 |
| Decompiled (local only) | 20 — **not committed** |

## Artifacts (local)

`prelim/`, `functions.*`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
