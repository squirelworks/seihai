# Differential testing: what is left (item 15)

Status of every divergence in `documentation/difftest.md` after the item-15 pull requests. The
evidence is the private lab (real Wine client against Seihai on a loopback GServer-v2, placeholder
accounts, run 2026-10-25). Numbers are from the ranked report (`difftest diff`), default Seihai options.

| # | divergence | verdict | where |
|---|---|---|---|
| 1 | rectangle links never fire | fixed | `link-geometry.md` |
| 2-4, 10 (login part) | login props, `UPDATEFILE`, `PROCESSLIST`, login frame bundling | fixed | `login-burst.md` |
| 5 | 2.22 `gr.x`/`gr.y` flags | fixed | `gr-flags.md` |
| 6 | 2.31 early and duplicate `ENTERLEVEL` | fixed (cause found: not the 46) | `early-enter-level.md` |
| 9 | walk speed | fixed | `walk-pace.md` |
| 11, 12 | `ADJACENTLEVEL` order, 2.22 crossing props | fixed | `gmap-order.md` |
| 7 | asset `WANTFILE`s | confirmed, skipped (below) | here |
| 8 | walk update fields and 2.22 cadence | confirmed, skipped (below) | here |
| 10 (rest) | frame bundling after login | confirmed, skipped (below) | here |
| 13 | opt-in senders | unchanged (decision for the maintainer) | `difftest.md` |

## 7. Asset requests

Real client, `gmap_cross` 2.31: `wbomb1.png`, `wbowi1.png`, `-.gif`, the nine `plisticon*.png`, and on the
first step `steps.wav`, `walk.gani`, `steps2.wav` (2.22 also `idle.gani`). Seihai now asks for the images
of the login frame (part 1), the icons, `head0.png`, `wbomb1.png` and `-.gif`. What is left:

* `walk.gani`, `steps.wav`, `steps2.wav` (and `idle.gani` at login on 2.17/2.22): the real client's clean
  folder has none of them, and it asks at the first step (2.31 `gmap_cross`: `steps.wav` and `walk.gani` in
  one frame, `steps2.wav` 100 ms later, i.e. at the second step). Seihai's lab asset folder has none of them
  either, but Seihai has no request path for a missing GANI or sound (the sound `WANTFILE` is an open item of
  the audio work), so it asks for none. The names and the timing are known from the capture; where the
  request belongs in Seihai's loop (and what a sound name is before the GANI that names it has arrived) is
  not decided, so this is skipped, not invented.
* `lv_*.nw`, `gm_nw.nw`, `gm2x2.gmap`: Seihai asks for the level files it enters, because it starts
  with none; the real client had them cached (it sent `UPDATEFILE gm2x2.gmap` instead). A cache-state
  difference of the lab, not a wire rule.
* `wbowi1.png`: the server sends `PLO_DEFAULTWEAPON` ids 8 then 7 and the real client asks for
  `wbomb1.png` then `wbowi1.png`. Seihai asks for the images of the weapons the server lists, so it
  misses the second. Which id is the bomb and which the bow is not decided by the capture (the pairing
  depends on whether the client walks its list in arrival or in id order). Skipped; no payload invented.

## 8. Walk update fields and the 2.22 cadence

Decoded from the real traces (`gmap_cross`, `door_warp`):

| build | first step | following steps | stop | cadence |
|---|---|---|---|---|
| 2.22 | `GANI=walk, X (or Y), SPRITE` | `X` (or `Y`) alone, one tile further each message | `GANI=idle` | one message per 100 ms (every second 50 ms step); the `gr` flags go per step |
| 2.31 | `GANI=walk, SPRITE, X2` (or `Y2`) | `X2` (or `Y2`) alone, half a tile | `GANI=idle` | one message per 50 ms |
| Seihai | `X, Y, SPRITE` | `X, Y, SPRITE` | nothing | one message per step (50 ms after `walk-pace.md`) |

The props are in ascending id order in every message. Not decided by the captures: what a stop on an odd
half-step sends for 2.22 (the idle message carries only `GANI`), how a turn or a diagonal step is split, and
when the 2.22 client flushes its pending position. Writing the rule from the straight-line cases would
invent those; skipped. Seihai's `X, Y, SPRITE` update is accepted by GServer-v2 and carries the same position.

## 10. Frame bundling after login

Done: the login frame (files, 46, flags, process list, props) has the real frame's order for 2.22 and 2.31
(minus the `wbowi1.png` request and `UDPPORT` above). Left: on a gmap login the real client puts `UPDATEFILE gm2x2.gmap` and the three
`ADJACENTLEVEL`s inside that frame, because it already holds the gmap file and so knows the neighbours. Seihai
has no gmap file yet (it asks for it in the same moment), so it cannot name the neighbours until the server
sends it; they go in a later frame. With a cached gmap file the same code path could bundle them; not done, and
not measurable in this lab without a Seihai asset folder that holds `gm2x2.gmap`.

## 13. Opt-in senders

`TAMPERCHECK` and `SHOWIMGPLAYER` are still opt-in (`--send-tampercheck`, `--send-showimg`). They are the only
`high` rows in the ranked report. With them on, the first id 24 now goes at the end of the post-login frame like
the real 2.31 client's. Whether they should become defaults is not decided here.

## Maintainer answers (2026-10-25)

- **Tamper check and id 24 as defaults:** undecided. Leave them opt-in (`--send-tampercheck`, `--send-showimg`) for now.
- **Request `walk.gani` and step sounds:** only if Seihai does not already have them. Do not re-request files that are present. *(Implemented in item 20: `FileSession::on_step`, `documentation/file-cache.md`. A built-in copy counts as present, so a normal run asks for none of the three.)*
- **Bomb and bow weapon ids:** not settled by the capture. Leave open; do not invent a mapping.
