# Salvage analysis: gserver.exe

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe`
- **Ghidra language:** x86:LE:32:default / windows
- **Format:** Portable Executable (PE)
- **Functions:** 1051 (internal 1007)
- **Imports / exports:** 141 / 1007
- **Interesting strings / decompiled:** 1762 / 39

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe: PE32 executable (console) Intel 80386, for MS Windows, 6 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 2.8M Jun 13  2002 /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe

# sha256
ad146b971eb1af76471d439f3f48b843e44c03b9b85f0ec9995877d6ba91bc01  /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe
```

## Key functions

startServer__12TServerFrameRC7JString@0040274c | parse__13TServerPlayerRC7JString@0040abd0 | connecttodb__Fv@0043edf4 | loginerror__13TServerPlayerRC7JString@00419008 | init_dynamic_string@00468038 | init_dynamic_array@00468aa4 | encryptpassword__FRC7JString@0045009c | init_block@00470774 | init_available_charsets@00461588 | init_alloc_root@00465720 | do_osfx__7ostream@00452cd8 | mainCRTStartup@004011d4 | connect2@0045a738 | init_dbug_state@0045f440 | init_client_errs@00460a0c | init_glob_errs@00463758 | connect@0045a6e0 | bind@0045a6f0 | listen@0045a6f8 | accept@0045a720 | _SetProperties__13TServerPlayerG7JString@00420e20 | parseChat__13TServerPlayer@00426588 | _SendAccount__13TServerPlayerRC7JString@00419588 | _LoadNW__12TServerLevelRC7JString@00431c3c | _SetProperties__10TServerNPCG7JStringb@0043a8c0 | enterLevel__13TServerPlayerG7JStringddT1@0041d118 | _SetProfile__FG7JStringRC7JStringi@00448c54 | _LoadDBAccount__FP13TServerPlayerRC7JStringG7JString@0043f360 | _SaveDBAccount__FP13TServerPlayer@00440840 | _SetAccount__FG7JStringi@004439dc | _GetProperty__13TServerPlayeri@0041ed18 | _SetRCPlayerProps__13TServerPlayerG7JStringP13TServerPlayerb@00418000 | length__C7JString@0044b8c8 | clear__7JString@0044b894 | subJString__C7JStringii@0044ba60 | mysql_real_connect@0045c20c | text__C7JString@0044b8dc | _main@0042ce24 | _SendData__13TServerPlayeriRC7JString@00409630 | _ReadWarpAccounts__Fv@00447a2c

## Top functions

_SetProperties__13TServerPlayerG7JString@00420e20(sz=20619) | parseChat__13TServerPlayer@00426588(sz=13136) | _SendAccount__13TServerPlayerRC7JString@00419588(sz=12402) | _LoadNW__12TServerLevelRC7JString@00431c3c(sz=9788) | _SetProperties__10TServerNPCG7JStringb@0043a8c0(sz=6932) | enterLevel__13TServerPlayerG7JStringddT1@0041d118(sz=6199) | _SetProfile__FG7JStringRC7JStringi@00448c54(sz=5080) | _LoadDBAccount__FP13TServerPlayerRC7JStringG7JString@0043f360(sz=4880) | __IO_dtoa@00459354(sz=4804) | _SaveDBAccount__FP13TServerPlayer@00440840(sz=4778) | _SetAccount__FG7JStringi@004439dc(sz=4349) | _GetProperty__13TServerPlayeri@0041ed18(sz=4088) | __IO_strtod@004580ac(sz=3991) | _SetRCPlayerProps__13TServerPlayerG7JStringP13TServerPlayerb@00418000(sz=3860) | _AddAccountFromRC__FG7JStringb@0044501c(sz=3826)

## Sample interesting strings

