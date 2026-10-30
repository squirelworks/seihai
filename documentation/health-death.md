# Health and death

What Seihai does when the player's hearts reach 0, and what it leaves open.
Everything here is derived behaviour of the 2.05 rev 2 client
(`decomp-findings-2026-10-24/01-death-respawn.md`, `constants-205.tsv`,
`life-states.tsv`, `status-bits-prop18.tsv`) and the server side in
Graal_analysis `04-death-respawn.md`. No lab capture of a death exists yet, so
nothing below is "observed on the wire"; the packet layouts come from the
documents, the timings from the decompile.

## Where it lives

- `client/src/life.rs`: the state machine (`Life`), the two messages
  (`claim_message`, `status_message`), `respawn_half_hearts` and
  `messages_for`, which turns an event into the messages for a version.
- `VersionProfile.life.death` (`DeathRules`, `DEATH_205` in `profile.rs`): the
  numbers. Rows for 2.00 and every later build have it (builds after 2.05
  are an inference, as for cached links); 1.33 to 1.42 have `None` because
  their handler was not traced. 2.00 to 2.04 take `DEATH_205` as read: see
  "2.00 to 2.04" below. With `None`, hearts at 0 do nothing and nothing is
  sent. `ClientVersion::sends_claim()` also needs id 14 in the build's send set.
- `OfflineSession`: `life`, `alignment` (AP), `entry` (the saved entry point),
  `damage_half`, `tick_life`, `respawn`, `note_entry` (the save rule), `no_pk`. The game loop in
  `bin/main.rs` ticks it, reads the keys and sends the messages.

## The sequence (tick = 50 ms)

| Tick | What happens |
|---|---|
| 0 | Hearts reached 0 (or below): the player is dead, faces down, the `playerdies` script event fires (2.01 and later; 2.00 has no such event). The victim sends `PLI_CLAIMPKER` now, if a hit by another player did it. |
| 0 to 27 | Controls ignored (movement, interact). The timer counts up and stops at 30 (1.5 s). |
| 28 | The dead bit (`0x08`) goes out in `PLI_PLAYERPROPS` with `PLPROP_STATUS` (18). Controls work again. |
| after 27 | A direction key, the action key or the sword key (by default arrows, I/J/K/L, A, S or Space) respawns. |

A hit that leaves hearts at 0 starts the death once; a per-frame check keeps
the player dead while hearts are 0 or below. Hearts going back above 0 does not
revive: only a key press does. A player who is dead cannot be hurt (script
`hurt` and `damage_half` do nothing).

## 2.00 to 2.04

The 2.00 r0 death path was read in its own decompile (Ghidra, from the
2.0 release, MD5 c93e659e...): the damage routine sets life state 2 and the
death timer to 0 at 0 hearts, the tick raises the timer to 30, keys respawn
once it is past 27, the STATUS byte gets `0x08` from 28, and the respawn gives
hearts by AP with the same steps (20 and 40, 3 and min(5, max)) and the same
x 30.5, y 30.0 default. One difference: 2.00 fires no `playerdies` event (the
string is not in the 2.00 builds; it is in 2.01 and later).

For the other ten builds (2.00 rev 1 to 3, 2.01 and rev 1, 2.02 and rev 1 and
rev 2, 2.03, 2.04) the same instructions were found in the machine code: the
`cmp [timer], 0x1c / jl / add 8` that sets the dead bit, the compares with
0x1c and 0x1e, and the AP steps in the respawn. The timer field moves (+0xdc
in 2.00 to 2.03, +0xe0 in 2.00 rev 3, +0x110 in 2.04); the constants do not.
The name-group entry save is in all of them too (Graal_analysis
`decomp-findings-2026-10-24-d/02-respawn-entry-save.md`). So 2.00 to 2.04 use
`DEATH_205`.

### Lab: a real 2.00 dying (2026-10-30, LAN)

A real 2.00 was killed by Seihai's sword (decode `lan-200d-death.txt` on the
lab machine). What it sent:

- Each hit: `PLAYERPROPS` CURPOWER (4, 3, 2, 1); the server echoed GANI
  `hurt`. The knockback went out as X steps of 2 tiles every 100 ms, which
  fits 1 tile per 50 ms tick.
