# Real 2.31 walking in the cloud lab (2026-10-31)

The 2.31 profile has the precise positions (`precise_pos`, X2/Y2/Z2), but
Seihai walked it with the classic X/Y until a live run checked it. This run
did, and Seihai 2.31 now walks with X2/Y2 and sends `client.` flags only.

## Setup

Cloud container only (no user PC). Wine 9 (`wine32:i386`), Xvfb `:9`, and a
net and mount namespace with only loopback up and a private hosts file
(`listserver.graal.in` and friends at 127.0.0.1). Server: the
`seihai-lab-servers-beta4` pack (GServer-v2 beta4 with the lab patches, lab
listserver v2), port 14802. Client: `Graal_231.exe` from
`Graal Reborn 2.22.0.zip`, run from that install folder. Capture: `tcpdump` on
`lo`, decoded with `seihai capture show --version 2.31`. Decoded logs (masked)
and screenshots are in the project folder at `v2x/lab-231-walk-2026-10-31/`.

## What the real 2.31 sends

- **Walk.** `PLAYERPROPS` every 50 ms with only the props that changed: GANI
  `walk` and SPRITE on the first step, then X2 (78) alone for a horizontal
  walk, Y2 (79) alone for a vertical one, X2 and Y2 for a diagonal, 0.5 tile
  a step; GANI `idle` at the stop. 51 X2 and 53 Y2 values in the run, **no X
  or Y (15, 16) at all**. This matches the earlier LAN capture `lan-231c`.
- **Login props.** NICKNAME, HEADGIF, COLORS, STATUS, UDPPORT, ADDITFLAGS,
  PLANGUAGE, OSTYPE, TEXTCODEPAGE: no position.
- **Script flags.** The account had the stock `-gr_movement` weapon (its
  `setstring gr.x/gr.y` runs every 50 ms): **no `FLAGSET` at all**. With a
  probe weapon added (`setplayerprop #c,..`, `setstring gr.probe,1`,
  `setstring client.probe,2`, `set probeflag` on `playerenters`), the only
  flag sent was `client.probe=2`. So the 2.31 client runs GS1 weapons and
  sends only `client.` flags, as the 4.04r2 does. The real 2.22 sends every
  flag (`gr-flags.md`), so this changed between 2.22 and 2.31.

## Seihai 2.31 before and after

| | before | after |
|---|---|---|
| Walk update | X, Y, SPRITE (15, 16, 17) | SPRITE, X2, Y2 (17, 78, 79) |
| Flags while walking | `gr.x=..`/`gr.y=..` every step (82 in the run) | none |
| Warp frame after 30 (cached link) and the own-link 46 | 46 + X, Y, SPRITE | 46 + SPRITE, X2, Y2 |

The real 2.31 watched Seihai 2.31 walk: GServer-v2 relayed each update to it
as S>C 8 with X/X2 or Y/Y2, and the real client drew `seihai231` at the new
place.

## What changed in the code

- `net::walk_message_for(version, ..)`: SPRITE/X2/Y2 when the profile's
  `precise_pos` is set (2.3, 2.31, 3.153 and the Login-lobby builds), else
  X/Y/SPRITE. `Connection` walk updates, `link_request_frame` and
  `enter_level_extras` use it.
- `net::sends_client_flags_only(version)`: 2.3, 2.31 and the Login-lobby
  builds.

## Not checked

- **3.153** was not run. The only 3.153 binary in the project is the Mac OS X
  PowerPC build, which Wine cannot run. It walks with X2/Y2 now because its
  binary parses X2/Y2/Z2 (`precise_pos`) and the 4.x builds after it send
  them; its flags are unchanged (every flag).
- **2.3** has no binary; it takes 2.31's values.
- Seihai still sends its position (X2/Y2 now) right after login and all three
  walk props each step; the real 2.31 sends no position at login and only the
  props that changed, with GANI `walk`/`idle`.
