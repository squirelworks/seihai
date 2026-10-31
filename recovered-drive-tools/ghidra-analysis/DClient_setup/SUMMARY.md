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
| Image base | 00400000 |
| Functions | 435 (internal 426) |
| Imports | 183 |
| Exports / entries | 2 |
| Interesting strings | 78 |
| Decompiled funcs | 25 |

## Top / key functions

**Largest internal:** FUN_00403d1c@00403d1c(sz=3061) | FUN_00406c0a@00406c0a(sz=2147) | FUN_0040c9a0@0040c9a0(sz=1704) | FUN_004187d0@004187d0(sz=1632) | FUN_0040e6a0@0040e6a0(sz=1533) | FUN_004084e7@004084e7(sz=1529) | FUN_0040d2f0@0040d2f0(sz=1395) | FUN_00404911@00404911(sz=1117) | FUN_0040322c@0040322c(sz=1107) | FUN_0040d870@0040d870(sz=1079) | FUN_0040a480@0040a480(sz=1077) | FUN_0040129d@0040129d(sz=968) | FUN_00406143@00406143(sz=917) | FUN_00418f60@00418f60(sz=898) | FUN_00407720@00407720(sz=895)

**Decompile targets:** entry@00413060 | FUN_00403d1c@00403d1c | FUN_00406c0a@00406c0a | FUN_0040c9a0@0040c9a0 | FUN_004187d0@004187d0 | FUN_0040e6a0@0040e6a0 | FUN_004084e7@004084e7 | FUN_0040d2f0@0040d2f0 | FUN_00404911@00404911 | FUN_0040322c@0040322c | FUN_0040d870@0040d870 | FUN_0040a480@0040a480 | FUN_0040129d@0040129d | FUN_00406143@00406143 | FUN_00418f60@00418f60 | FUN_00405557@00405557 | FUN_0040523a@0040523a | FUN_00405885@00405885 | FUN_0040746d@0040746d | FUN_00407720@00407720 | FUN_00407bf1@00407bf1 | FUN_00403c25@00403c25 | FUN_00411230@00411230 | FUN_0040680e@0040680e | FUN_0040b8d0@0040b8d0

See `functions.txt` / `functions.csv` for the full list.

## Imports

