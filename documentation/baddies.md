# Baddies

Derived from the 2.05 client (its `TComputer`), the 2.22 `arrays.dat`, GServer-v2 and the LAN captures in `lan-222g-testworld-npc-baddy.txt` (2.22) and `run-231c.md` (2.31). Code: `seihai_core::baddies` (one baddy), `offline/baddies.rs` (the session's list, hits, the wire), `render/baddies.rs` (drawing). Units are tiles unless a value says pixels.

## Who runs them

* **Offline** the client spawns the level's `BADDY x y type` / three verse lines / `BADDYEND` records (ids 1, 2, ...) and runs them.
* **Online** the server owns the list. PLO_BADDYPROPS (2) adds or updates one; a new level clears the list. Only the **level leader** (PLO_ISLEADER, 10) runs the AI. Every other client draws the props it receives.
* A baddy thinks on every second player tick: every 100 ms.

## Kinds

| type | image (`.gif` up to 2.05, `.png` after) | power |
|---|---|---|
| 0 gray soldier | baddygray | 2 |
| 1 blue soldier | baddyblue | 3 |
| 2 red soldier | baddyred | 4 |
| 3 shooting soldier (bow) | baddyblue | 3 |
| 4 swamp soldier | baddygray | 2 |
| 5 frog / hare | baddyhare | 1 |
| 6 octopus | baddyoctopus | 1 |
| 7 gold soldier | baddygold | 6 |
| 8 lizardon | baddylizardon | 12 |
| 9 dragon | baddydragon | 8 |

Modes: 0 walk, 1 look, 2 hunt, 3 hurt, 4 bumped (sword clash), 5 die, 6 swamp, 7 hare jump, 8 octopus shot, 9 dead.

## Wire

Every value is +32. A text value is a length byte and then the text.

PLO_BADDYPROPS / PLI_BADDYPROPS (15): `id`, then `prop, value` pairs.

| prop | value |
|---|---|
| 0 | id |
| 1, 2 | x * 2, y * 2 (rounded) |
| 3 | type |
| 4 | power, then the image (text) |
| 5 | mode |
| 6 | counter (the animation counter) |
| 7 | dir * 4 + head |
| 8, 9, 10 | verses: on sight, on hurt, on attack (text) |

* The captures confirm this. The 2.22 server sends 6 baddies of 67 to 68 bytes. On level entry the leader (2.22 and 2.31) sends 6 messages of 5,5,5,5,3,5 bytes, then 6 of 3 bytes 100 ms later.
* The real 2.00 as leader (`lan-200west.txt`) shows how a hunt goes, and Seihai matches it (test `the_hunt_matches_the_2_00_capture`):
  * the baddy looks around with counter 44;
  * it then hunts with counter 16, counting down to 5 without moving;
  * then the counter goes to 0, and the baddy moves half a tile a think while the counter cycles 3, 2, 1, 0.
* The same capture shows the touch damage: 1 half heart, and 2 from the gold soldier (baddy 6, type 7).
* In 2.00 and in most of the 2.22 runs, the touches come 2 s apart (the player's 40-tick hurt time), as in Seihai.
* The player's knockback from a touch is 1.0 tile a tick for 5 ticks, away from the baddy. The real 2.22 shows this in the lab's position log (`lab-baddies.md`), and it is the same knockback Seihai already uses for sword hits.
* One 2.22 run has hits 0.6 to 1.1 s apart: run 16-20 in `lab-baddies-20261031T032657-c861.md`.
  * It happened while the player was swinging at a soldier.
  * In the client, a sword clash puts the player into a pushed state, and Seihai does not do that yet. That is the likely cause, but it is not proven.
* That capture has no baddy getting hurt, so it says nothing about the knockback.
* The real 2.22 leader's sword kill (`lan-222kill-baddy.txt`, testbaddy) matches Seihai byte for byte (test `the_kill_matches_the_2_22_leader`):
  * Each hurt message carries the new power with an empty image, mode 3 and counter 7. The baddy has already taken its first knockback step in that think.
  * Then 5 more steps follow, 100 ms apart.
  * After those comes counter 1, then mode 5 with counter 14, counting down to 0.
  * The leader sends nothing after that. The server sends mode 9 about 1.2 s later.
* The leader stores each baddy's props on its first think and sends only the changed props after that.
* **Send rate.** The leader sends one PLI_BADDYPROPS per baddy per think, with only the changed props, and all of one think's messages go in one frame.
  * The real 2.22 (`lan-222d`) and 2.00 (`lan-200west`) leaders do this for as long as they stay in the level: 48 to 60 messages a second for 6 baddies, about 45 bytes a second.
  * The walk and look counters change every think, so a moving baddy is in nearly every think. A baddy with no change is left out.
  * So 12 baddies make about 120 messages a second, under 1 KB/s. Seihai matches it (test `a_leader_sends_one_message_per_baddy_per_think_like_2_22`: 579 in 10 s for the 6 `lan-222d` baddies).
  * There are no leader captures from 1.41 or 5.x.
* For a custom image the leader sends the type's default image name. GServer notes "client resets the custom image when the baddy is hurt".

PLI_ARROWADD (9): the leader sends each baddy shot as `x * 2`, `y * 2`, `dir`, `sprite`, `kind + 1`. The real 2.22 sends `87 79 20 8b 21` for an arrow facing north (sprite 107 + dir).
* The octopus's rock (sprite 131) goes through the same function in the client, so Seihai sends it the same way.
* A rock has not been seen on the wire.

PLI_BADDYHURT (16) → PLO_BADDYHURT (27): `id`, `floor((dx + 4) * 16)`, `floor((dy + 4) * 16)`, `power`. `dx, dy` is the player's centre minus the baddy's centre, clamped to ±4. A follower that hits a baddy sends this message, and the leader applies the hurt. A real 2.22 follower sent `21 28 60 21` (`lan-222hurt`). It stood at 42, 45 facing east, and the baddy was at its spawn point 46, 46, so dx is −3.5 and dy is 0.
* Next the follower sends PLI_BADDYPROPS `id, 4 power "" , 5 3` (`21 24 21 20 25 23`), which the server echoes to the others.
* Seihai sends both, byte for byte (test `a_follower_hit_matches_the_2_22_capture`).

## Geometry and damage

* Baddy centre `(x + 1, y + 1)`. Player centre `(x + 1.5, y + 2)`.
* Steps per think: north `(0, -0.25)`, west `(-0.25, 0)`, south `(0, 0.25)`, east `(0.25, 0)`.
* **Blocking:**
  * off the board;
  * a blocking tile, water, a chair or the bottom of a bed;
  * a chest.
* **Touching the player** costs 1 half heart, or 2 for the gold soldier. The player's hurt time stops a second hit straight away.
* **Sword:** the sword's power, tested while the swing is 50 to 200 ms old. A soldier (types 0, 1, 2, 7) that faces the player within 2 to 5 tiles blocks it. That is a clash: the sound is `swordon.wav`, the baddy is pushed at 0.8, and it takes no damage.
* **Thrown objects:** `2 * 3`, or `2 * 1` for a bush or a bomb.
* **Knockback:** a hurt pushes the baddy 2.0 tiles a think while its counter is above 2, which is 6 thinks. A step into a wall is undone. The 2.22 kill capture confirms it: 6 steps of +3 or +4 half tiles in x and −2 or −3 in y.

## Drawing

* Each frame is a list of pieces from `arrays.dat`: array 20 for the poses (index `ani * 4 + dir`), 22 for the death puff, 23 for the swamp. Each piece is `sprite << 16 | x | y`, with offsets in 2 px units. Array 24 holds the sprite rects.
* Copies of these arrays are built in, for folders without `arrays.dat`.
* Which picture a piece is cut from:
  * sprites 48–63, 104–106 and 117–130: the baddy's image;
  * 111–114: `wbow1`;
  * everything else: `sprites.png`.
* The verse is drawn centred over `(x + 1, y - 2)` for 20 thinks.
* Sounds: `steps.wav`, `swordon.wav`, `frog.wav` (hare jump), `compudead.wav`.

## Not done yet

* Arrows and bombs hurting baddies.
* The clash pushing the player back.
* The swamp splash.
* The terrain sprites drawn over a baddy that stands in grass or water.
* The `compusdied` NPC event.

## Probe

`cargo run -p seihai-mae --no-default-features --features dev --example baddy_probe -- <assets> <level.nw> <out> 30 30 400 swing` runs a level's baddies next to a standing player and writes a PNG every 500 ms. With `swing`, the player swings at any baddy within reach.