- The death, in one frame: `UPDATEFILE dead.gani`, `PLI_CLAIMPKER` with the
  killer's id (`20 26`, id 6), then `PLAYERPROPS` CURPOWER 0, GANI `dead`.
  (The server answered `FILESENDFAILED dead.gani`. Seihai does not send the
  UPDATEFILE.)
- STATUS with `0x08` set (0x1C) 1.35 s later, within a frame of tick 28.
- Respawn on a key press, in one frame: `PLI_LEVELWARP` to the level start
  (x 30, y 32, the saved entry), then `PLAYERPROPS` CURPOWER 6 (full hearts,
  high AP), GANI `idle`, X, Y, SPRITE 2 and STATUS 0x14 (bit cleared).
- The server cut the rupee, arrow and bomb counts and dropped five items
  (`ITEMADD`) round the body.
- The screen: everything but the HUD and the sprites goes black, and a
  parchment scroll in the middle reads "End". It has no timer; it stays until
  a key. Seihai does not draw this yet.

## Wire

- **Claim.** `PLI_CLAIMPKER` (14): `GSHORT attacker player id`, so
  `2E hi lo 0A` (14 + 0x20, two GCHARs, newline). Sent at tick 0, before the
  status. With no known attacker nothing is sent: the notes say the victim
  carries the attacker's id and give no other form. The server ignores a claim
  for itself or an unknown id (Graal_analysis).
- **Status.** `PLI_PLAYERPROPS` (2) with only prop 18. The value is the last
  STATUS the server gave (`PLO_PLAYERPROPS`, 0 if none) with bit `0x08` set. At
  respawn the same value with the bit cleared goes out, when the bit had been
  sent.
- **Respawn.** The notes say there is no respawn packet and no server
  permission: the client moves the player to its saved level entry point and,
  online, sends the current level again at that x/y, so the server sees an
  ordinary level warp. Seihai sends `PLI_LEVELWARP` (0) (never the cached `30`
  form) and then the status clear, in one frame.
- The server's side (Graal_analysis): the dead bit going 0 to 1 counts a death
  and drops items; going 1 to 0 sets hearts by AP and pushes CURPOWER.

## Hearts report

The real client tells the server its own hearts; GServer does not work them
out for a hit and never echoes the client's CURPOWER. A client that keeps
hearts to itself leaves the server holding full hearts. Seihai now sends,
in `PLI_PLAYERPROPS` (2), CURPOWER (2), GANI (10, a GSTRING) and SPRITE (17)
in that order, each only when it changed (`life::PowerReporter`,
`OfflineSession::take_power_report`, `send_power_report` in `bin/main.rs`):

| When | Sent | Source |
|---|---|---|
| A hit (sword, packet 40/46, thrown object) | CURPOWER, GANI `hurt`, SPRITE (the hurt sprite), in the frame of the hit | 2.05 lab run 1 (`22 25 2A 24 hurt 31 A0`) and the 2.22 lab |
| Hurt mode ends | GANI `idle` | 2.05 lab run 1 (`2A 24 idle`, 1.5 s after the hit) |
| A killing hit | CURPOWER 0, GANI `dead` | 2.22 lab |
| The server set hearts to 0 | GANI `dead` | 2.22 lab |
| Respawn | CURPOWER by the AP table, GANI `idle` | 2.22 lab; 2.05 live run 2026-10-29 (CURPOWER, `idle`, X, Y and STATUS in one props message after the level warp) |
| Lava burn, heart pickup, script `hurt` | CURPOWER | inferred, not captured |

Hearts from the server (MAXPOWER, CURPOWER) count as known and are not sent
back. Builds without death rules send nothing; builds without GANIs leave
GANI out. Not yet captured for 2.05: lava and heart pickup. The 2.05 live run also showed that a server CURPOWER 0 with MAXPOWER 0 loops the real client exactly as it loops Seihai.

### Developer option: an account with 0 heart containers

