# Salvage analysis: RemoteControl3

- **Binary:** `Developer Tools/# Graal Client Tools And Sources/RC [Remote Control]/windows_rc3_beta.rar.extracted/RemoteControl3.exe`
- **Also same PE in:** Main Client `v5/Graalv5.rar.extracted/…/RemoteControl3.exe`
- **Analyzed (UTC):** 2026-10-31 · Ghidra 12.2 · static only
- **Format:** PE32 GUI (x86), Windows toolchain, ~2.2 MB, heavily sectioned (16)

## Purpose (plain behaviour)

Staff **Remote Control** client for classic Graal servers. UI is **GTK** on Windows (gdk/glib/pixbuf imports) rather than Delphi.

Observed behaviour from symbols and strings (no offsets):

- Shows a **Graal Servers** list, version field, password field (optional “don’t save password”), and **Connect**.
- Talks to a **listserver / server pick** path (`serverlist_connect`, `ConnectToSelectedServer`, `startframe_connect`) and to an **NPC server** (`connectToNPCServer`, “NPCServer connected” label).
- After connect: **player list** (refresh, mass PM, admin message), **NPC list / local NPCs**, level-NPC dump, folder/config pages, profile / rights / account / access edit windows.
- Staff actions reflected in UI copy: ban time/reason, warp-to-player, warp players, disconnect, update level, away-mode when closing player list, optional “separate NC from RC chat”.
- Embeds **bzip2/zlib** compress helpers (`handle_compress`, inflate/deflate) and password encrypt/decrypt helpers — treat as opaque wire behaviour for clean-room notes.
- Syntax highlighters mention `text/x-graalscript`, ini, pascal, cpp — in-RC script editing surface.

**Do not run** this PE against public hosts; lab loopback only if ever needed.

## Recommended placement

| Tag | `seihai-rc` |
| --- | --- |
| Why | Live RC/NC session, staff file/NPC/player tooling over the wire. |
| Also | Raw PE + Ghidra dumps stay staged here (`analysis-only` artifacts). |
| Not | `setup-core`, `lat-core` (until shared edit crates exist), MIT paste of decompile |
| Licence | Proprietary/legacy client — clean-room protocol notes only |

## Ghidra stats (machine)

| Field | Value |
| --- | --- |
| Functions | ~3653 (internal ~3562) |
| Imports | ~529 (Win32 + GTK stack + Winsock/IPHLPAPI) |
| Interesting strings | ~405 |
| Decompiled (local only) | 25 — **not committed** |

Named helpers (names only): `mainSort`, `Init`, `serverlist_connect`, `connectToNPCServer`, `ConnectToSelectedServer`, `encryptPassword` / `decryptPassword`, `bindSocket` / `connectSocket`, `handle_compress`, window builders for player list / profile / rights / account / options / access.

## Artifacts (local, gitignored except this SUMMARY)

`prelim/`, `functions.txt`, `imports.txt`, `exports.txt`, `strings_interesting.txt`, `decompiled/`, `logs/`, `summary_stats.txt`
