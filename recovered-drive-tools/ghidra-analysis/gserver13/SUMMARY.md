# Salvage analysis: gserver13

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13`
- **Analyzed (UTC):** 20261031T204744Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped

# size
-rw-r--r-- 1 ubuntu ubuntu 870K Dec 22  1999 /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13

# sha256
be1c397d2c21049c7848a7c922455861ee0a4b413967c867c2e7efa90a7c9997  /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Executable and Linking Format (ELF) |
| Language | x86:LE:32:default |
| Compiler | gcc |
| Functions | 1128 (internal 915) |
| Imports | 106 |
| Exports / entries | 1673 |
| Interesting strings | 448 |
| Decompiled funcs | 40 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `parse`, `SetProperties`, `GetProperty`, `enterLevel`, `SendAccount`, `Timer1Timer`, `parseSetAccountProps`, `parseSetPlayerProps`, `ParseAccount`, `inflate_blocks`, `ReadWarpAccounts`, `ApplyAccountChange`, `ReadTempFile`, `main`, `inflate_codes`, `UpdateLevels`, `DeletePlayer`, `GetJStringRepresentation`, `Assign`, `parseWantPlayerProps`, `ReadBaddies`, `10TServerNPCRC7JStringT1dd`, `IsBannedIP`, `VerifyGuildName`, `parseWantAccountProps`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
<EXTERNAL> :: _IO_adjust_column
<EXTERNAL> :: _IO_default_doallocate
<EXTERNAL> :: _IO_default_finish
<EXTERNAL> :: _IO_default_pbackfail
<EXTERNAL> :: _IO_default_uflow
<EXTERNAL> :: _IO_default_xsgetn
<EXTERNAL> :: _IO_default_xsputn
<EXTERNAL> :: _IO_do_write
<EXTERNAL> :: _IO_doallocbuf
<EXTERNAL> :: _IO_file_attach
<EXTERNAL> :: _IO_file_close
<EXTERNAL> :: _IO_file_close_it
<EXTERNAL> :: _IO_file_doallocate
<EXTERNAL> :: _IO_file_fopen
<EXTERNAL> :: _IO_file_init
<EXTERNAL> :: _IO_file_open
<EXTERNAL> :: _IO_file_overflow
<EXTERNAL> :: _IO_file_read
<EXTERNAL> :: _IO_file_seek
<EXTERNAL> :: _IO_file_seekoff
<EXTERNAL> :: _IO_file_setbuf
<EXTERNAL> :: _IO_file_stat
<EXTERNAL> :: _IO_file_sync
<EXTERNAL> :: _IO_file_underflow
<EXTERNAL> :: _IO_file_write
<EXTERNAL> :: _IO_file_xsputn
<EXTERNAL> :: _IO_flockfile
<EXTERNAL> :: _IO_flush_all
<EXTERNAL> :: _IO_flush_all_linebuffered
<EXTERNAL> :: _IO_free_backup_area
<EXTERNAL> :: _IO_funlockfile
<EXTERNAL> :: _IO_getc
<EXTERNAL> :: _IO_getline_info
<EXTERNAL> :: _IO_init
<EXTERNAL> :: _IO_init_marker
<EXTERNAL> :: _IO_link_in
<EXTERNAL> :: _IO_marker_delta
<EXTERNAL> :: _IO_marker_difference
<EXTERNAL> :: _IO_padn
<EXTERNAL> :: _IO_peekc_locked
<EXTERNAL> :: _IO_putc
<EXTERNAL> :: _IO_remove_marker
<EXTERNAL> :: _IO_seekmark
<EXTERNAL> :: _IO_seekoff
<EXTERNAL> :: _IO_seekpos
<EXTERNAL> :: _IO_setb
<EXTERNAL> :: _IO_sgetn
<EXTERNAL> :: _IO_sputbackc
<EXTERNAL> :: _IO_sungetc
<EXTERNAL> :: _IO_switch_to_get_mode
<EXTERNAL> :: _IO_un_link
<EXTERNAL> :: _IO_unsave_markers
<EXTERNAL> :: _IO_vfscanf
<EXTERNAL> :: __bzero
<EXTERNAL> :: __errno_location
<EXTERNAL> :: __finite
<EXTERNAL> :: __finitef
<EXTERNAL> :: __finitel
<EXTERNAL> :: __gmon_start__
<EXTERNAL> :: __libc_start_main
<EXTERNAL> :: __printf_fp
<EXTERNAL> :: __underflow
<EXTERNAL> :: __xstat
<EXTERNAL> :: abort
<EXTERNAL> :: accept
<EXTERNAL> :: atof
<EXTERNAL> :: atol
<EXTERNAL> :: bind
<EXTERNAL> :: calloc
<EXTERNAL> :: close
<EXTERNAL> :: connect
<EXTERNAL> :: ctime
<EXTERNAL> :: exit
<EXTERNAL> :: fcntl
<EXTERNAL> :: fflush
<EXTERNAL> :: fprintf
<EXTERNAL> :: free
<EXTERNAL> :: getcwd
<EXTERNAL> :: gethostbyname
<EXTERNAL> :: gettimeofday
```

