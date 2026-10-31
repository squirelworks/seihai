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

## Key functions

main@080690b0 | startServer@0804dff0 | VerifyGuildName@0807b320 | connecttodb@08074820 | Connect@0807f120 | Bind@0807f240 | Accept@0807f6b0 | loginerror@0805cb30 | encryptpassword@0807f910 | init@0807ff10 | _start@0804d110 | init_dummy@0804d230 | init_dummy@0807fb90 | parse@08052d00 | SetProperties@08062090 | SendAccount@0805cc40 | parseChat@08065470 | LoadNW@0806c080 | SetProperties@08071a10 | enterLevel@0805f6a0 | GetProperty@08060a10 | SaveDBAccount@08075960 | LoadDBAccount@08074be0 | SetAccount@08077640 | GetProperty@08070ed0 | SetProfile@0807a7c0 | SetRCPlayerProps@0805c180 | ApplyAccountChange@080675a0 | LoadWeapons@080767d0 | ~JString@0807c240 | operator<<@0807c600 | JString@0807bff0 | clear@0807c260 | JString@0807c060 | JString@0807c160 | subJString@0807c460 | text@0807c2b0 | operator[]@0807d7c0 | operator+@0807ca40 | operator<<@0807c740

## Top functions

parse@08052d00(sz=30008) | SetProperties@08062090(sz=11379) | SendAccount@0805cc40(sz=7486) | parseChat@08065470(sz=6203) | LoadNW@0806c080(sz=4759) | SetProperties@08071a10(sz=4654) | enterLevel@0805f6a0(sz=3533) | GetProperty@08060a10(sz=3202) | SaveDBAccount@08075960(sz=3030) | LoadDBAccount@08074be0(sz=2882) | SetAccount@08077640(sz=2716) | GetProperty@08070ed0(sz=2691) | SetProfile@0807a7c0(sz=2339) | SetRCPlayerProps@0805c180(sz=2245) | main@080690b0(sz=1935)

## Sample interesting strings

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
```

## Decompiled

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

## Imports (head)

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
```

