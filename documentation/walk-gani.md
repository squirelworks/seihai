# Walking, idle and swimming animations

The local player used to be drawn from fixed offsets (the idle figure) unless a
sword swing, hurt or sit pose was on. `walk.gani` and `swim.gani` were only
stepped for their footstep and water sound cues, and only when sound was on.
So the player glided with a static body.

Now `walk.gani`, `swim.gani` and `idle.gani` (the files Seihai ships under
`assets/default/gani`, or the user's folder or a server's cache, first found)
are drawn:

* **Standing** (no arrow key held): `idle.gani`, step 0. Drawn through the GANI
  renderer, it is pixel-identical to the old built-in figure (a test checks
  all four directions), so nothing changes for a build without the files.
* **Walking**: the `walk.gani` frame that the walk has stepped to. The walk
  steps one frame per step taken, every 50 ms on foot (the pace in
  `walk-pace.md`), the same stepping that plays the footstep cues. A step the
  walls refuse does not move the animation, and a player who is only pushing
  against a wall (keys held, no step) keeps the frame it had.
* **Swimming**: the `swim.gani` frame, stepped once per swimming step (100 ms).
  There is no sword, hurt or sit pose in water, as before.
* Sword, hurt and sit keep their priority over the motion poses.

The sprite value other players are sent (S = direction, sword step, 32 for
hurt) is computed from the same state as before and does not depend on which
pose is drawn.

`walk.gani` and `swim.gani` now load whether or not sound is on (the cues are
still only played when it is).

## Open

* Other players are still drawn from the static figure; their walking is not
  animated.
* The real client's "pushing against a wall" pose (it switches state when a
  step is refused) and the walking-with-a-carried-object pose are not drawn.
* The frame timing of GANI steps with their own wait values is not used: one
  frame per step is the rule the existing walk stepping and its tests use.
