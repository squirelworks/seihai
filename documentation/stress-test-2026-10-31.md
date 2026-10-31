# Stress test (2026-10-31)

This is a stress test of Seihai itself. It ran many clients at once, long
sessions, rapid warps and level changes, chest, item and combat churn,
reconnects, and bad or oversized server packets. It covered 1.39, 1.41r1,
2.05, 2.22 and every build from 5.00r7 (5.07) to 6.03r7 (6.037): 5.00r7,
5.12, 5.24r4, 5.30r3, 5.31r4, 6.00r2, 6.015, 6.034, 6.03r7 and 6.03r7 Linux.
6.036 and the iPhone builds were not tested.

All load ran in a cloud container against a private GServer-v2 (`efb5b90`)
on 127.0.0.1. No public server, listserver or real account was involved, and
the gserver never contacted a listserver (`listip = 127.0.0.1`, `listport = 1`).
The harness is in `tools/stress/` and its README covers setup.

## Setup

- **Servers.** A game gserver on 14802, and a second one on 14801 acting as
  the Login lobby. The lobby answers a 4.x+ login with
  `PLO_SERVERWARP 0,Lab Game,127.0.0.1,14802`, so 5.x and 6.x clients go
  through the real lobby → server-warp → re-login path. Lab-only server
  patch: `tools/stress/gserver-lab-patch/`.
- **World.** An 8-level warp ring (script `setlevel2` every 0.3–0.5 s,
  alternating with a link under the arrival spot) and a 2x2 gmap with
  baddies. An arena with 12 baddies and 12 chests, and NPCs that every
  0.05–0.5 s lay rupees and hearts, put bombs, shoot arrows, explode, change
  `showimg` and chat. A hub with links and 20 chests. 200 linked levels for
  the long run.
- **Clients.** One Xvfb display and one HOME per client, driven with
  xdotool by role:
  - ring: chat `warpto` to a random level every 0.2–1.5 s;
  - gmap: long walks across the seams;
  - arena: sword, action and short walks every 0.05–0.4 s;
  - hub: chat spam of 1–60 characters, and walks through the edge links.
- **Reconnect churn.** Random clients got SIGTERM, SIGKILL, or SIGKILL with
  a relaunch 50 ms later.

## Runs

| run | builds | clients | length | churn |
|---|---|---|---|---|
| b1 | 1.39, 1.41r1, 2.05, 2.22 | 8 | 12 min | 27 launches |
| b2 | 5.00r7, 5.12, 5.24r4 | 6 | 12 min | 17 launches |
| b3 | 5.30r3, 5.31r4, 6.00r2 | 6 | 12 min | 23 launches |
| b4 | 6.015, 6.034, 6.03r7, 6.03r7 Linux | 8 | 12 min | 20 launches |
| long | 1.39, 2.22, 5.00r7, 6.03r7 | 8 | 30 min | none; warps over 200 levels |
| many | all 14 builds | 28 | 8 min | 46 launches |
| fault proxy | all 14 builds × 6 modes | 1 each | 30 s each | 84 sessions, 1350 faults |
| fuzz | parsers in-process | — | ~1.6 M inputs | — |

## Results by build

The b1–b4 numbers are from the runs at `871ed75` (main). Memory is RSS. The
slope is a linear fit over each launch after its first 30 s, for launches up
for at least 2 min. CPU is percent of one core on a 4-core VM shared with
Xvfb. Each client draws at 60 fps in software at 2x, so CPU is mostly
drawing.

| build | crashes / panics | worst RSS slope MB/min | max RSS MB | mean / max CPU % | worst C>S msgs/s |
|---|---|---|---|---|---|
| 1.39 | 0 | +0.41 | 25 | 27 / 40 | 68 |
| 1.41r1 | 0 | +0.41 | 26 | 29 / 42 | 168 (baddy leader) |
| 2.05 | 0 | +0.28 | 25 | 26 / 40 | 100 |
| 2.22 | 0 | +0.49 | 29 | 27 / 43 | 151 (baddy leader) |
| 5.00r7 | 0 | +0.29 | 25 | 26 / 39 | 145 |
| 5.12 | 0 | +0.54 | 25 | 27 / 36 | see finding 6 |
| 5.24r4 | 0 | +0.34 | 24 | 27 / 36 | 24 |
| 5.30r3 | 0 | +0.18 | 25 | 29 / 44 | 23 |
| 5.31r4 | 0 | +0.06 | 24 | 26 / 43 | 137 |
| 6.00r2 | 0 | +0.02 | 24 | 28 / 47 | 23 |
| 6.015 | 0 | +0.20 | 25 | 24 / 45 | 23 |
| 6.034 | 0 | +0.69 | 26 | 28 / 43 | 138 |
| 6.03r7 | 0 | +0.12 | 25 | 24 / 40 | 24 |
| 6.03r7 Linux | 0 | +0.50 | 26 | 25 / 41 | 135 |

- No client crashed, panicked, hung on SIGTERM or stalled (`stall.log`
  stayed empty) in 87 launches. The gservers stayed up too.
