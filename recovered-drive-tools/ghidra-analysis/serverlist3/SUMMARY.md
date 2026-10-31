# Salvage analysis: serverlist3

- **Binary:** `/workspace/recovered-drive-tools/_extracted/server-lister/Server Lister/slist/serverlist3` (also under `_inbox/extracted/Server Lister/slist/` and Developer Tools Server Lister extract)
- **Analyzed (UTC):** 20261031T204000Z (artifacts) · SUMMARY rewritten 20261031T211700Z
- **Ghidra:** 12.2
- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing).

## Purpose guess

Classic **Graal list / login server** (ELF32, statically linked, not stripped). One process listens for client and gserver connections, serves the public server directory, and backs account auth against MySQL. Takes a **full path to a configuration file** as argv; refuses to start if that file is missing. Emits console lines such as “Server lister started” with lister/gserver port context, player-limit accounting, and disconnect notices.

## Recommended placement (`SALVAGE_PLACEMENT.md`)

| Placement | `analysis-only` |
| --- | --- |
| Why | Legacy GPL-adjacent listserver binary; behaviour notes only. Prefer the recovered C# listserver rewrite tree under `_extracted/listserver-nested/graal-listserver/` for any modern rehost design. |
| Not | `setup-core` / `lat-core` / `seihai` / `seihai-rc` |
| Related | Win32 `ListServer` (Delphi), Server Lister package scripts (`gserverstats.pl`) |
| Licence risk | Legacy/static glibc+MySQL client — never paste into MIT crates |

## Identity

```
# file
ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), statically linked, for GNU/Linux 2.2.5, not stripped

# size
~3.3M

# sha256
eb5272ab1653f93fc00b41368caa52d7af565f16e3d575a18e4c1ba1e1da87c3
```

## Ghidra program

| Field | Value |
| --- | --- |
| Format | Executable and Linking Format (ELF) |
| Language | x86:LE:32:default |
| Compiler | gcc |
| Functions | 2197 (internal 2197) |
| Imports | 0 (static link) |
| Exports / entries | 4486 |
| Interesting strings | 2375 |
| Decompiled funcs | 50 |

## Behaviour (plain prose)

Config keys observed in the image include **clientsport**, **serversport**, and **playerlimit**. Runtime side-files include `maxplayers.txt`, `playercount.txt`, and `playerstats.txt` for free/used player budget.

SQL shapes (behaviour only): account rows with encrypted password, email, block/ban fields, admin level/worlds; serverlist rows with name, language, description, homepage, version, playercount, ip, port, and a freshness window on `lastwrite` (~10 minutes). Connection refusals surface as wrong-password or “not allowed to connect from” style messages. GServer side traffic is framed as world connections (“GServer for world … connected”).

Network stack is classic Berkeley sockets plus a bundled MySQL C client (TCP and UNIX-socket path strings, SSL option names). Compression helpers (zlib/deflate) are present. Typoed UI/ops string “Configuraton not found” confirms this era build.

## Prefer source / rewrite over deep RE

- `_extracted/listserver-nested/graal-listserver/` — C# listserver rewrite (primary reference for seihai-facing design).
- Server Lister package companions for ops HTML/stats scripts — notes only.

## Rebuild / rehost checklist

- [x] Identify language/runtime: static ELF32 C++/gcc + MySQL client + zlib.
- [x] CLI: config file path required as sole parameter.
- [x] Config keys: clientsport, serversport, playerlimit; side-files for player budgets.
- [x] Protocols: client list/login + gserver registration; MySQL account/serverlist tables.
- [ ] Prefer clean-room / C# rewrite notes over binary reuse.

## Artifacts

- `prelim/`, `functions.txt`, `functions.csv`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt` (local; do not commit decompiled C)
