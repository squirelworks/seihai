# Salvage analysis: BytecodeViewer.exe

- **Binary path (recorded):** `/workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe`
- **Ghidra language:** x86:LE:32:default / windows
- **Format:** Portable Executable (PE)
- **Functions:** 313 (internal 310)
- **Imports / exports:** 73 / 40
- **Interesting strings / decompiled:** 64 / 30

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe: PE32 executable (console) Intel 80386, for MS Windows, 5 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 88K Sep  2  2017 /workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe

# sha256
8e6b1f22947a6df814fc2f4be8169998e1f2bd1effc2becf51029eff3107f6a9  /workspace/recovered-drive-tools/_extracted/gs2-compiler/GS2 Compiler/BytecodeViewer/ViewerApp/BytecodeViewer.exe
```

## Key functions

entry@00402b5d | FUN_00404bdd@00404bdd | FUN_004072c4@004072c4 | FUN_00401ba5@00401ba5 | FUN_0040397c@0040397c | FUN_00401030@00401030 | FUN_00409690@00409690 | FUN_0040c120@0040c120 | FindHandler@0040b748 | FUN_00403515@00403515 | FUN_00406c36@00406c36 | FID_conflict:___crtMessageBoxW@00408a58 | FUN_004044ff@004044ff | _raise@00407fc5 | setSBUpLow@004082e3 | FUN_00402fe1@00402fe1 | parse_cmdline@0040658b | FUN_00406eb0@00406eb0 | FUN_00402e92@00402e92 | FID_conflict:_free@004018ca | FUN_00405faf@00405faf | FUN_004029e2@004029e2 | FUN_00402541@00402541 | doexit@0040435f | _malloc@0040192d | _LocaleUpdate@004046ba | FUN_004071e2@004071e2 | FUN_0040ae13@0040ae13 | FID_conflict:_wprintf@004019f1 | FUN_00407ef2@00407ef2

## Top functions

FUN_00404bdd@00404bdd(sz=2918) | FUN_004072c4@004072c4(sz=2090) | FUN_00401ba5@00401ba5(sz=1935) | FUN_0040397c@0040397c(sz=1872) | FUN_00401030@00401030(sz=1765) | FUN_00409690@00409690(sz=1427) | FUN_0040c120@0040c120(sz=1427) | FindHandler@0040b748(sz=923) | ___free_lc_time@0040a9b0(sz=896) | FUN_00403515@00403515(sz=677) | FUN_00406c36@00406c36(sz=575) | strtoxl@00404742(sz=548) | __setmbcp_nolock@004086c4(sz=501) | __crtLCMapStringA_stat@0040a4f6(sz=489) | FID_conflict:___crtMessageBoxW@00408a58(sz=482)

## Sample interesting strings

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
```

## Decompiled

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

## Imports (head)

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
```