```
ADVAPI32.DLL :: AdjustTokenPrivileges @ EXTERNAL:00000038
ADVAPI32.DLL :: LookupPrivilegeValueA @ EXTERNAL:0000003c
ADVAPI32.DLL :: OpenProcessToken @ EXTERNAL:0000003a
ADVAPI32.DLL :: RegCloseKey @ EXTERNAL:0000003b
ADVAPI32.DLL :: RegCreateKeyA @ EXTERNAL:0000003e
ADVAPI32.DLL :: RegEnumKeyExA @ EXTERNAL:00000039
ADVAPI32.DLL :: RegOpenKeyExA @ EXTERNAL:00000040
ADVAPI32.DLL :: RegQueryInfoKeyA @ EXTERNAL:0000003f
ADVAPI32.DLL :: RegQueryValueExA @ EXTERNAL:00000041
ADVAPI32.DLL :: RegSetValueExA @ EXTERNAL:0000003d
COMDLG32.DLL :: GetOpenFileNameA @ EXTERNAL:00000037
GDI32.DLL :: CreateFontIndirectA @ EXTERNAL:00000003
GDI32.DLL :: DeleteObject @ EXTERNAL:00000004
GDI32.DLL :: GetDeviceCaps @ EXTERNAL:00000005
GDI32.DLL :: GetStockObject @ EXTERNAL:00000001
GDI32.DLL :: GetTextMetricsA @ EXTERNAL:00000008
GDI32.DLL :: SelectObject @ EXTERNAL:00000007
GDI32.DLL :: SetBkColor @ EXTERNAL:00000006
GDI32.DLL :: SetTextColor @ EXTERNAL:00000002
KERNEL32.DLL :: CloseHandle @ EXTERNAL:0000009f
KERNEL32.DLL :: CompareStringA @ EXTERNAL:000000b0
KERNEL32.DLL :: CompareStringW @ EXTERNAL:00000083
KERNEL32.DLL :: CopyFileA @ EXTERNAL:00000099
KERNEL32.DLL :: CreateDirectoryA @ EXTERNAL:000000b5
KERNEL32.DLL :: CreateFileA @ EXTERNAL:00000056
KERNEL32.DLL :: CreateProcessA @ EXTERNAL:0000007f
KERNEL32.DLL :: DeleteFileA @ EXTERNAL:000000a4
KERNEL32.DLL :: DosDateTimeToFileTime @ EXTERNAL:00000074
KERNEL32.DLL :: ExitProcess @ EXTERNAL:0000006b
KERNEL32.DLL :: FileTimeToLocalFileTime @ EXTERNAL:0000006c
KERNEL32.DLL :: FileTimeToSystemTime @ EXTERNAL:0000006f
KERNEL32.DLL :: FindClose @ EXTERNAL:0000009c
KERNEL32.DLL :: FindFirstFileA @ EXTERNAL:000000a0
KERNEL32.DLL :: FlushFileBuffers @ EXTERNAL:00000081
KERNEL32.DLL :: FreeEnvironmentStringsA @ EXTERNAL:00000091
KERNEL32.DLL :: FreeEnvironmentStringsW @ EXTERNAL:00000090
KERNEL32.DLL :: FreeLibrary @ EXTERNAL:00000080
KERNEL32.DLL :: GetACP @ EXTERNAL:00000055
KERNEL32.DLL :: GetCPInfo @ EXTERNAL:00000059
KERNEL32.DLL :: GetCommandLineA @ EXTERNAL:00000064
KERNEL32.DLL :: GetCurrentDirectoryA @ EXTERNAL:00000067
KERNEL32.DLL :: GetCurrentProcess @ EXTERNAL:000000b7
KERNEL32.DLL :: GetCurrentProcessId @ EXTERNAL:0000005c
KERNEL32.DLL :: GetDriveTypeA @ EXTERNAL:0000007b
KERNEL32.DLL :: GetEnvironmentStrings @ EXTERNAL:0000008f
KERNEL32.DLL :: GetEnvironmentStringsW @ EXTERNAL:00000087
KERNEL32.DLL :: GetExitCodeProcess @ EXTERNAL:000000a6
KERNEL32.DLL :: GetFileAttributesA @ EXTERNAL:00000070
KERNEL32.DLL :: GetFileSize @ EXTERNAL:0000008c
KERNEL32.DLL :: GetFileType @ EXTERNAL:0000006e
KERNEL32.DLL :: GetFullPathNameA @ EXTERNAL:000000a7
KERNEL32.DLL :: GetLastError @ EXTERNAL:000000b4
KERNEL32.DLL :: GetLocalTime @ EXTERNAL:00000065
KERNEL32.DLL :: GetModuleFileNameA @ EXTERNAL:0000009a
KERNEL32.DLL :: GetModuleHandleA @ EXTERNAL:0000007a
KERNEL32.DLL :: GetOEMCP @ EXTERNAL:00000093
KERNEL32.DLL :: GetPrivateProfileStringA @ EXTERNAL:0000008d
KERNEL32.DLL :: GetProcAddress @ EXTERNAL:0000007e
KERNEL32.DLL :: GetShortPathNameA @ EXTERNAL:000000ac
KERNEL32.DLL :: GetStartupInfoA @ EXTERNAL:00000063
KERNEL32.DLL :: GetStdHandle @ EXTERNAL:00000062
KERNEL32.DLL :: GetStringTypeA @ EXTERNAL:00000085
KERNEL32.DLL :: GetStringTypeW @ EXTERNAL:00000084
KERNEL32.DLL :: GetSystemDirectoryA @ EXTERNAL:00000073
KERNEL32.DLL :: GetSystemTime @ EXTERNAL:00000066
KERNEL32.DLL :: GetTempFileNameA @ EXTERNAL:000000b2
KERNEL32.DLL :: GetTempPathA @ EXTERNAL:000000b6
KERNEL32.DLL :: GetTimeZoneInformation @ EXTERNAL:0000006a
KERNEL32.DLL :: GetVersion @ EXTERNAL:00000072
KERNEL32.DLL :: GetVersionExA @ EXTERNAL:00000071
KERNEL32.DLL :: GetWindowsDirectoryA @ EXTERNAL:00000077
KERNEL32.DLL :: GlobalAddAtomA @ EXTERNAL:00000088
KERNEL32.DLL :: GlobalAlloc @ EXTERNAL:00000053
KERNEL32.DLL :: GlobalDeleteAtom @ EXTERNAL:0000008a
KERNEL32.DLL :: GlobalFindAtomA @ EXTERNAL:000000ab
KERNEL32.DLL :: GlobalFree @ EXTERNAL:00000052
KERNEL32.DLL :: GlobalLock @ EXTERNAL:000000aa
KERNEL32.DLL :: GlobalUnlock @ EXTERNAL:000000a9
KERNEL32.DLL :: HeapAlloc @ EXTERNAL:00000069
KERNEL32.DLL :: HeapCreate @ EXTERNAL:0000005f
```

_… 103 more in `imports.txt`_

## Exports / entry points

```
entry @ 00413060 [Function]
entry @ 00413060 [function]
```

## Interesting strings

Useful for CLI flags, config paths, URLs, protocol tokens.

