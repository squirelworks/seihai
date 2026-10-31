# Status display (HUD)

Seihai draws the status display from the selected version's own `state.png`,
following the rules of the 2.05 rev 2 client's status routine. If no
`state.png` is found, it falls back to the two plain bars it drew before.
The code is in `client/src/render/hud.rs`.

## Sources

* **Code:** the 2.05 rev 2 status routine, read from the shared decompile.
  The 2.05 rev 2 executable supplied the heart-row table and the
  small-heart stagger constant (17.5).
* **Sheets:** the shipped `state.png` of 1.33, 1.38, 1.39, 1.41, NewMain 2.4,
  2.05 rev 2 and 2.22.
  * The 2.22 sheet matches the 2.05 rev 2 sheet except for one 27×14 patch
    at (190, 60), which the routine does not use.
  * The 2.1 test 3 sheet matches the 2.22 sheet in the same way.
* **Picture:** the lab screenshot `client222-onlinestartlocal.png`. The
  comparison `seihai-lab/screens/hud-222-vs-seihai.png` shows the real 2.22
  client on top and Seihai below. Everything taken from the sheets lands on
  the same pixels. Only the text differs, because the client uses Windows
  fonts.

## The rules

All positions are measured from the view's top-left corner. The routine
names a base point `B = (15, 15)`.

| Part | `showstats` bit | Sheet rect (x, y, w, h) | Where |
|---|---|---|---|
| Rupee, bomb and arrow icons (one strip) | 0x02 | (80, 33, 120, 16) | (274, 30) |
| Rupee count | 0x04 | digit `d` at (14·d, 81, 14, 16) | 1 to 7 digits, no leading zeros, each 12 px apart, centred on x 280, at y 48 |
| Bomb count | 0x08 | as above | two digits at x 326 and 338 |
| Arrow count | 0x10 | as above | two digits at x 374 and 386 |
| Hearts | 0x20 | big (40·half, 0, 40, 44), small (80 + 31·half, 0, 31, 33) | see below |
| Buttons | 0x01 | (202, 0, 22, 33), the same cell for all three | x 15, 80 and 145 at y 30. Only the first is drawn when weapons are off |
| Key names | 0x01 | text | on each button: 26 pt Helvetica, white, with a black copy one pixel down and right. A small copy of the item-list key (key 9) sits above the D button at (147, 27) |
| Glove icon | 0x01 | sprite 145 (power ≥ 2), with sprite 325 drawn over it (power ≥ 3) | (40, 29). The power is printed at (60, 31) when it is ≥ 2 |
| Sword icon | 0x01 | sprite 17 of the player's sword image | (105, 31), drawn when the player has a sword image. The sword power is printed at (125, 31) |
| Selected weapon | 0x01 | the weapon's whole image | x 170, centred on y 45 |
| AP bar | 0x40 | frame (0, 97, 130, 22), fill (0, 119, AP, 10) | (15, 65), with the fill at +22, +6 |
| MP bar | 0x80 | frame (0, 49, 130, 22), fill (0, 71, MP, 10) | (15, 90). Drawn only when weapons are on and MP > 0 |

* **Hearts.**
  * One heart is drawn per container while power remains. Empty containers
    are not drawn.
  * Heart `i` is full when `power - 1 >= i` and half when `i < power`
    (power is counted in hearts).
  * Up to 4 hearts of power use big hearts, starting at (W − 198, 24),
    45 px apart (W is the view width).
  * Above that the small hearts are used, 35 px apart.
    * A new row starts at hearts 5, 11, 16, 22 and 27.
    * Row `k + 1` sits 20·(k + 1) px lower and moves `round(17.5·(k + 1)) − 35`
      px sideways.
    * The whole block is 10 px lower while power is below 6 hearts.
* **Weapons off.** The client's weapons flag also gates the S and D keys.
  When it is off, only the A button, its key name and the AP bar are drawn.
  Seihai has no "weapons off" state yet, so it always passes on.
* **`showstats`.**
  * Bits 0x100 and 0x200 belong to the player icons and to the weapon
    image list.
  * Seihai does not draw those parts in the status display.
  * A script's `showstats bits` sets the bits the HUD draws (2026-10-31;
    2gta's `splashscreen.nw` sends `showstats 0` to clear the screen for
    its title). Until a script sets them, every bit is drawn.
    `disablestats` does not reach the HUD yet.

## Older builds (inferred)

* **1.38 to 1.41.** The sheets have three different blank buttons at
  x 158, 180 and 202 on the sheet. Seihai detects them from the sheet (the
  first one is opaque at (169, 15)) and draws one per slot.
* **1.33 to 1.37.** The letters are part of the button art, so no key name
  is drawn. The digits are not on the sheet (it is 81 px high), so Seihai
  uses its own font for the counts.
* **AP bar.** A sheet without the AP rows (97 px high, before 1.41) gets no
  AP bar.
* **Positions.** The screen positions for these builds are the 2.05 ones.
  No routine from these builds was read.

## Open

* Seihai does not track a selected weapon, so nothing is drawn in the D
  slot yet. Default weapon images are item 14.
* The MP value is 0 in Seihai, so its bar never shows.
* The fonts: Seihai doubles its 5×7 font for the key names and uses it as
  is for the power numbers.
* The 2.22 and 2.31 status routines were not read. Their sheet and the 2.22
  picture match 2.05, so the 2.05 rules are used.
