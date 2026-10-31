# Salvage analysis: gserver.exe

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe`
- **Ghidra language:** x86:LE:32:default / windows
- **Format:** Portable Executable (PE)
- **Functions:** 1051 (internal 1007)
- **Imports / exports:** 141 / 1007
- **Interesting strings / decompiled:** 1762 / 29

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe: PE32 executable (console) Intel 80386, for MS Windows, 6 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 2.8M Jun 13  2002 /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe

# sha256
ad146b971eb1af76471d439f3f48b843e44c03b9b85f0ec9995877d6ba91bc01  /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver.exe
```

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `startServer`, `parse`, `connecttodb`, `loginerror`, `init_dynamic_string`, `init_dynamic_array`, `encryptpassword`, `init_block`, `init_available_charsets`, `init_alloc_root`, `do_osfx`, `mainCRTStartup`, `init_dbug_state`, `init_client_errs`, `init_glob_errs`, `connect`, `bind`, `listen`, `accept`, `SetProperties`, `parseChat`, `SendAccount`, `LoadNW`, `enterLevel`, `length`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `SetProperties`, `parseChat`, `SendAccount`, `LoadNW`, `enterLevel`, `SetProfile`, `LoadDBAccount`, `IO_dtoa`, `SaveDBAccount`, `SetAccount`, `GetProperty`, `IO_strtod`, `SetRCPlayerProps`, `AddAccountFromRC`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Sample interesting strings

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
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

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
```

