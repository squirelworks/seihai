# A public GServer-RS run, 2.22 profile (2026-10-25)

What one login to a public GServer-RS taught us. Derived facts only: counts,
layouts and message names. The capture, the screenshots and the server's
address stay out of the repository.

## What the server sent

A full login burst (819 messages in one session; the viewer is
`seihai capture show FILE --version 2.22`). In order, the parts that matter
here: the own player props, the account's flags, the weapon list (one weapon,
`-gr_movement`, with its script), `SETACTIVELEVEL`, the level board
(`PLO_BOARDPACKET`, 64 x 64 tile ids), three `PLO_LEVELCHEST` records, no
`PLO_LEVELSIGN`, four `PLO_NPCPROPS` records (a player-like NPC and three
object NPCs), then the text and window messages.

## Why no chests were drawn

* The three chest records are `opened, x, y` only (three bytes each, flag 1).
  A chest the player has not opened carries two more bytes, item and sign
  index. So the server says all three chests are already opened for this
  account, at positions whose board tiles are plain grass.
* The board contains none of the chest tile ids of the 2.22 `arrays.dat`
  (closed `1784 1785 1800 1801`, opened `829 830 845 846`).
* **Correction.** This note first said the real client has no draw code for a
  chest record. It does: see `chest-drawing.md`. The record is drawn as a
  2 x 2 blit of the closed or opened chest tiles, so on the real client
  these three chests show (as opened chests).
* One of the object NPCs uses `chest.png`. That file was refused by the
  server (`FILESENDFAILED`) and server NPCs are not drawn by Seihai yet, so
  that image would be missing in either case.

Fixed in `chest-drawing.md`: Seihai now draws chest records the way the
client does.

What changed earlier: `Level::add_server_chests` and `add_server_signs` now put the
server's records on a level built from the server's board. Before, they were
stored raw and never used, so walking into an invisible chest was possible
and the chest could not be opened. Chests stay solid, as in the client.

## Script log lines

`script: object N: line L: expected `;`` now reads
`object N (weapon NAME): line L: ... [source line]` or
`object N (npc IMAGE at X,Y): line L: ... [source line]`.

The only script in the capture, the `-gr_movement` weapon, parses clean. Its
line breaks arrive as the two bytes `C2 A7` (the UTF-8 `§`); the weapon path
read them as Latin-1 text, so the whole script was one `//` comment line and
did nothing. Weapon scripts now read `§` (one byte or two) as a line break,
like level NPC scripts do. After that fix the weapon parses with no
diagnostics, so the `object 2: line 4` error comes from a script that is not
in this capture (a level NPC from a local level file, most likely). The new
log line names it.

## Capture reader

`tcpdump -i any` writes Linux cooked v2 (link type 276). The pcap reader now
accepts it (see `capture-viewer.md`).
