# Salvage analysis: NPCControl

- **Binary:** `Developer Tools/# Graal Client Tools And Sources/NC [NPC Control]/nc7132001 [V2].rar.extracted/nc7132001/NPCControl.exe`
- **Same-size twin:** `nc7132001.exe` in the same pack / `nc7132001.rar`
- **Analyzed (UTC):** 2026-10-31 · Ghidra 12.2 · static only
- **Format:** PE32 GUI (x86), **Borland Delphi**, ~471 KB

## Purpose (plain behaviour)

Classic **Graal NPC Control** staff client for talking to an NPC server / world control port (banner string **GServer2 NPC Control 2001/07/13** / **Graal NPC Control**).

Observed behaviour:

- Reads **`controlconfig.txt`** with at least `servername`, `serverport`, and `password` (also mirrored under `Software\Graal` registry).
- UI title **NPC Control** / **Graal NPC Control**; connection status “Connected at port …”, failures “Cannot connect to the server!”, “Decompression error!”.
- Manages **NPCs** and **weapons**: delete confirms, “Script from Class”, “Script of weapon …”, local-NPC unit (`ULocalNPCs`).
- Opens RTF/text script files; references `onlinestartlocal.graal` and sample level names (lab-only — ignore any embedded world names for public use).
- Ships zlib deflate strings (compressed wire or packed payloads) — note behaviour only.

Sibling pack also includes `RemoteControl-NW.exe` (see that SUMMARY) for the RC side of the same era.

**Do not run** against public hosts.

## Recommended placement

| Tag | `seihai-rc` |
| --- | --- |
| Why | Live NC session / NPC+weapon script admin over the wire. |
| Also | PE stays staged (`analysis-only` dumps). |
| Not | `setup-core`, `lat-core`, client trainers |
| Licence | Legacy Delphi — clean-room notes only |

## Ghidra stats (machine)

| Field | Value |
| --- | --- |
| Functions | ~2428 (internal ~2052) |
| Imports | ~394 |
| Interesting strings | ~102 |
| Decompiled (local only) | 30 — **not committed** |

## Artifacts (local)

`prelim/`, `functions.*`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
