# Combat

What Seihai does when another player hits it, when it swings its own
sword, and what it leaves open. Sources, all derived (no lab capture of a
real fight exists yet): `decomp-findings-2026-10-24/04-sword-hitbox.md`,
`01-death-respawn.md`, `constants-205.tsv`, `status-bits-prop18.tsv`,
`player-modes.tsv` and `sprite-step-table.tsv` (2.05 rev 2), and Graal_analysis
`protocol/packet-layouts.md` "Hits" and `02-weapons-combat.md` (ids and
layouts of 26/40 and 36/46).

A hit that takes the last heart starts the death of `health-death.md`, and
the claim names the attacker. The damage formula and the extra hit gates were
derived in item 7; `damage-respawn.md` has what is known and open.

## Where it lives

- `client/src/combat.rs`: the rules (`judge_sword`, `judge_hurtplayer`,
  `attacker_may_send`), the decoders for ids 40 and 46, the swing and hurt
  state (`CombatState`) and the session methods (`receive_all`,
  `receive_sword`, `receive_hurtplayer`, `start_swing`, `tick_combat`).
- `VersionProfile.life.combat` (`CombatRules`, `COMBAT_205` in `profile.rs`):
  the numbers. 2.00 to 2.05 have a row read in their own binaries (see
  "2.00 to 2.04" below); later builds take the 2.05 row as an inference, as for
  death. 1.33 to 1.42 have `None`: with `None` nothing is decoded, judged or
  sent, and ids 40 and 46 count as unhandled.
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

### Who reports a sword hit, by version

| Build | The swing on the wire | Who judges the hit | Hit message for a sword |
|---|---|---|---|
| 2.05 rev 2 | prop 10 (the pre-2.1 bow slot) = `c(10 + len)` + `sword`, SPRITE = direction only (lab); the decompile's victim test reads SPRITE S 6 to 10 | the victim's client | none |
| 2.17.1, 2.22, 2.31 | GANI (10) = `sword` (`ridesword`, `spin`), SPRITE = direction only | the victim's client | none |

The victim's side was read in all four builds: the sword's damage comes from
the attacker's SWORDPOWER prop as the victim last received it, and the swing
reaches others only as props (`decomp-findings-2026-10-24-d/01-sword-damage.md`).
No read build sends `PLI_HURTPLAYER` (26) for a sword: in 2.05 the attacker's
26 and the UDP id 21 come from the GS1 `hitplayer` command
(`protocol/udp-p2p.md`), so `hurtplayer_message` stays unwired for the sword.
The server only relays the props; GServer-v2 checks nothing about a hit. The
one server-made hit is the spin: on GANI `spin` GServer-v2 sends four
`PLO_HITOBJECTS` (46) around the player (`02-weapons-combat.md`).

**2.05 swings on the wire.** The real 2.05 client in the 2026-10-30
cross-laptop lab (its own client-to-server decode) never sent a swing
sprite: SPRITE stayed on the direction (128 + direction while hurt), and each
swing went out as prop 10 in the bow form with the animation's name, `sword`,
repeated about every 0.4 s with no position (`walk` and `hurt` the same
way). GServer-v2 stores that as a bow image, adds `.gif`, and relays it in the
same form. Seihai reads the name from that slot (`OtherLooks::bow_slot_gani`)
and judges a `sword` there like the 2.1+ GANI below, from the player's last
X, Y and direction. The 2.05 victim test read in the decompile (S 6 to 10) is
kept: real 2.05 clients took Seihai's sprite swings in the same run. A
GServer-v2 quirk to know: it encodes prop 10 by the *sender's* version, so a
2.1+ receiver gets a 2.05 player's bow form and a 2.05 receiver gets a 2.1+
player's GANI string. Seihai reads prop 10 by its own profile, so a 2.1+
Seihai misreads a 2.05 player's prop 10 on GServer-v2 (open; how the real
2.1+ clients cope was not checked).

**On a 2.x-generation server.** GServer-v2 beta4 (the cross-laptop lab
server) reads and writes prop 10 as a GANI string whenever its generation is
not classic, for every client: a 2.00 or 2.05 swing reaches a 2.0x login as
`c(5)` + `sword`, not in the bow form. Seihai's pre-2.1 profiles read that
form too (`props::bow_slot_holds_gani`, which says how the two are told
apart). Before this, a 2.05-profile watcher read it as bow power 5 and lost
the swing and the rest of the update, so 2.00's swings left no "hit by" line
at all.