```
004025f0	socket exception: cannot create socket
00402630	Port address %d is currently used by another programm. Try later!
00402673	Server started. Port: %d
004026b0	Couldn't bind the response socket
004026f0	Response socket established at port 14899
00402722	data/serverflags.txt
00402737	data/maxplayers.txt
00403926	logs/rclog.txt
00403d45	logs/errorlog.txt
00404530	TServerFrame::CreatePlayer(): Checking ipbanned.txt...
00404568	data/ipbanned.txt
00404590	TServerFrame::CreatePlayer(): Client was IP Banned!
004045d0	TServerFrame::CreatePlayer(): GetFreePlayerID() returned %d!
00404650	There are too many players (28000) on this server.
00404690	TServerFrame::CreatePlayer(): Sent(UNLIMITEDSIG) = %s
00404fac	RC disconnected:
00406029	levelnpcs/
004082b0	TServerPlayer::ClientRecieve(): buffer << sock->Read()
004082f0	TServerPlayer::ClientRecieve(): sock->error == %d
00408370	Your Graal.exe has sent to much data (>=128k in the input buffer)
00408410	Your Graal.exe has sent a wrong data package length.
00408490	Your Graal.exe has sent an empty data package.
00409005	The socket to client
0040909e	The connection to client
004090d0	The connection to you was blocked for more than one minute (still
004096b0	Error while sending data to client
0040a710	TServerPlayer::parse(): Sending Account...
0040a743	level13.graal
0040a810	"From jail you can only send PMs to admins (RCs)."
0040a8b0	The server got uncertified data packages from your connection.
0040a8fc	disconnects
0040a910	The server administrator has disconnected you.
0040a93f	Player
0040aa09	loaded attributes of player
0040aa90	loaded player attributes of account
0040aab6	from world
0040aacc	set attributes of player
0040aaf0	set player attributes of account
0040ab4f	set profile error:
0040ab63	warped player
```

## Decompiled

| File | Function | Address |
| --- | --- | --- |
| `startServer__12TServerFrameRC7JString_0040274c.c` | `startServer__12TServerFrameRC7JString` | `0040274c` |
| `connecttodb__Fv_0043edf4.c` | `connecttodb__Fv` | `0043edf4` |
| `loginerror__13TServerPlayerRC7JString_00419008.c` | `loginerror__13TServerPlayerRC7JString` | `00419008` |
| `init_dynamic_string_00468038.c` | `init_dynamic_string` | `00468038` |
| `init_dynamic_array_00468aa4.c` | `init_dynamic_array` | `00468aa4` |
| `encryptpassword__FRC7JString_0045009c.c` | `encryptpassword__FRC7JString` | `0045009c` |
| `init_block_00470774.c` | `init_block` | `00470774` |
| `init_available_charsets_00461588.c` | `init_available_charsets` | `00461588` |
| `init_alloc_root_00465720.c` | `init_alloc_root` | `00465720` |
| `do_osfx__7ostream_00452cd8.c` | `do_osfx__7ostream` | `00452cd8` |
| `mainCRTStartup_004011d4.c` | `mainCRTStartup` | `004011d4` |
| `connect2_0045a738.c` | `connect2` | `0045a738` |
| `init_dbug_state_0045f440.c` | `init_dbug_state` | `0045f440` |
| `init_client_errs_00460a0c.c` | `init_client_errs` | `00460a0c` |
| `init_glob_errs_00463758.c` | `init_glob_errs` | `00463758` |
| `connect_0045a6e0.c` | `connect` | `0045a6e0` |
| `bind_0045a6f0.c` | `bind` | `0045a6f0` |
| `listen_0045a6f8.c` | `listen` | `0045a6f8` |
| `accept_0045a720.c` | `accept` | `0045a720` |
| `_SetProperties__13TServerPlayerG7JString_00420e20.c` | `_SetProperties__13TServerPlayerG7JString` | `00420e20` |
| `parseChat__13TServerPlayer_00426588.c` | `parseChat__13TServerPlayer` | `00426588` |
| `_SendAccount__13TServerPlayerRC7JString_00419588.c` | `_SendAccount__13TServerPlayerRC7JString` | `00419588` |
| `_LoadNW__12TServerLevelRC7JString_00431c3c.c` | `_LoadNW__12TServerLevelRC7JString` | `00431c3c` |
| `_SetProperties__10TServerNPCG7JStringb_0043a8c0.c` | `_SetProperties__10TServerNPCG7JStringb` | `0043a8c0` |
| `enterLevel__13TServerPlayerG7JStringddT1_0041d118.c` | `enterLevel__13TServerPlayerG7JStringddT1` | `0041d118` |
| `_SetProfile__FG7JStringRC7JStringi_00448c54.c` | `_SetProfile__FG7JStringRC7JStringi` | `00448c54` |
| `_LoadDBAccount__FP13TServerPlayerRC7JStringG7JString_0043f360.c` | `_LoadDBAccount__FP13TServerPlayerRC7JStringG7JString` | `0043f360` |
| `_SaveDBAccount__FP13TServerPlayer_00440840.c` | `_SaveDBAccount__FP13TServerPlayer` | `00440840` |
| `_SetAccount__FG7JStringi_004439dc.c` | `_SetAccount__FG7JStringi` | `004439dc` |
| `_GetProperty__13TServerPlayeri_0041ed18.c` | `_GetProperty__13TServerPlayeri` | `0041ed18` |
| `_SetRCPlayerProps__13TServerPlayerG7JStringP13TServerPlayerb_00418000.c` | `_SetRCPlayerProps__13TServerPlayerG7JStringP13TServerPlayerb` | `00418000` |
| `length__C7JString_0044b8c8.c` | `length__C7JString` | `0044b8c8` |
| `clear__7JString_0044b894.c` | `clear__7JString` | `0044b894` |
| `subJString__C7JStringii_0044ba60.c` | `subJString__C7JStringii` | `0044ba60` |
| `mysql_real_connect_0045c20c.c` | `mysql_real_connect` | `0045c20c` |
| `text__C7JString_0044b8dc.c` | `text__C7JString` | `0044b8dc` |
| `_main_0042ce24.c` | `_main` | `0042ce24` |
| `_SendData__13TServerPlayeriRC7JString_00409630.c` | `_SendData__13TServerPlayeriRC7JString` | `00409630` |
| `_ReadWarpAccounts__Fv_00447a2c.c` | `_ReadWarpAccounts__Fv` | `00447a2c` |

