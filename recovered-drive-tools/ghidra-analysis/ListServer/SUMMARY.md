# Salvage analysis: ListServer (UPX-unpacked)

- **Binary:** `_extracted/unpacked/ListServer.unpacked.exe` (static `upx -d` of recovered Win32 ListServer; PE never executed)
- **Analyzed (UTC):** 20261031T204840Z · SUMMARY placement pass 20261031T212200Z
- **Ghidra:** 12.2
- **Format:** PE32 GUI (x86), Borland Delphi

## Purpose guess

Windows **list / server-directory** helper for classic Graal. Delphi VCL + Winsock (`listen` / `bind` / `connect` / `closesocket`). Reads **`servers.ini`**, stores client prefs under `Software\Graal\Client`, shows connect-to-server UI with version/players fields. Bundled zlib/deflate. Companion to ELF `serverlist3` — different packaging, same product niche.

**Do not run** this PE (or the packed original) against public networks.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `analysis-only` |
| --- | --- |
| Why | Legacy Delphi listserver UI; prefer C# rewrite notes under `_extracted/listserver-nested/graal-listserver/` and ELF `serverlist3` behaviour notes. |
| Not | `setup-core` / `lat-core` / `seihai` / `seihai-rc` |
| Related | `serverlist3` |
| Licence | Legacy third-party — clean-room only |

## Ghidra program

| Field | Value |
| --- | --- |
| Functions | 3299 (internal 2874) |
| Imports | 430 (ADVAPI32 registry, COMCTL32, GDI32, WSOCK32) |
| Exports | 3 |
| Interesting strings | 112 |
| Decompiled (local only) | 40 — **not committed** |

## Behaviour (plain prose)

Most symbols are anonymous Delphi stubs; behaviour comes from imports and strings: socket listen/connect path, `servers.ini` config, Graal client registry hive, JPEG/UI chrome. No MySQL client strings (unlike static ELF `serverlist3`) — this build looks more like a lightweight Win32 lister/client companion than the full Linux auth listserver.

## Artifacts (local)

`prelim/`, `functions.*`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
