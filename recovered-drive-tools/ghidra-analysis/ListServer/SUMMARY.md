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
| Functions | 3299 (internal 2874) |
| Imports | 430 |
| Exports / entries | 3 |
| Interesting strings | 112 |
| Decompiled funcs | 40 |

## Notable symbols (names only)

Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCreateKeyExA
ADVAPI32.DLL :: RegDeleteValueA
ADVAPI32.DLL :: RegEnumValueA
ADVAPI32.DLL :: RegFlushKey
ADVAPI32.DLL :: RegOpenKeyExA
ADVAPI32.DLL :: RegOpenKeyExA
ADVAPI32.DLL :: RegQueryInfoKeyA
ADVAPI32.DLL :: RegQueryValueExA
ADVAPI32.DLL :: RegQueryValueExA
ADVAPI32.DLL :: RegSetValueExA
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
COMCTL32.DLL :: ImageList_Read
COMCTL32.DLL :: ImageList_Remove
COMCTL32.DLL :: ImageList_ReplaceIcon
COMCTL32.DLL :: ImageList_SetBkColor
COMCTL32.DLL :: ImageList_SetDragCursorImage
COMCTL32.DLL :: ImageList_SetIconSize
COMCTL32.DLL :: ImageList_Write
COMCTL32.DLL :: InitCommonControls
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
GDI32.DLL :: CreateSolidBrush
GDI32.DLL :: DeleteDC
GDI32.DLL :: DeleteEnhMetaFile
GDI32.DLL :: DeleteObject
GDI32.DLL :: ExcludeClipRect
GDI32.DLL :: ExtTextOutA
GDI32.DLL :: GdiFlush
GDI32.DLL :: GetBitmapBits
GDI32.DLL :: GetBrushOrgEx
GDI32.DLL :: GetClipBox
GDI32.DLL :: GetCurrentPositionEx
GDI32.DLL :: GetDCOrgEx
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
GDI32.DLL :: GetTextExtentPoint32A
GDI32.DLL :: GetTextMetricsA
GDI32.DLL :: GetWinMetaFileBits
GDI32.DLL :: GetWindowOrgEx
GDI32.DLL :: IntersectClipRect
GDI32.DLL :: LineTo
GDI32.DLL :: MaskBlt
GDI32.DLL :: MoveToEx
GDI32.DLL :: PatBlt
GDI32.DLL :: PlayEnhMetaFile
```

_… 350 more in `imports.txt`_

## Exports / entry points

```
entry [Function]
entry [function]
switchD_0040906e::caseD_0 [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
SOFTWARE\Borland\Delphi\RTL
Software\Borland\Locales
Software\Borland\Delphi\Locales
MSH_WHEELSUPPORT_MSG
m/d/yy
Error
: HelpIntfs
Proportional
commdlg_help
MS_WINHELP
HelpKeyword0yA
crHelp
ImmGetConversionStatus
ImmSetConversionStatus
System\CurrentControlSet\Control\Keyboard Layouts\%.8x
MAINICON
TCustomComboBoxEx\-F
TSocketErrorEvent
TCustomWinSocket
TGetSocketEvent
TSocketNotifyEvent
listen
connect
closesocket
file error
stream error
data error
buffer error
incompatible version
\Software\Microsoft\Windows\CurrentVersion\Internet Settings
\Software\Graal\Client
<public-url>
Connect to server "
Version
Players
servers.ini
servers.ini
Runtime error     at 00000000
Error
deflate 1.1.4 Copyright 1995-2002 Jean-loup Gailly
version.dll
GetLastError
SetErrorMode
GetVersionExA
GetVersion
GetLastError
GetCPInfo
SetViewportOrgEx
WinHelpA
GetFileVersionInfoSizeA
GetFileVersionInfoA
WSAGetLastError
socket
listen
ioctlsocket
connect
closesocket
bind
JPEG error #%d
No help found for %s
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

