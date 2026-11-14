"""ncc_window.py RUN SOUNDS T0 T1: normalised xcorr of water.wav / steps2.wav against the recording, peaks >= 0.3 in [T0,T1] (s after beep)."""
import sys, wave, numpy as np
from scipy.signal import resample_poly
run, snd, t0, t1 = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4])
def load(p):
    w = wave.open(p); n, ch, sw, sr = w.getnframes(), w.getnchannels(), w.getsampwidth(), w.getframerate(); r = w.readframes(n)
    a = (np.frombuffer(r, np.uint8).astype(float) - 128) / 128 if sw == 1 else np.frombuffer(r, np.int16).astype(float) / 32768
    return a.reshape(-1, ch).mean(1), sr
rec, sr = load(f"{run}/rec.wav"); sh = float(open(f"{run}/analysis.txt").readline().split()[3])
for name in ["water.wav", "steps2.wav"]:
    t, s = load(f"{snd}/{name}"); t = resample_poly(t, sr // 100, s // 100) if s != sr else t; t = t - t.mean(); n = len(t)
    a = int((t0 + sh) * sr); b = int((t1 + sh) * sr); seg = rec[a:b + n]
    num = np.correlate(seg, t, "valid"); c2 = np.concatenate([[0], np.cumsum(seg ** 2)]); e = c2[n:] - c2[:-n]
    ncc = num / (np.sqrt(np.maximum(e, 1e-12)) * np.linalg.norm(t))
    pk = [i for i in range(1, len(ncc) - 1) if ncc[i] >= 0.3 and ncc[i] == ncc[max(0, i - 2000):i + 2000].max()]
    print(name, [(round(t0 + i / sr, 3), round(float(ncc[i]), 2)) for i in pk])