- **Threads and fds.** Every launch held 3 threads and 4–6 fds from start
  to end, so nothing leaked across warps or reconnects.
- **Send rate.** The C>S peaks over 100 msgs/s are all the level leader's
  `BADDYPROPS` in the arena: 12 baddies × one update per 100 ms think. That
  matches `baddies.md` and stays under 1 KB/s. Without baddies, the
  peak is 20–46 msgs/s: walk props, `WANTFILE` at login and `FLAGSET`.
- **Memory.** Growth is slow and mostly from warping. The long run is below.

### Long sessions (fixed build)

Eight clients (1.39, 2.22, 5.00r7, 6.03r7, two each) stayed logged in for
30 minutes with no reconnects. The two warp clients were the 1.39 and
5.00r7 ones. They warped 568 and 580 times over 193 and 196 different
levels, out of the 200 linked `st_w` levels and the ring.

| build | worst RSS slope MB/min | RSS start → end MB | mean / max CPU % |
|---|---|---|---|
| 1.39 | +0.15 | 22–23 → 24–29 | 25 / 38 |
| 2.22 | +0.08 | 23 → 26–27 | 27 / 42 |
| 5.00r7 | +0.15 | 23–24 → 28–29 | 27 / 42 |
| 6.03r7 | +0.27 | 23 → 27–33 | 27 / 41 |

There were no exits, panics, stalls or `warp not answered` lines. Every
client held 3 threads and 6 fds for the whole 30 minutes. The worst client
grew 10 MB in 30 minutes (6.03r7 in the arena). The growth is slow and
may be caches filling; I did not trace it further.

### Many clients (fixed build)

28 clients, two of each of the 14 builds, ran for 8 minutes on the same
4-core VM, with reconnect churn (46 launches in all).

- No client crashed, panicked or exited on its own. Both gservers stayed up.
- One `term_hang` was logged, but it was the harness: the churn thread
  relaunched a 5.12 client 0.8 s after the final SIGTERM went out, so that
  process never got one. `stress.py` no longer relaunches once a run is
  ending.
- With 28 clients on 4 cores, each client got 9–10 % of a core on average,
  and 2274 frames went over 100 ms (the worst 601 ms, almost all of it in
  `draw`). The clients slowed down but kept their connections.
- RSS stayed at 25–28 MB for every build. The worst slope was +0.80 MB/min
  (5.12) over launches of up to 8 minutes. That is short and noisy, and the
  30-minute run above is the better guide.
- C>S peaks were the same as in b1–b4: 95–167 msgs/s from baddy leaders,
  and 19–30 msgs/s for every other client.
- 5.12 logged `warp not answered` 4 times. This is finding 7.

## Findings

### Fixed in this branch

1. **1.39 and 1.41 went silently offline after pushing a wall.** A push
   or pull try sent `PLI_TRIGGERACTION` (38), which those builds cannot
   send. `send_messages` returned `Unsupported`, and the game loop set
   `conn = None` on it. The client then said "not connected" to every chat
   line while its socket stayed open. The server kept the player online
   until its own timeout ("Client stress01 has timed out"), so for those
   minutes the client and the server disagreed. This happened 6 times in b1,
   on both 1.39 and 1.41r1. Fix: `Connection::can_send`, and the try is not
   sent where the build has no 38.
2. **A dropped connection kept its socket.** The socket thread never noticed
   that its `Connection` was gone. It kept the link open and kept reading, so
   every path that sets `conn = None` left a ghost player on the server. The
   thread now closes the socket and reports "closed on Seihai's side" once
   the queue is gone. Regression test:
   `dropping_the_connection_closes_the_socket`.
3. **Compression bombs.** zlib and bz2 bodies were inflated with no bound.
   A bz2 frame of under 64 KB grew a client to 617 MB on 2.22, 812 MB on
   5.24r4 and 890 MB on 6.03r7 Linux. In those 6.x and 5.24r4 runs, one frame
   blocked the loop for 956 ms. A 6.x long frame may be up to 64 MB, which
   is enough zlib for tens of GB. Output is now capped at 32 MB per body
   (`frame::MAX_BODY`). On the fixed build the same runs peak at 28–77 MB and
   end with "connection closed: zlib body is over 32 MB once decompressed".
4. **Images that claim a huge size abort the client.** A 74-byte GIF that
   says 65535 × 65535, served as an NPC image, made the client ask for
   17 GB and abort (exit 134). A 60000 × 60000 PNG asked for 14 GB. PNG and
   GIF sizes over 8192 × 8192 pixels are now refused before any allocation,
   and a PNG's IDAT is inflated only as far as its rows need. On the fixed
   build: "image bomb2.gif not read: image size 65535 x 65535 is over the
   limit".
5. **A `.nw` cut right after a `BADDY` or `SIGN` header panicked**
   (`lines[1..0]` in `parse_baddy` and `parse_sign`). Seeded fuzzing of the
   level reader found it. A cut download or cache file reaches this path.
