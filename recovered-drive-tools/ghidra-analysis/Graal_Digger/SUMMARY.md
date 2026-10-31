# Salvage analysis: Graal Digger

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/Graal_Digger/Graal Digger.exe`
- **Analyzed (UTC):** 20261031T210058Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

Client **auto-dig / AFK shovel helper** (howto by Selzar). Identical truncated UPX PE as `recovered-hacks/Trainers` (`sha256` match). `upx -d` fails (`OverlayException`); Ghidra sees only UPX stub. Packed-stream hints AutoIt v2.63. Brief note only — trainer overlap.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `out-of-scope` |
| --- | --- |
| Why | Client digger / AFK helper — not servertools. See also `recovered-hacks/ghidra-analysis/Graal_Digger/SUMMARY.md`. |
| Licence risk | Untrusted / corrupt third-party binary |

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/Graal_Digger/Graal Digger.exe: PE32 executable (GUI) Intel 80386, for MS Windows, UPX compressed, 3 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 36K Dec 17  2002 /workspace/recovered-drive-tools/_inbox/extracted/Graal_Digger/Graal Digger.exe

# sha256
876723732378b36e8fd31d09120ec5a1b584cf46fb672ab9d5cf7a0a4f8b56d4  /workspace/recovered-drive-tools/_inbox/extracted/Graal_Digger/Graal Digger.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | windows |
| Functions | 1 (internal 1) |
| Imports | 0 |
| Exports / entries | 2 |
| Interesting strings | 0 |
| Decompiled funcs | 1 |

## Notable symbols (names only)

Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

_None exported (static binary, failed analysis, or no externals)._

## Exports / entry points

```
entry [Function]
entry [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

_Ghidra string pass empty; raw `strings` filter:_

```
/u!XEl
/MSVCRT
/edi:
OSTYPf._XP'VERSION7
/CloM
/SS'
/mVirmVirmV
/mVi
/mVi
/mvi
/mvi
/mVirmVir&d
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Rebuild / rehost checklist

- [ ] Identify language/runtime from imports (libc, libstdc++, Go runtime, .NET, Python, JVM).
- [ ] Extract CLI surface from `usage`/`help` strings and decompiled arg parsers.
- [ ] Note config file paths and env vars from interesting strings.
- [ ] Map network protocols (ports, URLs, RPC names) before rehosting.
- [ ] Prefer reimplementation from observed behavior over blind binary reuse when licensing/safety unclear.

## Artifacts

- `prelim/` — file(1), strings, readelf/objdump/rizin/radare2 snapshots
- `functions.txt`, `functions.csv`
- `imports.txt`, `exports.txt`
- `strings_interesting.txt`
- `decompiled/`
- `logs/` — analyzeHeadless + script logs
- `summary_stats.txt` — machine-readable stats

