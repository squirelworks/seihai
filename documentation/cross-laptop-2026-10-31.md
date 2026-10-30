# Cross-laptop lab: Linux servers, mixed clients (2026-10-31)

Two laptops on one LAN, with no public servers involved. The Linux laptop ran the lab listserver and GServer-v2, plus a Seihai watching client that sat in the start level for the whole cycle. The Windows laptop ran each real client in turn: it logged in, walked, opened the chest, said "hi", swung and threw, then quit. This note records what each side saw. Times are UTC (2026-10-30 evening UTC, 2026-10-31 local). It contains no credentials and no decompile detail.

## Sources

- The gserver log for this run (`logs/gserver-crosslaptop-2026-10-31.out` in the lab server folder) and `logs/listserver.log`.
- The watcher's stderr logs: `watcher-seihai222-2026-10-31.log` (build 72ed21d) and `watcher-seihai222-main-4b45cd3-2026-10-31.log` (build 4b45cd3). Also its `_debug/*/packets.log`, which has only packet ids and lengths, no payloads.
- The saved account files after each logout (position and chest flags).
- The Windows side's Wi-Fi captures and decodes on the Windows laptop: `seihai-lab-172\logs\lan-217-decode.txt`, `lan-222-decode.txt`, `lan-231-decode.txt` and `lan-205-decode.txt`, plus screenshots in `seihai-lab-172\screens\`.
- What the user saw on both screens, reported during the runs.
- Seihai source at 72ed21d and 4b45cd3, and the GServer-v2 beta4 source.

## Setup

| Part | What ran |
|------|----------|
| Listserver | Lab `listserver.py` on 0.0.0.0:14921–14923, advertising 192.168.0.122 |
| Gserver | GServer-v2 beta4 365bbd2 with the lab patches (`gserver-beta4-lab.patch`), "Seihai LAN Test" on 14802, generation `newmain`. Stock beta4 cannot get 2.05 or 2.17 into the game. |
| Allowed versions | `newmain = GNW24011:GNW28015,GNW07080` (2.00–2.31, plus 1.39) |
| Start level | `new1.nw` at 30,32: one link (west edge, to `testworld.nw`), one sign (30,22), one chest (30,36), one bush (26,26) and one rock (34,26) |
| Watcher | Seihai `--version 2.22` on Linux, logged in through 14922. It ran build 72ed21d for 2.17–2.31, then 4b45cd3 (main) for 2.00, 2.05 and Seihai-on-Windows. |
| Windows client | The test account, with the nickname Win-Tester. Its position, HP and the new1 chest flag were reset between runs while it was offline. |

The gserver ran without a restart for every client run. It was restarted once, before the cycle, to load the version list and start level. The listserver was restarted once, between the 2.17 attempts, to change the lab account's password.

## Results

