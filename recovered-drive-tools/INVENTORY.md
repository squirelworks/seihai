# Recovered Drive tools — inventory

Staging root: `/workspace/recovered-drive-tools/`  
Placement rules: `seihai-servertools/docs/design/SALVAGE_PLACEMENT.md`  
Ghidra outputs: `ghidra-analysis/`  
Generated / refreshed: 2026-10-31 (Ghidra follow-up queue)

**Rules:** do not commit raw binaries into `seihai-servertools`; static analysis only — never execute recovered PEs. Only `*.md` under this folder is git-tracked.

## Archive status

| ID | Path | Status | Placement | Ghidra |
| --- | --- | --- | --- | --- |
| DClientSetup1198 | `Tools/DClientSetup1198.zip` | **incomplete** (no EOCD; Drive source also 242431 B) | `stay-staged (bad archive)` → client installer `out-of-scope` | `ghidra-analysis/DClient_setup/` (`setup.exe` salvage) |
| Graal_Auto_Key | `_inbox/Graal_Auto_Key.zip` | complete | `out-of-scope` (trainer) | `ghidra-analysis/Graal_Auto_Key/` (UPX-unpacked) |
| Graal_Digger | `_inbox/Graal_Digger.zip` | complete zip; PE truncated/corrupt UPX | `out-of-scope` (trainer) | `ghidra-analysis/Graal_Digger/` (brief; = recovered-hacks digger) |
| Graal_Character_Designer_1.2.7 | `Tools/Graal Character Designer 1.2.7.zip` | complete | `out-of-scope / seihai (client)` | `ghidra-analysis/GCD/` |
| charactergenerator | `Tools/charactergenerator.7z` | complete | `out-of-scope / seihai (client)` | `ghidra-analysis/GChar/` |
| Server_Lister | `Developer Tools/.../Server Lister.rar` | complete | `analysis-only` | `ListServer/`, `serverlist3/` (done earlier) |
| GS2_Compiler | `Developer Tools/.../GS2 Compiler.rar` | complete | `analysis-only` | `BytecodeViewer/` (done earlier) |
| gserver_source | `.../gserver source.rar` | complete | `analysis-only` | `gserver13/`, **`gserverstat/`**, **`gserverstat2/`** |
| GServersource | `.../GServersource.rar` | complete | `analysis-only` | `gserver_pe_GServersource/`, `gserver-linux/`, `gserver-win/` |

## DClient re-download (2026-10-31)

| Check | Result |
| --- | --- |
| Drive folder | Owner private Drive `Tools` folder (ids omitted) |
| File | `DClientSetup1198.zip` on Drive (id omitted) |
| Drive `size` | **242431** (matches local truncated zip) |
| EOCD | missing — zip incomplete on Drive itself |
| Action | Local archive **not replaced** (re-download cannot improve). Partial extract kept: `_extracted/dclient-setup/setup.exe` (complete PE) + truncated `DClient.CAB`. |
| Related Drive note | `GServer Stats.zip` blocked by Google as malware/spam — not downloaded. |

## Ghidra queue (this follow-up)

| Binary | Out dir | Placement | Status |
| --- | --- | --- | --- |
| `gserver_source/gserverstat` | `ghidra-analysis/gserverstat/` | `analysis-only` | done |
| `gserver_source/gserverstat2` | `ghidra-analysis/gserverstat2/` | `analysis-only` | done |
| `GCD.exe` | `ghidra-analysis/GCD/` | `out-of-scope / seihai (client)` | done |
| `GChar.exe` | `ghidra-analysis/GChar/` | `out-of-scope / seihai (client)` | done |
| `Graal Auto Key.exe` (UPX→unpacked) | `ghidra-analysis/Graal_Auto_Key/` | `out-of-scope` | done |
| `Graal Digger.exe` (corrupt UPX) | `ghidra-analysis/Graal_Digger/` | `out-of-scope` | done (brief) |
| `dclient-setup/setup.exe` | `ghidra-analysis/DClient_setup/` | `stay-staged` / client `out-of-scope` | done |

Already done (not re-run): `ListServer/`, `gserver13/`, `gserver_pe_GServersource/`, `BytecodeViewer/`, `serverlist3/`, `gserver-linux/`, `gserver-win/`.

## Companion non-PE notes

| Artifact | Role | Placement |
| --- | --- | --- |
| `gserver_source/statmaker` | Shell loop driving `gserverstat`/`gserverstat2` → HTML | `analysis-only` |
| `Server Lister/gserverstats.pl` | Perl: read `playercount.txt` / `accountcount.txt` / `serveruptime.txt` + `ps` check | `analysis-only` (optional behaviour note for lab status; see `SALVAGE_CANDIDATES.md`) |

## Clean-room rewrite

See [`SALVAGE_CANDIDATES.md`](./SALVAGE_CANDIDATES.md). **No binaries** should be copied into `seihai-servertools`.
