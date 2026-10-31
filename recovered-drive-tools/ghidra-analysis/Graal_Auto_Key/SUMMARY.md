# Salvage analysis: Graal_Auto_Key.unpacked

- **Binary:** `/workspace/recovered-drive-tools/_extracted/unpacked/Graal_Auto_Key.unpacked.exe`
- **Analyzed (UTC):** 20261031T210030Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

Client **auto-key / key-sender trainer**. Packed sample is UPX; this analysis used a static `upx -d` unpack (PE never executed). Unpacked image is **AutoIt v2.64** with `keybd_event` / `mouse_event` / `SEND` / timers — keyboard macro against a focused Graal window.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `out-of-scope` |
| --- | --- |
| Why | Client trainer / cheat — decision tree: do not place in seihai-servertools. |
| Defense note | Same class as recovered-hacks trainers; host IOC = hash + AutoIt+UPX. |
| Licence risk | Untrusted third-party binary |

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/unpacked/Graal_Auto_Key.unpacked.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 4 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 85K Oct 31 20:58 /workspace/recovered-drive-tools/_extracted/unpacked/Graal_Auto_Key.unpacked.exe

# sha256
a99410a422694d84eb5ad1b93e2406c15f14a83a9d9d818bad3ab6f289ebf0f0  /workspace/recovered-drive-tools/_extracted/unpacked/Graal_Auto_Key.unpacked.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | windows |
| Image base | 00400000 |
| Functions | 303 (internal 301) |
| Imports | 158 |
| Exports / entries | 13 |
| Interesting strings | 34 |
| Decompiled funcs | 30 |

## Top / key functions

**Largest internal:** FUN_004023b7@004023b7(sz=2144) | FUN_0040a35d@0040a35d(sz=1909) | ___crtLCMapStringA@0040d0de(sz=886) | FUN_0040de05@0040de05(sz=854) | ___sbh_free_block@0040966a(sz=792) | ___sbh_alloc_block@00409e1e(sz=764) | ___sbh_resize_block@00409b3f(sz=735) | FUN_0040c4a2@0040c4a2(sz=720) | FUN_004061cb@004061cb(sz=695) | _memcpy@00408c00(sz=672) | _memmove@0040c840(sz=672) | __tzset@0040acf8(sz=595) | FUN_00406d2e@00406d2e(sz=517) | FUN_00401c17@00401c17(sz=493) | FUN_0040b54f@0040b54f(sz=492)

