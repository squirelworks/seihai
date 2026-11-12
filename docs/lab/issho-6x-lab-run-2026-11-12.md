# Issho 6.x lab run, 2026-11-12 (#441, #444)

Build: `seihai-issho` from #449's branch `claude/project-thread-bc7wsy` at 3cb9387, built with
`cargo build --release -p seihai-issho` (default features: audio, window). seihai-core v0.0.11 (d76d2bb).

Where: the headless Debian 13 lab VM on Mirae (2 vCPUs), Xvfb with Mesa llvmpipe, no window manager.
Everything ran on 127.0.0.1, with no public servers. All rows ran from about 12:36 to 12:48 UTC.

## Lab server

A private copy of the lab's modern world, "Issho 6x Lab", run by gserver-v2 (generation modern,
`allowedversions G3D20126:G3D0511C`). It listens on **127.0.0.1:14817** and is not registered with the
listserver. `--online` takes a gserver address; 14900 is the port gservers use to register with the listserver.

- `issho_a.nw`: a link on its right edge to `issho_b.nw` at 2,30.
- `issho_b.nw`: a link on its left edge back to A, and a link on its bottom edge to `issho_ga.nw` at 30,3.
- `isshomap.gmap`: 2x1, made of `issho_ga` and `issho_gb`.
- `steps2.wav` and `water.wav` are on the server only. They are removed from the client's assets, which are
  otherwise the Linux 3.04 client's levels and sounds.

Each row's actor account is deleted before the run, so the actor starts at `issho_a` 30,30. The observer is
a second Issho instance with the same profile (account `isshoobs`).

## Method

- The actor and the observer run in separate windows.
- xdotool sends arrow keys to the actor: Right 1.2 s, Left/Right 0.6 s, Right 4 s (link to B), Right 2.5 s,
  Down 6.5 s (link to the gmap), Right 6.5 s (gmap crossing), Left/Right 1 s. A screenshot follows each step.
- Audio goes to a PulseAudio null sink (`PULSE_SINK`), because the VM's AC97 sink stalls. It is recorded with
  `parec` and measured as RMS per 0.25 s.
- After logout, the server's saved account gives LEVEL, X and Y.
- Sound control: the same run with `steps2.wav` present locally gave 91 loud quarter-seconds (peak RMS 2881),
  so the audio path works.

## Results

| Row | Token | Login+warp | Level drawn | Walk (server + other client) | Link | Gmap crossing | Missing sound: WANTFILE / plays after |
|---|---|---|---|---|---|---|---|
| 6.00r2 | G3D26090 | PASS | PASS | PASS | PASS | PASS | PASS / FAIL |
| 6.015 | G3D3007A | PASS | PASS | PASS | PASS | PASS | PASS / FAIL |
| 6.034 | G3D2505C | PASS | PASS | PASS | PASS | PASS | PASS / FAIL |
| 6.03r7 | G3D0311C | PASS | PASS | PASS | PASS | PASS | PASS / FAIL |
| 6.03r7 Linux | G3D0511C | PASS | PASS | PASS | PASS | PASS | PASS / FAIL |

No panics in any row. The Issho window is 640x480 in every row. The gserver labels G3D26090 as "5.31x",
from gserver-v2's version table.

Evidence:
- **Login:** `logged in at 127.0.0.1:14817 as issho<row>`, then `level issho_a.nw … start (30, 30)`. The
  server logged `New login: … [<token>]`. The `--no-window --first-level` run returned 0 for all five rows.
- **Walking:** the observer's window draws the actor at x 39–40. The server's saved position matches the
  client's, for example 6.03r7: `LEVEL isshomap.gmap X 14 Y 14`, with the client showing 14.0, 14.0.
- **Links:** `link to issho_b.nw at 2 30`, then `link to issho_ga.nw at 30 3` and `requesting isshomap.gmap`.
- **Gmap:** `gmap crossing (0, 0) -> (1, 0) at 0 <y>`, then `level issho_gb.nw`. The crossing y was 18, 17.5,
  10.5, 14 and 11.5 by row, because the player keeps walking down after the link.
- **WANTFILE:** `sound: missing sound steps2.wav`, then `requesting steps2.wav`, once in every row.

### FAIL: a fetched sound never plays

After the request, about 45 s of walking recorded 0 loud quarter-seconds; the control run had 91.

Cause:
- seihai-core `client/mod.rs` creates `FileSession::new(file_resolver, false)`, so downloads are kept in
  memory only.
- seihai-play `sound_engine` builds `SoundEngine::with_dirs` over the asset folders, and
  `SoundEngine::load`/`find` only read files from those folders. Nothing handed the received bytes to the engine.
- `need_missing` then asks for nothing, because the store already has a copy. So the sound stayed silent with
  no warning.

Fix: seihai-core passes each sound the server sends to the view through `Live::arrived_sounds`, and
`SoundEngine::add` decodes it into the engine's cache. seihai-play hands them on in `Game::play_sounds`.

### Other findings

- **Other players' name tags read "default"** instead of the account name. A 6.x build has neither a
  `LoginBurst` nor a 4.x lobby frame, so Issho never sent its nickname and the server kept its default.
  Fix: seihai-core sends `PLI_PLAYERPROPS` with `NICKNAME` alone once the first level is in, for 6.x builds
  (`protocol::login::nick_props_message`). This is a stand-in: there is no capture of a real 6.x
  post-login frame yet.
- After a gmap crossing, the window header still shows the link's arrival `start (30, 3)`. The player's
  position itself is correct. Not fixed here.

## #444: real 6.x client

- **Graal 6.03r7 Linux x64, real and unmodified**, in the same VM. It ran in its own bwrap network namespace,
  with wildcard DNS to 127.0.0.1 and a copy of the lab world on 14900, where this build dials.
  - Window: **1024x768** (xwininfo "Graal": 1024x768+128+16).
  - It logged in as guest with G3D0511C, printed "Welcome to Issho real 603.", received `issho_a` and
    downloaded `pics1.png`.
  - The window stays black, so there is no HUD screenshot.
  - Tried without success: GLX on, MIT-SHM off, 1280x1024, `LIBGL_ALWAYS_SOFTWARE`,
    `MESA_GL_VERSION_OVERRIDE=3.3` / GLSL 330, and the xfwm4 window manager with focus and a click.
    `glxgears` in the same namespace draws and screenshots fine. The 2026-11-09 lab got the same black window.
- **Windows 6.00r2 under Wine: blocked.** The VM has no 32-bit Wine, and installing `wine32` needs root.
- **Next for the HUD:** run the packed 6.00r2 build natively in the Win10 lab VM.

Evidence (screenshots, actor and observer logs, server logs, account files, audio RMS, xwininfo) is kept
on the lab machine, outside the repo.
