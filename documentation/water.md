# Water

These notes cover how the player looks in water and the splashes. Walking
speed in water is in `walk-pace.md`. The thrown-object splash is in
`throwing.md`.

## What the 2.05 client does (2.05 rev 2)

The client checks the tile under the player on every game tick. The check is
skipped while the player swings the sword, rides a horse, or is not under
local control.

* **Into the liquid.** On a liquid tile the player goes into swim mode and
  gets the built-in `swim` animation, or `lava` on a lava tile (type 12).
  * When the player was not already swimming, the client starts a splash
    (break effect 5) at the player's position, `(x, y)`.
  * It starts two more splashes when the player was not walking into the
    water, for example after a warp, a knock or a respawn.
  * Each splash plays `water.wav`.
* **Out of the liquid.** Off a liquid tile, a swimming player gets one
  splash and goes back to standing.
* **The figure.** The swim animation draws the ripple, the head and the hat,
  with no body. In the shipped `swim.gani` the head is 20 px lower facing
  up and 16 px lower in the other directions (`asset-reference.md`). This
  is the "head only" look.
* **Built-in animations.** These are the client's own animation names, in
  table order: idle, walk, walkslow, kick, sword, ridesword, spin, push,
  grab, pull, lift, carrystill, hiddenstill, carry, hidden, shoot,
  ridestill, ridefire, rideeat, ride, sit, sleep, hurt, ridehurt, dead,
  maps1, maps2, maps3, swim, lava and carried.

## What Seihai does

* **Swimming figure.** When `swim.gani` is loaded, Seihai draws it. The
  built-in asset set has it, so normally the figure is the gani. Without
  it, `draw_player_sprites` draws the head 14 px lower and no body, which is
  2 to 6 px off the gani.
* **Splashes.** `refresh_water` starts the splashes when the water state
  changes:
  * one when walking in;
  * three when put in without walking;
  * one on the way out.
  They are drawn by the same code as a thrown object's splash (row 5 of the
  break table: eight frames of sprites 326 and 327), and each one queues
  `water.wav`.
* **Tested.** `entering_and_leaving_water_splashes` in `offline/mod.rs`.

## Inferred or open

* **Lava.** The client picks the lava animation inside its water branch, so
  its liquid test must be true for lava tiles too. The test itself is a jump
  table that was not followed. Seihai treats lava as floor (`TileKind::Lava`
  is not swimming) and has no lava gani. Changing this would change walking
  on lava, so it is left as it is until a lab check.
* **Splash spot.** The splash is anchored at the player's `(x, y)`, as the
  client reads its position getters. Whether those return the sprite's
  top-left (as in the carry code) or another point was not traced.
* **Not compared with the real client.** No lab capture of a player
  swimming exists, so Seihai's picture has not been diffed against one.
  Wine is not available in this container.
