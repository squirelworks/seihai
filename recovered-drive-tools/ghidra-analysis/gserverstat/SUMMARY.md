# Salvage analysis: gserverstat

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat`
- **Analyzed (UTC):** 20261031T205919Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

Offline **HTML leaderboard generator** for classic gserver account stats. Companion shell `statmaker` loops argv modes (`kills`, `deaths`, `rupees`, `highap`, `lowap`, `onlinetime`) and redirects stdout HTML into a htdocs `stats/` tree. Binary is egcs-era ELF32, **not stripped**, sharing account-parse helpers with classic `gserver` (`LoadAccounts`, `ParseAccount`, `SetProperties`). Reads `accounts/acc*`, emits green-themed HTML tables. Not a network service.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `analysis-only` |
| --- | --- |
| Why | Legacy gserver-adjacent account dump tooling; behaviour notes only — never paste into MIT crates. |
| Not | `setup-core` / `lat-core` / `seihai` / `seihai-rc` |
| Related | `_inbox/extracted/gserver_source/statmaker`, `gserverstats.pl` (Server Lister) |
| Licence risk | Legacy/GPL-adjacent gserver-era C++ — clean-room notes only |

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped

# size
-rw-r--r-- 1 ubuntu ubuntu 513K Dec 25  1999 /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat

# sha256
b8ae1a13147567d6544fb30ab383359e6c1c19849deb9d87d219f7cf90af867d  /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Executable and Linking Format (ELF) |
| Language | x86:LE:32:default |
| Compiler | gcc |
| Image base | 08048000 |
| Functions | 899 (internal 744) |
| Imports | 77 |
| Exports / entries | 1296 |
| Interesting strings | 223 |
| Decompiled funcs | 50 |

## Top / key functions

**Largest internal:** SetProperties__13TServerPlayerG7JString@0804bc18(sz=4347) | main@0804da2c(sz=4212) | ParseAccount__FG7JString@080538bc(sz=1910) | LoadAccounts__Fv@080525ac(sz=1232) | SaveAccount__FP12TTempAccountG7JString@08052ccc(sz=1186) | SaveAccounts__Fv@080532a0(sz=1160) | __throw_type_match_rtti@0805d8e8(sz=1156) | LoadAccount__14TServerAccount@08051420(sz=1023) | __13TServerPlayer@0804b4e0(sz=1014) | ReadNewAccounts__Fv@08054790(sz=993) | RemoveLineEnds__FG7JString@0804ac70(sz=959) | throw_helper@0805b244(sz=864) | ParseAccount2__FG7JString@0805428c(sz=737) | read_int__FR7istreamRUxRi@08056c44(sz=714) | write_int__FR7ostreamUxi@080575c8(sz=674)

**Decompile targets:** main@0804da2c | init__3iosP9streambufP7ostream@0805a558 | do_osfx__7ostream@080586a4 | entry@0804a6d0 | init__7filebuf@0805a640 | init_dummy@0804a788 | init_dummy@0805e4f8 | SetProperties__13TServerPlayerG7JString@0804bc18 | ParseAccount__FG7JString@080538bc | LoadAccounts__Fv@080525ac | SaveAccount__FP12TTempAccountG7JString@08052ccc | SaveAccounts__Fv@080532a0 | LoadAccount__14TServerAccount@08051420 | ReadNewAccounts__Fv@08054790 | RemoveLineEnds__FG7JString@0804ac70 | throw_helper@0805b244 | ParseAccount2__FG7JString@0805428c | read_int__FR7istreamRUxRi@08056c44 | write_int__FR7ostreamUxi@080575c8 | AddTempAccount__FP12TTempAccount@08052238 | LoadFromFile__7TStreamRC7JString@08050964 | GetClosed__12TServerChest@0804d364 | SaveAccountNames__Fv@08054d70 | end_fde_sort@0805c288 | GetIDAsJString__14TServerAccount@08050f84 | getline__7istreamPcic@080590e0 | RemoveExtension__FG7JString@0804a970 | SetCommaText__12TJStringListG7JString@08050658 | extract_cie_info@0805c6f4 | dcast__C17__class_type_infoRC9type_infoiPvPC9type_infoT3@0805d544 | AddWeapon__FRC7JString@0805205c | GetOpened__12TServerChest@0804d728 | _._7JString@0805e510 | flush__7ostream@08058394 | length__C7JString@0804f1d8 | Length__FRC7JString@0804f9d4 | _skip_ws__7istream@080584b8 | text__C7JString@0804f200 | SaveToFile__7TStreamRC7JString@08050c60 | _._9type_info@0805d364 | LowerCaseFilename__FG7JString@0804a7a0 | subJString__C7JStringii@0804f3ec | LowerCase__FRC7JString@0804faec | Copy__FRC7JStringii@0804fab8 | Add__12TJStringListG7JString@08050328 | FileExists__FRC7JString@08055164 | eh_context_initialize@0805ac74 | getGifNumber__FG7JStringRC7JString@0804b284 | GetWeaponName__FRC7JString@08051f30 | read__7istreamPci@0805684c

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
<EXTERNAL> :: _IO_adjust_column @ EXTERNAL:00000027
<EXTERNAL> :: _IO_default_doallocate @ EXTERNAL:00000005
<EXTERNAL> :: _IO_default_finish @ EXTERNAL:00000004
<EXTERNAL> :: _IO_default_pbackfail @ EXTERNAL:00000049
<EXTERNAL> :: _IO_default_uflow @ EXTERNAL:0000002f
<EXTERNAL> :: _IO_default_xsgetn @ EXTERNAL:00000006
<EXTERNAL> :: _IO_default_xsputn @ EXTERNAL:0000000e
<EXTERNAL> :: _IO_do_write @ EXTERNAL:0000001f
<EXTERNAL> :: _IO_doallocbuf @ EXTERNAL:0000003b
<EXTERNAL> :: _IO_file_attach @ EXTERNAL:00000045
<EXTERNAL> :: _IO_file_close @ EXTERNAL:00000012
<EXTERNAL> :: _IO_file_close_it @ EXTERNAL:00000019
<EXTERNAL> :: _IO_file_doallocate @ EXTERNAL:00000034
<EXTERNAL> :: _IO_file_fopen @ EXTERNAL:00000046
<EXTERNAL> :: _IO_file_init @ EXTERNAL:0000001c
<EXTERNAL> :: _IO_file_open @ EXTERNAL:00000038
<EXTERNAL> :: _IO_file_overflow @ EXTERNAL:0000001d
<EXTERNAL> :: _IO_file_read @ EXTERNAL:00000035
<EXTERNAL> :: _IO_file_seek @ EXTERNAL:00000009
<EXTERNAL> :: _IO_file_seekoff @ EXTERNAL:00000016
<EXTERNAL> :: _IO_file_setbuf @ EXTERNAL:0000002a
<EXTERNAL> :: _IO_file_stat @ EXTERNAL:0000001e
<EXTERNAL> :: _IO_file_sync @ EXTERNAL:00000018
<EXTERNAL> :: _IO_file_underflow @ EXTERNAL:00000025
<EXTERNAL> :: _IO_file_write @ EXTERNAL:00000048
<EXTERNAL> :: _IO_file_xsputn @ EXTERNAL:00000041
<EXTERNAL> :: _IO_flockfile @ EXTERNAL:00000010
<EXTERNAL> :: _IO_flush_all @ EXTERNAL:00000026
<EXTERNAL> :: _IO_flush_all_linebuffered @ EXTERNAL:0000000b
<EXTERNAL> :: _IO_free_backup_area @ EXTERNAL:0000002d
<EXTERNAL> :: _IO_funlockfile @ EXTERNAL:0000000c
<EXTERNAL> :: _IO_getc @ EXTERNAL:0000002b
<EXTERNAL> :: _IO_getline_info @ EXTERNAL:00000024
<EXTERNAL> :: _IO_init @ EXTERNAL:00000044
<EXTERNAL> :: _IO_init_marker @ EXTERNAL:0000000a
<EXTERNAL> :: _IO_link_in @ EXTERNAL:00000002
<EXTERNAL> :: _IO_marker_delta @ EXTERNAL:00000028
<EXTERNAL> :: _IO_marker_difference @ EXTERNAL:0000002c
<EXTERNAL> :: _IO_padn @ EXTERNAL:00000023
<EXTERNAL> :: _IO_peekc_locked @ EXTERNAL:00000036
<EXTERNAL> :: _IO_putc @ EXTERNAL:0000003f
<EXTERNAL> :: _IO_remove_marker @ EXTERNAL:00000001
<EXTERNAL> :: _IO_seekmark @ EXTERNAL:0000004b
<EXTERNAL> :: _IO_seekoff @ EXTERNAL:0000003e
<EXTERNAL> :: _IO_seekpos @ EXTERNAL:00000040
<EXTERNAL> :: _IO_setb @ EXTERNAL:00000032
<EXTERNAL> :: _IO_sgetn @ EXTERNAL:00000008
<EXTERNAL> :: _IO_sputbackc @ EXTERNAL:00000003
<EXTERNAL> :: _IO_sungetc @ EXTERNAL:0000000f
<EXTERNAL> :: _IO_switch_to_get_mode @ EXTERNAL:00000039
<EXTERNAL> :: _IO_un_link @ EXTERNAL:00000047
<EXTERNAL> :: _IO_unsave_markers @ EXTERNAL:00000014
<EXTERNAL> :: _IO_vfscanf @ EXTERNAL:0000003a
<EXTERNAL> :: __errno_location @ EXTERNAL:0000003c
<EXTERNAL> :: __finite @ EXTERNAL:00000022
<EXTERNAL> :: __finitef @ EXTERNAL:00000015
<EXTERNAL> :: __finitel @ EXTERNAL:0000004a
<EXTERNAL> :: __gmon_start__ @ EXTERNAL:0000004d
<EXTERNAL> :: __libc_start_main @ EXTERNAL:00000042
<EXTERNAL> :: __printf_fp @ EXTERNAL:0000001b
<EXTERNAL> :: __underflow @ EXTERNAL:0000002e
<EXTERNAL> :: abort @ EXTERNAL:0000001a
<EXTERNAL> :: atof @ EXTERNAL:00000020
<EXTERNAL> :: atol @ EXTERNAL:00000007
<EXTERNAL> :: exit @ EXTERNAL:0000003d
<EXTERNAL> :: fflush @ EXTERNAL:00000017
<EXTERNAL> :: fprintf @ EXTERNAL:00000021
<EXTERNAL> :: free @ EXTERNAL:0000004c
<EXTERNAL> :: getcwd @ EXTERNAL:00000031
<EXTERNAL> :: localeconv @ EXTERNAL:00000037
<EXTERNAL> :: malloc @ EXTERNAL:00000013
<EXTERNAL> :: memcpy @ EXTERNAL:0000000d
<EXTERNAL> :: sprintf @ EXTERNAL:00000033
<EXTERNAL> :: strcmp @ EXTERNAL:00000030
<EXTERNAL> :: strlen @ EXTERNAL:00000043
<EXTERNAL> :: tolower @ EXTERNAL:00000011
<EXTERNAL> :: unlink @ EXTERNAL:00000029
```

