# Chest drawing

A chest is drawn from its record, not from the board. (An earlier note here
said the opposite; it was wrong.)

## What the 2.05 client does

Every chest (a `CHEST x y item sign` line of a level file, or a
`PLO_LEVELCHEST` record) becomes an object with a draw routine of its own.
The routine, read from the 2.05 client's code:

* skips the chest unless its x and y are both from 0 to 62, and unless a
  global flag is clear (what the flag is was not established; it is a plain
  byte that several drawing routines test);
* takes the tile sheet (`pics1.png`, 2048 x 512, the same image the board
  uses) and blits **32 x 32 pixels** from one of two places:
  * **closed**: source (896, 240), which is tile 1784 and the three tiles
    after it in the sheet's layout: `1784 1785 / 1800 1801`;
  * **open**: source (464, 304), tile 829: `829 830 / 845 846`;
* draws it at `(x - camera x) * 16`, `(y - camera y) * 16`, clipped to the
  view; and
* open is the chest's `opened` flag.

These are the entries of `arrays.dat` arrays 13 (closed) and 14 (open), so
2.22 has the same art. A chest also blocks walking as a 2 x 2 box, opened or
not (`walking-collision.md`), and the A key opens a closed one.

## The records

`PLO_LEVELCHEST`: byte 0 is the opened flag (any value but zero means opened),
then x and y (each plus 32). Only when not opened, item and sign index follow
(each plus 32). The short form is how the server says the account has already
opened that chest. Such a chest is drawn open.

A level file's chests are closed. Chests opened in play are tracked in the
session until the level is left.

Level authors leave a hole under a chest on the board: in the default start
level of a public server the board has tile 2047 where the chest goes.

## What Seihai does

`render::tiles::draw_chests` draws every chest record after the board and
before the NPCs:

* a sheet that is not the built-in one: tiles 1784 or 829 as above;
* the built-in sheet (own art, other ids): a chest already on the board under
  the record is left as it is (opened ones get the open tiles, id + 2),
  otherwise the built-in grass chest `64 65 / 80 81` (open `66 67 / 82 83`).

## Signs

A sign record has no draw routine (its object is a plain one). Signs are not
drawn by the client; what is seen is the board's art, or nothing. Seihai does
the same.

## Open

* The meaning of the global flag that skips chests.
* Draw order against players and NPCs (Seihai: chests under both).
* Chests of a neighbouring level on a gmap are not drawn.
* A level's NPC with a `chest.png` image is an ordinary NPC. The public server
  in `public-gserver-run.md` refused to send that file.
