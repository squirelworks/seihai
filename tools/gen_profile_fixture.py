#!/usr/bin/env python3
"""Regenerate crates/seihai-core/src/protocol/profile_toml.tsv from the Graal_analysis version profiles.

usage: tools/gen_profile_fixture.py PATH/TO/Graal_analysis > crates/seihai-core/src/protocol/profile_toml.tsv

The TSV is the cleaned per-version data (ids and numbers only) that the unit tests in
`profile.rs` compare every `PROFILES` row against. Inputs, all in Graal_analysis:
  client-internals/version-profiles/v133-v139.toml   ([[version]] list, 1.33-1.39)
  client-internals/version-profiles/141-217.toml     (inheriting tables, 1.41-2.17; flattened
                                                      with that directory's resolve.py)
  version-profiles/2.22-2.31.toml                    ([[version]] list, 2.22/2.3/2.31)
  version-profiles/3.04-3.153.toml                   ([[version]] list, 3.04/3.153)
Needs Python 3.11 (tomllib). Columns:
  label token basis plo pli own_max npc_max other tamper list_port list_host list_kind list_status
  built_in_host built_in_port
Id lists are comma-separated ranges; `-` means the TOML gives no value. The last five columns
are the listserver endpoint rule (host always listserver.graal.in, legacy 14923, supported 14922,
confirmed or assumed) and the host and port the binary carries (history only).
"""
import importlib.util, re, sys, tomllib
from pathlib import Path

root = Path(sys.argv[1])
vp = root / 'client-internals/version-profiles'
spec = importlib.util.spec_from_file_location('resolve', vp / 'resolve.py')
resolve = importlib.util.module_from_spec(spec)
spec.loader.exec_module(resolve)


def expand(v):
    """List of ints, or a '0-24, 26' string / 'a-b (note)' string, to a sorted id list."""
    if v is None:
        return None
    if isinstance(v, list):
        return sorted(v)
    out = []
    for part in re.sub(r'\(.*?\)', '', v).split(','):
        part = part.strip()
        if not part:
            continue
        m = re.fullmatch(r'(\d+)(?:-(\d+))?', part)
        if not m:
            raise ValueError(f'cannot parse id list {v!r}')
        out += range(int(m[1]), int(m[2] or m[1]) + 1)
    return sorted(set(out))


def ranges(ids):
    if ids is None:
        return '-'
    out, run = [], []
    for i in ids:
        if run and i == run[-1] + 1:
            run.append(i)
        else:
            if run:
                out.append(run)
            run = [i]
    if run:
        out.append(run)
    return ','.join(str(r[0]) if len(r) == 1 else f'{r[0]}-{r[-1]}' for r in out)


def last(v):
    ids = expand(v)
    return str(ids[-1]) if ids else '-'


rows = []

# 1.21-1.32. Their pli_sent lists only the ids passed as constants, a lower
# bound: written with a leading ">" so the check reads it as a subset.
for v in tomllib.load(open(vp / 'v121-v132.toml', 'rb'))['version']:
    rows.append(dict(label=v['label'], token=v['token'], basis='bin',
                     plo=v['plo_known'], pli='>' + ranges(v['pli_sent']), own=last(v['plprops_known']),
                     npc=last(v['npcprops_known']), other=None,
                     tamper=0, port='-', ls={}))

# 1.33-1.39
for v in tomllib.load(open(vp / 'v133-v139.toml', 'rb'))['version']:
    rows.append(dict(label=v['label'], token=v['token'], basis='bin',
                     plo=v['plo_known'], pli=v['pli_sent'], own=last(v['plprops_known']),
                     npc=last(v['npcprops_known']), other=v['otherplprops_known'],
                     tamper=0, port='-', ls={}))

# 1.41-2.17
by_id = {p['id']: p for p in resolve.resolve(tomllib.load(open(vp / '141-217.toml', 'rb')))}
ids141 = [('v141r1', '1.41'), ('v142', '1.42'), ('v200', '2.00'), ('v201', '2.01'), ('v202', '2.02'),
          ('v202r2', '2.02 rev2'), ('v203', '2.03'), ('v204', '2.04'), ('v205', '2.05'),
          ('v21', '2.1'), ('v212', '2.12'), ('v213', '2.13'), ('v214', '2.14'), ('v215', '2.15'),
          ('v216', '2.16'), ('v217', '2.17')]
for tid, label in ids141:
    p = by_id[tid]
    inferred = any(str(t).startswith('inf') for k, t in p['src'].items() if k in ('plo_known', 'pli_sent'))
    npc = p.get('npcprops_parse')
    ls = p.get('listserver', {})
    tc = p.get('tampercheck', False)
    rows.append(dict(label=label, token=p['token'], basis='inf' if inferred else 'bin',
                     plo=p['plo_known'], pli=p['pli_sent'], own=str(p['props_max']),
                     npc=last(npc), other=p.get('otherprops_parse') if isinstance(p.get('otherprops_parse'), list) else None,
                     tamper=1 if tc else 0, port=str(ls.get('port', '-')), ls=ls))

# 2.22, 2.31 and 2.3 (2.3 is "2.31 without the fixes")
doc5 = tomllib.load(open(root / 'version-profiles/2.22-2.31.toml', 'rb'))
g5 = {v['label']: v for v in doc5['version']}
d5 = doc5['defaults']['gen5']
for label in ('2.22', '2.3', '2.31'):
    v = g5[label]
    src = g5['2.31'] if label == '2.3' else v
    rows.append(dict(label=label, token=v['token'], basis='inf' if v.get('inferred') else 'bin',
                     plo=src['plo_known'], pli=src['pli_sent'], own=str(src['props_max']),
                     npc=str(src['npcprops_max']), other=None, tamper=1 if 'tampercheck' in src else 0,
                     port=str(d5['listserver']['port']),
                     ls={**d5['listserver'], **({'port_status': v['listserver_port_status']} if 'listserver_port_status' in v else {})}))

# 3.04 and 3.153 (own_max is the highest prop each parses for itself, as Seihai's own set)
doc3 = tomllib.load(open(root / 'version-profiles/3.04-3.153.toml', 'rb'))
d3 = doc3['defaults']['g3d']
for v in doc3['version']:
    rows.append(dict(label=v['label'], token=v['token'], basis='bin',
                     plo=v['plo_known'], pli=v['pli_sent'], own=str(v['props_parse_max']),
                     npc=str(v['npcprops_max']), other=None, tamper=1 if d3.get('tampercheck') else 0,
                     port=str(d3['listserver']['port']), ls=d3['listserver']))

print('# generated by tools/gen_profile_fixture.py from Graal_analysis version profiles; do not edit')
print('# label\ttoken\tbasis\tplo\tpli\town_max\tnpc_max\tother\ttamper\tlist_port\tlist_host\tlist_kind\tlist_status\tbuilt_in_host\tbuilt_in_port')
for r in rows:
    print('\t'.join([r['label'], r['token'], r['basis'], ranges(expand(r['plo'])), (r['pli'] if isinstance(r['pli'], str) else ranges(expand(r['pli']))),
                     r['own'], r['npc'], ranges(expand(r['other'])), str(r['tamper']), r['port'],
                     r['ls'].get('host', '-'), r['ls'].get('kind', '-'), r['ls'].get('port_status', '-'),
                     r['ls'].get('built_in_host', '-'), str(r['ls'].get('built_in_port', '-'))]))
