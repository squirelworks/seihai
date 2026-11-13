# Lab #466: Ato water sound and `--no-sound` (2026-11-13)

This lab is step 6 of the Ato lab plan, run in Mirae's Linux RE Lab VM, headless. It used the same setup as the footsteps check:
- **Client:** Ato 3.153, from seihai main `a312be3` built with audio, using the real 3.04 client's sounds (`--assets ~/seihai-lab/client-304`).
- **Server:** the Enigma lab world, served by a seihai-gserver-v2 `101a2cce` gserver plus a listserver, inside a private loopback-only network namespace.
- **Recording:** a PulseAudio null sink of my own, `lab466null`.

## Method

`run.sh` holds the arrow keys with xdotool, along the route the earlier footsteps run used:
1. Walk down from the start level through the link into `egk-10.graal`.
2. Walk left over land and into shallow water, then into deep water, and float for 5 s.
3. Swim left across the level edge into `egj-10.graal`.
4. Swim right back into `egk-10.graal` and onto land, then walk right.

**Timing:**
- **Clock alignment:** the null sink is kept running with a stream of zeros. Without it, the sink suspends while idle and the recording's timeline compresses; my first two 6a runs had this problem, and I discarded them. A 0.1 s 3 kHz beep is played into the sink at a logged time and marks t = 0.
- **What has a timestamp:** every key press and screenshot, and every line of Ato's stdout (`ato-stdout-timed.txt`, Unix time). All are on the same clock.

**Detection:** `analyze.py` finds each sound by normalised cross-correlation against the real files:
- `water.wav`: 8 kHz, 0.438 s. The swim GANI plays it.
- `steps2.wav`: 0.183 s. The walk GANI plays it.

It lists every onset (ncc ≥ 0.8) against the events, plus the RMS per second (`analysis.txt`).

Footstep onsets follow each key press within about 0.3 s. For example, Down is pressed at 6.26 s and the first step plays at 6.53 s, so the alignment holds.

## 6a: water sound, sound on: pass, with one part not testable

Times are seconds after the beep, from `6a/analysis.txt`.

| Moment | Seen |
|---|---|
| Walking on land in `egk-10.graal` | `steps2.wav` every 0.2 s (7.81–10.23, 13.89–16.71, 20.38–20.78). No water. |
| Into the water (Left held from 20.26) | the last step at 20.78, then `water.wav` from 20.91 |
| Floating in deep water (22.22–27.43, no key) | `water.wav` every ~0.8 s (22.87, 23.61, 24.42, 25.29, 26.08, 26.87). The swim loop keeps splashing while the player floats. |
| Swimming left | 27.64, 28.42, … |
| **Level change** into `egj-10.graal` (Ato: `link to egj-10.graal at 61 45` at 28.11, `level egj-10.graal: …` at 28.50) | The splash that started at 27.64 had already ended (silent from 28.10). The next splash started at 28.42, between the link and the new level, and the swim went on in `egj-10.graal` (29.20, 30.04). |
| Turning around (Left up 30.48, Right down 30.88) | one beat skipped (30.04 to 31.38) |
| **Level change** back into `egk-10.graal` (link 32.77, level 33.11) | The splash from 32.16 ended at about 32.60, before the link. The next one started at 32.97, and the swim continued (33.78, 34.56). |
| **Onto land** (Right held, `5-back-on-land.png` at 24.5, 45.0) | The last water onset was at 34.99 (ncc 0.82). Footsteps resume at 35.37 and run on (37.16–38.56 on the next Right). No water after 35.4. |

Verdict:
- **Plays while swimming:** pass.
- **Stops on land:** pass.
- **Footsteps resume on land:** pass.
- **Stops on a level change:** not testable on this route.
  - At both level changes, no `water.wav` was playing when the link was crossed. The splash lasts 0.44 s and repeats every 0.8 s, and both links fell in the gap.
  - The player was still in water in the new level, so the splash rightly went on there.
  - A clean test needs a link where the player leaves water for a dry level, with a splash in progress as the link is crossed.

## 6b: `--no-sound`: pass

The same route, with `--no-sound`:
- **Silence:** after the beep, every one of the 1,806,404 recorded samples is zero (max abs 0). No `water.wav` or `steps2.wav` onsets were found, and Ato printed no `sound:` line. The beep itself was recorded (gain 0.37), so the recording path worked.
- **Walking and swimming:** both still work.
  - Ato logged the same route: `link to egk-10.graal at 54.5 19`, `level egk-10.graal …`, `link to egj-10.graal at 61 45`, `level egj-10.graal …`, `link to egk-10.graal at 0 45`, `level egk-10.graal …`.
  - The screenshots show the swim pose at (7.0, 45.0) in deep water (`6b/3-in-water.png`) and the player back on land (`6b/5-back-on-land.png`).

## Files

- `6a/`, `6b/`, one per run:
  - `ato-stdout-timed.txt`: Ato's stdout with Unix times
  - `events.txt`: keys, marks and screenshots
  - `analysis.txt`: onsets of both sounds, events, RMS per second
  - screenshots
- `run.sh`: the run script
- `analyze.py`: the analysis
- `gserver-enigma-console.txt`: the gserver's log

The recordings themselves (about 7 MB each) are not committed. `analysis.txt` has their RMS per second and every onset.
