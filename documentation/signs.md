# The sign panel

What the client shows while the player reads a sign, and how Seihai draws it. The code is `client/src/render/signs.rs`, the keys are in `bin/main.rs`, and `examples/sign_probe.rs` renders a level's first sign to PNG files, one a page.

Sources: the 2.05 client's sign drawing and its key handling while reading (read from the program, nothing of it kept here), and the `letters.png` sheets of 1.39 and 2.22.

## The sheet

`letters.png` is 256×224: 16 glyphs a row, each 16 wide and 32 tall, so glyph `g` is at (`g % 16 × 16`, `g / 16 × 32`). It carries the font, the button pictures and the pieces of the scroll frame. The client loads it once, at start.

Glyph order: a character's glyph is its place in

```
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!?-.,#>()#####"####':/~&### <####;
```

(then a newline). The `#` places are glyphs with no plain character. Each glyph has its own advance, the client's table of half widths doubled (I is 6 pixels, space 8, most letters 12).

| glyphs | use |
|---|---|
| 96, 97, 98, 99 | frame corners: top-left, top-right, bottom-left, bottom-right |
| 100, 101 | top and bottom edge |
| 102, 103 | left and right edge (32 pixels tall) |
| 91 to 94 | the A, B, X and Y buttons |
| 77 to 80 | the up, down, left and right pictures |
| 71 to 75, 87, 88, 67 | the other button pictures |

The pixels at (252..255, 223) are colours, not art. The client fills the inside of the frame with (254, 223) and, 8 pixels further in, (255, 223), and skips a fill whose colour is the sheet's transparent one. On the 2.22 sheet both are the same cream; on 1.39 the inner one is white.

## The panel

- 24 cells wide and 9 tall (16-pixel cells), centred across the view.
- On a 640-wide view it sits at y 256, or at y 128 when the player is more than 14 tiles down the view. At 800: 18 tiles, y 320 or 160. At 1024: 22 tiles, y 448 or 192. Which two values the client compares is inferred from the code's shape (the player's row against the view's top); the thresholds and positions are as read.
- The frame: the top and bottom edges every 16 pixels, the sides every 32 pixels, the corners over them, then the two fills from (x + 16, y + 24) to the right and bottom pieces.
- Text: three lines a page, from (x + 16, y + 16), 32 pixels apart. The sign's text is split at newlines.
- In a line, `#` and a letter draws button pictures: `#A`, `#B`, `#X`, `#Y`, `#u`, `#d`, `#l`, `#r`, `#h`, `#x`, `#y`, `#z`, `#4`, `#.`. `#` and any other letter draws glyph 86 and moves 16 pixels. `#i` and a character puts up an image from the level's list (not done in Seihai).
- A character outside the glyph order is drawn by the client in its system font, in the colour at (253, 223). Seihai leaves it out.

## Opening a sign

The 2.05 client opens a sign by walking, not with the A key. In its walk step, and in the per-tick check it runs when the facing changes or the player pushes against something, it tests, while the player faces up:

- the point (x + 1.5, y + 0.5), with (x, y) the player's position in tiles,
- lies in a sign's box: from (sign x, sign y) to (sign x + 2, sign y + 1), edges included,
- and is blocked (a wall tile, an object, an NPC or a chest).

Then reading starts on the first page, and the client plays a sound. The A key has no sign case. So a player walking up into a sign, or pushing up against it while standing still, reads it; one walking into it from the side does not.

Seihai does this in `OfflineSession::read_sign_ahead`, called from every walk attempt (`try_move`), whether the player moved or was stopped. "Blocked" there is a wall tile or a chest; NPCs and objects are not tested yet. After Esc, holding Up opens the sign again on the next step, which is what the rule gives; whether the client waits for the key to be released was not read.

## Keys while reading

While a sign is open the game keys work the sign and nothing else: the player does not walk, swing, lift, use an item or open the item list.

- The map key (M) ends reading at once.
- Up turns back three lines, when not on the first page.
- Every other game key (Left, Down, Right, D, S, A, the item list key and pause) turns to the next three lines, unless the page on show is not full (the sign has fewer lines than the page's first line plus three); then reading ends. A sign of exactly three lines therefore shows an empty second page before it closes, as in the client.
- A key counts only when no game key was down the frame before: the arrow that walked into the sign does nothing until it is let go, and keys do not repeat.
- Esc does not close the sign. As in 2.05 (`TAdventure::FormKeyDown`, which has no sign case), it asks whether to quit; answering no leaves the sign open. Before 2026-10-31 Seihai closed the sign on Esc, so a player pushing up into a sign had it open again at once and Esc never got to the quit prompt.
- Seihai reads "pressed" and "was down last frame" from the same source (`SignKeys`, down or pressed this frame). minifb reports a key down one frame before it reports it pressed, and taking one from each made every press look held: A, S and D did nothing while a sign was open (fixed 2026-10-31, tests in `render/signs.rs`).
- After reading ends, the game keys stay off the player until all are let go, so the S that closes a sign does not swing a frame later.
- The lock is in the session too, not only in the key loop: while `reading` is set, `try_move` takes no step and no turn, `start_swing` refuses, `interact` does nothing (no lift, throw, grab or chest), and `toggle_hide` does nothing. A while hidden does not bring the player out either. Tests: `an_open_sign_takes_the_walk_the_a_key_and_hiding`, `no_swing_while_a_sign_is_open`.
- The client plays a sound when the page turns and runs `playerendsreading` when reading ends. Neither is done in Seihai yet.

## Confirmed, inferred, not tested

- Confirmed from the program: the glyph grid, glyph order and advances, the frame pieces and fill colours, the panel size, the line layout, the `#` codes, the paging rule, and the walk-on opening rule (point, box, facing and the blocked test).
- Inferred: what the panel's height test compares, that key 0 is Up, and that facing 0 is up for the opening rule.
- Not tested against a running client: the opening rule is from the program only. Not tested against a real client screenshot. The probe's render on the 2.22 sheet is at `seihai-lab/screens/sign-panel-222.png` in the project files.
