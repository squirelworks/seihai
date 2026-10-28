# Claude Full Laundry List — status 2026-10-29

The 2026-10-28 list, checked against main.

## Done

- **Deep water** — blocks. Lava is swum and burns (half a heart on the 101st tick). Lava floor is walked and still burns. #111.
- **Edge links** — ±64 across the seam for big-map neighbours. #98.
- **Ledges** — jump after 18 blocked steps, push after 28, held grab and pull. #105.
- **Signs** — open by walking up into them, not with A. #100.
- **Grab and sword** — futile grab holds while A is down and plays the pull GANI. #105.
- **Draw order** — `drawoverplayer`, `drawunderplayer`, `drawaslight`. #99.
- **Offline .graal tiles** — black squares: keep tile 0. #101.
- **Visual effects** — `seteffect`, `setcoloreffect`, `setzoomeffect`, `changeimgcolors`, `changeimgzoom`. #106.
- **Script parser** — `elseif`, `|lo,hi>`, indexed fields, `<>` `=<` `=>`. 498 down to 444. #102.
- **Debug level name** — log names the current level. #104.
- **Hurt and death animations** — stepped like the real client. #107.
- **Other players' looks** — sword, shield, head, body, dead pose. #103, #110.
- **Tile types** — throw-through and bed lists from arrays.dat, `onwater`, `tiletype`. #108.
- **Configurable keys** — readme map, `keyset_` lines. #109.

## Still open

- **Parser leftovers** — 444 of 3038 scripts still have diagnostics. Array literals (`this.a = {1,2,3}`), `with`, `?:`, empty arguments, `#P1(-1)` codes. Not in 2.05.
- **Chat bubble fade** — the bubble still shows. The fade after a few seconds is not done.
- **Online board modify** — a moved push/pull block does not send a board modify. Same gap as lifting.
- **`canbepushed` / `canbepulled` NPCs** — they get the action event but do not move.
- **`changeimgvis`** — recognized, not drawn.
- **Effects on character NPCs** — not applied.
- **GANI attributes and colours** — prop 13 not applied to other players.
- **Hats** — not drawn.
- **`dead.gani`** — Seihai does not ship one. Without it, a dead player has no pose.

## What works

- Live login on 2gta.home.eevul.net:14913 and 45.59.132.11:30903
- Levels load from server boards
- Images requested and cached
- Hearts stay at default when server sends power 0
- Chat sends and shows in the bubble
- Chests open
- Objects lift and throw
- Developer mode wallhack toggles
- Offline .graal levels load
- Links fail cleanly when the target is not local
- Deep water blocks, lava burns
- Edge links land on the opposite edge
- Ledges jump, blocks push and pull
- Signs open on touch
- Draw order puts trees over the player
