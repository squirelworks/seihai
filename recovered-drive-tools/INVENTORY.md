# recovered-drive-tools inventory

Generated: `2026-10-31T20:50:06Z` (UTC). Snapshot under `/workspace/recovered-drive-tools/`.

- Placement bridge: [`SALVAGE_PLACEMENT.md`](./SALVAGE_PLACEMENT.md)
- Crate map: [`SEIHAI_SERVERTOOLS_MAP.md`](./SEIHAI_SERVERTOOLS_MAP.md)
- Machine-readable: [`inventory.json`](./inventory.json) (local; parent `.gitignore` tracks only `*.md` here)
- Counts: **8 complete** / **1 incomplete** top-level archives present; 1587 files under `_inbox/extracted/`

Hard rules: do **not** commit raw binaries into `seihai-servertools`; do **not** execute untrusted binaries. Ghidra imports only. Drive sibling may still write — coordinate by adding inventory/analysis only.

## Archives

| Path | Status | Size | SHA256 | file(1) | Extracted | Recommended placement |
|---|---|---|---|---|---|---|
| `Tools/DClientSetup1198.zip` | **incomplete** | 242.4 KB (242431) | `34207fd2d4c6377bfd9c3a0e37590b96f8d7eb6d4c363193c312e483fbe0c8ce` | Zip archive data, at least v2.0 to extract, compression method=deflate | partial → `_extracted/dclient-setup/ (setup.exe complete PE; DClient.CAB incomplete)` | stay-staged (bad archive) |
| `_inbox/Graal_Auto_Key.zip` | **complete** | 38.0 KB (38024) | `827d9b5549da03082d2025e920eef21210f8e84d156c29102b931798614c3149` | Zip archive data, at least v2.0 to extract, compression method=deflate | yes → `_inbox/extracted/Graal_Auto_Key` | out-of-scope |
| `_inbox/Graal_Digger.zip` | **complete** | 39.3 KB (39297) | `27e051d29205b797f2a44998457c5c97345628e610969133f34462524f89c046` | Zip archive data, at least v2.0 to extract, compression method=deflate | yes → `_inbox/extracted/Graal_Digger` | out-of-scope |
| `Tools/Graal Character Designer 1.2.7.zip` | **complete** | 190.6 KB (190587) | `d874b0c9699efd6d276a8cfc718349ed05b4c04d5f2189010eab0cb1a4408372` | Zip archive data, at least v1.0 to extract, compression method=store | yes → `_inbox/extracted/Graal_Character_Designer_1.2.7` | out-of-scope / seihai (client) |
| `Tools/charactergenerator.7z` | **complete** | 192.9 KB (192936) | `e686af714167db196f38f6ad5221976f1b73bd1ecaa1a0f407e3c3e867c39cad` | 7-zip archive data, version 0.3 | yes → `_inbox/extracted/charactergenerator` | out-of-scope / seihai (client) |
| `Developer Tools/# Graal Client Tools And Sources/Server Lister.rar` | **complete** | 1.35 MB (1349362) | `f549c055fad9914755e4d4542578af91b506f336ddc89aace020d26a9dad6675` | RAR archive data, v4, os: Win32, flags: Solid | yes → `_inbox/extracted/Server Lister` | analysis-only |
| `Developer Tools/NAT Tools And Scripts [NPC Administration Team]/GS2 Compiler.rar` | **complete** | 1.86 MB (1862622) | `4210be4d42c1d44e1289ca7aa7af19696830d91c210495033381e6727b6f6280` | RAR archive data, v4, os: Win32, flags: Solid | yes → `_inbox/extracted/GS2 Compiler` | analysis-only |
| `Developer Tools/# Graal Client Tools And Sources/GServer/GServer Source/gserver source.rar` | **complete** | 505.4 KB (505386) | `e226d3bde4edbe80e3f601eaedc682db659a3cd96ed72e52170097f0554ab946` | RAR archive data, v4, os: Win32, flags: Solid | yes → `_inbox/extracted/gserver_source` | analysis-only |
| `Developer Tools/# Graal Client Tools And Sources/GServer/GServer Source/GServersource.rar` | **complete** | 1.41 MB (1408823) | `68a99f3da1adf8232e0e008afc57412af14faf49d5fa8a53e3b7a264ca62cc19` | RAR archive data, v4, os: Win32, flags: Solid | yes → `_inbox/extracted/GServersource` | analysis-only |


## Placement summary

Roll-up of the Archives **Recommended placement** column (`SALVAGE_PLACEMENT.md` decision tree).

