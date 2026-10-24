# Walk pace

Queue item 15, from the differential testing (`documentation/difftest.md`, divergence 9: "the real
client moves 0.5 tile every 50 ms, Seihai about 7.8 tiles a second against 10").

## Cause

The walk step is already 50 ms (`WALK_STEP_MS`), but the game loop ticks about every 16 ms (the
`limit_update_rate` of the window). A step was due when 50 ms had passed since the last one, and the
clock restarted from the moment the step was taken. Three ticks are 48 ms, so every step waited for the
fourth tick: one step per 64 ms, 7.8 tiles a second. The real clients step every 50 ms.

## Change

`offline::next_step_clock` counts the next step from the time the last one was due, not from when it was
taken. A step noticed late (at 64 ms) leaves the next one due at 100 ms, so the average is one step per
50 ms. After a pause of two intervals or more the clock starts from now, so a held key after standing
still does not fire a burst. Swimming (100 ms a step) uses the same rule with its own interval.

## Before and after (lab, `gmap_cross`, real client against Seihai)

Interval between walk position messages (a message carries one half-tile step in 2.31, and the
position every step in Seihai):

| | real 2.31 | Seihai before | Seihai after |
|---|---|---|---|
| median interval | 50 ms | 64 ms | 48 ms |
| mean interval | 51.0 ms | 65.2 ms | 50.5 ms |
| position messages in the run | 437 | 344 | 442 |
| tiles per second (0.5 tile a step) | 9.8 | 7.7 | 9.9 |

2.22 real sends the position every second step (100 ms, one tile), so the same Seihai numbers
(65.2 ms before, 50.5 ms after) still differ from its 100 ms median; the number of `gr` flags, one per step,
now matches: 448 real, 349 before, 449 after. The "walk speed" row of the ranked report is gone for both
versions. What is left of the 2.22 cadence is in `documentation/difftest-leftovers.md`.
