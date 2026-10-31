# Salvage analysis: ListServer.unpacked

- **Binary:** `/workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe`
- **Analyzed (UTC):** 20261031T204840Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 643K Oct 31 20:47 /workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe

# sha256
dc93b82667f293b72716985e63e3064e0155d114b1b68567e65dd1b9caa94249  /workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | borlanddelphi |
| Image base | 00400000 |
| Functions | 3299 (internal 2874) |
| Imports | 430 |
| Exports / entries | 3 |
| Interesting strings | 112 |
| Decompiled funcs | 40 |

## Top / key functions

**Largest internal:** FUN_0044b900@0044b900(sz=2171) | FUN_0044b088@0044b088(sz=2129) | FUN_0040a40c@0040a40c(sz=1766) | FUN_004718a8@004718a8(sz=1665) | FUN_0042a338@0042a338(sz=1482) | FUN_004728d8@004728d8(sz=1215) | FUN_00478384@00478384(sz=1178) | FUN_0044d2a0@0044d2a0(sz=1176) | FUN_004126a0@004126a0(sz=1133) | FUN_0041205c@0041205c(sz=1094) | FUN_00477918@00477918(sz=1091) | FUN_0042b808@0042b808(sz=1056) | FUN_0047d798@0047d798(sz=1056) | FUN_00413bac@00413bac(sz=1030) | FUN_0041449c@0041449c(sz=1018)

