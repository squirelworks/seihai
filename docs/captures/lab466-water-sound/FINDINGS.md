# #466 captures: Ato 3.153 water sound, the derived data

Sources:
- the water sound and `--no-sound` runs (6a and 6b, 2026-11-13)
- five water-to-dry level-change runs (r1 to r5, 2026-11-14)

All of them ran on GwenTea2: Ato 3.153 against seihai-gserver-v2 serving an Enigma copy, with the real 3.04 sounds and a null-sink recording. The setup is in the write-ups under [`docs/lab/`](../README.md).

Times are seconds after the 3 kHz start beep. Onsets were found by cross-correlating the recording with the real `water.wav` and `steps2.wav`, and `ncc` is the match score.

## Tables

| File | What it holds |
|---|---|
| `sound-onsets.tsv` | Every detected `water.wav` and `steps2.wav` onset in 6a and r1 to r5, with its score and the input or Ato event just before it. 6b (`--no-sound`) has none |
| `level-change-crossings.tsv` | For each level-change run: the link time, the time the dry level loaded, splashes within 0.1 s of the load, and the first confident footstep after it |
| `splash-cut-fit.tsv` | Two crossings that happened mid-splash, fitted with a full 0.438 s splash and with one cut at the level change |

## Findings

### Cadence

- **Floating:** with no keys held in deep water, a splash plays every 0.80 s on average (0.74 to 0.87 s). That is the swim GANI's loop.
- **Walking:** `steps2.wav` plays every 0.20 s (median; 0.16 to 0.22 s).
- **Run 6a:** had 61 onsets: 18 water and 43 footsteps. The last water onset was at 34.99, and footsteps resumed at 35.37.
- **Run 6b (`--no-sound`):** had no onsets at all.

### Water to dry level change (r1 to r5)

| Run | Link | Dry level loaded | Link to level | Splash near the load | First step after the load |
|---|---|---|---|---|---|
| r1 | 22.75 | 23.06 | 0.31 | −0.015 | 0.389 |
| r2 | 22.69 | 23.00 | 0.31 | +0.046 | 0.407 |
| r3 | 23.20 | 23.50 | 0.30 | +0.015 | 0.376 |
| r4 | 23.70 | 24.01 | 0.31 | −0.004 | 0.400 |
| r5 | 23.50 | 23.80 | 0.30 | +0.035 | 0.439 |

- **Link to level:** every run took 0.30 to 0.31 s from the link to the new level loading.
- **Splash at the load:** every run had exactly one splash within ±0.05 s of the dry level loading. That is the leaving-the-water splash described in the write-up. None followed it.
- **Footsteps:** the first confident footstep came 0.38 to 0.44 s after the load. Weaker matches, with a score under 0.8, overlap the tail of the last splash and are left out.
- **Mid-splash crossings:** two crossings happened mid-splash, 0.336 s and 0.161 s into it. In both, a full 0.438 s splash fits the recording better than one cut at the level change:

  | Crossing | Residual, full splash | Residual, cut splash |
  |---|---|---|
  | 0.336 s in | 0.305 | 0.342 |
  | 0.161 s in | 0.241 | 0.538 |

  So a splash already playing is not stopped by the level change.

Still open, and needing a real client: whether a real 3.x also splashes when a warp takes the player out of the water.
