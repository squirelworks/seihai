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

## Key functions

entry@004410e0 | FUN_0043fd40@0043fd40 | FUN_0043f628@0043f628 | FUN_00416a88@00416a88 | FUN_0043f0f4@0043f0f4 | FUN_00428198@00428198 | FUN_00424dd8@00424dd8 | FUN_0043b014@0043b014 | FUN_00417d38@00417d38 | FUN_004327bc@004327bc | FUN_0041cbc8@0041cbc8 | FUN_004354c4@004354c4 | FUN_0040378c@0040378c | FUN_00402de0@00402de0 | FUN_00404dd4@00404dd4 | FUN_0040cb54@0040cb54 | FUN_00402d7c@00402d7c | FUN_0042468c@0042468c | FUN_00403284@00403284 | FUN_00402bdc@00402bdc

## Top functions

FUN_0043fd40@0043fd40(sz=1626) | FUN_0043f628@0043f628(sz=1534) | FUN_00416a88@00416a88(sz=1431) | FUN_0043f0f4@0043f0f4(sz=1091) | FUN_00428198@00428198(sz=977) | FUN_00424dd8@00424dd8(sz=951) | FUN_0043b014@0043b014(sz=903) | FUN_00417d38@00417d38(sz=832) | FUN_004327bc@004327bc(sz=725) | FUN_0041cbc8@0041cbc8(sz=721) | FUN_004354c4@004354c4(sz=715) | FUN_00409420@00409420(sz=710) | FUN_00420dd4@00420dd4(sz=708) | FUN_004152f0@004152f0(sz=633) | FUN_004276e4@004276e4(sz=596)

## Sample interesting strings

```
00402b1c	SOFTWARE\Borland\Delphi\RTL
00403746	Portions Copyright (c) 1983,97 Borland
00404c38	Software\Borland\Locales
00404c54	Software\Borland\Delphi\Locales
004065a4	MSH_WHEELSUPPORT_MSG
00409708	m/d/yy
00412d39	TCanvasD-A
00412d46	TCanvas -A
0041be88	crHelp
004288fc	ImmGetConversionStatus
00428914	ImmSetConversionStatus
004388b4	System\CurrentControlSet\Control\Keyboard Layouts\%.8x
00439330	MAINICON
0043b9ac	if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow wbow1.gif;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootarrow playerdir;  playerdarts--;}
0043ba5c	if (weaponfired && playerbombs>0) {  if (playerdir=0) putbomb 1,playerx+0.5,playery-1;  if (playerdir=1) putbomb 1,playerx-1.5,playery+1;  if (playerdir=2) putbomb 1,playerx+0.5,playery+2.5;  if (playerdir=3) putbomb 1,playerx+2.5,playery+1;  playerbombs--;}
0043bb68	if (weaponfired && playerbombs>0) {  if (playerdir=0) putbomb 2,playerx+0.5,playery-1;  if (playerdir=1) putbomb 2,playerx-1.5,playery+1;  if (playerdir=2) putbomb 2,playerx+0.5,playery+2.5;  if (playerdir=3) putbomb 2,playerx+2.5,playery+1;  playerbombs--;}
0043bc74	if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow wbow2.gif;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootfireball playerdir;  playerdarts--;}
0043bd28	if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootfireblast playerdir;  playerdarts--;}
0043bdd0	if (weaponfired && playerdarts>0) {  freezeplayer 0.6;  setbow wbow4.gif;  playersprite = 33;  timeout = 0.5; }if (timeout) {  shootnuke playerdir;  playerdarts--;}
0043be80	if (weaponfired && playerbombs>0) {  if (playerdir=0) putbomb 3,playerx+0.5,playery-1;  if (playerdir=1) putbomb 3,playerx-1.5,playery+1;  if (playerdir=2) putbomb 3,playerx+0.5,playery+2.5;  if (playerdir=3) putbomb 3,playerx+2.5,playery+1;  playerbombs--;}
0043c0a0	start1.graal
0043c0b8	house1.graal
0043c0d0	house3.graal
0043c38c	playerscount
0043c408	npcscount
0043c4c4	testplayer
0043c570	playersays
0043c594	playersays2
0043c9a0	playerx
0043c9b0	playery
0043c9c0	playerrupees
0043c9d8	playerdarts
0043c9ec	playerbombs
0043ca00	playerhearts
0043ca18	playerfullhearts
0043ca34	playerdir
0043ca48	playerglovepower
0043ca64	playerbombpower
0043ca7c	playershootpower
0043ca98	playerswordpower
```

## Decompiled

