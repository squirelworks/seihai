# Salvage analysis: serverlist3

- **Binary:** `/workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3`
- **Analyzed (UTC):** 20261031T205116Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), statically linked, for GNU/Linux 2.2.5, not stripped, too many notes (256)

# size
-rw-r--r-- 1 ubuntu ubuntu 3.3M Sep  2  2017 /workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3

# sha256
eb5272ab1653f93fc00b41368caa52d7af565f16e3d575a18e4c1ba1e1da87c3  /workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Executable and Linking Format (ELF) |
| Language | x86:LE:32:default |
| Compiler | gcc |
| Functions | 2197 (internal 2197) |
| Imports | 0 |
| Exports / entries | 4486 |
| Interesting strings | 2375 |
| Decompiled funcs | 50 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `IO_vfscanf`, `strtold_internal`, `strtod_internal`, `strtof_internal`, `printf_fp`, `printf_fphex`, `dl_relocate_object`, `inflate_blocks`, `dl_map_object_deps`, `gconv_transform_utf8_internal`, `mysql_real_connect`, `res_vinit`, `dl_map_object_from_fd`, `SendAccountInfo`, `SavePlayerCount`, `StartServer`, `parse`, `do_dyncast`, `main`, `CheckCorrectIP`, `encrypt_r`, `do_upcast`, `authunix_create`, `Connect`, `parseLogin`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

_None exported (static binary, failed analysis, or no externals)._

## Exports / entry points

```
Accept__7TSocket [Function]
Accept__7TSocket [function]
AddLineEnds__FG7JString [Function]
AddLineEnds__FG7JString [function]
AddRowToComp__16TGraalConnection [Function]
AddRowToComp__16TGraalConnection [function]
Add__12TJStringListP12TJStringList [Function]
Add__12TJStringListP12TJStringList [function]
Add__12TJStringListRC7JString [Function]
Add__12TJStringListRC7JString [function]
Add__5TListPv [Function]
Add__5TListPv [function]
Assign__12TJStringListP12TJStringList [Function]
Assign__12TJStringListP12TJStringList [function]
Assign__5TListP5TList [Function]
Assign__5TListP5TList [function]
Bbase__9streambuf [Function]
Bbase__9streambuf [function]
Bind__7TSocketibT2 [Function]
Bind__7TSocketibT2 [function]
Bptr__9streambuf [Function]
Bptr__9streambuf [function]
CalcPlayerCount__Fv [Function]
CalcPlayerCount__Fv [function]
CheckActivatePlayer__Fv [Function]
CheckActivatePlayer__Fv [function]
CheckCorrectIP__12TGraalServerP12TJStringList [Function]
CheckCorrectIP__12TGraalServerP12TJStringList [function]
CheckPlayerLimit__Fv [Function]
CheckPlayerLimit__Fv [function]
Clear__12TJStringList [Function]
Clear__12TJStringList [function]
Clear__5TList [Function]
Clear__5TList [function]
Close__7TSocket [Function]
Close__7TSocket [function]
CompressBuf__FRC7JString [Function]
CompressBuf__FRC7JString [function]
Connect__7TSocketRC7JStringib [Function]
Connect__7TSocketRC7JStringib [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
clientsport
serversport
playerlimit
SELECT accname,encrpass,email,blocked,onlyload,adminlevel,adminworlds
,UNIX_TIMESTAMP(bantime),banreason,regends FROM accounts
SELECT name,language,description,homepage,version
,playercount,ip,port FROM serverlist
WHERE UNIX_TIMESTAMP(NOW())-UNIX_TIMESTAMP(lastwrite)<600
SELECT count(*) FROM accounts
SELECT ip FROM serverlist WHERE tablename='
not allowed to connect from
GServer for world
connected.
The password is wrong
Welcome to Graal!
Thanks for upgrading your account and playing Graal2001.
Thanks for buying Graal! You will have no limitations on the
socket exception: cannot create lister socket
Port address
, GServer port:
Server lister started. Lister port:
maxplayers.txt
playercount.txt
Players:
, Free players:
Players:
Playerlimit:
playerstats.txt
disconnected.
Configuraton not found.
Pleace specify the full path of the configuration file as parameter.
<public-url>
12TGraalClient
12TGraalServer
16TGraalConnection
connecting to ip
gserver
Can't connect to the db:
DB request failed:
prob: Couldn't delete account: permission denied
prob: Couldn't delete account:
<public-host>
deflate 1.1.3 Copyright 1995-1998 Jean-loup Gailly
incompatible version
buffer error
data error
stream error
file error
oversubscribed literal/length tree
incomplete literal/length tree
invalid literal/length code
invalid literal/length code
connect_timeout
interactive-timeout
default-character-set
character-set-dir
ssl-capath
ssl-ca
ssl-cert
ssl-key
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

