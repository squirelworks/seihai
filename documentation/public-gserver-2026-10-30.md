# Public game-server checks, 2.05 and 2.22 (2026-10-30)

Laundry item 8. The logins were made from the user's own Linux machine, because the build box cannot reach the listserver ports (see `public-checks.md`). Account `seihaitest`, with the password passed through `--password-env`. Seihai `72ed21d` was run with `--no-serverwarp --no-scripts --no-sound --no-save-downloads`, and `dumpcap` captured one host and port per capture.

The pcaps and logs stay on that machine (folder mode 700, files 600). The servers are named S1 to S3 here, and their hosts and names are not recorded. S1 runs GServer-RS (signature 41). S2 and S3 run gs2 (signature 73). Each listed 0 players at the time.

Sources compared: the lab frame export `seihai-lab/run6.frames.jsonl`, session `run6-2`. That is a real 2.22 client (token `GNW03014`) on GServer-v2. Also compared: `protocol.md`, `login-burst.md`, `public-gserver-run.md` and `version-chain`. No lab frames exist for a 2.05 public refusal, so the 2.05 rows are compared with the docs only.

## Sessions

| session | result |
|---|---|
| S1 / 2.22 | logged in, full burst |
| S2 / 2.22 | logged in, full burst |
| S3 / 2.22 | logged in, full burst |
| S1 / 2.05 | connection closed with no reply (no `DISCMESSAGE`) |
| S2 / 2.05 | one plain zlib frame: `DISCMESSAGE` "Your client version is not allowed on this server. Allowed: 2.22, 1.41r1 - iPhone Belote 1.5", plus the level, then closed |
| S3 / 2.05 | not run: two refusals were treated as a stop sign |

Five game-server logins were made in all. There was no chat, no warp, no level change and no upload; `UPDATEFILE` appears only as the normal modtime check.

**Deviation: the character moved.** The Seihai window took keyboard focus on the user's desktop, so in all three 2.22 sessions the character walked around inside its start level. These moves show up as `PLAYERPROPS` (2) position and sprite updates and as `gr.x` / `gr.y` `FLAGSET`. Future runs should use a window that cannot take key input, either headless or in a separate X session.

## First 10 server frames (2.22)

Each entry gives the wire length and ctype (02 raw, 04 zlib, 06 bz2), then the message ids in order.

| frame | lab run6-2 (GServer-v2) | S1 (GServer-RS) | S2 (gs2) | S3 (gs2) |
|---|---|---|---|---|
| f0 | 3275/06: 25 168 9 194 34 34 43 43 33 33 190 14 6 100 101 39 0 4x3 174 10 42 156 179 41 82 | 3193/06: 25 44 168 9 28x11 34 34 43 43 33 190 14 6 101 39 0 4x3 174 10 42 156 3x4 179 41 82 | 12/02: 30 (idle.gani refused) | 62/04: 33 197 |
| f1 | 154/04: 47 180 182 | 154/04: 47 180 182 | 3640/06: 25 44 168 9 194 34 34 43 43 33 33 190 14 6 101 39 0 174 10 42 156 179 41 82 | 2314/06: 25 44 168 9 194 28 28 34 34 43 43 33 197 33 197 190 14 6 0 101 |
| f2 | 7/02: 42 | 45 file replies | 216/04: 47 180 8 182 | 848/04: 39 0 4x3 174 10 42 156 9 179 41 82 47 180 8 (no 182) |
| f3 to f9 | 45 / 30 / 9 file replies, 42 | 45 file replies, one 30 | 30 refusals, one 102 FILE (2098/06), 42 | 30, 28, one 102 FILE (2089/06) |

`DEFAULTWEAPON` (43) arrives as 8 then 7 on every server, as in the lab.

## Client side (Seihai vs the real 2.22 in run6-2)

| | real 2.22 (lab) | Seihai (public, S1) |
|---|---|---|
| first frame after login | 22/02: 37 LANGUAGE, 39 TAMPERCHECK | 10/02: 37 LANGUAGE |
| idle.gani request | inside the post-burst frame | 12/02: 23 WANTFILE in its own frame, before the server's first frame |
| post-burst frame | 34 x4, 23, 18 18, 44, 2, 24 | 23 x3, 34, 23, 18 18, 44, 2 |

The real client checks its cached files with 34 `UPDATEFILE`; Seihai had an empty cache, so most of its checks were 23 `WANTFILE`.

## Mismatches and new observations

1. **No tamper check.** The real 2.22 sends `TAMPERCHECK` (39) with `LANGUAGE`. Seihai sends it only with `--send-tampercheck`, so it did not here. All three public servers accepted the login without it. This confirms the 2026-10-24 note (`protocol.md`, "Tamper check") on two more server types.
2. **`PLI_PROCESSLIST` is sent, and the docs disagree.** Seihai answers 182 with `"seihai"`, as `login.rs` and `login-burst.md` say (the real client answers too: id 44 in run6-2). The "Deliberately not done" paragraphs in `protocol.md` were stale and are fixed in this change. When a server sends no 182 (S3), Seihai sends no 44, and S3 did not complain.
3. **Empty gen-5 frame.** On S3, C>S f6 was a frame with only the ctype byte and no message. No real-client capture shows such a frame. The cause has not been looked at. It is a Seihai bug to fix, not a server behaviour.
4. **The burst does not always open with `SIGNATURE`.** On S3 the server sent a weapon and `LOADSCRIPT` (197) frame first. On S2 the refusal of the idle.gani request (30) came first, because Seihai asks for it before the burst. Seihai handled both.
5. **The order of `SETACTIVELEVEL` (156).** On all three servers, as in the lab, 156 comes after the board, `LEVELMODTIME` and the chests. `public-gserver-run.md` lists it before the board; that summary was loose, not a different order.
6. **The level layer id depends on the version.** S2 sent the 972-byte layer as id 0 `LEVELBOARD` to 2.22 and as id 7 `BOARDMODIFY` to 2.05.
7. **A silent refusal.** S1 (GServer-RS) closed a 2.05 login without a `DISCMESSAGE`. Seihai reports `connection closed: failed to fill whole buffer`. A clearer message for a close before any frame would help.
8. **Only the lab sends id 100 before the board.** GServer-v2 sends 100 before `BOARDPACKET`; none of the public servers did.
9. **Ids the 2.22 lab session never had.** 44 `HASNPCSERVER` (all three), 28 `FLAGSET` in the burst, 3 `NPCPROPS` (S1), 8 `OTHERPLPROPS` in the 182 frame (S2, S3), 197 `LOADSCRIPT` (S3).
10. **Gaps in the viewer's tables.** `seihai capture show` decoded every frame without error, but it has no layout row for these ids:
    - 2.22 server to client: 0 (non-empty), 3, 4, 28, 33, 43, 101, 197
    - 2.22 client to server: 18, 44
    - 2.05 server to client: 16, 7

    Ids 39 and 180 are only partly decoded. The viewer also does not print the gen-5 ctype.

## What this settles and what stays open

- Settled: gen-5 framing, the cipher and bz2 decoding against three public servers, with no decode errors. Signatures 41 and 73. `DEFAULTWEAPON` 8 then 7 on public servers. Public servers do not enforce the tamper check at login.
- Settled: at least one public gs2 server refuses 2.05 by version (allowed: 2.22, 1.41r1 and an iPhone client).
- Open: a 2.05 game-server session on a public server (none accepted it). The empty frame (item 3). Viewer layouts (item 10).
