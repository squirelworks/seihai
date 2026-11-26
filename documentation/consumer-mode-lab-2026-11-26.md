# Consumer mode against seihai-gserver-v2 (lab, 2026-11-26)

The start window's consumer mode was checked with `consumer_probe`
(`crates/seihai-play/examples/consumer_probe.rs`). The probe takes the same
steps as the window, `consumer::first_version` then `consumer::after_refusal`,
with no window, and takes the first option when the window would ask.

Setup:

- The lab's Debian VM, everything on 127.0.0.1.
- seihai-gserver-v2 main `9c7e8ec` in `localhost` mode, so there was no login
  verification.
- seihai `72a2907` with seihai-core main `18f9b62`.
- One server per case, each with its own `allowedversions.txt`.
- In every case, the server log matched the probe. Each refused token was
  logged as "Client version not allowed", and each joined token was logged as
  accepted.
- No crashes or hangs. No login was accepted without the probe getting a
  level.

| Case | `allowedversions.txt` | Server printed | Probe | Result |
|---|---|---|---|---|
| A classic | `classic = GNW13110` | `1.41r1` | 6.03r7 refused, picks 1.41 | joined 1.41 |
| B newmain | `newmain = GNW24011:GNW28015, G3D16053:G3D28015` | `2.0 - 2.31, 3.0 - 3.153` | 6.03r7 refused, picks 2.31 | joined 2.31 |
| C modern | `modern = G3D14097:GNP1905C` | (no refusal) | 6.03r7 | joined 6.03r7 |
| D, like Graal the Adventure | `GNW03014`, `G3D22067` | `2.22, 5.007` | 6.03r7 refused, picks 2.22 | joined 2.22 |
| E, like S2 | `GNW03014`, `GNW13110:G3D2504D` | (no refusal) | 6.03r7 | joined 6.03r7 |
| F | `GNW13110`, `G3D14097:G3D0311C` | (no refusal) | 6.03r7 is in the range | joined 6.03r7 |
| G, newmain as 3D (`--3d`) | as B | `2.0 - 2.31, 3.0 - 3.153` | 5.12 refused, picks 3.04 | joined 3.04 |
| H1 | `GNW13110`, `G3D14097:G3D15010` | `1.41r1, 5.12 - 5.24` | 6.03r7 refused, asks Classic 1.41 / Modern 5.24r4 | joined 1.41 (first option) |
| H2 | `GNW13110`, `G3D14097:G3D2505C` | `1.41r1, 5.12 - 6.034/6.035` | 6.03r7 refused, asks Modern 5.31r4 / Classic 1.41 | joined 5.31r4 (first option) |

The order of the lines in the file did not change H1 or H2.

## Notes

- **F does not ask.** The first try, 6.03r7, is inside the modern range, so
  the server never refuses and never prints its list. A server that takes two
  eras is only a question when Seihai's first try is refused.
- **H1 recommends Classic.** The Modern range ends at 5.24r4, and the
  options rank like this:
  - 5.24r4 is only in seihai-gs2lib's table, so `version_pick` marks it
    `Unsure`. A stock server would refuse it.
  - 5.12 is a G3D build, so it ranks below the 2D builds on a 2D server.
  - 1.41 is sure and 2D, so it ranks first.

  This is the ranking in seihai-core `documentation/auto-version.md`, and it
  is kept. On a seihai-gserver-v2 server, 5.24r4 would have worked, and the
  player can still pick it on the card.
- **Separate finding (not consumer mode):** every 5.31r4 and 6.03r7 join
  (cases C, E, F and H2) logged
  `not sent: malformed message (no closing line feed): id 158 len 13 -gr_movement`
  once. Core's wire check refused to send a message with id 158 that names
  GServer-v2's `-gr_movement` weapon. It did not stop any join. It is
  tracked on its own.

The full logs and probe output are kept with the lab files, not in the repo.
