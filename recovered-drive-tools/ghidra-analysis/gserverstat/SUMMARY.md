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
| Functions | 899 (internal 744) |
| Imports | 77 |
| Exports / entries | 1296 |
| Interesting strings | 223 |
| Decompiled funcs | 50 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `SetProperties`, `main`, `ParseAccount`, `LoadAccounts`, `SaveAccount`, `SaveAccounts`, `throw_type_match_rtti`, `LoadAccount`, `13TServerPlayer`, `ReadNewAccounts`, `RemoveLineEnds`, `ParseAccount2`, `read_int`, `write_int`, `init`, `do_osfx`, `AddTempAccount`, `LoadFromFile`, `GetClosed`, `SaveAccountNames`, `end_fde_sort`, `GetIDAsJString`, `getline`, `RemoveExtension`, `SetCommaText`.

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
<EXTERNAL> :: __errno_location
<EXTERNAL> :: __finite
<EXTERNAL> :: __finitef
<EXTERNAL> :: __finitel
<EXTERNAL> :: __gmon_start__
<EXTERNAL> :: __libc_start_main
<EXTERNAL> :: __printf_fp
<EXTERNAL> :: __underflow
<EXTERNAL> :: abort
<EXTERNAL> :: atof
<EXTERNAL> :: atol
<EXTERNAL> :: exit
<EXTERNAL> :: fflush
<EXTERNAL> :: fprintf
<EXTERNAL> :: free
<EXTERNAL> :: getcwd
<EXTERNAL> :: localeconv
<EXTERNAL> :: malloc
<EXTERNAL> :: memcpy
<EXTERNAL> :: sprintf
<EXTERNAL> :: strcmp
<EXTERNAL> :: strlen
<EXTERNAL> :: tolower
<EXTERNAL> :: unlink
```

## Exports / entry points

```
.LFE141 [function]
.LFE20 [function]
.LFE23 [function]
AccountExists__FRC7JString [Function]
AccountExists__FRC7JString [function]
AddTempAccount__FP12TTempAccount [Function]
AddTempAccount__FP12TTempAccount [function]
AddWeapon__FRC7JString [Function]
AddWeapon__FRC7JString [function]
Add__12TJStringListG7JString [Function]
Add__12TJStringListG7JString [function]
Add__5TListPv [Function]
Add__5TListPv [function]
Assign__12TJStringListP12TJStringList [Function]
Assign__12TJStringListP12TJStringList [function]
Assigned__FPCv [Function]
Assigned__FPCv [function]
Bbase__9streambuf [Function]
Bbase__9streambuf [function]
Bptr__9streambuf [Function]
Bptr__9streambuf [function]
Clear__12TJStringList [Function]
Clear__12TJStringList [function]
Clear__5TList [Function]
Clear__5TList [function]
CompressBuf__FPCvi [Function]
CompressBuf__FPCvi [function]
Copy__FRC7JStringii [Function]
Copy__FRC7JStringii [function]
DecompressBuf__FPCvi [Function]
DecompressBuf__FPCvi [function]
Delete__12TJStringListi [Function]
Delete__12TJStringListi [function]
Delete__5TListi [Function]
Delete__5TListi [function]
ExtractFilepath__FRC7JString [Function]
ExtractFilepath__FRC7JString [function]
FileExists__FRC7JString [Function]
FileExists__FRC7JString [function]
FormCreate__12TServerFrame [Function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
/lib/ld-linux.so.2
__printf_fp
fprintf
sprintf
__libc_start_main
_IO_file_fopen
_IO_default_pbackfail
onlinestartlocal.graal
<body bgcolor="#009933" background="image/fond2b.gif" text="#CCCC99" link="#0000CC" vlink="#0000CC">
</table>
accounts/acc
cgi-bin/newaccounts.txt
cgi-bin/names.txt
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

