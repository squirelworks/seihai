#!/usr/bin/env python3
"""Seihai stress harness. Private lab only: every client connects to 127.0.0.1.

Runs N Seihai clients per version, each on its own Xvfb display and HOME,
drives them with xdotool (walk, sword, action, chat warps), churns
reconnects (TERM, KILL, quick relaunch), samples RSS/CPU/threads/fds per
process, records client ports for the pcap, and collects exits and panics.
"""
import argparse, json, os, random, re, signal, subprocess, sys, threading, time

SEIHAI = os.path.join(os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')), 'target', 'release', 'seihai')
LOBBY = ('5.', '6.')       # these log in through the Login lobby (14801)
HOST = '127.0.0.1'         # hard rule: loopback only

ap = argparse.ArgumentParser()
ap.add_argument('--versions', required=True)          # comma list
ap.add_argument('--per-version', type=int, default=2)
ap.add_argument('--duration', type=int, default=300)
ap.add_argument('--out', required=True)
ap.add_argument('--churn', type=float, default=0.0)  # reconnects per client per minute
ap.add_argument('--first-account', type=int, default=0)
ap.add_argument('--display-base', type=int, default=50)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--no-input', action='store_true')
ap.add_argument('--extra', default='')
ap.add_argument('--many-levels', action='store_true')
ap.add_argument('--bin', default=SEIHAI)
args = ap.parse_args()
random.seed(args.seed)
os.makedirs(args.out, exist_ok=True)
T0 = time.time()
stop = threading.Event()
lock = threading.Lock()
events = open(os.path.join(args.out, 'events.jsonl'), 'a')

def ev(**k):
    k['t'] = round(time.time() - T0, 2)
    with lock:
        events.write(json.dumps(k) + '\n'); events.flush()

def socket_ports(pid):
    """Local ports of the pid's TCP sockets to the lab servers."""
    try:
        inodes = set()
        for fd in os.listdir(f'/proc/{pid}/fd'):
            try:
                l = os.readlink(f'/proc/{pid}/fd/{fd}')
            except OSError:
                continue
            if l.startswith('socket:['):
                inodes.add(l[8:-1])
        ports = []
        for line in open('/proc/net/tcp').read().splitlines()[1:]:
            f = line.split()
            if f[9] in inodes:
                rport = int(f[2].split(':')[1], 16)
                if rport in (14801, 14802):
                    ports.append((int(f[1].split(':')[1], 16), rport))
        return ports
    except OSError:
        return []

def proc_stats(pid):
    try:
        st = open(f'/proc/{pid}/stat').read().rsplit(')', 1)[1].split()
        cpu = (int(st[11]) + int(st[12])) / os.sysconf('SC_CLK_TCK')
        status = open(f'/proc/{pid}/status').read()
        rss = int(re.search(r'VmRSS:\s+(\d+)', status).group(1))
        thr = int(re.search(r'Threads:\s+(\d+)', status).group(1))
        fds = len(os.listdir(f'/proc/{pid}/fd'))
        return cpu, rss, thr, fds
    except (OSError, AttributeError, ValueError):
        return None

class Client:
    def __init__(self, idx, version, account):
        self.idx, self.version, self.account = idx, version, account
        self.role = account % 4          # 0 ring, 1 gmap, 2 arena, 3 hub
        self.disp = args.display_base + idx
        self.home = os.path.join(args.out, f'c{idx:02d}')
        os.makedirs(self.home, exist_ok=True)
        self.proc = None
        self.launches = 0
        self.port = 14801 if version.startswith(LOBBY) else 14802
        self.xvfb = subprocess.Popen(['Xvfb', f':{self.disp}', '-screen', '0', '1024x768x24', '-ac', '-nolisten', 'tcp'],
                                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def launch(self):
        self.launches += 1
        log = open(os.path.join(self.home, f'run{self.launches:03d}.log'), 'w')
        env = dict(os.environ, DISPLAY=f':{self.disp}', HOME=self.home, LAB_PW='labpass-placeholder', RUST_BACKTRACE='1')
        cmd = [args.bin, '--online', f'{HOST}:{self.port}', '--account', f'stress{self.account:02d}',
               '--password-env', 'LAB_PW', '--version', self.version, '--no-sound'] + args.extra.split()
        assert cmd[2].startswith('127.0.0.1:')
        self.proc = subprocess.Popen(cmd, env=env, cwd=self.home, stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
        self.started = time.time()
        ev(kind='launch', c=self.idx, v=self.version, acct=self.account, pid=self.proc.pid, n=self.launches)

    def xd(self, *a):
        subprocess.run(['xdotool', *a], env=dict(os.environ, DISPLAY=f':{self.disp}'),
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def hold(self, key, secs):
        self.xd('keydown', key); time.sleep(secs); self.xd('keyup', key)

    def tap(self, key):
        self.hold(key, 0.08)

    def chat(self, text):
        self.hold('Return', 0.15); time.sleep(0.3)
        self.xd('type', '--delay', '20', text); time.sleep(0.2)
        self.hold('Return', 0.15)

    def drive(self):
        arrows = ['Left', 'Right', 'Up', 'Down']
        while not stop.is_set():
            if self.proc is None or self.proc.poll() is not None or time.time() - self.started < 4:
                time.sleep(0.5); continue
            if getattr(self, 'focused', None) != self.launches:
                # No window manager: the pointer gives the focus (an unfocused Seihai pauses).
                self.xd('mousemove', '400', '300'); self.focused = self.launches
                self.xd('search', '--pid', str(self.proc.pid), 'windowfocus')
            if not getattr(self, 'snapped', False) and time.time() - self.started > 20:
                self.snapped = True
                subprocess.run(['import', '-display', f':{self.disp}', '-window', 'root',
                                os.path.join(self.home, 'shot.png')], stderr=subprocess.DEVNULL)
            r = random.random()
            if self.role == 0:      # warps: chat warps around the ring, short walks
                if r < 0.5 and args.many_levels:
                    self.chat(f'warpto 30 30 st_w{random.randrange(200)}.nw')
                elif r < 0.5:
                    self.chat(f'warpto 30 30 st_r{random.randrange(8)}.nw')
                elif r < 0.6:
                    self.chat('warpto 30 30 st_arena.nw')
                else:
                    self.hold(random.choice(arrows), random.uniform(0.1, 0.8))
                time.sleep(random.uniform(0.2, 1.5))
            elif self.role == 1:    # gmap seams
                if r < 0.1:
                    self.chat('warpto 62 62 st_gm_a.nw')
                else:
                    self.hold(random.choice(arrows), random.uniform(1.0, 4.0))
            elif self.role == 2:    # combat and items
                if r < 0.45:
                    self.tap(random.choice(['s', 'space']))
                elif r < 0.6:
                    self.tap('a')
                elif r < 0.65:
                    self.chat('warpto 30 30 st_arena.nw')
                else:
                    self.hold(random.choice(arrows), random.uniform(0.1, 1.2))
                time.sleep(random.uniform(0.05, 0.4))
            else:                   # hub: chat spam, walks through links
                if r < 0.4:
                    self.chat('stress ' + ''.join(random.choice('abcdefghij ') for _ in range(random.randint(1, 60))))
                elif r < 0.5:
                    self.chat('warpto 30 30 st_hub.nw')
                else:
                    self.hold(random.choice(arrows), random.uniform(0.3, 3.0))
                time.sleep(random.uniform(0.1, 1.0))

clients = []
versions = [v.strip() for v in args.versions.split(',') if v.strip()]
acct = args.first_account
for v in versions:
    for k in range(args.per_version):
        clients.append(Client(len(clients), v, acct)); acct += 1
time.sleep(1.5)
for c in clients:
    c.launch(); time.sleep(0.3)

def reaper():
    while not stop.is_set():
        for c in clients:
            p = c.proc
            if p is not None and p.poll() is not None and not getattr(p, 'reported', False):
                p.reported = True
                rc = p.returncode
                ev(kind='exit', c=c.idx, v=c.version, rc=rc, sig=(-rc if rc < 0 else None), n=c.launches,
                   expected=getattr(p, 'expected', False), up=round(time.time() - c.started, 1))
                if not getattr(p, 'expected', False):
                    time.sleep(2)
                    if not stop.is_set():
                        c.launch()   # unexpected exit: note it and bring the client back
        time.sleep(0.5)

def churn():
    if args.churn <= 0:
        return
    while not stop.is_set():
        time.sleep(random.expovariate(args.churn * len(clients) / 60.0))
        c = random.choice(clients)
        p = c.proc
        if p is None or p.poll() is not None:
            continue
        how = random.choice(['term', 'kill', 'kill', 'quick'])
        p.expected = True
        p.send_signal(signal.SIGTERM if how == 'term' else signal.SIGKILL)
        ev(kind='churn', c=c.idx, how=how)
        try:
            p.wait(10)
        except subprocess.TimeoutExpired:
            ev(kind='term_hang', c=c.idx); p.kill(); p.wait()
        if not getattr(p, 'reported', False):
            p.reported = True
            ev(kind='exit', c=c.idx, v=c.version, rc=p.returncode, sig=(-p.returncode if p.returncode < 0 else None),
               n=c.launches, expected=True, up=round(time.time() - c.started, 1))
        time.sleep(0.05 if how == 'quick' else random.uniform(0.5, 3))
        if stop.is_set():
            break   # the run is ending: a relaunch now would miss the final SIGTERM
        c.launch()

samples = open(os.path.join(args.out, 'samples.tsv'), 'w')
samples.write('t\tc\tversion\tpid\tcpu_s\trss_kb\tthreads\tfds\tports\n')
def sampler():
    while not stop.is_set():
        t = round(time.time() - T0, 1)
        for c in clients:
            p = c.proc
            if p is None or p.poll() is not None:
                continue
            s = proc_stats(p.pid)
            if s:
                ports = ','.join(f'{a}>{b}' for a, b in socket_ports(p.pid))
                samples.write(f'{t}\t{c.idx}\t{c.version}\t{p.pid}\t{s[0]:.2f}\t{s[1]}\t{s[2]}\t{s[3]}\t{ports}\n')
        samples.flush()
        time.sleep(2)

ths = [threading.Thread(target=f, daemon=True) for f in (reaper, churn, sampler)]
if not args.no_input:
    ths += [threading.Thread(target=c.drive, daemon=True) for c in clients]
for t in ths:
    t.start()
try:
    while time.time() - T0 < args.duration:
        time.sleep(1)
finally:
    stop.set()
    time.sleep(1)
    for c in clients:
        if c.proc and c.proc.poll() is None:
            c.proc.expected = True
            c.proc.terminate()
    time.sleep(3)
    for c in clients:
        if c.proc and c.proc.poll() is None:
            ev(kind='term_hang', c=c.idx); c.proc.kill()
        ev(kind='final', c=c.idx, v=c.version, rc=c.proc.returncode if c.proc else None, launches=c.launches)
        c.xvfb.terminate()
    ev(kind='done')