## Exports / entry points

```
.LFE141 @ 0805ef7c [function]
.LFE20 @ 0805f19c [function]
.LFE23 @ 0805f028 [function]
AccountExists__FRC7JString @ 0805467c [Function]
AccountExists__FRC7JString @ 0805467c [function]
AddTempAccount__FP12TTempAccount @ 08052238 [Function]
AddTempAccount__FP12TTempAccount @ 08052238 [function]
AddWeapon__FRC7JString @ 0805205c [Function]
AddWeapon__FRC7JString @ 0805205c [function]
Add__12TJStringListG7JString @ 08050328 [Function]
Add__12TJStringListG7JString @ 08050328 [function]
Add__5TListPv @ 0804fed0 [Function]
Add__5TListPv @ 0804fed0 [function]
Assign__12TJStringListP12TJStringList @ 080502ac [Function]
Assign__12TJStringListP12TJStringList @ 080502ac [function]
Assigned__FPCv @ 080553d4 [Function]
Assigned__FPCv @ 080553d4 [function]
Bbase__9streambuf @ 0805a1d8 [Function]
Bbase__9streambuf @ 0805a1d8 [function]
Bptr__9streambuf @ 0805a1f8 [Function]
Bptr__9streambuf @ 0805a1f8 [function]
Clear__12TJStringList @ 080505d4 [Function]
Clear__12TJStringList @ 080505d4 [function]
Clear__5TList @ 08050220 [Function]
Clear__5TList @ 08050220 [function]
CompressBuf__FPCvi @ 08055114 [Function]
CompressBuf__FPCvi @ 08055114 [function]
Copy__FRC7JStringii @ 0804fab8 [Function]
Copy__FRC7JStringii @ 0804fab8 [function]
DecompressBuf__FPCvi @ 0805513c [Function]
DecompressBuf__FPCvi @ 0805513c [function]
Delete__12TJStringListi @ 0805046c [Function]
Delete__12TJStringListi @ 0805046c [function]
Delete__5TListi @ 0805009c [Function]
Delete__5TListi @ 0805009c [function]
ExtractFilepath__FRC7JString @ 0805523c [Function]
ExtractFilepath__FRC7JString @ 0805523c [function]
FileExists__FRC7JString @ 08055164 [Function]
FileExists__FRC7JString @ 08055164 [function]
FormCreate__12TServerFrame @ 0804b414 [Function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
080480f4	/lib/ld-linux.so.2
08049782	__printf_fp
08049812	fprintf
08049991	sprintf
08049a83	__libc_start_main
08049ac2	_IO_file_fopen
08049b08	_IO_default_pbackfail
0805f78c	onlinestartlocal.graal
0805f920	<body bgcolor="#009933" background="image/fond2b.gif" text="#CCCC99" link="#0000CC" vlink="#0000CC">
0805fad1	</table>
0805fb77	accounts/acc
0805fbb8	cgi-bin/newaccounts.txt
0805fbd4	cgi-bin/names.txt
.comment::00000001	GCC: (GNU) egcs-2.91.66 19990314/Linux (egcs-1.1.2 release)
.comment::0000003e	GCC: (GNU) egcs-2.91.66 19990314/Linux (egcs-1.1.2 release)
.comment::000004a3	GCC: (GNU) egcs-2.91.66 19990314/Linux (egcs-1.1.2 release)
.shstrtab::00000023	.note.ABI-tag
.shstrtab::00000047	.gnu.version
.shstrtab::00000054	.gnu.version_r
.shstrtab::0000007e	.init
.stabstr::00000007	/usr/src/packages/BUILD/glibc-2.1/csu/
.stabstr::00000376	../include/libc-symbols.h
.stabstr::00000390	/usr/src/packages/BUILD/glibc-2.1/cc/config.h
.stabstr::000003bf	../include/libintl.h
.stabstr::000003d4	../intl/libintl.h
.stabstr::000003e6	../include/features.h
.stabstr::000003fc	../include/sys/cdefs.h
.stabstr::00000413	../misc/sys/cdefs.h
.stabstr::00000427	/usr/lib/gcc-lib/i486-linux/egcs-2.91.66/include/stddef.h
.stabstr::00000461	../include/locale.h
.stabstr::00000475	../locale/locale.h
.stabstr::00000699	../include/xlocale.h
.stabstr::000006ae	../locale/xlocale.h
.stabstr::000007a8	../sysdeps/unix/sysv/linux/_G_config.h
.stabstr::000007cf	../sysdeps/unix/sysv/linux/bits/types.h
.stabstr::00000d51	../linuxthreads/sysdeps/pthread/bits/pthreadtypes.h
.stabstr::00000d85	../sysdeps/unix/sysv/linux/bits/sched.h
.stabstr::00001327	/home/graal/gserver13/stats/
.stabstr::00001344	gserver.cpp
.stabstr::000013d7	/usr/include/unistd.h
.stabstr::000013ed	/usr/include/features.h
.stabstr::00001405	/usr/include/sys/cdefs.h
.stabstr::0000141e	/usr/include/gnu/stubs.h
.stabstr::00001437	/usr/include/bits/posix_opt.h
.stabstr::00001455	/usr/include/bits/types.h
.stabstr::0000146f	/usr/local/lib/gcc-lib/i686-pc-linux-gnu/2.95.2/include/stddef.h
.stabstr::00001b71	/usr/include/bits/confname.h
.stabstr::00001b8e	/usr/include/getopt.h
.stabstr::00001bbc	/usr/include/stdlib.h
.stabstr::00001de1	/usr/include/sys/types.h
.stabstr::00001fb3	/usr/include/time.h
.stabstr::000020f6	/usr/include/endian.h
.stabstr::0000210c	/usr/include/bits/endian.h
.stabstr::00002127	/usr/include/sys/select.h
.stabstr::00002141	/usr/include/bits/select.h
.stabstr::0000215c	/usr/include/bits/sigset.h
.stabstr::00002371	/usr/include/sys/sysmacros.h
.stabstr::0000262c	/usr/include/alloca.h
.stabstr::00002671	/usr/include/netdb.h
.stabstr::00002686	/usr/include/rpc/netdb.h
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `main_0804da2c.c` | `main` | `0804da2c` |
| `init__3iosP9streambufP7ostream_0805a558.c` | `init__3iosP9streambufP7ostream` | `0805a558` |
| `do_osfx__7ostream_080586a4.c` | `do_osfx__7ostream` | `080586a4` |
| `entry_0804a6d0.c` | `entry` | `0804a6d0` |
| `init__7filebuf_0805a640.c` | `init__7filebuf` | `0805a640` |
| `init_dummy_0804a788.c` | `init_dummy` | `0804a788` |
| `init_dummy_0805e4f8.c` | `init_dummy` | `0805e4f8` |
| `SetProperties__13TServerPlayerG7JString_0804bc18.c` | `SetProperties__13TServerPlayerG7JString` | `0804bc18` |
| `ParseAccount__FG7JString_080538bc.c` | `ParseAccount__FG7JString` | `080538bc` |
| `LoadAccounts__Fv_080525ac.c` | `LoadAccounts__Fv` | `080525ac` |
| `SaveAccount__FP12TTempAccountG7JString_08052ccc.c` | `SaveAccount__FP12TTempAccountG7JString` | `08052ccc` |
| `SaveAccounts__Fv_080532a0.c` | `SaveAccounts__Fv` | `080532a0` |
| `LoadAccount__14TServerAccount_08051420.c` | `LoadAccount__14TServerAccount` | `08051420` |
| `ReadNewAccounts__Fv_08054790.c` | `ReadNewAccounts__Fv` | `08054790` |
| `RemoveLineEnds__FG7JString_0804ac70.c` | `RemoveLineEnds__FG7JString` | `0804ac70` |
| `throw_helper_0805b244.c` | `throw_helper` | `0805b244` |
| `ParseAccount2__FG7JString_0805428c.c` | `ParseAccount2__FG7JString` | `0805428c` |
| `read_int__FR7istreamRUxRi_08056c44.c` | `read_int__FR7istreamRUxRi` | `08056c44` |
| `write_int__FR7ostreamUxi_080575c8.c` | `write_int__FR7ostreamUxi` | `080575c8` |
| `AddTempAccount__FP12TTempAccount_08052238.c` | `AddTempAccount__FP12TTempAccount` | `08052238` |
| `LoadFromFile__7TStreamRC7JString_08050964.c` | `LoadFromFile__7TStreamRC7JString` | `08050964` |
| `GetClosed__12TServerChest_0804d364.c` | `GetClosed__12TServerChest` | `0804d364` |
| `SaveAccountNames__Fv_08054d70.c` | `SaveAccountNames__Fv` | `08054d70` |
| `end_fde_sort_0805c288.c` | `end_fde_sort` | `0805c288` |
| `GetIDAsJString__14TServerAccount_08050f84.c` | `GetIDAsJString__14TServerAccount` | `08050f84` |
| `getline__7istreamPcic_080590e0.c` | `getline__7istreamPcic` | `080590e0` |
| `RemoveExtension__FG7JString_0804a970.c` | `RemoveExtension__FG7JString` | `0804a970` |
| `SetCommaText__12TJStringListG7JString_08050658.c` | `SetCommaText__12TJStringListG7JString` | `08050658` |
| `extract_cie_info_0805c6f4.c` | `extract_cie_info` | `0805c6f4` |
| `dcast__C17__class_type_infoRC9type_infoiPvPC9type_infoT3_0805d544.c` | `dcast__C17__class_type_infoRC9type_infoiPvPC9type_infoT3` | `0805d544` |
| `AddWeapon__FRC7JString_0805205c.c` | `AddWeapon__FRC7JString` | `0805205c` |
| `GetOpened__12TServerChest_0804d728.c` | `GetOpened__12TServerChest` | `0804d728` |
| `_._7JString_0805e510.c` | `_._7JString` | `0805e510` |
| `flush__7ostream_08058394.c` | `flush__7ostream` | `08058394` |
| `length__C7JString_0804f1d8.c` | `length__C7JString` | `0804f1d8` |
| `Length__FRC7JString_0804f9d4.c` | `Length__FRC7JString` | `0804f9d4` |
| `_skip_ws__7istream_080584b8.c` | `_skip_ws__7istream` | `080584b8` |
| `text__C7JString_0804f200.c` | `text__C7JString` | `0804f200` |
| `SaveToFile__7TStreamRC7JString_08050c60.c` | `SaveToFile__7TStreamRC7JString` | `08050c60` |
| `_._9type_info_0805d364.c` | `_._9type_info` | `0805d364` |
| `LowerCaseFilename__FG7JString_0804a7a0.c` | `LowerCaseFilename__FG7JString` | `0804a7a0` |
| `subJString__C7JStringii_0804f3ec.c` | `subJString__C7JStringii` | `0804f3ec` |
| `LowerCase__FRC7JString_0804faec.c` | `LowerCase__FRC7JString` | `0804faec` |
| `Copy__FRC7JStringii_0804fab8.c` | `Copy__FRC7JStringii` | `0804fab8` |
| `Add__12TJStringListG7JString_08050328.c` | `Add__12TJStringListG7JString` | `08050328` |
| `FileExists__FRC7JString_08055164.c` | `FileExists__FRC7JString` | `08055164` |
| `eh_context_initialize_0805ac74.c` | `eh_context_initialize` | `0805ac74` |
| `getGifNumber__FG7JStringRC7JString_0804b284.c` | `getGifNumber__FG7JStringRC7JString` | `0804b284` |
| `GetWeaponName__FRC7JString_08051f30.c` | `GetWeaponName__FRC7JString` | `08051f30` |
| `read__7istreamPci_0805684c.c` | `read__7istreamPci` | `0805684c` |

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

