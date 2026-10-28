# Weapon images and the selected weapon

The status display draws the selected weapon's image next to the D button
(see `hud.md`). This page covers which weapons are in the list, which one is
selected, and which image each one has. The code is in
`client/src/weaponbar.rs`, and `main.rs` wires it in.

## Built-in weapons (`PLO_DEFAULTWEAPON`, 43)

The 2.05 rev 2 client has a table of 25 item names. Each entry has a weapon
script and an image. Only the seven ids that have a script become weapons.
The weapon's name is the item name with its first letter in upper case.

| id | item | weapon name | image (2.05) | image the 2.22 client asked for in the lab |
|---|---|---|---|---|
| 7 | bow | Bow | `wbowi1.gif` | `wbowi1.png` |
| 8 | bomb | Bomb | `wbomb1.gif` | `wbomb1.png` |
| 12 | superbomb | Superbomb | `wbomb2.gif` | `wbomb2.png` |
| 20 | fireball | Fireball | `wbowi2.gif` | `wbowi2.png` |
| 21 | fireblast | Fireblast | `wbowi3.gif` | `wbowi3.png` |
| 22 | nukeshot | Nukeshot | `wbowi4.gif` | `wbowi4.png` |
| 23 | joltbomb | Joltbomb | `wbomb3.gif` | `wbomb3.png` |

* **Other ids.** Ids 0 to 6, 9 to 11, 13 to 19 and 24 are pickups (rupees,
  bombs, darts, hearts, gloves, swords, shields, spin attack) and are not
  weapons.
* **Extension.** Seihai uses the build's image extension (`files.image_ext`):
  `.gif` before 2.1 and `.png` from 2.1 on.
* **Id 8.** The binary ties `wbomb1` to id 8. The lab capture could not show
  this, because the 2.22 client asked for `wbomb1.png` whether or not id 8
  was sent. `protocol::login::default_weapon_image` (the login image list)
  still leaves id 8 out. The login burst is not part of this change.

## The list and the selection (2.05 rev 2)

* **One list.** Server weapons (`PLO_NPCWEAPONADD`, 33) and built-in weapons
  share one list, in the order they arrive. A built-in weapon is skipped when
  a weapon of that name is already in the list.
* **Adding.** Each add selects the new last entry.
* **Selecting index `i`.** The client takes the first weapon from `i` on,
  wrapping round, whose name is not empty and does not start with `-`. Names
  that start with `-` are the hidden system weapons. If every weapon is
  hidden, `i` stays selected.
* **Deleting (`PLO_NPCWEAPONDEL`, 34).** The selection stays at its index if
  that index still exists. Otherwise it moves to the new last weapon.
* **What is drawn.** The status display draws the selected weapon's whole
  image at x 170, vertically centred on y 45.

Seihai rebuilds the list every frame from `SessionState::default_weapons`
and `SessionState::weapons`, so weapons that arrive after login are covered.

* Within one frame, built-in weapons are added before server weapons. The
  real client adds them in packet order. This only matters when both kinds
  arrive in the same frame.
* The image of the selected weapon is requested once through the file cache
  (`WANTFILE` or `UPDATEFILE`, by the build's rules). It is drawn as soon as
  it is in the cache or in the asset folder.

## Checked

* **Unit tests** (`weaponbar.rs`):
  * the latest weapon is selected, including one that arrives after login;
  * hidden weapons are skipped, wrapping round;
  * the selection after a delete;
  * no duplicate built-in weapon.
* **Lab, 2026-10-28:** Seihai as 2.22 against the lab GServer-v2 with the
  2.22 asset folder. The D slot shows the bomb (`wbomb1.png`), as the real
  2.22 client does in `client222-onlinestartlocal.png`. Seihai's frame is
  `seihai-lab/screens/seihai-hud-222-lab-online.png`.

## Open

* **Switching weapons.** The selection changes only when weapons are added
  or deleted. The client's weapon-select mode (D steps to the next weapon)
  and the Q screen are the inventory item, which is not in this change.
* **GIF images.** Seihai's decoder reads PNG only, so a `.gif` weapon image
  is drawn only if the resolver finds a `.png` under the same name.
