# Lab #465: Ato 4.04r2 on the lab Modern world (2026-11-13)

This is the last open #465 check, the level links. The other four checks are carried over from the earlier #465 runs on 2026-11-12. Everything ran in Mirae's Linux RE Lab VM, headless, on loopback only.

## The five checks

| Check | Result | Source |
|---|---|---|
| Login | pass | 2026-11-12, Ato c166b73 with `--first-level` on Modern 14805 (allowed-versions floor lowered to G3D17026): `New login: … [G3D17026 (4.042)]`, no kick, no unknown packet; `onlinestartlocal.nw` at (30, 30.5), 57 props. Seen again today: `[02:12 AM] :: New login: lab465a [New Client (2.22+)] [G3D17026 (4.042)]` |
| Server list | pass | 2026-11-12 GUI run: 5 servers from the lab listserver; "Lab Modern" logged in as 4.04r2 and drew `onlinestartlocal.nw` (4096 tiles, 3 chests) |
| Walking | pass | 2026-11-12 GUI run: (30, 30.5) to (25, 26). Today's runs below also walk |
| Other players | pass | 2026-11-12 GUI run: a 4.20r8 client at (30, 34); each window drew the other's marker and nickname |
| Links | **pass, both directions** | this run (below), with 4.20r8 as a regression check |

## The link run

**Builds**
- `seihai-ato`: seihai main `b31c2d5`, release build.
- gserver: seihai-gserver-v2 `101a2cce`, built with packet logging. I ran it on a copy of the Modern server folder, with the new levels below, inside a private loopback-only network namespace. That gives a packet log of every warp. The lab's own Modern binary writes no network dump.

**Level change** (only in the Modern lab folder `~/seihai-lab/world/servers/modern/world/`):
- The original is kept as `onlinestartlocal.nw.orig-lab465` (sha256 `39924e98…71f6eba`).
- `lab465_b.nw` is a copy of the original with the three CHEST lines removed.
- The LINK lines added:

```
onlinestartlocal.nw: LINK lab465_b.nw 30 27 2 1 30 32
lab465_b.nw:         LINK onlinestartlocal.nw 30 27 2 1 30 30.5
```

Both links are 2x1 tiles at (30, 27), on open grass, about three tiles north of the start at (30, 30.5).

**Walking.** Ato ran with its window on Xvfb, logged in with `--online 127.0.0.1:14805`, and was walked by holding Up twice for 1.5 s each (xdotool; key times are in `keys-*.txt`). That carries the player from A to B and back to A.

### 4.04r2

Ato stdout (`ato-404r2-stdout.txt`):
```
logged in at 127.0.0.1:14805 as lab465a (4.04r2)
board: 4096 tiles, 227 distinct [...], 1 links, 3 chests, 0 signs
link to lab465_b.nw at 30 32
level lab465_b.nw: from the server's board, start (30, 32), 0 NPCs (0 from the server, 0 in the file)
board: 4096 tiles, 227 distinct [...], 1 links, 0 chests, 0 signs
link to onlinestartlocal.nw at 30 30.5
level onlinestartlocal.nw: from the server's board, start (30, 30.5), 0 NPCs (0 from the server, 0 in the file)
link to lab465_b.nw at 30 32
level lab465_b.nw: from the server's board, start (30, 32), ...
```

The gserver's log (`gserver-networkdump.txt`):
```
[02:12:07] > In Packet from lab465a [ID 3]: [0] PLI_LEVELWARP (14 bytes)      lab465_b.nw
[02:12:07] < Out Packet to lab465a [ID 3]: [6] PLO_LEVELNAME (12 bytes)       lab465_b.nw
[02:12:14] > In Packet from lab465a [ID 3]: [30] PLI_LEVELWARPMOD (27 bytes)  onlinestartlocal.nw
[02:12:14] < Out Packet to lab465a [ID 3]: [6] PLO_LEVELNAME (20 bytes)       onlinestartlocal.nw
[02:12:14] > In Packet from lab465a [ID 3]: [30] PLI_LEVELWARPMOD (19 bytes)  lab465_b.nw
[02:12:14] < Out Packet to lab465a [ID 3]: [6] PLO_LEVELNAME (12 bytes)       lab465_b.nw
```

`a-3-after-up2.png` shows `lab465_b.nw` drawn, with the return link outlined.

### 4.20r8 (regression)

The same steps with account `lab465b`:
- Login: `New login: lab465b [New Client (2.22+)] [G3D20126 (4.208)]`.
- Ato logged A to B, B to A, A to B, and B to A.
- The gserver logged `PLI_LEVELWARP` to `lab465_b.nw` at 02:13:04, then `PLI_LEVELWARPMOD` at 02:13:05 (`onlinestartlocal.nw`), 02:13:11 (`lab465_b.nw`) and 02:13:11 (`onlinestartlocal.nw`). Each warp was answered with `PLO_LEVELNAME`.

**Kicks and unknown packets:** none in either run. The console has no kick line, and neither the console nor the dump has an "unknown" line. The only console errors are the listserver connection attempts: no listserver runs in the namespace, so the gserver accepts logins in localhost mode.

**Side notes:**
- The gserver sends `PLO_LEVELNAME` twice per warp.
- The first entry into a level uses `PLI_LEVELWARP`. Later entries, with the level already cached, use `PLI_LEVELWARPMOD`.

## Files

- `ato-404r2-stdout.txt` and `ato-420r8-stdout.txt`: Ato stdout
- `gserver-console.txt` and `gserver-networkdump.txt`: the gserver's log. The login password is masked by the gserver. The dump also holds one earlier 4.04r2 login at 02:11, where the keys did not reach the window.
- `link-lines.txt`: the LINK lines and the level files' hashes
- `keys-*.txt`: when each key was held
- `a-*.png` and `b-*.png`: screenshots before and after each walk
