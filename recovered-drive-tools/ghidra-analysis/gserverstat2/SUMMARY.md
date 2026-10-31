# Salvage analysis: gserverstat2

- **Binary:** `/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat2`
- **Analyzed (UTC):** 20261031T205935Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat2: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux.so.2, for GNU/Linux 2.0.0, not stripped

# size
-rw-r--r-- 1 ubuntu ubuntu 513K Dec 25  1999 /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat2

# sha256
966feaab46fea8b93e8b6c8a23cc333b9161f8f08f246b376cf4bfdb375fedbd  /workspace/recovered-drive-tools/_inbox/extracted/gserver_source/gserverstat2
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

Same string profile as `gserverstat`, aimed at admin weapon/power leaderboards. Paths still center on `accounts/acc*` and CGI name lists (`cgi-bin/newaccounts.txt`, `cgi-bin/names.txt`). HTML output keeps the green table theme (`image/fond2b.gif` background) and `onlinestartlocal.graal`. Debug/build strings again place the source under a `gserver13/stats/` tree (`gserver.cpp`) with egcs-2.91 / glibc-2.1 toolchain marks. Formatted libc I/O only — no socket/listen strings — matching an offline `stats/admin/*.html` emitter driven by `statmaker`.

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