**Decompile targets:** entry@00487f0c | FUN_0044b900@0044b900 | FUN_0044b088@0044b088 | FUN_0040a40c@0040a40c | FUN_004718a8@004718a8 | FUN_0042a338@0042a338 | FUN_004728d8@004728d8 | FUN_00478384@00478384 | FUN_0044d2a0@0044d2a0 | FUN_004126a0@004126a0 | FUN_0041205c@0041205c | FUN_00477918@00477918 | FUN_0042b808@0042b808 | FUN_0047d798@0047d798 | FUN_00413bac@00413bac | FUN_0041449c@0041449c | FUN_00481b38@00481b38 | FUN_0047a12c@0047a12c | FUN_00441ce8@00441ce8 | FUN_00474438@00474438 | FUN_00470d84@00470d84 | FUN_0048060c@0048060c | FUN_0047dfd4@0047dfd4 | FUN_0045b98c@0045b98c | FUN_004044d8@004044d8 | FUN_004038e4@004038e4 | FUN_00441630@00441630 | FUN_0041a744@0041a744 | FUN_00403874@00403874 | FUN_004036b8@004036b8 | FUN_00403a4c@00403a4c | FUN_00403eb4@00403eb4 | FUN_0040452c@0040452c | FUN_0043c154@0043c154 | FUN_004066c0@004066c0 | FUN_004044fc@004044fc | FUN_0041a5e8@0041a5e8 | FUN_00421d50@00421d50 | FUN_00402750@00402750 | FUN_0040292c@0040292c

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:0000007b
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:0000007e
ADVAPI32.DLL :: RegCreateKeyExA @ EXTERNAL:0000007a
ADVAPI32.DLL :: RegDeleteValueA @ EXTERNAL:00000079
ADVAPI32.DLL :: RegEnumValueA @ EXTERNAL:00000078
ADVAPI32.DLL :: RegFlushKey @ EXTERNAL:00000077
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000076
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:0000007d
ADVAPI32.DLL :: RegQueryInfoKeyA @ EXTERNAL:00000075
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000074
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:0000007c
ADVAPI32.DLL :: RegSetValueExA @ EXTERNAL:00000073
COMCTL32.DLL :: ImageList_Add @ EXTERNAL:00000091
COMCTL32.DLL :: ImageList_BeginDrag @ EXTERNAL:0000008a
COMCTL32.DLL :: ImageList_Create @ EXTERNAL:00000094
COMCTL32.DLL :: ImageList_Destroy @ EXTERNAL:00000093
COMCTL32.DLL :: ImageList_DragEnter @ EXTERNAL:00000088
COMCTL32.DLL :: ImageList_DragLeave @ EXTERNAL:00000087
COMCTL32.DLL :: ImageList_DragMove @ EXTERNAL:00000086
COMCTL32.DLL :: ImageList_DragShowNolock @ EXTERNAL:00000084
COMCTL32.DLL :: ImageList_Draw @ EXTERNAL:0000008d
COMCTL32.DLL :: ImageList_DrawEx @ EXTERNAL:0000008c
COMCTL32.DLL :: ImageList_EndDrag @ EXTERNAL:00000089
COMCTL32.DLL :: ImageList_GetBkColor @ EXTERNAL:0000008e
COMCTL32.DLL :: ImageList_GetDragImage @ EXTERNAL:00000083
COMCTL32.DLL :: ImageList_GetIconSize @ EXTERNAL:00000080
COMCTL32.DLL :: ImageList_GetImageCount @ EXTERNAL:00000092
COMCTL32.DLL :: ImageList_Read @ EXTERNAL:00000082
COMCTL32.DLL :: ImageList_Remove @ EXTERNAL:0000008b
COMCTL32.DLL :: ImageList_ReplaceIcon @ EXTERNAL:00000090
COMCTL32.DLL :: ImageList_SetBkColor @ EXTERNAL:0000008f
COMCTL32.DLL :: ImageList_SetDragCursorImage @ EXTERNAL:00000085
COMCTL32.DLL :: ImageList_SetIconSize @ EXTERNAL:0000007f
COMCTL32.DLL :: ImageList_Write @ EXTERNAL:00000081
COMCTL32.DLL :: InitCommonControls @ EXTERNAL:00000095
GDI32.DLL :: BitBlt @ EXTERNAL:000000d8
GDI32.DLL :: CopyEnhMetaFileA @ EXTERNAL:000000d7
GDI32.DLL :: CreateBitmap @ EXTERNAL:000000d6
GDI32.DLL :: CreateBrushIndirect @ EXTERNAL:000000d5
GDI32.DLL :: CreateCompatibleBitmap @ EXTERNAL:000000d4
GDI32.DLL :: CreateCompatibleDC @ EXTERNAL:000000d3
GDI32.DLL :: CreateDIBSection @ EXTERNAL:000000d2
GDI32.DLL :: CreateDIBitmap @ EXTERNAL:000000d1
GDI32.DLL :: CreateFontIndirectA @ EXTERNAL:000000d0
GDI32.DLL :: CreateHalftonePalette @ EXTERNAL:000000cf
GDI32.DLL :: CreatePalette @ EXTERNAL:000000ce
GDI32.DLL :: CreatePenIndirect @ EXTERNAL:000000cd
GDI32.DLL :: CreateSolidBrush @ EXTERNAL:000000cc
GDI32.DLL :: DeleteDC @ EXTERNAL:000000cb
GDI32.DLL :: DeleteEnhMetaFile @ EXTERNAL:000000ca
GDI32.DLL :: DeleteObject @ EXTERNAL:000000c9
GDI32.DLL :: ExcludeClipRect @ EXTERNAL:000000c8
GDI32.DLL :: ExtTextOutA @ EXTERNAL:000000c7
GDI32.DLL :: GdiFlush @ EXTERNAL:000000c6
GDI32.DLL :: GetBitmapBits @ EXTERNAL:000000c5
GDI32.DLL :: GetBrushOrgEx @ EXTERNAL:000000c4
GDI32.DLL :: GetClipBox @ EXTERNAL:000000c3
GDI32.DLL :: GetCurrentPositionEx @ EXTERNAL:000000c2
GDI32.DLL :: GetDCOrgEx @ EXTERNAL:000000c1
GDI32.DLL :: GetDIBColorTable @ EXTERNAL:000000c0
GDI32.DLL :: GetDIBits @ EXTERNAL:000000bf
GDI32.DLL :: GetDeviceCaps @ EXTERNAL:000000be
GDI32.DLL :: GetEnhMetaFileBits @ EXTERNAL:000000bd
GDI32.DLL :: GetEnhMetaFileHeader @ EXTERNAL:000000bc
GDI32.DLL :: GetEnhMetaFilePaletteEntries @ EXTERNAL:000000bb
GDI32.DLL :: GetObjectA @ EXTERNAL:000000ba
GDI32.DLL :: GetPaletteEntries @ EXTERNAL:000000b9
GDI32.DLL :: GetPixel @ EXTERNAL:000000b8
GDI32.DLL :: GetStockObject @ EXTERNAL:000000b7
GDI32.DLL :: GetSystemPaletteEntries @ EXTERNAL:000000b6
GDI32.DLL :: GetTextExtentPoint32A @ EXTERNAL:000000b5
GDI32.DLL :: GetTextMetricsA @ EXTERNAL:000000b4
GDI32.DLL :: GetWinMetaFileBits @ EXTERNAL:000000b3
GDI32.DLL :: GetWindowOrgEx @ EXTERNAL:000000b2
GDI32.DLL :: IntersectClipRect @ EXTERNAL:000000b1
GDI32.DLL :: LineTo @ EXTERNAL:000000b0
GDI32.DLL :: MaskBlt @ EXTERNAL:000000af
GDI32.DLL :: MoveToEx @ EXTERNAL:000000ae
GDI32.DLL :: PatBlt @ EXTERNAL:000000ad
GDI32.DLL :: PlayEnhMetaFile @ EXTERNAL:000000ac
```

_… 350 more in `imports.txt`_

## Exports / entry points

```
entry @ 00487f0c [Function]
entry @ 00487f0c [function]
switchD_0040906e::caseD_0 @ 00409176 [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
004035f0	SOFTWARE\Borland\Delphi\RTL
00406048	Software\Borland\Locales
00406064	Software\Borland\Delphi\Locales
00407500	MSH_WHEELSUPPORT_MSG
0040d104	m/d/yy
00415148	Error
0042e26f	: HelpIntfs
0043286d	Proportional
00434ac0	commdlg_help
0043581c	MS_WINHELP
00436d87	HelpKeyword0yA
00438074	crHelp
00446098	ImmGetConversionStatus
004460b0	ImmSetConversionStatus
00458a44	System\CurrentControlSet\Control\Keyboard Layouts\%.8x
004596b0	MAINICON
00462eae	TCustomComboBoxEx\-F
0046c8b2	TSocketErrorEvent
0046c8de	TCustomWinSocket
0046c922	TGetSocketEvent
0046c9ee	TSocketNotifyEvent
0046dab8	listen
0046db24	connect
0046de78	closesocket
0046f998	file error
0046f9a4	stream error
0046f9b4	data error
0046f9d4	buffer error
0046f9e4	incompatible version
00486cd8	\Software\Microsoft\Windows\CurrentVersion\Internet Settings
00486d48	\Software\Graal\Client
00487444	http://www.graalonline.com/accounts/number2.php?param=R3JhYWwncyBDcmVhdG9ycywgQWRtaW5pc3RyYXRpb24sIGFuZCBQbGF5ZXJzIGFsbCwgU1VDSyEgU1VDSyBJIFNBWSE6Zm9udHMvYXJpYWwudHRmOjEw
004875e8	Connect to server "
00487650	Version
0048766c	Players
00487958	servers.ini
00487ab4	servers.ini
00488054	Runtime error     at 00000000
00488074	Error
00488fe8	deflate 1.1.4 Copyright 1995-2002 Jean-loup Gailly
0048c8d5	version.dll
0048cad0	GetLastError
0048ccb8	SetErrorMode
0048cde2	GetVersionExA
0048cdf2	GetVersion
0048cebc	GetLastError
0048cf2c	GetCPInfo
0048d2f4	SetViewportOrgEx
0048d8ac	WinHelpA
0048e24e	GetFileVersionInfoSizeA
0048e268	GetFileVersionInfoA
0048e296	WSAGetLastError
0048e320	socket
0048e350	listen
0048e358	ioctlsocket
0048e386	connect
0048e390	closesocket
0048e39e	bind
0049cfbc	JPEG error #%d
0049d09a	No help found for %s
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `entry_00487f0c.c` | `entry` | `00487f0c` |
| `FUN_0044b900_0044b900.c` | `FUN_0044b900` | `0044b900` |
| `FUN_0044b088_0044b088.c` | `FUN_0044b088` | `0044b088` |
| `FUN_0040a40c_0040a40c.c` | `FUN_0040a40c` | `0040a40c` |
| `FUN_004718a8_004718a8.c` | `FUN_004718a8` | `004718a8` |
| `FUN_0042a338_0042a338.c` | `FUN_0042a338` | `0042a338` |
| `FUN_004728d8_004728d8.c` | `FUN_004728d8` | `004728d8` |
| `FUN_00478384_00478384.c` | `FUN_00478384` | `00478384` |
| `FUN_0044d2a0_0044d2a0.c` | `FUN_0044d2a0` | `0044d2a0` |
| `FUN_004126a0_004126a0.c` | `FUN_004126a0` | `004126a0` |
| `FUN_0041205c_0041205c.c` | `FUN_0041205c` | `0041205c` |
| `FUN_00477918_00477918.c` | `FUN_00477918` | `00477918` |
| `FUN_0042b808_0042b808.c` | `FUN_0042b808` | `0042b808` |
| `FUN_0047d798_0047d798.c` | `FUN_0047d798` | `0047d798` |
| `FUN_00413bac_00413bac.c` | `FUN_00413bac` | `00413bac` |
| `FUN_0041449c_0041449c.c` | `FUN_0041449c` | `0041449c` |
| `FUN_00481b38_00481b38.c` | `FUN_00481b38` | `00481b38` |
| `FUN_0047a12c_0047a12c.c` | `FUN_0047a12c` | `0047a12c` |
| `FUN_00441ce8_00441ce8.c` | `FUN_00441ce8` | `00441ce8` |
| `FUN_00474438_00474438.c` | `FUN_00474438` | `00474438` |
| `FUN_00470d84_00470d84.c` | `FUN_00470d84` | `00470d84` |
| `FUN_0048060c_0048060c.c` | `FUN_0048060c` | `0048060c` |
| `FUN_0047dfd4_0047dfd4.c` | `FUN_0047dfd4` | `0047dfd4` |
| `FUN_0045b98c_0045b98c.c` | `FUN_0045b98c` | `0045b98c` |
| `FUN_004044d8_004044d8.c` | `FUN_004044d8` | `004044d8` |
| `FUN_004038e4_004038e4.c` | `FUN_004038e4` | `004038e4` |
| `FUN_00441630_00441630.c` | `FUN_00441630` | `00441630` |
| `FUN_0041a744_0041a744.c` | `FUN_0041a744` | `0041a744` |
| `FUN_00403874_00403874.c` | `FUN_00403874` | `00403874` |
| `FUN_004036b8_004036b8.c` | `FUN_004036b8` | `004036b8` |
| `FUN_00403a4c_00403a4c.c` | `FUN_00403a4c` | `00403a4c` |
| `FUN_00403eb4_00403eb4.c` | `FUN_00403eb4` | `00403eb4` |
| `FUN_0040452c_0040452c.c` | `FUN_0040452c` | `0040452c` |
| `FUN_0043c154_0043c154.c` | `FUN_0043c154` | `0043c154` |
| `FUN_004066c0_004066c0.c` | `FUN_004066c0` | `004066c0` |
| `FUN_004044fc_004044fc.c` | `FUN_004044fc` | `004044fc` |
| `FUN_0041a5e8_0041a5e8.c` | `FUN_0041a5e8` | `0041a5e8` |
| `FUN_00421d50_00421d50.c` | `FUN_00421d50` | `00421d50` |
| `FUN_00402750_00402750.c` | `FUN_00402750` | `00402750` |
| `FUN_0040292c_0040292c.c` | `FUN_0040292c` | `0040292c` |

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

