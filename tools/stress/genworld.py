#!/usr/bin/env python3
"""Generate the stress-test world for the private lab gserver. Lab only."""
import os, sys
world = sys.argv[1]; accounts = sys.argv[2]; n_accounts = int(sys.argv[3])
src = open(os.path.join(world, 'onlinestartlocal.nw')).read().splitlines()
boards = [l for l in src if l.startswith('BOARD')]

def level(name, extra):
    with open(os.path.join(world, name), 'w') as f:
        f.write('GLEVNW01\n' + '\n'.join(boards) + '\n' + extra.strip() + '\n')

def npc(x, y, script, img='-'):
    return f'NPC {img} {x} {y}\n{script.strip()}\nNPCEND\n'

# Warp ring: 8 levels, each bounces the player on after 0.3 s, alternating
# setlevel2 (script warp) with a link the player stands in.
R = 8
for i in range(R):
    nxt = f'st_r{(i+1)%R}.nw'
    extra = ''
    if i % 2 == 0:
        extra += npc(10, 10, f'if (playerenters) timeout = 0.3;\nif (timeout) setlevel2 {nxt},30,30;')
    else:
        # A link under the arrival spot: the client walks into it on arrival.
        extra += f'LINK {nxt} 28 28 4 4 30 30\n'
        extra += npc(10, 10, f'if (playerenters) timeout = 0.5;\nif (timeout) {{ setlevel2 {nxt},30,30; }}')
    extra += f'CHEST 20 24 greenrupee -1\n'
    extra += npc(40, 40, 'if (playerenters) { setplayerprop #c,ring; }')
    level(f'st_r{i}.nw', extra)

# 2x2 gmap with a script that keeps nudging the player across seams.
gm = ['st_gm_a.nw', 'st_gm_b.nw', 'st_gm_c.nw', 'st_gm_d.nw']
with open(os.path.join(world, 'stgm.gmap'), 'w') as f:
    f.write('GRMAP001\nWIDTH 2\nHEIGHT 2\nLEVELNAMES\n"st_gm_a.nw","st_gm_b.nw",\n"st_gm_c.nw","st_gm_d.nw",\nLEVELNAMESEND\n')
for i, g in enumerate(gm):
    extra = npc(30, 30, 'if (playerenters) { setplayerprop #c,gm; }')
    for k in range(6):
        extra += f'BADDY {10+k*8} {12+k*6} {k%10}\n\n\n\nBADDYEND\n'
    level(g, extra)

# Arena: baddies, chests, item and bomb churn, chat churn.
arena = ''
for k in range(12):
    arena += f'BADDY {8+(k*5)%48} {8+(k*7)%48} {k%10}\n\n\n\nBADDYEND\n'
for k, it in enumerate(['greenrupee','bluerupee','redrupee','bombs','darts','heart','glove1','bow','fullheart','superbomb','fireball','spinattack']):
    arena += f'CHEST {6+k*4} 4 {it} -1\n'
arena += npc(5, 60, '''if (playerenters||timeout) {
  lay2 greenrupee,playerx+random(-3,3),playery+random(-3,3);
  lay2 heart,playerx+random(-3,3),playery+random(-3,3);
  timeout = 0.2;
}''')
arena += npc(6, 60, '''if (playerenters||timeout) {
  putbomb 1,playerx+random(-6,6),playery+random(-6,6);
  shootarrow playerx+5,playery,0;
  putexplosion 1,playerx+random(-8,8),playery+random(-8,8);
  timeout = 0.5;
}''')
arena += npc(7, 60, '''if (playerenters||timeout) {
  this.n = this.n + 1;
  setplayerprop #c,arena #v(this.n);
  timeout = 0.1;
}''')
arena += npc(8, 60, '''if (playerenters||timeout) {
  showimg 1,sprites.png,playerx,playery;
  changeimgvis 1,4;
  hide; show; setani walk,;
  timeout = 0.05;
}''', img='block.png')
arena += 'LINK st_hub.nw 0 0 64 1 30 60\n'
level('st_arena.nw', arena)

hub = npc(30, 20, 'if (playerenters) { setplayerprop #c,hub; }')
hub += 'LINK st_arena.nw 0 63 64 1 30 2\nLINK st_r0.nw 63 0 1 64 30 30\n'
for k in range(20):
    hub += f'CHEST {4+k*3} {50 + (k%3)*3} greenrupee -1\n'
level('st_hub.nw', hub)

# Accounts: roles by start level.
roles = [('st_r0.nw', 30, 30), ('st_gm_a.nw', 60, 60), ('st_arena.nw', 30, 30), ('st_hub.nw', 30, 30)]
tmpl = open(os.path.join(accounts, 'defaultaccount.txt')).read()
for n in range(n_accounts):
    lv, x, y = roles[n % len(roles)]
    t = tmpl.replace('NAME default', f'NAME stress{n:02d}').replace('NICK default', f'NICK stress{n:02d}')
    t = t.replace('LEVEL onlinestartlocal.nw', f'LEVEL {lv}').replace('X 30.00', f'X {x}.00').replace('Y 30.50', f'Y {y}.00')
    t = t.replace('MAXHP 3.00', 'MAXHP 20.00').replace('HP 3.00', 'HP 20.00')
    open(os.path.join(accounts, f'stress{n:02d}.txt'), 'w').write(t)
# Many distinct levels for long sessions (level-cache growth): st_w0..st_w199.
for k in range(200):
    extra = npc(30, 20, f'if (playerenters) {{ setplayerprop #c,w{k}; }}')
    extra += f'CHEST {5+k%50} {5+k%40} greenrupee -1\n'
    extra += f'BADDY {10+k%40} {40} {k%10}\n\n\n\nBADDYEND\n'
    extra += f'LINK st_w{(k+1)%200}.nw 63 0 1 64 1 30\nLINK st_w{(k-1)%200}.nw 0 0 1 64 62 30\n'
    level(f'st_w{k}.nw', extra)
print('ok')