**2.1+ swings.** A GANI client sends the swing as GANI `sword` once, and its
SPRITE holds only the direction. Seihai takes another player as swinging when
its GANI is `sword`, `ridesword` or `spin` (`combat::is_swing_gani`), and
judges it again on every later update of its place or animation while that
GANI stays on (the 2.05 sprite came with every step). These three are the
names the 2.05 client turns into S 6 to 10 (its gani-to-step rule,
`feature-specs/11-emotes-ganis.md`). That 2.17.1 and later judge a swing by
these names is an **inference**: their kick test reads the attacker's
animation (`kick`, index 3 of the 31 built-in names) and not a sprite, but
their swing test itself was not read. A server that renames the ganis (the
2.22 run on Graal the Adventure plays `cn_sword`) is not covered. Seihai's own
2.17, 2.22 and 3.04 swings already go out as GANI `sword` on the walk cadence
(`protocol::walkcadence`).

**Peer UDP.** Swings do not need the UDP channel. The 2.05 channel carries
the movement (id 0, with the first letter of the GANI, `S` while swinging),
but a client sends a level action over TCP as well whenever some player in the
level has no UDP port (`feature-specs/07-udp-peer.md`, "Routing rule"), and
Seihai advertises none (no UDPPORT, 31). Props always go over TCP too. So
Seihai sees every swing as props from the server. Whether 2.17 to 2.31 use
the channel at all is open (2.22 ships `noudp=true`; the 2.17 client in the
2026-10-30 cross-laptop run had no UDP socket).

A sword hit counts when all of these hold (`judge_sword`):

| Check | Value | Reject |
|---|---|---|
| attacker's sprite S | 6 to 10 (animation steps 9 to 13) | `NotSwinging` |
| victim hurtable | not dead (or hearts 0), not in hurt mode, not swimming | `Dead`, `AlreadyHurt`, `Swimming` |
| victim's AP | not exactly 100 | `Immune` |
| level | the level has not run the script command `noplayerkilling` | `NoPkZone` |
| attacker judged | not already judged in this second of the day (once per attacker per second) | `Throttled` |
| guild | the attacker's `(tag)` is not in the victim's nickname | `Friendly` |
| attacker's STATUS | bits 0x02 (hidden) and 0x20 clear | `AttackerBlocked` |
| ahead of the attacker, along its facing | 0 to 4 tiles, both ends count | `OutOfReach` |
| to the side | strictly under 2 tiles | `OffLine` |

The facing is the one in the sprite's low two bits, which is what was sent.
Distances are measured between the two players' X/Y props, so any fixed
offset inside the sprite cancels. On a gmap the other player is placed
relative to our cell first (`OtherPlayer::position_from`).

### Damage

The hit takes the **attacker's sword power in half hearts** (power 3 is 1.5
hearts), the power limited to -20..20. The power comes from the attacker's
SWORDPOWER prop (8): a byte of 0 to 9 is the power itself (the default swords
1 to 4 send their number), from 10 it is power + 30 followed by the image
name. A **negative power heals**, but only a victim with **AP 40 or more**;
below that the hit changes no hearts. A power of 0 changes nothing either. In
all three cases hurt mode still starts. Healing stops at the maximum hearts.
The profile holds the numbers (`sword_power_limit`, `heal_min_ap`).

Shield, item levels and the attacker's AP are not in the formula: the victim's
client reads none of them for a sword hit (checked in 2.05, 2.17.1, 2.22 and
2.31; the rule is the same in all four).

A hit takes hearts through `damage_half` (so a killing hit names the attacker
for `PLI_CLAIMPKER`, see `health-death.md`) and starts **hurt mode**: 40 ticks
(2 s) during which nothing can hurt the player again. While hurt, the sprite
Seihai sends has S = 32. Hurt mode does not lock the controls (the notes say
nothing about that).

## What is drawn (item 22)

The swing and hurt mode are shown with the player's own animations: the S key plays `sword.gani` step by step in time with the swing (its sound after step 0 is queued at the start), and an accepted hit shows `hurt.gani` by its `WAIT` (the stock file and Seihai's default: one step of `WAIT 30`, 31 ticks, as the 2.05 client was seen to show it live). Both follow `CombatState`, so they also run in the offline session. See `default-assets.md`, "Poses".

## Packets

- `PLO_HURTPLAYER` (40): `GSHORT attacker, GCHAR dx, GCHAR dy, GCHAR power`
  and an optional `GINT3 npc` when 6 or more bytes follow the id. Knockback is
  `(byte - 0x60) / 16` tiles, power `byte - 32` half hearts. It applies when
  the victim can be hurt, the attacker is a known player, AP is not 100, the
  level is not `noplayerkilling`, the once-a-second stamp and the guild test
  pass (the same ones as the sword), and the attacker is within 10 tiles
  (squared distance of the X/Y props at most 100); power is capped at one heart
  and 0 is accepted. There is no attacker STATUS test. The knockback is
  `(dx, dy)` (`damage-respawn.md`, "Packet path").