| Placement | Meaning | Archives in this snapshot |
|---|---|---|
| **analysis-only** | Study / Graal_analysis / sibling runtimes only — never paste into MIT `seihai-servertools` | `Server Lister.rar`, `GS2 Compiler.rar`, `gserver source.rar`, `GServersource.rar` |
| **out-of-scope** | Client trainers / bots — not servertools | `Graal_Auto_Key.zip`, `Graal_Digger.zip` |
| **out-of-scope / seihai (client)** | Offline client tooling if anything | `Graal Character Designer 1.2.7.zip`, `charactergenerator.7z` |
| **stay-staged (bad archive)** | Incomplete / corrupt — do not open a rewrite PR | `DClientSetup1198.zip` |

No `setup-core` / `lat-core` clean-room targets in this snapshot.

## Per-archive notes

### `Tools/DClientSetup1198.zip` — incomplete

- **Integrity:** truncated zip: no EOCD; setup.exe local member complete; DClient.CAB truncated (partial CAB also at _extracted/dclient-setup/DClient.CAB 207699/3003786 bytes)
- **Placement:** `stay-staged (bad archive)` — Incomplete download. setup.exe recoverable; CAB not. Once complete → likely client installer → out-of-scope for seihai-servertools.
- **Licence risk:** unknown until complete
- **Members (summary):** `setup.exe (complete member / partial salvage)`, `DClient.CAB (truncated)`
- **Binaries:**
  - `_extracted/dclient-setup/setup.exe` (139.8 KB (139776)) — PE32 executable (GUI) Intel 80386, for MS Windows, 4 sections; sha256=`7052d75548d0f34e290baf29aa7281b44b4eb38327a9078354e15a3dc8749da4` — Salvaged from truncated zip by Drive sibling; PE32 GUI

### `_inbox/Graal_Auto_Key.zip` — complete

- **Integrity:** unzip -t OK
- **Placement:** `out-of-scope` — Client trainer / auto-key. Not seihai-servertools.
- **Licence risk:** untrusted third-party binary
- **Members (summary):** `Graal Auto Key.exe`
- **Binaries:**
  - `_inbox/extracted/Graal_Auto_Key/Graal Auto Key.exe` (42.8 KB (42793)) — PE32 executable (GUI) Intel 80386, for MS Windows, UPX compressed, 3 sections; sha256=`463e8f757e82bc5a33d76f3e8194ebb839957f681ea56005c2a5c22669169cac`

### `_inbox/Graal_Digger.zip` — complete

- **Integrity:** unzip -t OK (local/central name mismatch); 7z extract OK
- **Placement:** `out-of-scope` — Client digger / AFK helper.
- **Licence risk:** untrusted third-party binary
- **Members (summary):** `Graal Digger.exe`
- **Binaries:**
  - `_inbox/extracted/Graal_Digger/Graal Digger.exe` (36.1 KB (36111)) — PE32 executable (GUI) Intel 80386, for MS Windows, UPX compressed, 3 sections; sha256=`876723732378b36e8fd31d09120ec5a1b584cf46fb672ab9d5cf7a0a4f8b56d4`

### `Tools/Graal Character Designer 1.2.7.zip` — complete

- **Integrity:** unzip -t OK
- **Placement:** `out-of-scope / seihai (client)` — Offline character designer → seihai client tools if anything; not servertools.
- **Licence risk:** legacy Graal tool
- **Members (summary):** `GCD.exe`, `GCD.dat`, `*.chr`, `GCD Readme.txt`
- **Binaries:**
  - `_inbox/extracted/Graal_Character_Designer_1.2.7/GCD.exe` (408.6 KB (408576)) — PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections; sha256=`d576bc534dc17425645a0adb46b401ad4a081b19f4315cf8054aef9529cfeed9`

### `Tools/charactergenerator.7z` — complete

- **Integrity:** 7z t OK
- **Placement:** `out-of-scope / seihai (client)` — Character generator → client/seihai, not servertools.
- **Licence risk:** legacy Graal tool
- **Members (summary):** `charactergenerator/GChar.exe`, `bmp/chr assets`, `GChar.ini`
- **Binaries:**
  - `_inbox/extracted/charactergenerator/charactergenerator/GChar.exe` (416.8 KB (416768)) — PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections; sha256=`fe73e10fff6db1a9db88fc054315df05623a36f4fedc6ff6cf9c0f92038a60ea`

### `Developer Tools/# Graal Client Tools And Sources/Server Lister.rar` — complete

