# Salvage analysis: GraalLevelGenNW

- **Binary:** `/workspace/recovered-drive-tools/Developer Tools/LAT Tool - [Level Administration Team]/Level Generators/GraalOnline Level Generator - NW - [2001-10-08].rar.extracted/GraalLevelGenNW/GraalLevelGenNW.exe`
- **Analyzed (UTC):** 20261031T211951Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/Developer Tools/LAT Tool - [Level Administration Team]/Level Generators/GraalOnline Level Generator - NW - [2001-10-08].rar.extracted/GraalLevelGenNW/GraalLevelGenNW.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 422K Oct  8  2001 /workspace/recovered-drive-tools/Developer Tools/LAT Tool - [Level Administration Team]/Level Generators/GraalOnline Level Generator - NW - [2001-10-08].rar.extracted/GraalLevelGenNW/GraalLevelGenNW.exe

# sha256
0b8471b65e3fd107eee61d2593edb6afb054c1f454131aa9ab842f4880b66802  /workspace/recovered-drive-tools/Developer Tools/LAT Tool - [Level Administration Team]/Level Generators/GraalOnline Level Generator - NW - [2001-10-08].rar.extracted/GraalLevelGenNW/GraalLevelGenNW.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | borlanddelphi |
| Functions | 2038 (internal 1678) |
| Imports | 375 |
| Exports / entries | 3 |
| Interesting strings | 118 |
| Decompiled funcs | 20 |

## Notable symbols (names only)

