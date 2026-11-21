# #458 captures: Ato 5.12 over Graal3D, the derived data

Sources: runs 1 to 3 (2026-11-13) and pass 2 (2026-11-14) on lab host A, Ato against seihai-gserver-v2 with `graal3d = true`, loopback only. The setup is in the write-ups under [`docs/lab/`](../README.md). Times are seconds.

Most of these are Ato's numbers: they describe what Seihai sends, and what the fork accepts. The `real512-*` table is an unmodified 5.12 in the Windows 10 lab VM (2026-11-19, [write-up](../../lab/2026-11-19-lab458-real512/README.md)). It settles the open question: a real 5.12 sends no velocity and has no walk or turn speed of its own.

## Tables

| File | What it holds |
|---|---|
| `login-milestones.tsv` | When each link milestone printed in runs 1, 2 and 3 |
| `run3-3d-timeline.tsv` | Every `3d:` line and file request of run 3, the run that reached step 6 |
| `run3-tcp-packet-order.tsv` | The TCP login of a 5.12 Graal3D client, packet by packet (direction, id, name, size), from the gserver's dump. The password was masked by the gserver and is not in the table |
| `run3-player-props.tsv` | The 56 player props the lab account received at login, raw bytes and decoded |
| `run3-ghost-seq.tsv` | Which UDP sequence number carried each ghost and the control object |
| `pass2-client-packets.tsv` | All 598 client packets of pass 2 decoded from the UDP capture: time, seq, position, sections, transform axis and angle, flags, chat |
| `pass2-gserver-trace-masks.tsv` | Section dirty masks the gserver's lab trace logged |
| `pass2-input-timeline.tsv` | The synthetic key presses, relative to the first key |
| `real512-client-packets.tsv` | All 140 data packets a real 5.12 sent in the 2026-11-19 run, from its 3D login to the end: time, seq, size, position, sections, transform axis and angle, flags, chat. Sizes include the script's `triggerserver` report events |

## Findings the write-ups did not have

### Ato's motion reaches the wire in real time

The pass 2 write-up first said the 10 s walk "arrived over 12.9 s" and that Ato under Xvfb sends motion slower than real time. The packets don't support that. The 12.9 s ran from the spawn transform at login (seq 4, 2.9 s before the first key), not from the first step.

| Leg | Key held | Transforms span | Change | Rate |
|---|---|---|---|---|
| Walk forward | 10.0 | 10.027 (4.078 to 14.105) | y 0 → 60.094 | 5.99 units/s |
| Turn left | 1.5 | 1.507 | angle 0 → +3.7791 | 2.51 rad/s |
| Walk forward | 2.0 | 2.032 | 12.04 units | 5.93 units/s |
| Turn right | 3.0 | 3.030 | +3.7791 → −3.7489 | 2.48 rad/s |

- The first transform left 2 ms after the key went down. The chat packet left 3 ms after Return.
- The gserver's trace clock agrees with the capture: the walk's first and last updates are at `t=812732` and `t=822764`, which is 10.032 s apart.
- The write-up and the Ato README are corrected in the same change as this folder.

### Packet cadence

- **While moving:** one packet with a transform every 36 ms (median). The 10th to 90th percentile range is 33 to 37 ms. That is about 28 packets a second.
- **While idle:** one packet every 1.003 s (median) with an empty control-object update. There were 118 of these.
- **Totals:** there were 598 client packets: 476 with a transform, 118 empty updates, 1 chat and 3 before the control object existed. None held a velocity section.
- **Per packet:** on the first walk leg, each transform moved 0.214 units (median; 0.106 to 0.261). Each packet carries the newest position, not a fixed step.

### Transform conventions

- **Spawn:** the spawn transform (seq 4) carried `[3 shape labcube.3ds] [5 transform pos 0 0 1 axis 1 0 0 angle 0] [13 flags 0x10]`. The gserver trace logged its dirty mask as `0x800028`.
- **Moving:** every later transform used `axis 0 0 1`, a rotation about +z only. Its mask was `0x20`, and the chat's mask was `0x1000`.
- **Angle:** heading 0 faces +y. Left raises the angle. After the left turn (+3.7791 rad), walking forward 12.04 units moved the player by (+7.168, −9.678). That is 12.04 × (−sin θ, cos θ), so forward is (−sin θ, cos θ).
- **No wrap:** the angle is not wrapped to ±π. It went from +3.7791 to −3.7489 in one turn.