- `PLO_HITOBJECTS` (46): `GSHORT attacker, GCHAR power*2, GCHAR x*2,
  GCHAR y*2` and an optional `GINT3 npc`. Applied: the hit point is
  `(b1/2, b2/2)`, the victim's centre is `(X+1.5, Y+2.0)`, a hit lands when the
  box is strictly under 2 tiles on each axis; same gates and cap as id 40; the
  knockback is centre minus point.
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
  sends the swing sprite, so it is not cut short. A 2.17, 2.22 or 3.04
  session sends GANI `sword` and the direction instead (see "Who reports a
  sword hit, by version"). A 2.00 to 2.05 session sends what the real 2.00
  and 2.05 sent in the lab: prop 10 in the bow form, `c(10 + 5)` + `sword`,
  as the swing starts (`spin` for the spin), `idle` when it ends, and SPRITE
  left on the direction (`combat::swing_in_bow_slot`, `net::bow_slot_message`).
  A real 2.00 took no damage from the sprite swing; 2.01 to 2.04 are taken to
  send the same as their neighbours (not captured).
- **Hits that are messages.** `hurtplayer_message` is the filter for the
  messages that do name a victim. **No game-loop code calls it yet**: the
  only sources would be script commands (`hitplayer`, `hitobjects`), and
  their argument meanings are not in the notes, so they stay "not applied"
  effects as before. Bombs and arrows are sent by the built-in weapons
  (`bombs.rs`, `arrows.rs`).

## Bombs, explosions and shots

Other players' bombs (`PLO_BOMBADD` 11, `PLO_BOMBDEL` 12), explosions
(`PLO_EXPLOSION` 36) and shots (`PLO_ARROWADD` 19) are read into
`SessionState.level_objects`. The game loop moves them into our gmap cell by the
sender's cell and hands them to the session. The rules come from the 2.05 rev 2
program (Graal_analysis `client-internals/damage-sources.md`, "Other players'
bombs, blasts and shots"):

- **A received bomb** goes through `putbomb` without a send, so it splashes in
  water, and it burns the fuse it carries. A received explosion is one cross of
  its reach and type, power read as 1 to 3. `PLO_BOMBDEL` removes the bomb at
  exactly its point.
- **A received shot** flies a tile a tick like our own (sprite and type as
  sent). It hits once its sprite's box (`w/32` by `h/32` tiles from the sprite
  table) overlaps `x + 0.5 .. x + 2.5`, `y + 1 .. y + 3` of the player.
  - With no sprite table loaded, a shot never hits the player.
  - It takes half a heart, or a whole one for a fireball or nuke.
  - It is then gone, and a fireblast or nuke leaves a reach-2 blast of type 1.
  - Received on a `noplayerkilling` level, it never hits the player. Our own
    weapon's arrows never hit us either.
- **Owner.** With bit 8 (flag 8 for a shot), the sender owns every piece or
  the shot. The owner counts only while the sender is still in the player list
  (`OfflineSession.present_players`, refreshed each frame online).
  - A player's piece or shot does not hurt on a `noplayerkilling` level.
  - When its hit takes the last heart, the owner is the killer for
    `PLI_CLAIMPKER`.
  - A piece or shot of nobody's, or of a player who left, always hurts.

**Inferred or not done:**
- The explosion's sound names are taken to be a bomb's (inferred).
- The way a shot knocks the player is not legible in the decompile. Seihai
  pushes along the flight, 5 ticks of 1 tile. A real 2.00 hit by Seihai's
  arrow flying west (lab, 2026-10-30) did the same: half a heart, GANI
  `hurt`, then about 5 tiles west over 5 ticks (4 on the second hit, when
  it ran into something), back to `idle` 1.55 s later. So for 2.00 it is
  lab-checked; 2.05 and later are taken as the same.
- A shield stopping or sending back a shot is not done.
- Flag 4 (remove a shot flying the other way) is not done.
- Whether arrows stop at walls is still open. Seihai's arrows do not stop.

**Lab (2026-10-30, real 2.05 against Seihai main before this change):** the
real 2.05 client was killed by a blast. In one frame it sent `BOARDMODIFY`,
`PLI_CLAIMPKER`, CURPOWER 0 and GANI `dead`. This is the receiving side of the
owner rule (an owned piece names its killer), seen in the real client. Seihai's
own receive side, the sound names and the shot rules were not run in the lab.

## 2.00 to 2.04

Read in the 2.00, 2.01, 2.02, 2.03 and 2.04 binaries and compared with 2.05
rev 2 (Graal_analysis `client-internals/combat-200-204.md`). Every one has the
same swing frames, reach box, damage source (the attacker's SWORDPOWER in half
hearts), prop 8 wire form and 40-tick hurt mode. The rows:

| Build | Row | What differs from 2.05 |
|---|---|---|
| 2.03, 2.04 | `COMBAT_205` | nothing Seihai checks (2.03 keeps the no-kill flag on the player, and lacks 2.04's untraced per-player field test) |
| 2.01, 2.02 | `COMBAT_201` | `sword_power_store: Raw6Bit`: the power is kept as is in 6 bits, so it reads 0 to 63 and a negative one wraps (63 for -1) instead of healing. The wrap is inferred from the store's mask |
| 2.00 | `COMBAT_200` | as 2.01, plus `packet_victim_gates: false` (id 40 skips the AP 100 and no-kill tests) and `judge_throttle: false` (no once-a-second stamp anywhere); 2.00 has no id 46 |

Not modelled, because Seihai does not check them for any build: 2.00 has no
kick test and no `playerhurt` event; its sword judge needs a play state of
exactly 0 (no sparring-zone exception).

**No death row yet.** 2.00 to 2.04 still have `death: None`: their death
sequence was not traced. A hit that takes the last heart leaves the player at 0
hearts, refused by further hits (hearts 0 counts as dead), with no death, claim
or respawn.

**The once-a-second stamp.** The same read found the per-attacker stamp only
on the hurt packet path, in every build up to 2.05 rev 2; it was not found in
the sword judge. Seihai still applies `judge_throttle` to swords in 2.01 and
later (unchanged here; open).

## Placeholders and open points

Nothing below is invented as if known; each is a labelled constant or a gap.

- **Damage per sword hit** is resolved (above). Open inside it: the exact
  knockback of a sword hit, and which of the build's other hurt routines
  (bombs, arrows, baddies) use which power. `damage-respawn.md` lists them.
- **Guild test needs both nicknames.** `Friendly` applies only when Seihai
  knows the attacker's nickname (`OtherPlayer.nick`) and its own. The
  `(tag)` is the first bracket pair of the attacker's nickname with at least
  one character inside, matched case-sensitively anywhere in the victim's.
  The same test guards the packet path in the real client only where the
  decompile showed it; Seihai applies it to swings only.
- **The once-a-second limit** uses the second of the day (UTC) of Seihai's
  own clock, as the client uses the system clock. The client stamps an
  attacker as soon as the swing, the victim's state, AP and the level pass
  and before the guild and box tests; Seihai does the same.
- **Hit range for packets and a per-player "same level" field.** The packet
  path compares one more per-player field between attacker and victim that is
  not traced (it may be a level or area marker). Seihai does not check it.
- **`noplayerkilling`** is the script command that sets the level flag; the
  client also prints the message `nopkzone` when the player's own attack is
  refused by it. Seihai prints nothing. The flag ends when a level loads.
- **Kick animation.** The real client skips a sword hit when the attacker is
  in its kick animation (2.05, by gani name; 2.17.1 and later by an object id
  of 3). A kick cannot be seen from the props Seihai receives, so it is not
  checked.
- **Swing frame times.** The real times come from the sword gani, which Seihai
  does not play. `swing_step_ticks` = 2 (100 ms per frame, 0.5 s a swing) is
  Seihai's own placeholder, for the swing it sends.
- **When a swing is judged.** The notes say the victim tests the hit, not how
  often. Seihai tests when a props update shows a swing sprite. A victim who
  walks into a swing without a new update from the attacker is not caught, and
  repeats within one swing are stopped by hurt mode.
- **NPC-sourced hits** (40 with an NPC id) are refused: the NPC's position is
  needed for the 10-tile test and online NPC props are not decoded.
- **Knockback** is applied, one tile a tick for five ticks, each step undone
  when one of 12 body-box probes is blocked (`damage-respawn.md`). Tile
  `Blocking` and chests block; a walk-through (wallhack) session skips the
  probes. A push across a gmap cell edge crosses it like a walk step.
- **Other players' bombs, explosions and shots** are applied (see "Bombs,
  explosions and shots" below). The `playerhurt` script event is not done.
- **Preconditions of a swing** the notes list and Seihai skips: the level or
  server allowing weapons (STATUS bit 0x10), the charged spin (mode 13), and
  riding. Seihai has no horse, and the weapons bit's wire meaning is not
  traced further.
- **Swim and editor/pause.** Swimming blocks hits (mode 10). So does pause
  (STATUS 0x01: the P key, or a window without the focus; `offline/pause.rs`),
  refused as `Paused`. The real clients let a paused player be hurt in a
  `sparringzone` level; Seihai has no sparring zones. Seihai has no editor
  state.
- **Distance rule.** Settled by the decompile: the squared distance between the X/Y props is at most 100, for ids 40 and 46 alike (a circle, not per axis).
- **Lab.** A 2026-10-30 cross-laptop run of an older Seihai build against a
  real 2.17 client saw no sword damage either way (thrown objects worked);
  this file's 2.1+ swing reading is the fix for the receiving side, not yet
  re-run. Before that, no run against the real client or a server; the layouts and rules
  are the documents' and the decompile's (2.05 rev 2, 2.17.1, 2.22, 2.31).
