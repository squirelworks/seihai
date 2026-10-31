# Other players' swords, shields and pictures

Seihai keeps these props of another player (`PLO_OTHERPLPROPS`) in `OtherPlayer::looks`:

| Prop | Kept as | Wire form |
|---|---|---|
| 8 SWORDPOWER | `sword` | `c(power + 30)` + image, or `c(power)` for 0 to 4 |
| 9 SHIELDPOWER | `shield` | `c(power + 10)` + image, or `c(power)` for 0 to 3 |
| 10 GANI | `gani` | a name (not the pre-2.1 bow form) |
| 11 HEADGIF | `head` | `c(len + 100)` + image, or `c(n)` for `headN.png` |
| 35 BODYIMG | `body` | a name |

A power with no image names the default picture (`sword2.png`, `shield1.png`). Power 0 is an empty name, and nothing is drawn for it. A later update without the prop keeps the old value.

## Drawing

Another player is drawn through a GANI when Seihai has the pose loaded:

0. A dead player: the STATUS prop (18) has the dead bit 0x08. It draws the last step of `dead.gani`, whatever prop 10 and S say. The 2.05 client sends no dead sprite; it sets the bit once its own death timer reaches 28 of 30 ticks, so its own death GANI is all but over by then. Showing the last step is inferred. The bit clears on respawn.
0. A hidden player: prop 10 is `hiddenstill` or `hidden`, or the STATUS hidden bit 0x02 is set and Seihai's build draws hiding as only the object (`HidePose::Hidden`, 2.05 and 2.22). It draws `hidden.gani` (prop 10 `hidden`) or `hiddenstill.gani`, which in the stock files draw only the shadow; without those files nothing of the figure is drawn. The CARRYSPRITE object is drawn where the player stands (`hide_origin`, `x + 0.5, y + 1`), as for the local player, not over the head. S cannot tell hiding from carrying: `hiddenstill` is S 18 and `hidden` walks 19, 20, 19, 18 (`offline/hide.rs`).
1. The GANI prop 10 names, when it is `idle`, `walk`, `sword`, `hurt`, `sit`, `carrystill`, `lift`, `carry` or `dead` (`dead` shows its last step). Its step comes from a 50 ms clock. `lift` without `lift.gani` draws `carrystill`, as the local player does.
2. Otherwise the pose the sprite number S (prop 17, `dir + 4 * S`) stands for: S 0 idle, 1 to 5 walk, 6 to 10 the sword steps, 18 carry still, 19 and 20 carry, 28 sit, 32 hurt. The walk frame is the inverse of the 2.05 step table (`WALK_S`). Any other S draws as idle.
3. If no pose is loaded, it is the plain idle figure as before.

**The hurt blink.** While another player's GANI is `hurt` or `ridehurt` (prop 10, or the bow slot of a pre-2.1 build, with or without the `.gif` GServer adds), every other 50 ms tick draws only the shadow, the `SPRITES` placements. The real 2.00 does exactly this (2.00 trace, 2026-10-30): the blink follows the GANI alone, with no timer, and the hurt sprite (S 32) never blinks another player. It stops when any other GANI arrives (`render::other_blink_off`).

The GANI's `SWORD` and `SHIELD` sprites take the player's own sword and shield pictures, and `HEAD` and `BODY` take their own head and body. A picture is looked for in the asset folders first (`swords/`, `shields/`, `heads/`, `bodies/`), then asked of the server. Until it arrives, the default picture is used: Seihai's head, body and `sword1.png`. No shield is drawn without a shield picture. The local player's shield is `shieldN.png` for its shield power.

`PoseSheets` gained `shield`, and `draw_gani_frame` draws `SHIELD` sprites. Seihai's own `idle`, `walk` and `sword` GANIs have `SHIELD` sprites at the stock places (`appearance.md`).

## What is confirmed and what is not

- Confirmed: the prop wire forms (`protocol/props.rs`, from the 2.05 tables) and the S table (Graal_analysis, 2.05 sprite step table).
- Seen live: a 2.22 client lifting a rock and hiding under it sends GANI `lift`, CARRYSPRITE, GANI `carrystill`, then GANI `hiddenstill` with STATUS 22 (bit 0x02 set), then `hidden` / `hiddenstill` as it walks.
- Not tested against a real client: whether 2.05 draws another player's walk from S or from its own clock, and how long it shows each step. Seihai's choice is above.
- Done since: other players' colours (prop 13, `appearance.md`) and GANI attributes 1 to 5 (props 37 to 41, the hat is 1; `appearance.md`).
