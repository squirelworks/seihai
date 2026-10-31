# Salvage analysis: gserver

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe`
- **Analyzed (UTC):** 20261031T205332Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe: PE32 executable (console) Intel 80386, for MS Windows, 6 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 2.8M Jun 13  2002 /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe

# sha256
ad146b971eb1af76471d439f3f48b843e44c03b9b85f0ec9995877d6ba91bc01  /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | windows |
| Image base | 00400000 |
| Functions | 1051 (internal 1007) |
| Imports | 141 |
| Exports / entries | 1007 |
| Interesting strings | 1762 |
| Decompiled funcs | 29 |

## Top / key functions

**Largest internal:** _SetProperties__13TServerPlayerG7JString@00420e20(sz=20619) | parseChat__13TServerPlayer@00426588(sz=13136) | _SendAccount__13TServerPlayerRC7JString@00419588(sz=12402) | _LoadNW__12TServerLevelRC7JString@00431c3c(sz=9788) | _SetProperties__10TServerNPCG7JStringb@0043a8c0(sz=6932) | enterLevel__13TServerPlayerG7JStringddT1@0041d118(sz=6199) | _SetProfile__FG7JStringRC7JStringi@00448c54(sz=5080) | _LoadDBAccount__FP13TServerPlayerRC7JStringG7JString@0043f360(sz=4880) | __IO_dtoa@00459354(sz=4804) | _SaveDBAccount__FP13TServerPlayer@00440840(sz=4778) | _SetAccount__FG7JStringi@004439dc(sz=4349) | _GetProperty__13TServerPlayeri@0041ed18(sz=4088) | __IO_strtod@004580ac(sz=3991) | _SetRCPlayerProps__13TServerPlayerG7JStringP13TServerPlayerb@00418000(sz=3860) | _AddAccountFromRC__FG7JStringb@0044501c(sz=3826)

**Decompile targets:** startServer__12TServerFrameRC7JString@0040274c | parse__13TServerPlayerRC7JString@0040abd0 | connecttodb__Fv@0043edf4 | loginerror__13TServerPlayerRC7JString@00419008 | init_dynamic_string@00468038 | init_dynamic_array@00468aa4 | encryptpassword__FRC7JString@0045009c | init_block@00470774 | init_available_charsets@00461588 | init_alloc_root@00465720 | do_osfx__7ostream@00452cd8 | mainCRTStartup@004011d4 | connect2@0045a738 | init_dbug_state@0045f440 | init_client_errs@00460a0c | init_glob_errs@00463758 | connect@0045a6e0 | bind@0045a6f0 | listen@0045a6f8 | accept@0045a720 | _SetProperties__13TServerPlayerG7JString@00420e20 | parseChat__13TServerPlayer@00426588 | _SendAccount__13TServerPlayerRC7JString@00419588 | _LoadNW__12TServerLevelRC7JString@00431c3c | _SetProperties__10TServerNPCG7JStringb@0043a8c0 | enterLevel__13TServerPlayerG7JStringddT1@0041d118 | length__C7JString@0044b8c8 | clear__7JString@0044b894 | subJString__C7JStringii@0044ba60 | mysql_real_connect@0045c20c

See `functions.txt` / `functions.csv` for the full list.

## Imports

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
MSVCRT.DLL :: _fmode @ EXTERNAL:00000036
MSVCRT.DLL :: _fpreset @ EXTERNAL:00000037
MSVCRT.DLL :: _iob @ EXTERNAL:00000039
MSVCRT.DLL :: _isatty @ EXTERNAL:00000020
MSVCRT.DLL :: _lseek @ EXTERNAL:00000021
MSVCRT.DLL :: _open @ EXTERNAL:00000022
MSVCRT.DLL :: _putenv @ EXTERNAL:00000023
MSVCRT.DLL :: _read @ EXTERNAL:00000024
MSVCRT.DLL :: _setmode @ EXTERNAL:0000003a
MSVCRT.DLL :: _sopen @ EXTERNAL:00000025
MSVCRT.DLL :: _strdup @ EXTERNAL:00000027
MSVCRT.DLL :: _unlink @ EXTERNAL:00000028
MSVCRT.DLL :: _write @ EXTERNAL:00000029
MSVCRT.DLL :: abort @ EXTERNAL:0000003b
MSVCRT.DLL :: atexit @ EXTERNAL:0000003c
MSVCRT.DLL :: atof @ EXTERNAL:0000003d
MSVCRT.DLL :: atoi @ EXTERNAL:0000003e
MSVCRT.DLL :: atol @ EXTERNAL:0000003f
MSVCRT.DLL :: calloc @ EXTERNAL:00000040
MSVCRT.DLL :: ctime @ EXTERNAL:00000041
MSVCRT.DLL :: exit @ EXTERNAL:00000042
MSVCRT.DLL :: fclose @ EXTERNAL:00000043
MSVCRT.DLL :: fflush @ EXTERNAL:00000044
MSVCRT.DLL :: fgets @ EXTERNAL:00000045
MSVCRT.DLL :: findfirst @ EXTERNAL:00000034
MSVCRT.DLL :: findnext @ EXTERNAL:00000035
MSVCRT.DLL :: floor @ EXTERNAL:00000046
MSVCRT.DLL :: fopen @ EXTERNAL:00000047
MSVCRT.DLL :: fprintf @ EXTERNAL:00000048
MSVCRT.DLL :: fputc @ EXTERNAL:00000049
MSVCRT.DLL :: fputs @ EXTERNAL:0000004a
MSVCRT.DLL :: fread @ EXTERNAL:0000004b
MSVCRT.DLL :: free @ EXTERNAL:0000004c
MSVCRT.DLL :: freopen @ EXTERNAL:0000004d
MSVCRT.DLL :: fseek @ EXTERNAL:0000004e
MSVCRT.DLL :: fstat @ EXTERNAL:0000001f
MSVCRT.DLL :: ftime @ EXTERNAL:00000038
MSVCRT.DLL :: fwrite @ EXTERNAL:0000004f
MSVCRT.DLL :: getc @ EXTERNAL:00000050
MSVCRT.DLL :: getenv @ EXTERNAL:00000051
```

