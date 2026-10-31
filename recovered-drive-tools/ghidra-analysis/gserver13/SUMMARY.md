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

## Key functions

parse__13TServerPlayerRC7JString@0804fb3c | SetProperties__13TServerPlayerG7JString@08061088 | GetProperty__13TServerPlayeri@0805e7bc | enterLevel__13TServerPlayerG7JStringddT1@0805c160 | SendAccount__13TServerPlayerRC7JString@0805a1a0 | Timer1Timer__12TServerFrame@08070e24 | parseSetAccountProps__13TServerPlayerG7JString@080570fc | GetProperty__10TServerNPCi@0806bdec | SetProperties__10TServerNPCG7JString@0806d528 | parseSetPlayerProps__13TServerPlayerG7JString@08058ecc | ParseAccount__FG7JString@08078f7c | inflate_blocks@08082690 | ReadWarpAccounts__Fv@0807b08c | ApplyAccountChange__13TServerPlayerP12TTempAccountP12TJStringList@08064e44 | ReadTempFile__12TServerLevel@08067330 | main@08072be4 | inflate_codes@08083e80 | UpdateLevels__12TServerFrameP12TJStringList@0804db0c | DeletePlayer__12TServerFrameP13TServerPlayer@0804d054 | GetJStringRepresentation__16TServerLevelLink@0806a720 | Assign__16TServerLevelLinkG7JString@08069cc4 | parseWantPlayerProps__13TServerPlayerG7JString@080586f4 | GetProperty__12TServerBaddyi@0806ef7c | ReadBaddies__12TServerLevelP7TStream@08068340 | __10TServerNPCRC7JStringT1dd@0806b2c0 | IsBannedIP__FP7TSocket@0807d5dc | VerifyGuildName__FG7JStringRC7JStringT1@0807c578 | parseWantAccountProps__13TServerPlayerG7JString@08056a60 | dumpCount__Fv@08070860 | ReadNewAccounts__Fv@0807a740 | parseBoardModify__13TServerPlayerG7JString@08055eb0 | SetProperties__12TServerBaddyG7JString@0806f964 | SaveAccount__FP12TTempAccountRC7JString@080783b0 | InSameGuild__FRC7JStringT0@0807ce38 | __13TServerPlayerP7TSocketi@0804e7cc | __throw_type_match_rtti@0808d1d8 | parseNewNPC__13TServerPlayerG7JString@08056454 | LoadAccounts__Fv@08077d3c | startServer__12TServerFrameRC7JString@0804bcb0 | Load__12TServerLevel@08066e44

## Top functions

parse__13TServerPlayerRC7JString@0804fb3c(sz=19512) | SetProperties__13TServerPlayerG7JString@08061088(sz=10997) | GetProperty__13TServerPlayeri@0805e7bc(sz=6840) | enterLevel__13TServerPlayerG7JStringddT1@0805c160(sz=6414) | SendAccount__13TServerPlayerRC7JString@0805a1a0(sz=5909) | Timer1Timer__12TServerFrame@08070e24(sz=5206) | parseSetAccountProps__13TServerPlayerG7JString@080570fc(sz=4260) | GetProperty__10TServerNPCi@0806bdec(sz=3834) | SetProperties__10TServerNPCG7JString@0806d528(sz=3719) | parseSetPlayerProps__13TServerPlayerG7JString@08058ecc(sz=3637) | ParseAccount__FG7JString@08078f7c(sz=3408) | inflate_blocks@08082690(sz=3261) | ReadWarpAccounts__Fv@0807b08c(sz=2667) | ApplyAccountChange__13TServerPlayerP12TTempAccountP12TJStringList@08064e44(sz=2647) | ReadTempFile__12TServerLevel@08067330(sz=2207)

## Sample interesting strings

```
080480f4	/lib/ld-linux.so.2
080498ae	connect
08049a53	socket
08049b06	__printf_fp
08049ba4	fprintf
08049bca	bind
08049cb6	listen
08049d96	sprintf
08049e96	__libc_start_main
08049ee2	_IO_file_fopen
08049f28	_IO_default_pbackfail
0808f660	socket exception: cannot create socket
0808f6d3	Port address
0808f6e1	Server started. Port:
0808f720	Couldn't bind the response socket
0808f760	Response socket established at port 14899
0808f78a	Couldn't bind port 143.
0808f7c7	log/rclog.txt
0808f7de	errorlog.txt
0808f7f8	levelnpcs/
0808f803	.npcs
0808f880	"The player cannot read the message because he/she
0808f8b3	has an older Graal version."
0808f8e2	disconnects
0808f900	The server administrator has disconnected you.
0808fa9a	accounts/acc
0808fb20	You need at least Graal version 1.3 to play on this server.
0808fb80	and password you typed in does not exist.
0808fdf5	GR-V1.02
0808fdfe	GR-V1.03
0808fe07	GR-V1.00
0808fe10	GR-V1.01
0808fe43	playerx
0808fe4b	playery
08090024	player:
0809002d	playermodify:
0809007c	npc:
08090082	npcmodify:
08090100	has been disconnected because he didn't move
08090140	You have been disconnected because you didn't move.
```