_… 26 more in `imports.txt`_

## Exports / entry points

```
.LFE141 [function]
.LFE20 [function]
.LFE23 [function]
Accept__7TSocket [Function]
Accept__7TSocket [function]
AccountExists__FRC7JString [Function]
AccountExists__FRC7JString [function]
AddModify__12TServerLevelP12TBoardModify [Function]
AddModify__12TServerLevelP12TBoardModify [function]
AddRowToComp__13TServerPlayer [Function]
AddRowToComp__13TServerPlayer [function]
AddTempAccount__FP12TTempAccount [Function]
AddTempAccount__FP12TTempAccount [function]
AddToList__10TServerNPCP13TServerPlayer [Function]
AddToList__10TServerNPCP13TServerPlayer [function]
AddToList__12TServerBaddyP13TServerPlayer [Function]
AddToList__12TServerBaddyP13TServerPlayer [function]
AddToList__12TServerLevelP13TServerPlayer [Function]
AddToList__12TServerLevelP13TServerPlayer [function]
AddToList__13TServerPlayerP13TServerPlayer [Function]
AddToList__13TServerPlayerP13TServerPlayer [function]
AddWeapon__FRC7JString [Function]
AddWeapon__FRC7JString [function]
Add__12TJStringListRC7JString [Function]
Add__12TJStringListRC7JString [function]
Add__5TListPv [Function]
Add__5TListPv [function]
ApplyAccountChange__13TServerPlayerP12TTempAccountP12TJStringList [Function]
ApplyAccountChange__13TServerPlayerP12TTempAccountP12TJStringList [function]
Assign__12TJStringListP12TJStringList [Function]
Assign__12TJStringListP12TJStringList [function]
Assign__16TServerLevelLinkG7JString [Function]
Assign__16TServerLevelLinkG7JString [function]
Bbase__9streambuf [Function]
Bbase__9streambuf [function]
Bind__7TSocketibT2 [Function]
Bind__7TSocketibT2 [function]
Bptr__9streambuf [Function]
Bptr__9streambuf [function]
Clear__12TJStringList [Function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
/lib/ld-linux.so.2
connect
socket
__printf_fp
fprintf
bind
listen
sprintf
__libc_start_main
_IO_file_fopen
_IO_default_pbackfail
socket exception: cannot create socket
Port address
Server started. Port:
Couldn't bind the response socket
Response socket established at port 14899
Couldn't bind port 143.
log/rclog.txt
errorlog.txt
levelnpcs/
.npcs
"The player cannot read the message because he/she
has an older Graal version."
disconnects
The server administrator has disconnected you.
accounts/acc
You need at least Graal version 1.3 to play on this server.
and password you typed in does not exist.
GR-V1.02
GR-V1.03
GR-V1.00
GR-V1.01
playerx
playery
player:
playermodify:
npc:
npcmodify:
has been disconnected because he didn't move
You have been disconnected because you didn't move.
players
Programm error. Contact Stefan Knorr!
onlinestartlocal.graal
accounts/acc
cgi-bin/newaccounts.txt
police2.graal
police4.graal
police5.graal
cgi-bin/warpaccounts.txt
cgi-bin/names.txt
guilds/guild
cgi-bin/ipbanned.txt
Decompress error:
deflate 1.1.3 Copyright 1995-1998 Jean-loup Gailly
incompatible version
buffer error
data error
stream error
file error
oversubscribed literal/length tree
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

