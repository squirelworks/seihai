# Ghidra salvage analyses

Produced by `/workspace/tools/ghidra/analyze_tool.sh`.  
Pipeline docs: `/workspace/tools/ghidra/README.md`.  
Placement: [`../SALVAGE_PLACEMENT.md`](../SALVAGE_PLACEMENT.md) · inventory: [`../INVENTORY.md`](../INVENTORY.md).

Focus: plain behaviour, CLI, protocols, config paths. Do **not** paste decompiled Delphi/GPL or GServer-v2 sources into MIT crates. Committed `SUMMARY.md` files must stay Seihai-safe (no Drive ids/URLs, no `FUN_*`, no binary offsets).

## Completed (corpus roll-up)

| Dir | Binary | Placement | Highlights |
| --- | --- | --- | --- |
| `RemoteControl3/` | GTK `RemoteControl3.exe` | `seihai-rc` | Server list, NPC-server connect, player/NPC admin UI |
| `remotecontrol_custom/` | Delphi Agret `remotecontrol.exe` | `seihai-rc` | Custom RC; registry `Software\Graal` |
| `RemoteControl-NW/` | NW `RemoteControl-NW.exe` | `seihai-rc` | New World–era Delphi RC |
| `RemoteControl_classic/` | `rc.rar` `RemoteControl.exe` | `seihai-rc` | Stock Delphi RC baseline |
| `NPCControl/` | `NPCControl.exe` | `seihai-rc` | `controlconfig.txt`; NPC/weapon scripts |
| `serverlist3/` | ELF `slist/serverlist3` | `analysis-only` | `clientsport` / `serversport` / `playerlimit` |
| `ListServer/` | UPX-unpacked `ListServer.exe` | `analysis-only` | Win32 Delphi + WSOCK32; do not run |
| `gserver-linux/` | `GServersource` ELF `gserver` | `analysis-only` | Prefer paired C++ source |
| `gserver-win/` / `gserver_pe_GServersource/` | Win `gserver.exe` | `analysis-only` | Same era; watch password headers |
| `gserver13/` | classic `gserver13` ELF | `analysis-only` | Classic Linux gserver |
| `Matriark_gserver/` | Matriark 1.39 ELF `gserver` | `analysis-only` | Static-stripped community build |
| `gserverstat/` / `gserverstat2/` | HTML stats emitters | `analysis-only` | Driven by `statmaker` |
| `BytecodeViewer/` | GS2 `BytecodeViewer.exe` | `analysis-only` | Package has compiler source |
| `TextConvert/` | `TextConvert.exe` | `analysis-only` | Graal→NW text helper |
| `TilesEditor/` | Qt64 `TilesEditor.exe` | `seihai` (→ `lat-core` live) | Offline tile/NPC/chest editor |
| `Assimilator/` | GAT `Assimilator.exe` | `seihai` / `analysis-only` | Batch Graal asset converter |
| `PNGView/` | GAT `PNGView.exe` | `analysis-only` / `seihai` | Offline image viewer |
| `scripteditor/` | NAT `scripteditor.exe` | `analysis-only` | Senolen GS1-era editor |
| `GraalLevelGenNW/` | LAT NW level gen | `seihai` / `analysis-only` | Offline level generator |
| `GCD/` / `GChar/` | character tools | `out-of-scope / seihai (client)` | Delphi character designers |
| `Graal_Auto_Key/` / `Graal_Digger/` | trainers | `out-of-scope` | Overlap recovered-hacks |
| `DClient_setup/` | salvaged `setup.exe` | `stay-staged` / client `out-of-scope` | Truncated Drive zip |

## Prefer source over RE when present

- `_inbox/extracted/GServersource/` — `gserver.cpp`, `npcs.cpp`, `levels.cpp`, makefiles.
- `_extracted/listserver-nested/graal-listserver/` — C# listserver rewrite.
- `_extracted/gs2-compiler/GS2 Compiler/BytecodeCompiler/` — C++ compiler sources.
- LAT `NW2PNG2_Source_Files.rar`, Tools `NW2PNG/*src*`, map-generator sources.

## Blockers

| Item | Reason |
| --- | --- |
| Seven Drive files | Google `cannotDownloadAbusiveFile` — see inventory |
| `WorldsSetup64-551.rar` | HTTP 404; `WorldsSetup64-518.rar` present instead |
| `DClientSetup1198.zip` | Truncated on Drive (no EOCD) |
| `Graal Digger.exe` | Corrupt UPX; trainer overlap |

## Re-run

```bash
/workspace/tools/ghidra/analyze_tool.sh \
  /path/to/bin \
  /workspace/recovered-drive-tools/ghidra-analysis/<name> [max_decomp]
```