## Decompiled

| File | Function | Address |
| --- | --- | --- |
| `parse__13TServerPlayerRC7JString_0804fb3c.c` | `parse__13TServerPlayerRC7JString` | `0804fb3c` |
| `SetProperties__13TServerPlayerG7JString_08061088.c` | `SetProperties__13TServerPlayerG7JString` | `08061088` |
| `GetProperty__13TServerPlayeri_0805e7bc.c` | `GetProperty__13TServerPlayeri` | `0805e7bc` |
| `enterLevel__13TServerPlayerG7JStringddT1_0805c160.c` | `enterLevel__13TServerPlayerG7JStringddT1` | `0805c160` |
| `SendAccount__13TServerPlayerRC7JString_0805a1a0.c` | `SendAccount__13TServerPlayerRC7JString` | `0805a1a0` |
| `Timer1Timer__12TServerFrame_08070e24.c` | `Timer1Timer__12TServerFrame` | `08070e24` |
| `parseSetAccountProps__13TServerPlayerG7JString_080570fc.c` | `parseSetAccountProps__13TServerPlayerG7JString` | `080570fc` |
| `GetProperty__10TServerNPCi_0806bdec.c` | `GetProperty__10TServerNPCi` | `0806bdec` |
| `SetProperties__10TServerNPCG7JString_0806d528.c` | `SetProperties__10TServerNPCG7JString` | `0806d528` |
| `parseSetPlayerProps__13TServerPlayerG7JString_08058ecc.c` | `parseSetPlayerProps__13TServerPlayerG7JString` | `08058ecc` |
| `ParseAccount__FG7JString_08078f7c.c` | `ParseAccount__FG7JString` | `08078f7c` |
| `inflate_blocks_08082690.c` | `inflate_blocks` | `08082690` |
| `ReadWarpAccounts__Fv_0807b08c.c` | `ReadWarpAccounts__Fv` | `0807b08c` |
| `ApplyAccountChange__13TServerPlayerP12TTempAccountP12TJStringList_08064e44.c` | `ApplyAccountChange__13TServerPlayerP12TTempAccountP12TJStringList` | `08064e44` |
| `ReadTempFile__12TServerLevel_08067330.c` | `ReadTempFile__12TServerLevel` | `08067330` |
| `main_08072be4.c` | `main` | `08072be4` |
| `inflate_codes_08083e80.c` | `inflate_codes` | `08083e80` |
| `UpdateLevels__12TServerFrameP12TJStringList_0804db0c.c` | `UpdateLevels__12TServerFrameP12TJStringList` | `0804db0c` |
| `DeletePlayer__12TServerFrameP13TServerPlayer_0804d054.c` | `DeletePlayer__12TServerFrameP13TServerPlayer` | `0804d054` |
| `GetJStringRepresentation__16TServerLevelLink_0806a720.c` | `GetJStringRepresentation__16TServerLevelLink` | `0806a720` |
| `Assign__16TServerLevelLinkG7JString_08069cc4.c` | `Assign__16TServerLevelLinkG7JString` | `08069cc4` |
| `parseWantPlayerProps__13TServerPlayerG7JString_080586f4.c` | `parseWantPlayerProps__13TServerPlayerG7JString` | `080586f4` |
| `GetProperty__12TServerBaddyi_0806ef7c.c` | `GetProperty__12TServerBaddyi` | `0806ef7c` |
| `ReadBaddies__12TServerLevelP7TStream_08068340.c` | `ReadBaddies__12TServerLevelP7TStream` | `08068340` |
| `__10TServerNPCRC7JStringT1dd_0806b2c0.c` | `__10TServerNPCRC7JStringT1dd` | `0806b2c0` |
| `IsBannedIP__FP7TSocket_0807d5dc.c` | `IsBannedIP__FP7TSocket` | `0807d5dc` |
| `VerifyGuildName__FG7JStringRC7JStringT1_0807c578.c` | `VerifyGuildName__FG7JStringRC7JStringT1` | `0807c578` |
| `parseWantAccountProps__13TServerPlayerG7JString_08056a60.c` | `parseWantAccountProps__13TServerPlayerG7JString` | `08056a60` |
| `dumpCount__Fv_08070860.c` | `dumpCount__Fv` | `08070860` |
| `ReadNewAccounts__Fv_0807a740.c` | `ReadNewAccounts__Fv` | `0807a740` |
| `parseBoardModify__13TServerPlayerG7JString_08055eb0.c` | `parseBoardModify__13TServerPlayerG7JString` | `08055eb0` |
| `SetProperties__12TServerBaddyG7JString_0806f964.c` | `SetProperties__12TServerBaddyG7JString` | `0806f964` |
| `SaveAccount__FP12TTempAccountRC7JString_080783b0.c` | `SaveAccount__FP12TTempAccountRC7JString` | `080783b0` |
| `InSameGuild__FRC7JStringT0_0807ce38.c` | `InSameGuild__FRC7JStringT0` | `0807ce38` |
| `__13TServerPlayerP7TSocketi_0804e7cc.c` | `__13TServerPlayerP7TSocketi` | `0804e7cc` |
| `__throw_type_match_rtti_0808d1d8.c` | `__throw_type_match_rtti` | `0808d1d8` |
| `parseNewNPC__13TServerPlayerG7JString_08056454.c` | `parseNewNPC__13TServerPlayerG7JString` | `08056454` |
| `LoadAccounts__Fv_08077d3c.c` | `LoadAccounts__Fv` | `08077d3c` |
| `startServer__12TServerFrameRC7JString_0804bcb0.c` | `startServer__12TServerFrameRC7JString` | `0804bcb0` |
| `Load__12TServerLevel_08066e44.c` | `Load__12TServerLevel` | `08066e44` |