- **Integrity:** unrar t All OK
- **Placement:** `analysis-only` — Classic ListServer → analysis-only / Graal_analysis. Nested graal-listserver-master C# source may inform seihai-listserver sibling; setup-core only holds listserver TOML helpers. Never commit or run ListServer.exe.
- **Licence risk:** ListServer.exe untrusted/malicious history; dumps may contain real hosts
- **Members (summary):** `ListServer.exe`, `slist/`, `servers.ini`, `gserverstats.pl`, `listserver.rar`, `ListServer.zip`, `graal-listserver-master.rar`
- **Binaries:**
  - `_inbox/extracted/Server Lister/ListServer.exe` (262.7 KB (262656)) — PE32 executable (GUI) Intel 80386, for MS Windows, UPX compressed, 3 sections; sha256=`3e15bcc71ede779809bad4c05f46c6128793fed5135fec3120810d15a1928b40` — UPX PE; DO NOT RUN (password stealer warning). Identical to nested listserver.rar copy.
- **Ghidra:** ghidra-analysis/ListServer/ (SUMMARY.md written; UPX-packed → 0 funcs; sibling also analyzing ListServer.unpacked.exe into same dir)

### `Developer Tools/NAT Tools And Scripts [NPC Administration Team]/GS2 Compiler.rar` — complete

- **Integrity:** unrar t All OK
- **Placement:** `analysis-only` — GS2 bytecode tooling → seihai-core parsers upstream, not servertools. Dumps contain real public hosts/IPs — strip before any docs.
- **Licence risk:** mixed; third-party server scripts in dumps
- **Members (summary):** `BytecodeCompiler sources`, `BytecodeViewer.exe`, `BytecodeDump scripts`, `Documentation`
- **Binaries:**
  - `_inbox/extracted/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe` (90.1 KB (90112)) — PE32 executable (console) Intel 80386, for MS Windows, 5 sections; sha256=`8e6b1f22947a6df814fc2f4be8169998e1f2bd1effc2becf51029eff3107f6a9`
- **Ghidra:** ghidra-analysis/BytecodeViewer/ (SUMMARY.md written)

### `Developer Tools/# Graal Client Tools And Sources/GServer/GServer Source/gserver source.rar` — complete

- **Integrity:** unrar t All OK
- **Placement:** `analysis-only` — Classic gserver C++/ELF → seihai-gserver / Graal_analysis. NEVER paste into seihai-servertools MIT crates. setup-core may later learn lab start discovery only.
- **Licence risk:** legacy/GPL-adjacent gserver source — behaviour notes only
- **Members (summary):** `gserver.cpp/h`, `gserver13 ELF`, `gserverstat*`, `makefile`, `zlib/gc`
- **Binaries:**
  - `_inbox/extracted/gserver_source/gserver13` (890.2 KB (890171)) — ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped; sha256=`be1c397d2c21049c7848a7c922455861ee0a4b413967c867c2e7efa90a7c9997`
  - `_inbox/extracted/gserver_source/gserverstat` (525.2 KB (525199)) — ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped; sha256=`b8ae1a13147567d6544fb30ab383359e6c1c19849deb9d87d219f7cf90af867d`
  - `_inbox/extracted/gserver_source/gserverstat2` (525.2 KB (525162)) — ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped; sha256=`966feaab46fea8b93e8b6c8a23cc333b9161f8f08f246b376cf4bfdb375fedbd`
- **Ghidra:** ghidra-analysis/gserver13/ (SUMMARY.md written)

### `Developer Tools/# Graal Client Tools And Sources/GServer/GServer Source/GServersource.rar` — complete

- **Integrity:** unrar t All OK
- **Placement:** `analysis-only` — Win32 gserver tree + gserver.exe. analysis-only / seihai-gserver. passwords.h present — do not commit.
- **Licence risk:** legacy gserver + passwords.h sensitive
- **Members (summary):** `gserver.exe`, `gserver.cpp/h`, `mysql/`, `passwords.h`, `levels.h`, `npcs.h`
- **Binaries:**
  - `_inbox/extracted/GServersource/gserver.exe` (2.85 MB (2850871)) — PE32 executable (console) Intel 80386, for MS Windows, 6 sections; sha256=`ad146b971eb1af76471d439f3f48b843e44c03b9b85f0ec9995877d6ba91bc01`
- **Ghidra:** ghidra-analysis/gserver_pe_GServersource/ (SUMMARY.md written; sibling also `gserver-win/`)

## Ghidra queue

Helper: `/workspace/tools/ghidra/analyze_tool.sh` (ready). Batch log: `ghidra-analysis/_batch.log`.
Sibling index: [`ghidra-analysis/README.md`](./ghidra-analysis/README.md) (done / skipped).

### Done (SUMMARY.md present)

