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

## Key functions

entry@00487f0c | FUN_0044b900@0044b900 | FUN_0044b088@0044b088 | FUN_0040a40c@0040a40c | FUN_004718a8@004718a8 | FUN_0042a338@0042a338 | FUN_004728d8@004728d8 | FUN_00478384@00478384 | FUN_0044d2a0@0044d2a0 | FUN_004126a0@004126a0 | FUN_0041205c@0041205c | FUN_00477918@00477918 | FUN_0042b808@0042b808 | FUN_0047d798@0047d798 | FUN_00413bac@00413bac | FUN_0041449c@0041449c | FUN_00481b38@00481b38 | FUN_0047a12c@0047a12c | FUN_00441ce8@00441ce8 | FUN_00474438@00474438 | FUN_00470d84@00470d84 | FUN_0048060c@0048060c | FUN_0047dfd4@0047dfd4 | FUN_0045b98c@0045b98c | FUN_004044d8@004044d8 | FUN_004038e4@004038e4 | FUN_00441630@00441630 | FUN_0041a744@0041a744 | FUN_00403874@00403874 | FUN_004036b8@004036b8 | FUN_00403a4c@00403a4c | FUN_00403eb4@00403eb4 | FUN_0040452c@0040452c | FUN_0043c154@0043c154 | FUN_004066c0@004066c0 | FUN_004044fc@004044fc | FUN_0041a5e8@0041a5e8 | FUN_00421d50@00421d50 | FUN_00402750@00402750 | FUN_0040292c@0040292c

## Top functions

FUN_0044b900@0044b900(sz=2171) | FUN_0044b088@0044b088(sz=2129) | FUN_0040a40c@0040a40c(sz=1766) | FUN_004718a8@004718a8(sz=1665) | FUN_0042a338@0042a338(sz=1482) | FUN_004728d8@004728d8(sz=1215) | FUN_00478384@00478384(sz=1178) | FUN_0044d2a0@0044d2a0(sz=1176) | FUN_004126a0@004126a0(sz=1133) | FUN_0041205c@0041205c(sz=1094) | FUN_00477918@00477918(sz=1091) | FUN_0042b808@0042b808(sz=1056) | FUN_0047d798@0047d798(sz=1056) | FUN_00413bac@00413bac(sz=1030) | FUN_0041449c@0041449c(sz=1018)

## Sample interesting strings

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
```

## Decompiled

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

## Imports (head)

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
```