| Run | Joined? | Watcher saw the Windows player | Windows saw the watcher | Chat | Chest | Combat | Problems |
|-----|---------|--------------------------------|-------------------------|------|-------|--------|----------|
| 2.05 rev 2 (GNW03081) | Yes, 16:34–16:37 | Yes: 372 OTHERPLPROPS | Yes, but its nick showed as "guest" | "hi" from Windows went out as CURCHAT. The watcher's own chat did **not** reach 2.05. | Opened on Windows (flag saved) | Throws hurt both ways. The 4b45cd3 watcher's swings hurt 2.05, which died twice and sent CLAIMPKER each time. 2.05's swings did **not** hurt the watcher. | 11 FILESENDFAILED after 4 WANTFILE. It first had to reach the lab listserver through a hosts line (see below). |
| 2.17 (GNW22122) | Yes, 16:06–16:12 | Moves (306 OTHERPLPROPS), throws and board changes | Yes, "Linux-Watcher" by the sign | "hi" sent once as CURCHAT. The watcher showed no bubble in the Windows screenshot, and the watcher side could not be checked. | Opened on Windows (flag saved). The watcher's own chest stayed closed, since chests are per account. | Thrown bushes and rocks hurt both ways. Swords did not land either way (72ed21d watcher). | First 3 logins refused for a wrong password. No disconnect and no version rejection. |
| 2.22 (GNW03014) | Yes, 16:13–16:15 | Moves (308 OTHERPLPROPS), throws, board changes. **Not** hiding under the rock. | Yes | "hi" sent as CURCHAT | Opened on Windows (flag saved) | Throws only. Swords did not land (72ed21d watcher). | The watcher drew the wrong level (see below) |
| 2.31 (GNW28015) | Yes, 16:19–16:22 | Moves (185 OTHERPLPROPS). A rock throw hit the watcher (Hurt, 2 half-hearts). | Yes, by the sign | "hi" bubble showed on Windows | Opened (rupees 3→4, flag saved) | 4 sword swings, none landed (72ed21d watcher) | Own player showed only a shadow for about 30 s while assets loaded |
| 2.00 (GNW24011) | **No** | Nothing | Not in the game | n/a | n/a | n/a | Gets the server list (shape A, twice). After Connect it opens an empty grey window, holds no sockets, and never connects to the gserver. Cause: the client folder had no `objects.dat` (see the lab setup notes). |
| Seihai 4b45cd3 as 2.22, on Windows | Logged in at 16:41, then froze | Only the join (2 OTHERPLPROPS), then nothing | A red "guest" marker at about 30,36 | None | None | None | Froze after its first frames: one 768 ms slow frame ("look images 446"), no input, and no client-to-server bytes after login. It did draw the server's `new1.nw` correctly. |
| 1.39 (GNW07080), GNW30123 | Not run | | | | | | GNW30123 is not in gs2lib. With it in `allowedversions.txt`, the gserver aborts at startup in `loadAllowedVersions` (null `getVersionString`), so it was left out. |

Real clients saw each other's moves, throws and chests normally wherever both were in. Every fault below is in Seihai, or in the lab setup.

## Seihai findings

These went to the Seihai fix PRs #194, #195 and #196.

