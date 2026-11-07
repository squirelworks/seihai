# seihai-issho

Issho (一緒, "together") is the **v6 client profile**.

The `seihai` launcher starts this binary when `--version` is a 6.x row
(`6.00r2`, `6.015`, `6.034`, `6.03r7`, `6.03r7 Linux`). With no
`--version` the launcher still starts `seihai-mae`. v3–v5 are not Issho.
A 7.x label is rejected and nothing is started.

This pass is the shell boundary. The binary prints the profile and exits.
It does not draw, log in, or open a sound device. `seihai-mae --version`
with a 6.x label still runs mae's flat client; mae does not link this
crate, and the start window inside mae does not hand a 6.x pick here yet.

## What Issho owns

The unpacked Windows 6.00 images are one `quattro_lib` program, not
Torque. Issho owns that shape:

- **Draw.** OpenGL presents the frame. Direct3D 9 is a capability check
  before that, not the presenter. `GuiGraalCtrl` and `GuiGraal3DCtrl`
  are both playfields. `GuiTilesCtrl` and `horizSizing` are absent in
  6.00. Nothing here issues a GL call yet.
- **Sound.** The slot is FMOD. The device stays closed.
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
both. Seihai has a profile row for 6.00r2 and none for 6.00r4, so 6.00r4
is recorded and not selectable.

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

Login, the playfield, FMOD output, the GS2 VM, `resolveHost` as a real
lookup, HTTP, CyaSSL, Torque, and Unity.
