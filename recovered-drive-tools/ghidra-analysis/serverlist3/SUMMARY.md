# Salvage analysis: serverlist3

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3`
- **Ghidra language:** x86:LE:32:default / gcc
- **Format:** Executable and Linking Format (ELF)
- **Functions:** 2197 (internal 2197)
- **Imports / exports:** 0 / 4486
- **Interesting strings / decompiled:** 2375 / 50

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), statically linked, for GNU/Linux 2.2.5, not stripped, too many notes (256)

# size
-rw-r--r-- 1 ubuntu ubuntu 3.3M Sep  2  2017 /workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3

# sha256
eb5272ab1653f93fc00b41368caa52d7af565f16e3d575a18e4c1ba1e1da87c3  /workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3
```

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `SendAccountInfo`, `SavePlayerCount`, `StartServer`, `parse`, `do_dyncast`, `main`, `CheckCorrectIP`, `encrypt_r`, `do_upcast`, `authunix_create`, `Connect`, `parseLogin`, `init`, `connecttodb2`, `RequestServerList`, `RequestServerIP`, `Bind`, `authdes_getucred`, `CheckActivatePlayer`, `Accept`, `do_find_public_subobj`, `authunix_refresh`, `encryptpassword`, `init_dynamic_string`, `main_trim`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `IO_vfscanf`, `strtold_internal`, `strtod_internal`, `strtof_internal`, `printf_fp`, `printf_fphex`, `dl_relocate_object`, `inflate_blocks`, `dl_map_object_deps`, `gconv_transform_utf8_internal`, `mysql_real_connect`, `res_vinit`, `dl_map_object_from_fd`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Sample interesting strings

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
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

```
```

