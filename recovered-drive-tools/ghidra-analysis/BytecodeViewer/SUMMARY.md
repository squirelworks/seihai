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

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `FindHandler`, `raise`, `setSBUpLow`, `parse_cmdline`, `doexit`, `malloc`, `LocaleUpdate`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `FindHandler`, `free_lc_time`, `strtoxl`, `setmbcp_nolock`, `crtLCMapStringA_stat`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Sample interesting strings

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
```

## Decompile note

Decompiled C was produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour.

## Imports (head)

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
```

