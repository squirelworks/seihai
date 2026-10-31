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

## Key functions

SendAccountInfo__12TGraalClientP12TJStringList@0804a9fc | SavePlayerCount__Fv@0804bd44 | StartServer__Fv@0804b694 | parse__12TGraalServerG7JString@08049b60 | do_dyncast__C17__class_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result@08076680 | main@0804cd30 | parse__12TGraalClientG7JString@0804a48c | CheckCorrectIP__12TGraalServerP12TJStringList@0804a238 | encrypt_r@0805cb78 | do_upcast__C17__class_type_infoQ216__user_type_info8sub_kindRC9type_infoPvRQ216__user_type_info13upcast_result@080764a0 | connect2@0805ce30 | authunix_create@080be8dc | Connect__7TSocketRC7JStringib@08051e28 | parseLogin__12TGraalServerG7JString@08049ff8 | init@080a0edc | connecttodb2__Fv@08052d58 | RequestServerList__FRC7JStringT0@08048d30 | RequestServerIP__FRC7JString@08048fd0 | Bind__7TSocketibT2@08052004 | authdes_getucred@08099720 | CheckActivatePlayer__Fv@080481e0 | Accept__7TSocket@08052620 | do_find_public_subobj__C17__class_type_infoiRC9type_infoPvT3@08076a10 | bindresvport@080cb8e4 | authunix_refresh@080beb58 | encryptpassword__FRC7JString@08053be0 | init_dynamic_string@08069570 | main_trim@0808f2d4 | CheckPlayerLimit__Fv@0804c7b8 | authnone_create@080cb7c4 | initstate_r@080a91a0 | do_dyncast__C14__si_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result@08076370 | authunix_validate@080bec68 | authunix_create_default@080beaa0 | dbthread_main__FPv@0805310c | init_reg_size_table@08073b80 | do_out@08089b30 | do_in@08089c40 | init_block@080576a4 | startsWith__FRC7JStringT0@0804f5f4 | do_unshift@08089bc0 | init_dynamic_array@08064310 | GetServerByWorldname__FRC7JString@08048a4c | do_upcast__C14__si_type_infoQ216__user_type_info8sub_kindRC9type_infoPvRQ216__user_type_info13upcast_result@080762f0 | do_dyncast__C16__user_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result@08076260 | do_length@08089ac4 | initdbthread__Fv@08052f2c | init_alloc_root@080631ac | do_find_public_subobj__C14__si_type_infoiRC9type_infoPvT3@08076430 | initstate@080a8f30

## Top functions

_IO_vfscanf@080817e0(sz=21491) | vfprintf@080b1fcc(sz=16562) | __strtold_internal@080adb34(sz=15506) | __strtod_internal@08079ea0(sz=15452) | vfwprintf@080d1240(sz=15340) | __strtof_internal@080aa274(sz=14203) | __printf_fp@0807f92c(sz=7352) | __printf_fphex@080b6884(sz=5302) | _dl_relocate_object@080cc164(sz=3991) | inflate_blocks@08058330(sz=3261) | _dl_map_object_deps@080cd2f8(sz=3209) | __gconv_transform_utf8_internal@080a4dfc(sz=3169) | mysql_real_connect@0805da9c(sz=3065) | __res_vinit@080bc1fc(sz=2951) | _dl_map_object_from_fd@0809c260(sz=2901)

## Sample interesting strings

```
080d99ba	clientsport
080d99c6	serversport
080d99d2	playerlimit
080d99e0	SELECT accname,encrpass,email,blocked,onlyload,adminlevel,adminworlds
080d9a40	,UNIX_TIMESTAMP(bantime),banreason,regends FROM accounts
080d9aa0	SELECT name,language,description,homepage,version
080d9ae0	,playercount,ip,port FROM serverlist
080d9b20	WHERE UNIX_TIMESTAMP(NOW())-UNIX_TIMESTAMP(lastwrite)<600
080d9b75	SELECT count(*) FROM accounts
080d9ba0	SELECT ip FROM serverlist WHERE tablename='
080d9bde	not allowed to connect from
080d9bfc	GServer for world
080d9c0f	connected.
080d9c21	The password is wrong
080d9c6d	Welcome to Graal!
080d9d00	Thanks for upgrading your account and playing Graal2001.
080d9da0	Thanks for buying Graal! You will have no limitations on the
080d9f40	socket exception: cannot create lister socket
080d9fb3	Port address
080d9fc1	, GServer port:
080d9fe0	Server lister started. Lister port:
080da005	maxplayers.txt
080da025	playercount.txt
080da042	Players:
080da07e	, Free players:
080da08f	Players:
080da099	Playerlimit:
080da0a7	playerstats.txt
080da0d4	disconnected.
080da0e3	Configuraton not found.
080da100	Pleace specify the full path of the configuration file as parameter.
080da156	http://www.graal2001.com
080da16f	12TGraalClient
080da17e	12TGraalServer
080da18d	16TGraalConnection
080da226	connecting to ip
080da240	gserver
080da251	Can't connect to the db:
080da277	DB request failed:
080da2a0	prob: Couldn't delete account: permission denied
```

## Decompiled