1. **Sword hits, received.** In every version the victim's client judges a sword hit, and `PLI_HURTPLAYER` is only used by the `hitplayer` script command. No run had any HURTPLAYER in either direction. Seihai judges an incoming swing only from the attacker's SPRITE prop (`receive_sword`, `client/src/combat.rs`), but 2.17, 2.22 and 2.31 swing with GANI=sword. So their swings never hurt the watcher. The 2.05 swings didn't hurt it on 4b45cd3 either: all three of the watcher's HP drops in that run came right after a thrown object landed. No "hit by" or swim lines were logged. Why 2.05's swings were refused is still open (#194).
2. **Sword hits, sent.** Build 72ed21d sent its swings as SPRITE only, so 2.17 drew the swing but took no damage. Build 4b45cd3 includes #168 and #190 and sends GANI=sword, and 2.05 took those hits and died twice.
3. **Hiding.** 2.22 sent GANI lift → carrystill → hiddenstill/hidden, CARRYSPRITE=1 and STATUS 20→22 (bit 0x02). The watcher received the props, but `other_pose` (`client/src/render/others.rs`) maps only idle, walk, sword, hurt, sit, carrystill, carry and dead. Everything else falls back to SPRITE and then idle, and STATUS 0x02 is never checked. So the hiding player was drawn standing. The carried object itself is drawn (#195).
4. **Level source.** The watcher drew its bundled `graal-2.22/new1.nw`, which is the distro's indoor room with one NPC at 30,7, not the server's board. `level_from_board = resolver.level(n).is_none()` (`client/src/bin/main.rs`) lets a local file of the same name win over PLO_BOARDPACKET and LEVELMODTIME. The server's BOARDMODIFY tiles were painted onto the wrong map. Seihai on Windows had no local copy, so it drew the server's level correctly (#196).
5. **UDP.** The watcher binds no UDP socket and sends no UDPPORT prop (`client/src/protocol/login.rs`). The 2.05 peer channel in `client/src/net/udp.rs` isn't used online. 2.17 bound no UDP either, and the 2.17 capture shows no UDP between the laptops.
6. **Nickname.** Real clients showed the watcher as "guest", which is its account name, not the nickname in the account file.
7. **Chat out.** The watcher's chat, sent as CURCHAT, did not show on 2.05.
8. **Idle kick.** The server's `maxnomovement = 1200` disconnected the idle watcher once, at 15:50. It was relaunched before 2.17.

## Lab setup notes

- **2.00 and 2.05 look up `www.graalonline.com`** (Cloudflare, with rotating IPs such as 104.26.3.198 and 172.67.71.216), not `listserver.graal.in`. An early attempt went to a resolved IP, which made it look hard-coded. What worked was a hosts line `192.168.0.122 www.graalonline.com` on the Windows laptop. A route plus a `lo` alias for one IP only caught the address that had been cached.
- **The Windows laptop had no admin rights,** so the 14921–14923 traffic from 2.17, 2.22, 2.31 and Seihai went through a small TCP relay on 127.0.0.1 there. The hosts line was added by the user.
- **2.00 needs its own `objects.dat`.** The 2.00 folder was a copy of the 2.22 distro, which has none. 2.00 checks for the file when Start is pressed and sets a fatal flag if it is missing. The list still works, but Connect then returns before it dials, which leaves the grey window. The list reply and the gserver were not at fault: 2.00 reads the IP from field 6 and the port from field 7, as sent. Addresses are in Graal_analysis `client-internals/v2.00-connect-path.md`. Confirmed on 2026-10-30 at about 17:17 UTC: with NewMain 26's `objects.dat` copied into the folder and the hosts line back, 2.00 opened no dialog at Start or Connect. It joined "Seihai LAN Test" over TCP to 192.168.0.122:14802, drew the server's `new1.nw` with 3 hearts, and chatted (capture `captures\lan-200b.pcapng`, screenshot `screens\lan-200b-2.png`). The gserver logged the login as `GNW24011 (2.0)`, with no rejection and no disconnect. A Seihai watcher relaunched at 17:18 UTC (build 4b45cd3, `--version 2.22`, in `new1.nw` at 30,32) received 56 OTHERPLPROPS from the 2.00 player, 1 THROWCARRIED and 1 BOARDMODIFY in its first 30 seconds. The watcher had been kicked for inactivity at 17:02 UTC and was not in the game when 2.00 joined.
- **Naming the lab level `new1.nw`** collided with the distro file of the same name, which caused Seihai finding 4. Future labs should use a name no client ships with.
- **Linux screenshots weren't possible from a script.** GNOME refuses the D-Bus screenshot call, and Seihai's F12 screenshot needs a key press in its window.
- **The watcher's packet log** records only ids and lengths. Positions, facing and chat on the watcher side come from its stderr and from what the user saw, not from decoded props.
- **Cleanup:** remove the hosts lines on the Windows laptop (`graalonline.com`, `www.graalonline.com`) and its 104.26.3.198 route. On the Linux laptop, run `sudo ip addr del 104.26.3.198/32 dev lo`.

## The Windows freeze (follow-up)

Seihai's stderr from the Windows run stops at the first slow frame, after "position 30 32 dir 1". The first frame queues its file requests, and the post-login frame is due on the second, so the very first send after the reader started never went out. The login itself went out before the reader started. The same build on Linux, against a stand-in 2.22 server with the same flags, sends its post-login frame at about 0.3 s and walks normally.

Up to 4b45cd3 the reader thread did a blocking read on a `try_clone` of the socket while the game loop wrote on the original handle. On Windows the clone is a `WSADuplicateSocket` handle of the same socket, and with a filter driver or LSP in the path, I/O on one socket can be serialised. A write stuck behind a read that only ends on the next server packet fits everything seen: no bytes out, almost no CPU, keys ignored, and a process Windows could not end. This is the likely cause, not a confirmed one. Since the fix, one thread owns the socket for both directions with the socket non-blocking, and the game loop's sends are queued to it.

The game loop also has a stall watch now. A pass that stops for 3 s is written to `_debug/stall.log` (under the working directory) and to stderr, naming the last part of the pass that finished. If Windows still freezes, that line names the part.
