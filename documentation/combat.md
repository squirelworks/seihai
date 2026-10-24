# Combat

What Seihai does when another player hits it, when it swings its own
sword, and what it leaves open. Sources, all derived (no lab capture of a
real fight exists yet): `decomp-findings-2026-10-24/04-sword-hitbox.md`,
`01-death-respawn.md`, `constants-205.tsv`, `status-bits-prop18.tsv`,
`player-modes.tsv` and `sprite-step-table.tsv` (2.05 rev 2), and Graal_analysis
`protocol/packet-layouts.md` "Hits" and `02-weapons-combat.md` (ids and
layouts of 26/40 and 36/46).

This PR is stacked on #27 (health and death, `chiaki/health-death`): a hit
that takes the last heart starts that death, and the claim names the attacker.

## Where it lives

- `client/src/combat.rs`: the rules (`judge_sword`, `judge_hurtplayer`,
  `attacker_may_send`), the decoders for ids 40 and 46, the swing and hurt
  state (`CombatState`) and the session methods (`receive_all`,
  `receive_sword`, `receive_hurtplayer`, `start_swing`, `tick_combat`).
- `VersionProfile.life.combat` (`CombatRules`, `COMBAT_205` in `profile.rs`):
  the numbers. 2.05 and every later build have the row (later builds are an
  inference, as for death); 1.33 to 2.04 have `None`. With `None` nothing is
  decoded, judged or sent, and ids 40 and 46 count as unhandled.
- `SessionState` notes each other player's SPRITE, STATUS, ALIGNMENT and
  SWORDPOWER (`OtherPlayer.sprite/status/ap/sword_power`), lists players
  whose props showed a swing (`swings`), and decodes ids 40 and 46 (`hits`).
- `bin/main.rs`: judges what arrived each frame, runs the swing and hurt
  clock, and sends the sprite.

## Who decides

The victim's client. A sword swing sends no "I hit you" message: the
attacker broadcasts its position, direction and sprite (`dir + 4 * S`), and
every other client tests whether it is in the box in front of the swing.
Seihai does the same, on the receiving side, whenever another player's props
arrive with a swing sprite.

A sword hit counts when all of these hold (`judge_sword`):

| Check | Value | Reject |
|---|---|---|
| attacker's sprite S | 6 to 10 (animation steps 9 to 13) | `NotSwinging` |
| victim hurtable | not dead (or hearts 0), not in hurt mode, not swimming | `Dead`, `AlreadyHurt`, `Swimming` |
| victim's AP | not exactly 100 | `Immune` |
| ahead of the attacker, along its facing | 0 to 4 tiles, both ends count | `OutOfReach` |
| to the side | strictly under 2 tiles | `OffLine` |

The facing is the one in the sprite's low two bits, which is what was sent.
Distances are measured between the two players' X/Y props, so any fixed
offset inside the sprite cancels. On a gmap the other player is placed
relative to our cell first (`OtherPlayer::position_from`).

A hit takes hearts through `damage_half` (so a killing hit names the attacker
for `PLI_CLAIMPKER`, see `health-death.md`) and starts **hurt mode**: 40 ticks
(2 s) during which nothing can hurt the player again. While hurt, the sprite
Seihai sends has S = 32. Hurt mode does not lock the controls (the notes say
nothing about that).

## Packets

- `PLO_HURTPLAYER` (40): `GSHORT attacker, GCHAR dx, GCHAR dy, GCHAR power`
  and an optional `GINT3 npc` when 6 or more bytes follow the id. Knockback is
  `(byte - 0x60) / 16` tiles, power `byte - 32` half hearts. It applies when
  the victim can be hurt and the attacker is within 10 tiles (distance in a
  straight line); power is capped at one heart. AP is **not** checked: the
  notes make AP 100 immune to the sword only.
- `PLO_HITOBJECTS` (46): `GSHORT attacker, GCHAR power*2, GCHAR x*2,
  GCHAR y*2` and an optional `GINT3 npc`. Decoded into `hits`, **not applied**
  (below).
- `PLI_HURTPLAYER` (26), the attacker's form: `GSHORT victim, GCHAR (dx+4)*16,
  GCHAR (dy+4)*16, GCHAR power`. `combat::hurtplayer_message` builds it and
  refuses (`Result::Err`) when the victim would: in the profile's send set,
  the victim's STATUS has the dead bit, its sprite shows hurt (S = 32), or it
  is more than 10 tiles away. The power is capped at one heart.

## The attacker's side

- **Sword.** The attacker sends nothing that a victim could refuse: the swing
  is a sprite broadcast. The S key starts a swing (`start_swing`: not dead,
  not swimming, a sword, no warp pending, no swing already out). Others see
  S = 6, 7, 8, 9, 10 in the walk message, then idle. Walking during a swing
  sends the swing sprite, so it is not cut short.
- **Hits that are messages.** `hurtplayer_message` is the filter for the
  messages that do name a victim. **No game-loop code calls it yet**: the
  only sources would be script commands (`hitplayer`, `hitobjects`), and
  their argument meanings are not in the notes, so they stay "not applied"
  effects as before. A bomb or arrow sender is likewise not built.

## Placeholders and open points

Nothing below is invented as if known; each is a labelled constant or a gap.

- **Damage per sword hit** (item 7, unresolved). The notes say "inferred:
  sword power divided by 2 in hearts, not confirmed". `CombatRules::
  sword_half_per_power` = 1 half heart per point of the attacker's sword power
  (at least 1) is that inference, as a placeholder. The attacker's power is its
  SWORDPOWER prop (8) minus 30; a player whose power was never sent cannot
  hurt (`UnknownPower`). Packet hits carry their own power and need no formula.
- **Swing frame times.** The real times come from the sword gani, which Seihai
  does not play. `swing_step_ticks` = 2 (100 ms per frame, 0.5 s a swing) is
  Seihai's own placeholder, for the swing it sends.
- **When a swing is judged.** The notes say the victim tests the hit, not how
  often. Seihai tests when a props update shows a swing sprite. A victim who
  walks into a swing without a new update from the attacker is not caught, and
  repeats within one swing are stopped by hurt mode.
- **Id 46 is not applied.** The notes say it hurts a player "whose centre is
  within 2 tiles of (x, y) on each axis". Where a player's centre is, relative
  to the X/Y prop, is not in them, so any choice would be a guess.
- **NPC-sourced hits** (40 with an NPC id) are refused: the NPC's position is
  needed for the 10-tile test and online NPC props are not decoded.
- **Knockback** (decoded) is not applied. The notes give no rule for how a
  knockback moves the player (distance, speed, collision).
- **Explosions** (`PLO_EXPLOSION` 36, within 1 tile on both axes) and arrows,
  and the `playerhurt` script event, are not done.
- **Preconditions of a swing** the notes list and Seihai skips: the level or
  server allowing weapons (STATUS bit 0x10), the charged spin (mode 13), and
  riding. Seihai has no horse, and the weapons bit's wire meaning is not
  traced further.
- **Swim and editor/pause.** Swimming blocks hits (mode 10). The editor/pause
  exemption has no Seihai equivalent.
- **Distance rule.** "Within 10 tiles" is taken as a straight-line distance.
  Graal_analysis words 46's rule per axis and 40's without saying; a per-axis
  test would accept more hits near the corners.
- **Lab.** No run against the real client or a server; the layouts and rules
  are the documents' and the decompile's.