## Imports (head)

```
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:00000001
ADVAPI32.DLL :: RegEnumValueA @ EXTERNAL:00000002
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000003
KERNEL32.DLL :: CloseHandle @ EXTERNAL:00000004
KERNEL32.DLL :: CreateFileA @ EXTERNAL:00000005
KERNEL32.DLL :: CreateSemaphoreA @ EXTERNAL:00000006
KERNEL32.DLL :: DeleteCriticalSection @ EXTERNAL:00000007
KERNEL32.DLL :: EnterCriticalSection @ EXTERNAL:00000008
KERNEL32.DLL :: ExitProcess @ EXTERNAL:00000009
KERNEL32.DLL :: GetCurrentDirectoryA @ EXTERNAL:0000000a
KERNEL32.DLL :: GetLastError @ EXTERNAL:0000000b
KERNEL32.DLL :: GetSystemTime @ EXTERNAL:0000000c
KERNEL32.DLL :: GetWindowsDirectoryA @ EXTERNAL:0000000d
KERNEL32.DLL :: InitializeCriticalSection @ EXTERNAL:0000000e
KERNEL32.DLL :: InterlockedDecrement @ EXTERNAL:0000000f
KERNEL32.DLL :: InterlockedIncrement @ EXTERNAL:00000010
KERNEL32.DLL :: LeaveCriticalSection @ EXTERNAL:00000011
KERNEL32.DLL :: ReadFile @ EXTERNAL:00000012
KERNEL32.DLL :: ReleaseSemaphore @ EXTERNAL:00000013
KERNEL32.DLL :: SetNamedPipeHandleState @ EXTERNAL:00000014
KERNEL32.DLL :: SetThreadPriority @ EXTERNAL:00000015
KERNEL32.DLL :: SetUnhandledExceptionFilter @ EXTERNAL:00000016
KERNEL32.DLL :: Sleep @ EXTERNAL:00000017
KERNEL32.DLL :: WaitForSingleObject @ EXTERNAL:00000018
KERNEL32.DLL :: WaitNamedPipeA @ EXTERNAL:00000019
KERNEL32.DLL :: WriteFile @ EXTERNAL:0000001a
MSVCRT.DLL :: _HUGE @ EXTERNAL:0000002a
MSVCRT.DLL :: __getmainargs @ EXTERNAL:0000002b
MSVCRT.DLL :: __p__environ @ EXTERNAL:0000002c
MSVCRT.DLL :: __set_app_type @ EXTERNAL:0000002d
MSVCRT.DLL :: _beginthread @ EXTERNAL:0000002e
MSVCRT.DLL :: _cexit @ EXTERNAL:0000002f
MSVCRT.DLL :: _chdir @ EXTERNAL:0000001b
MSVCRT.DLL :: _close @ EXTERNAL:0000001c
MSVCRT.DLL :: _endthread @ EXTERNAL:00000030
MSVCRT.DLL :: _errno @ EXTERNAL:00000031
MSVCRT.DLL :: _fdopen @ EXTERNAL:0000001d
MSVCRT.DLL :: _fileno @ EXTERNAL:0000001e
MSVCRT.DLL :: _fileno @ EXTERNAL:00000032
MSVCRT.DLL :: _findclose @ EXTERNAL:00000033
```

