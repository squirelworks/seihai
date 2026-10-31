# Graal3D (February 2000) in Seihai

Date: 2026-10-31.

`seihai-g3d` is a client for Graal3D, the standalone 3D side project of
February 2000 (`Graal3D.exe` and its own `GServer.exe`). It is not the Torque
3D of the 3.x–5.x clients, which belongs to `seihai-ato` (see
`ato-3d-path.md`).

The wire facts come from Graal_analysis `graal3d/` (`protocol.md`,
`formats.md`, `client-behaviour.md`), which were checked against the real
client and server in a loopback lab. No Graal Reborn server speaks this
protocol; it plays against the original `GServer.exe` or the reference
server in Graal_analysis.

## Running it

```
seihai --version graal3d --online HOST[:PORT] --account A --password-env VAR \
       [--nick N] [--assets GRAAL3D_FOLDER] [--levels DIR] [--no-udp]
seihai-g3d ... --no-window [--first-level] [--say TEXT] [--walk TICKS] [--seconds N]
seihai-g3d ... --screenshot FILE.bmp [--walk TICKS] [--turn TICKS] [--say TEXT]
seihai-g3d --level-info FILE.x
```

The port defaults to the shipped server's. `--assets` is a Graal3D install
folder: the protocol never sends textures or the player meshes, so they
come from there (as they did for the real client). Without it the level
draws in its material colours and players are not drawn.

Each level file is saved in `--levels` (by default `levels/` in the asset
folder, or a folder in the system temp directory). The server's level name
is reduced to a plain file name first, so a server cannot write outside
that folder.

Keys in the window: numpad 8 / 2 or Up / Down walk forward and back,
numpad 4 / 6 or Left / Right step sideways (which turns the follow camera),
numpad 5 or Space jumps, numpad 0 turns the follow camera off and on, Enter
opens and sends the chat line, Esc closes it or quits.

## What it does

| Part | Module | Notes |
|---|---|---|
| Frames | `codec` | `u16` big-endian length + zlib; messages `c(id)` + payload + `\n`; `+32` codec; `§` for line breaks in level text. No key byte, no scramble. |
| Login | `session` | `GR3DV002`, account, password, and the first props line in the same frame, as the real client sends them. |
| Props | `props` | Ids 0–17, 50 and 51 both ways. The client sends only what changed since its last send. The first line carries the whole look (6–12) and the UDP port (15), as captured from the real client. |
| Level | `session`, `xfile` | Packet 1 (name) then 0 (text, or empty for "you already have it"). The text is saved and read as a DirectX `.x` text mesh. |
| Other players | `session` | Packet 2 creates on first sight; prop 51 removes; prop 50 marks a player as in your level. |
| UDP | `session` | One socket (unless `--no-udp`), its port sent as prop 15. Position (props 1–5, no ids) and chat go straight to players in the same level that have a port and an address. Datagrams are accepted only from a known player's address and port. |
| Chat | `session` | `say` sends to UDP peers at once and to the server as prop 13 in the next props diff. Lines over 200 bytes are cut and end in `...`. |

Assumed, not captured: the client does not send back props the server just
set in packet 3. (The lab's real client had its movement routine patched out
under Wine, so this could not be seen.)

## `.x` reader

`xfile` reads the text `.x` files Graal3D uses: templates are skipped, named
top-level materials are resolved when a mesh's material list references
them, and faces, per-face materials and texture coordinates are kept.
Binary and compressed `.x` files are refused. The shipped `startlevel.x`
reads as one mesh of 139 vertices and 83 faces with seven textures, which
matches Graal_analysis.

## Lab check, 2026-10-31

On a loopback lab in the cloud container, with the original `GServer.exe`
under Wine and, separately, the Graal_analysis reference server:

- Two `seihai-g3d` clients logged in, got their own props, the level and
  each other's full props. The saved `startlevel.x` was byte-identical to
  the server's copy.
- B's chat and ten walking steps reached A over UDP; A's chat reached B
  through the server.
- A wrong password got the server's "An account with the name and password
  you typed in doesn't exist." and a closed socket. The rejected session's
  props reached the others as a throwaway player, as Graal_analysis notes.
- Both servers produced the same events for this run.

## Drawing (stage 2)

| Part | Module | Notes |
|---|---|---|
| Renderer | `render` | Software: perspective camera, near-plane clipping, depth buffer, perspective-correct textures (nearest, wrapped), one directional light. A stand-in for Direct3D Retained Mode, not a copy of its shading. 640×480. |
| Textures | `bmp` | 24-bit, and 8-bit with a palette, plain or RLE8 (the shipped level textures use RLE8). |
| Player model | `model` | Eight meshes posed by a keyframe animation whose time is prop 5: 0 standing, 1–40 one walk cycle, 40 the jump pose. The position and rotation keys were read from the client binary. The hands are drawn in their arm's frame, because their meshes are in the arm's coordinates; that frame tree is inferred. The model faces −z. Props 6–9 pick the body and head meshes and textures, and props 10, 11 and 17 colour the arms, shoes and hands. |
| Walking | `world` | 8 units a tick, 12 after 25 ticks. Back alone walks backwards along the direction faced when the key went down. ±4000 on x and z. Starting to walk sets prop 5 to 0 or 20 at random, and each walking tick moves it on by one. |
| Jumping | `world` | Up 10, 10, 9, 8, 6, 3, 1 a tick, then the table back down and 10 a tick until landing. Letting go early starts the fall at once. A new jump needs the key pressed again. |
| Ground | `world` | Seihai's own rule, since the real face test was not read: stand on the highest upward face under the player no more than 16 above the feet, and refuse a step into a surface between 16 and 50 above them. |
| Camera | `world` | Follows at 220 units, closing the gap by `((d − 220) / 220)² / 10` a tick and turning up to 5° a tick. The player's facing (prop 4) is the camera-to-player direction. The camera height and look point are Seihai's choices. |
| Chat log | `view` | `nick: text` at the bottom left, x 5, newest at the bottom, white, 10 s a line. There are no chat bubbles in Graal3D. |

Seihai's choices where the binary wasn't read: prop 5 returns to 0 (the
standing pose) when the player stops; the field of view; the light.

`cargo run -p seihai-g3d --example model -- GRAAL3D_FOLDER out.bmp` draws the
model from the front and the side at walk times 0, 1, 11 and 21.

## Lab check, stage 2 (2026-10-31)

Against the original `GServer.exe` under Wine: one client stood still and
another walked and turned. `--screenshot` from the walker showed the
textured level, both players posed, and both chat lines in the log. The
window under Xvfb walked with the arrow keys, and Enter, typing and Enter
put the line in the log and sent it.

## Next

- The real client's collision test and camera height, from the binary.
- Nicknames: Graal3D shows none over players; a player list might help.
- The F1 / F2 option windows (UDP options, the look editor).
