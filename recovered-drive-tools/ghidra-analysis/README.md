# Ghidra salvage analyses

Produced by `/workspace/tools/ghidra/analyze_tool.sh`.  
Pipeline docs: `/workspace/tools/ghidra/README.md`.  
Placement: `seihai-servertools/docs/design/SALVAGE_PLACEMENT.md` · inventory: [`../INVENTORY.md`](../INVENTORY.md).

Focus: purpose, CLI, protocols, config paths for rebuild/rehost notes. Do **not** paste decompiled Delphi/GPL or GServer-v2 sources into MIT crates.

## Completed

| Dir | Binary | Placement | Highlights |
| --- | --- | --- | --- |
| `serverlist3/` | `slist/serverlist3` | `analysis-only` | ELF32 classic listserver; config keys `clientsport`/`serversport`/`playerlimit`. |
| `ListServer/` | UPX-unpacked `ListServer.exe` | `analysis-only` | Win32 Delphi + WSOCK32; `servers.ini`. Do not run. |
| `gserver-linux/` | `GServersource/gserver` | `analysis-only` | ELF32 (prefer paired C++ source). |
| `gserver-win/` | `GServersource/gserver.exe` | `analysis-only` | Win32 PE same era. |
| `gserver13/` | `gserver_source/gserver13` | `analysis-only` | Classic Linux gserver ELF. |
| `gserver_pe_GServersource/` | `gserver.exe` | `analysis-only` | Win32 tree; watch `passwords.h`. |
| `BytecodeViewer/` | GS2 `BytecodeViewer.exe` | `analysis-only` | Console PE; package has compiler source. |
| `TextConvert/` | `TextConvert.exe` | `analysis-only` / client-adjacent | Graal→NW text helper. |
| `gserverstat/` | `gserverstat` | `analysis-only` | HTML leaderboards (`kills`/`deaths`/…); `statmaker` loop. |
| `gserverstat2/` | `gserverstat2` | `analysis-only` | Admin power/weapon HTML stats. |
| `GCD/` | `GCD.exe` | `out-of-scope / seihai (client)` | Character Designer 1.2.7 (Delphi). |
| `GChar/` | `GChar.exe` | `out-of-scope / seihai (client)` | Character generator sibling. |
| `Graal_Auto_Key/` | UPX-unpacked Auto Key | `out-of-scope` | AutoIt v2.64 key sender trainer. |
| `Graal_Digger/` | truncated UPX digger | `out-of-scope` | Trainer overlap w/ recovered-hacks; UPX corrupt. |
| `DClient_setup/` | salvaged `setup.exe` | `stay-staged` / client `out-of-scope` | VB6 bootstrapper; CAB truncated on Drive. |

## Prefer source over RE when present

- `_inbox/extracted/GServersource/` — `gserver.cpp`, `npcs.cpp`, `levels.cpp`, makefiles, MySQL bits.
- `_extracted/listserver-nested/graal-listserver/` — C# listserver rewrite.
- `_extracted/gs2-compiler/GS2 Compiler/BytecodeCompiler/` — C++ compiler sources.
- `_inbox/extracted/gserver_source/statmaker` + Server Lister `gserverstats.pl` — ops scripts (notes only).

## Blockers

| Item | Reason |
| --- | --- |
| `DClientSetup1198.zip` | Truncated on Drive (242431 B, no EOCD). Re-download did not help. |
| `GServer Stats.zip` (Drive) | Google blocks download (`cannotDownloadAbusiveFile`). |
| `Graal Digger.exe` | Corrupt UPX overlay; same hash as recovered-hacks trainer. |

## Re-run

```bash
/workspace/tools/ghidra/analyze_tool.sh \
  /path/to/bin \
  /workspace/recovered-drive-tools/ghidra-analysis/<name> [max_decomp]
```
