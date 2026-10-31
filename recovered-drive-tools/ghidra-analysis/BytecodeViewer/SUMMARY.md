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
| Functions | 313 (internal 310) |
| Imports | 73 |
| Exports / entries | 40 |
| Interesting strings | 64 |
| Decompiled funcs | 30 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `FindHandler`, `free_lc_time`, `strtoxl`, `setmbcp_nolock`, `crtLCMapStringA_stat`, `raise`, `setSBUpLow`, `parse_cmdline`, `doexit`, `malloc`, `LocaleUpdate`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
KERNEL32.DLL :: AreFileApisANSI
KERNEL32.DLL :: CloseHandle
KERNEL32.DLL :: CreateFileW
KERNEL32.DLL :: DecodePointer
KERNEL32.DLL :: DeleteCriticalSection
KERNEL32.DLL :: EncodePointer
KERNEL32.DLL :: EnterCriticalSection
KERNEL32.DLL :: ExitProcess
KERNEL32.DLL :: FlushFileBuffers
KERNEL32.DLL :: FreeEnvironmentStringsW
KERNEL32.DLL :: GetACP
KERNEL32.DLL :: GetCPInfo
KERNEL32.DLL :: GetCommandLineA
KERNEL32.DLL :: GetConsoleCP
KERNEL32.DLL :: GetConsoleMode
KERNEL32.DLL :: GetCurrentProcess
KERNEL32.DLL :: GetCurrentProcessId
KERNEL32.DLL :: GetCurrentThreadId
KERNEL32.DLL :: GetEnvironmentStringsW
KERNEL32.DLL :: GetFileType
KERNEL32.DLL :: GetLastError
KERNEL32.DLL :: GetModuleFileNameA
KERNEL32.DLL :: GetModuleFileNameW
KERNEL32.DLL :: GetModuleHandleExW
KERNEL32.DLL :: GetModuleHandleW
KERNEL32.DLL :: GetOEMCP
KERNEL32.DLL :: GetProcAddress
KERNEL32.DLL :: GetProcessHeap
KERNEL32.DLL :: GetStartupInfoW
KERNEL32.DLL :: GetStdHandle
KERNEL32.DLL :: GetStringTypeW
KERNEL32.DLL :: GetSystemTimeAsFileTime
KERNEL32.DLL :: HeapAlloc
KERNEL32.DLL :: HeapFree
KERNEL32.DLL :: HeapQueryInformation
KERNEL32.DLL :: HeapReAlloc
KERNEL32.DLL :: HeapSize
KERNEL32.DLL :: InitializeCriticalSectionAndSpinCount
KERNEL32.DLL :: InterlockedDecrement
KERNEL32.DLL :: InterlockedIncrement
KERNEL32.DLL :: IsDebuggerPresent
KERNEL32.DLL :: IsProcessorFeaturePresent
KERNEL32.DLL :: IsValidCodePage
KERNEL32.DLL :: LCMapStringW
KERNEL32.DLL :: LeaveCriticalSection
KERNEL32.DLL :: LoadLibraryExW
KERNEL32.DLL :: LoadLibraryW
KERNEL32.DLL :: MultiByteToWideChar
KERNEL32.DLL :: OutputDebugStringW
KERNEL32.DLL :: QueryPerformanceCounter
KERNEL32.DLL :: RaiseException
KERNEL32.DLL :: ReadConsoleInputA
KERNEL32.DLL :: ReadConsoleW
KERNEL32.DLL :: ReadFile
KERNEL32.DLL :: RtlUnwind
KERNEL32.DLL :: SetConsoleMode
KERNEL32.DLL :: SetConsoleTextAttribute
KERNEL32.DLL :: SetEndOfFile
KERNEL32.DLL :: SetFilePointer
KERNEL32.DLL :: SetFilePointerEx
KERNEL32.DLL :: SetLastError
KERNEL32.DLL :: SetStdHandle
KERNEL32.DLL :: SetUnhandledExceptionFilter
KERNEL32.DLL :: Sleep
KERNEL32.DLL :: TerminateProcess
KERNEL32.DLL :: TlsAlloc
KERNEL32.DLL :: TlsFree
KERNEL32.DLL :: TlsGetValue
KERNEL32.DLL :: TlsSetValue
KERNEL32.DLL :: UnhandledExceptionFilter
KERNEL32.DLL :: WideCharToMultiByte
KERNEL32.DLL :: WriteConsoleW
KERNEL32.DLL :: WriteFile
```

## Exports / entry points

```
CPtoLocaleName [function]
CallCatchBlock [function]
CatchIt [function]
Catch_All [function]
ExFilterRethrow [function]
FID_conflict:GetTableIndexFromLocaleName [function]
FID_conflict:_AtlDownlevelLocaleNameToLCID [function]
FID_conflict:_CallMemberFunction1 [function]
FID_conflict:___CxxFrameHandler3 [function]
FID_conflict:___crtMessageBoxW [function]
FID_conflict:__dosmaperr [function]
FID_conflict:_free [function]
FID_conflict:_wopen [function]
FID_conflict:_wprintf [function]
FindHandler [function]
FindHandlerForForeignException [function]
TranslatorGuardHandler [function]
Unwind [function]
doexit [function]
entry [Function]
entry [function]
fast_error_exit [function]
flsall [function]
getSystemCP [function]
operator_new [function]
parse_cmdline [function]
setSBCS [function]
setSBUpLow [function]
siglookup [function]
std::exception::exception [function]
std::exception::exception [function]
std::exception::exception [function]
std::exception::operator= [function]
strtoxl [function]
terminate [function]
type_info::operator== [function]
write_char [function]
write_multi_char [function]
write_string [function]
x_ismbbtype_l [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
R6017  - unexpected multithread lock error
R6018  - unexpected heap error
R6024  - not enough space for _onexit/atexit table
R6033  - Attempt to use MSIL code from this assembly during native code initialization This indicates a bug in your application. It is most likely the result of calling an MSIL-compiled (/clr) function from a native constructor or from DllMain.
R6034  - inconsistent onexit begin-end variables
DOMAIN error
SING error
TLOSS error
runtime error
@R6002  - floating point support not loaded
Runtime Error!  Program:
ja-JP
zh-CN
ko-KR
zh-TW
MM/dd/yy
MM/dd/yy
en-US
de-DE
sl-SI
hi-IN
bn-IN
pa-IN
ta-IN
te-IN
kn-IN
sa-IN
de-CH
sr-SP-Latn
se-SE
de-AT
sr-SP-Cyrl
fr-CH
de-LI
bs-BA-Latn
sr-BA-Latn
bn-in
bs-ba-latn
de-at
de-ch
de-de
de-li
en-tt
fr-ch
hi-in
id-id
kn-in
pa-in
sa-in
se-se
sl-si
sr-ba-latn
sr-sp-cyrl
sr-sp-latn
ta-in
te-in
`placement delete closure'
GetLastError
IsDebuggerPresent
SetLastError
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

