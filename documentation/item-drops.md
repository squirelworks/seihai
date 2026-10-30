# Item drops from bushes and pots

What a cut or lifted bush and a lifted pot leave, and the items on the floor. Read in the 1.3, 1.39, 1.41 and 2.05 rev 2 binaries and checked in the lab with real 1.39, 2.05 rev 2 and 2.22 clients, then with Seihai against the same lab servers. The findings and captures are in Graal_analysis `client-internals/item-drops.md`. Code: `client/src/items.rs`, `client/src/offline/drops.rs`, `render::draw_floor_items`.

## Per build (`ItemDrops` in the version profile)

| Builds | Profile | Who drops | What Seihai sends online |
|---|---|---|---|
| 1.33 to 1.37 | `DROPS_133` | The client rolls | Pickup: applied, then `PLI_ITEMDEL` (13). Roll and timeout: nothing over TCP |
| 1.38 to 1.41 | `DROPS_138` | The client rolls | Roll: `PLI_ITEMADD` (12). Pickup: `PLI_ITEMTAKE` (32), the server gives it with props. Timeout: `PLI_ITEMDEL` |
| 1.42 to 2.31 | `DROPS_142` | The server (`PLO_ITEMADD`, 22) | Pickup: `PLI_ITEMTAKE`. Timeout: `PLI_ITEMDEL` |

- **The roll** (1.33 to 1.41): one Delphi `Random(100)` for every cut (any cut-table entry, tall grass too) and every lifted bush: 0 to 24 a green rupee, 25 and 26 a blue rupee, 27 to 31 a heart, 32 to 36 bombs, otherwise nothing. A lifted pot always leaves a heart. Signs and stones leave nothing, and no client rolls arrows.
- The item lies at the object's top-left tile. Offline a 1.x build rolls the same way; a 2.x build drops nothing offline, as the real one.
- **Not sent:** the UDP messages 19 and 20 that 1.33 to 1.41 send to the other players. Seihai has no UDP.
- **Not done:** bombs destroying bushes, pots and stones (the 1.x blast rolls a bush and gives a heart for a pot). Seihai has no bomb blast on the board yet.

## The floor item (2.05 rev 2)

- Life 164 ticks of 50 ms (about 8.2 s); below 0 it goes.
- Taken once the life is below 160 when `|px + 1.5 - (x + 1)| < 2` and `|py + 2 - (y + 1)| < 2`. Not while dead. Sound `extra.wav`.
- Offline (and online before 1.38) the pickup is applied: rupees 1, 5, 30, 100 (up to 9,999,999), five bombs or arrows (up to 99), one heart.
- Online from 1.38 the server's RUPEESCOUNT, BOMBSCOUNT and ARROWSCOUNT props (3, 4, 5) set the counts.
- Blinks (not drawn on odd ticks) below life 37; bounces `|sin(life / 3)| · (life / 164)^16 · 3` tiles for the first 22 ticks.
- Drawn under the players (inferred): shadow sprite 42 at `(x + 0.5, y + 1 + h / 2 - 0.5)`, the item centred on `(x + 1, y + 1)`.
- A level change clears the floor. `PLO_ITEMDEL` removes the item at its position.

## Lab check (2026-10-30, lab servers only)

- **2.05 against the old lab pack:** a cut got S>C 22 (bombs), drawn with shadow and bounce; Seihai sent C>S 13 8.39 s later (the real 2.05: 8.20 s). A heart walked over after 1.09 s: C>S 32, then S>C 9 CURPOWER.
- **1.39 against beta4 (`generation = classic`):** a lifted bush rolled a green rupee: C>S 1 and C>S 12 in the same frame, then C>S 32 on pickup and S>C 9 RUPEESCOUNT; the HUD went from 1 to 2. The real 1.39 sent the same messages.

## Open

- 1.41's pickup and timeout are assumed to be 1.39's.
- The 1.x pickup box was read only as 1.3's pixel overlap; the 2.05 tile rule is used for every build.
- Whether items draw under or over the players.
