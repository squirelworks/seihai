#!/usr/bin/env python3
"""Summarise one stress run: exits, panics, stalls, memory, CPU, send rate."""
import collections, glob, json, os, re, subprocess, sys

O = sys.argv[1]
S = os.path.join(os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')), 'target', 'release', 'seihai')  # decoder only
out = []
def p(*a):
    out.append(' '.join(str(x) for x in a))

events = [json.loads(l) for l in open(os.path.join(O, 'events.jsonl'))]
launch = {e['pid']: e for e in events if e['kind'] == 'launch'}
dur = max(e['t'] for e in events)
p(f'# Run {os.path.basename(O)}: {dur:.0f} s, {len(launch)} launches, '
  f'{len({e["c"] for e in launch.values()})} clients')

# --- exits -------------------------------------------------------------
unexp = [e for e in events if e['kind'] == 'exit' and not e['expected']]
hangs = [e for e in events if e['kind'] == 'term_hang']
p(f'\n## Exits\nunexpected exits: {len(unexp)}; clients that ignored SIGTERM for 10 s / 3 s: {len(hangs)}')
for e in unexp:
    p(f'  c{e["c"]:02d} {e["v"]} launch {e["n"]}: rc={e["rc"]} after {e["up"]} s')

# --- logs --------------------------------------------------------------
pat = {
    'panic': re.compile(r'panicked'),
    'stall': re.compile(r'stall|stuck', re.I),
    'warp not answered': re.compile(r'warp not answered'),
    'connection closed': re.compile(r'connection closed|disconnected', re.I),
    'error': re.compile(r'\berror\b', re.I),
}
counts = collections.Counter(); panics = []
per_version = collections.defaultdict(collections.Counter)
cl_version = {e['c']: e['v'] for e in launch.values()}
for f in sorted(glob.glob(os.path.join(O, 'c*/run*.log'))):
    c = int(os.path.basename(os.path.dirname(f))[1:])
    lines = open(f, errors='replace').read().splitlines()
    for i, l in enumerate(lines):
        for k, r in pat.items():
            if r.search(l):
                counts[k] += 1; per_version[cl_version.get(c, '?')][k] += 1
                if k == 'panic':
                    panics.append((f, '\n'.join(lines[i:i + 12])))
for f in glob.glob(os.path.join(O, 'c*/_debug/stall.log')):
    n = len(open(f).read().splitlines())
    counts['stall.log lines'] += n
p('\n## Client log lines\n' + ', '.join(f'{k}: {v}' for k, v in sorted(counts.items())) if counts else '\n## Client log lines\nnone of note')
for v, cnt in sorted(per_version.items()):
    p(f'  {v}: ' + ', '.join(f'{k} {n}' for k, n in sorted(cnt.items())))
seen = set()
for f, txt in panics:
    key = txt.splitlines()[0][:160]
    if key in seen:
        continue
    seen.add(key)
    p(f'\n### panic in {os.path.relpath(f, O)}\n```\n{txt}\n```')

# --- samples -----------------------------------------------------------
rows = collections.defaultdict(list)
ports = collections.defaultdict(set)
for l in open(os.path.join(O, 'samples.tsv')).read().splitlines()[1:]:
    t, c, v, pid, cpu, rss, thr, fds, pr = (l.split('\t') + [''])[:9]
    rows[int(pid)].append((float(t), float(cpu), int(rss), int(thr), int(fds)))
    for x in filter(None, pr.split(',')):
        ports[int(pid)].add(int(x.split('>')[0]))

def slope(pts):
    n = len(pts)
    if n < 3:
        return 0.0
    mx = sum(x for x, _ in pts) / n; my = sum(y for _, y in pts) / n
    d = sum((x - mx) ** 2 for x, _ in pts)
    return sum((x - mx) * (y - my) for x, y in pts) / d if d else 0.0

p('\n## Per launch (memory MB, CPU % of one core, threads, fds)')
p('client | version | up s | RSS start | RSS end | RSS max | RSS slope MB/min (after 30 s) | CPU avg | CPU max (2 s) | threads max | fds start→end')
agg = collections.defaultdict(list)
for pid, r in sorted(rows.items(), key=lambda kv: (launch.get(kv[0], {}).get('c', 0), kv[0])):
    e = launch.get(pid, {})
    if len(r) < 3:
        continue
    up = r[-1][0] - r[0][0]
    late = [(t, rss / 1024) for t, _, rss, _, _ in r if t - r[0][0] >= 30]
    sl = slope(late) * 60
    cpus = [(r[i][1] - r[i - 1][1]) / max(r[i][0] - r[i - 1][0], 0.1) * 100 for i in range(1, len(r))]
    avg = (r[-1][1] - r[0][1]) / max(up, 0.1) * 100
    p(f'c{e.get("c", -1):02d} | {e.get("v", "?")} | {up:.0f} | {r[0][2]/1024:.0f} | {r[-1][2]/1024:.0f} | '
      f'{max(x[2] for x in r)/1024:.0f} | {sl:+.2f} | {avg:.0f} | {max(cpus):.0f} | {max(x[3] for x in r)} | {r[0][4]}→{r[-1][4]}')
    agg[e.get('v', '?')].append((up, sl, avg, max(cpus), max(x[2] for x in r) / 1024))

p('\n## Per version')
p('version | launches | longest up s | worst RSS slope MB/min (launches up ≥ 120 s) | mean CPU % | max CPU % | max RSS MB')
for v, a in sorted(agg.items()):
    long = [x[1] for x in a if x[0] >= 120]
    p(f'{v} | {len(a)} | {max(x[0] for x in a):.0f} | {max(long) if long else float("nan"):+.2f} | '
      f'{sum(x[2] for x in a)/len(a):.0f} | {max(x[3] for x in a):.0f} | {max(x[4] for x in a):.0f}')

# --- send rate from the pcap ------------------------------------------
pcap = os.path.join(O, 'lo.pcap')
if os.path.exists(pcap) and '--no-pcap' not in sys.argv:
    p('\n## Client send rate (decoded C>S messages, per 1 s window)')
    p('client | version | port | msgs | max msgs/s | top id in the peak second | max bytes/s | unknown/undecoded msgs')
    rates = collections.defaultdict(list)
    tmp = os.path.join(O, 'split'); os.makedirs(tmp, exist_ok=True)
    for pid, prt in sorted(ports.items(), key=lambda kv: (launch.get(kv[0], {}).get('c', 0), kv[0])):
        e = launch.get(pid, {})
        for port in sorted(prt):
            part = os.path.join(tmp, f'{port}.pcap')
            subprocess.run(['tcpdump', '-r', pcap, '-w', part, f'tcp port {port}'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            tsv = part + '.tsv'
            if os.path.exists(tsv):
                os.remove(tsv)
            subprocess.run([S, 'capture', 'show', part, '--dir', 'c2s', '--format', 'tsv', '--out', tsv],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if not os.path.exists(tsv):
                continue
            sec = collections.Counter(); secb = collections.Counter(); ids = collections.defaultdict(collections.Counter)
            unk = n = 0
            for l in open(tsv, errors='replace'):
                if l.startswith('#'):
                    continue
                f = l.rstrip('\n').split('\t')
                if len(f) < 8:
                    continue
                n += 1
                s = int(f[2]) // 1000
                sec[s] += 1; secb[s] += int(f[6] or 0); ids[s][f[5]] += 1
                if f[7] != 'decoder' and f[7] != 'table' or (len(f) > 10 and f[10]):
                    unk += 1
            if not sec:
                continue
            pk = max(sec, key=sec.get)
            top = ids[pk].most_common(1)[0]
            p(f'c{e.get("c", -1):02d} | {e.get("v", "?")} | {port} | {n} | {sec[pk]} | {top[0]} ×{top[1]} | {max(secb.values())} | {unk}')
            rates[e.get('v', '?')].append((sec[pk], max(secb.values()), unk))
    p('\nversion | worst msgs/s | worst bytes/s | unknown msgs')
    for v, a in sorted(rates.items()):
        p(f'{v} | {max(x[0] for x in a)} | {max(x[1] for x in a)} | {sum(x[2] for x in a)}')

# --- server ------------------------------------------------------------
h = open(os.path.join(O, 'harness.log')).read()
p('\n## Servers\n' + ('GSERVER DIED during the run' if 'GSERVER DIED' in h else 'both lab gservers stayed up'))
if h.strip():
    p('harness log tail:\n```\n' + '\n'.join(h.strip().splitlines()[-10:]) + '\n```')
open(os.path.join(O, 'report.md'), 'w').write('\n'.join(out) + '\n')
print('\n'.join(out))