| Priority | Binary | Output dir | Placement |
|---|---|---|---|
| high | `_inbox/extracted/Server Lister/ListServer.exe` (+ unpacked sibling) | `ghidra-analysis/ListServer/` | analysis-only |
| high | `_extracted/server-lister/Server Lister/slist/serverlist3` | `ghidra-analysis/serverlist3/` | analysis-only |
| high | `_inbox/extracted/gserver_source/gserver13` | `ghidra-analysis/gserver13/` | analysis-only |
| high | `_inbox/extracted/GServersource/gserver.exe` | `ghidra-analysis/gserver_pe_GServersource/` (+ sibling `gserver-win/`) | analysis-only |
| high | `_extracted/.../GServersource/gserver` (ELF) | `ghidra-analysis/gserver-linux/` | analysis-only |
| medium-high | `_inbox/extracted/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe` | `ghidra-analysis/BytecodeViewer/` | analysis-only |
| medium | `_extracted/graal-to-nw/TextConvert.exe` | `ghidra-analysis/TextConvert/` | analysis-only / client converter |

### Queued

| Priority | Binary | Output dir | Placement |
|---|---|---|---|
| medium | `_inbox/extracted/gserver_source/gserverstat` | `ghidra-analysis/gserverstat/` | analysis-only |
| medium | `_inbox/extracted/gserver_source/gserverstat2` | `ghidra-analysis/gserverstat2/` | analysis-only |
| low | `_extracted/dclient-setup/setup.exe` | `ghidra-analysis/DClient_setup/` | stay-staged / out-of-scope (client installer fragment) |
| low | `_inbox/extracted/Graal_Character_Designer_1.2.7/GCD.exe` | `ghidra-analysis/GCD/` | out-of-scope / seihai (client) |
| low | `_inbox/extracted/charactergenerator/charactergenerator/GChar.exe` | `ghidra-analysis/GChar/` | out-of-scope / seihai (client) |
| low | `_inbox/extracted/Graal_Auto_Key/Graal Auto Key.exe` | `ghidra-analysis/Graal_Auto_Key/` | out-of-scope |
| low | `_inbox/extracted/Graal_Digger/Graal Digger.exe` | `ghidra-analysis/Graal_Digger/` | out-of-scope |

### Skipped / blockers (from sibling README)

| Item | Reason |
|---|---|
| `DClientSetup1198.zip` | Truncated download — **stay-staged** |
| `Graal_Auto_Key`, `Graal_Digger` | Client trainers — **out-of-scope** |
| `charactergenerator` / `GCD.exe` | Client character tools — low priority |
| `scripteditor.exe` | VB5/OCX-heavy — defer unless GS2 tooling needs it |


## Extraction map

| Archive | Extract dir |
|---|---|
| `Tools/DClientSetup1198.zip` | `_extracted/dclient-setup/ (setup.exe complete PE; DClient.CAB incomplete)` _(incomplete archive)_ |
| `_inbox/Graal_Auto_Key.zip` | `_inbox/extracted/Graal_Auto_Key` |
| `_inbox/Graal_Digger.zip` | `_inbox/extracted/Graal_Digger` |
| `Tools/Graal Character Designer 1.2.7.zip` | `_inbox/extracted/Graal_Character_Designer_1.2.7` |
| `Tools/charactergenerator.7z` | `_inbox/extracted/charactergenerator` |
| `Developer Tools/# Graal Client Tools And Sources/Server Lister.rar` | `_inbox/extracted/Server Lister` |
| `Developer Tools/NAT Tools And Scripts [NPC Administration Team]/GS2 Compiler.rar` | `_inbox/extracted/GS2 Compiler` |
| `Developer Tools/# Graal Client Tools And Sources/GServer/GServer Source/gserver source.rar` | `_inbox/extracted/gserver_source` |
| `Developer Tools/# Graal Client Tools And Sources/GServer/GServer Source/GServersource.rar` | `_inbox/extracted/GServersource` |

Also: Drive sibling parallel tree under `_extracted/` (server-lister, gs2-compiler, gserver-source, listserver-nested, unpacked, dclient-setup). Nested Server Lister members under `_inbox/extracted/Server Lister/nested/`.

## Blockers / open

- `Tools/DClientSetup1198.zip` still truncated (~242 KB); CAB incomplete even in `_extracted/dclient-setup/`. Needs Drive re-download.
- No `setup-core` / `lat-core` rewrite candidates yet — all server-side material is `analysis-only`.
- Client trainers / character tools present → `out-of-scope`.
- Drive sibling has more queued downloads (`_next_batch.json`, `_queue.json`, `_oversized.json`); refresh inventory when new complete archives land.
- ListServer packed PE had 0 internal funcs; unpacked sibling SUMMARY is present under `ghidra-analysis/ListServer/`.

