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
       [--nick N] [--no-udp] [--levels DIR] [--first-level] \
       [--say TEXT] [--walk TICKS] [--seconds N]
seihai-g3d --level-info FILE.x
```

The port defaults to the shipped server's. Stage 1 is headless: it logs what
the server sends and saves each level file in `--levels` (by default a
folder in the system temp directory). The server's level name is reduced to
a plain file name first, so a server cannot write outside that folder.

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

## Next

Stage 2 draws the level and the players: a software renderer for the `.x`
meshes in a window, the follow camera and walking from
`client-behaviour.md`, collision against the level faces, and the chat log.
