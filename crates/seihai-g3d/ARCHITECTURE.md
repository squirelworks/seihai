# seihai-g3d architecture and handoff

Date: 2026-10-31. For whoever picks this up next (Cursor, Grok, Chiaki or a
person). User-facing notes are in `documentation/graal3d.md`; this file is
about the code.

`seihai-g3d` is the client for **Graal3D (February 2000)**, the standalone 3D
side project with its own server (`GR3DV002`). It is not the Torque 3D of the
3.x–5.x clients: that is `seihai-ato` (Stage 4, `documentation/ato-3d-path.md`).

Sources of truth, in order: Graal_analysis `graal3d/` (`protocol.md`,
`formats.md`, `client-behaviour.md`, `lab-2026-10-30.md`), lab captures,
then the client binary. Write binary findings up as plain behaviour: no
addresses, no decompiled code (project rule).

## Module map

```
main.rs      CLI, the Game struct (one tick = network + movement), the three
             run modes: window (minifb), --no-window (log), --screenshot
lib.rs       module list
codec.rs     +32 codec, u16 BE + zlib frames, Deframer, § line breaks
props.rs     Prop enum (ids 0–17, 50, 51), Player state, Differ (per-tick
             diff of the client's own props)
session.rs   Session: TCP login + receive dispatch + UDP peer socket.
             Emits Event values; knows nothing about drawing
xfile.rs     DirectX .x text reader: templates skipped, materials resolved
bmp.rs       BMP read (24-bit, 8-bit plain/RLE8) and write
render.rs    software rasterizer: Camera, Frame, Mat (3×4), Placement,
             draw_mesh
model.rs     player model: 8 parts, keyframes read from the client, pose()
world.rs     Walker (walk/run/jump), FollowCam + CameraRig, Collider trait,
             Ground (the stand-in collider), bob()
view.rs      Assets (files from --levels and --assets), draw_player, the
             chat log, draw()
config.rs    graal3dconfig.txt read/write
admin.rs     admin protocol ids and encoders (scaffold, not used yet)
examples/model.rs   renders the model in several poses to a BMP
```

## One tick (50 ms, the client's timer)

1. `Session::tick` reads TCP frames and UDP datagrams and returns `Event`s.
   It applies props to `me` / `others`, saves level text, and marks
   server-set own props as sent (so they are not echoed).
2. `Game::tick` turns events into state: level parsed into `XFile` and
   `Ground`, textures loaded, chat lines pushed, camera placed on first
   own props.
3. `Walker::tick` moves `me` from the held keys, asking a `Collider`.
4. `FollowCam::tick` follows.
5. Next `Session::tick` sends the props diff (TCP) and, if props 1–5
   changed, a UDP position datagram to each peer.
6. `view::draw` renders; the window shows the frame.

## Fact or stand-in

| Thing | Status | Where |
|---|---|---|
| Wire: login, props, ids, UDP datagrams | fact (lab capture) | codec, props, session |
| First props line = props 6–12 + 15 | fact (real client capture) | props::Differ |
| Not echoing server-set own props | stand-in (not capturable under Wine) | session |
| Level / model / texture formats | fact | xfile, bmp |
| Model part offsets and walk/jump keys | fact (client binary) | model |
| Hand frames are children of the arms | inferred from mesh coordinates | model |
| Walk speeds, run delay, jump table, back-walk rule | fact (client binary) | world |
| Camera distance 220 and 5°/tick turn | fact | world::CameraRig |
| Camera height, look point, field of view | **stand-in in code** (G3D-2); numbers are in Graal_analysis `graal3d/camera.md` | world::CameraRig |
| Collision | **stand-in in code** (G3D-1); rule is in Graal_analysis `graal3d/collision.md` | world::Ground via Collider |
| Prop 5 back to 0 when stopping | stand-in | world::Walker |
| Shading, light | stand-in (D3D RM not copied) | render |

## Open tasks

Ordered milestones (M1–M5, what hosting needs, what is out of scope):
`documentation/plannedmilestones.md`, "Graal3D (February 2000)". Each task
below is still one branch and PR. Run `cargo test -p seihai-g3d` and
`cargo clippy -p seihai-g3d --all-targets` before pushing.

- **G3D-1 Faithful collision (M1).** Plain behaviour is in Graal_analysis
  `graal3d/collision.md` (#164). Implement a new `Collider` in `world.rs`
  and use it in `main.rs` (`Game::ground`). Keep `Ground` for tests or
  delete it. Done when the new collider passes the existing walker tests
  that still hold, plus tests for the real rule, and the lab screenshot
  walk still works.
- **G3D-2 Camera numbers (M1).** Plain behaviour is in Graal_analysis
  `graal3d/camera.md` (#163): follow 220, look height +80, turn ±5°/tick,
  pitch −0.15 rad, field 0.5 (~53.13°). Set them in `CameraRig::default()`.
  Done when the TODOs are gone and a lab screenshot still looks sane.
- **G3D-3 Torque 3D servers** (not this crate; held): Ato Stage 4, until a
  capture of a 3.x/4.x Torque connect exists. See
  `documentation/ato-3d-path.md`.
- **G3D-4 Options windows (M1).** F1 "Hardware options" (fixed UDP port, no
  UDP) and F2 "Player look" (props 6–11, 17), written back with
  `Config::to_text`. Needs `Session::connect` to take a fixed UDP port.
- **G3D-5 Admin session (M2).** `--admin`: log in with `admin::login`, show
  the account list (50) and player list (55), send 61–64 from `admin.rs`.
  The account-record layout for 52/53/60 needs a lab capture first.
- **G3D-6 Without assets (M1).** Draw players as coloured boxes when the
  meshes are missing, and say once which files are missing.
- **G3D-7 Start window (M1).** The client's connect window (nickname, host,
  port, account, password) in egui, like `seihai-ato`'s start window, and
  the launcher's GUI version list offering "Graal3D".
- **G3D-8 Lab with two real clients (M2).** Needs Microsoft's `d3drm.dll` in
  the Wine prefix (DirectX redistributable; the container's network policy
  blocked it). Then capture real UDP between two real clients and check
  Seihai against it.
- **G3D-9 A Graal3D server in Rust (M3).** Port the Python reference server's
  behaviour from `protocol.md` into a **separate** Rust binary (sibling to
  this client, not a stage of `seihai-gserver`). Worlds stay `.x` + local
  BMPs; see plannedmilestones M4 / M5 for content and servertools lab
  start.

Code markers: `grep -rn "TODO(G3D-" crates/seihai-g3d`.

## Lab recipe (cloud container, loopback only)

Never point the client or tools at public servers.

1. `dpkg --add-architecture i386 && apt-get update && apt-get install -y wine32:i386 xvfb xdotool imagemagick`
2. `Xvfb :5 &`, `export DISPLAY=:5 WINEPREFIX=<prefix> WINEARCH=win32`,
   `wineboot -i`.
3. Server folder: `GServer.exe` from `graal3d.7z`, `levels/startlevel.x`,
   and an `accounts.txt` in the format in Graal_analysis `formats.md`.
   Start `wine GServer.exe` and press OK in its "Start on port" dialog
   (xdotool click on the button).
4. Or the Python reference server from Graal_analysis
   `graal3d/reference-server/` (no Wine needed).
5. Client:
   `cargo run -p seihai-g3d -- --online 127.0.0.1 --account A --password-env VAR --assets <graal3d folder> --screenshot out.bmp --walk 30`
   and `--no-window --seconds N` for logs. Two clients at once check UDP.
6. Careful: `pkill -f <name>` with the name on the same shell line kills
   the shell itself; use `ps aux | grep '[n]ame'`.
