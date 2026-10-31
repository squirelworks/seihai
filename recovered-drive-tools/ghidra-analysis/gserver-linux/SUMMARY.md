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
| Image base | 08048000 |
| Functions | 628 (internal 419) |
| Imports | 105 |
| Exports / entries | 865 |
| Interesting strings | 641 |
| Decompiled funcs | 40 |

## Top / key functions

**Largest internal:** parse@08052d00(sz=30008) | SetProperties@08062090(sz=11379) | SendAccount@0805cc40(sz=7486) | parseChat@08065470(sz=6203) | LoadNW@0806c080(sz=4759) | SetProperties@08071a10(sz=4654) | enterLevel@0805f6a0(sz=3533) | GetProperty@08060a10(sz=3202) | SaveDBAccount@08075960(sz=3030) | LoadDBAccount@08074be0(sz=2882) | SetAccount@08077640(sz=2716) | GetProperty@08070ed0(sz=2691) | SetProfile@0807a7c0(sz=2339) | SetRCPlayerProps@0805c180(sz=2245) | main@080690b0(sz=1935)

**Decompile targets:** main@080690b0 | startServer@0804dff0 | VerifyGuildName@0807b320 | connecttodb@08074820 | Connect@0807f120 | Bind@0807f240 | Accept@0807f6b0 | loginerror@0805cb30 | encryptpassword@0807f910 | init@0807ff10 | _start@0804d110 | init_dummy@0804d230 | init_dummy@0807fb90 | parse@08052d00 | SetProperties@08062090 | SendAccount@0805cc40 | parseChat@08065470 | LoadNW@0806c080 | SetProperties@08071a10 | enterLevel@0805f6a0 | GetProperty@08060a10 | SaveDBAccount@08075960 | LoadDBAccount@08074be0 | SetAccount@08077640 | GetProperty@08070ed0 | SetProfile@0807a7c0 | SetRCPlayerProps@0805c180 | ApplyAccountChange@080675a0 | LoadWeapons@080767d0 | ~JString@0807c240 | operator<<@0807c600 | JString@0807bff0 | clear@0807c260 | JString@0807c060 | JString@0807c160 | subJString@0807c460 | text@0807c2b0 | operator[]@0807d7c0 | operator+@0807ca40 | operator<<@0807c740

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
<EXTERNAL> :: _Unwind_Resume @ EXTERNAL:0000005e
<EXTERNAL> :: __cxa_begin_catch @ EXTERNAL:00000051
<EXTERNAL> :: __cxa_call_unexpected @ EXTERNAL:00000013
<EXTERNAL> :: __cxa_end_catch @ EXTERNAL:00000025
<EXTERNAL> :: __cxa_finalize @ EXTERNAL:0000004d
<EXTERNAL> :: __cxa_rethrow @ EXTERNAL:00000004
<EXTERNAL> :: __deregister_frame_info @ EXTERNAL:0000002c
<EXTERNAL> :: __deregister_frame_info_bases @ EXTERNAL:0000001c
<EXTERNAL> :: __errno_location @ EXTERNAL:0000001d
<EXTERNAL> :: __gmon_start__ @ EXTERNAL:00000069
<EXTERNAL> :: __gxx_personality_v0 @ EXTERNAL:00000028
<EXTERNAL> :: __libc_start_main @ EXTERNAL:0000003e
<EXTERNAL> :: __register_frame_info @ EXTERNAL:0000000f
<EXTERNAL> :: __register_frame_info_bases @ EXTERNAL:0000003d
<EXTERNAL> :: __strtod_internal @ EXTERNAL:00000006
<EXTERNAL> :: __strtol_internal @ EXTERNAL:00000036
<EXTERNAL> :: __xstat @ EXTERNAL:0000002d
<EXTERNAL> :: accept @ EXTERNAL:00000022
<EXTERNAL> :: bind @ EXTERNAL:00000046
<EXTERNAL> :: close @ EXTERNAL:00000014
<EXTERNAL> :: compress @ EXTERNAL:0000005f
<EXTERNAL> :: connect @ EXTERNAL:0000005a
<EXTERNAL> :: crypt @ EXTERNAL:00000060
<EXTERNAL> :: ctime @ EXTERNAL:0000004b
<EXTERNAL> :: exit @ EXTERNAL:00000052
<EXTERNAL> :: fclose @ EXTERNAL:0000004a
<EXTERNAL> :: fcntl @ EXTERNAL:00000047
<EXTERNAL> :: fflush @ EXTERNAL:00000017
<EXTERNAL> :: fgets @ EXTERNAL:00000032
<EXTERNAL> :: fopen @ EXTERNAL:0000005d
<EXTERNAL> :: fread @ EXTERNAL:00000029
<EXTERNAL> :: free @ EXTERNAL:00000056
<EXTERNAL> :: fseek @ EXTERNAL:0000002f
<EXTERNAL> :: ftell @ EXTERNAL:0000001f
<EXTERNAL> :: fwrite @ EXTERNAL:00000065
<EXTERNAL> :: getcwd @ EXTERNAL:00000058
<EXTERNAL> :: gethostbyname @ EXTERNAL:0000004f
<EXTERNAL> :: gettimeofday @ EXTERNAL:0000004c
<EXTERNAL> :: inet_addr @ EXTERNAL:0000003b
<EXTERNAL> :: inet_ntoa @ EXTERNAL:00000024
<EXTERNAL> :: listen @ EXTERNAL:00000026
<EXTERNAL> :: localeconv @ EXTERNAL:00000021
<EXTERNAL> :: malloc @ EXTERNAL:00000027
<EXTERNAL> :: memchr @ EXTERNAL:0000001e
<EXTERNAL> :: memcpy @ EXTERNAL:00000048
<EXTERNAL> :: memmove @ EXTERNAL:0000002a
<EXTERNAL> :: memset @ EXTERNAL:00000059
<EXTERNAL> :: mysql_error @ EXTERNAL:00000042
<EXTERNAL> :: mysql_fetch_row @ EXTERNAL:00000007
<EXTERNAL> :: mysql_free_result @ EXTERNAL:00000049
<EXTERNAL> :: mysql_init @ EXTERNAL:0000000e
<EXTERNAL> :: mysql_ping @ EXTERNAL:00000041
<EXTERNAL> :: mysql_query @ EXTERNAL:00000054
<EXTERNAL> :: mysql_real_connect @ EXTERNAL:0000005b
<EXTERNAL> :: mysql_store_result @ EXTERNAL:00000005
<EXTERNAL> :: operator.delete @ EXTERNAL:00000020
<EXTERNAL> :: operator.delete[] @ EXTERNAL:0000003f
<EXTERNAL> :: operator.new @ EXTERNAL:0000005c
<EXTERNAL> :: operator.new[] @ EXTERNAL:00000002
<EXTERNAL> :: printf @ EXTERNAL:00000045
<EXTERNAL> :: pthread_mutex_lock @ EXTERNAL:00000031
<EXTERNAL> :: pthread_mutex_unlock @ EXTERNAL:00000061
<EXTERNAL> :: rand @ EXTERNAL:00000067
<EXTERNAL> :: realloc @ EXTERNAL:00000043
<EXTERNAL> :: recv @ EXTERNAL:00000063
<EXTERNAL> :: recvfrom @ EXTERNAL:00000011
<EXTERNAL> :: select @ EXTERNAL:00000019
<EXTERNAL> :: send @ EXTERNAL:00000057
<EXTERNAL> :: sendto @ EXTERNAL:0000002b
<EXTERNAL> :: setsockopt @ EXTERNAL:0000002e
<EXTERNAL> :: signal @ EXTERNAL:00000016
<EXTERNAL> :: socket @ EXTERNAL:00000066
<EXTERNAL> :: sprintf @ EXTERNAL:00000064
<EXTERNAL> :: strcat @ EXTERNAL:00000044
<EXTERNAL> :: strchr @ EXTERNAL:00000009
<EXTERNAL> :: strcmp @ EXTERNAL:00000012
<EXTERNAL> :: strcpy @ EXTERNAL:00000068
<EXTERNAL> :: strdup @ EXTERNAL:0000000d
<EXTERNAL> :: strlen @ EXTERNAL:00000034
<EXTERNAL> :: strncmp @ EXTERNAL:00000039
```

_… 25 more in `imports.txt`_

## Exports / entry points

```
AddAccount @ 080782b0 [Function]
AddAccount @ 080782b0 [function]
AddAccountFromRC @ 080785c0 [Function]
AddAccountFromRC @ 080785c0 [function]
AddFileSeparators @ 0804d8f0 [Function]
AddFileSeparators @ 0804d8f0 [function]
AddLineEnds @ 0804d740 [Function]
AddLineEnds @ 0804d740 [function]
CfgRead @ 0804d240 [Function]
CfgRead @ 0804d240 [function]
CompressBuf @ 0807bc60 [Function]
CompressBuf @ 0807bc60 [function]
DecompressBuf @ 0807bd20 [Function]
DecompressBuf @ 0807bd20 [function]
DeleteAccount @ 08079760 [Function]
DeleteAccount @ 08079760 [function]
ExtractFilepath @ 0807d220 [Function]
ExtractFilepath @ 0807d220 [function]
FileExists @ 0807d1b0 [Function]
FileExists @ 0807d1b0 [function]
GetAccount @ 08077180 [Function]
GetAccount @ 08077180 [function]
GetAccountCount @ 08078d80 [Function]
GetAccountCount @ 08078d80 [function]
GetAccountsList @ 08078f20 [Function]
GetAccountsList @ 08078f20 [function]
GetAdminLevel @ 08079520 [Function]
GetAdminLevel @ 08079520 [function]
GetLevelFile @ 0804daa0 [Function]
GetLevelFile @ 0804daa0 [function]
GetProfile @ 0807a3d0 [Function]
GetProfile @ 0807a3d0 [function]
GetUTCFileModTime @ 0804df70 [Function]
GetUTCFileModTime @ 0804df70 [function]
GetWorlds @ 08079330 [Function]
GetWorlds @ 08079330 [function]
IsIPInList @ 0807b780 [Function]
IsIPInList @ 0807b780 [function]
JString::JString @ 0807bfc0 [Function]
JString::JString @ 0807bfc0 [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
080480f4	/lib/ld-linux.so.2
08049e0b	mysql_error
08049e51	mysql_real_connect
0804a063	_ZNSt15basic_streambufIcSt11char_traitsIcEE9pbackfailEi
0804a380	_ZSt19__throw_ios_failurePKc
0804abd6	_ZSt20__throw_length_errorPKc
0804b16d	_ZNSt12__basic_fileIcE9pbackfailEi
0804c01f	_ZNSt13basic_filebufIcSt11char_traitsIcEE9pbackfailEi
0804c172	printf
0804c190	connect
0804c1ef	socket
0804c228	bind
0804c280	listen
0804c2ee	sprintf
0804c335	fopen
0804c33b	__libc_start_main
0808362a	Server started. Port: %d
08083660	Response socket established at port 14899
08083692	data/serverflags.txt
080836a7	data/maxplayers.txt
080836c0	Couldn't bind the response socket
08083700	socket exception: cannot create socket
08083740	Port address %d is currently used by another programm. Try later!
080837a2	logs/rclog.txt
080837ed	logs/errorlog.txt
08083800	TServerFrame::CreatePlayer(): Checking ipbanned.txt...
08083838	data/ipbanned.txt
08083860	TServerFrame::CreatePlayer(): Client was IP Banned!
080838a0	TServerFrame::CreatePlayer(): GetFreePlayerID() returned %d!
08083920	TServerFrame::CreatePlayer(): Sent(UNLIMITEDSIG) = %s
08083960	There are too many players (28000) on this server.
08083993	RC disconnected:
080839be	levelnpcs/
08083a00	TServerPlayer::ClientRecieve(): buffer << sock->Read()
08083a40	TServerPlayer::ClientRecieve(): sock->error == %d
08083ac0	Your Graal.exe has sent to much data (>=128k in the input buffer)
08083b60	Your Graal.exe has sent an empty data package.
08083bab	The socket to client
08083c4e	The connection to client
08083c80	The connection to you was blocked for more than one minute (still
08083ce0	Error while sending data to client
08083d20	TServerPlayer::parse(): Sending Account...
08083d53	level13.graal
08083e20	"From jail you can only send PMs to admins (RCs)."
08083ee0	The server got uncertified data packages from your connection.
08083f2c	disconnects
08083f40	The server administrator has disconnected you.
08083f83	Player
08083fc0	The server got illegal data from your connection.
08084079	loaded attributes of player
080840af	from world
080840c0	loaded player attributes of account
0808413e	set attributes of player
08084160	set player attributes of account
080841bf	set profile error:
080841d3	warped player
0808433c	' has added an npc in
08084360	Adding npcs is not allowed in this level.
080843b0	Login error:
080843c0	The server got corrupt login data from your connection.
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `main_080690b0.c` | `main` | `080690b0` |
| `startServer_0804dff0.c` | `TServerFrame::startServer` | `0804dff0` |
| `VerifyGuildName_0807b320.c` | `VerifyGuildName` | `0807b320` |
| `connecttodb_08074820.c` | `connecttodb` | `08074820` |
| `Connect_0807f120.c` | `TSocket::Connect` | `0807f120` |
| `Bind_0807f240.c` | `TSocket::Bind` | `0807f240` |
| `Accept_0807f6b0.c` | `TSocket::Accept` | `0807f6b0` |
| `loginerror_0805cb30.c` | `TServerPlayer::loginerror` | `0805cb30` |
| `encryptpassword_0807f910.c` | `encryptpassword` | `0807f910` |
| `init_0807ff10.c` | `std::ios::init` | `0807ff10` |
| `_start_0804d110.c` | `_start` | `0804d110` |
| `init_dummy_0804d230.c` | `init_dummy` | `0804d230` |
| `init_dummy_0807fb90.c` | `init_dummy` | `0807fb90` |
| `parse_08052d00.c` | `TServerPlayer::parse` | `08052d00` |
| `SetProperties_08062090.c` | `TServerPlayer::SetProperties` | `08062090` |
| `SendAccount_0805cc40.c` | `TServerPlayer::SendAccount` | `0805cc40` |
| `parseChat_08065470.c` | `TServerPlayer::parseChat` | `08065470` |
| `LoadNW_0806c080.c` | `TServerLevel::LoadNW` | `0806c080` |
| `SetProperties_08071a10.c` | `TServerNPC::SetProperties` | `08071a10` |
| `enterLevel_0805f6a0.c` | `TServerPlayer::enterLevel` | `0805f6a0` |
| `GetProperty_08060a10.c` | `TServerPlayer::GetProperty` | `08060a10` |
| `SaveDBAccount_08075960.c` | `SaveDBAccount` | `08075960` |
| `LoadDBAccount_08074be0.c` | `LoadDBAccount` | `08074be0` |
| `SetAccount_08077640.c` | `SetAccount` | `08077640` |
| `GetProperty_08070ed0.c` | `TServerNPC::GetProperty` | `08070ed0` |
| `SetProfile_0807a7c0.c` | `SetProfile` | `0807a7c0` |
| `SetRCPlayerProps_0805c180.c` | `TServerPlayer::SetRCPlayerProps` | `0805c180` |
| `ApplyAccountChange_080675a0.c` | `TServerPlayer::ApplyAccountChange` | `080675a0` |
| `LoadWeapons_080767d0.c` | `LoadWeapons` | `080767d0` |
| `_JString_0807c240.c` | `JString::~JString` | `0807c240` |
| `operator__0807c600.c` | `JString::operator<<` | `0807c600` |
| `JString_0807bff0.c` | `JString::JString` | `0807bff0` |
| `clear_0807c260.c` | `JString::clear` | `0807c260` |
| `JString_0807c060.c` | `JString::JString` | `0807c060` |
| `JString_0807c160.c` | `JString::JString` | `0807c160` |
| `subJString_0807c460.c` | `JString::subJString` | `0807c460` |
| `text_0807c2b0.c` | `JString::text` | `0807c2b0` |
| `operator__0807d7c0.c` | `TList::operator[]` | `0807d7c0` |
| `operator+_0807ca40.c` | `operator+` | `0807ca40` |
| `operator__0807c740.c` | `JString::operator<<` | `0807c740` |

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

