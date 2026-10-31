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
| 6.03r7 | `G3D0311C` | 92 | Linux 6.03r7's (60) | Tempus Sans ITC 24 |
| 6.036 | **missing** | – | – | no binary and no token: **no row**. It is only named by 5.00r7's updater packages (`Graal6.exe` 6.036, probably 6.03r6). |

Every row is client type 5, gen 5, no tamper check, no showimg timer, and follows 3.153 in the version chain (`documentation/version-chain.md` lists each step's changes and unknowns).

- **Client ids.** 5.00r7's send wrappers give only the constant ids, a floor. The row is that floor, plus 11 and 38 (seen in the lab), plus the ids both 3.153 and 6.03r7 send (0, 3, 20, 27, 30, 46 and the rest of the 2.x set). 40 and 43 are in neither 5.00r7 nor 6.03r7 and are left out. The 5.12 to 5.31r4 send code was not read, so they keep 5.00r7's set.
- **Props.** No 5.x or 6.x prop parser was read. The rows keep 3.153's own and other prop sets (X2/Y2/Z2 included, which the lab saw these clients send).
- **Chat.** 5.x draws a bold sans serif (5.24's config says `font=Arial`): a new `ChatFace::Arial`. 6.00r2 and 6.03r7 use Tempus Sans ITC 24.

## The Login lobby (`protocol::lobby`)

A row with `protocol.lobby` set is a 5.x/6.x build:

- **No listserver.** Seihai logs in to the address it is given as a game server and names it `Login`, as the clients do. The listserver step is skipped for these profiles even when the address looks like a listserver. The client's own built-in address, `loginserver.graalonline.com:14900`, is kept as history (`ServerList::Direct`).
- **Login line.** After the password comes an unprefixed comma list: `win`, three machine fields, and from 5.12 on the quoted Windows version `"6.2 9200 "` (what every capture shows). Seihai does not read the machine: it sends each of the three fields as `""`, which is the value the real 5.00r7 sent for its MAC field under Wine. GServer-v2 took all three empty in the lab check below.
- **Then** id 37 (language) in its own gen-5 frame, as captured. The base-file requests that follow in the captures (`basepackage.gupd` by 23 or 47) are not sent.
- **Login mode.** While the current server name starts with "login" (any case), Seihai draws the level, chests and NPC images, but not its own player, other players, chat over heads or the HUD. That is what the real clients do (5.00r7, 5.12, 5.24r4, 5.31r4, 6.00r2 and 6.03r7 in the lab). Seihai still draws gani NPCs and baddies there; the lab saw no gani NPCs drawn and had no baddies on the Login level.
- **Id 178.** A `PLO_SERVERWARP` is followed as for every build (`--no-serverwarp` turns it off, and then a lobby build stays in Login mode, with a warning). The lobby builds split the text **on commas only**: the lab's unquoted `0,Lab Server,127.0.0.1,14900` reached 127.0.0.1:14900 and the window title read "Lab Server" in every build, where the 2.x comma-text reader would split the name at its space. The test `lab_warp_bytes_for_the_lobby_builds` feeds the exact lab bytes (`0xD2` + that text) to every lobby profile. After the warp the new name is the server's name, Login mode ends, and the player, the HUD and other players are drawn on the flat path.
- **3.x stays held.** The 5.x/6.x tokens start with `G3D`, as the 3.x ones do, but these builds run in `seihai-mae`, not `seihai-ato`, and nothing opens Torque.

Seihai is a client, so it receives the 178; it cannot send one. On the lab's GServer-v2 the warp only comes from the proxy (`g5mitm.py` with `rules.py`), because GServer-v2 sends 178 only when its listserver answers a server-info query.

## 5.12's cipher step count

5.12 steps the gen-5 iterator `len / 4` times, rounded down, on zlib and bz2 frames (still at most 4), leaves a partial last group alone, and sends every frame compressed (`Gen5::round_down`, `protocol.gen5_round_down`). Every other build rounds up. This is faithful to the client, so a 5.12 profile talks to a server that steps as 5.12 does. GServer-v2 does not, and the lab puts its translating proxy between them (it keys on the `G3D14097` token).

## Lab check (item 6)

Run on 2026-10-31 against the commit merged in #281, all on loopback: a copy of the lab's GServer-v2 build (with `G3D22067,G3D14097` allowed), a listserver copy (GServer-v2 refuses every login without one), and the lab proxy in front of it, which also injected the 178.

- **Login.** 5.00r7 and 5.12 were both accepted; the server logged them as `G3D22067 (5.07)` and `G3D14097 (5.12)`. Then Seihai sent `37 English`, `46`, two `2` packets and file requests by 23 and 34.
- **182 never came.** GServer-v2 did not send S>C 182 in any run, so the proxy injected `0,Lab Server,127.0.0.1,<proxy port>` after 30 s.
- **Warp.** Seihai followed it every time and logged in again. Before the warp: level, chests and minimap only. After it: the player with shield, hearts, the A/S/D counters and the HUD, and the server's welcome PM. Both builds looked the same.
- **5.12 cipher.** The proxy decoded every 5.12 frame with round-down stepping and saw no illegal data, so Seihai's 5.12 stepping matches.
- **S>C ids seen** (same on both builds): 0, 4, 6, 8, 9, 10, 14, 25, 34, 37, 39, 41, 42, 43, 44, 45, 47, 82, 100, 101, 156, 168, 174, 179, 180, 190, 194. Of these 82, 190 and 194 (all empty here) have no case in `dispatch.rs`; 100 is the raw-data prefix. `seihai-mae` counts unhandled ids but does not print them.
- **The second login is turned away if it comes too fast.** Seihai closes the old socket and connects again within about 10 ms. GServer-v2 drops the old player only 1 to 5 s later, and until then answers the new login with a `double login` disconnect or S>C 16 `Account is already in use.` The runs passed with the proxy holding the new login 4 s (5.00r7) and 8 s (5.12) before it connected. The real clients do the same thing. In the 5.00r7 binary (`GraalEngine.dll`), the 178 handler stores the text, the next frame tick calls `connectToServer`, and that function deletes the old connection and makes the new one in the same call, with no wait. The network thread polls every 1 ms. S>C 16 only stores the text for the reconnect dialog, and nothing retries the login after a warp. The only automatic reconnect is to the backup login hosts, when the server is named exactly `Login` and the connect itself fails. The real clients got through in the earlier labs because they log in as guest, and GServer-v2 skips its in-use check for guests (`TPlayerLogin.cpp`). A real client with a named account would get the same refusal. 5.12's code was not read; it comes from the same tree three months later. So Seihai keeps reconnecting at once, as the clients do.

## Not done

- Ids 109 and 198 (5.24 on) and 111 (6.03r7) are in the server-id sets but not decoded.
- The 6.x form of a frame of 0xFFFF bytes or more (length `0xFFFF`, then a 4-byte length) is not read.
- The machine-field values (accepted empty by GServer-v2, untested elsewhere) and the base-file requests, above.
- The warp reconnect does not wait or retry, as in 5.00r7, so on GServer-v2 a named account needs the old player to be gone first (lab check, above). 5.12's reconnect path was not read.
- No public check: this session's network could not resolve `listserver.graal.in`, so which port the public service takes 5.x/6.x logins on was not checked. Use `--online host:port` or the launcher's direct box.
- The chat bar on these builds is the server's GUI script; Seihai keeps its own.
