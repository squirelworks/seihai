# 4.x profiles

Date: 2026-10-31. Three builds: Windows 4.04r2, Mac OS X 4.20r6 and Windows 4.20r8. They sit between 3.153 and 5.00r7 in the version chain (`documentation/version-chain.md`).

Sources, all in Graal_analysis: `protocol/v4x-clients.md` (static reads of all three builds), `client-internals/live-behaviour-404.md` (the real 4.04r2 under Wine on GServer-v2) and the new `version-profiles/4.04-4.20.toml` (the same facts in table form, read by `tools/gen_profile_fixture.py`). No wire byte here is new: every one is in those notes.

## The rows

| Label | Token | Server ids | Client ids | What is special |
|---|---|---:|---|---|
| 4.04r2 | `G3D17026` | 98 | the 4.20r8 floor without 162, plus 3.153's | the first Login-lobby build; live-checked with the real client |
| 4.20r6 | `G3D27076` | 99 | as 4.20r8 | Mac (PowerPC, never run); login platform `mac`; no case for 41 |
| 4.20r8 | `G3D20126` | 101 | send code read (a floor), plus 3.153's | read from a third-party packet-logger copy of the engine DLL |

Every row is client type 5, gen 5, no tamper check, no showimg timer, the Login lobby, the built-in `loginserver.graalonline.com:14900`, and 3.153's prop sets.

- **Server ids.** 4.04r2 is 3.153's set plus the update packages (105, 106), ghost mode (170), the window list (182, 183), `onServerListerConnect` (190), `SHOOT2` (191), the cache check (193), the script-list clear (194) and server ganis (195). 4.20r8 adds the layered board change (107), 108 and 196. The Mac 4.20r6 has 4.20r8's set without 41 and 196. The note counts that set as 97, but the set it describes has 99 ids; the binary was not re-read, so the row keeps the listed set and says so in its `unknown`.
- **Client ids.** Only 4.20r8's send code was read, and some sends pass the id in a register, so that list is a floor. The rows take it, plus the ids both 3.153 and 5.00r7 send (as the 5.00r7 row did with its neighbours); 40 and 43 are sent by neither 4.x nor 5.00r7 and are left out. 4.04r2 has no 108 handler, so it does not get 162 (the answer to 108). 11, 23, 34, 37, 38, 44 and 157 were seen from the real 4.04r2.
- **5.00r7 re-checked.** It now follows 4.20r8: server ids 140 and 197 in, 100, 108 and 196 out; client ids 47 and 159 in, 162 out. Its lobby, list address and chat face come from the 4.x rows. The resolved 5.00r7 row is unchanged.
- **Chat.** The real 4.04r2 draws white bold sans with a blue shadow. The rows use `ChatFace::Arial` at 24, as 5.00r7 does.

## The Login lobby

The 4.x builds are the first with no listserver protocol: the 4.04r2 engine sets its draw-layer mask to 0 while the server name starts with `login`, and 178 hands it to the game server. So the rows set `protocol.lobby`, and everything in `documentation/v5v6-profiles.md` ("The Login lobby") applies: Seihai logs in to the given address as `Login`, draws only the level, chests and NPC images until a 178, and splits the 178 text on commas only.

The login tail is four fields, as in 5.00r7: `win,"","",""` (`LOBBY_404`), and `mac,"","",""` for the Mac 4.20r6 (`LOBBY_420_MAC`). The real clients send MD5s (Windows: product id, MAC, `C:\` serial; Mac: two hashes and an empty field). Seihai does not read the machine.

## 3D stays held

4.04r2's `Graal3DEngine.dll` (Torque protocol 19) and the Mac 4.20r6's 3D side (protocol 21) are in `seihai-ato::torque` as profiles only. These rows have the lobby, so `seihai-launcher` and `seihai-mae` keep them on the flat path and nothing opens Torque.

## Lab check

Run on 2026-10-31 in the cloud container, all on loopback, Seihai `seihai-mae` release build under Xvfb. Server: the prebuilt lab pack `seihai-lab/servers/seihai-lab-servers.tar.gz` started with `./run-lab.sh stock` (stock gs2lib), with `G3D17026`, `G3D27076` and `G3D20126` added to `allowedversions.txt`. In front of it, the 6.x lab's `warpproxy.py`, which logs both directions and injects S>C 178 `0,Lab Server,127.0.0.1,14810` 5 s after the first login. The files are in the project folder, `v4/lab-4x-2026-10-31/`.

| | 4.04r2 | 4.20r8 | 4.20r6 |
|---|---|---|---|
| Login accepted | yes; server log `G3D17026 (4.042)` | yes; `G3D20126 (4.208)` | yes; stock gs2lib does not know `G3D27076` (see below) |
| Login line | `%` + key + token + account + password + `win,"","",""` (server log `Identity: win,"","",""`) | same | same with `mac,"","",""` |
| Before the warp (3 s) | level and chests; no player, no HUD | same | same |
| 178 | old connection closed, new login with a new key | same | same |
| After the warp (12 s) | window title `Graal 4.04r2 - Lab Server`; player, A/S/D boxes, AP bar, hearts and counters | `Graal 4.20r8 - Lab Server`; same | `Graal 4.20r6 - Lab Server`; same |
| C>S after each login | 37 `English`, 46 | same | same |

Server ids seen on 4.04r2 and 4.20r8: 0, 4, 6, 9, 10, 14, 25, 30, 33, 34, 39, 41, 42, 43, 47, 82, 100, 101, 156, 168, 174, 179, 180, 182, 190, 194. All are in both sets (100 is the raw-data prefix, read by the frame layer). On 4.20r6 GServer-v2 sent 7 and left out 0, 156, 174, 179 and 180: stock gs2lib has no entry for `G3D27076`, so it treats the build as an older client. 41 also came on 4.20r6, which has no case for it, so Seihai drops it as the row says.

Not covered: the real 4.x clients were not run against Seihai's rows here (the 4.04r2 live run is the client against GServer-v2, not Seihai). The server sent none of 105 to 108, 170, 191, 193, 195 or 196, so those handlers are covered by unit tests only. Seihai sends 46 after each login because the rows take it from 3.153 and 5.00r7; the real 4.04r2 was not seen sending it.

## Not done

- The Mac 4.20r6 id count (97 in the note, 99 in the set it lists) and its send ids.
- The 4.04r2 send ids (taken from 4.20r8).
- The base-file requests after the language (`basepackage.gupd`, `bigmap.txt`) are not sent, as for 5.x.
- The stock 4.20r8 engine is in a password-protected archive; only the packet-logger copy was read.
