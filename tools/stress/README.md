# Stress lab

Stress tests Seihai against a private GServer-v2 on loopback. It never
contacts a public server. Every client connects to `127.0.0.1`, and
`stress.py` asserts this. The results of the first run are in
`documentation/stress-test-2026-10-31.md`.

## Parts

| file | what it does |
|---|---|
| `genworld.py` | Writes the test world into a gserver's `world/` and the accounts `stress00`…`stress63`. The world has an 8-level warp ring with script warps and links, a 2x2 gmap with baddies, an arena (12 baddies, chests, NPCs that lay items, put bombs, shoot arrows and send chat every 0.05–0.5 s) and a hub. It also writes 200 linked levels `st_w0`…`st_w199` for long sessions. The account number picks the role: `n % 4` = ring, gmap, arena, hub. |
| `stress.py` | Runs N clients per version. Each client gets its own Xvfb display and HOME, so each has its own focus and settings. It drives each client with xdotool by role: chat `warpto`, walking, sword, action, chat spam. It also churns reconnects (SIGTERM, SIGKILL, quick relaunch), samples RSS, CPU, threads, fds and the client's TCP port every 2 s, and logs launches and exits to `events.jsonl`. |
| `analyze.py` | Writes `report.md` for a run. It covers unexpected exits, panics, stalls, `warp not answered`, RSS slope per launch, CPU, and the C>S send rate per second. The send rate comes from the run's pcap, split by client port and decoded with `seihai capture show`. |
| `run.sh NAME [stress.py args]` | Runs one stress run end to end. It makes fresh accounts, starts both lab servers, captures a pcap of loopback ports 14801–14802, runs the harness, then stops the servers. `batches.sh` runs the four version batches. |
| `servers.sh up/down` | Starts and stops the game gserver (`run/gs`, port 14802) and a Login lobby (`run/login`, 14801). The lobby sends 4.x+ clients on with `PLO_SERVERWARP`. |
| `faultproxy/` | A proxy between Seihai and the gserver. It decodes server frames with Seihai's own codecs, changes them, and re-encodes them with a fresh cipher. Modes: `mutate` (replays recent server messages with bytes flipped, cut or grown), `inject` and `oversize` (hostile messages such as huge boards, lying raw-data lengths, long strings and out-of-range warps), `bomb` (zlib/bz2 bodies of 8–256 MB), `framing` (zero-length, junk and cut frames, bad compression byte, 2 GB long length), `flood` (one message repeated thousands of times). `fuzzcore` is a second binary that feeds mutated input in-process to `SessionState`, every message id, the protocol parsers, level, gani, PNG/GIF, GS1 and GS2 readers. |
| `fz.py`, `fzservers.sh` | Runs the fault proxy: one client session per version, mode and seed. A second gserver pair (14803, and a lobby on 14804 that points at the proxy on 15803) keeps it apart from `run.sh`. |
| `linktest.sh` | Logs one client in and walks it east into the hub's edge link. Set `DEBUG=1` to get `_debug/*/packets.log`. |
| `gserver-lab-patch/` | The lab-only changes to GServer-v2 `efb5b90` and its gs2lib (`b839106`). With `GS_LAB_NOLIST=1` the server accepts any account with no listserver. `GS_LAB_SERVERWARP` makes it act as the Login lobby. 2.05 (`GNW03081`) and 5.24r4 (`G3D15010`) are mapped to known protocol IDs, and an unknown token no longer aborts the server. Do not use these changes outside the lab. |

## Setup (Linux)

```sh
export SEIHAI_LAB=~/seihai-lab        # private, outside the repo
apt install xvfb xdotool tcpdump imagemagick flex bison cmake ninja-build libbz2-dev zlib1g-dev
git clone https://github.com/xtjoeytx/GServer-v2 $SEIHAI_LAB/gs-main
cd $SEIHAI_LAB/gs-main && git checkout efb5b90
git submodule update --init dependencies/gs2lib dependencies/gs2compiler
(cd dependencies/gs2compiler && git submodule update --init --recursive)
git apply --ignore-whitespace <repo>/tools/stress/gserver-lab-patch/gserver-v2-lab-login.patch
(cd dependencies/gs2lib && git apply --ignore-whitespace <repo>/tools/stress/gserver-lab-patch/gs2lib-lab-tokens.patch)
cmake -GNinja -S. -Bbuild -DCMAKE_BUILD_TYPE=RelWithDebInfo -DTESTS=OFF -DUPNP=OFF -DV8NPCSERVER=OFF && cmake --build build
mkdir -p $SEIHAI_LAB/run && cp -r bin $SEIHAI_LAB/run/gs
```

In `run/gs/servers/default/config/serveroptions.txt` set `listip = 127.0.0.1`,
`listport = 1`, `serverinterface = 127.0.0.1`, `warptoforall = true`,
`disconnectifnotmoved = false` and `gmaps = stgm.gmap`. Put the tokens of the
builds under test in `allowedversions.txt`. Copy `run/gs` to `run/login` and
set its `serverport` to 14801 and its `name` to `Login`. For `fz.py`, make two
more copies, `run/fz` (14803) and `run/fzlogin` (14804).

Run the scripts from the repo. They find the Seihai binary at
`target/release/seihai` (`stress.py --bin` and `fz.py --bin` take another
one) and keep everything else under `$SEIHAI_LAB`:

```sh
cargo build --release
cargo build --release --manifest-path tools/stress/faultproxy/Cargo.toml
tools/stress/run.sh smoke --versions 2.22,6.03r7 --per-version 4 --duration 120 --churn 0.5
python3 tools/stress/analyze.py $SEIHAI_LAB/out/smoke
tools/stress/fzservers.sh up
python3 tools/stress/fz.py $SEIHAI_LAB/out/fz 2.22,6.03r7 inject,bomb,mutate --dur 30 --p 0.3 --after 3
```

Accounts and the password are placeholders. Pcaps and `out/` stay in the lab
folder (mode 700). Never commit them.
