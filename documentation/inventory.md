# Item list (Q)

What the Q key opens, read from the 2.05 client (derived, no addresses or
code quoted), and what Seihai does. The real client's "inventory" is this
item list: a window of the player's weapons. Counts (rupees, bombs, arrows,
hearts) are on the HUD, not in it.

## The real client (2.05)

* **Key.** The game actions are, in order: up, left, down, right, item,
  attack, action, map, switchtochat, itemlist, pause. Q is the default key
  for `itemlist` (the 2.22 defaults; the 2.05 defaults are not listed in the
  notes, 1.37's fixed keys also used Q).
* **Opening.** Q opens the list from the normal state (or from another
  window) when a player flag allows it; Q again closes it. While it is open
  the player does not walk, swing or interact.
* **Entries.** One per weapon in the player's weapon list: the built-in
  weapons the server gave (`PLO_DEFAULTWEAPON`) and the named NPC weapons
  (`PLO_NPCWEAPONADD`). A name with a `/` puts the weapon in a folder: the
  folders of the current level are listed first as 32 x 32 folder cells.
  Names starting with `-` are never selected (hidden weapons).
* **Layout.** A framed window from (16, 80) to 16 pixels in from the right
  and bottom edges. The cells sit inside it, from 48 px in on the left,
  112 px from the top, 48 px in on the right and 64 px up from the bottom.
  A cell is as wide as the larger of its image and its label, plus 10 px,
  and as tall as image plus label plus 15 px; cells are packed into the
  first free place, left to right, top to bottom. The image is centred with
  5 px above it, the label centred under it. A weapon with no image shows a
  fixed 32 x 32 placeholder. The selected cell is filled with
  RGB(255, 212, 128). Scroll arrows show when cells are above or below the
  visible part. Font size 16.
* **Keys while open.** An arrow stretches the selected cell to the window's
  edge in that direction; among the cells that overlap the stretched box,
  the one whose near edge is closest is selected. Moving the selection
  selects that weapon at once (the item key will fire it). The item key or
  Enter on a folder opens it; the attack key on a folder goes back up. Any
  other game key (item, attack, action on a weapon) closes the list. Esc
  closes it. A mouse click on a cell selects it.
* **Packets.** Selecting sends nothing. Delete on a selected NPC weapon
  removes it from the list and, online, sends `PLI_NPCWEAPONDEL` (29) with
  the weapon name (`PLI_NPCDEL` (22) with the NPC id for other NPCs).

## Seihai (first slice)

`client/src/inventory.rs`, drawn and driven from `bin/main.rs`.

* Q toggles the list, offline and online. Esc, Q, D, S, A, Space and Enter
  close it. Arrows move the selection with the 2.05 nearest-cell rule.
  While it is open (and in the frame that closes it) the player does not
  walk, swing, interact or open the chat bar.
* Entries: built-in weapons by id (7 bow, 8 bomb, 12 superbomb, 20
  fireball, 21 fireblast, 22 nukeshot, 23 joltbomb; the names come from the
  item table), then the server's named weapons; hidden (`-`) weapons and
  repeats are left out. Offline: built-in weapons the player got from
  chests.
* Layout: the 2.05 window and content rectangles, and cells packed left to
  right and wrapped with the 2.05 cell sizes.
* Images: each cell draws its weapon's image, centred with 5 px above it,
  and the cell takes the image's size. A built-in weapon uses its table
  image (`wbowi1`, `wbomb1` and so on, with the build's extension, see
  `weapon-images.md`), a server weapon the image from `PLO_NPCWEAPONADD`.
  While the list is open each image not yet loaded is asked for once
  through the file cache (the same request the D slot makes) and drawn
  when it is in the cache, the asset folders or the built-in set; until
  then, and for a weapon with no image, the cell shows the 32 x 32
  placeholder outline. stderr: `requesting NAME (item list)`.
* Seihai's own addition: a counts line (rupees, bombs, arrows, hearts,
  sword, shield, glove) at the top of the window. The real window does not
  have it.
* stderr prints `item list opened (N entries)`, `item list: selected NAME`
  and `item list closed`.
* The pick becomes the selected weapon (the one beside the D button), as
  in 2.05. D with nothing carried fires it: the weapon's script gets
  `weaponfired` (stderr: `weapon fired: NAME`). Carrying, D still hides.

Checked in the lab: offline (`--level onlinestartlocal.nw`, no weapons) and
online as 2.22 against the lab GServer (bomb and bow listed, the hidden
weapon left out, the arrows select, A closes). Screens:
`seihai-lab/screens/item-list-offline.png` and
`item-list-online-lab-222.png` in the project files.

## Not done

* Folders, scroll arrows, mouse clicks.
* When the real client asks for the list's images (at login, on add, or
  on opening) was not read; Seihai asks when the list is opened.
* The built-in weapons (Bow, Bomb and the rest) have no script in Seihai
  yet, so D on them does nothing. In 2.00 and 2.05 each is a short GS1
  weapon script that fires only with ammo (`playerdarts>0`,
  `playerbombs>0`) and uses `shootarrow` / `putbomb`, which Seihai does
  not run yet (the weapon system is item 2 of `02-weapons-combat`).
* Delete with `PLI_NPCWEAPONDEL`.
* The flag that can forbid opening the list was not identified; Seihai
  always allows it.
* 2.22 and 2.31 were not read; the 2.05 rules are used for every build. The
  real client's window was not captured in the lab for a side-by-side.