| File | Function | Address |
| --- | --- | --- |
| `SendAccountInfo__12TGraalClientP12TJStringList_0804a9fc.c` | `SendAccountInfo__12TGraalClientP12TJStringList` | `0804a9fc` |
| `SavePlayerCount__Fv_0804bd44.c` | `SavePlayerCount__Fv` | `0804bd44` |
| `StartServer__Fv_0804b694.c` | `StartServer__Fv` | `0804b694` |
| `parse__12TGraalServerG7JString_08049b60.c` | `parse__12TGraalServerG7JString` | `08049b60` |
| `do_dyncast__C17__class_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result_08076680.c` | `do_dyncast__C17__class_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result` | `08076680` |
| `main_0804cd30.c` | `main` | `0804cd30` |
| `parse__12TGraalClientG7JString_0804a48c.c` | `parse__12TGraalClientG7JString` | `0804a48c` |
| `CheckCorrectIP__12TGraalServerP12TJStringList_0804a238.c` | `CheckCorrectIP__12TGraalServerP12TJStringList` | `0804a238` |
| `encrypt_r_0805cb78.c` | `encrypt_r` | `0805cb78` |
| `do_upcast__C17__class_type_infoQ216__user_type_info8sub_kindRC9type_infoPvRQ216__user_type_info13upcast_result_080764a0.c` | `do_upcast__C17__class_type_infoQ216__user_type_info8sub_kindRC9type_infoPvRQ216__user_type_info13upcast_result` | `080764a0` |
| `connect2_0805ce30.c` | `connect2` | `0805ce30` |
| `authunix_create_080be8dc.c` | `authunix_create` | `080be8dc` |
| `Connect__7TSocketRC7JStringib_08051e28.c` | `Connect__7TSocketRC7JStringib` | `08051e28` |
| `parseLogin__12TGraalServerG7JString_08049ff8.c` | `parseLogin__12TGraalServerG7JString` | `08049ff8` |
| `init_080a0edc.c` | `init` | `080a0edc` |
| `connecttodb2__Fv_08052d58.c` | `connecttodb2__Fv` | `08052d58` |
| `RequestServerList__FRC7JStringT0_08048d30.c` | `RequestServerList__FRC7JStringT0` | `08048d30` |
| `RequestServerIP__FRC7JString_08048fd0.c` | `RequestServerIP__FRC7JString` | `08048fd0` |
| `Bind__7TSocketibT2_08052004.c` | `Bind__7TSocketibT2` | `08052004` |
| `authdes_getucred_08099720.c` | `authdes_getucred` | `08099720` |
| `CheckActivatePlayer__Fv_080481e0.c` | `CheckActivatePlayer__Fv` | `080481e0` |
| `Accept__7TSocket_08052620.c` | `Accept__7TSocket` | `08052620` |
| `do_find_public_subobj__C17__class_type_infoiRC9type_infoPvT3_08076a10.c` | `do_find_public_subobj__C17__class_type_infoiRC9type_infoPvT3` | `08076a10` |
| `bindresvport_080cb8e4.c` | `bindresvport` | `080cb8e4` |
| `authunix_refresh_080beb58.c` | `authunix_refresh` | `080beb58` |
| `encryptpassword__FRC7JString_08053be0.c` | `encryptpassword__FRC7JString` | `08053be0` |
| `init_dynamic_string_08069570.c` | `init_dynamic_string` | `08069570` |
| `main_trim_0808f2d4.c` | `main_trim` | `0808f2d4` |
| `CheckPlayerLimit__Fv_0804c7b8.c` | `CheckPlayerLimit__Fv` | `0804c7b8` |
| `authnone_create_080cb7c4.c` | `authnone_create` | `080cb7c4` |
| `initstate_r_080a91a0.c` | `initstate_r` | `080a91a0` |
| `do_dyncast__C14__si_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result_08076370.c` | `do_dyncast__C14__si_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result` | `08076370` |
| `authunix_validate_080bec68.c` | `authunix_validate` | `080bec68` |
| `authunix_create_default_080beaa0.c` | `authunix_create_default` | `080beaa0` |
| `dbthread_main__FPv_0805310c.c` | `dbthread_main__FPv` | `0805310c` |
| `init_reg_size_table_08073b80.c` | `init_reg_size_table` | `08073b80` |
| `do_out_08089b30.c` | `do_out` | `08089b30` |
| `do_in_08089c40.c` | `do_in` | `08089c40` |
| `init_block_080576a4.c` | `init_block` | `080576a4` |
| `startsWith__FRC7JStringT0_0804f5f4.c` | `startsWith__FRC7JStringT0` | `0804f5f4` |
| `do_unshift_08089bc0.c` | `do_unshift` | `08089bc0` |
| `init_dynamic_array_08064310.c` | `init_dynamic_array` | `08064310` |
| `GetServerByWorldname__FRC7JString_08048a4c.c` | `GetServerByWorldname__FRC7JString` | `08048a4c` |
| `do_upcast__C14__si_type_infoQ216__user_type_info8sub_kindRC9type_infoPvRQ216__user_type_info13upcast_result_080762f0.c` | `do_upcast__C14__si_type_infoQ216__user_type_info8sub_kindRC9type_infoPvRQ216__user_type_info13upcast_result` | `080762f0` |
| `do_dyncast__C16__user_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result_08076260.c` | `do_dyncast__C16__user_type_infoiQ216__user_type_info8sub_kindRC9type_infoPvT3T4RQ216__user_type_info14dyncast_result` | `08076260` |
| `do_length_08089ac4.c` | `do_length` | `08089ac4` |
| `initdbthread__Fv_08052f2c.c` | `initdbthread__Fv` | `08052f2c` |
| `init_alloc_root_080631ac.c` | `init_alloc_root` | `080631ac` |
| `do_find_public_subobj__C14__si_type_infoiRC9type_infoPvT3_08076430.c` | `do_find_public_subobj__C14__si_type_infoiRC9type_infoPvT3` | `08076430` |
| `initstate_080a8f30.c` | `initstate` | `080a8f30` |

## Imports (head)

```
```

