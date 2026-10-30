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

## Per build (`profile.mannerisms`)

Source: Graal_analysis `player-mannerisms-live.md`, "Cross-version table" (1.39, NewMain 1.41.1, 2.05 rev 2 and 2.22 read live). The row is set in the version chain (`protocol/chain.rs`) and taken by the session with `OfflineSession::set_mannerisms`.

| | 1.39 (`MANNER_139`) | NewMain 1.41 (`MANNER_141`) | 2.05 (`MANNER_205`) | 2.22 (`MANNER_222`) |
|---|---|---|---|---|
| hurt blink (figure left out every other tick, shadow kept) | once the hurt pose is over (about 10 ticks without `hurt.gani`) | from the hit | from the hit | from the hit |
| knockback | 4 tiles | 5 | 5 | 5 |
| arrow let go while pulling | pull stays | back to `grab` | pull stays | pull stays |
| A let go after a pull | pull loops | `grab` stays until the next move | pull loops until the next move | = 2.05 |
| swim while floating still | runs on | back to step 0 and held | runs on | back to step 0 at the stop, then runs on |
| hide (D while carrying) | carry poses | carry poses | `hiddenstill` / `hidden` | = 2.05 |

Where each row starts in the chain:

* 1.33 to 1.41 take the 1.39 row. Only 1.39 was run; 1.33 to 1.38 are inferred.
* The NewMain 1.41 row starts at Seihai's 1.42 step. That step is the first build in the chain with ganis, and the analysis ran NewMain 1.41.1 (`graalnm24`), which is the first gani client. Pairing those two is inferred.
* 2.00 to 2.04 keep the NewMain row (not run). 2.05 sets its own row, 2.1 to 2.17 keep it, and 2.22 and later take the 2.22 row.
* The hide row is recorded, but Seihai does not have hiding yet.
* 1.39 has no combat rules in Seihai, so its 4-tile knockback does not take effect until a pre-2.05 hit path exists.

## Not known or not done

* **1.39.** It has no ganis. Its push frames change about every 2 ticks, not 4 (video only). Seihai has no built-in pre-gani poses, so a 1.39 profile draws whatever ganis it finds.
* **The pull sprite after A is let go.** The pose was read live; what sprite number the 2.05 client sends others then was not traced. Seihai sends the pull sprite while the pose is on (inferred).
* **Walk with `WAIT`.** The walk GANI still steps once per move. The stock `walk.gani` has no `WAIT`, so that matches; `walkslow` (8 × 2) is not used.
* **Hiding under a carried object** is a separate change.
