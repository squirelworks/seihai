# Salvage analysis: gserver13

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13`
- **Ghidra language:** x86:LE:32:default / gcc
- **Format:** Executable and Linking Format (ELF)
- **Functions:** 1128 (internal 915)
- **Imports / exports:** 106 / 1673
- **Interesting strings / decompiled:** 448 / 40

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped

# size
-rw-r--r-- 1 ubuntu ubuntu 870K Dec 22  1999 /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13

# sha256
be1c397d2c21049c7848a7c922455861ee0a4b413967c867c2e7efa90a7c9997  /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserver13
```

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `parse`, `SetProperties`, `GetProperty`, `enterLevel`, `SendAccount`, `Timer1Timer`, `parseSetAccountProps`, `parseSetPlayerProps`, `ParseAccount`, `inflate_blocks`, `ReadWarpAccounts`, `ApplyAccountChange`, `ReadTempFile`, `main`, `inflate_codes`, `UpdateLevels`, `DeletePlayer`, `GetJStringRepresentation`, `Assign`, `parseWantPlayerProps`, `ReadBaddies`, `10TServerNPCRC7JStringT1dd`, `IsBannedIP`, `VerifyGuildName`, `parseWantAccountProps`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `parse`, `SetProperties`, `GetProperty`, `enterLevel`, `SendAccount`, `Timer1Timer`, `parseSetAccountProps`, `parseSetPlayerProps`, `ParseAccount`, `inflate_blocks`, `ReadWarpAccounts`, `ApplyAccountChange`, `ReadTempFile`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Sample interesting strings

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
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

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
```

