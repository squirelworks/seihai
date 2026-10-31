# Salvage analysis: GChar

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/charactergenerator/charactergenerator/GChar.exe`
- **Analyzed (UTC):** 20261031T210011Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

**GChar / Character Generator** — sibling of GCD (same Bruges lineage, v1.0). Offline classic character designer with `.chi`/`.chr` assets and `GChar.ini`. Client content tool.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `out-of-scope / seihai (client)` |
| --- | --- |
| Why | Client character tooling; do not place in servertools. |
| Not | `setup-core` / `lat-core` / `seihai-rc` |
| Licence risk | Legacy third-party tool |

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/charactergenerator/charactergenerator/GChar.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 407K Apr 23  2014 /workspace/recovered-drive-tools/_inbox/extracted/charactergenerator/charactergenerator/GChar.exe

# sha256
fe73e10fff6db1a9db88fc054315df05623a36f4fedc6ff6cf9c0f92038a60ea  /workspace/recovered-drive-tools/_inbox/extracted/charactergenerator/charactergenerator/GChar.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | borlanddelphi |
| Functions | 1934 (internal 1562) |
| Imports | 375 |
| Exports / entries | 2 |
| Interesting strings | 90 |
| Decompiled funcs | 30 |

## Notable symbols (names only)

Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCreateKeyExA
ADVAPI32.DLL :: RegFlushKey
ADVAPI32.DLL :: RegOpenKeyExA
ADVAPI32.DLL :: RegOpenKeyExA
ADVAPI32.DLL :: RegQueryValueExA
ADVAPI32.DLL :: RegQueryValueExA
COMCTL32.DLL :: ImageList_Add
COMCTL32.DLL :: ImageList_BeginDrag
COMCTL32.DLL :: ImageList_Create
COMCTL32.DLL :: ImageList_Destroy
COMCTL32.DLL :: ImageList_DragEnter
COMCTL32.DLL :: ImageList_DragLeave
COMCTL32.DLL :: ImageList_DragMove
COMCTL32.DLL :: ImageList_DragShowNolock
COMCTL32.DLL :: ImageList_Draw
COMCTL32.DLL :: ImageList_DrawEx
COMCTL32.DLL :: ImageList_EndDrag
COMCTL32.DLL :: ImageList_GetBkColor
COMCTL32.DLL :: ImageList_GetDragImage
COMCTL32.DLL :: ImageList_GetIconSize
COMCTL32.DLL :: ImageList_GetImageCount
COMCTL32.DLL :: ImageList_GetImageInfo
COMCTL32.DLL :: ImageList_Read
COMCTL32.DLL :: ImageList_Remove
COMCTL32.DLL :: ImageList_Replace
COMCTL32.DLL :: ImageList_ReplaceIcon
COMCTL32.DLL :: ImageList_SetBkColor
COMCTL32.DLL :: ImageList_SetDragCursorImage
COMCTL32.DLL :: ImageList_SetIconSize
COMCTL32.DLL :: InitCommonControls
COMDLG32.DLL :: GetOpenFileNameA
COMDLG32.DLL :: GetSaveFileNameA
GDI32.DLL :: BitBlt
GDI32.DLL :: CopyEnhMetaFileA
GDI32.DLL :: CreateBitmap
GDI32.DLL :: CreateBrushIndirect
GDI32.DLL :: CreateCompatibleBitmap
GDI32.DLL :: CreateCompatibleDC
GDI32.DLL :: CreateDIBSection
GDI32.DLL :: CreateDIBitmap
GDI32.DLL :: CreateFontIndirectA
GDI32.DLL :: CreateHalftonePalette
GDI32.DLL :: CreatePalette
GDI32.DLL :: CreatePenIndirect
GDI32.DLL :: CreateRectRgn
GDI32.DLL :: CreateSolidBrush
GDI32.DLL :: DeleteDC
GDI32.DLL :: DeleteEnhMetaFile
GDI32.DLL :: DeleteObject
GDI32.DLL :: EnumFontFamiliesExA
GDI32.DLL :: EnumFontsA
GDI32.DLL :: ExcludeClipRect
GDI32.DLL :: ExtFloodFill
GDI32.DLL :: ExtTextOutA
GDI32.DLL :: GetBitmapBits
GDI32.DLL :: GetBrushOrgEx
GDI32.DLL :: GetClipBox
GDI32.DLL :: GetCurrentPositionEx
GDI32.DLL :: GetDIBColorTable
GDI32.DLL :: GetDIBits
GDI32.DLL :: GetDeviceCaps
GDI32.DLL :: GetEnhMetaFileBits
GDI32.DLL :: GetEnhMetaFileHeader
GDI32.DLL :: GetEnhMetaFilePaletteEntries
GDI32.DLL :: GetObjectA
GDI32.DLL :: GetPaletteEntries
GDI32.DLL :: GetPixel
GDI32.DLL :: GetStockObject
GDI32.DLL :: GetSystemPaletteEntries
GDI32.DLL :: GetTextExtentPointA
GDI32.DLL :: GetTextMetricsA
GDI32.DLL :: GetWinMetaFileBits
GDI32.DLL :: GetWindowOrgEx
GDI32.DLL :: IntersectClipRect
GDI32.DLL :: LineTo
GDI32.DLL :: MaskBlt
GDI32.DLL :: MoveToEx
GDI32.DLL :: PatBlt
```

_… 295 more in `imports.txt`_

## Exports / entry points

```
entry [Function]
entry [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
SOFTWARE\Borland\Delphi\RTL
Portions Copyright (c) 1983,97 Borland
Software\Borland\Delphi\Locales
m/d/yy
crHelp
ImmGetConversionStatus
ImmSetConversionStatus
System\CurrentControlSet\Control\Keyboard Layouts\%.8x
MAINICON
commdlg_help
Gif loading failed!
ConnectMode
\shell\open\command
PlayerDesignerAbout
PlayerDesignerInstalling
TMainFormh>D
PlayerDesigner1
: Error
Unable to find Graal  Please move to your graal directory and run again
unable to find graal
headsconfig.txt
levels\heads\
levels\gifs\
levels\webgifs\
The file headsconfig.txt has not been found in the Graal folder!
Config
PlayerDesigner3
: Error
graaldir
graaldir=
logerrors
\GChar.log
GChar.ini
Graal.exe
graaldir
C:\Program Files\Graal\
C:\Games\Graal\
Please enter the directory where Graal was installed:
PlayerDesigner2
Graal Characters
The file headsconfig.txt has not been found in the Graal folder!
The file headsconfig.txt has not been found in the Graal folder!
The file headsconfig.txt has not been found in the Graal folder!
Graal Character Designer
Runtime error     at 00000000
Error
GetLastError
SetErrorMode
GetVersionExA
GetVersion
GetLastError
GetCPInfo
SetViewportOrgEx
WinHelpA
DdeGetLastError
DdeDisconnect
DdeConnect
Failed to get data for '%s'
Unable to insert a line
Clipboard does not support Icons
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