## Imports (head)

```
<EXTERNAL> :: _IO_adjust_column @ EXTERNAL:00000035
<EXTERNAL> :: _IO_default_doallocate @ EXTERNAL:00000005
<EXTERNAL> :: _IO_default_finish @ EXTERNAL:00000004
<EXTERNAL> :: _IO_default_pbackfail @ EXTERNAL:00000065
<EXTERNAL> :: _IO_default_uflow @ EXTERNAL:00000043
<EXTERNAL> :: _IO_default_xsgetn @ EXTERNAL:00000008
<EXTERNAL> :: _IO_default_xsputn @ EXTERNAL:00000011
<EXTERNAL> :: _IO_do_write @ EXTERNAL:00000029
<EXTERNAL> :: _IO_doallocbuf @ EXTERNAL:00000055
<EXTERNAL> :: _IO_file_attach @ EXTERNAL:00000060
<EXTERNAL> :: _IO_file_close @ EXTERNAL:00000015
<EXTERNAL> :: _IO_file_close_it @ EXTERNAL:00000020
<EXTERNAL> :: _IO_file_doallocate @ EXTERNAL:0000004c
<EXTERNAL> :: _IO_file_fopen @ EXTERNAL:00000062
<EXTERNAL> :: _IO_file_init @ EXTERNAL:00000024
<EXTERNAL> :: _IO_file_open @ EXTERNAL:00000050
<EXTERNAL> :: _IO_file_overflow @ EXTERNAL:00000025
<EXTERNAL> :: _IO_file_read @ EXTERNAL:0000004d
<EXTERNAL> :: _IO_file_seek @ EXTERNAL:0000000b
<EXTERNAL> :: _IO_file_seekoff @ EXTERNAL:0000001d
<EXTERNAL> :: _IO_file_setbuf @ EXTERNAL:0000003b
<EXTERNAL> :: _IO_file_stat @ EXTERNAL:00000028
<EXTERNAL> :: _IO_file_sync @ EXTERNAL:0000001f
<EXTERNAL> :: _IO_file_underflow @ EXTERNAL:00000033
<EXTERNAL> :: _IO_file_write @ EXTERNAL:00000064
<EXTERNAL> :: _IO_file_xsputn @ EXTERNAL:0000005b
<EXTERNAL> :: _IO_flockfile @ EXTERNAL:00000013
<EXTERNAL> :: _IO_flush_all @ EXTERNAL:00000034
<EXTERNAL> :: _IO_flush_all_linebuffered @ EXTERNAL:0000000e
<EXTERNAL> :: _IO_free_backup_area @ EXTERNAL:00000040
<EXTERNAL> :: _IO_funlockfile @ EXTERNAL:0000000f
<EXTERNAL> :: _IO_getc @ EXTERNAL:0000003d
<EXTERNAL> :: _IO_getline_info @ EXTERNAL:00000032
<EXTERNAL> :: _IO_init @ EXTERNAL:0000005e
<EXTERNAL> :: _IO_init_marker @ EXTERNAL:0000000c
<EXTERNAL> :: _IO_link_in @ EXTERNAL:00000002
<EXTERNAL> :: _IO_marker_delta @ EXTERNAL:00000038
<EXTERNAL> :: _IO_marker_difference @ EXTERNAL:0000003f
<EXTERNAL> :: _IO_padn @ EXTERNAL:00000031
<EXTERNAL> :: _IO_peekc_locked @ EXTERNAL:0000004e
```

