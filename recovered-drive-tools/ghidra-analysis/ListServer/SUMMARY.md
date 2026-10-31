# Salvage analysis: ListServer.unpacked.exe

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe`
- **Ghidra language:** x86:LE:32:default / borlanddelphi
- **Format:** Portable Executable (PE)
- **Functions:** 3299 (internal 2874)
- **Imports / exports:** 430 / 3
- **Interesting strings / decompiled:** 112 / 40

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 643K Oct 31 20:47 /workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe

# sha256
dc93b82667f293b72716985e63e3064e0155d114b1b68567e65dd1b9caa94249  /workspace/recovered-drive-tools/_extracted/unpacked/ListServer.unpacked.exe
```

## Notable symbols (names only)

Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Notable symbols (names only)

Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Sample interesting strings

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
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

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
```

