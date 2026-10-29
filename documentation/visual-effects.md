# Visual effects

The 2001 new-features notes describe these commands. The 2.05 code stores them as follows:

| Command | Stored on | Default |
|---|---|---|
| `seteffect r,g,b,a` | the whole map (`OfflineSession::screen_effect`) | 0,0,0,0 |
| `setcoloreffect r,g,b,a` | the NPC (`Npc::effect.color`) | 1,1,1,1 |
| `setzoomeffect z` | the NPC (`Npc::effect.zoom`) | 1 |
| `changeimgcolors i,r,g,b,a` | `showimg` image `i` | 1,1,1,1 |
| `changeimgzoom i,z` | `showimg` image `i` | 1 |

Like the 2.05 client, Seihai runs them only offline or when the server has an NPC server (`vocab::online_gated`).

## How Seihai draws them

- **Effect on.** A picture takes the effect path when one of its colour values is below 1 or its zoom is not 1. This is the same test the 2.05 client makes.
- **Colour.** Each pixel's red, green and blue are multiplied by the effect's red, green and blue. The pixel is then laid over what is under it with the effect's alpha.
- **Zoom.** The picture is scaled about the centre of its rectangle (nearest pixel). Its blocking does not change.
- **Lights.** A `drawaslight` NPC with an effect on adds its colour to what is under it, instead of being laid over it.
- **seteffect.** The colour is laid over the whole map with its alpha. This happens after the `drawoverplayer` NPCs and the `showimg` images, and before the `drawaslight` NPCs, so lights show through the dark. It is drawn under the names, the bubble and the status bar.

## What is confirmed and what is not

- Confirmed from the 2.05 code: the stored values and defaults; the effect-on test; that `setcoloreffect` and `setzoomeffect` are refused online without an NPC server.
- From the notes: that `seteffect` sits under `drawaslight` NPCs and the stats. The notes also say the effects are seen "only with a 3D card".
- Inferred, not tested:
  - The blend formulas.
  - That lights add their colour.
  - That Seihai behaves as a client with a 3D card. The 2.05 client without one takes the effect path only for an alpha below 1.
- Not done: effects on NPCs drawn as a character (`showcharacter`). `changeimgvis` is in `npc-draw-order.md`.
