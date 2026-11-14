# Lab #466: does the water sound stop on a level change? (2026-11-14)

This lab follows up the 2026-11-13 run (`../2026-11-13-lab466-water-sound/`). That run could not test a level change, because both of its links were crossed between splashes, from water into more water. This lab adds a link from deep water to a dry spot in another level and crosses it while a splash is playing.

I ran it on GwenTea2, natively. Ato and the gserver ran inside a loopback-only network namespace (`ns.sh`: a `bwrap --unshare-net` holder that has only `lo`) on Xvfb `:87`, as the #458 pass 2 run did. I used no VM and opened no windows on the host desktop.

## Setup

The setup is the same as the Mirae run, adapted to GwenTea2:
- **Client:** Ato 3.153 (`--version 3.153`), from seihai main `4d1a28d`, built with default features (audio on). The sha256 is in `sha256-ato.txt`.
- **Sounds:** the real 3.04 Linux client tree (`--assets`), from `cadence-lab-2026-11-13/c304/tree`. Its `water.wav` has the same sha256 (`eb29d4b3…`) as the 3.04 `setup.data/release/sounds/water.wav`.
  - `water.wav`: 8 kHz, 0.438 s, one-shot. The swim GANI plays it.
  - `steps2.wav`: 0.183 s. The walk GANI plays it.
