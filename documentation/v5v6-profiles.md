# 5.x and 6.x profiles

Date: 2026-10-31. Work order `claude-laundry-v5v6-2026-10-31.md`, items 1, 2, 3, 5 and 6. Item 4 (the 5.24 file layout) is in PR #280; item 7 (the 6.x lab check) is the 6.x thread's.

Sources, all in Graal_analysis: `protocol/v5x-clients.md`, `protocol/v6x-clients.md`, `protocol/v5v6-windows-unpacked.md`, `protocol/v5v6-lab-play.md`, `version-profiles/lab-v5-2026-10-30.md`, and the new `version-profiles/5.00-6.03.toml` (the same facts in table form, read by `tools/gen_profile_fixture.py`). The 6.x rows also use the 6.x thread's notes (`v5v6/6x-findings.md` in the project folder). No wire byte here is new: every one is in a capture or a note.

## The rows

| Label | Token | Server ids | Client ids | What is special |
|---|---|---:|---|---|
| 5.00r7 | `G3D22067` | 100 | binary floor, see below | four login fields |
| 5.12 | `G3D14097` | 100 | as 5.00r7 | five login fields; round-down cipher steps; sends every frame compressed |
| 5.24r4 | `G3D15010` | 97 | as 5.00r7 | stock gs2lib does not know the token (PR #280) |
| 5.30r3 | `G3D26090` | 95 | as 5.00r7 | runs only with the clock set back |
| 5.31r4 | `G3D26090` | 95 | as 5.00r7 | draws no tiles before the warp (Seihai does) |
| 6.00r2 | `G3D26090` | 93 | Linux 6.01r1's (inferred) | Tempus Sans ITC 24 |
| 6.015 | `G3D3007A` | 91 (Linux 6.01r1's, inferred) | Linux 6.01r1's (inferred) | **no binary**: the token 6.00r7 and 6.01r5 send, from GServer-v2's `allowedversions.txt`; stock gs2lib knows it |
| 6.034 | `G3D2505C` | 92 (Linux 6.03r7's, inferred) | Linux 6.03r7's (inferred) | **no binary**: the token 6.03r4 sends, from the same file; stock gs2lib knows it |
| 6.03r7 | `G3D0311C` | 92 | Linux 6.03r7's (60) | Tempus Sans ITC 24 |
| 6.03r7 Linux | `G3D0511C` | 92 | Linux 6.03r7's (60) | the same release as 6.03r7; logs in with `linux` and four machine ids (all sent as `""`) |
| 6.036 | **missing** | – | – | no binary and no token: **no row**. It is only named by 5.00r7's updater packages (`Graal6.exe` 6.036, probably 6.03r6). |

Every row is client type 5, gen 5, no tamper check, no showimg timer, and follows 4.20r8 in the version chain (the 4.x rows, `documentation/v4-profiles.md`, came later the same day; the lobby, the list and the chat are now set there) (`documentation/version-chain.md` lists each step's changes and unknowns).

- **Client ids.** 5.00r7's send wrappers give only the constant ids, a floor. The row is that floor, plus 11 and 38 (seen in the lab), plus the ids both 3.153 and 6.03r7 send (0, 3, 20, 27, 30, 46 and the rest of the 2.x set). 40 and 43 are in neither 5.00r7 nor 6.03r7 and are left out. The 5.12 to 5.31r4 send code was not read, so they keep 5.00r7's set.
- **Props.** No 5.x or 6.x prop parser was read. The rows keep 3.153's own and other prop sets (X2/Y2/Z2 included, which the lab saw these clients send).
- **Chat.** 5.x draws a bold sans serif (5.24's config says `font=Arial`): a new `ChatFace::Arial`. 6.00r2 and 6.03r7 use Tempus Sans ITC 24.

## The Login lobby (`protocol::lobby`)

A row with `protocol.lobby` set is a 5.x/6.x build:

- **No listserver.** Seihai logs in to the address it is given as a game server and names it `Login`, as the clients do. The listserver step is skipped for these profiles even when the address looks like a listserver. The client's own built-in address, `loginserver.graalonline.com:14900`, is kept as history (`ServerList::Direct`).
- **Login line.** After the password comes an unprefixed comma list: `win`, three machine fields, and from 5.12 on the quoted Windows version `"6.2 9200 "` (what every capture shows). Seihai does not read the machine: it sends each of the three fields as `""`, which is the value the real 5.00r7 sent for its MAC field under Wine. GServer-v2 took all three empty in the lab check below.
- **Then** id 37 (language) in its own gen-5 frame, as captured. The base-file requests that follow in the captures (`basepackage.gupd` by 23 or 47) are not sent.
- **Login mode.** While the current server name starts with "login" (any case), Seihai draws the level, chests and NPC images, but not its own player, other players, chat over heads or the HUD. That is what the real clients do (5.00r7, 5.12, 5.24r4, 5.31r4, 6.00r2 and 6.03r7 in the lab). Seihai still draws gani NPCs and baddies there; the lab saw no gani NPCs drawn and had no baddies on the Login level. 5.31r4 and 6.03r7 (and the Linux 6.03r7, the same release) also draw no tiles there, only the chests and image NPCs, as the real ones did in the lab where 5.24r4 and 5.30r3 drew them (Graal_analysis `protocol/v5v6-lab-play.md`); `Lobby::login_tiles` holds it, and 6.00r2, not recorded, keeps the tiles.
- **A server picked from a list is not the Login server.** The real clients always reach the Login server first and only learn a game server's name from its 178. When the server was picked from Seihai's server list (its name saved with the address in `last_server_name`), Seihai starts under that name, as if the Login server had already warped there, so the player, other players and the HUD are drawn from the start. This is what makes 6.03r7 on GServer-RS and 6.00r2 on hastur playable: neither sends a 178. An address typed straight in (no saved name) still starts as `Login`, and a list name that starts with "login" still means Login mode (`lobby::first_server_name`).
- **Gani props with parameters.** From 4.x on a server sends an NPC's gani prop as the name and its parameters, `zlttp_gralats,3`. The file asked for and loaded is the part before the first comma (`npcart::ani_file`); the prop itself is kept as sent.
- **Id 178.** A `PLO_SERVERWARP` is followed as for every build (`--no-serverwarp` turns it off, and then a lobby build stays in Login mode, with a warning). The lobby builds split the text **on commas only**: the lab's unquoted `0,Lab Server,127.0.0.1,14900` reached 127.0.0.1:14900 and the window title read "Lab Server" in every build, where the 2.x comma-text reader would split the name at its space. The test `lab_warp_bytes_for_the_lobby_builds` feeds the exact lab bytes (`0xD2` + that text) to every lobby profile. After the warp the new name is the server's name, Login mode ends, and the player, the HUD and other players are drawn on the flat path.
- **3.x stays held.** The 5.x/6.x tokens start with `G3D`, as the 3.x ones do, but these builds run in `seihai-mae`, not `seihai-ato`, and nothing opens Torque.

Seihai is a client, so it receives the 178; it cannot send one. On the lab's GServer-v2 the warp only comes from the proxy (`g5mitm.py` with `rules.py`), because GServer-v2 sends 178 only when its listserver answers a server-info query.

## 5.12's cipher step count

5.12 steps the gen-5 iterator `len / 4` times, rounded down, on zlib and bz2 frames (still at most 4), leaves a partial last group alone, and sends every frame compressed (`Gen5::round_down`, `protocol.gen5_round_down`). Every other build rounds up. This is faithful to the client, so a 5.12 profile talks to a server that steps as 5.12 does. GServer-v2 does not: its short bodies then throw the two ciphers out of step, and the server stops reading the client. The lab either puts its translating proxy between them (it keys on the `G3D14097` token) or runs the gserver with `tools/stress/gserver-lab-patch/*-512-rounddown.patch` and `GS_LAB_512_ROUNDDOWN=1`, which steps 5.12's compressed bodies rounded down.

## Lab check (item 6)

Run on 2026-10-31 against the commit merged in #281, all on loopback: a copy of the lab's GServer-v2 build (with `G3D22067,G3D14097` allowed), a listserver copy (GServer-v2 refuses every login without one), and the lab proxy in front of it, which also injected the 178.

- **Login.** 5.00r7 and 5.12 were both accepted; the server logged them as `G3D22067 (5.07)` and `G3D14097 (5.12)`. Then Seihai sent `37 English`, `46`, two `2` packets and file requests by 23 and 34.
- **182 never came.** GServer-v2 did not send S>C 182 in any run, so the proxy injected `0,Lab Server,127.0.0.1,<proxy port>` after 30 s.
- **Warp.** Seihai followed it every time and logged in again. Before the warp: level, chests and minimap only. After it: the player with shield, hearts, the A/S/D counters and the HUD, and the server's welcome PM. Both builds looked the same.
- **5.12 cipher.** The proxy decoded every 5.12 frame with round-down stepping and saw no illegal data, so Seihai's 5.12 stepping matches.
- **S>C ids seen** (same on both builds): 0, 4, 6, 8, 9, 10, 14, 25, 34, 37, 39, 41, 42, 43, 44, 45, 47, 82, 100, 101, 156, 168, 174, 179, 180, 190, 194. Of these 82, 190 and 194 (all empty here) have no case in `dispatch.rs`; 100 is the raw-data prefix. `seihai-mae` counts unhandled ids but does not print them.
- **The second login is turned away if it comes too fast.** Seihai closes the old socket and connects again within about 10 ms. GServer-v2 drops the old player only 1 to 5 s later, and until then answers the new login with a `double login` disconnect or S>C 16 `Account is already in use.` The runs passed with the proxy holding the new login 4 s (5.00r7) and 8 s (5.12) before it connected. The real clients do the same thing. In the 5.00r7 binary (`GraalEngine.dll`), the 178 handler stores the text, the next frame tick calls `connectToServer`, and that function deletes the old connection and makes the new one in the same call, with no wait. The network thread polls every 1 ms. S>C 16 only stores the text for the reconnect dialog, and nothing retries the login after a warp. The only automatic reconnect is to the backup login hosts, when the server is named exactly `Login` and the connect itself fails. The real clients got through in the earlier labs because they log in as guest, and GServer-v2 skips its in-use check for guests (`TPlayerLogin.cpp`). A real client with a named account would get the same refusal. 5.12's code was not read; it comes from the same tree three months later. Seihai still makes its first try at once, as the clients do, and then retries after that refusal (next section).

## Warp reconnect retry (named accounts)

Seihai adds one thing the real clients do not have. When the login that follows a 178 is answered with a `PLO_DISCMESSAGE` (16) saying the account is in use (`Account is already in use.`, or the lab's `double login` wording) within 10 s of that login, Seihai waits and logs in to the same host and port again, with a new key each time, as the first try. The waits are 1, 2, 4, 8 and 16 s, 31 s in all. GServer-v2 lets a new login take over an account whose old player has sent nothing for more than 30 s (`TPlayerLogin.cpp`), so the last try gets in even if the old socket never closes. The first try still goes at once, so a server that drops the old player in time sees exactly the real client's reconnect. Any other close, a later one, or a sixth refusal ends the session with the server's text as before. The chat shows `account still in use on the new server, trying again in N s` while it waits. Code: `seihai::net::warp_retry`.

Lab check, 2026-10-31, in the cloud container: the prebuilt lab pack (`seihai-lab-servers.tar.gz`, GServer-v2 efb5b90, `./run-lab.sh stock`), Seihai as 2.22 with the named account `seihaiwarp`, and the 6.x lab's gen-5 warp proxy changed to send the 178 to the gserver itself (127.0.0.1:14802) and to hold its server side open for a set time after Seihai leaves, which stands in for a server that is slow to drop the old player.

| Old player held | Before the fix | With the fix |
|---|---|---|
| 0 s | in at once | in at once (no retry) |
| 5 s | `disconnected: Account is already in use.` | refused at 0, 1 and 3 s; in on retry 3 (about 7 s) |
| 40 s (longer than the whole schedule) | not run | refused five times; in on retry 5 (about 31 s), by the 30 s takeover |

## 5.00r7 against stock gs2lib, real client beside it (2026-10-31)

A second check of the 5.00r7 row only, run on top of #291. Server: the prebuilt lab pack (GServer-v2 `efb5b90`) started with `./run-lab.sh stock`, so the **unpatched** gs2lib `b839106` handled every login; the only config change was `G3D22067` in `allowedversions.txt`. A gen-5 logging proxy (the 6.x lab's `warpproxy.py`) sat in front and injected `0,Lab Server,127.0.0.1,<proxy port>` 5 s after the first login. Everything ran in a loopback-only network namespace in the cloud container. The real 5.00r7 (the `Graal5_rootdir_files` install, its `license.graal` pointed at the proxy) ran under Wine 9.0 on Xvfb with Mesa GL.

- **Token and name.** In the stock library's version table `G3D22067` is entry 49, named `5.07`, between `G3D20126` (`4.208`) and `G3D14097` (`5.12`), read from the library's two parallel arrays. The server logged both clients as `Version: G3D22067 (5.07)`. "5.07" is gs2lib's name for 5.00 rev 7; there is no separate 5.07 build.
- **Login line.** Real: `%` key `G3D22067` `%guest` `(7Y3bdzjf` `win,<md5>,"",<md5>` `\n`. Seihai: the same shape with its account and `win,"","",""` `\n`. Both were accepted. Then id 37 `English` in its own frame from both.
- **Framing.** Every C→S frame of the real client was type 2 up to 38 bytes and type 4 (zlib) from 57 bytes, which fits Seihai's `< 0x30` raw / `< 0x1000` zlib rule. The proxy decoded every Seihai frame with the same gen-5 iterator.
- **Login mode and 178.** Both drew only the level and chests before the warp, followed the 178 to the named host and port at once, logged in again with a new key, and then drew the player and HUD; the window title became "Lab Server" in both. The real client was a guest; Seihai used a named account and was not refused.
- **File layout.** gs2lib puts `G3D22067` far above `CLVER_2_14`, so it sends the 2.1+ layout, and both clients took it. Small file: `102` = GINT5 modtime + name + data (`&w6t\x91*wbomb1.png\x89PNG…`). File over 64 KB (541,248 bytes): `68` name, `84` size, the data as `102` chunks each with the modtime and name, then `69`. The real client saved `state.png` and Seihai saved `wbomb1.png` byte for byte.
- **Base files.** The real client asked for `basepackage.gupd` with **47** (with a CRC) on the Login connection and with **23** after the warp. Its install holds `updatepackages/basepackage.gupd` but no `updatepackages/Lab%032Server/` folder, so 47 is used when that server's copy is cached. Seihai still does not ask for base files (see the 5.00r7 unknowns).

No profile field had to change. The only edit is the stale login unknown, which now says GServer-v2 accepts the three empty machine fields.

A login to the public `2gta.home.eevul.net:14913` (which lists `5.07` as allowed) could not be tried from the cloud: the container has no DNS or raw TCP route out, only an HTTP proxy.

## Not done

- Ids 109 and 198 (5.24 on) are in the server-id sets but not decoded. 111 (6.03r7) carries no fields: the client saves the whole body as `creationtime.dat` and sends nothing back (6.03 decompile), so Seihai drops it.
- The 6.x rows read the 6.x form of a frame of 0xFFFF bytes or more: `FF FF`, then the length as a `u32be` (`protocol.gen5_long_frames`, `FrameLen::Long`). It was read in the Linux 6.01r1 and 6.03r7 builds; 6.00r2 takes it from 6.01r1. No server has been seen sending it, and GServer-v2 never does.
- The machine-field values (accepted empty by GServer-v2, untested elsewhere) and the base-file requests, above.
- 5.12's reconnect path was not read.
- No public check: this session's network could not resolve `listserver.graal.in`, so which port the public service takes 5.x/6.x logins on was not checked. Use `--online host:port` or the launcher's direct box.
- Public allowed lists: Graal the Adventure allows only 5.00r7 among these rows (`Allowed: 2.22, 5.07`). A gs2 server's range `1.41r1 - iPhone Belote 1.5` should take every row but 5.24r4 (inferred from gs2lib's table). See Graal_analysis `server-notes/public-allowed-versions.md`.
- The chat bar on these builds is the server's GUI script; Seihai keeps its own.

## 6.03r7 Linux (2026-10-31)

The Linux x64 6.03r7 is the build the Windows 6.03r7 row's send ids and long frame length were read from (Graal_analysis `protocol/v6x-clients.md`), so its row is 6.03r7's with two changes: the token `G3D0511C` (gs2lib: `6.037 (Linux)`) and the login list, `linux` followed by four machine ids (MD5 of the `eth0` MAC, MD5 of the `/etc` inode, an empty field, the kernel release and version). Seihai sends all four as `""`; the kernel string was never captured, so it is not made up. The real Linux client was not run.

Lab check, cloud container, loopback only: the prebuilt lab pack started with `./run-lab.sh stock` (stock gs2lib, which already knows the token), `G3D0511C` added to `allowedversions.txt`, the 6.x lab's `warpproxy.py` in front injecting 178 after 5 s. The server logged `Version: G3D0511C (6.037 (Linux))` and `Identity: linux,"","","",""` for both logins; Seihai drew only the level before the warp and the player, HUD and chests after it (window as for the Windows 6.03r7 run). C→S after each login: 37, 46 (and 2, 23 on the first).

GServer-v2's default `allowedversions.txt` lists the other 5.x/6.x tokens gs2lib knows: `G3D3007A` (6.0.0.7 and 6.0.1.5, gs2lib `6.015`) and `G3D2505C` (6.0.3.4). No binary of either is in the project; both now have inferred rows (6.015 and 6.034, see the table above), checked in the lab with stock gs2lib on 2026-10-31.
