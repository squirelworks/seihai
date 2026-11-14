"""Find water.wav and steps2.wav onsets in a null-sink recording; RMS per second.

usage: analyze466.py <run dir with rec.wav and events.txt> <sounds dir> <out.md>
"""
import sys, wave, numpy as np
from scipy.signal import resample_poly, fftconvolve
from math import gcd

run, snd, outp = sys.argv[1:4]

def load(path):
    w = wave.open(path)
    n, ch, sw, sr = w.getnframes(), w.getnchannels(), w.getsampwidth(), w.getframerate()
    raw = w.readframes(n)
    if sw == 1:
        a = (np.frombuffer(raw, np.uint8).astype(np.float64) - 128) / 128
    else:
        a = np.frombuffer(raw, np.int16).astype(np.float64) / 32768
    a = a.reshape(-1, ch).mean(axis=1)
    return a, sr

rec, sr = load(f"{run}/rec.wav")
ev = [l.split(None, 1) for l in open(f"{run}/events.txt").read().splitlines()]
import os
if os.path.exists(f"{run}/ato-stdout-timed.txt"):
    for l in open(f"{run}/ato-stdout-timed.txt"):
        t, rest = l.rstrip("\n").split(" ", 1)
        if rest.startswith(("link to", "level ", "sound:")):
            ev.append([t, "ato: " + rest[:70]])
ev.sort(key=lambda e: float(e[0]))
t0 = float([e for e in ev if e[1] == "beep"][0][0]) if any(e[1] == "beep" for e in ev) else float(ev[0][0])
events = [(float(t) - t0, what) for t, what in ev]

