# Salvage analysis: BytecodeViewer

- **Binary:** `/workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe`
- **Analyzed (UTC):** 20261031T205222Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe: PE32 executable (console) Intel 80386, for MS Windows, 5 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 88K Sep  2  2017 /workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe

# sha256
8e6b1f22947a6df814fc2f4be8169998e1f2bd1effc2becf51029eff3107f6a9  /workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | windows |
| Image base | 00400000 |
| Functions | 313 (internal 310) |
| Imports | 73 |
| Exports / entries | 40 |
| Interesting strings | 64 |
| Decompiled funcs | 30 |

## Top / key functions

**Largest internal:** FUN_00404bdd@00404bdd(sz=2918) | FUN_004072c4@004072c4(sz=2090) | FUN_00401ba5@00401ba5(sz=1935) | FUN_0040397c@0040397c(sz=1872) | FUN_00401030@00401030(sz=1765) | FUN_00409690@00409690(sz=1427) | FUN_0040c120@0040c120(sz=1427) | FindHandler@0040b748(sz=923) | ___free_lc_time@0040a9b0(sz=896) | FUN_00403515@00403515(sz=677) | FUN_00406c36@00406c36(sz=575) | strtoxl@00404742(sz=548) | __setmbcp_nolock@004086c4(sz=501) | __crtLCMapStringA_stat@0040a4f6(sz=489) | FID_conflict:___crtMessageBoxW@00408a58(sz=482)