When the server's MAXPOWER is 0 (for example an account file with `MAXHP 0`),
GServer-v2 caps every hearts value at 0, including the hearts it gives on
revive, so the player dies again after each respawn. A real client loops the
same way. `--dev-repair-hearts` (only in builds with the `dev` feature) sends
MAXPOWER 3 (GServer's default `heartlimit`) once, when the server's login
props say 0. A server built without its NPC server takes it, fills the hearts
and answers with CURPOWER; one built with it ignores the client's MAXPOWER.
No real client sends this.

## Respawn hearts

| AP | Hearts |
|---|---|
| below 20 | 3 |
| 20 to 39 | min(5, max hearts) |
| 40 or more | max hearts |

Seihai keeps hearts in half hearts (CURPOWER is hearts x 2). AP starts at 50
(the GServer default account) and follows `PLPROP_ALIGNMENT` (32) from the
server. MAXPOWER (1), CURPOWER (2) and STATUS (18) from the server are also
applied now, so a server that sets hearts to 0 kills the player.

## Respawn destination

The saved entry point is the level, x, y and direction of the **last level
entry that was saved**; `note_entry` decides. The client does not save every
entry. It saves when:

1. there was no level before (login or the first entry), or
2. exactly one of the level left and the new level is in the current gmap's
   level list. The client multiplies the two list positions (-1 for "not in
   the list") and saves on a negative product, so a level at position 0 counts
   like "not in the list" (a quirk Seihai keeps), or
3. **2.05 only:** otherwise, the two levels are in different **name groups**.

The name group is the level's file name without its folder, lower-cased, up to
and including its last `§` or `$`; a name with neither is in the empty group,
so two plain names are always the same group. 2.1 test 2 and every later build
have no group test: only rules 1 and 2. (2.17.1, 2.22 and 2.31 were read; the
2.1 test builds are inferred, because the delimiters are not in their
binaries.) `DeathRules::entry_name_group` is the profile switch.

In practice, with plain level names the respawn point is the level and spot
of the login, or of the last time the player stepped into or out of a gmap.
If nothing was saved and the client is online, the player goes to x 30.5,
y 30.0 facing down in the current level. Offline with nothing saved, the
player stays where they are.

After a respawn the real client runs the same save routine once more, whether
or not the level changed (see `damage-respawn.md`, open). Seihai does not.

## Not done, and why

- **Same-name-group rule** is done (above). Open: the order and timing of the
  gmap list against the level change on arrival (Seihai uses the gmap it holds
  when `note_entry` runs), the list's order and de-duplication (Seihai takes
  the named cells in file order), and the end-of-respawn save.
- **Offline default start level.** The notes name `tutorial_b-4.graal` but no
  position, so it is not used.
- **Script warps offline** do not update the saved entry point (that code is in
  the script-warp work); online they do, through the normal level arrival.
- **Dead gani** is done: while dead the player is drawn with `dead.gani`, one
  step per 50 ms tick of the death timer, held on its last step (`pose.rs`).
  Seihai's default set has its own `dead.gani` (our art): 6 steps of the hurt
  pose, 8 kneeling, then lying on the back, held. The steps and their timing
  are Seihai's choice. The sprite prop on the wire stays idle plus direction, as the notes
  say. Other players' dead pose (the dead bit in their STATUS) is not drawn
  yet.
- **CURPOWER on damage** is done (see "Hearts report" below).
- **Order of the respawn frame** (level warp then status clear) is a guess;
  the notes do not say.
- **Hurt state** (mode 3 for 40 ticks) is now modelled for hits from other
  players (`documentation/combat.md`). Script `hurt` and `damage_half` are
  still not gated by it, the `playerhurt` script event is not done (knockback is, see
  `damage-respawn.md`).
- **Damage amounts** are resolved for sword hits (`combat.md`, "Damage").
  `damage_half` still takes half hearts from its caller; script `hurt` is not
  yet routed through the hit rules.
- **Editor/pause exemption.** A player in the editor or paused state cannot be
  hurt unless the level is a sparring zone (an NPC script command
  `sparringzone` sets it). Seihai pauses with P or when its window loses the
  focus (`offline/pause.rs`) and then refuses every hit; it has no editor
  state and no sparring zones.
- No lab run. A capture of a real client dying (claim, status frames, respawn
  frame) would confirm the layouts and the order.
