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
| Functions | 303 (internal 301) |
| Imports | 158 |
| Exports / entries | 13 |
| Interesting strings | 34 |
| Decompiled funcs | 30 |

## Notable symbols (names only)

Observed application/helper symbols (no addresses): `crtLCMapStringA`, `sbh_free_block`, `sbh_alloc_block`, `sbh_resize_block`, `memcpy`, `memmove`, `tzset`, `cvtdate`, `strlen`, `malloc`, `free`, `realloc`.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
ADVAPI32.DLL :: AdjustTokenPrivileges
ADVAPI32.DLL :: LookupPrivilegeValueA
ADVAPI32.DLL :: OpenProcessToken
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCreateKeyExA
ADVAPI32.DLL :: RegDeleteValueA
ADVAPI32.DLL :: RegOpenKeyExA
ADVAPI32.DLL :: RegQueryValueExA
ADVAPI32.DLL :: RegSetValueExA
COMDLG32.DLL :: GetOpenFileNameA
GDI32.DLL :: GetStockObject
GDI32.DLL :: GetTextMetricsA
KERNEL32.DLL :: CloseHandle
KERNEL32.DLL :: CompareStringA
KERNEL32.DLL :: CompareStringW
KERNEL32.DLL :: CopyFileA
KERNEL32.DLL :: CreateDirectoryA
KERNEL32.DLL :: CreateFileA
KERNEL32.DLL :: CreateProcessA
KERNEL32.DLL :: DeleteFileA
KERNEL32.DLL :: ExitProcess
KERNEL32.DLL :: FindClose
KERNEL32.DLL :: FindFirstFileA
KERNEL32.DLL :: FindNextFileA
KERNEL32.DLL :: FlushFileBuffers
KERNEL32.DLL :: FreeEnvironmentStringsA
KERNEL32.DLL :: FreeEnvironmentStringsW
KERNEL32.DLL :: FreeLibrary
KERNEL32.DLL :: GetACP
KERNEL32.DLL :: GetCPInfo
KERNEL32.DLL :: GetCommandLineA
KERNEL32.DLL :: GetCurrentDirectoryA
KERNEL32.DLL :: GetCurrentProcess
KERNEL32.DLL :: GetCurrentProcessId
KERNEL32.DLL :: GetCurrentThreadId
KERNEL32.DLL :: GetEnvironmentStrings
KERNEL32.DLL :: GetEnvironmentStringsW
KERNEL32.DLL :: GetEnvironmentVariableA
KERNEL32.DLL :: GetExitCodeProcess
KERNEL32.DLL :: GetFileType
KERNEL32.DLL :: GetFullPathNameA
KERNEL32.DLL :: GetLastError
KERNEL32.DLL :: GetLocaleInfoA
KERNEL32.DLL :: GetModuleFileNameA
KERNEL32.DLL :: GetModuleHandleA
KERNEL32.DLL :: GetOEMCP
KERNEL32.DLL :: GetPrivateProfileStringA
KERNEL32.DLL :: GetProcAddress
KERNEL32.DLL :: GetStartupInfoA
KERNEL32.DLL :: GetStdHandle
KERNEL32.DLL :: GetStringTypeA
KERNEL32.DLL :: GetStringTypeW
KERNEL32.DLL :: GetSystemInfo
KERNEL32.DLL :: GetSystemTimeAsFileTime
KERNEL32.DLL :: GetTickCount
KERNEL32.DLL :: GetTimeZoneInformation
KERNEL32.DLL :: GetVersionExA
KERNEL32.DLL :: GlobalAlloc
KERNEL32.DLL :: GlobalLock
KERNEL32.DLL :: GlobalUnlock
KERNEL32.DLL :: HeapAlloc
KERNEL32.DLL :: HeapCreate
KERNEL32.DLL :: HeapDestroy
KERNEL32.DLL :: HeapFree
KERNEL32.DLL :: HeapReAlloc
KERNEL32.DLL :: HeapSize
KERNEL32.DLL :: LCMapStringA
KERNEL32.DLL :: LCMapStringW
KERNEL32.DLL :: LoadLibraryA
KERNEL32.DLL :: MultiByteToWideChar
KERNEL32.DLL :: OpenProcess
KERNEL32.DLL :: QueryPerformanceCounter
KERNEL32.DLL :: ReadFile
KERNEL32.DLL :: RemoveDirectoryA
KERNEL32.DLL :: RtlUnwind
KERNEL32.DLL :: SetEndOfFile
KERNEL32.DLL :: SetEnvironmentVariableA
KERNEL32.DLL :: SetFilePointer
KERNEL32.DLL :: SetHandleCount
KERNEL32.DLL :: SetStdHandle
```

_… 78 more in `imports.txt`_

## Exports / entry points

```
FID_conflict:__fread_lk [function]
FID_conflict:__time32 [function]
copy_environ [function]
cvtdate [function]
entry [Function]
entry [function]
findenv [function]
flsall [function]
report_failure [function]
setSBCS [function]
write_char [function]
write_multi_char [function]
write_string [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
A_OSVERSION
HideAutoItDebug
AutoIt Error
Line %d: %s  Error: %s
ERROR
Select File
HKEY_CURRENT_CONFIG
Powered by AutoIt - http://www.hiddensoft.com/AutoIt/
ERRORLEVEL
runtime error
TLOSS error
SING error
DOMAIN error
R6024  - not enough space for _onexit/atexit table
R6018  - unexpected heap error
R6017  - unexpected multithread lock error
This application has requested the Runtime to terminate it in an unusual way. Please contact the application's support team for more information.
Runtime Error!  Program:
A security error of unknown cause has been detected which has corrupted the program's internal state.  The program cannot safely continue execution and must now be terminated.
Unknown security failure detected!
GetLastError
GetEnvironmentVariableA
GetEnvironmentStringsW
GetEnvironmentStrings
GetCPInfo
GetVersionExA
AdjustTokenPrivileges
OpenProcessToken
Third-party compiled AutoIt script.  For details visit http://www.hiddensoft.com/AutoIt/
FileVersion
ProductVersion
Stack underflow error (Return with no matching Gosub, or, EndRepeat with no matching Repeat).
Unable to process shutdown (access rights error).
AutoIt Error
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

