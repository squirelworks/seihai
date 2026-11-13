# Lab #458 run 3: Ato 5.12 Graal3D link with seihai-core v0.1.5 (2026-11-13)

This rerun of [the first run](../2026-11-13-lab458-graal3d/README.md) uses the core fix that accepts Sky files the server names with no extension. The setup is the same: Mirae's Linux RE Lab VM, headless, in a private loopback-only network namespace. Nothing else ran in that namespace.

| Part | Version |
|---|---|
| `seihai-ato` | seihai branch `claude/project-thread-fua6dt` (PR #498) at `2eba2f9`, seihai-core / seihai-draw v0.1.5; sha256 `a7276d27…718753c` |
| gserver | seihai-gserver-v2 `101a2cce` (`gs2emu`), world `lab3d`, `graal3d = true`, TCP 127.0.0.1:14830, UDP 14831 |

Command: `seihai-ato --version 5.12 --online 127.0.0.1:14830 --no-window --account labato --password-env GRAAL_PW --graal3d`, run for 240 s and then stopped with SIGINT. tcpdump captured 847 packets on UDP 14831.

My first launch reached the gserver before it had finished loading and got "connection refused". I discarded that launch. The run here is the second launch, made once UDP 14831 was listening.

## Result

**`3d: login step 6` prints:**
`1.423 3d: login step 6: every file request answered (files: 8, not on the server: 0)`

| Line | Seen (seconds from start) |
|---|---|
| Connection established | `0.652 3d: Connection established (19 classes)` |
| Phase 1 | `0.702 3d: MissionStartPhase1` |
| Ghosts | `0.703 ghost 0 GameObject Ground`, `0.709 ghost 1 GameObject Cube`, `0.709 ghost 2 Sun`, `0.883 ghost 3 Sky`, `0.883 ghost 4 GameObject G3DPlayer_labato`, `1.320 ghost 5 Camera` |
| Phase 2 | `1.374 3d: MissionStartPhase2` |
| Lighting | `1.374 3d: Mission lighting done` |
| Control object | `1.423 3d: control object is ghost 4 GameObject G3DPlayer_labato` |
| Login step 6 | `1.423 3d: login step 6: every file request answered (files: 8, not on the server: 0)` |

As in run 1, the ghost lines print between Phase 1 and Phase 2.

**Sky files:**
- **Requested (0.883):** sky_fr, sky_rt, sky_bk, sky_lf, sky_up, sky_dn.
- **Received:**
  - 1.320: `got sky_fr (180 bytes)`, `sky_rt (180)`, `sky_bk (180)`, `sky_lf (180)`
  - 1.352: `sky_up (151)`
  - 1.369: `sky_dn (177)`
- Then:
  - 1.369 `3d: the Sky's 6 textures are in`
  - 1.423 `3d: Sky read again with its textures`
- The shapes came in at 0.889: `got ground.3ds (432 bytes)`, `got labcube.3ds (426 bytes)`.
- The gserver's dump shows `PLI_WANTFILE` for all 8 names, each answered with `PLO_FILE`. There was no `FILESENDFAILED`.

**Timeouts, errors, and what comes after step 6:**
- No timeouts and no errors. stderr is empty apart from the runner's own marker line.
- Step 6 is the last `3d:` line. Nothing more printed for the rest of the 240 s, and the link stayed up until SIGINT.
- The gserver logged the close (`link from 127.0.0.1 closed`) only at the stop.

## Files

- `ato-stdout.txt` and `ato-stdout-timed.txt`: stdout, plain and with monotonic seconds
- `ato-stderr.txt`
- `gserver-console.txt`: the gserver's log of the link
- `gserver-networkdump.txt`: from server start to the last file reply. The gserver masks the password.
- `graal3d-udp-14831.pcap` and `tcpdump.txt`: the UDP capture
