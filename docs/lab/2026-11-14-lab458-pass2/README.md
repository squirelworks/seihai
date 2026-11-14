# Lab #458 pass 2: Ato 5.12 walks and chats over Graal3D (2026-11-14)

Pass 2 of [#458](https://github.com/squirelworks/seihai/issues/458) checks that seihai-gserver-v2 sees Ato's player move and talk over the Graal3D link. Pass 1 is in [run 3](../2026-11-13-lab458-graal3d-run3/README.md).

The lab ran on GwenTea2. The gserver and Ato both ran inside a loopback-only network namespace (`ns.sh`: `bwrap --unshare-net` holder, `lo` only), and Ato's window was on Xvfb `:71`. Nothing touched the host desktop or any outside network.

| Part | Version |
|---|---|
| `seihai-ato` | seihai `main` at `4462cf5` (#517), release build; sha256 `f0a9ab56…2f27f52` |
| gserver | seihai-gserver-v2 `seihai` at `c851b4e` (#80) **plus a lab-only trace** (`gserver-lab-trace.patch`), built on GwenTea2; sha256 `38527d52…6676da`. World `lab3d`, `graal3d = true`, `localhost = true`, TCP 127.0.0.1:14830, UDP 14831 |

The trace is needed because the stock gserver applies a client's object update without logging it. The patch prints one `G3DTRACE` line for every client section it takes from the control object when `SEIHAI_G3D_TRACE` is set:
- `s5` is the transform.
- `s2` is the velocity.
- `s9` is the chat.

The patch is only for this lab and is not meant for upstream.

## What ran

`lab2-ato.sh r4` runs these steps:
1. It starts the gserver and tcpdump on UDP 14831.
2. It launches `seihai-ato --version 5.12 --online 127.0.0.1:14830 --account labato --password-env GRAAL_PW --graal3d` with its window on Xvfb.
3. Once `login step 6` prints, `xt.py` drives the keys through XTest. GwenTea2 has no `xdotool`, and XTest sends the same synthetic X events. The sequence is:
   - hold Up 10 s
   - hold Left 1.5 s
   - hold Up 2 s
   - hold Right 3 s
   - Tab, type `hi e:smile`, Return

   The exact timings are in `events.txt`, and the screenshots are `0-start.png` to `3-said.png`.

Login reached step 6 at 0.23 s (`3d: Connection established` at 0.068 s). The link stayed up until the gserver shut down.

## Result: pass

**Transform (section 5) changes while walking.**
- The gserver logged 476 `s5` lines (`gserver-log.txt`).
- The position went from (0, 0, 1) to (0, 60.094, 1) on the 10 s walk.
- The angle went to +3.779 on the left turn and back down to −3.749 on the right turn.
- The final position was (7.168, 50.416, 1).
- The position and transform in each packet agree.
- The client sent an update about every 36 ms (median).

**Chat (section 9) arrives.**
- Gserver log: `G3DTRACE t=834041 pid=3 mask=0x1000 pos=(7.168,50.416,1.000) s9 chat="hi e:smile"`
- Ato stdout at the same moment: `3d: say hi e:smile`
- `3-said.png` shows the bubble over the player.

**The UDP capture agrees.**
- `graal3d-udp-14831.pcap` holds 1643 packets.
- `g3d_pcap.rs` decodes the client's sections (`pcap-client-sections.txt`): 476 transform sections, 1 chat section, and **0 velocity sections**.

### Ato's numbers

| | Moved | Key held | Rate | Ato constant |
|---|---|---|---|---|
| Walk | 60.094 units (first leg), 12.043 (second) | 10 s, 2 s | 6.0 units/s | `WALK_SPEED = 6.0` |
| Turn left | +3.779 rad | 1.5 s | 2.5 rad/s | `TURN_SPEED = 2.5` |
| Turn right | −7.528 rad | 3 s | 2.5 rad/s | `TURN_SPEED = 2.5` |

Measured on the gserver's clock, each leg's updates are spread over more time than the key was held:
- The 10 s walk arrived over 12.9 s, about 4.65 units/s.
- The 1.5 s turn arrived over 2.56 s, and the 3 s turn over 3.50 s.

The totals still match the hold times exactly. So Ato under Xvfb with software GL sends its simulated motion more slowly than real time, without losing any of it. Compare speeds by distance per key-hold time, not by the packet clock.

Ato sends **no velocity section**, only transforms.

## Real 5.12 client: skipped

Step 6 of the plan was to record a real 5.12's velocity and its walking and turning speeds, so the placeholder constants above can be replaced. I skipped it. Memory wasn't the limit (over 7 GB free). These were:
- **No 5.12 in the Win10 VM.** 5.12 isn't installed in GwenTea2's Win10 VM.
- **No UDP path from the VM.** The VM's lab network is a QEMU restricted NIC with `guestfwd`, which forwards TCP only. The Torque link on UDP 14831 has no route to the gserver.
- **Walking never worked on a real 5.12.** The gserver's `CHANGES-SEIHAI.md` (2026-11-12) says a real 5.12 drew the sky and its walking script compiled, "but the lab client's arrow keys have not reached it yet". That is an open problem, not something a lab run can settle.
- **Wine doesn't help.** Under Wine on GwenTea2 the 5.12 3D window stays black (`lab-gserver-2026-11-12/FINAL3.md`).
- **The proxy is also needed.** The client would need the gen-5 round-down translating proxy in front of gserver-v2 (`documentation/v5v6-profiles.md`).

So whether a real 5.12 sends velocity, and its real speeds, are still open. The best route is probably a static read of the 5.12 `Graal3DEngine.dll` player move code, or a real 5.12 on Windows hardware once its keys reach the walking script.

## Files

- `ato-stdout.txt`, `ato-stdout-timed.txt` (epoch-stamped), `ato-stderr.txt` (empty)
- `gserver-log.txt`: the gserver's `serverlog.txt` for the run, with the `G3DTRACE` lines
- `graal3d-udp-14831.pcap`, `tcpdump.txt`
- `pcap-client-sections.txt`: client sections decoded from the pcap by `g3d_pcap.rs` (a seihai-ato example, kept here as a lab tool)
- `events.txt`, `0-start.png` … `3-said.png`: input timeline and screenshots
- `gserver-lab-trace.patch`: the gserver trace patch
- `lab2-ato.sh`, `xt.py`, `ns.sh`: the harness

## Build notes

- The gserver build failed at first because the generated GS1 parser comes from the ANTLR 4.13.2 jar, while GwenTea2's system runtime is 4.10. Configuring with `-DANTLR4_JAR_LOCATION=<antlr-4.10.1-complete.jar>` fixes it.
- An earlier attempt at this pass crashed the machine during the gserver build. This run built with `ninja -j3` and a watchdog that would pause the build below 2.5 GB free. Free memory never went below 5.7 GB.