def template(name):
    a, s = load(f"{snd}/{name}")
    g = gcd(s, sr)
    a = resample_poly(a, sr // g, s // g)
    return a - a.mean()

def onsets(tpl, thr):
    # normalised cross-correlation
    n = len(tpl)
    num = fftconvolve(rec, tpl[::-1], mode="valid")
    c2 = np.concatenate([[0.0], np.cumsum(rec ** 2)]); e = c2[n:] - c2[:-n]  # same as convolving with ones(n), faster
    ncc = num / (np.sqrt(np.maximum(e, 1e-12)) * np.linalg.norm(tpl))
    hits, i = [], 0
    while i < len(ncc):
        if ncc[i] > thr and e[i] > 1e-4:
            j = i + int(np.argmax(ncc[i:i + n // 2]))
            hits.append((j / sr - SHIFT, float(ncc[j])))
            i = j + n // 2
        else:
            i += 1
    return hits

# align: the beep (3 kHz, 0.1 s) marks t = 0
import numpy as _np
bt = _np.sin(2 * _np.pi * 3000 * _np.arange(4410) / sr)
num = fftconvolve(rec, bt[::-1], mode="valid")
beep_at = int(_np.argmax(num)) / sr
rec_peak = float(_np.max(num) / (_np.linalg.norm(bt) ** 2))
rec = rec.copy(); rec[int(beep_at*sr):int(beep_at*sr)+4410] = 0
SHIFT = beep_at
water = onsets(template("water.wav"), 0.8)
steps = onsets(template("steps2.wav"), 0.8)

rms = []
for s in range(int(len(rec) / sr - SHIFT)):
    seg = rec[int((s + SHIFT) * sr):int((s + 1 + SHIFT) * sr)]
    rms.append(20 * np.log10(max(np.sqrt(np.mean(seg ** 2)), 1e-9)))

def ev_at(t):
    last = "start"
    for et, w in events:
        if et <= t:
            last = w
    return last

with open(outp, "w") as o:
    o.write(f"beep found at {beep_at:.3f} s into the recording (gain {rec_peak:.2f}); times below are seconds after the beep event\n")
    o.write(f"recording {len(rec)/sr:.1f} s at {sr} Hz; peak abs {np.abs(rec).max():.4f}\n\n")
    o.write("events (s after the beep):\n")
    for et, w in events:
        o.write(f"  {et:7.2f} {w}\n")
    o.write(f"\nwater.wav onsets ({len(water)}):\n")
    for t, c in water:
        o.write(f"  {t:7.2f}  ncc {c:.2f}  (after: {ev_at(t)})\n")
    o.write(f"\nsteps2.wav onsets ({len(steps)}):\n")
    for t, c in steps:
        o.write(f"  {t:7.2f}  ncc {c:.2f}  (after: {ev_at(t)})\n")
    o.write("\nRMS per second (dBFS):\n")
    for s, r in enumerate(rms):
        o.write(f"  {s:3d}  {r:7.1f}\n")
print(open(outp).read())

# --- 2026-11-14 level-change addendum: per crossing, was a splash in progress, and where does its audio end?
WLEN = len(template("water.wav")) / sr
def env_end(t_on, span=0.8, bin_s=0.005, floor_db=-60):
    """Last 5 ms bin within [t_on, t_on+span] whose RMS is above floor_db, stopping at the next onset of either sound."""
    nxt = [t for t, _ in water + steps if t > t_on + 0.01]
    stop = min([t_on + span] + nxt)
    last = None; t = t_on
    while t < stop:
        seg = rec[int((t + SHIFT) * sr):int((t + bin_s + SHIFT) * sr)]
        if len(seg) and 20 * np.log10(max(np.sqrt(np.mean(seg ** 2)), 1e-9)) > floor_db:
            last = t + bin_s
        t += bin_s
    return last, stop
water_head = onsets(template("water.wav")[:int(0.1 * sr)], 0.8)  # first 0.1 s only: finds a splash even if it is cut short
crossings = [(t, w) for t, w in events if w.startswith("ato: link to") or w.startswith("ato: level ")]
with open(outp, "a") as o:
    o.write(f"\nlevel-change check (water.wav template {WLEN:.3f} s):\n")
    for t, w in crossings:
        o.write(f"  {t:7.3f} {w}\n")
    links = [t for t, w in events if w.startswith("ato: link to")]
    levels = [t for t, w in events if w.startswith("ato: level ")]
    for tl in links:
        tv = min([x for x in levels if x >= tl] or [float('nan')])
        o.write(f"\n  crossing: link {tl:.3f}, new level {tv:.3f}\n")
        o.write(f"    water.wav head-only (0.1 s) onsets within 2 s of the link: {[round(t, 3) for t, _ in water_head if abs(t - tl) < 2]}\n")
        prev = [(t, c) for t, c in sorted(water + [(t, c) for t, c in water_head if all(abs(t - u) > 0.05 for u, _ in water)]) if t <= tl + 0.01]
        if prev:
            t_on, c = prev[-1]
            end, stop = env_end(t_on)
            inprog = t_on <= tl < t_on + WLEN
            o.write(f"    last water onset {t_on:.3f} (ncc {c:.2f}); nominal end {t_on + WLEN:.3f}; "
                    f"in progress at link: {'YES' if inprog else 'no'}; in progress at level: {'YES' if t_on <= tv < t_on + WLEN else 'no'}\n")
            o.write(f"    audio above -60 dBFS until {end:.3f} (searched to {stop:.3f}) -> played {end - t_on:.3f} s of {WLEN:.3f} s\n" if end else "    no audio above -60 dBFS\n")
        after_w = [t for t, _ in water if t > tl + 0.01]
        after_s = [t for t, _ in steps if t > tl]
        o.write(f"    water onsets after the link: {', '.join('%.2f' % t for t in after_w) or 'none'}\n")
        o.write(f"    footstep onsets after the link: {', '.join('%.2f' % t for t in after_s) or 'none'}\n")
print(open(outp).read()[-2500:])

# --- envelope of each water.wav onset near a crossing vs the template (50 ms bins): a cut splash drops to silence early
TW = template("water.wav")
with open(outp, "a") as o:
    links = [t for t, w in events if w.startswith("ato: link to")]
    levels = [t for t, w in events if w.startswith("ato: level ")]
    o.write("\nenvelope check: rec RMS / (fitted gain x template RMS) per 50 ms bin; footstep onsets marked *\n")
    for tl in links[1:]:
        tv = min([x for x in levels if x >= tl] or [tl])
        for t_on, c in water:
            if not (tl - 0.6 <= t_on <= tv + 0.6):
                continue
            a = int((t_on + SHIFT) * sr); seg = rec[a:a + len(TW)]
            g = float(np.dot(seg, TW) / np.dot(TW, TW))
            o.write(f"  onset {t_on:.3f} (link {tl:.3f}, level {tv:.3f}) gain {g:.3f}:\n   ")
            for b in range(0, len(TW) + int(0.15 * sr), int(0.05 * sr)):
                r = rec[a + b:a + b + int(0.05 * sr)]; tp = TW[b:b + int(0.05 * sr)] if b < len(TW) else np.zeros(1)
                rr = np.sqrt(np.mean(r ** 2)) if len(r) else 0; tr = abs(g) * np.sqrt(np.mean(tp ** 2)) if len(tp) else 0
                star = "*" if any(t_on + b / sr <= s < t_on + (b + 0.05 * sr) / sr for s, _ in steps) else ""
                mark = "L" if t_on + b / sr <= tl < t_on + (b + 0.05 * sr) / sr else ("V" if t_on + b / sr <= tv < t_on + (b + 0.05 * sr) / sr else "")
                o.write(f" {b / sr:.2f}:{(rr / tr if tr > 1e-6 else float('inf')) if rr > 1e-6 else 0:.2f}{mark}{star}")
            o.write("\n")