_… 61 more in `imports.txt`_

## Exports / entry points

```
_Accept__7TSocket @ 0044f734 [function]
_AddAccountFromRC__FG7JStringb @ 0044501c [function]
_AddAccount__FRC7JStringN20bT3iT0 @ 00444b74 [function]
_AddFileSeparators__FG7JString @ 00401c34 [function]
_AddLineEnds__FG7JString @ 0040199c [function]
_AddModify__12TServerLevelP13TServerPlayerG7JString @ 004353a8 [function]
_AddRowToComp__13TServerPlayer @ 0040912c [function]
_AddToList__12TServerLevelP13TServerPlayer @ 004350e4 [function]
_Add__12TJStringListRC7JString @ 0044cedc [function]
_Add__5TListPv @ 0044cc70 [function]
_ApplyAccountChange__13TServerPlayerP13TServerPlayerb @ 00429ca0 [function]
_Assign__12TJStringListP12TJStringList @ 0044d3d0 [function]
_Assign__12TServerChestP12TJStringList @ 0043dd64 [function]
_Assign__16TServerLevelLinkP12TJStringList @ 00437338 [function]
_Assign__16TServerLevelLinkRC7JString @ 004377a0 [function]
_Assign__5TListP5TList @ 0044ce44 [function]
_Balloc @ 00457408 [function]
_BaseName @ 004604bc [function]
_Bbase__9streambuf @ 00478618 [function]
_Bcopy @ 00457478 [function]
_Bfree @ 00457458 [function]
_Bind__7TSocketibT2 @ 0044f1c0 [function]
_Binit @ 004573dc [function]
_Brealloc @ 004574b0 [function]
_CfgRead__FPcP10CfgStrings @ 00401260 [function]
_Clear__12TJStringList @ 0044d34c [function]
_Clear__12TServerLevel @ 0042f378 [function]
_Clear__5TList @ 0044ce18 [function]
_ClientReceive__13TServerPlayer @ 004084c0 [function]
_CloseFile @ 00460340 [function]
_Close__7TSocket @ 0044f910 [function]
_CompressBuf__FRC7JString @ 0044b310 [function]
_CreatePlayer__12TServerFrameP7TSocket @ 004046c8 [function]
_DBUGOpenFile @ 00460248 [function]
_DbugExit @ 004603dc [function]
_DbugMalloc @ 0046042c [function]
_DecompressBuf__FRC7JString @ 0044b414 [function]
_Delay @ 004604e8 [function]
_DelayArg @ 004604e0 [function]
_DeleteAccount__FRC7JStringi @ 00446f3c [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

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
0040ab90	The server got illegal data from your connection.
00416c06	' has added an npc in
00416c30	Adding npcs is not allowed in this level.
00418fae	Login error:
00418fd0	The server got corrupt login data from your connection.
00419170	TServerPlayer::SendAccount(): Client Connected:
004191b0	TServerPlayer::SendAccount(): Client Type = CLIENTPLAYER (%d)
004191f0	TServerPlayer::SendAccount(): Client Type = CLIENTRC (%d)
00419230	TServerPlayer::SendAccount(): Version = %s
00419290	You need Graal version 1.38 rev1 - 1.39 rev 1 to play on this server.
004192f0	You need RC 2000/7/7 to connect to this server.
00419330	TServerPlayer::SendAccount(): account name = %s
00419370	TServerPlayer::SendAccount(): password = %s
004193d4	The password is wrong!
004194d2	data/ipgps.txt
00419550	Another RC logged in from the same IP.
00420d66	data/settings.txt
00420d90	has sent wrong player attributes (
00420dd0	The server got illegal player attributes data from your connection.
004263de	update level
```

## Decompiled C

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
| `length__C7JString_0044b8c8.c` | `length__C7JString` | `0044b8c8` |
| `clear__7JString_0044b894.c` | `clear__7JString` | `0044b894` |
| `subJString__C7JStringii_0044ba60.c` | `subJString__C7JStringii` | `0044ba60` |
| `mysql_real_connect_0045c20c.c` | `mysql_real_connect` | `0045c20c` |

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

