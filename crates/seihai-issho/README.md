# seihai-issho

Issho (一緒, "together") is the **v6 client profile**.

The `seihai` launcher starts this binary when `--version` is a 6.x row
(`6.00r2`, `6.015`, `6.034`, `6.03r7`, `6.03r7 Linux`). With no
`--version` the launcher still starts `seihai-mae`. v3–v5 are not Issho.
A 7.x label is rejected and nothing is started.

With only `--version` the binary prints the profile and exits.

With `--online host:port` (`-o`, `--server`), `--account NAME` and
`--password-env VAR` it logs in to that game server as the 6.x build and
opens the game window. The loop is `seihai-play`'s, the one `seihai-ato`
runs (Issho does not depend on Ato): core's client session logs in (the
build's login line, the Login server's warp to the game server, the
post-login frames where the build has recorded ones), the level from its
board (or a local copy at the server's mod time with `--assets DIR`) is
drawn flat, the arrow keys walk it, links and gmap edges are crossed, the
other players and the server's board and NPC changes are shown, and the
walk's sounds play through core's sound engine; a sound not on disk is
asked of the server with `WANTFILE`. `--no-sound` keeps it quiet. The
server NPCs' GS2 scripts are listed, not run. The password is read from
the variable, never from the command line.

`--no-window` only logs each level, as before (`--first-level` hangs up
once the first level is logged). `--level NAME` plays level files
offline in the window, looked up in `--assets`.

```
LAB_PW=… seihai-issho --version 6.03r7 --online 127.0.0.1:14900 --account guest --password-env LAB_PW
seihai-issho --version 6.03r7 --level start.nw --assets DIR
```

The window is the shared flat stand-in, not the 6.x playfield: 640 x 480,
markers for the players, no HUD, no chat, no start window. The roadmap
is in `documentation/roadmap.md`, "Issho (6.x)", and issue 440.
`seihai-mae --version` with a 6.x label still runs mae's flat client;
mae does not link this crate.

The `window` feature (default) brings the window and `seihai-play`. The
launcher links this crate without it, for the version checks only.

## What Issho owns

The unpacked Windows 6.00 images are one `quattro_lib` program, not
Torque. Issho owns that shape:

- **Draw.** OpenGL presents the frame. Direct3D 9 is a capability check
  before that, not the presenter. `GuiGraalCtrl` and `GuiGraal3DCtrl`
  are both playfields. `GuiTilesCtrl` and `horizSizing` are absent in
  6.00. The window is egui's, through `seihai-play`; nothing
  here is the 6.x OpenGL presenter yet.
- **Sound.** The slot is FMOD. Seihai plays through core's sound engine
  (the `audio` feature), as Ato does, not FMOD.
- **GS2 shell.** Startup scripts, including `StartScript_GraalGui` and
  the dialogs. The bytecode VM is not ported. Graal_analysis pull
  requests [276](https://github.com/squirelworks/Graal_analysis/pull/276)
  (`cursor/gs2-from-engines-cae0`, the script machine from the 4.x, 5.x
  and unpacked 6.00 engines) and
  [278](https://github.com/squirelworks/Graal_analysis/pull/278)
  (`cursor/gs2-client-vm-cae0`, container, opcodes, and script load)
  are the notes for a later VM. This crate does not copy them.
- **Connection.** `TSocket`. Not Torque's `GameConnection`.
- **World.** The `TServer*` objects (player, NPC, level, link, weapon,
  and the level objects beside them).

6.00r2 has no `resolveHost` and no HTTP keep-alive header. 6.00r4 adds
both. Both are profile rows and both select Issho. The 6.00r4 row turns
on core's `resolveHost` builtin, a real DNS lookup.

Packed Windows 6.037 and 6.113 are packer stubs with a QPlay.exe CyaSSL
export surface. They are not the engine. CyaSSL is not implemented.
Those labels are not profile rows, so they are rejected.

Facts are the behaviour notes in Graal_analysis:
`client-internals/engine-600r2.md`, `engine-600r4.md`, `packed-6037.md`,
`packed-6113.md`. No binary was run for this crate.

## What stays elsewhere

| Crate | Owns |
|---|---|
| `seihai-core` | Shared protocol facts and formats: framing, profiles, levels, gmaps, ganis, GS1. Issho reads the version row from here and does not restate it. |
| `seihai-mae` | 1.x and 2.x, and the flat play path still used for 4.x and 5.x. Not linked from Issho. |
| `seihai-ato` | v3–v5 Torque. Issho does not depend on Ato and does not call it. |
| `seihai-play` | The game loop and flat renderer Ato and Issho share. |
| `seihai-g3d` | February 2000 Graal3D. A different program. |
| `seihai-launcher` | Reads `--version` and starts one of the lines. v6 starts Issho. |

graal-client-repro is the "how the original was shaped" track. This crate
does not link it and does not share code with it.

## v7

v7 is reserved and unsupported. Unity is the expected engine. There is
no v7 profile, no Unity code, and no Unity version matrix.
`EngineProfile::V7` exists so a later engine has a variant to fill.
Nothing constructs it, and `select_label` rejects `7`, `7.0`, and `v7`.

Issue 377 described Issho as the post-Torque core shared by 6.x and 7.x.
This crate does not do that. 7.x is not a supported version profile.

## Not in this pass

The 6.x playfield and HUD, FMOD, the GS2 VM, HTTP, CyaSSL, Torque, and Unity.
