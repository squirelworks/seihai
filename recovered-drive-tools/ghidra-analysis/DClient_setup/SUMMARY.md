# Salvage analysis: setup

- **Binary:** `/workspace/recovered-drive-tools/_extracted/dclient-setup/setup.exe`
- **Analyzed (UTC):** 20261031T210043Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

Microsoft Visual Basic 6 **Package & Deployment** bootstrapper (`setup.exe`) for a "DClient" installer. Strings: `CabFile`, `SETUP.LST`, `VerInstallFileA`, Uninstall registry keys. Companion `DClient.CAB` in the same truncated zip is incomplete (~207 KiB of ~3.0 MiB), so payload (`DClient.exe`, `msvbvm60.dll`, …) cannot be fully recovered from this archive.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `stay-staged (bad archive)` → eventually `out-of-scope` (client installer) |
| --- | --- |
| Why | Client-side installer fragment; incomplete Drive zip (EOCD missing; Drive source size = 242431 B same as local). |
| Not | Any seihai-servertools crate |
| Licence risk | Unknown until complete CAB; treat as untrusted |

## Identity

```
# file
/workspace/recovered-drive-tools/_extracted/dclient-setup/setup.exe: PE32 executable (GUI) Intel 80386, for MS Windows, 4 sections

# size
-rw-r--r-- 1 ubuntu ubuntu 137K Mar 13  2000 /workspace/recovered-drive-tools/_extracted/dclient-setup/setup.exe

# sha256
7052d75548d0f34e290baf29aa7281b44b4eb38327a9078354e15a3dc8749da4  /workspace/recovered-drive-tools/_extracted/dclient-setup/setup.exe
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Portable Executable (PE) |
| Language | x86:LE:32:default |
| Compiler | windows |
| Functions | 435 (internal 426) |
| Imports | 183 |
| Exports / entries | 2 |
| Interesting strings | 78 |
| Decompiled funcs | 25 |

## Notable symbols (names only)

Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour.

Full local function lists stay in untracked `functions.txt` / `functions.csv` under the analysis dir.

## Imports

```
ADVAPI32.DLL :: AdjustTokenPrivileges
ADVAPI32.DLL :: LookupPrivilegeValueA
ADVAPI32.DLL :: OpenProcessToken
ADVAPI32.DLL :: RegCloseKey
ADVAPI32.DLL :: RegCreateKeyA
ADVAPI32.DLL :: RegEnumKeyExA
ADVAPI32.DLL :: RegOpenKeyExA
ADVAPI32.DLL :: RegQueryInfoKeyA
ADVAPI32.DLL :: RegQueryValueExA
ADVAPI32.DLL :: RegSetValueExA
COMDLG32.DLL :: GetOpenFileNameA
GDI32.DLL :: CreateFontIndirectA
GDI32.DLL :: DeleteObject
GDI32.DLL :: GetDeviceCaps
GDI32.DLL :: GetStockObject
GDI32.DLL :: GetTextMetricsA
GDI32.DLL :: SelectObject
GDI32.DLL :: SetBkColor
GDI32.DLL :: SetTextColor
KERNEL32.DLL :: CloseHandle
KERNEL32.DLL :: CompareStringA
KERNEL32.DLL :: CompareStringW
KERNEL32.DLL :: CopyFileA
KERNEL32.DLL :: CreateDirectoryA
KERNEL32.DLL :: CreateFileA
KERNEL32.DLL :: CreateProcessA
KERNEL32.DLL :: DeleteFileA
KERNEL32.DLL :: DosDateTimeToFileTime
KERNEL32.DLL :: ExitProcess
KERNEL32.DLL :: FileTimeToLocalFileTime
KERNEL32.DLL :: FileTimeToSystemTime
KERNEL32.DLL :: FindClose
KERNEL32.DLL :: FindFirstFileA
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
KERNEL32.DLL :: GetDriveTypeA
KERNEL32.DLL :: GetEnvironmentStrings
KERNEL32.DLL :: GetEnvironmentStringsW
KERNEL32.DLL :: GetExitCodeProcess
KERNEL32.DLL :: GetFileAttributesA
KERNEL32.DLL :: GetFileSize
KERNEL32.DLL :: GetFileType
KERNEL32.DLL :: GetFullPathNameA
KERNEL32.DLL :: GetLastError
KERNEL32.DLL :: GetLocalTime
KERNEL32.DLL :: GetModuleFileNameA
KERNEL32.DLL :: GetModuleHandleA
KERNEL32.DLL :: GetOEMCP
KERNEL32.DLL :: GetPrivateProfileStringA
KERNEL32.DLL :: GetProcAddress
KERNEL32.DLL :: GetShortPathNameA
KERNEL32.DLL :: GetStartupInfoA
KERNEL32.DLL :: GetStdHandle
KERNEL32.DLL :: GetStringTypeA
KERNEL32.DLL :: GetStringTypeW
KERNEL32.DLL :: GetSystemDirectoryA
KERNEL32.DLL :: GetSystemTime
KERNEL32.DLL :: GetTempFileNameA
KERNEL32.DLL :: GetTempPathA
KERNEL32.DLL :: GetTimeZoneInformation
KERNEL32.DLL :: GetVersion
KERNEL32.DLL :: GetVersionExA
KERNEL32.DLL :: GetWindowsDirectoryA
KERNEL32.DLL :: GlobalAddAtomA
KERNEL32.DLL :: GlobalAlloc
KERNEL32.DLL :: GlobalDeleteAtom
KERNEL32.DLL :: GlobalFindAtomA
KERNEL32.DLL :: GlobalFree
KERNEL32.DLL :: GlobalLock
KERNEL32.DLL :: GlobalUnlock
KERNEL32.DLL :: HeapAlloc
KERNEL32.DLL :: HeapCreate
```

_… 103 more in `imports.txt`_

## Exports / entry points

```
entry [Function]
entry [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
runtime error
TLOSS error
SING error
DOMAIN error
R6024  - not enough space for _onexit/atexit table
R6018  - unexpected heap error
R6017  - unexpected multithread lock error
Runtime Error!  Program:
wsprintfA
wvsprintfA
AdjustTokenPrivileges
OpenProcessToken
GetFileVersionInfoA
GetFileVersionInfoSizeA
VERSION.dll
GetLastError
SetErrorMode
GetVersionExA
GetVersion
GetCPInfo
GetEnvironmentStrings
GetEnvironmentStringsW
CONFIG:
attrib -r %s > nul
WININIT.INI
Error ExitWindows Error #%d
AdjustTokenPrivileges Error #%d in fDoReboot
OpenProcessToken Error #%d in fDoReboot().
SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce
RegisterTypeLib of %s failed : %lx
LoadTypeLib of %s failed : %lx
"HKEY_LOCAL_MACHINE\%s\SharedDlls", "%s"
/REGSERVER
-e %d
"FAILED"
SOFTWARE\Microsoft\Windows\CurrentVersion
SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
FileVersion
ProductVersion
File was not found or was an older version -- new file copied
ERROR:
WARNING:
Error writing to logfile '|1'
Setup fatal error: Unable to generate installation log file.
PLEASE DO NOT MODIFY OR DELETE THIS FILE!
It will be used to automatically remove all application components from your computer if you choose to do so.
There was an unexpected problem trying to create or write to the logfile '|1'.  Error number is |2.
Error attempting to run the application removal program to remove temporary installation files.
Please insert disk |1.
Cannot find file '|1'.  Please insert disk '|2'.
An error occurred while registering the file '|1'
Unexpected error while registering file '|1'
LoadLibrary() failed while registering file '|1'
DllRegisterServer() in the file '|1' returned failure
One possible cause for this error is that RPC may not be installed on your computer.
Cannot update system file '|1'. You may not have permission to update system files on this machine.
Setup has run into a conflict with a running application.  Please close all applications and select OK to try again.
Please verify the drive door is closed and that the disk is formatted and free of errors.
Could not read from the file named:
The destination file is in-use.  Please ensure that all other applications are closed.
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

