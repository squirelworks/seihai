# Differential testing: what is left (item 15)

Status of every divergence in `documentation/difftest.md` after the item-15 pull requests. The
evidence is the private lab (real Wine client against Seihai on a loopback GServer-v2, placeholder
accounts, run 2026-10-25). Numbers are from the ranked report (`difftest diff`), default Seihai options.

| # | divergence | verdict | where |
|---|---|---|---|
| 1 | rectangle links never fire | fixed | `link-geometry.md` |
| 2-4, 10 (login part) | login props, `UPDATEFILE`, `PROCESSLIST`, login frame bundling | fixed | `login-burst.md` |
| 5 | 2.22 `gr.x`/`gr.y` flags | fixed (the flags come from the server's `-gr_movement` weapon; Seihai sends script flags) | `gr-flags.md` |
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
  one frame, `steps2.wav` 100 ms later, i.e. at the second step). Done: `idle.gani` goes in the login frame
  where the build asks for it (`LoginInfo::idle_gani`), and the three first-step names go with the first and
  second step when Seihai has no copy at all (`FileSession::on_step`, the maintainer answer below). A
  built-in copy counts, so a default run still asks for none of them; `--no-builtin` runs Seihai without the
  built-in set, as the real client's clean folder, so the lab can compare the two. The path behind it,
  `FileSession::need_missing`, is also used for a sound a script or a GANI cue plays and for the GANI an NPC
  names (`setcharani`, `setani`), gated by the profile (`pli` 23, and `ui.ganis` for a GANI). A sound named by
  a GANI that has not arrived yet is not asked for: its name is only known once the GANI is read, and then it
  goes through the same path when the cue plays. A GANI named by another player's `GANI` prop that Seihai
  does not draw is not asked for (Seihai draws only its pose set for others; not captured).

  Lab check, 2026-10-30, on Windows (stock GServer beta4 with the 2.22 distro's GANIs, sounds, heads and
  bodies copied into its world, the lab `listserver.py`, all on 127.0.0.1). Seihai main `a0e4ce4`,
  `--no-builtin`, against the real 2.22 and 2.31 clients. Every run used a fresh folder without the four
  files, a fresh account and a short walk right. Times are from login.

  | run | `idle.gani` | `steps.wav` + `walk.gani` | `steps2.wav` |
  |---|---|---|---|
  | real 2.31, run 1 | none | step-1 frame | in the step-1 frame too |
  | real 2.31, run 2 | none | step-1 frame | +69 ms, step-2 frame |
  | Seihai 2.31 (2 runs) | none | own frame, step 1 in the next frame (same ms) | +98 / +118 ms, own frame |
  | real 2.22 (2 runs) | frame 2, ~+250 ms, with the login `PLAYERPROPS` | step-1 frame | +68 / +69 ms, own frame |
  | Seihai 2.22 | frame 2, +18 ms, alone | own frame, then step 1 | +60 ms, own frame |

  The same four names, in the same order, at the same steps. The frames, decoded message by message
  from the same captures:

  * Real 2.31 and 2.22 put the first-step `WANTFILE`s **first** in the frame of the step's
    `PLAYERPROPS`, with nothing else in it (2.31: `steps.wav`, `walk.gani`, then the props; when
    `steps2.wav` goes at step 2 it leads that step's props frame). Real 2.22 sent `steps2.wav` alone in a
    tick frame with no props. **Fixed:** `Connection::queue_files` holds them for the next walk update
    that changes something (2.31) or the next tick frame (2.22), ahead of the flags and the props;
    unsent after 150 ms they go alone.
  * Real 2.22 `idle.gani` is the **first** message of the post-login frame: `WANTFILE idle.gani`, the
    icon checks, the login `PLAYERPROPS`, `SHOWIMGPLAYER`. That frame went at +250 ms, about 240 ms
    after the level (stock beta4 sends no `PLO_LISTPROCESSES`; 2.31's frame had the same gap). Seihai
    asked at +18 ms, alone, after the level. **Fixed:** the check is the first message of
    `LoginBurst::take`, and that frame now goes 240 ms after the level when no 182 comes (next item).
    The rerun of #184 sent it alone at +871 ms because the timer started when the game loop took
    the level, not when it was read; the timer now starts at the read (`spawn_reader_timed`).
  * `steps2.wav` usually goes at the second step, but in one 2.31 run it went in the step-1 frame. So the
    second step is the usual case, not a rule.
  * `hat0.png`: a **lab-folder difference, not a wire rule**. Every real client folder had
    `levels/hats/hat0.png`, the file the server's `walk.gani` and `idle.gani` name in `DEFAULTATTR1`, and
    the real clients sent nothing for it (no `UPDATEFILE` either). Seihai's lab folder had only
    `pics1.png`, so it asked for the missing picture with `WANTFILE`, as the loaders do. With the file
    present Seihai sends nothing for it either (`look_image_pass` asks only for a missing picture). No
    code change. A fair rerun puts `hats/hat0.png` in Seihai's folder too.
  * On stock beta4 (no `PLO_LISTPROCESSES`), Seihai 2.22 and 2.31 sent no post-login frame at all,
    while the real clients sent theirs 236 to 246 ms after the level. **Fixed:** the frame is due then
    (`FRAME_AFTER_LEVEL`), in the shape decoded from those captures (`login-burst.md`, "Without
    `PLO_LISTPROCESSES`"). Seihai 2.22 also sent an empty frame at +3022 ms: the late login 46, which
    2.22 does not owe, went out with no messages. **Fixed:** nothing goes when there is nothing to send.
* `lv_*.nw`, `gm_nw.nw`, `gm2x2.gmap`: Seihai asks for the level files it enters, because it starts
  with none; the real client had them cached (it sent `UPDATEFILE gm2x2.gmap` instead). A cache-state
  difference of the lab, not a wire rule.
* `wbowi1.png`: the server sends `PLO_DEFAULTWEAPON` ids 8 then 7 and the real client asks for
  `wbomb1.png` then `wbowi1.png`. **Done.** Id 7 is the bow (`wbowi1.png`, lab 2026-10-25, real 2.22
  with the ids filtered to 7 alone) and id 8 the bomb (`wbomb1`, the 2.05 rev 2 item table,
  Graal_analysis `client-internals/status-display.md`). The login frame asks the image of each
  default weapon in arrival order, after the server weapons' images, once per name
  (`protocol::login::default_weapon_image`).

## 8. Walk update fields and the 2.22 cadence

Decoded from the real traces (`gmap_cross`, `door_warp`):

| build | first step | following steps | stop | cadence |
|---|---|---|---|---|
| 2.22 | `GANI=walk, X (or Y), SPRITE` | `X` (or `Y`) alone, one tile further each message | `GANI=idle` | one message per 100 ms (every second 50 ms step); the `gr` flags go per step |
| 2.31 | `GANI=walk, SPRITE, X2` (or `Y2`) | `X2` (or `Y2`) alone, half a tile | `GANI=idle` | one message per 50 ms |
| Seihai 2.22 | as the real 2.22 | as the real 2.22 | the last position, then `GANI=idle` | as the real 2.22 (`protocol/walkcadence.rs`) |
| Seihai, other builds | `X, Y, SPRITE` | `X, Y, SPRITE` | nothing | one message per step (50 ms after `walk-pace.md`) |

2.22 is now done. Lab, 2026-10-30, on the lab GServer-v2: the real 2.22 walked
right, swung the sword, and walked left, and Seihai 2.22 then ran the same steps
with a fresh account each time. Seihai's props matched message for message. They
had the same fields, the same positions (30.5, 31.5 ... 35.5 and 35, 34 ... 30),
one message every second 50 ms tick, `GANI idle` two ticks after the last step,
and `GANI sword` with no `SPRITE`. The Graal the Adventure capture (Graal_analysis
`live-runs/2gta-222-2026-10-30/`) settled the stop on an odd half-step: that
position goes in the next slot, and `idle` goes in the one after.

What is still different:

* Seihai's `gr` flags come from its script timer, which is not on the 50 ms tick.
  Some frames carry two flags and some props go without one. The real client puts
  one flag in each tick's frame.
* The sword: `idle` followed `sword` after 287 ms in Seihai and 241 ms in the real
  client. The swing now lasts the 5-step `sword.gani` (`set_swing_ticks`), and
  the rest is the tick grid.
* The bush cut goes in the sword's frame (the tick frame takes the tile changes),
  as in the Graal the Adventure capture. The lab walk hit no bush, so this was not
  seen on the wire from Seihai.
* A turn or a diagonal step was not captured.

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
