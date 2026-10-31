# Salvage analysis: gserver

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver`
- **Ghidra language:** x86:LE:32:default / gcc
- **Format:** Executable and Linking Format (ELF)
- **Functions:** 628 (internal 419)
- **Imports / exports:** 105 / 865
- **Interesting strings / decompiled:** 641 / 40

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.2.5, not stripped

# size
-rw-r--r-- 1 ubuntu ubuntu 1.3M Jun 13  2002 /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver

# sha256
56d9f0f8ba666c2e9ceb556932c01317374cf9d0e56c730b4d2643ce56947f9e  /workspace/recovered-drive-tools/_inbox/extracted/GServersource/gserver
```

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `main`, `startServer`, `VerifyGuildName`, `connecttodb`, `Connect`, `Bind`, `Accept`, `loginerror`, `encryptpassword`, `init`, `start`, `parse`, `SetProperties`, `SendAccount`, `parseChat`, `LoadNW`, `enterLevel`, `GetProperty`, `SaveDBAccount`, `LoadDBAccount`, `SetAccount`, `SetProfile`, `SetRCPlayerProps`, `ApplyAccountChange`, `LoadWeapons`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `parse`, `SetProperties`, `SendAccount`, `parseChat`, `LoadNW`, `enterLevel`, `GetProperty`, `SaveDBAccount`, `LoadDBAccount`, `SetAccount`, `SetProfile`, `SetRCPlayerProps`, `main`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Sample interesting strings

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
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

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
```

