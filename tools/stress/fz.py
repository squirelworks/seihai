#!/usr/bin/env python3
"""Bad-packet runs: Seihai behind the fault proxy. Loopback only.

fz.py OUT VERSIONS MODES [--dur S] [--p P] [--seeds N]
Each (version, mode, seed) is one client session of up to --dur seconds.
"""
import argparse, json, os, re, subprocess, time

SEIHAI = os.path.join(os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')), 'target', 'release', 'seihai')
PROXY = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'faultproxy', 'target', 'release', 'faultproxy')
ap = argparse.ArgumentParser()
ap.add_argument('out'); ap.add_argument('versions'); ap.add_argument('modes')
ap.add_argument('--dur', type=float, default=40)
ap.add_argument('--p', type=float, default=0.1)
ap.add_argument('--seeds', type=int, default=2)
ap.add_argument('--after', type=float, default=6)
ap.add_argument('--display', type=int, default=91)
ap.add_argument('--port', type=int, default=15803)
ap.add_argument('--acct', type=int, default=2)
ap.add_argument('--bin', default=SEIHAI)
a = ap.parse_args()
os.makedirs(a.out, exist_ok=True)
res = open(os.path.join(a.out, 'results.jsonl'), 'a')

def rss(pid):
    try:
        s = open(f'/proc/{pid}/status').read()
        return int(re.search(r'VmRSS:\s+(\d+)', s).group(1))
    except (OSError, AttributeError):
        return 0

xvfb = subprocess.Popen(['Xvfb', f':{a.display}', '-screen', '0', '1024x768x24', '-ac', '-nolisten', 'tcp'],
                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(1)
try:
    for v in a.versions.split(','):
        for mode in a.modes.split(','):
            for seed in range(1, a.seeds + 1):
                tag = f'{v.replace(" ", "_")}-{mode}-{seed}'
                d = os.path.join(a.out, tag); os.makedirs(d, exist_ok=True)
                plog = open(os.path.join(d, 'proxy.log'), 'w')
                proxy = subprocess.Popen([PROXY, str(a.port), '14803', v, mode, str(a.p), str(a.after), str(seed * 7919)],
                                         stdout=plog, stderr=subprocess.STDOUT)
                time.sleep(0.3)
                lobby = v.startswith(('5.', '6.'))
                target = '127.0.0.1:14804' if lobby else f'127.0.0.1:{a.port}'
                env = dict(os.environ, DISPLAY=f':{a.display}', HOME=d, LAB_PW='labpass-placeholder', RUST_BACKTRACE='1')
                clog = open(os.path.join(d, 'seihai.log'), 'w')
                t0 = time.time()
                p = subprocess.Popen([a.bin, '--online', target, '--account', f'stress{a.acct:02d}', '--password-env', 'LAB_PW',
                                      '--version', v, '--no-sound'], env=env, cwd=d, stdout=clog, stderr=subprocess.STDOUT)
                peak = 0; cpu0 = None
                focused = False
                while time.time() - t0 < a.dur and p.poll() is None:
                    peak = max(peak, rss(p.pid))
                    if not focused and time.time() - t0 > 3:
                        subprocess.run(['xdotool', 'mousemove', '400', '300'], env=env, stderr=subprocess.DEVNULL)
                        focused = True
                    time.sleep(0.25)
                rc = p.poll()
                alive = rc is None
                hung = False
                if alive:
                    p.terminate()
                    try:
                        p.wait(5)
                    except subprocess.TimeoutExpired:
                        hung = True; p.kill(); p.wait()
                proxy.kill(); proxy.wait(); plog.close(); clog.close()
                txt = open(os.path.join(d, 'seihai.log'), errors='replace').read()
                pan = re.findall(r"panicked at [^\n]*\n[^\n]*", txt)
                faults = sum(1 for l in open(os.path.join(d, 'proxy.log')) if re.search(r'mutated|hostile|bomb|framing|flood', l))
                r = dict(v=v, mode=mode, seed=seed, rc=rc, alive_at_end=alive, term_hang=hung, up=round(time.time() - t0, 1),
                         peak_rss_mb=round(peak / 1024), panics=pan[:3], faults=faults,
                         last=txt.strip().splitlines()[-1][:200] if txt.strip() else '')
                res.write(json.dumps(r) + '\n'); res.flush()
                print(json.dumps(r), flush=True)
finally:
    xvfb.terminate()
