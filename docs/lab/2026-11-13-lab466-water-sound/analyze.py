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
    e = np.convolve(rec ** 2, np.ones(n), mode="valid")
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