**Decompile targets:** entry@00408359 | FUN_004023b7@004023b7 | FUN_0040a35d@0040a35d | FUN_0040de05@0040de05 | FUN_0040c4a2@0040c4a2 | FUN_004061cb@004061cb | _memcpy@00408c00 | _memmove@0040c840 | FUN_00406d2e@00406d2e | FUN_00401c17@00401c17 | FUN_0040b54f@0040b54f | FUN_00408a1f@00408a1f | FUN_00402d9b@00402d9b | cvtdate@0040af4b | FUN_0040d939@0040d939 | FUN_0040e23d@0040e23d | FUN_004093d8@004093d8 | FUN_00404a21@00404a21 | FUN_00406b21@00406b21 | _strlen@00407630 | FUN_00406f33@00406f33 | FUN_00406fae@00406fae | _malloc@004078a9 | _free@004077ff | FUN_0040721b@0040721b | FUN_0040561d@0040561d | FUN_0040b8cf@0040b8cf | FUN_004022b4@004022b4 | _realloc@0040a135 | FUN_00406636@00406636

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
ADVAPI32.DLL :: AdjustTokenPrivileges @ EXTERNAL:00000052
ADVAPI32.DLL :: LookupPrivilegeValueA @ EXTERNAL:00000053
ADVAPI32.DLL :: OpenProcessToken @ EXTERNAL:00000054
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:00000055
ADVAPI32.DLL :: RegCreateKeyExA @ EXTERNAL:00000058
ADVAPI32.DLL :: RegDeleteValueA @ EXTERNAL:00000051
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000056
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000050
ADVAPI32.DLL :: RegSetValueExA @ EXTERNAL:00000057
COMDLG32.DLL :: GetOpenFileNameA @ EXTERNAL:00000059
GDI32.DLL :: GetStockObject @ EXTERNAL:0000005b
GDI32.DLL :: GetTextMetricsA @ EXTERNAL:0000005a
KERNEL32.DLL :: CloseHandle @ EXTERNAL:00000048
KERNEL32.DLL :: CompareStringA @ EXTERNAL:00000014
KERNEL32.DLL :: CompareStringW @ EXTERNAL:00000044
KERNEL32.DLL :: CopyFileA @ EXTERNAL:0000000f
KERNEL32.DLL :: CreateDirectoryA @ EXTERNAL:0000000e
KERNEL32.DLL :: CreateFileA @ EXTERNAL:00000023
KERNEL32.DLL :: CreateProcessA @ EXTERNAL:00000047
KERNEL32.DLL :: DeleteFileA @ EXTERNAL:0000000d
KERNEL32.DLL :: ExitProcess @ EXTERNAL:0000003b
KERNEL32.DLL :: FindClose @ EXTERNAL:0000000a
KERNEL32.DLL :: FindFirstFileA @ EXTERNAL:0000000b
KERNEL32.DLL :: FindNextFileA @ EXTERNAL:0000000c
KERNEL32.DLL :: FlushFileBuffers @ EXTERNAL:00000024
KERNEL32.DLL :: FreeEnvironmentStringsA @ EXTERNAL:00000029
KERNEL32.DLL :: FreeEnvironmentStringsW @ EXTERNAL:00000027
KERNEL32.DLL :: FreeLibrary @ EXTERNAL:00000045
KERNEL32.DLL :: GetACP @ EXTERNAL:0000002d
KERNEL32.DLL :: GetCPInfo @ EXTERNAL:0000002b
KERNEL32.DLL :: GetCommandLineA @ EXTERNAL:0000003c
KERNEL32.DLL :: GetCurrentDirectoryA @ EXTERNAL:0000004f
KERNEL32.DLL :: GetCurrentProcess @ EXTERNAL:00000004
KERNEL32.DLL :: GetCurrentProcessId @ EXTERNAL:0000001d
KERNEL32.DLL :: GetCurrentThreadId @ EXTERNAL:00000008
KERNEL32.DLL :: GetEnvironmentStrings @ EXTERNAL:00000028
KERNEL32.DLL :: GetEnvironmentStringsW @ EXTERNAL:00000026
KERNEL32.DLL :: GetEnvironmentVariableA @ EXTERNAL:00000013
KERNEL32.DLL :: GetExitCodeProcess @ EXTERNAL:00000046
KERNEL32.DLL :: GetFileType @ EXTERNAL:00000036
KERNEL32.DLL :: GetFullPathNameA @ EXTERNAL:0000004c
KERNEL32.DLL :: GetLastError @ EXTERNAL:00000003
KERNEL32.DLL :: GetLocaleInfoA @ EXTERNAL:00000019
KERNEL32.DLL :: GetModuleFileNameA @ EXTERNAL:0000004b
KERNEL32.DLL :: GetModuleHandleA @ EXTERNAL:00000009
KERNEL32.DLL :: GetOEMCP @ EXTERNAL:0000002c
KERNEL32.DLL :: GetPrivateProfileStringA @ EXTERNAL:00000021
KERNEL32.DLL :: GetProcAddress @ EXTERNAL:00000043
KERNEL32.DLL :: GetStartupInfoA @ EXTERNAL:0000003d
KERNEL32.DLL :: GetStdHandle @ EXTERNAL:00000037
KERNEL32.DLL :: GetStringTypeA @ EXTERNAL:00000022
KERNEL32.DLL :: GetStringTypeW @ EXTERNAL:00000020
KERNEL32.DLL :: GetSystemInfo @ EXTERNAL:00000017
KERNEL32.DLL :: GetSystemTimeAsFileTime @ EXTERNAL:0000003f
KERNEL32.DLL :: GetTickCount @ EXTERNAL:0000001e
KERNEL32.DLL :: GetTimeZoneInformation @ EXTERNAL:0000003e
KERNEL32.DLL :: GetVersionExA @ EXTERNAL:0000004a
KERNEL32.DLL :: GlobalAlloc @ EXTERNAL:00000012
KERNEL32.DLL :: GlobalLock @ EXTERNAL:00000011
KERNEL32.DLL :: GlobalUnlock @ EXTERNAL:00000010
KERNEL32.DLL :: HeapAlloc @ EXTERNAL:00000040
KERNEL32.DLL :: HeapCreate @ EXTERNAL:00000033
KERNEL32.DLL :: HeapDestroy @ EXTERNAL:00000034
KERNEL32.DLL :: HeapFree @ EXTERNAL:00000041
KERNEL32.DLL :: HeapReAlloc @ EXTERNAL:00000030
KERNEL32.DLL :: HeapSize @ EXTERNAL:0000002f
KERNEL32.DLL :: LCMapStringA @ EXTERNAL:0000001c
KERNEL32.DLL :: LCMapStringW @ EXTERNAL:0000001b
KERNEL32.DLL :: LoadLibraryA @ EXTERNAL:00000042
KERNEL32.DLL :: MultiByteToWideChar @ EXTERNAL:0000004e
KERNEL32.DLL :: OpenProcess @ EXTERNAL:00000006
KERNEL32.DLL :: QueryPerformanceCounter @ EXTERNAL:0000001f
KERNEL32.DLL :: ReadFile @ EXTERNAL:00000039
KERNEL32.DLL :: RemoveDirectoryA @ EXTERNAL:00000002
KERNEL32.DLL :: RtlUnwind @ EXTERNAL:00000015
KERNEL32.DLL :: SetEndOfFile @ EXTERNAL:0000001a
KERNEL32.DLL :: SetEnvironmentVariableA @ EXTERNAL:0000004d
KERNEL32.DLL :: SetFilePointer @ EXTERNAL:0000003a
KERNEL32.DLL :: SetHandleCount @ EXTERNAL:00000038
KERNEL32.DLL :: SetStdHandle @ EXTERNAL:00000025
```

_… 78 more in `imports.txt`_

## Exports / entry points

```
FID_conflict:__fread_lk @ 004073e8 [function]
FID_conflict:__time32 @ 00407d39 [function]
copy_environ @ 0040e1dc [function]
cvtdate @ 0040af4b [function]
entry @ 00408359 [Function]
entry @ 00408359 [function]
findenv @ 0040e18f [function]
flsall @ 004086a8 [function]
report_failure @ 0040b2a9 [function]
setSBCS @ 0040b394 [function]
write_char @ 0040a2cf [function]
write_multi_char @ 0040a302 [function]
write_string @ 0040a326 [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
0040f2e8	A_OSVERSION
0040f40c	HideAutoItDebug
0040f890	AutoIt Error
0040f8a0	Line %d: %s  Error: %s
0040f918	ERROR
0040f928	Select File
0040f9b0	HKEY_CURRENT_CONFIG
0040fc1c	Powered by AutoIt - http://www.hiddensoft.com/AutoIt/
0040fcf8	ERRORLEVEL
0040fdf4	runtime error
0040fe04	TLOSS error
0040fe14	SING error
0040fe24	DOMAIN error
0040fef4	R6024  - not enough space for _onexit/atexit table
0040ff58	R6018  - unexpected heap error
0040ff7c	R6017  - unexpected multithread lock error
0040ffd8	This application has requested the Runtime to terminate it in an unusual way. Please contact the application's support team for more information.
0041011c	Runtime Error!  Program:
00410660	A security error of unknown cause has been detected which has corrupted the program's internal state.  The program cannot safely continue execution and must now be terminated.
00410714	Unknown security failure detected!
00410ba8	GetLastError
00410c9c	GetEnvironmentVariableA
00410de0	GetEnvironmentStringsW
00410e12	GetEnvironmentStrings
00410e5e	GetCPInfo
00411022	GetVersionExA
004110c2	AdjustTokenPrivileges
004110f2	OpenProcessToken
00445a70	Third-party compiled AutoIt script.  For details visit http://www.hiddensoft.com/AutoIt/
00445b82	FileVersion
00445c0a	ProductVersion
00445f30	Stack underflow error (Return with no matching Gosub, or, EndRepeat with no matching Repeat).
00445fec	Unable to process shutdown (access rights error).
00446050	AutoIt Error
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `entry_00408359.c` | `entry` | `00408359` |
| `FUN_004023b7_004023b7.c` | `FUN_004023b7` | `004023b7` |
| `FUN_0040a35d_0040a35d.c` | `FUN_0040a35d` | `0040a35d` |
| `FUN_0040de05_0040de05.c` | `FUN_0040de05` | `0040de05` |
| `FUN_0040c4a2_0040c4a2.c` | `FUN_0040c4a2` | `0040c4a2` |
| `FUN_004061cb_004061cb.c` | `FUN_004061cb` | `004061cb` |
| `_memcpy_00408c00.c` | `_memcpy` | `00408c00` |
| `_memmove_0040c840.c` | `_memmove` | `0040c840` |
| `FUN_00406d2e_00406d2e.c` | `FUN_00406d2e` | `00406d2e` |
| `FUN_00401c17_00401c17.c` | `FUN_00401c17` | `00401c17` |
| `FUN_0040b54f_0040b54f.c` | `FUN_0040b54f` | `0040b54f` |
| `FUN_00408a1f_00408a1f.c` | `FUN_00408a1f` | `00408a1f` |
| `FUN_00402d9b_00402d9b.c` | `FUN_00402d9b` | `00402d9b` |
| `cvtdate_0040af4b.c` | `cvtdate` | `0040af4b` |
| `FUN_0040d939_0040d939.c` | `FUN_0040d939` | `0040d939` |
| `FUN_0040e23d_0040e23d.c` | `FUN_0040e23d` | `0040e23d` |
| `FUN_004093d8_004093d8.c` | `FUN_004093d8` | `004093d8` |
| `FUN_00404a21_00404a21.c` | `FUN_00404a21` | `00404a21` |
| `FUN_00406b21_00406b21.c` | `FUN_00406b21` | `00406b21` |
| `_strlen_00407630.c` | `_strlen` | `00407630` |
| `FUN_00406f33_00406f33.c` | `FUN_00406f33` | `00406f33` |
| `FUN_00406fae_00406fae.c` | `FUN_00406fae` | `00406fae` |
| `_malloc_004078a9.c` | `_malloc` | `004078a9` |
| `_free_004077ff.c` | `_free` | `004077ff` |
| `FUN_0040721b_0040721b.c` | `FUN_0040721b` | `0040721b` |
| `FUN_0040561d_0040561d.c` | `FUN_0040561d` | `0040561d` |
| `FUN_0040b8cf_0040b8cf.c` | `FUN_0040b8cf` | `0040b8cf` |
| `FUN_004022b4_004022b4.c` | `FUN_004022b4` | `004022b4` |
| `_realloc_0040a135.c` | `_realloc` | `0040a135` |
| `FUN_00406636_00406636.c` | `FUN_00406636` | `00406636` |

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

