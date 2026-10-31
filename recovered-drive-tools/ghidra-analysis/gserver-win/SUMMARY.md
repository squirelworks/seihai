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
| Functions | 1051 (internal 1007) |
| Imports | 141 |
| Exports / entries | 1007 |
| Interesting strings | 1762 |
| Decompiled funcs | 29 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `SetProperties`, `parseChat`, `SendAccount`, `LoadNW`, `enterLevel`, `SetProfile`, `LoadDBAccount`, `IO_dtoa`, `SaveDBAccount`, `SetAccount`, `GetProperty`, `IO_strtod`, `SetRCPlayerProps`, `AddAccountFromRC`, `startServer`, `parse`, `connecttodb`, `loginerror`, `init_dynamic_string`, `init_dynamic_array`, `encryptpassword`, `init_block`, `init_available_charsets`, `init_alloc_root`, `do_osfx`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegEnumValueA
ADVAPI32.DLL :: RegOpenKeyExA
KERNEL32.DLL :: CloseHandle
KERNEL32.DLL :: CreateFileA
KERNEL32.DLL :: CreateSemaphoreA
KERNEL32.DLL :: DeleteCriticalSection
KERNEL32.DLL :: EnterCriticalSection
KERNEL32.DLL :: ExitProcess
KERNEL32.DLL :: GetCurrentDirectoryA
KERNEL32.DLL :: GetLastError
KERNEL32.DLL :: GetSystemTime
KERNEL32.DLL :: GetWindowsDirectoryA
KERNEL32.DLL :: InitializeCriticalSection
KERNEL32.DLL :: InterlockedDecrement
KERNEL32.DLL :: InterlockedIncrement
KERNEL32.DLL :: LeaveCriticalSection
KERNEL32.DLL :: ReadFile
KERNEL32.DLL :: ReleaseSemaphore
KERNEL32.DLL :: SetNamedPipeHandleState
KERNEL32.DLL :: SetThreadPriority
KERNEL32.DLL :: SetUnhandledExceptionFilter
KERNEL32.DLL :: Sleep
KERNEL32.DLL :: WaitForSingleObject
KERNEL32.DLL :: WaitNamedPipeA
KERNEL32.DLL :: WriteFile
MSVCRT.DLL :: _HUGE
MSVCRT.DLL :: __getmainargs
MSVCRT.DLL :: __p__environ
MSVCRT.DLL :: __set_app_type
MSVCRT.DLL :: _beginthread
MSVCRT.DLL :: _cexit
MSVCRT.DLL :: _chdir
MSVCRT.DLL :: _close
MSVCRT.DLL :: _endthread
MSVCRT.DLL :: _errno
MSVCRT.DLL :: _fdopen
MSVCRT.DLL :: _fileno
MSVCRT.DLL :: _fileno
MSVCRT.DLL :: _findclose
MSVCRT.DLL :: _fmode
MSVCRT.DLL :: _fpreset
MSVCRT.DLL :: _iob
MSVCRT.DLL :: _isatty
MSVCRT.DLL :: _lseek
MSVCRT.DLL :: _open
MSVCRT.DLL :: _putenv
MSVCRT.DLL :: _read
MSVCRT.DLL :: _setmode
MSVCRT.DLL :: _sopen
MSVCRT.DLL :: _strdup
MSVCRT.DLL :: _unlink
MSVCRT.DLL :: _write
MSVCRT.DLL :: abort
MSVCRT.DLL :: atexit
MSVCRT.DLL :: atof
MSVCRT.DLL :: atoi
MSVCRT.DLL :: atol
MSVCRT.DLL :: calloc
MSVCRT.DLL :: ctime
MSVCRT.DLL :: exit
MSVCRT.DLL :: fclose
MSVCRT.DLL :: fflush
MSVCRT.DLL :: fgets
MSVCRT.DLL :: findfirst
MSVCRT.DLL :: findnext
MSVCRT.DLL :: floor
MSVCRT.DLL :: fopen
MSVCRT.DLL :: fprintf
MSVCRT.DLL :: fputc
MSVCRT.DLL :: fputs
MSVCRT.DLL :: fread
MSVCRT.DLL :: free
MSVCRT.DLL :: freopen
MSVCRT.DLL :: fseek
MSVCRT.DLL :: fstat
MSVCRT.DLL :: ftime
MSVCRT.DLL :: fwrite
MSVCRT.DLL :: getc
MSVCRT.DLL :: getenv
```

_… 61 more in `imports.txt`_

## Exports / entry points

```
_Accept__7TSocket [function]
_AddAccountFromRC__FG7JStringb [function]
_AddAccount__FRC7JStringN20bT3iT0 [function]
_AddFileSeparators__FG7JString [function]
_AddLineEnds__FG7JString [function]
_AddModify__12TServerLevelP13TServerPlayerG7JString [function]
_AddRowToComp__13TServerPlayer [function]
_AddToList__12TServerLevelP13TServerPlayer [function]
_Add__12TJStringListRC7JString [function]
_Add__5TListPv [function]
_ApplyAccountChange__13TServerPlayerP13TServerPlayerb [function]
_Assign__12TJStringListP12TJStringList [function]
_Assign__12TServerChestP12TJStringList [function]
_Assign__16TServerLevelLinkP12TJStringList [function]
_Assign__16TServerLevelLinkRC7JString [function]
_Assign__5TListP5TList [function]
_Balloc [function]
_BaseName [function]
_Bbase__9streambuf [function]
_Bcopy [function]
_Bfree [function]
_Bind__7TSocketibT2 [function]
_Binit [function]
_Brealloc [function]
_CfgRead__FPcP10CfgStrings [function]
_Clear__12TJStringList [function]
_Clear__12TServerLevel [function]
_Clear__5TList [function]
_ClientReceive__13TServerPlayer [function]
_CloseFile [function]
_Close__7TSocket [function]
_CompressBuf__FRC7JString [function]
_CreatePlayer__12TServerFrameP7TSocket [function]
_DBUGOpenFile [function]
_DbugExit [function]
_DbugMalloc [function]
_DecompressBuf__FRC7JString [function]
_Delay [function]
_DelayArg [function]
_DeleteAccount__FRC7JStringi [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
socket exception: cannot create socket
Port address %d is currently used by another programm. Try later!
Server started. Port: %d
Couldn't bind the response socket
Response socket established at port 14899
data/serverflags.txt
data/maxplayers.txt
logs/rclog.txt
logs/errorlog.txt
TServerFrame::CreatePlayer(): Checking ipbanned.txt...
data/ipbanned.txt
TServerFrame::CreatePlayer(): Client was IP Banned!
TServerFrame::CreatePlayer(): GetFreePlayerID() returned %d!
There are too many players (28000) on this server.
TServerFrame::CreatePlayer(): Sent(UNLIMITEDSIG) = %s
RC disconnected:
levelnpcs/
TServerPlayer::ClientRecieve(): buffer << sock->Read()
TServerPlayer::ClientRecieve(): sock->error == %d
Your Graal.exe has sent to much data (>=128k in the input buffer)
Your Graal.exe has sent a wrong data package length.
Your Graal.exe has sent an empty data package.
The socket to client
The connection to client
The connection to you was blocked for more than one minute (still
Error while sending data to client
TServerPlayer::parse(): Sending Account...
level13.graal
"From jail you can only send PMs to admins (RCs)."
The server got uncertified data packages from your connection.
disconnects
The server administrator has disconnected you.
Player
loaded attributes of player
loaded player attributes of account
from world
set attributes of player
set player attributes of account
set profile error:
warped player
The server got illegal data from your connection.
' has added an npc in
Adding npcs is not allowed in this level.
Login error:
The server got corrupt login data from your connection.
TServerPlayer::SendAccount(): Client Connected:
TServerPlayer::SendAccount(): Client Type = CLIENTPLAYER (%d)
TServerPlayer::SendAccount(): Client Type = CLIENTRC (%d)
TServerPlayer::SendAccount(): Version = %s
You need Graal version 1.38 rev1 - 1.39 rev 1 to play on this server.
You need RC 2000/7/7 to connect to this server.
TServerPlayer::SendAccount(): account name = %s
TServerPlayer::SendAccount(): password = %s
The password is wrong!
data/ipgps.txt
Another RC logged in from the same IP.
data/settings.txt
has sent wrong player attributes (
The server got illegal player attributes data from your connection.
update level
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

