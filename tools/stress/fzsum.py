import sys,json
for l in open(sys.argv[1]):
    try: r=json.loads(l)
    except Exception: print(l.strip()); continue
    print(f"{r['v']:<12} {r['mode']:<9} rc={r['rc']} alive={r['alive_at_end']} hang={r['term_hang']} rss={r['peak_rss_mb']}MB faults={r['faults']} panics={len(r['panics'])} | {r['last'][:100]}")
    for p in r['panics']: print('   PANIC', p.replace('\n',' / ')[:300])
