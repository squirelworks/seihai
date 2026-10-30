# GANI step timing

Source: Graal_analysis `client-internals/player-mannerisms-live.md` (2.05 rev 2 read live, tick by tick, and the stock gani sets). A tick is the 50 ms game tick.

## The rule

A GANI step is shown for `1 + WAIT` ticks, where `WAIT n` is the line after the step's rows (the stock `hidden.gani` writes `WAIT4` with no space; both are read). A file with no `WAIT` steps once a tick. Code: `Gani::frame_ticks`, `Gani::total_ticks`, `Gani::frame_at_tick` in `client/src/assets/gani.rs`; the poses in `client/src/pose.rs`.

## The poses

| pose | real 2.05 rev 2 (live) | Seihai now |
|---|---|---|
| push | from the 18th blocked step, 5 steps × 4 ticks (`WAIT 3`), back to step 0 when the count wraps 38 → 18 | same; the sprite the others see (S 11, 12, 13, 11, 12) follows the same step |
| pull | 3 steps × 2 ticks (`WAIT 1`); stays on, looping, after the arrow and A are let go, until a key moves the player | same; a swing, a hit, a lift, death or a jump also end it |
| hurt | 31 ticks (`WAIT 30`), then idle | by the file's waits; the default `hurt.gani` now has `WAIT 30` |
| lift | 6 ticks (`WAIT 5`), then carrystill | `lift.gani` by its waits, then the carry poses; without the file, carrystill at once |
| swim | 8 steps × 2 ticks, **animating while still**; 0.5 tile a tick, the walking pace | runs on the tick clock, moving or not; `SWIM_STEP_MS` is 50 (was 100, with no source) |
| dead | by the file's waits | by the file's waits, held on the last step |

`push.gani`, `grab.gani`, `pull.gani` and `lift.gani` now load at start from the asset folders as well (before, only a copy the server sent was loaded).

## Not known or not done

* **1.39.** It has no ganis. Its push frames change about every 2 ticks, not 4 (video only). Seihai has no built-in pre-gani poses, so a 1.39 profile draws whatever ganis it finds.
* **The pull sprite after A is let go.** The pose was read live; what sprite number the 2.05 client sends others then was not traced. Seihai sends the pull sprite while the pose is on (inferred).
* **Walk with `WAIT`.** The walk GANI still steps once per move. The stock `walk.gani` has no `WAIT`, so that matches; `walkslow` (8 × 2) is not used.
* **Blink when hurt** and **hiding under a carried object** are separate changes.