### Link timing (runs 1 to 3)

| Milestone | Run 1 | Run 2 | Run 3 |
|---|---|---|---|
| Connection established | 4.256 | 0.754 | 0.652 |
| MissionStartPhase1 | 4.350 | 0.834 | 0.702 |
| Player ghost | 4.521 | 0.897 | 0.883 |
| Mission lighting done | 4.654 | 1.017 | 1.374 |
| Control object | 4.708 | 1.054 | 1.423 |
| Login step 6 | not reached | not reached | 1.423 |

- **Run 1:** took 4.26 s to establish the UDP link, about 3.6 s longer than runs 2 and 3.
- **Runs 1 and 2:** missed step 6 because the Sky textures never all arrived (the core v0.1.5 fix).
- **Run 3:** the six Sky files took 0.49 s, from the request at 0.883 to the last at 1.369. That gap set when lighting finished.

### Ghosts and sequence numbers (run 3)

| Ghost | Class | UDP seq |
|---|---|---|
| 0 | GameObject `Ground` | 1 |
| 1 | GameObject `Cube` | 2 |
| 2 | Sun | 2 |
| 3 | Sky | 3 |
| 4 | GameObject `G3DPlayer_<account>` | 3 |
| 5 | Camera `G3DCamera_<account>` | 4 |
| control object = ghost 4 | sent and delivered | 6 |

### TCP login order (run 3)

The server sent these, in this order:
1. `PLO_SIGNATURE`, `PLO_ONLINEMODE`, `PLO_SERVERLISTCONNECTED`, `PLO_RESETCLIENT`
2. `PLO_RPGWINDOW`, `PLO_STARTMESSAGE`, `PLO_SERVERTEXT`, `PLO_STAFFGUILDS`, `PLO_STATUSLIST`, `PLO_PLAYERPROPS` (183 bytes)
3. Two `NPCWEAPONDEL`, two `DEFAULTWEAPON` and two `NPCWEAPONADD`
4. `PLO_PLAYERWARP`, then the level: `LEVELNAME`, `LEVELBOARD`, an 8193-byte `BOARDPACKET`, three `LEVELCHEST`, `LEVELMODTIME`, `LEVELNAME` again
5. `ISLEADER`, `GHOSTICON`, `ISLEADER`, `SETACTIVELEVEL`, `PLAYERPROPS` (3 bytes)

Then the client sent `PLI_LANGUAGE`, `PLI_ENTERLEVEL` and eight `PLI_WANTFILE`: the two `.3ds` shapes and the six Sky textures. Each got one `PLO_FILE`, and `PLO_NEWWORLDTIME` closed the login.

### A real 5.12 walking (2026-11-19)

A client script moved the player through `player.position` and `player.transform`; the [write-up](../../lab/2026-11-19-lab458-real512/README.md) has the setup.

| Leg | Transform updates | Change |
|---|---|---|
| Walk forward | 26 | y −6 → 1.8, 0.3 per update |
| Turn left | 7 | rotation only |
| Walk forward | 17 | (−0.23, 1.99) → (−3.91, 5.07) |
| Turn right | 8 | rotation only, back through zero |

- **Cadence:** one transform per packet, only after the script changed it. The median interval while walking was 0.17 s, the script's own tick under the VM's software OpenGL. Idle packets came about every 1.1 s.
- **Rotation:** no rotation goes out as axis (1, 0, 0), angle 0. A right turn goes out as axis (0, 0, −1) with a positive angle. The axis carries small errors (0.9991 to 1.0005).
- **Units:** the angle a script writes into `transform` is in degrees, and the wire angle is in radians (1.0 written, 0.0175 sent).
- **Velocity:** none. Writing `player.velocity` read back as `0,0,0`, and no section 2 went out.
- **Chat:** one section 9, `hi e:smile`, from `player.chat`.
