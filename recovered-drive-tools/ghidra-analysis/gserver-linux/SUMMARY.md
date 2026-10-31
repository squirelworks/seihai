# Salvage analysis: gserver

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver`
- **Analyzed (UTC):** 20261031T205250Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.2.5, not stripped

# size
-rw-r--r-- 1 ubuntu ubuntu 1.3M Jun 13  2002 /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver

# sha256
56d9f0f8ba666c2e9ceb556932c01317374cf9d0e56c730b4d2643ce56947f9e  /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Executable and Linking Format (ELF) |
| Language | x86:LE:32:default |
| Compiler | gcc |
| Functions | 628 (internal 419) |
| Imports | 105 |
| Exports / entries | 865 |
| Interesting strings | 641 |
| Decompiled funcs | 40 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `parse`, `SetProperties`, `SendAccount`, `parseChat`, `LoadNW`, `enterLevel`, `GetProperty`, `SaveDBAccount`, `LoadDBAccount`, `SetAccount`, `SetProfile`, `SetRCPlayerProps`, `main`, `startServer`, `VerifyGuildName`, `connecttodb`, `Connect`, `Bind`, `Accept`, `loginerror`, `encryptpassword`, `init`, `start`, `ApplyAccountChange`, `LoadWeapons`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
<EXTERNAL> :: _Unwind_Resume
<EXTERNAL> :: __cxa_begin_catch
<EXTERNAL> :: __cxa_call_unexpected
<EXTERNAL> :: __cxa_end_catch
<EXTERNAL> :: __cxa_finalize
<EXTERNAL> :: __cxa_rethrow
<EXTERNAL> :: __deregister_frame_info
<EXTERNAL> :: __deregister_frame_info_bases
<EXTERNAL> :: __errno_location
<EXTERNAL> :: __gmon_start__
<EXTERNAL> :: __gxx_personality_v0
<EXTERNAL> :: __libc_start_main
<EXTERNAL> :: __register_frame_info
<EXTERNAL> :: __register_frame_info_bases
<EXTERNAL> :: __strtod_internal
<EXTERNAL> :: __strtol_internal
<EXTERNAL> :: __xstat
<EXTERNAL> :: accept
<EXTERNAL> :: bind
<EXTERNAL> :: close
<EXTERNAL> :: compress
<EXTERNAL> :: connect
<EXTERNAL> :: crypt
<EXTERNAL> :: ctime
<EXTERNAL> :: exit
<EXTERNAL> :: fclose
<EXTERNAL> :: fcntl
<EXTERNAL> :: fflush
<EXTERNAL> :: fgets
<EXTERNAL> :: fopen
<EXTERNAL> :: fread
<EXTERNAL> :: free
<EXTERNAL> :: fseek
<EXTERNAL> :: ftell
<EXTERNAL> :: fwrite
<EXTERNAL> :: getcwd
<EXTERNAL> :: gethostbyname
<EXTERNAL> :: gettimeofday
<EXTERNAL> :: inet_addr
<EXTERNAL> :: inet_ntoa
<EXTERNAL> :: listen
<EXTERNAL> :: localeconv
<EXTERNAL> :: malloc
<EXTERNAL> :: memchr
<EXTERNAL> :: memcpy
<EXTERNAL> :: memmove
<EXTERNAL> :: memset
<EXTERNAL> :: mysql_error
<EXTERNAL> :: mysql_fetch_row
<EXTERNAL> :: mysql_free_result
<EXTERNAL> :: mysql_init
<EXTERNAL> :: mysql_ping
<EXTERNAL> :: mysql_query
<EXTERNAL> :: mysql_real_connect
<EXTERNAL> :: mysql_store_result
<EXTERNAL> :: operator.delete
<EXTERNAL> :: operator.delete[]
<EXTERNAL> :: operator.new
<EXTERNAL> :: operator.new[]
<EXTERNAL> :: printf
<EXTERNAL> :: pthread_mutex_lock
<EXTERNAL> :: pthread_mutex_unlock
<EXTERNAL> :: rand
<EXTERNAL> :: realloc
<EXTERNAL> :: recv
<EXTERNAL> :: recvfrom
<EXTERNAL> :: select
<EXTERNAL> :: send
<EXTERNAL> :: sendto
<EXTERNAL> :: setsockopt
<EXTERNAL> :: signal
<EXTERNAL> :: socket
<EXTERNAL> :: sprintf
<EXTERNAL> :: strcat
<EXTERNAL> :: strchr
<EXTERNAL> :: strcmp
<EXTERNAL> :: strcpy
<EXTERNAL> :: strdup
<EXTERNAL> :: strlen
<EXTERNAL> :: strncmp
```

_… 25 more in `imports.txt`_

## Exports / entry points

```
AddAccount [Function]
AddAccount [function]
AddAccountFromRC [Function]
AddAccountFromRC [function]
AddFileSeparators [Function]
AddFileSeparators [function]
AddLineEnds [Function]
AddLineEnds [function]
CfgRead [Function]
CfgRead [function]
CompressBuf [Function]
CompressBuf [function]
DecompressBuf [Function]
DecompressBuf [function]
DeleteAccount [Function]
DeleteAccount [function]
ExtractFilepath [Function]
ExtractFilepath [function]
FileExists [Function]
FileExists [function]
GetAccount [Function]
GetAccount [function]
GetAccountCount [Function]
GetAccountCount [function]
GetAccountsList [Function]
GetAccountsList [function]
GetAdminLevel [Function]
GetAdminLevel [function]
GetLevelFile [Function]
GetLevelFile [function]
GetProfile [Function]
GetProfile [function]
GetUTCFileModTime [Function]
GetUTCFileModTime [function]
GetWorlds [Function]
GetWorlds [function]
IsIPInList [Function]
IsIPInList [function]
JString::JString [Function]
JString::JString [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
/lib/ld-linux.so.2
mysql_error
mysql_real_connect
_ZNSt15basic_streambufIcSt11char_traitsIcEE9pbackfailEi
_ZSt19__throw_ios_failurePKc
_ZSt20__throw_length_errorPKc
_ZNSt12__basic_fileIcE9pbackfailEi
_ZNSt13basic_filebufIcSt11char_traitsIcEE9pbackfailEi
printf
connect
socket
bind
listen
sprintf
fopen
__libc_start_main
Server started. Port: %d
Response socket established at port 14899
data/serverflags.txt
data/maxplayers.txt
Couldn't bind the response socket
socket exception: cannot create socket
Port address %d is currently used by another programm. Try later!
logs/rclog.txt
logs/errorlog.txt
TServerFrame::CreatePlayer(): Checking ipbanned.txt...
data/ipbanned.txt
TServerFrame::CreatePlayer(): Client was IP Banned!
TServerFrame::CreatePlayer(): GetFreePlayerID() returned %d!
TServerFrame::CreatePlayer(): Sent(UNLIMITEDSIG) = %s
There are too many players (28000) on this server.
RC disconnected:
levelnpcs/
TServerPlayer::ClientRecieve(): buffer << sock->Read()
TServerPlayer::ClientRecieve(): sock->error == %d
Your Graal.exe has sent to much data (>=128k in the input buffer)
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
The server got illegal data from your connection.
loaded attributes of player
from world
loaded player attributes of account
set attributes of player
set player attributes of account
set profile error:
warped player
' has added an npc in
Adding npcs is not allowed in this level.
Login error:
The server got corrupt login data from your connection.
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