```
0041a818	runtime error
0041a828	TLOSS error
0041a838	SING error
0041a848	DOMAIN error
0041a918	R6024  - not enough space for _onexit/atexit table
0041a97c	R6018  - unexpected heap error
0041a9a0	R6017  - unexpected multithread lock error
0041aacc	Runtime Error!  Program:
0041b0f8	wsprintfA
0041b15c	wvsprintfA
0041b2ca	AdjustTokenPrivileges
0041b2fa	OpenProcessToken
0041b428	GetFileVersionInfoA
0041b43e	GetFileVersionInfoSizeA
0041b456	VERSION.dll
0041b646	GetLastError
0041b7a6	SetErrorMode
0041b848	GetVersionExA
0041b86e	GetVersion
0041ba1e	GetCPInfo
0041ba90	GetEnvironmentStrings
0041baa8	GetEnvironmentStringsW
0041c084	CONFIG:
0041c13c	attrib -r %s > nul
0041c18c	WININIT.INI
0041c1d0	Error ExitWindows Error #%d
0041c1ec	AdjustTokenPrivileges Error #%d in fDoReboot
0041c230	OpenProcessToken Error #%d in fDoReboot().
0041c268	SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce
0041c29c	RegisterTypeLib of %s failed : %lx
0041c2c0	LoadTypeLib of %s failed : %lx
0041c434	"HKEY_LOCAL_MACHINE\%s\SharedDlls", "%s"
0041c62c	/REGSERVER
0041c6b8	-e %d
0041c8cc	"FAILED"
0041c9d8	SOFTWARE\Microsoft\Windows\CurrentVersion
0041ca04	SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
004288ba	FileVersion
00428b4a	ProductVersion
00428c54	File was not found or was an older version -- new file copied
00428cd0	ERROR:
00428cde	WARNING:
00428d18	Error writing to logfile '|1'
00428d54	Setup fatal error: Unable to generate installation log file.
00428dce	PLEASE DO NOT MODIFY OR DELETE THIS FILE!
00428eb4	It will be used to automatically remove all application components from your computer if you choose to do so.
00428f90	There was an unexpected problem trying to create or write to the logfile '|1'.  Error number is |2.
00429102	Error attempting to run the application removal program to remove temporary installation files.
0042939c	Please insert disk |1.
004293ca	Cannot find file '|1'.  Please insert disk '|2'.
00429470	An error occurred while registering the file '|1'
004294d4	Unexpected error while registering file '|1'
0042959e	LoadLibrary() failed while registering file '|1'
00429670	DllRegisterServer() in the file '|1' returned failure
004296dc	One possible cause for this error is that RPC may not be installed on your computer.
00429d30	Cannot update system file '|1'. You may not have permission to update system files on this machine.
00429eae	Setup has run into a conflict with a running application.  Please close all applications and select OK to try again.
0042a03e	Please verify the drive door is closed and that the disk is formatted and free of errors.
0042a190	Could not read from the file named:
0042a270	The destination file is in-use.  Please ensure that all other applications are closed.
```

## Decompiled C

| File | Function | Address |
| --- | --- | --- |
| `entry_00413060.c` | `entry` | `00413060` |
| `FUN_00403d1c_00403d1c.c` | `FUN_00403d1c` | `00403d1c` |
| `FUN_00406c0a_00406c0a.c` | `FUN_00406c0a` | `00406c0a` |
| `FUN_0040c9a0_0040c9a0.c` | `FUN_0040c9a0` | `0040c9a0` |
| `FUN_004187d0_004187d0.c` | `FUN_004187d0` | `004187d0` |
| `FUN_0040e6a0_0040e6a0.c` | `FUN_0040e6a0` | `0040e6a0` |
| `FUN_004084e7_004084e7.c` | `FUN_004084e7` | `004084e7` |
| `FUN_0040d2f0_0040d2f0.c` | `FUN_0040d2f0` | `0040d2f0` |
| `FUN_00404911_00404911.c` | `FUN_00404911` | `00404911` |
| `FUN_0040322c_0040322c.c` | `FUN_0040322c` | `0040322c` |
| `FUN_0040d870_0040d870.c` | `FUN_0040d870` | `0040d870` |
| `FUN_0040a480_0040a480.c` | `FUN_0040a480` | `0040a480` |
| `FUN_0040129d_0040129d.c` | `FUN_0040129d` | `0040129d` |
| `FUN_00406143_00406143.c` | `FUN_00406143` | `00406143` |
| `FUN_00418f60_00418f60.c` | `FUN_00418f60` | `00418f60` |
| `FUN_00405557_00405557.c` | `FUN_00405557` | `00405557` |
| `FUN_0040523a_0040523a.c` | `FUN_0040523a` | `0040523a` |
| `FUN_00405885_00405885.c` | `FUN_00405885` | `00405885` |
| `FUN_0040746d_0040746d.c` | `FUN_0040746d` | `0040746d` |
| `FUN_00407720_00407720.c` | `FUN_00407720` | `00407720` |
| `FUN_00407bf1_00407bf1.c` | `FUN_00407bf1` | `00407bf1` |
| `FUN_00403c25_00403c25.c` | `FUN_00403c25` | `00403c25` |
| `FUN_00411230_00411230.c` | `FUN_00411230` | `00411230` |
| `FUN_0040680e_0040680e.c` | `FUN_0040680e` | `0040680e` |
| `FUN_0040b8d0_0040b8d0.c` | `FUN_0040b8d0` | `0040b8d0` |

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

