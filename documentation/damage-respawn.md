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

## Implemented

Profile-gated through `CombatRules` and `DeathRules` (`entry_name_group`), with
tests: the signed damage with the heal rule, `noplayerkilling` (set by the
script command, cleared when a level loads), the guild test, the status gate,
the once-a-second limit per attacker, the signed sword-power decode, the entry
save rule (`OfflineSession::note_entry`, `name_group`, `Gmap::list_index`).

## Open

- **A per-player field compared in the packet path.** `PLO_HURTPLAYER` is
  dropped unless a per-player field of the attacker equals the victim's. Its
  meaning is not traced (a level or area marker is likely). Not checked.
- **Status bit 0x20.** The sword gate needs it clear; the earlier notes only say
  "a numeric field is not negative". Seihai applies the mask as given.
- **Which hurt routines use which power.** The routine that hurts the player
  takes a power in hearts and a knockback vector. The sword path's exact
  knockback is not read off; bombs, arrows and baddies call it with their own
  powers that were not mapped. Knockback is not applied.
- **Kick animation exclusion.** A sword hit is skipped while the attacker is in
  its kick animation (2.05, by gani name) or has object id 3 (2.17.1 and
  later). It cannot be seen from the props, so it is not applied.
- **The guild test in the packet path.** The test is in the routine shared by
  swings and some packets; whether `PLO_HURTPLAYER` goes through it was not
  established. Seihai applies it to swings only.
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
