# Differential testing lab

Runs the same scripted session against the real Wine client and against Seihai,
captures both on a private loopback network, and diffs the wire frames. The
tools it uses are in the `seihai` crate (`difftest` binary, `seihai::difftest`
module); the design and the first results are in
`documentation/difftest.md`.

```text
tools/difftest/
  lab.sh               the scenario runner (up / run / pair / diff / all / down)
  scenarios/*.steps    one file per scenario, shared by both clients
```

## What it needs (all outside the repo)

`$DIFFTEST_ROOT` (mode 700) holds the lab's copies:

| path | what |
|---|---|
| `hosts` | one line mapping the listserver names to 127.0.0.1 |
| `server/` | a GServer-v2 tree (`gs2emu`, `servers/lab/...`) with the lab levels and an account template `labb31.txt` |
| `ls/` | the graal-serverlist `listserver` and its `settings.ini` |
| `pristine217/`, `pristine222/`, `pristine231/` | clean client folders (no level cache) |
| `wp/` | a Wine prefix **copy** used only by this lab |
| `assets/` | a Seihai asset folder (`pics1.png`, `sprites.png`, `bodies/`); its `levels/` is dropped per run |
| `cap/` | pcaps, mode 700, never committed (they hold the placeholder login) |
| `out/` | traces and diffs |

Every run starts from fresh copies of `server/`, `ls/` and the client folder, so
no run sees another's state, and the account's start level and position come
from the scenario.

## Safety rules the script enforces

* The namespace must be named `graallabd*`. The shared lab namespaces
  (`graallab`, `graallabb`, `graallabc`, `graallabg`) are refused.
* The displays default to `:31` (real client) and `:32` (Seihai). `:3` is
  refused, and so is a display that is already in use.
* `/etc/hosts` is never edited. The lab's hosts file is bind-mounted over it
  inside a private mount namespace of the lab processes.
* `down` kills only processes inside the private namespace, only the Xvfb
  processes the script started, and runs `wineserver -k` only on the lab's own
  prefix copy. It never calls any other lab's stop script.
* Only loopback sockets inside the namespace are used. No public server.
* Accounts and the password are placeholders (`dtacct`, `labpass-placeholder`).

## Use

```sh
export DIFFTEST_ROOT=/path/to/private/lab
cargo build --release
tools/difftest/lab.sh pair 2.31 gmap_cross            # real, Seihai, diff
tools/difftest/lab.sh run 2.22 chat seihai-parity     # one capture
tools/difftest/lab.sh all "2.22 2.31" "login gmap_cross chat"
tools/difftest/lab.sh retrace                         # re-decode every pcap and re-diff (no lab needed)
tools/difftest/lab.sh down
```

Do not edit `lab.sh` while a run is going: bash reads a script as it executes it.

`seihai` runs Seihai with its default options; `seihai-parity` adds the opt-in
senders (`--send-tampercheck --send-showimg`) so the diff shows what is left
once those are on.

The two steps can be run by hand on any pcap:

```sh
target/release/difftest trace cap.pcap --version 2.31 --scenario gmap_cross --client real --out real.tsv
target/release/difftest trace cap2.pcap --version 2.31 --scenario gmap_cross --client seihai --out sei.tsv
target/release/difftest diff real.tsv sei.tsv --out report      # report.txt + report.tsv
```

## Scenario steps

```text
start <level> <x> <y>   where the account starts
wait <seconds>
hold <Key> <seconds>    an arrow key held down
chat <text>             local chat (a chat-bubble prop inside PLI_PLAYERPROPS)
mark <name>             a label in the run's marks file
click <x> <y>           real client only
```

Walking is timed by the clock: the real client and Seihai walk at their own
speeds, so a scenario holds a key long enough to cross, not for an exact
distance.

## Not covered yet

* `serverwarp.steps` needs an injection forwarder (the real client cannot ask
  for a server warp by itself; the lab server only sends 178 to a client that
  sent PLI 41). It is defined, not run by `all`.
* The real client's own UI automation uses fixed screen coordinates for the
  2.22 and 2.31 login screens; 2.17 may need its own.

## Known limits of the harness

* The UI driver uses the 2.22/2.31 launcher's fixed 1024x768 coordinates. The
  2.17 client has a different launcher (Training / Start / Options), so the
  real 2.17 capture is empty. The 2.17 Seihai side runs.
* `serverwarp.steps` needs a forwarder that injects a warp from the server
  side; it is written but not run by `all`.
* The scenarios walk by holding keys for a fixed time, so the real client
  (faster) and Seihai (slower) end up at different positions. See
  `documentation/difftest.md`, "Harness caveats".
