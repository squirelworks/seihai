# Salvage analysis: RemoteControl (classic `rc.rar`)

- **Binary:** `Developer Tools/# Graal Client Tools And Sources/RC [Remote Control]/rc.rar.extracted/RemoteControl.exe`
- **Analyzed (UTC):** 2026-10-31 · Ghidra 12.2 · static only
- **Format:** PE32 GUI (x86), Borland Delphi, ~888 KB

## Purpose (plain behaviour)

Stock **Graal Remote Control** from the classic `rc.rar` line (sibling of `rc2.rar` / Custom Agret builds).

Observed behaviour:

- Title **Graal Remote Control** / **Graal RemoteControl**; **Graal Server List** connect UI (“Connecting to server …”, disconnect / cannot-connect messages).
- Registry **`Software\Graal`**, `onlinestartlocal.graal`, password fields.
- NPC tooling: attributes, script/flags from NPC, NPC level list, local NPCs, delete-NPC confirm; “Connected to NPC-Server”; player count labels.
- Account/player edit units (`UEditAccountPlayer`, `TPlayerStats`); Winsock `connect`; zlib deflate embedded.

Comparative baseline for `remotecontrol_custom/` (Agret) and `RemoteControl-NW/`.

**Do not run** against public hosts.

## Recommended placement

| Tag | `seihai-rc` |
| --- | --- |
| Why | Canonical Delphi RC client for clean-room staff-session behaviour. |
| Staging | PE/dumps remain `analysis-only` artifacts |
| Not | `setup-core`, trainers |

## Ghidra stats (machine)

| Field | Value |
| --- | --- |
| Functions | ~3623 (internal ~3163) |
| Imports | ~466 |
| Interesting strings | ~137 |
| Decompiled (local only) | 20 — **not committed** |

## Artifacts (local)

`prelim/`, `functions.*`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
