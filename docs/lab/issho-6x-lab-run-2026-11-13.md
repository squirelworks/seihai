# Issho 6.x lab runs, 2026-11-13 (#441, #444, #445)

Two runs of `seihai-issho` against a private gserver-v2 world on loopback, one on Windows and one on Linux.
They follow `issho-6x-lab-run-2026-11-12.md` with the hits, death, respawn and HUD work since (#491, #493,
#495) and check the two fixes the Windows run led to (#496). No public servers.

## Lab server

The "Issho 6x Lab" world from the 11-12 run (gserver-v2, generation modern), one copy on each host, listening on
127.0.0.1:14817 and not registered with the listserver.

- `issho_a.nw` links on its right edge to `issho_b.nw` at 2,30.
- `issho_b.nw` links back to A and, on its bottom edge, to `issho_ga.nw` at 30,3.
- `isshomap.gmap` is 2x1: `issho_ga` and `issho_gb`.
- `steps2.wav` and `water.wav` are on the server only.

Each row's actor account is new. The observer is a second Issho (account `isshoobs`) that attacks the actor
with the sword.

## Windows run

- **Where:** a Win10 Pro 19045 lab VM, 4 vCPUs, Microsoft Basic Render Driver (no GPU). Its only network is a
  restricted user-mode NIC that forwards one address to the host's lab server.
- **Build:** seihai `main` at 9d4e2c4 (seihai-core v0.1.3), `cargo build --release -p seihai-issho`,
  Rust 1.93.1 `x86_64-pc-windows-gnu`.
- **Method:** the actor is Windows Issho in the VM, with keys sent by the VM monitor. The observer is Linux Issho
  on the host under Xvfb. The VM's audio output is recorded to a wav on the host.

| Row | Token | Window + 6.x HUD | Login | Walk seen | Link A->B | Link B->gmap | Gmap crossing | WANTFILE / arrives / plays | Chat | Swing seen | Hit taken / HUD hearts drop | Death / respawn |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 6.00r2 | G3D26090 | PASS | PASS | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS / **FAIL** | PASS / PASS |
| 6.015 | G3D3007A | PASS | PASS | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS / **FAIL** | PASS / PASS |
| 6.034 | G3D2505C | PASS | PASS | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS / **FAIL** | not reached |
| 6.03r7 | G3D0311C | PASS | PASS | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS / **FAIL** | not reached |

- The VM has no OpenGL 2.0, so Issho logged that `egui_glow` could not open the window and fell back to wgpu
  (DX12 WARP) by itself. The window is 1024x768 with the 6.x HUD as on Linux.
- On 6.034 and 6.03r7 the observer's script landed only 2 or 3 hits, so the actor never died. That was the
  harness, not the client; the Linux run below covers death on both builds.
- **HUD hearts:** the actor logged each hit and the server saved the lower HP, but the HUD kept three full
  hearts through death. The HUD read only the server's CURPOWER, which gserver-v2 does not echo back to the
  player. Fixed in #496.
- **Walk pace:** under software rendering the actor walked 2 to 4 tiles a second, not 10. The walk took at most
  one 0.5-tile step per frame. Fixed in #496: a slow frame takes the steps it missed.
- No panics in any row.

## Linux run

- **Where:** the Debian lab VM, Xvfb with Mesa llvmpipe. The VM's load was 4 to 24 from other labs that day.
- **Build:** seihai `main` at b31c2d5 (seihai-core v0.1.5, with #496), `cargo build --release -p
  seihai-issho`, rustc 1.99.1, no warnings. For comparison the rows were also run on 9d4e2c4, the build the
  Windows run used.
- **Method:** as on 11-12, except that each walk runs until Issho logs the link or the crossing, because fixed
  key-hold times did not reach the links at that load. The pace is measured over 15 holds of 2.0 s.

| Row | Token | Login + warp | Window + 6.x HUD | Walk seen | Links + gmap crossing | WANTFILE / arrives / plays | Chat | Swing seen | HUD hearts | Death / respawn |
|---|---|---|---|---|---|---|---|---|---|---|
| 6.00r2 | G3D26090 | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS | PASS / PASS |
| 6.015 | G3D3007A | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS | PASS / PASS |
| 6.034 | G3D2505C | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS | PASS / PASS |
| 6.03r7 | G3D0311C | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS | PASS / PASS |
| 6.03r7 Linux | G3D0511C | PASS | PASS | PASS | PASS | PASS / PASS / PASS | PASS | PASS | PASS | PASS / PASS |

- **HUD hearts:** three, then two and a half after the first hit, none when dead, and three again after the
  respawn. On 9d4e2c4 they stayed at three throughout, as on Windows.
- **Walk pace:** 9.25 to 10.25 tiles a second, mean about 9.9, against about 5 on 9d4e2c4 at the same load.
- **Links:** A->B, B->gmap and the gmap crossing both ways, seen by the server and the observer.
- **Sound:** `steps2.wav` is asked for, arrives and plays: 120 to 130 loud quarter-seconds a row.
- **Death:** six sword hits kill the actor, the observer sees the dead pose, and an arrow key respawns it at
  `issho_a` 30.5,30.
- No panics in any row.

## Open

- **Counters after a death:** the bombs, and sometimes the arrows, read 0 after the respawn in every row on both
  hosts; the server saves BOMBS 0. To be checked against a real 6.x client before it is called a bug.
- **Plain-zlib frame:** once, on 9d4e2c4 on Linux, the connection closed right after login with `unknown gen-5
  compression byte 0x78`: core's game-server read refuses a plain-zlib frame. Not reproduced, no capture.
- **One hit per swing:** a swing arrives as several hit frames and core judges only the first in each second per
  attacker, so a swing whose first frame is out of reach misses entirely. This made hits hard to land in the
  Windows run; not confirmed as a deviation from the real client.
- **Header after a crossing:** the header still shows the link's arrival point after a gmap crossing (as on 11-12).
- **Vendored builds:** a `cargo vendor` build of seihai-core fails, because it includes `../../../../assets` and
  `../../../../documentation`, which exist only in the repository layout. Git and path builds are unaffected.
- **Lab note:** deleting an account file before login makes gserver-v2 answer "No level available"; the Linux
  run wrote fresh account files instead.
