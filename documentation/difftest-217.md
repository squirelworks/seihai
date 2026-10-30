# 2.17 real-client differential (2026-10-30)

Laundry item 7 (2026-10-30). The first difftest run (`difftest.md`) skipped 2.17 because the real client's
launcher needs its own UI steps. This run has them. It covers `login` and `gmap_cross`: real 2.17.1
(`Graal_2171.exe`) under Wine against Seihai `--version 2.17` built from `main` (`6a5d755`). No code was
changed.

## Lab

The private lab was rebuilt in a cloud container from the project's lab packs. The steps are in the
project files under `seihai-lab/rebuild-2026-10-30.md`. Accounts and the password are placeholders, the
network namespace is `graallabd`, and only loopback is used. The masked traces and the reports are in
`seihai-lab/difftest-217-2026-10-30/`. The pcaps stay in the lab and are not committed.

* **Servers.**
  * GServer-v2 efb5b90, the build the 2.22/2.31 runs used. All the comparisons below come from this server.
  * GServer-v2 beta4 plus the lab patches, as a second server.
  * Each was run with its own lab listserver (`listserver.py`).
* **Levels.** The original `lv_a.nw` and gm2x2 levels are not in project files. These runs use
  plain-grass levels with the same names and the same 2x2 `gm2x2.gmap`.
* **Clients.**
  * Real: the `graal_reborn` folder of the Graal Reborn 2.2.2.0 zip, copied fresh for every run.
  * Seihai: an asset folder that holds only `pics1.png`, `sprites.png` and `bodies/`.
* **2.17 launcher steps (1024x768).**
  1. Fill nickname (511,313), account (500,367) and password (511,421), and click Start (510,575).
  2. On the server list, click the first row (300,331), then Connect (510,575).
  3. The launcher remembers the last account and adds whatever is typed to it. So delete
     `HKCU\Software\Graal` and clear each field before typing.
  4. Run the registry delete inside the namespace, or kill the wineserver afterwards. Wine opens sockets
     in the wineserver, so one left outside the namespace makes the client fail to connect.

## Results (efb5b90, parity run unless it says otherwise)

| scenario | high / medium / low (default) | high / medium / low (parity) |
|---|---|---|
| login | 3 / 9 / 8 | 2 / 9 / 8 |
| gmap_cross (real with the gmap cached, see 1) | 3 / 13 / 12 | 2 / 14 / 11 |

What matched:
* The login line and the gen-3 cipher.
* `TAMPERCHECK` bytes with `--send-tampercheck`.
* `ADJACENTLEVEL` order (`gm_sw`, `gm_ne`, `gm_se`).
* The crossing prop order (`X, GMAPLEVELX` / `Y, GMAPLEVELY`).
* The west, south and north crossing values.
* The server's replies to all of the above.

## New divergences (not in `difftest.md` or `difftest-leftovers.md`)

1. **Real 2.17 cannot cross a gmap seam unless the `.gmap` is already cached.**
   * From a clean folder, the real client gets `PLAYERWARP2 ... gm2x2.gmap`. It asks for the gmap with
     `WANTFILE`, gets `FILE`, and saves it to `maps/`. It never sends `ADJACENTLEVEL`. At x = 61.5 it
     switches to the `push` GANI and stops at the edge.
   * The same thing happened on both servers.
   * With `maps/gm2x2.gmap` copied in before login, it sends the three 35s in its first frame after the
     burst and crosses in all four directions.
   * Seihai fetches the gmap and then sends the 35s, so it crosses from a clean folder.
   * This is a real-client limit, not something Seihai has to copy. It also means a 2.17 capture needs the
     gmap pre-cached.
2. **Seihai 2.17 sends its post-login frame only after `PLO_LISTPROCESSES` (182), and real 2.17 does not
   wait for it.**
   * beta4 does not send 182 to 2.17. The real client still sends its frame about 110 ms after the burst:
     `UPDATEFILE`s, the `gr.x`/`gr.y` flags, the props and `SHOWIMGPLAYER`, with no `PROCESSLIST`.
   * On beta4, Seihai sent none of that for the whole session: no login props and no `UPDATEFILE`.
   * On efb5b90, which sends 182, both send the frame.
   * The trigger in `login-burst.md` ("only after 182") holds for efb5b90 but is not what the 2.17 client
     keys on.
3. **2.17 sends `SHOWIMGPLAYER` (24) when solo.**
   * It sends `GCHAR 9` in the post-login frame, then about every 5.1 s (t = 0.16, 5.2, 10.3, 16.8 ...
     in `gmap_cross`), with no other player in the level.
   * Seihai has no 2.17 entry for it (`ShowImgSender::for_version(V217)` is `None`), so even
     `--send-showimg` sends nothing. This is the only high divergence left in the 2.17 parity runs.
4. **2.17 walks like 2.22.**
   * The first step sends `GANI=walk, X, SPRITE`. After that it sends `X` (or `Y`) alone, one tile further,
     every 100 ms, and `GANI=idle` when it stops.
   * Seihai 2.17 sends `X, Y, SPRITE` every 50 ms, the "other builds" row of `difftest-leftovers.md` §8.
   * The 2.22 cadence (`protocol/walkcadence.rs`) is what the 2.17 capture shows.
   * The same cadence explains the east crossing value: real `X=0.5, GMAPLEVELX=1` against Seihai `X=0`.
     That is the 2.22 note in `gmap-order.md`, now seen in 2.17 too.
5. **Login frame bundling.**
   * Real 2.17 sends `LOGIN`, `LANGUAGE` and `TAMPERCHECK` in one TCP frame.
   * Seihai sends `LOGIN` alone, then `LANGUAGE` (and `TAMPERCHECK`) in a second frame.
   * The beta4 lab patch 1 exists because of the real client's bundling.

## Known divergences seen again in 2.17

* **Login props.**
  * Seihai leaves out `UDPPORT`, as documented.
  * `NICKNAME` differs only because the harness typed a nickname into the real launcher.
* **`PROCESSLIST` length.** Real 272 bytes, Seihai 8.
* **Asset requests.**
  * Real 2.17 has the files in its folder, so it sends `UPDATEFILE` for:
    * `sword1.png`, `shield1.png`, `body.png`, `wbomb1.png`;
    * then `wbowi1.png` and six `plisticon*.png`;
    * on the first step, `walk.gani`, `steps.wav` and `steps2.wav` in one frame;
    * `push.gani` at the wall.
  * Seihai's minimal folder asks for 15 files with `WANTFILE`. This is the cache difference from
    `difftest-leftovers.md` §7.
  * Real 2.17 never asked for `idle.gani` in these runs, because the shipped folder has it.
* **The `gr.x`/`gr.y` flags.** They come from the server's `-gr_movement` weapon, as in `gr-flags.md`.
  Seihai's flags carry the cell position and its own timing.

## Skipped

* `chat` and `door_warp` for 2.17. They were not asked for.
* Any fix. Items 2 to 4 have a capture behind them now, but item 7 allows no code changes. None of them
  is already fixed by a later PR.
