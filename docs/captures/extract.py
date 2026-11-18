#!/usr/bin/env python3
"""Rebuild the tables in docs/captures/ from the offline lab capture trees.

The raw lab captures for #458 (Graal3D link, walk and chat) and #466 (water
sound) used to live on the `claude/lab-458-graal3d` and
`claude/lab-466-water-sound` branches. Those branches are gone; the raw files
are kept privately (see docs/captures/README.md). This script reads an
extracted copy of those trees and writes the derived, publishable tables:
numbers, timings and packet orders, with host paths and screenshots left out.

Usage:
    python3 -I docs/captures/extract.py <lab-458 tree> <lab-466 tree> <out dir>

Each tree is the root that holds `docs/lab/...` from the branch tip.
"""
from __future__ import annotations

import re
import statistics
import sys
from pathlib import Path

HOME_RE = re.compile(r'/home/[A-Za-z0-9_.-]+(/[^\s]*)?')


def scrub(s: str) -> str:
    """Drop host paths; keep the file name only."""
    return HOME_RE.sub(lambda m: '<lab>/' + (m.group(1) or '').rsplit('/', 1)[-1], s)


def write(path: Path, rows: list[list[object]], header: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('w', encoding='utf-8', newline='\n') as f:
        f.write('\t'.join(header) + '\n')
        for r in rows:
            f.write('\t'.join(str(x) for x in r) + '\n')


# ---------------------------------------------------------------- lab 458

def ato_steps(path: Path) -> list[tuple[float, str]]:
    out = []
    for line in path.read_text(encoding='utf-8', errors='replace').splitlines():
        m = re.match(r'\s*([0-9.]+)\s+(.*)', line)
        if m:
            out.append((float(m.group(1)), m.group(2)))
    return out


MILESTONES = [
    ('connection established', r'3d: Connection established'),
    ('MissionStartPhase1', r'3d: MissionStartPhase1'),
    ('first ghost', r'3d: ghost 0 '),
    ('level board read', r'^level '),
    ('Sky ghost', r'3d: ghost \d+ Sky'),
    ('player ghost', r'3d: ghost \d+ GameObject G3DPlayer_'),
    ('Camera ghost', r'3d: ghost \d+ Camera'),
    ('Sky textures in', r"Sky's 6 textures are in"),
    ('MissionStartPhase2', r'3d: MissionStartPhase2'),
    ('Mission lighting done', r'Mission lighting done'),
    ('control object', r'3d: control object is ghost'),
    ('login step 6', r'login step 6'),
]


def lab458(root: Path, out: Path) -> dict[str, object]:
    lab = root / 'docs/lab'
    runs = {
        'run1': lab / '2026-11-13-lab458-graal3d/ato-stdout-timed.txt',
        'run2': lab / '2026-11-13-lab458-graal3d/run2-ato-stdout-timed.txt',
        'run3': lab / '2026-11-13-lab458-graal3d-run3/ato-stdout-timed.txt',
    }
    steps = {k: ato_steps(v) for k, v in runs.items()}
    rows = []
    for name, pat in MILESTONES:
        row: list[object] = [name]
        for k in runs:
            t = next((f'{t:.3f}' for t, s in steps[k] if re.search(pat, s)), '-')
            row.append(t)
        rows.append(row)
    write(out / 'lab458-graal3d/login-milestones.tsv', rows, ['milestone', *runs])

    # Every 3d: line of run 3, the pass that reached step 6.
    rows = [[f'{t:.3f}', scrub(s)] for t, s in steps['run3'] if s.startswith('3d:') or 'requesting' in s]
    write(out / 'lab458-graal3d/run3-3d-timeline.tsv', rows, ['t_s', 'ato_line'])

    # Player props as the 5.12 lab account received them (run 3).
    rows = []
    for t, s in steps['run3']:
        m = re.match(r'\s*(\d+) ([A-Z0-9?]+): ([0-9a-f]+)(?: = (\d+))?(?: "(.*)")?$', s)
        if m:
            rows.append([m.group(1), m.group(2), m.group(3), m.group(4) or '', m.group(5) or ''])
    write(out / 'lab458-graal3d/run3-player-props.tsv', rows, ['id', 'name', 'raw_hex', 'value', 'text'])

    # TCP packet order of a 5.12 Graal3D login, from the gserver packet dump.
    dump = lab / '2026-11-13-lab458-graal3d-run3/gserver-networkdump.txt'
    rows = []
    hdr = re.compile(r'^\[[^]]*\] ([<>]) (?:(Login) Packet from \S+|(?:Out|In) Packet (?:to|from) \S+ \[ID \d+\]: \[(\d+)( RAW)?\] (\S+)) \((\d+) bytes\)')
    n = 0
    for line in dump.read_text(encoding='utf-8', errors='replace').splitlines():
        m = hdr.match(line)
        if not m:
            continue
        n += 1
        d = 'C->S' if m.group(1) == '>' else 'S->C'
        if m.group(2):
            rows.append([n, d, '-', 'login', '', m.group(6)])
        else:
            rows.append([n, d, m.group(3), m.group(5), 'raw' if m.group(4) else '', m.group(6)])
    write(out / 'lab458-graal3d/run3-tcp-packet-order.tsv', rows, ['n', 'dir', 'id', 'name', 'framing', 'bytes'])

    # Ghost creation as the gserver logged it.
    rows = []
    for line in (lab / '2026-11-13-lab458-graal3d-run3/gserver-console.txt').read_text(errors='replace').splitlines():
        m = re.search(r'ghost (\d+) \((\S+) ?([^)]*)\) created on the client \(seq (\d+)\)', line)
        if m:
            rows.append([m.group(1), m.group(2), re.sub(r'_\w+$', '_<account>', m.group(3).strip()) or '-', m.group(4)])
        m = re.search(r'control object \(ghost (\d+)\) (sent|delivered) in seq (\d+)|control object delivered \(seq (\d+)\)', line)
        if m:
            rows.append([m.group(1) or '4', 'control object', m.group(2) or 'delivered', m.group(3) or m.group(4)])
    write(out / 'lab458-graal3d/run3-ghost-seq.tsv', rows, ['ghost', 'class', 'name_or_event', 'udp_seq'])

    # Pass 2: client sections decoded from the UDP capture.
    p2 = lab / '2026-11-14-lab458-pass2'
    sec_re = re.compile(r'^(\d+\.\d+) \+([0-9.]+) seq (\d+) fp (\d+) pos \(([-0-9.]+), ([-0-9.]+), ([-0-9.]+)\)(?: control (\d+) update: (.*))?$')
    xf_re = re.compile(r'\[5 transform pos ([-0-9.]+) ([-0-9.]+) ([-0-9.]+) axis ([-0-9.]+) ([-0-9.]+) ([-0-9.]+) angle ([-0-9.]+)\]')
    packets = []
    for line in (p2 / 'pcap-client-sections.txt').read_text().splitlines():
        m = sec_re.match(line)
        if not m:
            continue
        rel = float(m.group(2))
        upd = m.group(9) or ''
        kinds = re.findall(r'\[(\d+) ', upd)
        xf = xf_re.search(upd)
        chat = re.search(r'\[9 chat "([^"]*)"\]', upd)
        packets.append({
            'rel': rel, 'seq': int(m.group(3)), 'fp': int(m.group(4)),
            'pos': tuple(float(m.group(i)) for i in (5, 6, 7)),
            'sections': ','.join(kinds) if kinds else ('empty' if upd == 'empty' else '-'),
            'xf': xf, 'chat': chat.group(1) if chat else '',
            'flags': (re.search(r'\[13 flags (0x[0-9a-f]+)\]', upd) or [None, ''])[1] if '[13' in upd else '',
        })
    rows = []
    for p in packets:
        xf = p['xf']
        rows.append([f"{p['rel']:.3f}", p['seq'], p['fp'], *(f'{v:.3f}' for v in p['pos']), p['sections'],
                     *( [xf.group(i) for i in range(4, 8)] if xf else ['', '', '', '']), p['flags'], p['chat']])
    write(out / 'lab458-graal3d/pass2-client-packets.tsv', rows,
          ['t_s', 'seq', 'fp', 'x', 'y', 'z', 'sections', 'axis_x', 'axis_y', 'axis_z', 'angle', 'flags', 'chat'])

    # Packet cadence while moving vs idle.
    xfp = [p for p in packets if p['xf'] and p['seq'] > 4]
    gaps = [b['rel'] - a['rel'] for a, b in zip(xfp, xfp[1:]) if b['rel'] - a['rel'] < 0.5]
    idle = [p for p in packets if p['sections'] == 'empty']
    idle_gaps = [b['rel'] - a['rel'] for a, b in zip(idle, idle[1:]) if b['seq'] == a['seq'] + 1]
    steps_y = [b['xf'] and a['xf'] and abs(float(b['xf'].group(2)) - float(a['xf'].group(2))) for a, b in zip(xfp, xfp[1:])]
    # First leg: pure +y walk.
    leg1 = [p for p in xfp if float(p['xf'].group(1)) == 0.0 and float(p['xf'].group(7)) == 0.0]
    leg1_steps = [float(b['xf'].group(2)) - float(a['xf'].group(2)) for a, b in zip(leg1, leg1[1:]) if b['seq'] == a['seq'] + 1]
    leg1_rate = [d / (b['rel'] - a['rel']) for d, a, b in zip(leg1_steps, leg1, leg1[1:]) if b['rel'] > a['rel']]
    stats = {
        'packets': len(packets),
        'transform_packets': sum(1 for p in packets if p['xf']),
        'empty_packets': len(idle),
        'chat_packets': sum(1 for p in packets if p['chat']),
        'moving_gap_median_ms': round(statistics.median(gaps) * 1000, 1),
        'moving_gap_p10_ms': round(statistics.quantiles(gaps, n=10)[0] * 1000, 1),
        'moving_gap_p90_ms': round(statistics.quantiles(gaps, n=10)[-1] * 1000, 1),
        'idle_gap_median_ms': round(statistics.median(idle_gaps) * 1000, 1) if idle_gaps else None,
        'leg1_packets': len(leg1),
        'leg1_step_median_units': round(statistics.median(leg1_steps), 4),
        'leg1_step_min_units': round(min(leg1_steps), 4),
        'leg1_step_max_units': round(max(leg1_steps), 4),
        'leg1_wire_rate_median_ups': round(statistics.median(leg1_rate), 3),
    }
    masks: dict[str, int] = {}
    for line in (p2 / 'gserver-log.txt').read_text(errors='replace').splitlines():
        m = re.search(r'G3DTRACE .*mask=(0x[0-9a-f]+) .* (s\d+)', line)
        if m:
            masks[f'{m.group(1)} {m.group(2)}'] = masks.get(f'{m.group(1)} {m.group(2)}', 0) + 1
    write(out / 'lab458-graal3d/pass2-gserver-trace-masks.tsv',
          [[k.split()[0], k.split()[1], v] for k, v in sorted(masks.items())], ['mask', 'section', 'count'])
    # Input timeline relative to the first key.
    ev = []
    t0 = None
    for line in (p2 / 'events.txt').read_text().splitlines():
        t, _, what = line.partition(' ')
        if what.startswith('down') and t0 is None:
            t0 = float(t)
        ev.append((float(t), scrub(what)))
    write(out / 'lab458-graal3d/pass2-input-timeline.tsv',
          [[f'{t - t0:+.3f}', w] for t, w in ev if not w.startswith('shot') and not w.startswith('focus')],
          ['t_from_first_key_s', 'event'])
    return stats


# ---------------------------------------------------------------- lab 466

def onsets(analysis: Path) -> list[list[object]]:
    rows = []
    sound = None
    for line in analysis.read_text(errors='replace').splitlines():
        m = re.match(r'(\S+\.wav) onsets \((\d+)\)', line)
        if m:
            sound = m.group(1)
            continue
        m = re.match(r'\s+([0-9.]+)\s+ncc ([0-9.]+)\s+\(after: (.*)\)', line)
        if m and sound:
            rows.append([sound, m.group(1), m.group(2), scrub(m.group(3))])
    return rows


def lab466(root: Path, out: Path) -> dict[str, object]:
    lab = root / 'docs/lab'
    stats: dict[str, object] = {}
    allrows = []
    for run in ('6a', '6b'):
        a = lab / f'2026-11-13-lab466-water-sound/{run}/analysis.txt'
        for r in onsets(a):
            allrows.append([f'water-sound {run}', *r])
    for run in ('r1', 'r2', 'r3', 'r4', 'r5'):
        a = lab / f'2026-11-14-lab466-level-change/{run}/analysis.txt'
        for r in onsets(a):
            allrows.append([f'level-change {run}', *r])
    write(out / 'lab466-water-sound/sound-onsets.tsv', allrows, ['run', 'sound', 't_after_beep_s', 'ncc', 'after_event'])

    # Splash period while floating (6a, float window) and footstep cadence.
    six_a = [r for r in allrows if r[0] == 'water-sound 6a']
    w = [float(r[2]) for r in six_a if r[1] == 'water.wav']
    w_float = [t for t in w if 22.5 <= t <= 27.5]
    stats['float_splash_period_s'] = round(statistics.mean(b - a for a, b in zip(w_float, w_float[1:])), 3)
    stats['float_splash_period_min_max_s'] = (round(min(b - a for a, b in zip(w_float, w_float[1:])), 3),
                                              round(max(b - a for a, b in zip(w_float, w_float[1:])), 3))
    st = [float(r[2]) for r in six_a if r[1] == 'steps2.wav']
    sgaps = sorted(round(b - a, 3) for a, b in zip(st, st[1:]) if b - a < 0.4)
    stats['step_gap_values_s'] = sorted(set(sgaps))[:8]
    stats['step_gap_median_s'] = statistics.median(sgaps) if sgaps else None

    # Crossing windows and level-change timing per run.
    rows = []
    for run in ('r1', 'r2', 'r3', 'r4', 'r5'):
        a = (lab / f'2026-11-14-lab466-level-change/{run}/analysis.txt').read_text(errors='replace')
        link = re.search(r'\n\s+([0-9.]+) ato: link to onlinestartlocal', a)
        level = re.search(r'\n\s+([0-9.]+) ato: level onlinestartlocal\.graal[^\n]*\n(?:(?!ato: level)[^\n]*\n)*?', a)
        lv = [float(x) for x in re.findall(r'\n\s+([0-9.]+) ato: level onlinestartlocal', a)]
        lt = float(link.group(1)) if link else None
        lvt = next((x for x in lv if lt and x > lt), None)
        # Refined onsets (3 decimals, lower threshold) from ncc-window.txt.
        ncc = (lab / '2026-11-14-lab466-level-change/ncc-window.txt').read_text()
        blk = ncc.split(f'== {run}: crossing window')[1].split('== ')[0]
        blk2 = ncc.split(f'== {run}: dry land')[1].split('== ')[0]
        wt = [float(x) for x in re.findall(r'\(([0-9.]+), [0-9.]+\)', blk.split('steps2.wav')[0])]
        stp = [float(t) for t, c in re.findall(r'\(([0-9.]+), ([0-9.]+)\)', blk.split('steps2.wav')[1] + blk2.split('steps2.wav')[1])
               if float(c) >= 0.8]  # weaker hits overlap the last splash
        near = [x for x in wt if lvt and abs(x - lvt) <= 0.1]
        before = [x for x in wt if lvt and x < lvt - 0.1]
        last_before = max(before) if before else None
        first_step = min((x for x in stp if lvt and x > lvt), default=None)
        rows.append([run, lt, lvt, round(lvt - lt, 3) if lt and lvt else '',
                     round(lvt - last_before, 3) if last_before else '',
                     ','.join(f'{x - lvt:+.3f}' for x in near) or '-',
                     round(first_step - lvt, 3) if first_step else ''])
    write(out / 'lab466-water-sound/level-change-crossings.tsv', rows,
          ['run', 'link_s', 'level_loaded_s', 'link_to_level_s', 'level_minus_prev_splash_s',
           'water_onsets_within_0.1s_of_level_s', 'first_step_after_level_s'])
    cut = (lab / '2026-11-14-lab466-level-change/cutfit.txt').read_text()
    rows = []
    for m in re.finditer(r'level change ([0-9.]+) = ([0-9.]+) s into the first splash\n\s+model first splash FULL[^\n]*energy ([0-9.]+)\n\s+model first splash CUT[^\n]*energy ([0-9.]+)', cut):
        rows.append([m.group(1), m.group(2), m.group(3), m.group(4), 'full' if float(m.group(3)) < float(m.group(4)) else 'cut'])
    write(out / 'lab466-water-sound/splash-cut-fit.tsv', rows,
          ['level_change_s', 'into_splash_s', 'residual_full', 'residual_cut', 'better_model'])
    return stats


def main(argv: list[str]) -> int:
    if len(argv) != 4:
        print(__doc__)
        return 2
    out = Path(argv[3])
    s1 = lab458(Path(argv[1]), out)
    s2 = lab466(Path(argv[2]), out)
    for k, v in {**s1, **s2}.items():
        print(f'{k}: {v}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
