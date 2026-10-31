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
| Image base | 00400000 |
| Functions | 1934 (internal 1562) |
| Imports | 375 |
| Exports / entries | 2 |
| Interesting strings | 90 |
| Decompiled funcs | 30 |

## Top / key functions

**Largest internal:** FUN_00417c24@00417c24(sz=1437) | FUN_00439c9c@00439c9c(sz=1426) | FUN_00447cac@00447cac(sz=1227) | FUN_00449c54@00449c54(sz=1102) | FUN_00437b64@00437b64(sz=1077) | FUN_00438dc4@00438dc4(sz=1067) | FUN_00448dd0@00448dd0(sz=1009) | FUN_00419014@00419014(sz=835) | FUN_0042d50c@0042d50c(sz=815) | FUN_0043a400@0043a400(sz=744) | FUN_004374d0@004374d0(sz=703) | FUN_0043285c@0043285c(sz=701) | FUN_00407f84@00407f84(sz=667) | FUN_00414b2c@00414b2c(sz=633) | FUN_004296a4@004296a4(sz=617)

**Decompile targets:** entry@0044ae18 | FUN_00417c24@00417c24 | FUN_00439c9c@00439c9c | FUN_00447cac@00447cac | FUN_00449c54@00449c54 | FUN_00437b64@00437b64 | FUN_00438dc4@00438dc4 | FUN_00448dd0@00448dd0 | FUN_00419014@00419014 | FUN_0042d50c@0042d50c | FUN_0043a400@0043a400 | FUN_004374d0@004374d0 | FUN_0043285c@0043285c | FUN_00407f84@00407f84 | FUN_00414b2c@00414b2c | FUN_004296a4@004296a4 | FUN_004389c8@004389c8 | FUN_00413bac@00413bac | FUN_00403920@00403920 | FUN_004049a0@004049a0 | FUN_00403088@00403088 | FUN_00420b48@00420b48 | FUN_0040be58@0040be58 | FUN_004033fc@004033fc | FUN_00402f6c@00402f6c | FUN_00402e10@00402e10 | FUN_00408568@00408568 | FUN_0041d62c@0041d62c | FUN_00402f08@00402f08 | FUN_00403974@00403974

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:0000002a
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:0000003b
ADVAPI32.DLL :: RegCreateKeyExA @ EXTERNAL:0000003a
ADVAPI32.DLL :: RegFlushKey @ EXTERNAL:00000039
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000029
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000038
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000028
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000037
COMCTL32.DLL :: ImageList_Add @ EXTERNAL:0000015f
COMCTL32.DLL :: ImageList_BeginDrag @ EXTERNAL:00000157
COMCTL32.DLL :: ImageList_Create @ EXTERNAL:00000162
COMCTL32.DLL :: ImageList_Destroy @ EXTERNAL:00000161
COMCTL32.DLL :: ImageList_DragEnter @ EXTERNAL:00000155
COMCTL32.DLL :: ImageList_DragLeave @ EXTERNAL:00000154
COMCTL32.DLL :: ImageList_DragMove @ EXTERNAL:00000153
COMCTL32.DLL :: ImageList_DragShowNolock @ EXTERNAL:00000151
COMCTL32.DLL :: ImageList_Draw @ EXTERNAL:0000015b
COMCTL32.DLL :: ImageList_DrawEx @ EXTERNAL:00000159
COMCTL32.DLL :: ImageList_EndDrag @ EXTERNAL:00000156
COMCTL32.DLL :: ImageList_GetBkColor @ EXTERNAL:0000015c
COMCTL32.DLL :: ImageList_GetDragImage @ EXTERNAL:00000150
COMCTL32.DLL :: ImageList_GetIconSize @ EXTERNAL:0000014e
COMCTL32.DLL :: ImageList_GetImageCount @ EXTERNAL:00000160
COMCTL32.DLL :: ImageList_GetImageInfo @ EXTERNAL:0000014c
COMCTL32.DLL :: ImageList_Read @ EXTERNAL:0000014f
COMCTL32.DLL :: ImageList_Remove @ EXTERNAL:00000158
COMCTL32.DLL :: ImageList_Replace @ EXTERNAL:0000015a
COMCTL32.DLL :: ImageList_ReplaceIcon @ EXTERNAL:0000015e
COMCTL32.DLL :: ImageList_SetBkColor @ EXTERNAL:0000015d
COMCTL32.DLL :: ImageList_SetDragCursorImage @ EXTERNAL:00000152
COMCTL32.DLL :: ImageList_SetIconSize @ EXTERNAL:0000014d
COMCTL32.DLL :: InitCommonControls @ EXTERNAL:00000163
COMDLG32.DLL :: GetOpenFileNameA @ EXTERNAL:00000165
COMDLG32.DLL :: GetSaveFileNameA @ EXTERNAL:00000164
GDI32.DLL :: BitBlt @ EXTERNAL:000000bd
GDI32.DLL :: CopyEnhMetaFileA @ EXTERNAL:000000bc
GDI32.DLL :: CreateBitmap @ EXTERNAL:000000bb
GDI32.DLL :: CreateBrushIndirect @ EXTERNAL:000000ba
GDI32.DLL :: CreateCompatibleBitmap @ EXTERNAL:000000b9
GDI32.DLL :: CreateCompatibleDC @ EXTERNAL:000000b8
GDI32.DLL :: CreateDIBSection @ EXTERNAL:000000b7
GDI32.DLL :: CreateDIBitmap @ EXTERNAL:000000b6
GDI32.DLL :: CreateFontIndirectA @ EXTERNAL:000000b5
GDI32.DLL :: CreateHalftonePalette @ EXTERNAL:000000b4
GDI32.DLL :: CreatePalette @ EXTERNAL:000000b3
GDI32.DLL :: CreatePenIndirect @ EXTERNAL:000000b2
GDI32.DLL :: CreateRectRgn @ EXTERNAL:000000b1
GDI32.DLL :: CreateSolidBrush @ EXTERNAL:000000b0
GDI32.DLL :: DeleteDC @ EXTERNAL:000000af
GDI32.DLL :: DeleteEnhMetaFile @ EXTERNAL:000000ae
GDI32.DLL :: DeleteObject @ EXTERNAL:000000ad
GDI32.DLL :: EnumFontFamiliesExA @ EXTERNAL:000000ac
GDI32.DLL :: EnumFontsA @ EXTERNAL:000000ab
GDI32.DLL :: ExcludeClipRect @ EXTERNAL:000000aa
GDI32.DLL :: ExtFloodFill @ EXTERNAL:000000a9
GDI32.DLL :: ExtTextOutA @ EXTERNAL:000000a8
GDI32.DLL :: GetBitmapBits @ EXTERNAL:000000a7
GDI32.DLL :: GetBrushOrgEx @ EXTERNAL:000000a6
GDI32.DLL :: GetClipBox @ EXTERNAL:000000a5
GDI32.DLL :: GetCurrentPositionEx @ EXTERNAL:000000a4
GDI32.DLL :: GetDIBColorTable @ EXTERNAL:000000a3
GDI32.DLL :: GetDIBits @ EXTERNAL:000000a2
GDI32.DLL :: GetDeviceCaps @ EXTERNAL:000000a1
GDI32.DLL :: GetEnhMetaFileBits @ EXTERNAL:000000a0
GDI32.DLL :: GetEnhMetaFileHeader @ EXTERNAL:0000009f
GDI32.DLL :: GetEnhMetaFilePaletteEntries @ EXTERNAL:0000009e
GDI32.DLL :: GetObjectA @ EXTERNAL:0000009d
GDI32.DLL :: GetPaletteEntries @ EXTERNAL:0000009c
GDI32.DLL :: GetPixel @ EXTERNAL:0000009b
GDI32.DLL :: GetStockObject @ EXTERNAL:0000009a
GDI32.DLL :: GetSystemPaletteEntries @ EXTERNAL:00000099
GDI32.DLL :: GetTextExtentPointA @ EXTERNAL:00000098
GDI32.DLL :: GetTextMetricsA @ EXTERNAL:00000097
GDI32.DLL :: GetWinMetaFileBits @ EXTERNAL:00000096
GDI32.DLL :: GetWindowOrgEx @ EXTERNAL:00000095
GDI32.DLL :: IntersectClipRect @ EXTERNAL:00000094
GDI32.DLL :: LineTo @ EXTERNAL:00000093
GDI32.DLL :: MaskBlt @ EXTERNAL:00000092
GDI32.DLL :: MoveToEx @ EXTERNAL:00000091
GDI32.DLL :: PatBlt @ EXTERNAL:00000090
```

_… 295 more in `imports.txt`_

## Exports / entry points

```
entry @ 0044ae18 [Function]
entry @ 0044ae18 [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
00402d50	SOFTWARE\Borland\Delphi\RTL
004038da	Portions Copyright (c) 1983,97 Borland
004047f8	Software\Borland\Delphi\Locales
00408240	m/d/yy
0041b9e8	crHelp
00423018	ImmGetConversionStatus
00423030	ImmSetConversionStatus
0042b4e0	System\CurrentControlSet\Control\Keyboard Layouts\%.8x
0042ba1c	MAINICON
004383dc	commdlg_help
00438804	Gif loading failed!
00440923	ConnectMode
004435a4	\shell\open\command
00443a9a	PlayerDesignerAbout
00443dc0	PlayerDesignerInstalling
00444665	TMainFormh>D
0044467a	PlayerDesigner1
00444c58	: Error
00444c60	Unable to find Graal  Please move to your graal directory and run again
00444cb0	unable to find graal
00444ce4	headsconfig.txt
00445ef8	levels\heads\
00445f10	levels\gifs\
00445f28	levels\webgifs\
0044661c	The file headsconfig.txt has not been found in the Graal folder!
00447a05	Config
00447abe	PlayerDesigner3
00448210	: Error
00448254	graaldir
00448268	graaldir=
004482ec	logerrors
00448300	\GChar.log
00448ce4	GChar.ini
00448d0c	Graal.exe
00448d20	graaldir
00448d4c	C:\Program Files\Graal\
00448d6c	C:\Games\Graal\
00448d98	Please enter the directory where Graal was installed:
004499dd	PlayerDesigner2
00449c40	Graal Characters
0044a0c0	The file headsconfig.txt has not been found in the Graal folder!
0044a844	The file headsconfig.txt has not been found in the Graal folder!
0044aa60	The file headsconfig.txt has not been found in the Graal folder!
0044aeb8	Graal Character Designer
0044b024	Runtime error     at 00000000
0044b044	Error
0044d886	GetLastError
0044dbf4	SetErrorMode
0044dd32	GetVersionExA
0044dd42	GetVersion
0044ddc0	GetLastError
0044de2a	GetCPInfo
0044dfd4	SetViewportOrgEx
0044e454	WinHelpA
0044effe	DdeGetLastError
0044f0be	DdeDisconnect
0044f0ce	DdeConnect
0045b7fa	Failed to get data for '%s'
0045b990	Unable to insert a line
0045b9c0	Clipboard does not support Icons
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `entry_0044ae18.c` | `entry` | `0044ae18` |
| `FUN_00417c24_00417c24.c` | `FUN_00417c24` | `00417c24` |
| `FUN_00439c9c_00439c9c.c` | `FUN_00439c9c` | `00439c9c` |
| `FUN_00447cac_00447cac.c` | `FUN_00447cac` | `00447cac` |
| `FUN_00449c54_00449c54.c` | `FUN_00449c54` | `00449c54` |
| `FUN_00437b64_00437b64.c` | `FUN_00437b64` | `00437b64` |
| `FUN_00438dc4_00438dc4.c` | `FUN_00438dc4` | `00438dc4` |
| `FUN_00448dd0_00448dd0.c` | `FUN_00448dd0` | `00448dd0` |
| `FUN_00419014_00419014.c` | `FUN_00419014` | `00419014` |
| `FUN_0042d50c_0042d50c.c` | `FUN_0042d50c` | `0042d50c` |
| `FUN_0043a400_0043a400.c` | `FUN_0043a400` | `0043a400` |
| `FUN_004374d0_004374d0.c` | `FUN_004374d0` | `004374d0` |
| `FUN_0043285c_0043285c.c` | `FUN_0043285c` | `0043285c` |
| `FUN_00407f84_00407f84.c` | `FUN_00407f84` | `00407f84` |
| `FUN_00414b2c_00414b2c.c` | `FUN_00414b2c` | `00414b2c` |
| `FUN_004296a4_004296a4.c` | `FUN_004296a4` | `004296a4` |
| `FUN_004389c8_004389c8.c` | `FUN_004389c8` | `004389c8` |
| `FUN_00413bac_00413bac.c` | `FUN_00413bac` | `00413bac` |
| `FUN_00403920_00403920.c` | `FUN_00403920` | `00403920` |
| `FUN_004049a0_004049a0.c` | `FUN_004049a0` | `004049a0` |
| `FUN_00403088_00403088.c` | `FUN_00403088` | `00403088` |
| `FUN_00420b48_00420b48.c` | `FUN_00420b48` | `00420b48` |
| `FUN_0040be58_0040be58.c` | `FUN_0040be58` | `0040be58` |
| `FUN_004033fc_004033fc.c` | `FUN_004033fc` | `004033fc` |
| `FUN_00402f6c_00402f6c.c` | `FUN_00402f6c` | `00402f6c` |
| `FUN_00402e10_00402e10.c` | `FUN_00402e10` | `00402e10` |
| `FUN_00408568_00408568.c` | `FUN_00408568` | `00408568` |
| `FUN_0041d62c_0041d62c.c` | `FUN_0041d62c` | `0041d62c` |
| `FUN_00402f08_00402f08.c` | `FUN_00402f08` | `00402f08` |
| `FUN_00403974_00403974.c` | `FUN_00403974` | `00403974` |

Read these for CLI parsing, config paths, and protocol handlers when rebuilding/rehosting.

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

