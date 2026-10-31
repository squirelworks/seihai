# Laying items (S held + A)

Hold the attack key (S) and press the action key (A) to pick an item, then
press A alone to put it down in front of the player. Code:
`crates/seihai-core/src/offline/laying.rs`, the keys in
`crates/seihai-mae/src/bin/main.rs`, the held item in
`render::draw_throw_layer`.

The 1.21 readme names it ("Laying Items: Select Item: hold Attack Button (S)
\+ press Action Button (A); Lay Item: Action Button (A)"). The rules below
were read in the **2.05 rev 2** binary **[bin]**. No real client was captured
laying an item yet.

## Per build

| Builds | Lay mode | What goes out |
|---|---|---|
| 1.1 to 2.31 (Windows) | Present: the same 24-entry lay order, `item2.wav` and `item.wav` are in every binary from 1.1 to 2.31 **[bin, table only]**. The 2.05 rules are used for all of them; only 2.05 rev 2 was read | 1.38 on: `PLI_ITEMADD` for floor items. 1.33 to 1.37: nothing (see below) |
| 3.153 (Mac) | The same lay order table and `playerlaysitem` are in the binary **[bin, table only]** | Not in Seihai: Ato has no items, sword or inventory yet |
| 3.04 (Linux) | The binary is packed; not checked | – |

## 2.05 rev 2 **[bin]**

- **Picking**: A does this only
  after its other uses, so not when carrying, lifting or grabbing; then the
  attack key must be held and the point in front of the player (by facing
  north, west, south, east: `(1.5, 0)`, `(-0.5, 2)`, `(1.5, 4)`, `(3.5, 2)`) must be neither blocking nor
  water (type 11). The first item the player has in the lay order is picked;
  none, nothing happens. The player goes to mode 9 with the `idle` gani, and
  `sounds\item2.wav` plays at `(x + 1.5, y + 2)`.
- **Lay order**: 0, 1, 2, 19, 3, 4, 5, 6, 16, 8, 12, 23, 7, 20,
  21, 22, 9, 15, 17, 10, 13, 18, 14, 24. The heart container (11) is not in it.
- **Mode 9**: A with S held picks the next item the player has
  (from the one after the current, wrapping) and plays `item2.wav`. A alone
  lays. Any other game key (arrows, D, S) goes back to mode 0 and idle.
- **Drawn**: the item's sprite (2.05 item sprite table),
  centred on `(x + 1.5, y)`, one tile higher on a horse.
- **Laying**: if the player still has the item, it is added at
  `(x + 0.5 + dx, y + 1 + dy)` with `(dx, dy)` by facing north `(0, -3)`, west
  `(-2, 0)`, south `(0, 2)`, east `(2, 0)`, life 164, then taken
  from the player, then `sounds\item.wav` at `(x + 1.5, y + 2)`.
- **Who has what** and **what it costs**:

| Item | Can lay when | Taken |
|---|---|---|
| green / blue / red / gold rupee (0, 1, 2, 19) | rupees ≥ 1 / 5 / 30 / 100 | that many rupees |
| bombs (3), darts (4) | ≥ 5 | 5 |
| heart (5) | hearts ≥ 1.5 | one heart |
| glove1 (6), glove2 (16) | glove power 2 / 3 | glove power 1 / 2 |
| sword (10), battleaxe (13), lizardsword (18), goldensword (14) | sword power 1 / 2 / 3 / 4 with the stock image | sword power 0 / 1 / 2 / 3 |
| shield (9), mirrorshield (15), lizardshield (17) | shield power 1 / 2 / 3 with the stock image | shield power 0 / 1 / 2 |
| spinattack (24) | the spin is allowed | the spin |
| 7, 8, 11, 12, 20 to 23 | never | – |

  A custom sword or shield image (not `sword1` … `shield3`) can be laid as
  item 10 or 9 at any power; 2.05 then lays a special item carrying the
  picture. Seihai does not track the picture and skips this.
- **Wire**: online, the item add sends
  `PLI_ITEMADD` (12) `GCHAR x*2, GCHAR y*2, GCHAR item` (rounded halves) and
  UDP 19, but **only for floor items** (0 to 5, 7, 8, 19, 24). A laid glove, sword or shield is never sent.
  Picking such an item up again sends nothing either (ITEMTAKE only for
  floor items, else the item is applied locally); its timeout
  still sends `PLI_ITEMDEL`.

## Other builds

- **1.38 to 1.41** send `PLI_ITEMADD` for a rolled drop (1.39 capture,
  `item-drops.md`), so Seihai sends it for a lay too.
- **1.42 to 2.31**: the 2.05 item add above.
- **1.33 to 1.37** tell the other players of a rolled drop by UDP only
  (`item-drops.md`). That a lay goes the same way is **inferred**, not read.
  Seihai has no UDP, so these builds send nothing for a lay.

## Not done

- No real client was captured laying, so the bytes are from the binary only.
  A lab run (hold S, press A, A) on 2.05 rev 2 and 2.22 would confirm them,
  and show whether the server sends the new counts (rupees are not sent by
  Seihai at all yet; bombs and arrows go out as counts as after a shot).
- Custom sword and shield images, the horse, the `playerlaysitem` event (it
  fires when an item's life reaches 140).
- 3.x: Ato first needs items and an inventory.
