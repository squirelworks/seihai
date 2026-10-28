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
  numbers. Rows for 2.05 and every later build have it (later builds are an
  inference, as for cached links); 1.33 to 2.04 have `None` because their
  handler was not traced. With `None`, hearts at 0 do nothing and nothing is
  sent. `ClientVersion::sends_claim()` also needs id 14 in the build's send set.
- `OfflineSession`: `life`, `alignment` (AP), `entry` (the saved entry point),
  `damage_half`, `tick_life`, `respawn`, `note_entry` (the save rule), `no_pk`. The game loop in
  `bin/main.rs` ticks it, reads the keys and sends the messages.

## The sequence (tick = 50 ms)

| Tick | What happens |
|---|---|
| 0 | Hearts reached 0 (or below): the player is dead, faces down, the `playerdies` script event fires. The victim sends `PLI_CLAIMPKER` now, if a hit by another player did it. |
| 0 to 27 | Controls ignored (movement, interact). The timer counts up and stops at 30 (1.5 s). |
| 28 | The dead bit (`0x08`) goes out in `PLI_PLAYERPROPS` with `PLPROP_STATUS` (18). Controls work again. |
| after 27 | A direction key, the action key or the sword key (by default arrows, I/J/K/L, A, S or Space) respawns. |

A hit that leaves hearts at 0 starts the death once; a per-frame check keeps
the player dead while hearts are 0 or below. Hearts going back above 0 does not
revive: only a key press does. A player who is dead cannot be hurt (script
`hurt` and `damage_half` do nothing).

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
- **Dead gani.** The decompile says the local player plays the `dead` gani. The
  renderer draws no player gani yet, so nothing is drawn. The sprite prop on
  the wire stays idle plus direction, as the notes say.
- **CURPOWER on damage.** Whether the real client sends its own CURPOWER when
  hearts drop is not in the notes, so Seihai sends none. The server learns of
  the death from the dead bit.
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
  `sparringzone` sets it). Seihai has neither state.
- No lab run. A capture of a real client dying (claim, status frames, respawn
  frame) would confirm the layouts and the order.