**Decompile targets:** entry@00402b5d | FUN_00404bdd@00404bdd | FUN_004072c4@004072c4 | FUN_00401ba5@00401ba5 | FUN_0040397c@0040397c | FUN_00401030@00401030 | FUN_00409690@00409690 | FUN_0040c120@0040c120 | FindHandler@0040b748 | FUN_00403515@00403515 | FUN_00406c36@00406c36 | FID_conflict:___crtMessageBoxW@00408a58 | FUN_004044ff@004044ff | _raise@00407fc5 | setSBUpLow@004082e3 | FUN_00402fe1@00402fe1 | parse_cmdline@0040658b | FUN_00406eb0@00406eb0 | FUN_00402e92@00402e92 | FID_conflict:_free@004018ca | FUN_00405faf@00405faf | FUN_004029e2@004029e2 | FUN_00402541@00402541 | doexit@0040435f | _malloc@0040192d | _LocaleUpdate@004046ba | FUN_004071e2@004071e2 | FUN_0040ae13@0040ae13 | FID_conflict:_wprintf@004019f1 | FUN_00407ef2@00407ef2

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
KERNEL32.DLL :: AreFileApisANSI @ EXTERNAL:00000024
KERNEL32.DLL :: CloseHandle @ EXTERNAL:00000010
KERNEL32.DLL :: CreateFileW @ EXTERNAL:0000001e
KERNEL32.DLL :: DecodePointer @ EXTERNAL:00000015
KERNEL32.DLL :: DeleteCriticalSection @ EXTERNAL:0000001b
KERNEL32.DLL :: EncodePointer @ EXTERNAL:00000014
KERNEL32.DLL :: EnterCriticalSection @ EXTERNAL:00000018
KERNEL32.DLL :: ExitProcess @ EXTERNAL:00000022
KERNEL32.DLL :: FlushFileBuffers @ EXTERNAL:00000049
KERNEL32.DLL :: FreeEnvironmentStringsW @ EXTERNAL:00000031
KERNEL32.DLL :: GetACP @ EXTERNAL:00000041
KERNEL32.DLL :: GetCPInfo @ EXTERNAL:00000043
KERNEL32.DLL :: GetCommandLineA @ EXTERNAL:00000011
KERNEL32.DLL :: GetConsoleCP @ EXTERNAL:0000003d
KERNEL32.DLL :: GetConsoleMode @ EXTERNAL:00000008
KERNEL32.DLL :: GetCurrentProcess @ EXTERNAL:00000036
KERNEL32.DLL :: GetCurrentProcessId @ EXTERNAL:0000002e
KERNEL32.DLL :: GetCurrentThreadId @ EXTERNAL:0000002b
KERNEL32.DLL :: GetEnvironmentStringsW @ EXTERNAL:00000030
KERNEL32.DLL :: GetFileType @ EXTERNAL:00000017
KERNEL32.DLL :: GetLastError @ EXTERNAL:00000003
KERNEL32.DLL :: GetModuleFileNameA @ EXTERNAL:0000002c
KERNEL32.DLL :: GetModuleFileNameW @ EXTERNAL:00000026
KERNEL32.DLL :: GetModuleHandleExW @ EXTERNAL:00000023
KERNEL32.DLL :: GetModuleHandleW @ EXTERNAL:0000001f
KERNEL32.DLL :: GetOEMCP @ EXTERNAL:00000042
KERNEL32.DLL :: GetProcAddress @ EXTERNAL:00000020
KERNEL32.DLL :: GetProcessHeap @ EXTERNAL:0000001d
KERNEL32.DLL :: GetStartupInfoW @ EXTERNAL:0000001c
KERNEL32.DLL :: GetStdHandle @ EXTERNAL:00000002
KERNEL32.DLL :: GetStringTypeW @ EXTERNAL:00000048
KERNEL32.DLL :: GetSystemTimeAsFileTime @ EXTERNAL:0000002f
KERNEL32.DLL :: HeapAlloc @ EXTERNAL:00000005
KERNEL32.DLL :: HeapFree @ EXTERNAL:00000004
KERNEL32.DLL :: HeapQueryInformation @ EXTERNAL:0000000d
KERNEL32.DLL :: HeapReAlloc @ EXTERNAL:0000000b
KERNEL32.DLL :: HeapSize @ EXTERNAL:0000000c
KERNEL32.DLL :: InitializeCriticalSectionAndSpinCount @ EXTERNAL:0000001a
KERNEL32.DLL :: InterlockedDecrement @ EXTERNAL:00000021
KERNEL32.DLL :: InterlockedIncrement @ EXTERNAL:0000002a
KERNEL32.DLL :: IsDebuggerPresent @ EXTERNAL:00000012
KERNEL32.DLL :: IsProcessorFeaturePresent @ EXTERNAL:00000013
KERNEL32.DLL :: IsValidCodePage @ EXTERNAL:00000040
KERNEL32.DLL :: LCMapStringW @ EXTERNAL:00000047
KERNEL32.DLL :: LeaveCriticalSection @ EXTERNAL:00000019
KERNEL32.DLL :: LoadLibraryExW @ EXTERNAL:0000003f
KERNEL32.DLL :: LoadLibraryW @ EXTERNAL:00000045
KERNEL32.DLL :: MultiByteToWideChar @ EXTERNAL:00000007
KERNEL32.DLL :: OutputDebugStringW @ EXTERNAL:00000044
KERNEL32.DLL :: QueryPerformanceCounter @ EXTERNAL:0000002d
KERNEL32.DLL :: RaiseException @ EXTERNAL:00000033
KERNEL32.DLL :: ReadConsoleInputA @ EXTERNAL:0000000e
KERNEL32.DLL :: ReadConsoleW @ EXTERNAL:00000009
KERNEL32.DLL :: ReadFile @ EXTERNAL:00000006
KERNEL32.DLL :: RtlUnwind @ EXTERNAL:0000003c
KERNEL32.DLL :: SetConsoleMode @ EXTERNAL:0000000f
KERNEL32.DLL :: SetConsoleTextAttribute @ EXTERNAL:00000001
KERNEL32.DLL :: SetEndOfFile @ EXTERNAL:0000003e
KERNEL32.DLL :: SetFilePointer @ EXTERNAL:0000000a
KERNEL32.DLL :: SetFilePointerEx @ EXTERNAL:00000027
KERNEL32.DLL :: SetLastError @ EXTERNAL:00000029
KERNEL32.DLL :: SetStdHandle @ EXTERNAL:00000016
KERNEL32.DLL :: SetUnhandledExceptionFilter @ EXTERNAL:00000035
KERNEL32.DLL :: Sleep @ EXTERNAL:00000028
KERNEL32.DLL :: TerminateProcess @ EXTERNAL:00000037
KERNEL32.DLL :: TlsAlloc @ EXTERNAL:00000038
KERNEL32.DLL :: TlsFree @ EXTERNAL:0000003b
KERNEL32.DLL :: TlsGetValue @ EXTERNAL:00000039
KERNEL32.DLL :: TlsSetValue @ EXTERNAL:0000003a
KERNEL32.DLL :: UnhandledExceptionFilter @ EXTERNAL:00000034
KERNEL32.DLL :: WideCharToMultiByte @ EXTERNAL:00000032
KERNEL32.DLL :: WriteConsoleW @ EXTERNAL:00000046
KERNEL32.DLL :: WriteFile @ EXTERNAL:00000025
```

## Exports / entry points

```
CPtoLocaleName @ 004081d4 [function]
CallCatchBlock @ 0040b472 [function]
CatchIt @ 0040b680 [function]
Catch_All@0040b668 @ 0040b668 [function]
ExFilterRethrow @ 0040b6fb [function]
FID_conflict:GetTableIndexFromLocaleName @ 0040a3c4 [function]
FID_conflict:_AtlDownlevelLocaleNameToLCID @ 0040a412 [function]
FID_conflict:_CallMemberFunction1 @ 0040b098 [function]
FID_conflict:___CxxFrameHandler3 @ 0040b319 [function]
FID_conflict:___crtMessageBoxW @ 00408a58 [function]
FID_conflict:__dosmaperr @ 00402ee0 [function]
FID_conflict:_free @ 004018ca [function]
FID_conflict:_wopen @ 00401902 [function]
FID_conflict:_wprintf @ 004019f1 [function]
FindHandler @ 0040b748 [function]
FindHandlerForForeignException @ 0040bae3 [function]
TranslatorGuardHandler @ 0040af9d [function]
Unwind@0040c7b0 @ 0040c7b0 [function]
doexit @ 0040435f [function]
entry @ 00402b5d [Function]
entry @ 00402b5d [function]
fast_error_exit @ 00402b36 [function]
flsall @ 004092b7 [function]
getSystemCP @ 0040820e [function]
operator_new @ 00402b8d [function]
parse_cmdline @ 0040658b [function]
setSBCS @ 0040827a [function]
setSBUpLow @ 004082e3 [function]
siglookup @ 0040817a [function]
std::exception::exception @ 0040699f [function]
std::exception::exception @ 004069c4 [function]
std::exception::exception @ 004069df [function]
std::exception::operator= @ 00406a0d [function]
strtoxl @ 00404742 [function]
terminate @ 00407f2a [function]
type_info::operator== @ 00406b35 [function]
write_char @ 00405765 [function]
write_multi_char @ 004057ab [function]
write_string @ 004057d7 [function]
x_ismbbtype_l @ 00409e4b [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
0040d6b0	R6017  - unexpected multithread lock error
0040d710	R6018  - unexpected heap error
0040d7b0	R6024  - not enough space for _onexit/atexit table
0040db10	R6033  - Attempt to use MSIL code from this assembly during native code initialization This indicates a bug in your application. It is most likely the result of calling an MSIL-compiled (/clr) function from a native constructor or from DllMain.
0040dd00	R6034  - inconsistent onexit begin-end variables
0040dd68	DOMAIN error
0040dd88	SING error
0040dda4	TLOSS error
0040ddc8	runtime error
0040de9e	@R6002  - floating point support not loaded
0040defc	Runtime Error!  Program:
0040e3a8	ja-JP
0040e3b4	zh-CN
0040e3c0	ko-KR
0040e3cc	zh-TW
0040e554	MM/dd/yy
0040e754	MM/dd/yy
0040e7a4	en-US
0040f928	de-DE
0040fa48	sl-SI
0040fb14	hi-IN
0040fb8c	bn-IN
0040fb98	pa-IN
0040fbb0	ta-IN
0040fbbc	te-IN
0040fbc8	kn-IN
0040fbec	sa-IN
0040fc80	de-CH
0040fce0	sr-SP-Latn
0040fd1c	se-SE
0040fd74	de-AT
0040fda4	sr-SP-Cyrl
0040fe14	fr-CH
0040fe54	de-LI
0040fe84	bs-BA-Latn
0040fedc	sr-BA-Latn
004101ec	bn-in
004101f8	bs-ba-latn
00410240	de-at
0041024c	de-ch
00410258	de-de
00410264	de-li
00410304	en-tt
0041046c	fr-ch
004104c0	hi-in
004104fc	id-id
00410550	kn-in
00410638	pa-in
004106b0	sa-in
004106d4	se-se
004106ec	sl-si
0041077c	sr-ba-latn
00410794	sr-sp-cyrl
004107ac	sr-sp-latn
004107f8	ta-in
00410804	te-in
00411628	`placement delete closure'
0041258e	GetLastError
00412684	IsDebuggerPresent
00412846	SetLastError
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `entry_00402b5d.c` | `entry` | `00402b5d` |
| `FUN_00404bdd_00404bdd.c` | `FUN_00404bdd` | `00404bdd` |
| `FUN_004072c4_004072c4.c` | `FUN_004072c4` | `004072c4` |
| `FUN_00401ba5_00401ba5.c` | `FUN_00401ba5` | `00401ba5` |
| `FUN_0040397c_0040397c.c` | `FUN_0040397c` | `0040397c` |
| `FUN_00401030_00401030.c` | `FUN_00401030` | `00401030` |
| `FUN_00409690_00409690.c` | `FUN_00409690` | `00409690` |
| `FUN_0040c120_0040c120.c` | `FUN_0040c120` | `0040c120` |
| `FindHandler_0040b748.c` | `FindHandler` | `0040b748` |
| `FUN_00403515_00403515.c` | `FUN_00403515` | `00403515` |
| `FUN_00406c36_00406c36.c` | `FUN_00406c36` | `00406c36` |
| `FID_conflict____crtMessageBoxW_00408a58.c` | `FID_conflict:___crtMessageBoxW` | `00408a58` |
| `FUN_004044ff_004044ff.c` | `FUN_004044ff` | `004044ff` |
| `_raise_00407fc5.c` | `_raise` | `00407fc5` |
| `setSBUpLow_004082e3.c` | `setSBUpLow` | `004082e3` |
| `FUN_00402fe1_00402fe1.c` | `FUN_00402fe1` | `00402fe1` |
| `parse_cmdline_0040658b.c` | `parse_cmdline` | `0040658b` |
| `FUN_00406eb0_00406eb0.c` | `FUN_00406eb0` | `00406eb0` |
| `FUN_00402e92_00402e92.c` | `FUN_00402e92` | `00402e92` |
| `FID_conflict__free_004018ca.c` | `FID_conflict:_free` | `004018ca` |
| `FUN_00405faf_00405faf.c` | `FUN_00405faf` | `00405faf` |
| `FUN_004029e2_004029e2.c` | `FUN_004029e2` | `004029e2` |
| `FUN_00402541_00402541.c` | `FUN_00402541` | `00402541` |
| `doexit_0040435f.c` | `doexit` | `0040435f` |
| `_malloc_0040192d.c` | `_malloc` | `0040192d` |
| `_LocaleUpdate_004046ba.c` | `_LocaleUpdate::_LocaleUpdate` | `004046ba` |
| `FUN_004071e2_004071e2.c` | `FUN_004071e2` | `004071e2` |
| `FUN_0040ae13_0040ae13.c` | `FUN_0040ae13` | `0040ae13` |
| `FID_conflict__wprintf_004019f1.c` | `FID_conflict:_wprintf` | `004019f1` |
| `FUN_00407ef2_00407ef2.c` | `FUN_00407ef2` | `00407ef2` |

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