6. **The capture viewer could not read 5.12 client frames.** Every 5.12
   C>S frame read as `BADFRAME`, because the decoder ignored
   `gen5_round_down`. It also read 6.x server frames with u16 lengths only.
   Both now follow the profile.

### Reported, not fixed

7. **5.12 level links went unanswered on GServer-v2 (`warp not answered`,
   15 times in b2). This was the lab server, not Seihai; the lab patch now
   fixes it.** 5.12 sends every body zlib-compressed and steps the cipher
   once per whole 4-byte group (`len / 4`), but gs2lib steps once per
   started group. The two agree only when a body is a multiple of 4 bytes,
   or 16 bytes or longer. The first short body with any other length put
   the server's cipher a step ahead, and it could read nothing after that.
   Seihai's 5.12 wire matches the real client, so it stays as it is. The
   lab gserver now steps as 5.12 does when `GS_LAB_512_ROUNDDOWN` is set
   (`tools/stress/gserver-lab-patch/*-512-rounddown.patch`). On the patched
   lab server, the unchanged 5.12 build followed the hub's link at once;
   on the stock one, it got 2 `warp not answered`. The real 5.12 needs the
   same server-side change, or the translating proxy, to play on
   GServer-v2.
8. **Each client uses about 25–30 % of a core** at 60 fps under Xvfb. Most
   of it is the software draw at 2x and `present`, so about 10 clients fill
   a 4-core VM. Past that, frames go over 100 ms: 174 times in b1–b4, and
   2274 times (up to 601 ms) in the 28-client run. Nothing broke, but the
   "many clients" numbers are CPU-bound.
   **Fixed later (`seihai::present`).** A profile of an idle 2.22 client
   put 60 % of its CPU in minifb, which scales the whole frame to the
   window and sends every pixel with `XPutImage` on each update, 15 % in
   packing RGB into words, and 3 % in rebuilding the script `board[]`
   with one run scan per tile. The loop now sends a frame only when it
   differs from the last one sent (and at least every 250 ms), packs
   pixels with `chunks_exact`, and builds the board in one pass. Offline
   under Xvfb, an idle client went from 23 % to 4–5 % and a walking one
   from 24 % to 10–13 %; Xvfb's own share fell from 10 % to 1–4 %. In the
   lab (8 × 2.22, 120 s, `run.sh`) the mean went from 23 % to 16 %, with
   no exits, stalls or log lines of note. The arena clients stay near
   23 %: their NPCs change the picture about 16 times a second, close to
   the 2.x 50 ms game frame, and each frame sent is still a full
   `XPutImage` (44 % of that client's CPU).
   **Then on X11 the frames go through shared memory.** `ShmWindow` hands
   them to the window minifb opened through softbuffer's X11 backend,
   which uses MIT-SHM (or a plain `PutImage` when the server has none);
   input still comes through minifb. Seihai does the scaling itself, and
   it picks the same pixels as minifb (screenshots of both paths match
   exactly). On Wayland minifb already uses `wl_shm`, so it keeps
   minifb's path there, as do Windows and macOS. `SEIHAI_NO_SHM=1`
   turns it off. Offline, a walking client went from 17 % to 7 %. In the
   same lab run the mean went from 16 % to 11 %, and the arena clients
   from 23 % to 15–16 %. RSS is about 4 MB higher, which is the shared
   segment, and each client holds one more fd.
9. **The 1.41r1, 2.22 and 5.31r4 leaders send `BADDYPROPS` for every baddy
   every think, even when unchanged.** At 12 baddies that is 120 msgs/s
   from one client. **Checked since: this is what the real clients do.**
   The real 2.22 (`lan-222d`, 5 minutes) and 2.00 (`lan-200west`) leaders
   send one message per baddy per think for as long as they stay in the
   level, 48 to 60 a second for 6 baddies, and leave out only a baddy with
   no change, as Seihai does. Each think's messages go in one frame in both.
   Seihai's rate is kept as it is and pinned by a test (see `baddies.md`).
10. **The fault proxy found no panic in 84 sessions.** Those sessions had
    1350 hostile, oversized, mutated, flooded and broken-framing faults,
    including lying raw-data lengths, 60 KB board lines, warps to 255,255,
    and 5000× floods of one message. In-process fuzzing of `SessionState`,
    every message id, the protocol parsers, levels, ganis, images, GS1 and
    GS2 found only items 4 and 5. A flood or a broken frame ends the
    session cleanly, with "connection closed: …".

## Not covered

- Real client binaries were not run, so item 7 needs a check with a real
  client. Item 9 was checked against the 2.00 and 2.22 captures.
- Only GServer-v2 was tested, not GServer-RS. GS1 NPC scripts do not run on
  5.x/6.x (they need GS2), so the script-driven warps and item churn only
  reached the 1.x and 2.x builds. The 5.x/6.x clients got the chat warps,
  links, walking, sword and the server-side traffic.
- 6.036 and the iPhone builds were not tested.
- Windows was not tested. No user machine was used.