| File | Function | Address |
| --- | --- | --- |
| `entry_004410e0.c` | `entry` | `004410e0` |
| `FUN_0043fd40_0043fd40.c` | `FUN_0043fd40` | `0043fd40` |
| `FUN_0043f628_0043f628.c` | `FUN_0043f628` | `0043f628` |
| `FUN_00416a88_00416a88.c` | `FUN_00416a88` | `00416a88` |
| `FUN_0043f0f4_0043f0f4.c` | `FUN_0043f0f4` | `0043f0f4` |
| `FUN_00428198_00428198.c` | `FUN_00428198` | `00428198` |
| `FUN_00424dd8_00424dd8.c` | `FUN_00424dd8` | `00424dd8` |
| `FUN_0043b014_0043b014.c` | `FUN_0043b014` | `0043b014` |
| `FUN_00417d38_00417d38.c` | `FUN_00417d38` | `00417d38` |
| `FUN_004327bc_004327bc.c` | `FUN_004327bc` | `004327bc` |
| `FUN_0041cbc8_0041cbc8.c` | `FUN_0041cbc8` | `0041cbc8` |
| `FUN_004354c4_004354c4.c` | `FUN_004354c4` | `004354c4` |
| `FUN_0040378c_0040378c.c` | `FUN_0040378c` | `0040378c` |
| `FUN_00402de0_00402de0.c` | `FUN_00402de0` | `00402de0` |
| `FUN_00404dd4_00404dd4.c` | `FUN_00404dd4` | `00404dd4` |
| `FUN_0040cb54_0040cb54.c` | `FUN_0040cb54` | `0040cb54` |
| `FUN_00402d7c_00402d7c.c` | `FUN_00402d7c` | `00402d7c` |
| `FUN_0042468c_0042468c.c` | `FUN_0042468c` | `0042468c` |
| `FUN_00403284_00403284.c` | `FUN_00403284` | `00403284` |
| `FUN_00402bdc_00402bdc.c` | `FUN_00402bdc` | `00402bdc` |

## Imports (head)

```
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:0000002a
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:00000039
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000029
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000038
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000028
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000037
COMCTL32.DLL :: ImageList_Add @ EXTERNAL:00000153
COMCTL32.DLL :: ImageList_BeginDrag @ EXTERNAL:0000014c
COMCTL32.DLL :: ImageList_Create @ EXTERNAL:00000156
COMCTL32.DLL :: ImageList_Destroy @ EXTERNAL:00000155
COMCTL32.DLL :: ImageList_DragEnter @ EXTERNAL:0000014a
COMCTL32.DLL :: ImageList_DragLeave @ EXTERNAL:00000149
COMCTL32.DLL :: ImageList_DragMove @ EXTERNAL:00000148
COMCTL32.DLL :: ImageList_DragShowNolock @ EXTERNAL:00000146
COMCTL32.DLL :: ImageList_Draw @ EXTERNAL:0000014f
COMCTL32.DLL :: ImageList_DrawEx @ EXTERNAL:0000014e
COMCTL32.DLL :: ImageList_EndDrag @ EXTERNAL:0000014b
COMCTL32.DLL :: ImageList_GetBkColor @ EXTERNAL:00000150
COMCTL32.DLL :: ImageList_GetDragImage @ EXTERNAL:00000145
COMCTL32.DLL :: ImageList_GetIconSize @ EXTERNAL:00000143
COMCTL32.DLL :: ImageList_GetImageCount @ EXTERNAL:00000154
COMCTL32.DLL :: ImageList_GetImageInfo @ EXTERNAL:00000141
COMCTL32.DLL :: ImageList_Read @ EXTERNAL:00000144
COMCTL32.DLL :: ImageList_Remove @ EXTERNAL:0000014d
COMCTL32.DLL :: ImageList_ReplaceIcon @ EXTERNAL:00000152
COMCTL32.DLL :: ImageList_SetBkColor @ EXTERNAL:00000151
COMCTL32.DLL :: ImageList_SetDragCursorImage @ EXTERNAL:00000147
COMCTL32.DLL :: ImageList_SetIconSize @ EXTERNAL:00000142
GDI32.DLL :: BitBlt @ EXTERNAL:000000ad
GDI32.DLL :: CreateBitmap @ EXTERNAL:000000ac
GDI32.DLL :: CreateBrushIndirect @ EXTERNAL:000000ab
GDI32.DLL :: CreateCompatibleBitmap @ EXTERNAL:000000aa
GDI32.DLL :: CreateCompatibleDC @ EXTERNAL:000000a9
GDI32.DLL :: CreateDIBSection @ EXTERNAL:000000a8
GDI32.DLL :: CreateDIBitmap @ EXTERNAL:000000a7
GDI32.DLL :: CreateFontIndirectA @ EXTERNAL:000000a6
GDI32.DLL :: CreateHalftonePalette @ EXTERNAL:000000a5
GDI32.DLL :: CreatePalette @ EXTERNAL:000000a4
GDI32.DLL :: CreatePenIndirect @ EXTERNAL:000000a3
GDI32.DLL :: CreateRectRgn @ EXTERNAL:000000a2
```

