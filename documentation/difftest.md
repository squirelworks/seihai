# Differential testing

Roadmap step 5: run the same scripted session against the real Wine client and
against Seihai on a private loopback lab, decode both captures with Seihai's
own codecs, and diff the frame sequences. This page is the method, the first
results, and the ranked list of divergences found. None of them is fixed in
this change; they are listed as follow-ups.

Branch note: the script engine (#23) is **not** merged, so this work is based on
`main` (`de01935`) and does not depend on it.

## Parts

| part | where | what |
|---|---|---|
| decoder | `seihai::difftest::{pcap,decode,trace}`, `difftest trace` | pcap (Ethernet, SLL, raw, null; IPv4 TCP) -> TCP reassembly -> frames -> per-direction message list with id, name, decoded fields, time and frame number. Gen5 (XOR + zlib/bz2), gen3 (junk-byte client cipher) and plain zlib are handled by Seihai's own codecs. Output is a `.tsv` trace. |
| masking | `decode.rs` | Keys, accounts, the password (always `<PASS>`, never read out), player ids, mod times, UDP port, IPs, online seconds, the world time and accounts inside text are masked. Large payloads become length + crc32. `--no-mask` exists for local debugging only. |
| differ | `seihai::difftest::diff`, `difftest diff` | Aligns the two traces per connection and direction (LCS over run-collapsed ids) and reports missing and extra ids, ordering, frame bundling, field mismatches, count and timing differences, with a severity. Writes `PREFIX.txt` (readable) and `PREFIX.tsv`. `--fail-on SEV` gives a non-zero exit for CI. |
| runner | `tools/difftest/lab.sh`, `scenarios/*.steps` | Private namespace `graallabd`, its own Xvfb displays (`:31` real, `:32` Seihai; `:3` is refused), fresh copies of the server, listserver and client folder per run, the lab hosts file bind-mounted inside a private mount namespace (no `/etc/hosts` edit). Details and safety rules: `tools/difftest/README.md`. |

To read a capture or a trace by eye (names, decoded fields, raw hex, filters, a packet's layout across versions) use `seihai capture`; see `capture-viewer.md`.

Tolerances (all in `DiffOptions`, all overridable): walk updates (C>S 2) and the
variable-count server messages (S>C 8, 9) are compared by first message and by
cadence, not by count; positions within 0.6 tile; timer-like ids (C>S 24, 39)
by interval; the file names in WANTFILE and UPDATEFILE and the prop sets of
walk updates are compared as sets over the stream; the focus click's
TRIGGERACTION (C>S 38) is harness noise and reported as info only. Timing uses a
20 % relative allowance with a 5 ms floor, measured inside bursts (gaps over
1 s are ignored) so idle time does not count. Server-to-client findings rank one
step below client-to-server ones, because they are mostly consequences.

Two Seihai variants are captured for every scenario: `seihai` (defaults) and
`seihai-parity` (`--send-tampercheck --send-showimg`, the two opt-in senders).
Comparing the two separates "not enabled by default" from "does not exist".

## What was run

Lab: GServer-v2 and graal-serverlist copies, accounts `dtacct` with the
placeholder password, 2x2 gmap (`gm2x2.gmap`, `gm_nw/ne/sw/se.nw`), and the
level links of `lv_a.nw`. No public server was involved.

| version | scenario | real | Seihai | parity | result (high / medium / low, parity run) |
|---|---|---|---|---|---|
| 2.31 | login | yes | yes | yes | 3 / 10 / 21 |
| 2.31 | gmap_cross | yes | yes | yes | 2 / 13 / 23 |
| 2.31 | chat | yes | yes | yes | 2 / 11 / 21 |
| 2.31 | door_warp | yes | yes | yes | 3 / 14 / 23 |
| 2.22 | login | yes | yes | yes | 4 / 7 / 2 |
| 2.22 | gmap_cross | yes | yes | yes | 3 / 10 / 8 |
| 2.22 | chat | yes | yes | yes | 3 / 9 / 2 |
| 2.22 | door_warp | yes | yes | yes | 4 / 13 / 24 |
| 2.17 | login, gmap_cross | yes (2026-10-30) | yes | yes | see `difftest-217.md` |
| any | serverwarp | not run | not run | not run | needs an injection forwarder |

The low counts for 2.31 are mostly one root cause counted many times (the extra
warp, below). The per-run `.txt` and `.tsv` reports and the traces live in the
lab's `out/` directory outside the repository; pcaps stay in a mode-700 folder
and are never committed.

Chat matched: both clients send the same two `CURCHAT` props with the same
values and the same bundling, in both versions. Apart from the login-time
differences below, chat has no divergence of its own.

2.31 gmap crossing matched on the crossing frame itself: `ENTERLEVEL` followed
by a `GMAPLEVELX/Y` + `X2/Y2` props message, in one frame, with the new
coordinate `0` or `63.5`, in all four directions.

## "unstick me" runs (2.22 real and Seihai; 2.31 real)

Scenarios `unstick`, `unstick_jail` and `unstick_npc` are described in
`unstick-me.md`: the real client sends `unstick me` as plain `CURCHAT`;
Seihai's bytes are identical; a client-side script's rewrite reaches the wire
in the real client and (after that PR) in Seihai.

## Ranked divergences

Status of each one after the item-15 pull requests: `documentation/difftest-leftovers.md`.

Evidence is the parity run unless it says otherwise. "Both" means 2.22 and 2.31.

### High (the real client sends something Seihai never sends, or sends it wrongly)

1. **Level rectangle links never fire** (both, door_warp). `lv_a.nw` has
   `LINK lv_c.nw 10 28 2 5 60 30`. The real client walks left along y=30.5,
   enters the rectangle and sends `LEVELWARP` (id 0) for `lv_c.nw` at 60,30.
   Seihai walks through the same rectangle (positions 13.5, 9.5, 5.5, 2.5 at
   y=30.5) and sends nothing. Edge links (`lv_w.nw`) work. Later the real client
   also sends `LEVELWARPMOD` (id 30) on the way back; that is a consequence.
   Cause not diagnosed. The warp was reproduced in two versions.
2. **`PLI_PLAYERPROPS` at login is missing** (both). The real client sends one
   props message after the account burst: `NICKNAME, HEADGIF, COLORS, STATUS,
   UDPPORT, ADDITFLAGS, PLANGUAGE, OSTYPE, TEXTCODEPAGE`. Seihai sends no props
   at login.
3. **`PLI_UPDATEFILE` (34) is missing** (both). The real client sends three
   (`sword1.png`, `shield1.png`, `body.png`), and a fourth for `gm2x2.gmap` in
   the gmap scenarios, with mod times. Seihai sends none, so the server never
   answers `FILEUPTODATE` (45).
4. **`PLI_PROCESSLIST` (44) is missing** (both). One message at login. The
   content (the client's process list) differs on every run, so only the
   length is compared.
5. **2.22: `PLI_FLAGSET` (18) with `gr.x` / `gr.y` is missing.** The real 2.22
   client sends `gr.x=30` and `gr.y=30.5` at login and then one `gr.x=` or
   `gr.y=` flag per walk step (224 to 448 messages per run). Seihai sends none.
   2.31 does not.
6. **2.31: `ENTERLEVEL` (46) goes out before the login burst, and twice.** The
   real client sends one `46` after the server's login burst, in the frame that
   carries the props. Seihai sends it in its own frame at t=0 and again after
   the burst. In the lab the server answers the early one by warping the player
   to `onlinestartlocal.nw` first (extra `PLAYERWARP`, `LEVELNAME`,
   `BOARDPACKET`, three `LEVELCHEST`, `NEWWORLDTIME`, about 22 ms in), then
   warps again to the account's level. Seen in every 2.31 run; 2.22 does not
   send 46 and has no extra warp. That the early 46 causes the extra warp is the
   most likely reading of the timing, not something I proved by changing it.

### Medium (different content or order)

7. **WANTFILE (23) file set** (both). The real client asks for its assets:
   `head0.png`, `-.gif`, the nine `plisticon*.png` icons, `wbomb1.png`,
   `wbowi1.png`, and on walking `walk.gani`, `steps.wav`, `steps2.wav`
   (2.22 also `idle.gani`). Seihai asks for none of them. Seihai asks for the
   level files it is entering (`lv_a.nw`, `lv_d.nw`, `lv_w.nw`,
   `onlinestartlocal.nw`, `gm_nw.nw`, `gm2x2.gmap`), which the real client
   never requests in these runs. The server answers with extra `FILE` (102)
   messages and with `FILESENDFAILED` (30) for the assets.
8. **Walk update props** (both). 2.31 real: `X2` (and `Y2`), with `GANI=walk` on
   the first step and `SPRITE` only when it changes. 2.22 real: `X` or `Y` only,
   plus `GANI`/`SPRITE` when they change. Seihai sends `X, Y, SPRITE` on every step.
9. **Walk speed and cadence** (both). The real client moves 0.5 tile every 50 ms
   (2.31, one props message per step) or 1 tile per 100 ms (2.22, props every
   second step). Seihai sends 0.5 tile every 64 ms, about 7.8 tiles per second
   against 10. This is what moves the two clients to different places in a
   timed scenario.
10. **Frame bundling at login** (both). The real client puts `UPDATEFILE`,
    `FLAGSET`, `PROCESSLIST` and the props message in one frame; Seihai sends
    its messages as they happen. On gmap entry in 2.22 `ADJACENTLEVEL` and
    props bundling also differ.
11. **`ADJACENTLEVEL` (35) order** (both, gmap). Real: `gm_sw`, `gm_ne`, `gm_se`.
    Seihai: `gm_ne`, `gm_sw`, `gm_se`. The server's `LEVELNAME` replies follow
    that order.
12. **2.22 gmap crossing props** (2.22 only). Real sends `X, GMAPLEVELX` with
    `X=0.5` after crossing east; Seihai sends `GMAPLEVELX, X` with `X=0`.
    Vertical crossings have equal values but the same prop order difference
    (`Y, GMAPLEVELY` against `GMAPLEVELY, Y`). 2.31 matches exactly.
13. **Opt-in senders** (default `seihai` runs only). The real clients send
    `TAMPERCHECK` (39: once on 2.22 login and about nine times in a 2.31 login
    scenario,) and `SHOWIMGPLAYER` (24, 2.31, periodic).
    Seihai sends them only with `--send-tampercheck --send-showimg`; with them
    the intervals match within tolerance. Decide whether they should become
    defaults.

### Low and consequences

14. Server-to-client differences that follow from the above: missing
    `FILEUPTODATE` and `FILESENDFAILED`, extra `FILE` and `LEVELCHEST`, extra
    `PLAYERWARP`/`LEVELNAME`/`BOARDPACKET`/... (item 6), one more `NEWWORLDTIME`,
    ordering of the login burst around the extra warp. They should disappear as
    items 3, 6 and 7 are fixed; re-run to confirm.
15. 2.31 first `GMAPLEVEL` frame and the rest of the crossing are identical, and
    no divergence was found in `CURCHAT`, `ENTERLEVEL` on crossing, or the
    `X2/Y2` values after a crossing (checked, listed so nobody searches again).

## Harness caveats

Things that are the lab and not the client, found and fixed or noted:

* `door_warp` is position-dependent. Because Seihai walks slower (item 9), the
  two clients are not at the same place at the same time, so the edge-link
  `LEVELWARP` carries `playerx` = 17 / 17.5 (real) against 18 (Seihai) and, on
  2.22, the real client left through `lv_c.nw` while Seihai reached `lv_w.nw`.
  Those field and target differences are consequences of items 1 and 9, not
  separate bugs. A fixed-position scenario (start at a chosen tile) would remove
  this.
* Seihai polls keys once per frame; a plain `xdotool key Return` (12 ms press)
  was lost half the time and dropped one of the two chat lines. The runner now
  holds Return for 150 ms. After that fix both chat lines arrive in both versions.
* Seihai must not start before the game server has registered with the
  listserver; otherwise it gets no server list. The runner waits for the
  listserver's "New Server" line and then a settle time.
* The real client's focus click sends one `TRIGGERACTION` (38); ignored as noise.
* `PLI_PROCESSLIST` content changes every run; compared by length.

## Not covered yet

* **2.17 real client.** Its launcher is a different window (Training / Start /
  Options). The 2026-10-30 run has its UI steps and results: `difftest-217.md`.
  `lab.sh` still has only the 2.22/2.31 steps.
* **serverwarp**: the scenario file exists; it needs a proxy that injects a
  server-side warp into both sessions.
* **Longer sessions**, weapons, NPC scripts, and PM/guild chat.
* Only one account and one gserver build were used.

## Follow-ups

In rank order, one change each:

1. Diagnose and fix rectangle links (item 1) with a regression in the offline
   engine tests.
2. Send the login props message (2), `UPDATEFILE` (3), `PROCESSLIST` (4) and
   bundle them as the real client does (10).
3. 2.22: send the `gr.x` / `gr.y` flags (5).
4. 2.31: send `ENTERLEVEL` once, after the burst (6), and compare the server
   side again.
5. Request the real client's asset list and stop asking for level files (7).
6. Match the walk update props and cadence (8, 9); this also makes `door_warp`
   deterministic.
7. `ADJACENTLEVEL` order (11) and the 2.22 crossing props (12).
8. Decide on the opt-in senders (13).
9. 2.17 launcher steps, a fixed-start scenario, and `serverwarp`.

## Running it

```sh
export DIFFTEST_ROOT=/path/to/private/lab      # see tools/difftest/README.md
cargo build --release
tools/difftest/lab.sh all "2.22 2.31" "login gmap_cross chat door_warp"
target/release/difftest trace some.pcap --version 2.31 --out real.tsv
target/release/difftest diff real.tsv seihai.tsv --out report
```

`cargo test -p seihai --lib difftest` runs the unit tests for the pcap reader,
the decoder, the trace format and the differ.
