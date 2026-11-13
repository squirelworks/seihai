# Lab #458: Ato 5.12 Graal3D link against seihai-gserver-v2 (2026-11-13)

Mirae's Linux RE Lab VM, headless. Everything ran inside a private network namespace with loopback only (`unshare -r -n`); nothing reached the network.

## Builds

| Part | Version |
|---|---|
| `seihai-ato` | seihai main `9d4e2c4` (seihai-core / seihai-draw tag v0.1.3), `cargo build --release -p seihai-ato`; sha256 `df91e48c…2047ace` |
| gserver | seihai-gserver-v2 `101a2cce` (`gs2emu`, sha256 `82eb6e88…5f1288`). Main is now `039e501a` (#63); its diff only touches RC text and skips `PLTYPE_AWAIT` sockets in `sendPacketToAll`. It does not touch Graal3D, so I did not rebuild. |
| listserver | seihai-listserver `ae213fc`; it ran but this test does not use it (direct `--online`) |

The gserver world is `lab3d`: `graal3d = true`, TCP 127.0.0.1:14830, UDP 14831. `config/graal3dworld.txt` holds Ground, Cube, Sun, a six-texture Sky, a Camera and the `-Move3D` weapon3d.

## Run

```
seihai-ato --version 5.12 --online 127.0.0.1:14830 --no-window --account labato --password-env GRAAL_PW --graal3d
tcpdump -i lo -w graal3d-udp-14831.pcap udp port 14831
```

Run 1 lasted 240 s and was then stopped with SIGINT. The capture holds 834 packets over 239.8 s.

## Result

| # | Expected | Seen (run 1, seconds from start) | Verdict |
|---|---|---|---|
| 1 | `3d: Connection established` | `4.256 3d: Connection established (19 classes)` | pass |
| 2 | both mission phases | `4.350 3d: MissionStartPhase1`, `4.654 3d: MissionStartPhase2` | pass |
| 3 | `Mission lighting done` | `4.654 3d: Mission lighting done` | pass |
| 4 | the ghost lines | `3d: ghost 0 GameObject Ground` … `3d: ghost 5 Camera` (4.350–4.575) | all six present, but they print between Phase1 and Phase2, so before line 3, not after it |
| 5 | `3d: control object is ghost N GameObject G3DPlayer_<account>` | `4.708 3d: control object is ghost 4 GameObject G3DPlayer_labato` | pass |

- **Timeouts over 60 s:** none. The link stayed up for the whole 240 s, and the gserver logged the close only at SIGINT.
- **`3d: login step 6`:** never printed. Every shape was answered (`got ground.3ds`, `got labcube.3ds`), but no Sky texture was.

### Why login step 6 is missing

- The gserver answered `sky_fr`, `sky_rt`, `sky_bk` and `sky_lf` at once, each with a `PLO_FILE` named without an extension and holding PNG data (`gserver-networkdump-run1.txt`). `sky_up` and `sky_dn` were never asked for on the wire.
- Ato printed nothing after 4.708 s. That includes core's own `requesting` and level lines, which come from the TCP side.
- In a second, 50 s run (`run2-*`) the stall came at the same point. A gdb dump at 25 s (`run2-gdb-threads.txt`, stripped binary) shows the `seihai-net` thread and one more thread parked in a futex syscall. Only the UDP thread kept ticking in `clock_nanosleep`.
- So the TCP network thread seems to block, most likely on a lock, as the first Sky `PLO_FILE` replies come in. core's `FileAskState::settle` never runs for them, and login step 6 waits forever.
- A build with symbols would pin down where it blocks.

## Files

- `ato-stdout.txt`, `ato-stdout-timed.txt` (the same lines with monotonic seconds), `ato-stderr.txt`
- `gserver-console.txt`: the gserver's log of the link, from login to disconnect
- `gserver-networkdump-run1.txt`: the gserver's packet dump from login to the last file reply. The password is masked by the gserver.
- `graal3d-udp-14831.pcap`, `tcpdump.txt`: the UDP capture
- `run2-ato-stdout-timed.txt`, `run2-gdb-threads.txt`: the diagnostic rerun
