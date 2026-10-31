# Salvage analysis: remotecontrol (Custom / Agret)

- **Binary:** `Developer Tools/# Graal Client Tools And Sources/RC [Remote Control]/Custom RemoteControl.rar.extracted/Custom RemoteControl/remotecontrol.exe`
- **Analyzed (UTC):** 2026-10-31 · Ghidra 12.2 · static only
- **Format:** PE32 GUI (x86), **Borland Delphi**, ~1.0 MB

## Purpose (plain behaviour)

Community-modified **Graal Remote Control** (“Modified by Agret” version string). Classic Delphi VCL RC rather than the GTK RemoteControl3 line.

Observed behaviour from strings/imports:

- Window title **Graal Remote Control**; connects via Winsock (`connect`) to a chosen server from a **Graal Server List** UI (“Connecting to server …”, failure/disconnect messages).
- Tracks **Players:** count and **NPC-Server** connection state (“Connected to NPC-Server”).
- Config/registry: `Software\Graal`, `onlinestartlocal.graal`, flags such as `dontsavepassword`, icon toggles for player list / NPC list / local NPCs, label keys for players and NPC server.
- NPC tooling: attributes, script-from-NPC, flags-from-NPC, NPC level list, local NPCs unit, delete-NPC confirm dialog.
- Account/player edit surfaces (`UEditAccountPlayer`, `TPlayerStats`).
- Same general staff RC role as stock Delphi RC packs (`rc.rar` / `rc2.rar` lineage) — useful comparative sample for `seihai-rc` wire behaviour.

**Do not run** against public servers.

## Recommended placement

| Tag | `seihai-rc` |
| --- | --- |
| Why | Staff RC session client; protocol/UI behaviour for clean-room RC. |
| Also | Keep PE staged; `analysis-only` for raw dumps. |
| Not | `setup-core` / `lat-core` / trainer scope |
| Licence | Legacy Delphi client — notes only, no decompile paste |

## Ghidra stats (machine)

| Field | Value |
| --- | --- |
| Functions | ~3707 (internal ~3241) |
| Imports | ~471 (typical Delphi + WSOCK32/advapi/comctl) |
| Exports | PE entry stubs only (~3) |
| Interesting strings | ~159 |
| Decompiled (local only) | 30 — **not committed** |

## Artifacts (local)

`prelim/`, `functions.*`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
