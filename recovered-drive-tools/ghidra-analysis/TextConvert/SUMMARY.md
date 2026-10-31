# Salvage analysis: TextConvert.exe

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_extracted/graal-to-nw/TextConvert.exe`
- **Ghidra language:** x86:LE:32:default / borlanddelphi
- **Format:** Portable Executable (PE)
- **Functions:** 1839 (internal 1500)
- **Imports / exports:** 342 / 3
- **Interesting strings / decompiled:** 111 / 20

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/graal-to-nw/TextConvert.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 8 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 303K Jul  3  2001 /workspace/recovered-drive-tools/_extracted/graal-to-nw/TextConvert.exe

# sha256
3122f8b15b24374351e01cc11654fa69d80c741ced652588a60a3fae4c088570  /workspace/recovered-drive-tools/_extracted/graal-to-nw/TextConvert.exe
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
Portions Copyright (c) 1983,97 Borland
Software\Borland\Locales
Software\Borland\Delphi\Locales
MSH_WHEELSUPPORT_MSG
m/d/yy
TCanvasD-A
TCanvas -A
crHelp
ImmGetConversionStatus
ImmSetConversionStatus
System\CurrentControlSet\Control\Keyboard Layouts\%.8x
MAINICON
if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow wbow1.gif;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootarrow playerdir;  playerdarts--;}
if (weaponfired && playerbombs>0) {  if (playerdir=0) putbomb 1,playerx+0.5,playery-1;  if (playerdir=1) putbomb 1,playerx-1.5,playery+1;  if (playerdir=2) putbomb 1,playerx+0.5,playery+2.5;  if (playerdir=3) putbomb 1,playerx+2.5,playery+1;  playerbombs--;}
if (weaponfired && playerbombs>0) {  if (playerdir=0) putbomb 2,playerx+0.5,playery-1;  if (playerdir=1) putbomb 2,playerx-1.5,playery+1;  if (playerdir=2) putbomb 2,playerx+0.5,playery+2.5;  if (playerdir=3) putbomb 2,playerx+2.5,playery+1;  playerbombs--;}
if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow wbow2.gif;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootfireball playerdir;  playerdarts--;}
if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootfireblast playerdir;  playerdarts--;}
if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow wbow4.gif;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootnuke playerdir;  playerdarts--;}
if (weaponfired && playerbombs>0) {  if (playerdir=0) putbomb 3,playerx+0.5,playery-1;  if (playerdir=1) putbomb 3,playerx-1.5,playery+1;  if (playerdir=2) putbomb 3,playerx+0.5,playery+2.5;  if (playerdir=3) putbomb 3,playerx+2.5,playery+1;  playerbombs--;}
start1.graal
house1.graal
house3.graal
playerscount
npcscount
testplayer
playersays
playersays2
playerx
playery
playerrupees
playerdarts
playerbombs
playerhearts
playerfullhearts
playerdir
playerglovepower
playerbombpower
playershootpower
playerswordpower
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

```
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCloseKey
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
COMCTL32.DLL :: ImageList_ReplaceIcon
COMCTL32.DLL :: ImageList_SetBkColor
COMCTL32.DLL :: ImageList_SetDragCursorImage
COMCTL32.DLL :: ImageList_SetIconSize
GDI32.DLL :: BitBlt
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
```

