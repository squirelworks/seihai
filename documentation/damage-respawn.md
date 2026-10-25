# Damage and respawn: known and open

Item 7 of the overnight queue: the real sword damage formula and the respawn
"same name group" rule. Both come from reading the victim client's code in the
2.05 rev 2, 2.17.1, 2.22 and 2.31 builds (plus the 2.1 test builds where noted),
cross-checked against the earlier derived notes. Nothing here was confirmed
in a lab or against a server. The derived facts, without any addresses or code,
are in the Graal_analysis note `decomp-findings-2026-10-24-d`.

## Known

**Damage of a sword hit.** Half hearts = the attacker's sword power, limited to
-20..20. The same in 2.05, 2.17.1, 2.22 and 2.31.

- The wire byte of SWORDPOWER (8) is 0 to 4 for the default swords 1 to 4, and
  power + 30 (10 to 50) with the image name after it otherwise. The sender
  limits the power to -20..20.
- A negative power heals the victim if its AP is 40 or more; the victim's
  hearts stop at the maximum. Below AP 40 a healing hit changes no hearts.
- Hurt mode (mode 3, 40 ticks) starts for every accepted hit, including power 0
  and a healing hit.
- Shield power, item levels and the attacker's AP are not read.
- The routine that takes hearts away never lets a heal through under AP 40, and
  it is where hearts at 0 start the death.

**Extra gates a sword hit must pass** (beyond the reach box, AP 100 and the
victim's state of `combat.md`): the level has not run `noplayerkilling`; the
attacker was not already judged in the same second of the day; the attacker's
`(tag)` is not in the victim's nickname; the attacker's STATUS has neither the
hidden bit (0x02) nor bit 0x20. A packet `PLO_HURTPLAYER` (40) is also stopped
by `noplayerkilling`.

**Respawn destination.** The point saved by the last accepted level entry; the
save rules are in `health-death.md`, "Respawn destination". The name group is
2.05 only; 2.1 test 2 and everything later keep just the gmap-list rule.

## Knockback (settled, implemented)

Read in 2.05 rev 2, 2.17.1, 2.22 and 2.31; the shape is the same in all four
and the 12-point table is identical in all four, so it is one constant set for
every profile that has combat rules.

- The hurt routine takes hearts, a knockback vector (dx, dy) and starts hurt
  mode (mode 3, counter 40). The vector is scaled to length 1 tile; a zero
  vector means east.
- For the first 5 ticks of hurt mode (counter 39 down to 35, after the
  decrement) the player moves 1 tile along the vector each tick, then 12
  body-box probe points are tested; if any is blocked, that tick's step is
  undone (the push continues next tick from the old place). Total reach on open
  ground: 5 tiles in 250 ms.
- The probes lie on x = X + {0.6, 1.5, 2.4} and y = Y + {1.1, 1.7, 2.3, 2.9}
  (X, Y are the position props); the box is centred on (X+1.5, Y+2.0), the same
  centre that id 46 uses.
- Sources of the vector: a sword hit pushes from the attacker's position to the
  victim's; `PLO_HURTPLAYER` carries `(dx, dy)` in its bytes; `PLO_HITOBJECTS`
  uses the victim's centre minus the hit point.

## Packet path (corrected from the first notes)

Read in 2.05 rev 2 and 2.22. `PLO_HURTPLAYER` (40) and `PLO_HITOBJECTS` (46)
go through the same gates as the sword, in this order: attacker is a known
player; victim hurtable; AP is not 100; level not `noplayerkilling`; the
once-a-second stamp per attacker (the same table the sword uses, and stamped
before the guild test); the `(tag)` guild test; the per-player field test
(below); then for 40 the distance test (squared distance of X/Y at most 100, i.e.
10 tiles) and for 46 the box test (hit point is (b1/2, b2/2); the victim's
centre is (X+1.5, Y+2.0); strictly under 2 tiles on each axis). There is
**no** attacker STATUS test on the packet path. The power is capped at one
heart; a power of 0 is accepted (hurt mode and knockback, no hearts).

## Implemented

Profile-gated through `CombatRules` and `DeathRules` (`entry_name_group`), with
tests: the signed damage with the heal rule, `noplayerkilling` (set by the
script command, cleared when a level loads), the guild test, the status gate,
the once-a-second limit per attacker, the signed sword-power decode, the entry
save rule (`OfflineSession::note_entry`, `name_group`, `Gmap::list_index`),
knockback (`combat::KNOCKBACK_*`, `OfflineSession::knock_step`), the packet
gates of ids 40 and 46 (AP, throttle, guild, power 0) and id 46's box. The
knockback constants are in `combat.rs`, not in `CombatRules`, because they are
the same in every build read; this keeps `profiles_golden.txt` unchanged.

## Verdicts on the leftovers

| Item | Verdict |
|---|---|
| Knockback of a hit | implemented (above) |
| Per-player field compared in packet 40 | documented and skipped: it is a parent/owner link, zero for ordinary players, so the compare is always true for them. Inferred, not seen on a wire. Capture that would settle it: a real 2.22 client hit by a 40 whose attacker id is a player that owns something (a horse or similar); none is available offline |
| Guild test in the packet path | settled: it applies (implemented) |
| Id 46 | settled and implemented |
| Bomb, arrow, baddy powers | documented and skipped: the hurt routine's callers use 0.5 hearts by default and 1 for certain object types, but which object kind maps to bomb, arrow or baddy was not resolved, and Seihai has no bomb, arrow or baddy objects to send them. Capture: a real client hit by a server-side bomb and by an arrow, read the power byte |
| End-of-respawn save | needs capture: respawn the real client twice online (first with the server's level-warp reply delayed) and see whether the next respawn point is the death spot |
| CURPOWER on damage | needs capture: a real client at 12 half hearts hit by an injected id 40, trace for a PLI_PLAYERPROPS with CURPOWER |

## Open

- **The per-player field** (see the table): skipped, inferred to be always true
  for ordinary players.
- **Status bit 0x20.** The sword gate needs it clear; the earlier notes only say
  "a numeric field is not negative". Seihai applies the mask as given.
- **Kick animation exclusion.** A sword hit is skipped while the attacker is in
  its kick animation (2.05, by gani name) or has object id 3 (2.17.1 and
  later). It cannot be seen from the props, so it is not applied.
- **Gmap list order and timing.** The client indexes the current gmap's level
  list; Seihai takes the named cells in file order, case-insensitively, and
  uses the gmap it holds when the entry is noted. A real client may update its
  gmap before or after the entry; this decides whether entering a gmap from
  outside saves the entry.
- **The end-of-respawn save.** The respawn routine calls the entry save once
  more after it asks for the warp. Online, the level change may complete
  later, so the saved point could become the death spot. Not modelled; it
  needs a lab run.
- **2.00 to 2.04 and the 2.1 test builds.** The entry rule was read in 2.05 and
  2.17.1/2.22/2.31. Before 2.05 there are no death rules in the profile; 2.1
  test 2 and 3 are taken as having no group test because the delimiter set is
  not in their binaries.
- **Offline default start.** `tutorial_b-4.graal` is the default entry when
  nothing is saved offline, but the coordinates' axis order is not settled, so it
  is not used. A `sparringzone` NPC script command (which also lets a paused or
  editing player be hurt) saves the entry through the `playerenters` event path;
  not modelled.
- **Whether the real client sends CURPOWER on damage.** Unchanged: not known.
- **Lab.** None of this has been seen on a wire.
