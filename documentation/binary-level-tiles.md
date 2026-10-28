# Binary level tiles (`.graal`, `.zelda`)

A binary level decodes to 64 by 64 tile ids. Seihai stores the board as one full run per row, and every id is kept, tile 0 included.

## The black squares in offline play

Before this fix, the reader left tile 0 out. A row's leading 0s, and any 0 straight after another 0, had no tile at all, and the renderer drew those places black. Tile 0 is the top-left cell of `pics1.png`: grass on the 1.x sheet, on the 2.x sheet and on Seihai's built-in sheet.

`cargo run --example graal_tiles_probe -- pics1.png LEVEL.graal...` counts the places that hold no tile. It also lists the tile ids that land on an all-black cell of the sheet. Run over 1717 levels from five 1.x builds against a 2.x `pics1.png`, it gave these results:

- Before the fix: 79 levels had 129 places with no tile. After: none.
- The highest id used was 2545 (GR-V1.03), which is inside a 2048 by 512 sheet. No id falls outside the sheet.
- One id, 632, sits on a black cell of the 2.x sheet. It is used in 1146 levels. On the 1.x `pics1.gif` the same cell is dark grey (41, 41, 41), so it is meant to be dark and is not a reading error.

## Not checked

- Seihai does not read `pics1.gif`. A 1.x folder needs its sheet as `pics1.png`.
- Whether the 1.x and 2.x sheets differ in other cells. A level drawn with the other era's sheet can show the wrong picture without any black square.
