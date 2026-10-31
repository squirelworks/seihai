# Salvage analysis: GCD

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/Graal_Character_Designer_1.2.7/GCD.exe`
- **Analyzed (UTC):** 20261031T205951Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

**Graal Character Designer 1.2.7** (Bruges / Andrew Bridges, 1999). Delphi GUI offline editor for classic character appearance (colors, heads, CHI/CHR). Expects Graal client folder (`headsconfig.txt`, `levels/heads|gifs|webgifs`, `Graal.exe` path probes).

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `out-of-scope / seihai (client)` |
| --- | --- |
| Why | Offline character / sprite tooling → Seihai client / appearance editors if anything; **not** seihai-servertools. |
| Not | `setup-core` / `lat-core` / `seihai-rc` |
| Licence risk | Legacy third-party Delphi tool |

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/Graal_Character_Designer_1.2.7/GCD.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 399K Nov 20  1999 /workspace/recovered-drive-tools/_inbox/extracted/Graal_Character_Designer_1.2.7/GCD.exe

# sha256
d576bc534dc17425645a0adb46b401ad4a081b19f4315cf8054aef9529cfeed9  /workspace/recovered-drive-tools/_inbox/extracted/Graal_Character_Designer_1.2.7/GCD.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | borlanddelphi |
| Image base | 00400000 |
| Functions | 1884 (internal 1530) |
| Imports | 375 |
| Exports / entries | 2 |
| Interesting strings | 88 |
| Decompiled funcs | 30 |

## Top / key functions

**Largest internal:** FUN_00417c3c@00417c3c(sz=1437) | FUN_00439cb4@00439cb4(sz=1426) | FUN_00440500@00440500(sz=1095) | FUN_00437b7c@00437b7c(sz=1077) | FUN_00438ddc@00438ddc(sz=1067) | FUN_0041902c@0041902c(sz=835) | FUN_0042d524@0042d524(sz=815) | FUN_0043a418@0043a418(sz=744) | FUN_004374e8@004374e8(sz=703) | FUN_00432874@00432874(sz=701) | FUN_004380a4@004380a4(sz=692) | FUN_004490ec@004490ec(sz=683) | FUN_00448b18@00448b18(sz=678) | FUN_00407f9c@00407f9c(sz=667) | FUN_00414b44@00414b44(sz=633)

**Decompile targets:** entry@004495ec | FUN_00417c3c@00417c3c | FUN_00439cb4@00439cb4 | FUN_00440500@00440500 | FUN_00437b7c@00437b7c | FUN_00438ddc@00438ddc | FUN_0041902c@0041902c | FUN_0042d524@0042d524 | FUN_0043a418@0043a418 | FUN_004374e8@004374e8 | FUN_00432874@00432874 | FUN_004380a4@004380a4 | FUN_004490ec@004490ec | FUN_00448b18@00448b18 | FUN_00407f9c@00407f9c | FUN_00414b44@00414b44 | FUN_004296bc@004296bc | FUN_004389e0@004389e0 | FUN_00403938@00403938 | FUN_004049b8@004049b8 | FUN_00420b60@00420b60 | FUN_004030a0@004030a0 | FUN_0040be70@0040be70 | FUN_00403414@00403414 | FUN_00402f84@00402f84 | FUN_00402e28@00402e28 | FUN_00408580@00408580 | FUN_00402f20@00402f20 | FUN_0040398c@0040398c | FUN_0041d644@0041d644

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
entry @ 004495ec [Function]
entry @ 004495ec [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
00402d68	SOFTWARE\Borland\Delphi\RTL
004038f2	Portions Copyright (c) 1983,97 Borland
00404810	Software\Borland\Delphi\Locales
00408258	m/d/yy
0041ba00	crHelp
00423030	ImmGetConversionStatus
00423048	ImmSetConversionStatus
0042b4f8	System\CurrentControlSet\Control\Keyboard Layouts\%.8x
0042ba34	MAINICON
004383f4	commdlg_help
0043881c	Gif loading failed!
00440289	PlayerDesigner2
004404ec	Graal Characters
00440968	The file headsconfig.txt has not been found in the Graal folder!
004410d4	The file headsconfig.txt has not been found in the Graal folder!
004412dc	The file headsconfig.txt has not been found in the Graal folder!
004414f0	headsconfig.txt
00441e0b	ConnectMode
00444a8c	\shell\open\command
00444f82	PlayerDesignerAbout
00445290	PlayerDesignerInstalling
004456cc	PlayerSexBtn&
004459c5	TMainForm8SD
004459da	PlayerDesigner1
00445df0	: Error
00445df8	Unable to find Graal  Please move to your graal directory and run again
00447048	levels\heads\
00447060	levels\gifs\
00447078	levels\webgifs\
00447720	headsconfig.txt
00447740	The file headsconfig.txt has not been found in the Graal folder!
0044867a	PlayerDesigner3
004489d4	: Error
00448a04	graaldir
00448a18	graaldir=
00449028	Graal.exe
00449048	C:\Program Files\Graal\
00449068	C:\Games\Graal\
00449080	C:\Graal\
00449094	graaldir
004490b4	Please enter the directory where Graal was installed:
0044968c	Graal Character Designer
0044a024	Runtime error     at 00000000
0044a044	Error
0044c886	GetLastError
0044cbf4	SetErrorMode
0044cd32	GetVersionExA
0044cd42	GetVersion
0044cdc0	GetLastError
0044ce2a	GetCPInfo
0044cfd4	SetViewportOrgEx
0044d454	WinHelpA
0044dffe	DdeGetLastError
0044e0be	DdeDisconnect
0044e0ce	DdeConnect
0045a7fa	Failed to get data for '%s'
0045a990	Unable to insert a line
0045a9c0	Clipboard does not support Icons
0045aa66	&Help
0045ab44	An error returned from DDE  ($0%x)
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `entry_004495ec.c` | `entry` | `004495ec` |
| `FUN_00417c3c_00417c3c.c` | `FUN_00417c3c` | `00417c3c` |
| `FUN_00439cb4_00439cb4.c` | `FUN_00439cb4` | `00439cb4` |
| `FUN_00440500_00440500.c` | `FUN_00440500` | `00440500` |
| `FUN_00437b7c_00437b7c.c` | `FUN_00437b7c` | `00437b7c` |
| `FUN_00438ddc_00438ddc.c` | `FUN_00438ddc` | `00438ddc` |
| `FUN_0041902c_0041902c.c` | `FUN_0041902c` | `0041902c` |
| `FUN_0042d524_0042d524.c` | `FUN_0042d524` | `0042d524` |
| `FUN_0043a418_0043a418.c` | `FUN_0043a418` | `0043a418` |
| `FUN_004374e8_004374e8.c` | `FUN_004374e8` | `004374e8` |
| `FUN_00432874_00432874.c` | `FUN_00432874` | `00432874` |
| `FUN_004380a4_004380a4.c` | `FUN_004380a4` | `004380a4` |
| `FUN_004490ec_004490ec.c` | `FUN_004490ec` | `004490ec` |
| `FUN_00448b18_00448b18.c` | `FUN_00448b18` | `00448b18` |
| `FUN_00407f9c_00407f9c.c` | `FUN_00407f9c` | `00407f9c` |
| `FUN_00414b44_00414b44.c` | `FUN_00414b44` | `00414b44` |
| `FUN_004296bc_004296bc.c` | `FUN_004296bc` | `004296bc` |
| `FUN_004389e0_004389e0.c` | `FUN_004389e0` | `004389e0` |
| `FUN_00403938_00403938.c` | `FUN_00403938` | `00403938` |
| `FUN_004049b8_004049b8.c` | `FUN_004049b8` | `004049b8` |
| `FUN_00420b60_00420b60.c` | `FUN_00420b60` | `00420b60` |
| `FUN_004030a0_004030a0.c` | `FUN_004030a0` | `004030a0` |
| `FUN_0040be70_0040be70.c` | `FUN_0040be70` | `0040be70` |
| `FUN_00403414_00403414.c` | `FUN_00403414` | `00403414` |
| `FUN_00402f84_00402f84.c` | `FUN_00402f84` | `00402f84` |
| `FUN_00402e28_00402e28.c` | `FUN_00402e28` | `00402e28` |
| `FUN_00408580_00408580.c` | `FUN_00408580` | `00408580` |
| `FUN_00402f20_00402f20.c` | `FUN_00402f20` | `00402f20` |
| `FUN_0040398c_0040398c.c` | `FUN_0040398c` | `0040398c` |
| `FUN_0041d644_0041d644.c` | `FUN_0041d644` | `0041d644` |

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