- **Server:** seihai-gserver-v2 `c851b4e` (the #458 pass 2 build), serving a copy of the Enigma 12-21-06 world. Its config and folder setup match Mirae's (generation `newmain`, port 14806, the dump's `foldersconfig.txt`, start at `onlinestartlocal.graal` 23.5, 30.5). It ran in localhost mode, so there was no listserver.
- **Lab edit, in the copy only:** `oldeg/egk-10.graal` gets one extra link (`egk-10-lab-link.diff`):

      LINK onlinestartlocal.graal 4 40 1 10 23.5 30.5

  - The link is a 1×10 strip at x = 4 in the deep water on the left of `egk-10`, where the earlier run floated at (7, 45). It leads to the grass start spot of `onlinestartlocal.graal`.
  - I made the edit with `Graal_analysis/tools/level-convert`: GR-V1.03 to `.nw`, add the line, then back to `.graal`. A round trip of the unchanged level gave identical output, and the edited level decodes to the original plus this one line.
- **Recording:** a PipeWire null sink `lab466null`, created with `pw-cli` (GwenTea2 runs PipeWire and has no `pactl`).
  - A zero stream (`pw-cat … < /dev/zero`) keeps the sink from suspending.
  - `pw-record` with `stream.capture.sink=true` records the sink's monitor.
  - Ato's ALSA output goes to the sink through `PIPEWIRE_NODE=lab466null`.
  - A 0.1 s, 3 kHz beep played into the sink marks t = 0. Every time in this README is in seconds after the beep.

## Route (`run.sh TAG FLOAT`)

The route is the same as Mirae's up to the float:
1. Down 0.2 s, then Down 3.3 s, into `egk-10`.
2. Left 0.2 s, then Left 2.5 s on land, then wait 3 s.
3. Left 1.6 s into deep water, where the player floats at about (8.5, 45).
4. Wait FLOAT seconds, then **hold Left 1.5 s**. This swims into the link, and the key stays down after the player arrives on the grass.
5. Wait 3 s, then Down 1.0 s on dry land.

I ran it five times, with FLOAT set to 2.0, 1.75, 2.25, 2.5 and 2.75 s, to move the crossing to different points of the splash cycle.

## Analysis

- **`analyze.py`:** Mirae's script, with three changes:
  - It gets the energy term from a cumulative sum, which is faster and gives the same result.
  - It adds a per-crossing section: whether a splash was in progress at the link and at the level change, and the water and footstep onsets after the link.
  - It adds an envelope check: the recorded RMS divided by (fitted gain × template RMS), in 50 ms bins.
- **`ncc_window.py`:** a normalised cross-correlation over a time window that keeps peaks down to 0.3. Two `water.wav` copies that overlap pull each other below the 0.8 onset threshold, and this script still finds them. Its output is in `ncc-window.txt`.
- **`cutfit.py`:** for two overlapping splashes, a least-squares fit of the recording against two models of the first splash: played in full, or cut at the level change. Its output is in `cutfit.txt`.

**Events:** "link" is the time of Ato's `link to …` stdout line, when Ato asks for the new level. "level" is the time of its `level … from the server's board` line, when the new level is in place. Ato's stdout carries Unix timestamps on the same clock as the key and beep events.

## Results

The table lists times in seconds after the beep. "Splash" is the last `water.wav` onset before the link; it ends at onset + 0.438.

| Run | FLOAT | Splash | Link | Into the splash at link | Level | Into the splash at level | Splash plays… | Splash at level load | First footstep |
|---|---|---|---|---|---|---|---|---|---|
| r1 | 2.0 | 22.320 | 22.754 | 0.434 (at its end) | 23.057 | ended | full (to 22.760) | 23.045, full 0.438 s | 23.45 |
| r2 | 1.75 | 22.662 | 22.692 | **0.030** | 22.998 | **0.336** | **full**: fit 0.305 full vs 0.342 cut | 23.046, full | 23.41 |
| r3 | 2.25 | 23.344 (between link and level) | 23.199 | gap (the splash from 22.577 ended at 23.015) | 23.505 | **0.161** | **full**: fit 0.241 full vs 0.538 cut | 23.515, full | 23.88 |
| r4 | 2.5 | 23.451 | 23.698 | **0.247** | 24.005 | ended (23.889) | **full**: 50 ms ratios 0.98–1.15 to 0.40 s, then silence | 24.006, full | 24.41 |
| r5 | 2.75 | 23.281 | 23.496 | **0.215** | 23.804 | ended (23.719) | **full**: ratios 0.98–1.15 to 0.40 s, then silence | 23.835, full | 24.24 |

Link to level took 0.30–0.31 s in every run.

### 1. Is a splash in progress cut at the crossing? No, it plays out.
- **Link crossed mid-splash (r4, r5):** the link was crossed 0.247 s and 0.215 s into a splash. The splash played its full 0.438 s: the 50 ms envelope ratio stays at 0.98–1.15 through the 0.40 s bin, then drops to zero.
- **Level change mid-splash (r2, r3):** the new level arrived 0.336 s (r2) and 0.161 s (r3) into a splash. A new splash overlaps it about 0.05 s later (point 2), so I fitted both models.
  - **r3:** a full-length first splash fits much better (residual 0.24) than one cut at the level change (0.54).
  - **r2:** the full splash also fits better (0.31 vs 0.34). Here only the last 0.10 s differs between the models, so the margin is small.
- **Verdict:** a splash in progress is not cut by the link or by the level change. `water.wav` is a one-shot, and `stop_all_loops()` on a new level does not touch it, which matches `seihai-play` `Game::play_sounds`.

### 2. Does water.wav play after arriving on dry land? Yes, exactly once, at the level load.
- **One more splash at the level load:** in all five runs, a `water.wav` starts at the moment the dry level loads. Its onset is 0.001–0.048 s from the `level` line (r1 23.045 / 23.057, r2 23.046 / 22.998, r3 23.515 / 23.505, r4 24.006 / 24.005, r5 23.835 / 23.804).
  - It plays in full (0.438 s) with the player on grass.
  - It is not the swim cycle's next beat: in r4 it came 0.555 s after the previous splash, against a cycle of about 0.75–0.85 s.
  - It seems the swim stroke is restarted, or its first frame's `PLAYSOUND water.wav` re-fired, when the level is replaced, before Ato's next check of the tile under the player puts it back to walking.
- **One splash during the transit (r3):** a splash started 0.145 s after the link and before the level arrived (23.344). The swim cycle keeps running during the 0.3 s wait for the new level.
- **None after that:** after level + 0.2 s, I found no `water.wav` in any run, even at an NCC threshold of 0.3 (`ncc-window.txt`).

### 3. Do footsteps play on dry land? Yes.
- **On arrival:** `steps2.wav` starts 0.39–0.44 s after the level loads, while Left is still held (r1 23.45, r2 23.41, r3 23.88, r4 24.41, r5 24.24). Each run then has 2–4 steps at a 0.17–0.21 s interval.
- **On the later Down:** 2 more steps each run.
- **Overlap:** the first step overlaps the end of the level-load splash. That is why two runs show weak `steps2` matches (ncc 0.46–0.49) inside that splash.

## Verdict for #466

- **Water stops on a level change:** pass, with one extra splash.
  - The swim sound does not carry on into the dry level, and footsteps take over.
  - But every crossing from water to dry land plays one more full `water.wav`, triggered at the level load, with the player already on dry land.
  - A splash already playing is never cut. Whether that is right depends on what the real 3.x client does, which this lab did not measure. Not cutting is consistent with `water.wav` being a one-shot.
- **Open question for the code:** does the real client also fire a splash on the level load? If not, Ato should not restart the swim GANI on the new level, or should re-check the tile under the player first.

## Files

- **`r1/` … `r5/`, one per run:**
  - `analysis.txt`: events, all onsets, the per-crossing section, the envelope check, RMS per second
  - `events.txt`: keys and screenshots, Unix time
  - `ato-stdout-timed.txt`: Ato's stdout with Unix times
  - `2-in-water.png`: floating before the crossing
  - `3-arrived.png`: on the grass of `onlinestartlocal.graal`
- `cutfit.txt`, `ncc-window.txt`: the overlap fits and the low-threshold scans
- `egk-10-lab-link.diff`: the only change to the world copy
- `gserver-enigma-console-r1.txt`: the gserver's console log for r1. All five runs logged in (`lab466r1`–`r5`, G3D28015 / 3.153).
- **Scripts:** `run.sh`, `analyze.py`, `ncc_window.py`, `cutfit.py`, `ns.sh`, `xt.py` (the XTest key driver; GwenTea2 has no xdotool), `stamp.py`, `mypids.sh`
- `sha256-ato.txt`: the Ato binary's hash

The recordings themselves (about 7 MB each) are not committed. They stay on GwenTea2 in `~/Claude/labs/lab466-levelchange-2026-11-14/out/`.
