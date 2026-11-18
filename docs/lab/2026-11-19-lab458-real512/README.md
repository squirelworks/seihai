# Lab #458: a real 5.12 walks and chats over Graal3D (2026-11-19)

This lab closes the real-client half of [#458](https://github.com/squirelworks/seihai/issues/458) pass 2. Ato's half passed on 2026-11-14 ([pass 2](../2026-11-14-lab458-pass2/README.md)). The open question was what an unmodified Graal 5.12 sends while its player walks, turns and talks in 3D mode: whether it sends velocity, and at what walking and turning speeds.

**Short answer:** a real 5.12 has no walking of its own in 3D mode. The arrow keys only set Graal's key state and call script hooks. A client script moves the player by writing to the player's own `GameObject`, and the client then sends that object's transform (section 5) in its next packet. It sends no velocity. So there is no "real" walking or turning speed to copy: on a real server the speeds came from the server's 3D scripts, and Ato's constants are as good as any.

## Why earlier labs could not walk

Earlier labs (2026-11-12, labs 10 to 12b) found that a 5.12 weapon script saw the arrow keys through `keydown2`, but could not find its own 3D object: `findobject`, a lookup by name, and a walk over `MissionGroup` all came back empty. The lab notes put that down to the arrow keys or the object, and the gserver gained a `triggerserver(..., "g3dpos", ...)` workaround so that the server would move the object instead.

A static read of the 5.12 `Graal3DEngine.dll` showed the real route. **[bin]**

- The key bindings (`moveforward`, `moveforward2`, `turnleft` and the rest) go to one handler. Walking actions call Graal's `graalkeypressed(direction, down)`, which is the 2D key state that `keydown2` reads. The 2D-only names (`moveup2d` and so on) do the same. `turnleft`, `turnright`, `panup`, `pandown` and `mousefire` are swallowed, and `yaw` and `pitch` only change the mouse angles. Nothing in the engine moves the player.
- The engine registers a callback with `GraalEngine.dll` (`graal_setGetGraalPlayerScriptObject`) that decides what a script's `player` is. When the universe's current object is a `GameObject`, `player` is that object's script side. Otherwise it is the `ServerConnection` (`GameConnection`).
- `GameObject::packUpdate` (the client side) writes the sections whose mask bits a script set: name, velocity, shape, skin, transform, movetype, movement data, shape data, chat, an always-clear script-classes flag, attributes, nickname and flags. So the client sends a section only after a script changes that property.

The run below confirms it: **once the control object arrives (login step 5), `player.objecttype()` is `GameObject`** and its position is the spawn point. Before step 5 it is `GameConnection`. A script that writes `player.position` moves the player, and the client sends the change.

## Setup

Everything ran inside the Windows 10 lab VM on Mirae (the clone `Graal RE Lab v7probe2`, headless, NIC cable disconnected, 4 GB). It used loopback only, with no injection into the client.

| Part | Version |
|---|---|
| Client | unmodified Graal 5.12 (`G3D14097`), guest clock held at 2007-11-19, stale `FILENAMECACHE.txt` and `levels3d` left out |
| gserver | seihai-gserver-v2 `seihai` branch, Windows CI artifact of 2026-11-18 (after #102). World `lab3d` (the 2026-11-13 lab world), `graal3d = true`, `graal3dport = 14841`, `serverside = true` (so the 3D weapon is sent as GS2 bytecode), `localhost = true`, TCP 127.0.0.1:14830 |
| Lobby | a small Node script on 127.0.0.1:14900 that answers the gen-5 login with one server warp to `3 Lab 3D` (the hosts file sends the login host to 127.0.0.1) |
| UDP | the same script relays 127.0.0.1:14831 (what the client dials, port + 1) to the gserver's 14841 and logs every datagram with a timestamp |
| Script | [`walk3d.gs2`](walk3d.gs2), sent as the `-Probe3D` 3D weapon (`weapon3d -Probe3D config/probe3d.gs2`) |

The script waits for `player` to become a `GameObject`. Then it walks the player with the arrow keys at Ato's speeds: 6 units/s along the facing, and turns of 2.5 rad/s, through `player.position` and `player.transform`. It moves the camera behind the player and sets `player.chat` on `C`. On `V` and `B` it tries to set and clear a velocity. It reports each step back with `triggerserver`, which the gserver logs as an ignored trigger. The 5.12 console log is only written when the client closes, so this was the only live view.

The keys came from the host through `VBoxManage controlvm keyboardputscancode`, after a click had focused the 3D view: Up about 3 s, Left about 1 s, Up about 2 s, Right about 1 s, then `C`, then `V` and `B`.

## Result: pass

Run 7 (one client; runs 2 to 6 were setup and are not counted).

**The real client walks, and sends its transform.**
- 140 client data packets after login. 50 carried a transform (section 5) while the player moved. The rest carried the empty 13-bit update, as before.
- Up: 26 updates, (0, −6, 1) to (0, 1.8, 1), in steps of 0.3 (one script tick each).
- Left: 7 updates, rotation only.
- Up: 17 updates, (−0.23, 1.99, 1) to (−3.91, 5.07, 1), along the new facing.
- Right: 8 updates, rotation only, back through zero.
- The game section's own position (the client's `pos` field) always matched the transform's position.
- A walking packet was 49 bytes. An idle packet was 21 bytes, about every 1.1 s.

**One update per packet, on change only.** The client sent a transform only in a packet after the script changed it, never twice for one change. While walking, updates came about every 0.17 s (median). That is the script's timer: it asked for 0.05 s, but ran about six times a second under the VM's software OpenGL. The client asks for a 32 ms / 450-byte maximum rate, so on faster hardware the same script would send faster.

**Rotation.**
- The client sends a zero rotation as axis (1, 0, 0), angle 0, as already seen at the control-object echo.
- It sends a turn to the right as axis (0, 0, −1) with a positive angle, not (0, 0, 1) with a negative one.
- The axis is normalised with small errors (0.9991 to 1.0005).
- **The angle a script writes in `transform` is in degrees; the wire is in radians.** The script wrote 1.0 after 0.4 s of turning, and the wire carried 0.0175 (1° in radians).

**Velocity: none.** Writing `player.velocity`, as an array or as a string, had no effect: it read back as `0,0,0`, and no velocity section (section 2) went out. A 5.12 player sends no velocity unless something gives its object a movement object first (`movetype`), which no lab server does.

**Chat (section 9) arrives.** `player.chat = "hi e:smile"` went out once, as section 9 with the text `hi e:smile`. The 3D view itself has no chat bar (Enter opens nothing), so on a real server chat in 3D came from a script too.

## Compared with Ato

| | Real 5.12 (this lab) | Ato (2026-11-14) |
|---|---|---|
| Who moves the player | a client script (`player.position`, `player.transform`) | Ato itself, from the arrow keys |
| Walking speed | none of its own; whatever the script sets | `WALK_SPEED` 6.0 units/s |
| Turning speed | none of its own; whatever the script sets | `TURN_SPEED` 2.5 rad/s |
| Transform (section 5) | in the next packet after each change | in the next packet after each change, every 36 ms while a key is held |
| Velocity (section 2) | never | never |
| Turn to the right | axis (0, 0, −1), positive angle | axis (0, 0, 1), negative angle |
| No rotation | axis (1, 0, 0), angle 0 | axis (0, 0, 1), angle 0 |
| Chat (section 9) | `player.chat` from a script | the window's chat line |

Both are valid: a reader gets the same rotation from either axis. Nothing in this lab asks Ato to change its wire format. The only difference that matters is behaviour: on a real 5.x 3D server, how a player walks is the server's script, not the client. Ato's speeds stay as placeholders, and a server that wants 5.12 players to walk must send a 3D weapon like [`walk3d.gs2`](walk3d.gs2).

## Ato and the real 5.12 in one world (run 8)

Ato (`seihai-ato` built from `main` at `a8b8cec`, `--version 5.12 --graal3d`, account `labato`) joined the same lab world as the running real 5.12, first with `--no-window`, then with its window. Its window drew in the VM under its software renderer.

- Ato reached login step 6 beside the real client, with all 8 files.
- While the real 5.12 walked, the gserver passed its moves on to Ato: 24 transform updates for the real player's ghost.
- While Ato walked (Up for about 2 s), the gserver passed Ato's moves on to the real 5.12: 34 transform updates, (0, −6, 1) to (0, 11.08, 1). The real client took them and stayed linked.
- Ato's window drew the real player as a second cube under the nickname `unknown`, the gserver's name for a guest's object.

One gap showed up: Ato also runs the 3D weapon's GS2. Its GS2 had no `objecttype()`, and its `player` was not its 3D object, so `walk3d.gs2` logged `function player.objecttype not found` every tick and could not move Ato.

**Fixed in this PR** (`scripts3d.rs`, `session.rs`):
- From the control object on, a 3D script's `player` is Ato's own object, as on the real 5.12. `player.objecttype()` gives `GameConnection` before that and `GameObject` after it.
- `player.position`, `player.transform` (angle in degrees) and `player.chat` read and write the own object. What a script writes goes out as sections 5 and 9, the same way Ato's own moves do. Writing `player.velocity` does nothing, as on the real client.
- `keydown2` reads the 3D window's arrows (37 to 40).
- Once a script has moved the player, Ato stops walking it from the keys. The script owns the walk, as on a real 5.12.
- Unit tests cover each of these.

A limit is left in core's GS2 VM (seihai-core): an object stored in a variable or passed as a parameter loses its root. `this.me = player; this.me.position = …` therefore does not reach the player in Ato, while `player.position = …` does. `walk3d.gs2` uses the variable form, so in Ato it still finds nothing. A script written with `player.` directly walks both clients. The new code has unit tests only. It was not run live, because the VM went back to the Windows 10 thread.

## What this changes elsewhere

- seihai-gserver-v2 `docs/graal3d.md` says a 5.12 client script "could not reach its own 3D object", so the gserver moves the object (`g3dpos`). A script can reach it: it is `player` from login step 5 on. The `g3dpos` route still works, but it is not needed.
- Graal_analysis `torque/lab-2026-11-12-udp-trace.md` (lab 12b) says the client "cannot move its own control object by itself". It can, through `player`.

## Notes

- The gserver-v2 GS2 compiler rejected one version of the script with no error text (`weapon3d -Probe3D: not compiled ... sending the source`). The only change from the version that compiled was a line with `this.said` and the string `"hi e:smile"`. Building the string from parts, under another variable name, compiled. The cause wasn't narrowed down further.
- A client that was already running when the gserver restarted logged in again on its own, so runs 4 to 6 had two or three 5.12 clients in the world. Run 7 had one.

## Capture summary

Derived tables are in [`docs/captures/lab458-graal3d/real512-*.tsv`](../../captures/lab458-graal3d/FINDINGS.md). The raw UDP log (hex datagrams), the gserver and lobby logs, the screenshots and the lab scripts stay offline with the owner's lab material, not in this tree.
