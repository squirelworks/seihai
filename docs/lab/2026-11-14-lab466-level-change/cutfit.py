"""cutfit.py RUN SOUNDS A1 A2 TV: two overlapping water.wav onsets near A1, A2 (s after beep); level change at TV.
Fits rec = g1*T(A1 [full | cut at TV]) + g2*T(A2) + steps2 at any onsets given in STEPS env, by least squares; reports residual energy of each model."""
import sys, os, wave, numpy as np
from scipy.signal import resample_poly
run, snd = sys.argv[1], sys.argv[2]; A1, A2, TV = map(float, sys.argv[3:6])
def load(p):
    w = wave.open(p); n, ch, sw, sr = w.getnframes(), w.getnchannels(), w.getsampwidth(), w.getframerate(); r = w.readframes(n)
    a = (np.frombuffer(r, np.uint8).astype(float) - 128) / 128 if sw == 1 else np.frombuffer(r, np.int16).astype(float) / 32768
    return a.reshape(-1, ch).mean(1), sr
rec, sr = load(f"{run}/rec.wav"); sh = float(open(f"{run}/analysis.txt").readline().split()[3])
def tpl(name):
    t, s = load(f"{snd}/{name}"); t = resample_poly(t, sr // 100, s // 100) if s != sr else t; return t - t.mean()
T = tpl("water.wav"); S = tpl("steps2.wav"); n = len(T)
def refine(a):  # best sample alignment within +-10 ms
    i0 = int((a + sh) * sr); best = max(range(i0 - 441, i0 + 442), key=lambda i: np.dot(rec[i:i + n], T) / (np.linalg.norm(rec[i:i + n]) + 1e-12)); return best
i1, i2 = refine(A1), refine(A2); iv = int((TV + sh) * sr)
lo, hi = i1 - 2205, max(i1, i2) + n + 2205; y = rec[lo:hi]
steps = [int((float(x) + sh) * sr) for x in os.environ.get("STEPS", "").split()]
def col(sig, at, cut=None):
    c = np.zeros(hi - lo); s = sig.copy()
    if cut is not None: s[max(0, cut - at):] = 0
    a = at - lo; b = min(hi - lo, a + len(s)); c[a:b] = s[:b - a]; return c
def fit(cut):
    X = np.stack([col(T, i1, iv if cut else None), col(T, i2)] + [col(S, j) for j in steps], 1)
    g, *_ = np.linalg.lstsq(X, y, rcond=None); r = y - X @ g
    return g, float(np.sum(r ** 2) / np.sum(y ** 2))
print(f"onsets refined: {(i1 / sr - sh):.4f}, {(i2 / sr - sh):.4f}; level change {TV:.3f} = {(iv - i1) / sr:.3f} s into the first splash")
for cut in (False, True):
    g, rr = fit(cut); print(f"  model first splash {'CUT at level change' if cut else 'FULL 0.438 s':22s}: gains {np.round(g, 3).tolist()}, residual/signal energy {rr:.4f}")
