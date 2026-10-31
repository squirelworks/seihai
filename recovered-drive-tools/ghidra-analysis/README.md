# Ghidra salvage analyses

Produced by `/workspace/tools/ghidra/analyze_tool.sh`.
Pipeline docs: `/workspace/tools/ghidra/README.md`.

Focus: purpose, CLI, protocols, config paths for rebuild/rehost into
`seihai-servertools` (clean-room MIT). Do **not** paste decompiled Delphi/GPL
or GServer-v2 sources into MIT crates — see `SALVAGE_PLACEMENT.md`.

## Completed

| Dir | Binary | Highlights |
| --- | --- | --- |
| `serverlist3/` | `Server Lister/slist/serverlist3` | ELF32 static **not stripped**. Classic listserver. Config keys `clientsport`/`serversport`/`playerlimit`; MySQL; argv = config path. Decompiles: `main`, `StartServer__Fv`, `parse__*`, `TSocket::*`. |
| `ListServer/` | UPX-unpacked `ListServer.exe` | Win32 Delphi PE + `WSOCK32`. Companion `servers.ini`. |
| `gserver-linux/` | `GServersource/gserver` | ELF32 (paired with **full C++ source** in same archive — prefer source). Decompiles: `main`, `TServerFrame::startServer`, `TServerPlayer::parse*`, `LoadNW`, DB account helpers. |
| `gserver-win/` | `GServersource/gserver.exe` | Win32 PE of same era gserver. |
| `gserver_pe_GServersource/` | same Win PE (alt batch path) | Duplicate/parallel run of `gserver.exe`. |
| `gserver13/` | later/related gserver build | See SUMMARY for identity. |
| `BytecodeViewer/` | GS2 `BytecodeViewer.exe` | Console PE; package also has compiler **source**. |
| `TextConvert/` | `GRAALtoNWconverter/TextConvert.exe` | Delphi PE — Graal→NW text/level conversion helper. |
| `gserverstat/` | `gserver_source/gserverstat` | ELF32 account/stats helper; not stripped. |
| `gserverstat2/` | `gserver_source/gserverstat2` | Near-twin of `gserverstat` (same era). |

Inventory + placement: [`../INVENTORY.md`](../INVENTORY.md), [`../SALVAGE_PLACEMENT.md`](../SALVAGE_PLACEMENT.md).

## Prefer source over RE when present

- `_inbox/extracted/GServersource/` — `gserver.cpp`, `npcs.cpp`, `levels.cpp`, makefiles, MySQL bits.
- `_extracted/listserver-nested/graal-listserver/` — C# listserver rewrite (`Program.cs`, `appsettings.json`).
- `_extracted/gs2-compiler/GS2 Compiler/BytecodeCompiler/` — C++ compiler sources.

## Skipped / blockers

| Item | Reason |
| --- | --- |
| `DClientSetup1198.zip` | Truncated download (`Unexpected end of archive` / bad CAB). |
| `Graal_Auto_Key`, `Graal_Digger` | Client trainers — **out of scope** for servertools. |
| `charactergenerator` / `GCD.exe` | Client character tools — low priority for servertools. |
| `scripteditor.exe` | VB5 GUI editor; OCX-heavy — defer unless GS2 tooling needs it. |
| More Drive files | Sibling may still be downloading (`_catalog.json`). Re-run helper as new ELFs/PEs appear. |

## Re-run

```bash
/workspace/tools/ghidra/analyze_tool.sh \
  /path/to/bin \
  /workspace/recovered-drive-tools/ghidra-analysis/<name> [max_decomp]
```
